use crate::prelude::*;
use audio_player::AudioPlayerEvent;
use futures::channel::oneshot;
use selectia_audio_file::AudioFile;

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
    pub status: Arc<RwLock<DeckFileStatus>>,
    pub path: PathBuf,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeckFileStatus {
    Loading,
    Ready,
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
            status: Arc::new(RwLock::new(DeckFileStatus::Loading)),
        };

        *self.file.write().await = Some(DeckFile {
            file: Arc::new(RwLock::new(audio_file)),
            state: state.clone(),
        });
        self.dispatcher
            .dispatch(AudioPlayerEvent::DeckFileUpdated { id: self.id, state })
            .await?;
        tokio::spawn(self.clone().background_load_file_content());
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

        let state = {
            let mut state = self.file.write().await;
            let state = state.as_mut().unwrap();
            *state.state.status.write().await = DeckFileStatus::Ready;
            state.state.clone()
        };

        self.dispatcher
            .dispatch(AudioPlayerEvent::DeckFileUpdated { id: self.id, state })
            .await?;
        Ok(())
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

