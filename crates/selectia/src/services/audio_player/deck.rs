use crate::prelude::*;
use audio_player::AudioPlayerEvent;
use eyre::OptionExt;
use futures::{channel::oneshot, pin_mut};
use selectia_audio_file::audio_file::{AudioFilePayload, EncodedAudioFile};
use std::sync::atomic::AtomicU32;

#[derive(Clone)]
pub struct PlayerDeck {
    id: u32,
    last_file_id: Arc<AtomicU32>,
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
    pub id: (u32, u32),
    pub status: Arc<RwLock<DeckFileStatus>>,
    pub updated: Arc<AtomicBool>,
    pub path: PathBuf,
    pub metadata: DeckFileMetadataSnapshot,
}

impl PartialEq for DeckFile {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
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
pub struct DeckFilePayload {
    pub payload: Arc<RwLock<AudioFilePayload>>,
    pub preview: Option<Arc<RwLock<AudioFilePayload>>>,
}

#[derive(Clone, Debug)]
pub enum DeckFileStatus {
    Loading {
        progress: u32,
    },
    Playing {
        offset: Arc<AtomicU32>,
        payload: DeckFilePayload,
    },
    Paused {
        offset: Arc<AtomicU32>,
        payload: DeckFilePayload,
    },
}

impl DeckFileStatus {
    pub fn payload(&self) -> Option<DeckFilePayload> {
        match self {
            DeckFileStatus::Playing { payload, .. } | DeckFileStatus::Paused { payload, .. } => Some(payload.clone()),
            DeckFileStatus::Loading { .. } => None,
        }
    }

    pub fn offset(&self) -> Option<Arc<AtomicU32>> {
        match self {
            DeckFileStatus::Playing { offset, .. } | DeckFileStatus::Paused { offset, .. } => Some(offset.clone()),
            DeckFileStatus::Loading { .. } => None,
        }
    }
}

impl PlayerDeck {

    pub fn new(id: u32, dispatcher: EventDispatcher<AudioPlayerEvent>) -> Self {
        Self {
            id,
            last_file_id: Arc::new(AtomicU32::new(0)),
            file: Arc::new(RwLock::new(None)),
            dispatcher,
        }
    }

    pub async fn load_file(
        &self,
        path: impl AsRef<Path>,
    ) -> eyre::Result<(DeckFile, Option<DeckFile>)> {
        let metadata = DeckFileMetadataSnapshot {
            title: path.as_ref().to_string_lossy().to_string(),
        };

        let loaded_file = DeckFile {
            id: (self.id, self.last_file_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1),
            path: path.as_ref().to_path_buf(),
            status: Arc::new(RwLock::new(DeckFileStatus::Loading { progress: 0 })),
            updated: Arc::new(AtomicBool::new(false)),
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

        tokio::spawn(
            self.clone()
                .background_load_file_content(path.as_ref().to_path_buf()),
        );

        Ok((loaded_file, previous))
    }

    pub async fn update_status<F: FnOnce(&mut DeckFileStatus) -> R, R>(&self, f: F) -> eyre::Result<R> {
        let file = self.file.read().await;
        let file = file.as_ref().ok_or_eyre("No file loaded")?;
        let mut status = file.status.write().await;
        let result = f(&mut status);
        self.dispatcher
            .dispatch(AudioPlayerEvent::DeckFileStatusUpdated {
                id: self.id,
                status: status.clone(),
            })
            .await?;
        Ok(result)
    }

    pub async fn set_status(&self, status: DeckFileStatus) -> eyre::Result<()> {
        self.update_status(|s| *s = status).await?;
        Ok(())
    }

    async fn background_load_file_content(self, path: PathBuf) -> eyre::Result<()> {
        let payload = tokio::task::spawn_blocking(move || {
            let audio_file = EncodedAudioFile::from_file(&path)?;
            let payload = audio_file.read_into_payload()?;
            let preview = payload.generate_preview();
            selectia_audio_file::error::AudioFileResult::Ok(DeckFilePayload {
                payload: Arc::new(RwLock::new(payload)),
                preview: preview.ok().map(|preview| Arc::new(RwLock::new(preview))),
            })
        })
        .await??;

        let snapshot = DeckFilePayloadSnapshot::new(&payload).await;
        self.set_status(DeckFileStatus::Paused { offset: Arc::new(AtomicU32::new(0)), payload }).await?;

        let _ = self
            .dispatcher
            .dispatch(AudioPlayerEvent::DeckFilePayloadUpdated {
                id: self.id,
                payload: snapshot,
            })
            .await;
        Ok(())
    }
}

impl DeckFilePayloadSnapshot {
    pub async fn new(payload: &DeckFilePayload) -> Self {
        let audio_payload = payload.payload.read().await;
        let duration = audio_payload.duration;
        let sample_rate = audio_payload.sample_rate;
        let channels_count = audio_payload.channels as usize;
        let samples_count = audio_payload.buffer.buffer.len();

        // ** Note ** We dont want to keep `audio_payloaded` locked during the clone of sanpshot that can take a while
        drop(audio_payload);

        let preview_payload = match payload.preview.as_ref() {
            Some(preview) => {
                let preview_payload = preview.read().await;
                Some(DeckFilePreview {
                    original_sample_rate: sample_rate,
                    sample_rate: preview_payload.sample_rate,
                    channels_count: preview_payload.channels as usize,
                    samples: preview_payload.buffer.buffer.clone(),
                })
            }
            None => None,
        };

        Self {
            duration,
            sample_rate,
            channels_count,
            samples_count,
            preview: preview_payload,
        }
    }
}

impl std::fmt::Debug for PlayerDeck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PlayerDeck {{ id: {} }}", self.id)
    }
}
