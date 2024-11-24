use interactive_list_context::InteractiveListContext;
use tauri::{AppHandle, State};

use crate::prelude::*;

#[derive(Clone)]
pub struct App {
    pub(crate) handle: Option<AppHandle>,
    pub(crate) database: Database,
    pub(crate) state_machine: StateMachine,
    pub(crate) file_loader: FileLoader,
    pub(crate) interactive_list_context: ContextProvider<InteractiveListContext>,
}

pub struct AppState(pub(crate) Arc<RwLock<App>>);

pub type AppArg<'a> = State<'a, AppState>;


impl App {
    pub fn handle(&self) -> &AppHandle {
        self.handle.as_ref().expect("handle() `App::handle` called before setup")
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
