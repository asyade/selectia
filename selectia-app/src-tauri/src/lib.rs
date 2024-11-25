// Allow unused imports to silence warnings for early development
#![allow(unused_imports)]

use std::clone;

use interactive_list_context::InteractiveListContext;
use selectia::database::models::TagName;
use selectia::database::views::{
    entry_view::{EntryView, EntryViewFilter},
    TagView,
};
use tauri::{Manager, State};

use crate::prelude::*;

mod scheduler;
mod commands;
mod context;
mod error;
mod prelude;
mod app;
use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    let app_state = AppState(Arc::new(RwLock::new(App::new().await)));

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
    let handle = app.handle().clone();
    let app_state = app.state::<AppState>().0.clone();
    // @TODO: **Important** there is race condition here, nothing protect the app to be used during background task execution causing crash if the handle is unwraped which is the case in the `App::handle()` function
    tokio::spawn(async move {
        let mut app_state = app_state.write().await;
        app_state.setup(handle).await.expect("Failed to setup app");
    });
    Ok(())
}
