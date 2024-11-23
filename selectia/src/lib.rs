#![feature(path_file_prefix)]

use std::pin::Pin;

use database::models::{TagName, Tag};
use tasks::generate_cluster::GenerateClusterTask;
use views::entry_view::{EntryView, EntryViewFilter};

use crate::prelude::*;

pub mod database;
pub mod prelude;
pub mod result;
pub mod services;
pub mod tasks;
pub mod ext;

#[derive(Clone)]
pub struct Engine {
    database: Database,
    state_machine: StateMachine,
    file_loader: FileLoader,
}

#[derive(Clone, Debug)]
pub struct EngineConfig {
    pub database_path: PathBuf,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self { database_path: PathBuf::from("/tmp/selectia.db") }
    }
}

impl Engine {
    #[instrument]
    pub async fn new(config: EngineConfig) -> Result<Self> {
        let database = Database::new(&config.database_path).await?;
        let state_machine = state_machine(database.clone());
        let file_loader = file_loader(state_machine.clone());
        info!("Ready !");
        Ok(Self {  database, state_machine, file_loader })
    }

    pub async fn stop(self) -> Result<Self> {
        self.file_loader.join().await?;
        let futures: Vec<Pin<Box<dyn Future<Output = Result<()>>>>> = vec![
            // Box::pin(self.embedding.join()),
            // Box::pin(self.file_loader.join()),
            Box::pin(self.state_machine.join()),
        ];
        futures::future::join_all(futures).await;
        Ok(self)
    }

    pub async fn with_embedding(self) -> Result<Self> {
        let embedding = embedding(self.state_machine.clone());
        self.state_machine.register_channel(embedding.sender().clone()).await;
        Ok(self)
    }

    pub async fn load_directory(self, path: PathBuf) -> Result<()> {
        LoadDirectory::new(self.file_loader.clone(), path)?.load().await?;
        Ok(())
    }

    pub async fn generate_cluster(self) -> Result<Self> {
        GenerateClusterTask::new(self.database.clone()).await?;
        Ok(self)
    }

    pub async fn get_tag_names(self) -> Result<Vec<TagName>> {
        self.database.get_tag_names().await
    }

    pub async fn get_tags_by_name(self, tag_name: &str) -> Result<Vec<Tag>> {
        self.database.get_tags_by_name(tag_name).await
    }

    pub async fn get_entries(self, filter: &EntryViewFilter) -> Result<Vec<EntryView>> {
        self.database.get_entries(filter).await
    }

    pub async fn list(self) -> Result<Self> {
        let files = self.database.list_files().await?;
        for file in files {
            println!("{}", file.path);
        }
        Ok(self)
    }

}
