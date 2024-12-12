use std::{
    ops::{Deref, DerefMut},
    sync::RwLockReadGuard,
};

use audio_player::{audio_player, AudioPlayer, AudioPlayerEvent, AudioPlayerService};
use dto::Events;
use interactive_list_context::InteractiveListContext;
use selectia::database::models::Task;
use state_machine::StateMachineEvent;
use tauri::{AppHandle, Emitter, Manager, State};
use theater::{
    context::OwnedTheaterContext,
    dispatcher::{async_channel_iterator, channel_iterator},
};
use tokio::sync::RwLockWriteGuard;
use worker::{
    tasks::{BackgroundTask, FileAnalysisTask, TaskPayload, TaskStatus},
    worker, Worker, WorkerEvent, WorkerTask,
};

use crate::{prelude::*, settings::Settings};

use crate::commands::*;

#[derive(Clone)]
pub struct App {
    pub(crate) context: OwnedTheaterContext,
}

pub type AppArg<'a> = State<'a, App>;

impl App {
    pub async fn new() -> Self {
        let context = OwnedTheaterContext::new().await;

        let settings = Settings::load().await.expect("Failed to load settings");

        context
            .register_singleton(
                Database::new(&settings.database_path)
                    .await
                    .expect("Failed to initialize database"),
            )
            .await
            .expect("Failed to register database singleton");

        AudioPlayerService::spawn(&context)
            .await
            .expect("Failed to spawn AudioPlayerService");
        StateMachine::spawn(&context)
            .await
            .expect("Failed to spawn StateMachine");
        FileLoader::spawn(&context)
            .await
            .expect("Failed to spawn FileLoader");
        Demuxer::spawn(&context, settings.demuxer_data_path.clone())
            .await
            .expect("Failed to spawn Demuxer");
        Worker::spawn(&context)
            .await
            .expect("Failed to spawn Worker");

        App {
            context: context,
        }
    }

    /// Called once tauri is ready this function will create required binding betwen the global Theater and the tauri runtime.
    pub async fn setup(&self, handle: AppHandle) -> eyre::Result<()> {
        handle.manage(self.context.get_singleton::<Database>().await?);
        handle.manage(self.context.get_singleton_address::<AudioPlayerService>().await?);
        handle.manage(self.context.get_singleton_address::<StateMachine>().await?);
        handle.manage(self.context.get_singleton_address::<FileLoader>().await?);
        handle.manage(self.context.get_singleton_address::<Worker>().await?);
        handle.manage(ContextProvider::<InteractiveListContext>::new());

        self.context.register_singleton(handle.clone()).await.expect("Failed to register app handle");
        let ui_dispatcher = handle.clone();
        self.context
            .get_singleton_dispatcher::<Worker, WorkerEvent>()
            .await?
            .register(channel_iterator(move |msg| match msg {
                WorkerEvent::QueueTaskCreated { id, status } => {
                    let task = dto::WorkerQueueTask {
                        id,
                        status: status.into(),
                    };
                    let _ = ui_dispatcher.emit_event(dto::WorkerQueueTaskCreatedEvent { task });
                }
                WorkerEvent::QueueTaskUpdated {
                    id,
                    status,
                    removed,
                } => {
                    let task = if removed {
                        None
                    } else {
                        Some(dto::WorkerQueueTask {
                            id,
                            status: status.into(),
                        })
                    };
                    let _ = ui_dispatcher.emit_event(dto::WorkerQueueTaskUpdatedEvent { id, task });
                }
            }))
            .await;

        let ui_dispatcher = handle.clone();
        self.context
            .get_singleton_dispatcher::<AudioPlayerService, AudioPlayerEvent>()
            .await?
            .register(channel_iterator(move |msg| match msg {
                AudioPlayerEvent::DeckCreated { id } => {
                    let _ = ui_dispatcher.emit_event(dto::AudioDeckCreatedEvent { id });
                }
                AudioPlayerEvent::DeckFileMetadataUpdated { id, metadata } => {
                    let _ = ui_dispatcher.emit_identified_event(
                        id,
                        dto::AudioDeckFileMetadataUpdatedEvent {
                            id,
                            metadata: metadata.into(),
                        },
                    );
                }
                AudioPlayerEvent::DeckFilePayloadUpdated { id, payload } => {
                    let _ = ui_dispatcher.emit_identified_event(
                        id,
                        dto::AudioDeckFilePayloadUpdatedEvent {
                            id,
                            payload: payload.into(),
                        },
                    );
                }
                AudioPlayerEvent::DeckFileStatusUpdated { id, status } => {
                    let _ = ui_dispatcher.emit_identified_event(
                        id,
                        dto::AudioDeckFileStatusUpdatedEvent {
                            id,
                            status: status.into(),
                        },
                    );
                }
            }))
            .await;
        self.context.ready().await;
        Ok(())
    }
}
