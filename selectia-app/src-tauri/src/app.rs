use audio_player::{audio_player, AudioPlayerEvent, AudioPlayerService};
use interactive_list_context::InteractiveListContext;
use selectia::database::models::Task;
use state_machine::StateMachineEvent;
use tauri::{AppHandle, Emitter, State};
use worker::{
    tasks::{BackgroundTask, FileAnalysisTask, TaskPayload, TaskStatus},
    worker, Worker, WorkerEvent, WorkerTask,
};

use crate::dto::to_frontend::*;
use crate::{prelude::*, settings::Settings};

#[derive(Clone)]
pub struct App {
    pub(crate) handle: Option<AppHandle>,
    pub(crate) database: Database,
    pub(crate) audio_player: AudioPlayerService,
    pub(crate) worker: Worker,
    pub(crate) state_machine: StateMachine,
    pub(crate) file_loader: FileLoader,
    pub(crate) interactive_list_context: ContextProvider<InteractiveListContext>,
}

pub struct AppState(pub(crate) Arc<RwLock<App>>);

pub type AppArg<'a> = State<'a, AppState>;

impl App {
    pub async fn new() -> Self {
        let settings = Settings::load().expect("Failed to load settings");
        let database = Database::new(&settings.database_path).await.unwrap();
        let audio_player = audio_player(database.clone());
        let state_machine = state_machine(database.clone());
        let file_loader = file_loader(state_machine.clone());

        let worker = worker(database.clone());

        App {
            handle: None,
            database,
            audio_player,
            worker,
            state_machine,
            file_loader,
            interactive_list_context: ContextProvider::new(),
        }
    }

    pub async fn setup(&mut self, handle: AppHandle) -> eyre::Result<()> {
        self.handle = Some(handle.clone());

        let handle_clone = handle.clone();
        self.worker
            .register_channel(channel_iterator(move |msg| {
                match msg {
                    WorkerEvent::QueueTaskCreated { id, status } => {
                        let task = WorkerQueueTask { id, status };
                        let _ = handle_clone.emit(
                            "worker-queue-task-created",
                            WorkerQueueTaskCreatedEvent { task },
                        );
                    }
                    WorkerEvent::QueueTaskUpdated {
                        id,
                        status,
                        removed,
                    } => {
                        let task = if removed {
                            None
                        } else {
                            Some(WorkerQueueTask { id, status })
                        };
                        let _ = handle_clone.emit(
                            "worker-queue-task-updated",
                            WorkerQueueTaskUpdatedEvent { id, task },
                        );
                    }
                }
            }))
            .await;

        let handle_clone = handle.clone();
        self.audio_player
            .register_channel(channel_iterator(move |msg| {
                match msg {
                    AudioPlayerEvent::DeckCreated { id } => {
                        let _ = handle_clone.emit("audio-deck-created", AudioDeckCreatedEvent { id });
                    }
                    AudioPlayerEvent::DeckFileUpdated { id, state } => {
                        let file = Some(DeckFileView {
                            title: state.path.to_string_lossy().to_string(),
                            length: 0.0,
                            offset: 0.0,
                        });
                        let _ = handle_clone.emit("audio-deck-updated", AudioDeckUpdatedEvent { id, file });
                    }
                }
            }))
            .await;

        let app_handle = self.clone();
        self.state_machine
            .register_channel(async_channel_iterator(move |msg| {
                let app_handle = app_handle.clone();
                async move {
                    match msg {
                        StateMachineEvent::FileIngested { file, new: true } => {
                            if let Err(e) =
                                app_handle.schedule_file_analysis(file.metadata_id).await
                            {
                                error!("Failed to schedule file analysis: {}", e);
                            }
                        }
                        _ => {}
                    }
                }
            }))
            .await;

        Ok(())
    }

    pub async fn schedule_file_analysis(&self, metadata_id: i64) -> eyre::Result<()> {
        let task = TaskPayload::FileAnalysis(FileAnalysisTask { metadata_id });
        self.worker.send(WorkerTask::Schedule(task)).await?;
        Ok(())
    }

    pub fn handle(&self) -> &AppHandle {
        self.handle
            .as_ref()
            .expect("handle() `App::handle` called before setup")
    }

    pub async fn load_directory(self, path: PathBuf) -> eyre::Result<()> {
        LoadDirectory::new(self.file_loader.clone(), path)?
            .load()
            .await?;
        Ok(())
    }

    pub async fn get_tag_names(self) -> eyre::Result<Vec<TagName>> {
        self.database.get_tag_names().await
    }

    pub async fn get_tags_by_name(self, tag_name: &str) -> eyre::Result<Vec<Tag>> {
        self.database.get_tags_by_name(tag_name).await
    }

    pub async fn get_entries(&self, filter: &EntryViewFilter) -> eyre::Result<Vec<EntryView>> {
        self.database.get_entries(filter).await
    }

    pub async fn list(self) -> eyre::Result<Self> {
        let files = self.database.list_files().await?;
        for file in files {
            println!("{}", file.path);
        }
        Ok(self)
    }
}
