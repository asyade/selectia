use std::{collections::BTreeMap, sync::atomic::AtomicU32};

use crate::prelude::*;

pub use deck::PlayerDeck;
mod deck;

pub type AudioPlayerService = AddressableServiceWithDispatcher<AudioPlayerTask, AudioPlayerEvent>;

#[derive(Clone)]
pub enum AudioPlayerTask {
    CreateDeck {
    },
    Exit,
}

#[derive(Clone)]
pub enum AudioPlayerEvent {
    Exit,
}

pub fn audio_player() -> AudioPlayerService {
    AddressableServiceWithDispatcher::new(move |mut receiver, sender, dispatcher| async move {
        let next_deck_id = Arc::new(AtomicU32::new(1));
        let decks: Arc<RwLock<BTreeMap<u32, PlayerDeck>>> = Arc::new(RwLock::new(BTreeMap::new()));
    
        while let Some(task) = receiver.blocking_recv() {
            match task {
                AudioPlayerTask::Exit => {
                    break;
                },
                AudioPlayerTask::CreateDeck { } => {
                    let id = next_deck_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    decks.write().await.insert(id, PlayerDeck::new());
                }
            }
        }

        Ok(())
    })
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


impl CancelableTask for AudioPlayerTask {
    fn cancel() -> Self {
        Self::Exit
    }
}

impl CancelableTask for AudioPlayerEvent {
    fn cancel() -> Self {
        Self::Exit
    }
}

