use std::{collections::BTreeMap, sync::atomic::AtomicU32};

use crate::prelude::*;

use deck::DeckFileState;
pub use deck::PlayerDeck;
mod deck;

pub type AudioPlayerService = AddressableServiceWithDispatcher<AudioPlayerTask, AudioPlayerEvent>;

#[derive(Clone, Debug)]
pub enum AudioPlayerTask {
    CreateDeck {
        callback: TaskCallback<u32>,
    },
    GetDecks {
        callback: TaskCallback<BTreeMap<u32, PlayerDeck>>,
    },
    LoadTrack {
        deck_id: u32,
        metadata_id: i64,
    },
}

#[derive(Clone, Debug)]
pub enum AudioPlayerEvent {
    DeckCreated { id: u32 },
    DeckFileUpdated { 
        id: u32,
        state: DeckFileState,    
    },
}

pub struct AudioPlayer {
    database: Database,
    decks: Arc<RwLock<BTreeMap<u32, PlayerDeck>>>,
    next_deck_id: Arc<AtomicU32>,
    dispatcher: EventDispatcher<AudioPlayerEvent>,
}

pub fn audio_player(database: Database) -> AudioPlayerService {
    AddressableServiceWithDispatcher::new(move |mut receiver, _sender, dispatcher| async move {
        AudioPlayer::new(database, dispatcher)
            .handle(&mut receiver)
            .await?;
        Ok(())
    })
}

impl AudioPlayer {
    pub fn new(database: Database, dispatcher: EventDispatcher<AudioPlayerEvent>) -> Self {
        Self {
            database,
            decks: Arc::new(RwLock::new(BTreeMap::new())),
            next_deck_id: Arc::new(AtomicU32::new(1)),
            dispatcher,
        }
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
                info!("Creating deck");
                let id = self
                    .next_deck_id
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                self.decks.write().await.insert(id, PlayerDeck::new(id, self.dispatcher.clone()));
                let _ = callback.resolve(id).await?;
                let _ = self
                    .dispatcher
                    .dispatch(AudioPlayerEvent::DeckCreated { id })
                    .await?;
            }
            AudioPlayerTask::GetDecks { callback } => {
                let decks = self.decks.read().await.clone();
                let _ = callback.resolve(decks).await?;
            }
            AudioPlayerTask::LoadTrack {
                deck_id,
                metadata_id,
            } => {
                info!("Loading track");
                let file = self.database.get_file_from_metadata_id(metadata_id).await?;
                let deck = self
                    .decks
                    .read()
                    .await
                    .get(&deck_id)
                    .cloned()
                    .ok_or(eyre!("Deck not found"))?;
                deck.load_file(file.path).await?;
            }
        }
        Ok(())
    }
}

//     pub async fn create_deck(&self) -> u32 {
//         let id = self.next_deck_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
//         self.decks.write().await.insert(id, PlayerDeck::new());
//         id
//     }

//     pub async fn get_deck(&self, id: u32) -> Option<PlayerDeck> {
//         self.decks.read().await.get(&id).cloned()
//     }
// }

impl Task for AudioPlayerTask {}

impl Task for AudioPlayerEvent {}
