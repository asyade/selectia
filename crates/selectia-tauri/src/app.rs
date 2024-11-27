use std::{ops::{Deref, DerefMut}, sync::RwLockReadGuard};

use audio_player::{audio_player, AudioPlayerEvent, AudioPlayerService};
use dto::Events;
use interactive_list_context::InteractiveListContext;
use selectia::database::models::Task;
use state_machine::StateMachineEvent;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::RwLockWriteGuard;
use worker::{
    tasks::{BackgroundTask, FileAnalysisTask, TaskPayload, TaskStatus},
    worker, Worker, WorkerEvent, WorkerTask,
};

use crate::{prelude::*, settings::Settings};

use crate::commands::*;

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

pub struct AppState(pub Arc<RwLock<App>>);

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

    pub fn emit<T: Into<Events>>(&self, event: T) -> eyre::Result<()> {
        let event: Events = event.into();
        let handle = self
            .handle
            .as_ref()
            .expect("handle() `App::handle` called before setup");
        handle.emit(event.name(), event)?;
        Ok(())
    }

    pub async fn setup(&mut self, handle: AppHandle) -> eyre::Result<()> {
        self.handle = Some(handle.clone());

        let app_handle = self.clone();
        self.worker
            .register_channel(channel_iterator(move |msg| match msg {
                WorkerEvent::QueueTaskCreated { id, status } => {
                    let task = dto::WorkerQueueTask {
                        id,
                        status: status.into(),
                    };
                    let _ = app_handle.emit(dto::WorkerQueueTaskCreatedEvent { task });
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
                    let _ = app_handle.emit(dto::WorkerQueueTaskUpdatedEvent { id, task });
                }
            }))
            .await;

        let app_handle = self.clone();
        self.audio_player
            .register_channel(channel_iterator(move |msg| {
                info!("audio_player_event: {:?}", &msg);
                match msg {
                    AudioPlayerEvent::DeckCreated { id } => {
                        let _ = app_handle.emit(dto::AudioDeckCreatedEvent { id });
                    }
                    AudioPlayerEvent::DeckFileUpdated { id, state } => {
                        let file = Some(dto::DeckFileView {
                            title: state.path.to_string_lossy().to_string(),
                            length: 0.0,
                            offset: 0.0,
                        });
                        let _ = app_handle.emit(dto::AudioDeckUpdatedEvent { id, file });
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

impl AppState {
    pub fn new(app: App) -> Self {
        AppState(Arc::new(RwLock::new(app)))
    }
}

impl AsRef<Arc<RwLock<App>>> for AppState {
    fn as_ref(&self) -> &Arc<RwLock<App>> {
        &self.0
    }
}

impl AsMut<Arc<RwLock<App>>> for AppState {
    fn as_mut(&mut self) -> &mut Arc<RwLock<App>> {
        &mut self.0
    }
}

impl Deref for AppState {
    type Target = Arc<RwLock<App>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AppState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
