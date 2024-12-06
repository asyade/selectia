use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
    sync::atomic::AtomicU32,
};

use crate::prelude::*;

use backend::{Backend, BackendHandle, BackendMessage};
use cpal::traits::{DeviceTrait, HostTrait};
use dasp::sample;
pub use deck::*;
use eyre::OptionExt;
use futures::executor::block_on;
mod backend;
mod deck;

pub type AudioPlayerService = AddressableServiceWithDispatcher<AudioPlayerTask, AudioPlayerEvent>;

#[derive(Clone, Debug)]
pub enum AudioPlayerTask {
    CreateDeck {
        callback: TaskCallback<u32>,
    },
    GetDecks {
        callback: TaskCallback<BTreeMap<u32, DeckSnapshot>>,
    },
    LoadTrack {
        deck_id: u32,
        metadata_id: i64,
    },
    SetDeckFileStus {
        deck_id: u32,
        status: DeckFileStatus,
        callback: TaskCallback<()>,
    },
}

#[derive(Clone, Debug)]
pub enum AudioPlayerEvent {
    DeckCreated {
        id: u32,
    },
    DeckFileMetadataUpdated {
        id: u32,
        metadata: DeckFileMetadataSnapshot,
    },
    DeckFilePayloadUpdated {
        id: u32,
        payload: DeckFilePayloadSnapshot,
    },
    DeckFileStatusUpdated {
        id: u32,
        status: DeckFileStatus,
    },
}

pub struct AudioPlayer {
    database: Database,
    decks: DeckMixer,
    dispatcher: EventDispatcher<AudioPlayerEvent>,
    backend: Arc<RwLock<Option<BackendHandle>>>,
}

#[derive(Clone)]
pub struct DeckMixer {
    next_deck_id: Arc<AtomicU32>,
    decks: Arc<RwLock<BTreeMap<u32, PlayerDeck>>>,
}

// TODO: put buffer in his own mutex so we can edit source list without blocking the backend thread
pub struct BufferedSamplesSource<T> {
    buffer: Vec<T>,
    provider: SamplesSource,
}

impl<T> BufferedSamplesSource<T> {
    pub fn new(provider: SamplesSource) -> Self {
        Self {
            buffer: Vec::new(),
            provider,
        }
    }

    pub fn fill_buffer(&mut self, buffer_size: usize, sample_rate: u32, channels: usize) -> &[T]
    where
        T: cpal::Sample + cpal::FromSample<f32>,
    {
        self.buffer.resize(buffer_size, T::from_sample(0.0));
        match &mut self.provider {
            SamplesSource::File(file) => {
                let file_lock = file.file.blocking_read();
                let payload = file_lock.payload();

                if let Some(payload) = payload {
                    let mut status = file.state.status.blocking_write();
                    match &mut *status {
                        DeckFileStatus::Playing { offset } => {
                            for (i, sample) in self.buffer.iter_mut().enumerate() {
                                *sample = T::from_sample(
                                    payload.buffer.buffer[((*offset + i as u32)
                                        % payload.buffer.buffer.len() as u32)
                                        as usize],
                                );
                            }
                            *offset =
                                (*offset + self.buffer.len() as u32) % payload.buffer.buffer.len() as u32;
                            file.state
                                .updated
                                .store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                        _ => {
                            self.buffer.fill(T::from_sample(0.0));
                        }
                    }
                } else {
                    self.buffer.fill(T::from_sample(0.0));
                }
            }
        }
        &self.buffer
    }
}

#[derive(PartialEq)]
pub enum SamplesSource {
    File(DeckFile),
}

pub fn audio_player(database: Database) -> AudioPlayerService {
    AddressableServiceWithDispatcher::new(move |mut receiver, _sender, dispatcher| async move {
        AudioPlayer::new(database, dispatcher)
            .await
            .expect("Failed to create audio player")
            .handle(&mut receiver)
            .await?;
        Ok(())
    })
}

impl AudioPlayer {
    pub async fn new(
        database: Database,
        dispatcher: EventDispatcher<AudioPlayerEvent>,
    ) -> Result<Self> {
        let instance = Self {
            backend: Arc::new(RwLock::new(None)),
            database,
            decks: DeckMixer::new(),
            dispatcher,
        };
        instance.init_default_backend().await?;
        Ok(instance)
    }

    async fn init_default_backend(&self) -> Result<()> {
        let (device, config) = tokio::task::spawn_blocking(move || {
            let host = cpal::default_host();
            let device = host.default_output_device().unwrap();
            let config = device.default_output_config().unwrap();
            (device, config)
        })
        .await?;
        let backend =
            Backend::new(self.dispatcher.clone(), self.decks.clone(), device, config).await?;
        self.backend.write().await.replace(backend);
        Ok(())
    }

    pub async fn handle(self, receiver: &mut ServiceReceiver<AudioPlayerTask>) -> Result<()> {
        while let Some(task) = receiver.recv().await {
            if let Err(e) = self.handle_task(task).await {
                error!("Failed to handle task: {e}");
            }
        }
        Ok(())
    }

    #[instrument(skip(self, task))]
    async fn handle_task(&self, task: AudioPlayerTask) -> Result<()> {
        match task {
            AudioPlayerTask::CreateDeck { callback } => {
                let id = self.decks.create_deck(self.dispatcher.clone()).await?;
                let _ = callback.resolve(id).await?;
                let _ = self
                    .dispatcher
                    .dispatch(AudioPlayerEvent::DeckCreated { id })
                    .await?;
            }
            AudioPlayerTask::GetDecks { callback } => {
                let decks = self.decks.get_all_decks().await?;
                let _ = callback.resolve(decks).await?;
            }
            AudioPlayerTask::LoadTrack {
                deck_id,
                metadata_id,
            } => {
                let file = self.database.get_file_from_metadata_id(metadata_id).await?;
                
                let deck: PlayerDeck = self.decks.get_deck(deck_id).await?;
                let (loaded_file, previous) = deck.load_file(file.path).await?;
        

                let backend = self.backend.write().await;
                let backend = backend.as_ref().ok_or_eyre("Backend not loaded")?;
                if let Some(previous) = previous {
                    backend.send(BackendMessage::DeleteSource(SamplesSource::File(previous))).await?;
                }
                backend.send(BackendMessage::CreateSource(SamplesSource::File(loaded_file))).await?;
            }
            AudioPlayerTask::SetDeckFileStus {
                deck_id,
                status,
                callback,
            } => {
                self.decks.set_deck_file_status(deck_id, status).await?;
                let _ = callback.resolve(()).await?;
            }
        }
        Ok(())
    }
}

impl DeckMixer {
    pub fn new() -> Self {
        Self {
            next_deck_id: Arc::new(AtomicU32::new(1)),
            decks: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    async fn create_deck(&self, dispatcher: EventDispatcher<AudioPlayerEvent>) -> Result<u32> {
        let id = self
            .next_deck_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.decks
            .write()
            .await
            .insert(id, PlayerDeck::new(id, dispatcher));
        Ok(id)
    }

    async fn get_deck(&self, id: u32) -> Result<PlayerDeck> {
        self.decks
            .read()
            .await
            .get(&id)
            .cloned()
            .ok_or(eyre!("Deck not found"))
    }

    async fn set_deck_file_status(&self, deck_id: u32, status: DeckFileStatus) -> Result<()> {
        let deck = self.get_deck(deck_id).await?;
        deck.set_status(status).await?;
        Ok(())
    }

    async fn get_all_decks(&self) -> Result<BTreeMap<u32, DeckSnapshot>> {
        let decks = self.decks.read().await.clone();
        let mut result = BTreeMap::new();
        for (id, deck) in decks.iter() {
            let lock: sync::RwLockReadGuard<'_, Option<DeckFile>> = deck.file.read().await;
            let payload = match lock.as_ref() {
                Some(lock) => {
                    let metadata = lock.metadata.clone();
                    let lock = lock.file.read().await;
                    let preview = lock.preview();
                    let payload = lock.payload().unwrap();
                    Some((metadata, DeckFilePayloadSnapshot::new(payload, preview)))
                }
                None => None,
            };

            let (metadata, payload) = payload
                .map(|(metadata, payload)| (Some(metadata), Some(payload)))
                .unwrap_or((None, None));

            let status = {
                let lock = deck.file.read().await;
                if let Some(lock) = lock.as_ref() {
                    Some(lock.state.status.read().await.clone())
                } else {
                    None
                }
            }
            .unwrap_or(DeckFileStatus::Loading { progress: 0 });

            result.insert(
                *id,
                DeckSnapshot {
                    id: *id,
                    metadata,
                    payload,
                    status,
                },
            );
        }

        Ok(result)
    }
}

impl Task for AudioPlayerTask {}

impl Task for AudioPlayerEvent {}
