use interactive_list_context::InteractiveListContext;
use selectia::analyser::EntriesAnalyser;
use selectia::database::models::TagName;
use selectia::database::Database;
use selectia::tasks::generate_cluster::GenerateClusterTask;
use selectia::{
    database::views::{
        entry_view::{EntryView, EntryViewFilter},
        TagView,
    },
};
use std::pin::Pin;
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};
use tauri::{Manager, State};

use crate::prelude::*;

mod context;
mod prelude;

#[derive(Clone)]
pub struct App {
    database: Database,
    state_machine: StateMachine,
    file_loader: FileLoader,
    interactive_list_context: ContextProvider<InteractiveListContext>,
}

pub type AppResult<T> = Result<T, String>;


pub struct AppState(Arc<RwLock<App>>);

pub type AppArg<'a> = State<'a, AppState>;

#[tauri::command]
#[instrument(skip(app))]
async fn interactive_list_create_context<'a>(app: AppArg<'a>) -> AppResult<String> {
    let context = InteractiveListContext::new(app.0.read().unwrap().clone());
    let lock = app
        .0
        .write()
        .unwrap()
        .interactive_list_context
        .clone();
    let id = lock.create_context(context).await;
    info!(context_id = id.to_string(), "Created interactive list context");
    Ok(id.to_string())
}

#[tauri::command]
#[instrument(skip(app))]
async fn interactive_list_delete_context<'a>(context_id: String, app: AppArg<'a>) -> AppResult<()> {
    info!(context_id = context_id, "Deleting interactive list context");
    let lock = app
        .0
        .write()
        .unwrap()
        .interactive_list_context
        .clone();

    lock.delete_context(ContextId::try_from(context_id).map_err(|e| e.to_string())?)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
#[instrument(skip(app))]
async fn interactive_list_get_tag_creation_suggestions<'a>(context_id: String, tag_name_id: i64, input: String, app: AppArg<'a>) -> AppResult<Vec<String>> {
    let lock = app.0.read().unwrap().interactive_list_context.clone();
    lock.get_context(ContextId::try_from(context_id).map_err(|e| e.to_string())?)
        .await
        .map_err(|e| e.to_string())?
        .get_tag_creation_suggestions(tag_name_id, input)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[instrument(skip(app))]
async fn interactive_list_create_tag<'a>(context_id: String, metadata_id: i64, name_id: i64, value: String, app: AppArg<'a>) -> AppResult<()> {
    let lock = app.0.read().unwrap().interactive_list_context.clone();
    lock.get_context(ContextId::try_from(context_id).map_err(|e| e.to_string())?)
        .await
        .map_err(|e| e.to_string())?
        .create_tag(metadata_id, name_id, value)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[instrument(skip(app))]
async fn get_interactive_list_context_entries<'a>(
    context_id: String,
    filter: EntryViewFilter,
    app: AppArg<'a>,
) -> AppResult<Vec<EntryView>> {
    info!("Getting interactive list context entries");
    let lock = app
        .0
        .read()
        .unwrap()
        .interactive_list_context
        .clone();

    let context = lock
        .get_context(ContextId::try_from(context_id).map_err(|e| e.to_string())?)
        .await
        .map_err(|e| e.to_string())?;
    let entries = context.get_entries(filter).await.map_err(|e| e.to_string())?;
    Ok(entries.into_iter().map(EntryView::from).collect())
}

#[tauri::command]
async fn import_folder<'a>(directory: String, app: AppArg<'a>) -> AppResult<String> {
    let fut = app
        .0
        .write()
        .unwrap()
        .clone()
        .load_directory(PathBuf::from(directory));
    fut.await.map_err(|e| e.to_string())?;
    Ok("ok".to_string())
}

#[tauri::command]
async fn get_tag_names<'a>(app: AppArg<'a>) -> AppResult<Vec<TagName>> {
    let fut = app.0.read().unwrap().clone().get_tag_names();
    let tags = fut.await.map_err(|e| e.to_string())?;
    Ok(tags)
}

#[tauri::command]
async fn get_tags_by_name<'a>(tag_name: String, app: AppArg<'a>) -> AppResult<Vec<TagView>> {
    let fut = app
        .0
        .read()
        .unwrap()
        .clone()
        .get_tags_by_name(&tag_name);
    let tags = fut.await.map_err(|e| e.to_string())?;
    Ok(tags.into_iter().map(TagView::from).collect())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    let database = Database::new(&PathBuf::from("/tmp/selectia.db")).await.unwrap();
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
        self.state_machine.register_channel(embedding.sender().clone()).await;
        Ok(self)
    }

    pub async fn load_directory(self, path: PathBuf) -> eyre::Result<()> {
        LoadDirectory::new(self.file_loader.clone(), path)?.load().await?;
        Ok(())
    }

    pub async fn generate_cluster(self) -> eyre::Result<Self> {
        GenerateClusterTask::new(self.database.clone()).await?;
        Ok(self)
    }

    pub async fn get_tag_names(self) -> eyre::Result<Vec<TagName>> {
        self.database.get_tag_names().await
    }

    pub async fn set_metadata_tag(&self, metadata_id: i64, tag_name_id: i64, value: String) -> eyre::Result<()> {
        self.database.set_metadata_tag_by_tag_name_id(metadata_id, tag_name_id, value).await
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
