#![allow(dead_code)]
use crate::prelude::*;
use audio_player::{
    AudioPlayerEvent, AudioPlayerService, BufferedSamplesSource, DeckMixer, PlayerDeck,
    SamplesSource,
};
use cpal::{traits::*, StreamConfig, SupportedBufferSize};
use cpal::{FromSample, Sample, SizedSample};
use dasp::sample::SignedSample;
use dasp::signal;
use dasp::Signal;
use std::any::Any;
use std::collections::BTreeMap;
use tokio::sync::mpsc;

pub type BackendSender = mpsc::Sender<BackendMessage>;
pub type BackendReceiver = mpsc::Receiver<BackendMessage>;

#[derive(PartialEq)]
pub enum BackendMessage {
    CreateSource(SamplesSource),
    DeleteSource(SamplesSource),
}

pub struct BackendHandle {
    stream: (BackendSender, tokio::task::JoinHandle<Result<()>>),
}

impl BackendHandle {
    pub async fn send(&self, message: BackendMessage) -> Result<()> {
        self.stream.0.send(message).await?;
        Ok(())
    }
}

pub struct Backend {
    decks: DeckMixer,
    config: cpal::SupportedStreamConfig,
    device: cpal::Device,
}

impl Backend {
    pub async fn new(
        dispatcher: EventDispatcher<AudioPlayerEvent>,
        decks: DeckMixer,
        device: cpal::Device,
        config: cpal::SupportedStreamConfig,
    ) -> Result<BackendHandle> {
        let backend = Self {
            decks,
            config,
            device,
        };
        let stream = backend.run_stream(dispatcher).await?;
        Ok(BackendHandle { stream })
    }

    async fn run<T>(
        self,
        dispatcher: EventDispatcher<AudioPlayerEvent>,
    ) -> Result<(BackendSender, tokio::task::JoinHandle<Result<()>>)>
    where
        T: SizedSample + FromSample<f32> + Send + 'static + SignedSample,
    {
        let (sender, mut receiver) = mpsc::channel(128);
        let sources = Arc::new(Mutex::new(Vec::new()));

        let decks = self.decks.clone();

        for deck in decks.decks.read().await.values() {
            let file = deck.file.read().await;
            if let Some(file) = file.clone() {
                sources
                    .lock()
                    .await
                    .push(BufferedSamplesSource::new(SamplesSource::File(file)));
            }
        }

        let sources_clone = sources.clone();
        tokio::spawn(async move {
            while let Some(task) = receiver.recv().await {
                match task {
                    BackendMessage::CreateSource(source) => {
                        sources_clone
                            .lock()
                            .await
                            .push(BufferedSamplesSource::new(source));
                    }
                    BackendMessage::DeleteSource(source) => {
                        sources_clone.lock().await.retain(|s| s.provider != source);
                    }
                }
            }
        });

        let device = self.device.clone();
        let handle = tokio::task::spawn_blocking(move || {
            let config: StreamConfig = self.config.clone().into();

            let sample_rate = config.sample_rate.0;
            let channels = config.channels as usize;

            let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

            let stream = device.build_output_stream(
                &config,
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    write_data(&sources, data, sample_rate, channels);
                },
                err_fn,
                None,
            )?;
            stream.play()?;
            loop {
                let deck_lock = self.decks.decks.blocking_read();
                for (id, deck) in deck_lock.iter() {
                    let file = deck.file.blocking_read();
                    if let Some(file) = file.as_ref() {
                        if file
                            .state
                            .updated
                            .fetch_update(
                                std::sync::atomic::Ordering::SeqCst,
                                std::sync::atomic::Ordering::SeqCst,
                                |_| Some(false),
                            )
                            .unwrap()
                        {
                            if let Err(e) = dispatcher.dispatch_blocking(
                                AudioPlayerEvent::DeckFileStatusUpdated {
                                    id: *id,
                                    status: file.state.status.blocking_read().clone(),
                                },
                            ) {
                                error!("Failed to dispatch deck file updated event: {e}");
                            }
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
        });
        Ok((sender, handle))
    }

    pub async fn run_stream(
        self,
        dispatcher: EventDispatcher<AudioPlayerEvent>,
    ) -> Result<(BackendSender, tokio::task::JoinHandle<Result<()>>)> {
        match self.config.sample_format() {
            cpal::SampleFormat::I8 => self.run::<i8>(dispatcher).await,
            cpal::SampleFormat::I16 => self.run::<i16>(dispatcher).await,
            // cpal::SampleFormat::I24 => self.run::<I24>(device, &config.into()).await,
            cpal::SampleFormat::I32 => self.run::<i32>(dispatcher).await,
            // cpal::SampleFormat::I48 => self.run::<I48>(device, &config.into()).await,
            cpal::SampleFormat::I64 => self.run::<i64>(dispatcher).await,
            // cpal::SampleFormat::U8 => self.run::<u8>(dispatcher).await,
            // cpal::SampleFormat::U16 => self.run::<u16>(dispatcher).await,
            // cpal::SampleFormat::U24 => self.run::<U24>(device, &config.into()).await,
            // cpal::SampleFormat::U32 => self.run::<u32>(dispatcher).await,
            // cpal::SampleFormat::U48 => self.run::<U48>(device, &config.into()).await,
            // cpal::SampleFormat::U64 => self.run::<u64>(dispatcher).await,
            cpal::SampleFormat::F32 => self.run::<f32>(dispatcher).await,
            cpal::SampleFormat::F64 => self.run::<f64>(dispatcher).await,
            sample_format => Err(eyre!("Unsupported sample format: {:?}", sample_format)),
        }
    }
}

fn write_data<T>(
    sources: &Arc<Mutex<Vec<BufferedSamplesSource<T>>>>,
    output: &mut [T],
    sample_rate: u32,
    channels: usize,
) where
    T: cpal::Sample + cpal::FromSample<f32> + SignedSample,
{
    let mut sources = sources.blocking_lock();

    for (idx, source) in sources.iter_mut().enumerate() {
        source.fill_buffer(output.len(), sample_rate, channels);
        if idx == 0 {
            output.copy_from_slice(source.buffer.as_slice());
        } else {
            let base_signal = signal::from_iter(output.chunks(2).map(|e| [e[0], e[1]]));
            let source_signal = signal::from_iter(source.buffer.chunks(2).map(|e| [e[0], e[1]]));
            let mixed_signal = base_signal.add_amp(source_signal);

            // TODO: Use ring buffer here
            let collected: Vec<_> = mixed_signal.take(output.len() / 2).flatten().collect();
            output.copy_from_slice(&collected);
        }
    }
}