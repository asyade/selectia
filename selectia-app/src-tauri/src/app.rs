use interactive_list_context::InteractiveListContext;
use state_machine::StateMachineEvent;
use tauri::{AppHandle, Emitter, State};
use worker::{tasks::{BackgroundTask, FileAnalysisTask, TaskPayload}, worker, Worker, WorkerTask};

use crate::prelude::*;

#[derive(Clone)]
pub struct App {
    pub(crate) handle: Option<AppHandle>,
    pub(crate) database: Database,
    pub(crate) worker: Worker,
    pub(crate) state_machine: StateMachine,
    pub(crate) file_loader: FileLoader,
    pub(crate) interactive_list_context: ContextProvider<InteractiveListContext>,
}

pub struct AppState(pub(crate) Arc<RwLock<App>>);

pub type AppArg<'a> = State<'a, AppState>;

#[derive(Serialize, Clone)]
pub struct FileIngestedEvent {
    is_new: bool,
    metadata_id: i64,
}

impl App {
    pub async fn new() -> Self {
        let database = Database::new(&PathBuf::from("/tmp/selectia.db"))
            .await
            .unwrap();
        let state_machine = state_machine(database.clone());
        let file_loader = file_loader(state_machine.clone());

        let worker = worker(database.clone());

        App {
            handle: None,
            database,
            worker,
            state_machine,
            file_loader,
            interactive_list_context: ContextProvider::new(),
        }
    }

    pub async fn setup(&mut self, handle: AppHandle) -> eyre::Result<()> {
        self.handle = Some(handle.clone());
        let app_handle = self.clone();

        self.state_machine
            .register_channel(channel_iterator(move |msg| {
                let app_handle = app_handle.clone();
                async move {
                    match msg {
                        StateMachineEvent::FileIngested { file, new: true } => {
                            if let Err(e) = app_handle.schedule_file_analysis(file.metadata_id).await {
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
        let task = TaskPayload::FileAnalysis(FileAnalysisTask {
            metadata_id,
        });
        self.worker.send(WorkerTask::Schedule(task)).await?;
        Ok(())
    }

    pub fn handle(&self) -> &AppHandle {
        self.handle
            .as_ref()
            .expect("handle() `App::handle` called before setup")
    }

    pub async fn stop(self) -> eyre::Result<Self> {
        self.file_loader.join().await?;
        let futures: Vec<Pin<Box<dyn Future<Output = eyre::Result<()>>>>> = vec![
            // Box::pin(self.embedding.join()),
            // Box::pin(self.file_loader.join()),
            Box::pin(self.state_machine.join()),
        ];
        futures::future::join_all(futures).await;
        Ok(self)
    }

    pub async fn with_embedding(self) -> eyre::Result<Self> {
        let embedding = embedding(self.state_machine.clone());
        self.state_machine
            .register_channel(embedding.sender().clone())
            .await;
        Ok(self)
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
