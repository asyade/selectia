// Allow unused imports to silence warnings for early development
#![allow(unused_imports)]

use interactive_list_context::InteractiveListContext;
use selectia::database::models::TagName;
use selectia::database::views::{
    entry_view::{EntryView, EntryViewFilter},
    TagView,
};
use std::sync::{Arc, RwLock};
use tauri::{Manager, State};

use crate::prelude::*;

mod commands;
mod context;
mod error;
mod prelude;
mod app;
use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    let database = Database::new(&PathBuf::from("/tmp/selectia.db"))
        .await
        .unwrap();
    let state_machine = state_machine(database.clone());
    let file_loader = file_loader(state_machine.clone());

    let app_state = AppState(Arc::new(RwLock::new(App {
        database,
        state_machine,
        file_loader,
        interactive_list_context: ContextProvider::new(),
    })));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            import_folder,
            get_tag_names,
            get_tags_by_name,
            get_interactive_list_context_entries,
            interactive_list_get_tag_creation_suggestions,
            interactive_list_create_tag,
            interactive_list_delete_context,
            interactive_list_create_context,
        ])
        .setup(setup)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let _handle = app.handle().clone();
    let _app_state = app.state::<AppState>().0.clone();

    Ok(())
}

impl App {
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

    pub async fn set_metadata_tag(
        &self,
        metadata_id: i64,
        tag_name_id: i64,
        value: String,
    ) -> eyre::Result<()> {
        self.database
            .set_metadata_tag_by_tag_name_id(metadata_id, tag_name_id, value)
            .await
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
