use selectia::database::models::{Tag, TagName};
use selectia::{
    database::{
        views::{entry_view::{EntryView, EntryViewFilter}, TagView},
    },
    EngineConfig,
};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
};
use tauri::{
    ipc::{InvokeResponseBody, IpcResponse},
    Emitter, Manager, State,
};
use thiserror::Error;
pub struct App {
    engine: selectia::Engine,
}

pub type AppResult<T> = Result<T, String>;

pub struct AppState(Arc<RwLock<App>>);

pub type AppArg<'a> = State<'a, AppState>;

#[tauri::command]
async fn import_folder<'a>(directory: String, app: AppArg<'a>) -> AppResult<String> {
    let fut = app
        .0
        .write()
        .unwrap()
        .engine
        .clone()
        .load_directory(PathBuf::from(directory));
    fut.await.map_err(|e| e.to_string())?;
    Ok("ok".to_string())
}

#[tauri::command]
async fn get_tag_names<'a>(app: AppArg<'a>) -> AppResult<Vec<TagName>> {
    let fut = app.0.read().unwrap().engine.clone().get_tag_names();
    let tags = fut.await.map_err(|e| e.to_string())?;
    Ok(tags)
}

#[tauri::command]
async fn get_tags_by_name<'a>(tag_name: String, app: AppArg<'a>) -> AppResult<Vec<TagView>> {
    let fut = app
        .0
        .read()
        .unwrap()
        .engine
        .clone()
        .get_tags_by_name(&tag_name);
    let tags = fut.await.map_err(|e| e.to_string())?;
    Ok(tags.into_iter().map(TagView::from).collect())
}

#[tauri::command]
async fn get_entries<'a>(filter: EntryViewFilter, app: AppArg<'a>) -> AppResult<Vec<EntryView>> {
    let fut = app.0.read().unwrap().engine.clone().get_entries(&filter);
    let entries = fut.await.map_err(|e| e.to_string())?;
    Ok(entries.into_iter().map(EntryView::from).collect())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    let app_state = AppState(Arc::new(RwLock::new(App {
        engine: selectia::Engine::new(EngineConfig::default())
            .await
            .unwrap(),
    })));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            import_folder,
            get_tag_names,
            get_tags_by_name,
            get_entries
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
