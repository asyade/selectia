use std::collections::BTreeMap;

use audio_player::AudioPlayerTask;
use dto::{EntryChangedEvent, EntryListChangedEvent, TagListChangedEvent};
use interactive_list_context::InteractiveListContext;
use selectia::database::views::TagView;
use tauri::Emitter;
use worker::tasks::TaskStatus;

use crate::prelude::*;

#[tauri::command]
#[instrument(skip(app))]
pub async fn interactive_list_create_context<'a>(app: AppArg<'a>) -> AppResult<String> {
    let context: InteractiveListContext = InteractiveListContext::new(app.0.read().await.clone());
    let lock = app.0.write().await.interactive_list_context.clone();
    let id = lock.create_context(context).await;
    info!(
        context_id = id.to_string(),
        "Created interactive list context"
    );
    Ok(id.to_string())
}

#[tauri::command]
#[instrument(skip(app))]
pub async fn interactive_list_delete_context<'a>(
    context_id: String,
    app: AppArg<'a>,
) -> AppResult<()> {
    info!(context_id = context_id, "Deleting interactive list context");
    let lock = app.0.write().await.interactive_list_context.clone();

    lock.delete_context(ContextId::try_from(context_id).map_err(|e| e.to_string())?)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
#[instrument(skip(app))]
pub async fn interactive_list_get_tag_creation_suggestions<'a>(
    context_id: String,
    tag_name_id: i64,
    input: String,
    app: AppArg<'a>,
) -> AppResult<Vec<String>> {
    let lock = app.0.read().await.interactive_list_context.clone();
    lock.get_context(ContextId::try_from(context_id).map_err(|e| e.to_string())?)
        .await
        .map_err(|e| e.to_string())?
        .get_tag_creation_suggestions(tag_name_id, input)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[instrument(skip(app))]
pub async fn interactive_list_create_tag<'a>(
    context_id: String,
    metadata_id: i64,
    name_id: i64,
    value: String,
    app: AppArg<'a>,
) -> AppResult<()> {
    let entry = {
        let lock = app.0.read().await.interactive_list_context.clone();
        lock.get_context(ContextId::try_from(context_id).map_err(|e| e.to_string())?)
            .await
            .map_err(|e| e.to_string())?
            .create_tag(metadata_id, name_id, value)
            .await
            .map_err(|e| e.to_string())?
    };
    let handle = app.0.read().await;
    handle.emit(EntryChangedEvent { entry: entry.into() }).map_err(|e| e.to_string())?;
    handle.emit(TagListChangedEvent {}).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
#[instrument(skip(app))]
pub async fn get_interactive_list_context_entries<'a>(
    context_id: String,
    filter: EntryViewFilter,
    app: AppArg<'a>,
) -> AppResult<Vec<EntryView>> {
    info!("Getting interactive list context entries");
    let lock = app.0.read().await.interactive_list_context.clone();

    let context = lock
        .get_context(ContextId::try_from(context_id).map_err(|e| e.to_string())?)
        .await
        .map_err(|e| e.to_string())?;
    let entries = context
        .get_entries(filter)
        .await
        .map_err(|e| e.to_string())?;
    Ok(entries.into_iter().map(EntryView::from).collect())
}

#[tauri::command]
pub async fn import_folder<'a>(directory: String, app: AppArg<'a>) -> AppResult<String> {
    let fut = app
        .0
        .write()
        .await
        .clone()
        .load_directory(PathBuf::from(directory));
    fut.await.map_err(|e| e.to_string())?;
    let handle = app.0.read().await;
    handle.emit(EntryListChangedEvent {}).map_err(|e| e.to_string())?;
    Ok("ok".to_string())
}

#[tauri::command]
pub async fn get_tag_names<'a>(app: AppArg<'a>) -> AppResult<Vec<TagName>> {
    let fut = app.0.read().await.clone().get_tag_names();
    let tags = fut.await.map_err(|e| e.to_string())?;
    Ok(tags)
}

#[tauri::command]
pub async fn get_tags_by_name<'a>(tag_name: String, app: AppArg<'a>) -> AppResult<Vec<TagView>> {
    let fut = app.0.read().await.clone().get_tags_by_name(&tag_name);
    let tags = fut.await.map_err(|e| e.to_string())?;
    Ok(tags.into_iter().map(TagView::from).collect())
}

#[tauri::command]
pub async fn get_worker_queue_tasks<'a>(app: AppArg<'a>) -> AppResult<Vec<dto::WorkerQueueTask>> {
    let all_tasks = app
        .0
        .read()
        .await
        .database
        .get_tasks()
        .await
        .map_err(|e| e.to_string())?;
    Ok(all_tasks
        .into_iter()
        .map(|t| dto::WorkerQueueTask {
            id: t.id,
            status: TaskStatus::try_from(t.status.as_str()).unwrap().into(),
        })
        .collect())
}

#[tauri::command]
pub async fn get_worker_queue_task<'a>(
    task_id: i64,
    app: AppArg<'a>,
) -> AppResult<dto::WorkerQueueTask> {
    let task = app
        .0
        .read()
        .await
        .database
        .get_task(task_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(dto::WorkerQueueTask {
        id: task.id,
        status: TaskStatus::try_from(task.status.as_str()).unwrap().into(),
    })
}

#[tauri::command]
pub async fn create_audio_deck<'a>(app: AppArg<'a>) -> AppResult<u32> {
    let (callback, receiver) = TaskCallback::new();
    app.0
        .write()
        .await
        .audio_player
        .send(AudioPlayerTask::CreateDeck { callback })
        .await
        .map_err(|e| e.to_string())?;
    Ok(receiver.wait().await.unwrap())
}

#[tauri::command]
pub async fn get_audio_decks<'a>(app: AppArg<'a>) -> AppResult<Vec<dto::DeckView>> {
    let (callback, receiver) = TaskCallback::new();
    app.0
        .write()
        .await
        .audio_player
        .send(AudioPlayerTask::GetDecks { callback })
        .await
        .map_err(|e| e.to_string())?;
    let decks = receiver.wait().await.unwrap();
    Ok(decks
        .into_iter()
        .map(|(id, _deck)| dto::DeckView { id, file: None })
        .collect())
}

#[tauri::command]
pub async fn load_audio_track<'a>(
    deck_id: u32,
    metadata_id: i64,
    app: AppArg<'a>,
) -> AppResult<()> {
    app.0
        .write()
        .await
        .audio_player
        .send(AudioPlayerTask::LoadTrack {
            deck_id,
            metadata_id,
        })
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}