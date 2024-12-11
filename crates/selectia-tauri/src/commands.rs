use std::collections::BTreeMap;

use audio_player::{AudioPlayer, AudioPlayerService, AudioPlayerTask, TrackTarget};
use dto::{EntryChangedEvent, EntryListChangedEvent, TagListChangedEvent};
use interactive_list_context::InteractiveListContext;
use selectia::database::views::TagView;
use tauri::Emitter;
use worker::{
    tasks::{FileAnalysisTask, StemExtractionTask, TaskPayload, TaskStatus},
    Worker, WorkerTask,
};

use crate::prelude::*;

#[tauri::command]
#[instrument(skip(app))]
pub async fn interactive_list_create_context<'a>(app: AppArg<'a>) -> AppResult<String> {
    let context: InteractiveListContext =
        InteractiveListContext::new(app.read().await.clone()).await;
    let lock = app.read().await.interactive_list_context.clone();
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
    let lock = app.read().await.interactive_list_context.clone();
    lock.delete_context(ContextId::try_from(context_id)?)
        .await?;
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
    let lock = app.read().await.interactive_list_context.clone();
    let res = lock
        .get_context(ContextId::try_from(context_id)?)
        .await?
        .get_tag_creation_suggestions(tag_name_id, input)
        .await?;
    Ok(res)
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
        let lock = app.read().await.interactive_list_context.clone();
        lock.get_context(ContextId::try_from(context_id)?)
            .await?
            .create_tag(metadata_id, name_id, value)
            .await?
    };
    let handle = app.read().await;
    // handle.emit(EntryChangedEvent {
    //     entry: entry.into(),
    // })?;
    // handle.emit(TagListChangedEvent {})?;
    Ok(())
}

#[tauri::command]
#[instrument(skip(app))]
pub async fn get_interactive_list_context_entries<'a>(
    context_id: String,
    filter: EntryViewFilter,
    app: AppArg<'a>,
) -> AppResult<Vec<EntryView>> {
    let lock = app.read().await.interactive_list_context.clone();
    let context = lock.get_context(ContextId::try_from(context_id)?).await?;
    let entries = context.get_entries(filter).await?;
    Ok(entries.into_iter().map(EntryView::from).collect())
}

#[tauri::command]
pub async fn import_folder<'a>(directory: String, app: AppArg<'a>) -> AppResult<String> {
    let file_loader = app.read().await.context.get_service::<FileLoader>().await?;

    LoadDirectory::new(file_loader.clone(), PathBuf::from(directory))?
        .load()
        .await?;
    Ok("ok".to_string())
}

#[tauri::command]
pub async fn get_tag_names<'a>(app: AppArg<'a>) -> AppResult<Vec<TagName>> {
    let tags = app
        .read()
        .await
        .context
        .get_service::<Database>()
        .await?
        .get_tag_names()
        .await?;
    Ok(tags)
}

#[tauri::command]
pub async fn get_tags_by_name<'a>(tag_name: String, app: AppArg<'a>) -> AppResult<Vec<TagView>> {
    let tags = app
        .read()
        .await
        .context
        .get_service::<Database>()
        .await?
        .get_tags_by_name(&tag_name)
        .await?;
    Ok(tags.into_iter().map(TagView::from).collect())
}

#[tauri::command]
pub async fn get_worker_queue_tasks<'a>(app: AppArg<'a>) -> AppResult<Vec<dto::WorkerQueueTask>> {
    let all_tasks = app
        .read()
        .await
        .context
        .get_service::<Database>()
        .await?
        .get_tasks()
        .await?;
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
        .read()
        .await
        .context
        .get_service::<Database>()
        .await?
        .get_task(task_id)
        .await?;
    Ok(dto::WorkerQueueTask {
        id: task.id,
        status: TaskStatus::try_from(task.status.as_str()).unwrap().into(),
    })
}

#[tauri::command]
pub async fn create_audio_deck<'a>(app: AppArg<'a>) -> AppResult<u32> {
    let (callback, receiver) = TaskCallback::new();
    app.read()
        .await
        .context
        .get_service::<AudioPlayerService>()
        .await?
        .send(AudioPlayerTask::CreateDeck { callback })
        .await?;
    Ok(receiver.wait().await.unwrap())
}

#[tauri::command]
pub async fn get_audio_decks<'a>(app: AppArg<'a>) -> AppResult<Vec<dto::DeckView>> {
    let (callback, receiver) = TaskCallback::new();
    app.read()
        .await
        .context
        .get_service::<AudioPlayerService>()
        .await?
        .send(AudioPlayerTask::GetDecks { callback })
        .await?;
    let decks = receiver.wait().await.unwrap();
    Ok(decks
        .into_iter()
        .map(|(id, deck)| {
            let file = dto::DeckFileView {
                metadata: deck.metadata.map(|m| m.into()),
                payload: deck.payload.map(|p| p.into()),
                status: deck.status.into(),
            };
            dto::DeckView {
                id,
                file: Some(file),
            }
        })
        .collect())
}

#[tauri::command]
pub async fn load_audio_track_from_metadata<'a>(
    deck_id: u32,
    metadata_id: i64,
    app: AppArg<'a>,
) -> AppResult<()> {
    app.read()
        .await
        .context
        .get_service::<AudioPlayerService>()
        .await?
        .send(AudioPlayerTask::LoadTrack {
            deck_id,
            target: TrackTarget::Metadata { metadata_id },
        })
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn load_audio_track_from_variation<'a>(
    deck_id: u32,
    file_variation_id: i64,
    app: AppArg<'a>,
) -> AppResult<()> {
    app.read()
        .await
        .context
        .get_service::<AudioPlayerService>()
        .await?
        .send(AudioPlayerTask::LoadTrack {
            deck_id,
            target: TrackTarget::FileVariation { file_variation_id },
        })
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn set_deck_file_status<'a>(
    deck_id: u32,
    status: dto::DeckFileStatus,
    app: AppArg<'a>,
) -> AppResult<()> {
    let (callback, receiver) = TaskCallback::new();
    let paused = match status {
        dto::DeckFileStatus::Paused { .. } => true,
        dto::DeckFileStatus::Playing { .. } => false,
        _ => {
            error!("Invalid deck file status");
            return Err(eyre::eyre!("Invalid deck file status").into());
        }
    };
    let offset = match status {
        dto::DeckFileStatus::Paused { offset, .. } => offset,
        dto::DeckFileStatus::Playing { offset, .. } => offset,
        _ => {
            error!("Invalid deck file status");
            return Err(eyre::eyre!("Invalid deck file status").into());
        }
    };
    app.read()
        .await
        .context
        .get_service::<AudioPlayerService>()
        .await?
        .send(AudioPlayerTask::SetDeckFileStatus {
            deck_id,
            paused,
            offset,
            callback,
        })
        .await?;
    receiver.wait().await.unwrap();
    Ok(())
}

#[tauri::command]
pub async fn extract_stems<'a>(metadata_id: i64, app: AppArg<'a>) -> AppResult<()> {
    let task = TaskPayload::FileAnalysis(FileAnalysisTask { metadata_id });
    app.read()
        .await
        .context
        .get_service::<Worker>()
        .await?
        .send(WorkerTask::Schedule(task))
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn get_file_variations_for_metadata<'a>(
    metadata_id: i64,
    app: AppArg<'a>,
) -> AppResult<Vec<dto::FileVariation>> {
    let database = app.read().await.context.get_service::<Database>().await?;
    let file = database.get_file_from_metadata_id(metadata_id).await?;
    let variations = database.get_file_variations(file.id).await?;
    Ok(variations
        .into_iter()
        .map(dto::FileVariation::from)
        .collect())
}
