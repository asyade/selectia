use crate::prelude::*;
use futures::channel::oneshot;
use selectia_audio_file::AudioFile;

#[derive(Clone)]
pub struct PlayerDeck {
    file: Arc<RwLock<Option<DeckFile>>>,
}

pub struct DeckFile {
    file: Arc<RwLock<AudioFile>>,
    status: Arc<RwLock<DeckFileState>>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeckFileState {
    Loading,
    Ready,
}

impl PlayerDeck {
    pub fn new() -> Self {
        Self {
            file: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn load_file(&self, path: impl AsRef<Path>) -> eyre::Result<()> {
        let file = tokio::fs::File::open(&path).await?;
        let file = Box::new(file.into_std().await);
        let audio_file = AudioFile::from_source(file, path)?;
        *self.file.write().await = Some(DeckFile {
            file: Arc::new(RwLock::new(audio_file)),
            status: Arc::new(RwLock::new(DeckFileState::Ready)),
        });
        Ok(())
    }
}

