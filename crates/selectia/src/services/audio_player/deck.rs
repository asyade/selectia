use crate::prelude::*;
use audio_player::AudioPlayerEvent;
use eyre::OptionExt;
use futures::{channel::oneshot, pin_mut};
use selectia_audio_file::{audio_file::AudioFilePayload, AudioFile};
use std::sync::atomic::AtomicU32;

#[derive(Clone)]
pub struct PlayerDeck {
    id: u32,
    pub file: Arc<RwLock<Option<DeckFile>>>,
    dispatcher: EventDispatcher<AudioPlayerEvent>,
}

#[derive(Clone, Debug)]
pub struct DeckSnapshot {
    pub id: u32,
    pub metadata: Option<DeckFileMetadataSnapshot>,
    pub payload: Option<DeckFilePayloadSnapshot>,
    pub status: DeckFileStatus,
}

#[derive(Clone)]
pub struct DeckFile {
    pub file: Arc<RwLock<AudioFile>>,
    pub state: DeckFileState,
    pub metadata: DeckFileMetadataSnapshot,
}

impl PartialEq for DeckFile {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.file, &other.file)
    }
}

#[derive(Clone)]
pub struct DeckFileState {
    /// The status of the current file.
    /// It is updated by the loader thread at the beginning of the file loading process.
    /// Once loaded, the player thread will update the status.
    pub status: Arc<RwLock<DeckFileStatus>>,
    pub updated: Arc<AtomicBool>,
    pub path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct DeckFileMetadataSnapshot {
    pub title: String,
}

#[derive(Clone, Debug)]
pub struct DeckFilePayloadSnapshot {
    pub duration: f64,
    pub sample_rate: f64,
    pub channels_count: usize,
    pub samples_count: usize,
    pub preview: Option<DeckFilePreview>,
}

#[derive(Clone, Debug)]
pub struct DeckFilePreview {
    pub original_sample_rate: f64,
    pub sample_rate: f64,
    pub channels_count: usize,
    pub samples: Vec<f32>,
}

#[derive(Clone, Debug)]
pub enum DeckFileStatus {
    Loading { progress: u32 },
    Playing { offset: u32 },
    Paused { offset: u32 },
}

impl PlayerDeck {
    pub fn new(id: u32, dispatcher: EventDispatcher<AudioPlayerEvent>) -> Self {
        Self {
            id,
            file: Arc::new(RwLock::new(None)),
            dispatcher,
        }
    }

    pub async fn load_file(
        &self,
        path: impl AsRef<Path>,
    ) -> eyre::Result<(DeckFile, Option<DeckFile>)> {
        let file = tokio::fs::File::open(&path).await?;
        let file = Box::new(file.into_std().await);
        let audio_file = AudioFile::from_source(file, &path)?;

        let metadata = DeckFileMetadataSnapshot {
            title: path.as_ref().to_string_lossy().to_string(),
        };

        let state = DeckFileState {
            path: path.as_ref().to_path_buf(),
            status: Arc::new(RwLock::new(DeckFileStatus::Loading { progress: 0 })),
            updated: Arc::new(AtomicBool::new(false)),
        };

        let loaded_file = DeckFile {
            file: Arc::new(RwLock::new(audio_file)),
            state: state.clone(),
            metadata: metadata.clone(),
        };

        let previous = {
            let mut file_lock = self.file.write().await;
            file_lock.replace(loaded_file.clone())
        };

        let _ = self
            .dispatcher
            .dispatch(AudioPlayerEvent::DeckFileMetadataUpdated {
                id: self.id,
                metadata,
            })
            .await;

        self.dispatcher
            .dispatch(AudioPlayerEvent::DeckFileStatusUpdated {
                id: self.id,
                status: DeckFileStatus::Loading { progress: 0 },
            })
            .await?;

        tokio::spawn(self.clone().background_load_file_content());

        Ok((loaded_file, previous))
    }

    pub async fn set_status(&self, status: DeckFileStatus) -> eyre::Result<()> {
        let state = self
            .file
            .read()
            .await
            .as_ref()
            .ok_or_eyre("No file loaded")?
            .state
            .clone();
        state.set_status(status.clone()).await;
        self.dispatcher
            .dispatch(AudioPlayerEvent::DeckFileStatusUpdated {
                id: self.id,
                status,
            })
            .await?;
        Ok(())
    }

    async fn get_file(&self) -> eyre::Result<Arc<RwLock<AudioFile>>> {
        let file = self.file.read().await;
        let file = file.as_ref().ok_or(eyre!("No file loaded"))?;
        Ok(file.file.clone())
    }

    #[instrument]
    async fn background_load_file_content(self) -> eyre::Result<()> {
        let file = self.get_file().await?;

        let snapshot = tokio::task::spawn_blocking(move || {
            info!("Decoding file");
            let mut lock = file.blocking_write();
            let _ = lock.decode_to_end().unwrap();
            let _ = lock.generate_preview();
            let preview = lock.preview();
            let payload = lock.payload().unwrap();
            DeckFilePayloadSnapshot::new(payload, preview)
        })
        .await?;
        let _ = self
            .dispatcher
            .dispatch(AudioPlayerEvent::DeckFilePayloadUpdated {
                id: self.id,
                payload: snapshot,
            })
            .await;
        self.set_status(DeckFileStatus::Paused { offset: 0 })
            .await?;
        Ok(())
    }
}

impl DeckFilePayloadSnapshot {
    pub fn new(payload: &AudioFilePayload, preview: Option<&AudioFilePayload>) -> Self {
        Self {
            duration: payload.duration,
            sample_rate: payload.sample_rate,
            channels_count: payload.channels as usize,
            samples_count: payload.buffer.buffer.len(),
            preview: preview.map(|preview| DeckFilePreview {
                original_sample_rate: payload.sample_rate,
                sample_rate: preview.sample_rate,
                channels_count: preview.channels as usize,
                samples: preview.buffer.buffer.clone(),
            }),
        }
    }
}

impl DeckFileState {
    pub async fn set_status(&self, status: DeckFileStatus) {
        *self.status.write().await = status;
    }
}

impl std::fmt::Debug for PlayerDeck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PlayerDeck {{ id: {} }}", self.id)
    }
}

impl std::fmt::Debug for DeckFileState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DeckFileState {{ path: {:?} }}", self.path)
    }
}
