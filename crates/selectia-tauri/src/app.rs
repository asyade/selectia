use std::{
    ops::{Deref, DerefMut},
    sync::RwLockReadGuard,
};

use audio_player::{audio_player, AudioPlayer, AudioPlayerEvent, AudioPlayerService};
use demuxer::{Demuxer, DemuxerEvent, DemuxerStatus, DemuxerTask};
use dto::Events;
use interactive_list_context::InteractiveListContext;
use selectia::database::models::Task;
use state_machine::StateMachineEvent;
use tauri::{AppHandle, Emitter, State};
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
    pub(crate) interactive_list_context: ContextProvider<InteractiveListContext>,
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
            .await;

        audio_player({ &*context }.clone()).await;
        state_machine({ &*context }.clone()).await;
        file_loader({ &*context }.clone()).await;
        demuxer::demuxer({ &*context }.clone(), settings.demuxer_data_path.clone()).await;
        worker({ &*context }.clone()).await;

        App {
            context: context,
            interactive_list_context: ContextProvider::new(),
        }
    }

    /// Called once tauri is ready this function will create required binding betwen the global Theater and the tauri runtime.
    pub async fn setup(&self, handle: AppHandle) -> eyre::Result<()> {
        self.context.register_singleton(handle.clone()).await;
        let ui_dispatcher = handle.clone();
        self.context
            .get_service::<Worker>()
            .await?
            .register_channel(channel_iterator(move |msg| match msg {
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
            .get_service::<AudioPlayerService>()
            .await?
            .register_channel(channel_iterator(move |msg| match msg {
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

        info!("Setup done, starting scene ...");
        self.context.ready().await;
        Ok(())
    }

    pub async fn schedule_file_analysis(&self, metadata_id: i64) -> eyre::Result<()> {
        let task = TaskPayload::FileAnalysis(FileAnalysisTask { metadata_id });
        self.context
            .get_service::<Worker>()
            .await?
            .send(WorkerTask::Schedule(task))
            .await?;
        Ok(())
    }
}
