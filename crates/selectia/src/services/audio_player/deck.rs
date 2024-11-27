use crate::prelude::*;
use audio_player::AudioPlayerEvent;
use eyre::OptionExt;
use futures::{channel::oneshot, pin_mut};
use selectia_audio_file::AudioFile;
use std::sync::atomic::AtomicU32;

#[derive(Clone)]
pub struct PlayerDeck {
    id: u32,
    file: Arc<RwLock<Option<DeckFile>>>,
    dispatcher: EventDispatcher<AudioPlayerEvent>,
}

pub struct DeckFile {
    file: Arc<RwLock<AudioFile>>,
    state: DeckFileState,
}

#[derive(Clone)]
pub struct DeckFileState {
    /// The status of the current file.
    /// It is updated by the loader thread at the beginning of the file loading process.
    /// Once loaded, the player thread will update the status.
    status: Arc<RwLock<DeckFileStatus>>,
    path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct DeckFileStateSnapshot {
    pub status: DeckFileStatus,
    pub path: PathBuf,
}


#[derive(Clone, Debug)]
pub enum DeckFileStatus {
    Loading {
        progress: u32,
    },
    Playing {
        offset: u32,
    },
    Paused {
        offset: u32,
    },
}

impl PlayerDeck {
    pub fn new(id: u32, dispatcher: EventDispatcher<AudioPlayerEvent>) -> Self {
        Self {
            id,
            file: Arc::new(RwLock::new(None)),
            dispatcher,
        }
    }

    pub async fn load_file(&self, path: impl AsRef<Path>) -> eyre::Result<()> {
        let file = tokio::fs::File::open(&path).await?;
        let file = Box::new(file.into_std().await);
        let audio_file = AudioFile::from_source(file, &path)?;

        let state = DeckFileState {
            path: path.as_ref().to_path_buf(),
            status: Arc::new(RwLock::new(DeckFileStatus::Loading { progress: 0 })),
        };

        *self.file.write().await = Some(DeckFile {
            file: Arc::new(RwLock::new(audio_file)),
            state: state.clone(),
        });
        self.dispatcher
            .dispatch(AudioPlayerEvent::DeckFileUpdated { id: self.id, state: DeckFileStateSnapshot::from_state(&state).await })
            .await?;
        tokio::spawn(self.clone().background_load_file_content());
        Ok(())
    }

    pub async fn set_status(&self, status: DeckFileStatus) -> eyre::Result<()> {
        let state = self.file.read().await.as_ref().ok_or_eyre("No file loaded")?.state.clone();
        state.set_status(status).await;
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

        tokio::task::spawn_blocking(move || {
            info!("Decoding file");
            let mut lock = file.blocking_write();
            lock.decode().unwrap();
            info!("File decoded");
        })
        .await?;

        let _player_task_handle = tokio::spawn(self.clone().player_task());
        Ok(())
    }

    async fn player_task(self) -> eyre::Result<()> {
        let file = self.get_file().await?;
        let (sample_rate, channels) = {
            let mut lock = file.read().await;
            let sample_rate = lock.sample_rate();
            let channels = lock.channels();
            (sample_rate, channels)
        };

        let mut timer = tokio::time::interval(std::time::Duration::from_millis(1000));

        loop {
            timer.tick().await;
            info!("TICK");
            info!("{:?}", channels);
        }
        Ok(())
    }
}

impl DeckFileState {
    pub async fn set_status(&self, status: DeckFileStatus) {
        *self.status.write().await = status;
    }
}

impl DeckFileStateSnapshot {
    pub async fn from_state(state: &DeckFileState) -> Self {
        Self { status: state.status.read().await.clone(), path: state.path.clone() }
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
