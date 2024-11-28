#![allow(dead_code)]
use std::any::Any;
use std::collections::BTreeMap;

use crate::prelude::*;
use audio_player::{
    AudioPlayerEvent, AudioPlayerService, DeckMixer, PlayerDeck,
};
use cpal::{traits::*, StreamConfig, SupportedBufferSize};
use cpal::{FromSample, Sample, SizedSample};
use tokio::sync::mpsc;

pub type BackendSender = mpsc::Sender<BackendMessage>;
pub type BackendReceiver = mpsc::Receiver<BackendMessage>;

pub enum BackendMessage {
    Drop,
    Introspect,
}

pub struct BackendHandle {
    stream: (BackendSender, tokio::task::JoinHandle<Result<()>>),
}

pub struct Backend {
    decks: DeckMixer,
    config: cpal::SupportedStreamConfig,
    device: cpal::Device,
}

impl Backend {
    pub fn new(
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
        let stream = backend.run_stream(dispatcher);
        Ok(BackendHandle { stream })
    }

    pub fn run_stream(
        self,
        dispatcher: EventDispatcher<AudioPlayerEvent>,
    ) -> (BackendSender, tokio::task::JoinHandle<Result<()>>) {
        let (sender, _receiver) = mpsc::channel(128);
        let introspect = sender.clone();
        let decks = self.decks.clone();
        let device = self.device.clone();
        let handle = tokio::task::spawn_blocking(move || {
            let config: StreamConfig = self.config.clone().into();
            let _stream = match self.config.sample_format() {
                cpal::SampleFormat::I8 => run::<i8>(decks, &device, &config, introspect),
                cpal::SampleFormat::I16 => run::<i16>(decks, &device, &config, introspect),
                // cpal::SampleFormat::I24 => run::<I24>(device, &config.into()),
                cpal::SampleFormat::I32 => run::<i32>(decks, &device, &config, introspect),
                // cpal::SampleFormat::I48 => run::<I48>(device, &config.into()),
                cpal::SampleFormat::I64 => run::<i64>(decks, &device, &config, introspect),
                cpal::SampleFormat::U8 => run::<u8>(decks, &device, &config, introspect),
                cpal::SampleFormat::U16 => run::<u16>(decks, &device, &config, introspect),
                // cpal::SampleFormat::U24 => run::<U24>(device, &config.into()),
                cpal::SampleFormat::U32 => run::<u32>(decks, &device, &config, introspect),
                // cpal::SampleFormat::U48 => run::<U48>(device, &config.into()),
                cpal::SampleFormat::U64 => run::<u64>(decks, &device, &config, introspect),
                cpal::SampleFormat::F32 => run::<f32>(decks, &device, &config, introspect),
                cpal::SampleFormat::F64 => run::<f64>(decks, &device, &config, introspect),
                sample_format => Err(eyre!("Unsupported sample format: {:?}", sample_format)),
            }?;
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
                            if let Err(e) =
                                dispatcher.dispatch_blocking(AudioPlayerEvent::DeckFileStatusUpdated {
                                    id: *id,
                                    status: file.state.status.blocking_read().clone(),
                                })
                            {
                                error!("Failed to dispatch deck file updated event: {e}");
                            }
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            Ok(())
        });
        (sender, handle)
    }
}

fn run<T>(
    decks: DeckMixer,
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    introspect: BackendSender,
) -> Result<cpal::Stream>
where
    T: SizedSample + FromSample<f32>,
{
    let sample_rate = config.sample_rate.0;
    let channels = config.channels as usize;

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            decks.write_data(data, sample_rate, channels);
            let _ = introspect.blocking_send(BackendMessage::Introspect);
        },
        err_fn,
        None,
    )?;
    stream.play()?;
    Ok(stream)
}
