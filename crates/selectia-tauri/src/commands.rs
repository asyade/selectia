use std::collections::BTreeMap;

use audio_player::{AudioPlayer, AudioPlayerService, AudioPlayerTask, TrackTarget};
use dto::{EntryChangedEvent, EntryListChangedEvent, TagListChangedEvent};
use interactive_list_context::InteractiveListContext;
use selectia::database::views::TagView;
use tauri::{AppHandle, Emitter, State};
use worker::{
    tasks::{FileAnalysisTask, StemExtractionTask, TaskPayload, TaskStatus},
    Worker, WorkerTask,
};

use crate::prelude::*;

#[tauri::command]
#[instrument(skip(app, provider))]
pub async fn interactive_list_create_context<'a>(
    app: AppArg<'a>,
    provider: State<'_, ContextProvider<InteractiveListContext>>,
) -> AppResult<String> {
    let context: InteractiveListContext = InteractiveListContext::new(&*app).await;
    let id = provider.create_context(context).await;
    info!(
        context_id = id.to_string(),
        "Created interactive list context"
    );
    Ok(id.to_string())
}

#[tauri::command]
#[instrument(skip(provider))]
pub async fn interactive_list_delete_context(
    context_id: String,
    provider: State<'_, ContextProvider<InteractiveListContext>>,
) -> AppResult<()> {
    info!(context_id = context_id, "Deleting interactive list context");
    provider
        .delete_context(ContextId::try_from(context_id)?)
        .await?;
    Ok(())
}

#[tauri::command]
#[instrument(skip(provider))]
pub async fn interactive_list_get_tag_creation_suggestions(
    context_id: String,
    tag_name_id: i64,
    input: String,
    provider: State<'_, ContextProvider<InteractiveListContext>>,
) -> AppResult<Vec<String>> {
    Ok(provider
        .get_context(ContextId::try_from(context_id)?)
        .await?
        .get_tag_creation_suggestions(tag_name_id, input)
        .await?)
}

#[tauri::command]
#[instrument(skip(provider))]
pub async fn interactive_list_create_tag<'a>(
    context_id: String,
    metadata_id: i64,
    name_id: i64,
    value: String,
    provider: State<'_, ContextProvider<InteractiveListContext>>,
) -> AppResult<()> {
    provider
        .get_context(ContextId::try_from(context_id)?)
        .await?
        .create_tag(metadata_id, name_id, value)
        .await?;
    Ok(())
}

#[tauri::command]
#[instrument(skip(provider))]
pub async fn get_interactive_list_context_entries(
    context_id: String,
    filter: EntryViewFilter,
    provider: State<'_, ContextProvider<InteractiveListContext>>,
) -> AppResult<Vec<EntryView>> {
    let context = provider
        .get_context(ContextId::try_from(context_id)?)
        .await?;
    let entries = context.get_entries(filter).await?;
    Ok(entries.into_iter().map(EntryView::from).collect())
}

#[tauri::command]
pub async fn import_folder(
    directory: String,
    file_loader: State<'_, AddressableService<FileLoaderTask>>,
) -> AppResult<String> {
    LoadDirectory::new({ &*file_loader }.clone(), PathBuf::from(directory))?
        .load()
        .await?;
    Ok("ok".to_string())
}

#[tauri::command]
pub async fn get_tag_names(database: State<'_, Database>) -> AppResult<Vec<TagName>> {
    let tags = database.get_tag_names().await?;
    Ok(tags)
}

#[tauri::command]
pub async fn get_tags_by_name(
    tag_name: String,
    database: State<'_, Database>,
) -> AppResult<Vec<TagView>> {
    let tags = database.get_tags_by_name(&tag_name).await?;
    Ok(tags.into_iter().map(TagView::from).collect())
}

#[tauri::command]
pub async fn get_worker_queue_tasks(
    database: State<'_, Database>,
) -> AppResult<Vec<dto::WorkerQueueTask>> {
    let all_tasks = database.get_tasks().await?;
    Ok(all_tasks
        .into_iter()
        .map(|t| dto::WorkerQueueTask {
            id: t.id,
            status: TaskStatus::try_from(t.status.as_str()).unwrap().into(),
        })
        .collect())
}

#[tauri::command]
pub async fn get_worker_queue_task(
    task_id: i64,
    database: State<'_, Database>,
) -> AppResult<dto::WorkerQueueTask> {
    let task = database.get_task(task_id).await?;
    Ok(dto::WorkerQueueTask {
        id: task.id,
        status: TaskStatus::try_from(task.status.as_str()).unwrap().into(),
    })
}

#[tauri::command]
pub async fn create_audio_deck(
    audio_player_service: State<'_, AddressableService<AudioPlayerTask>>,
) -> AppResult<u32> {
    let (callback, receiver) = TaskCallback::new();
    audio_player_service
        .send(AudioPlayerTask::CreateDeck { callback })
        .await?;
    Ok(receiver.wait().await.unwrap())
}

#[tauri::command]
pub async fn get_audio_decks(
    audio_player_service: State<'_, AddressableService<AudioPlayerTask>>,
) -> AppResult<Vec<dto::DeckView>> {
    let (callback, receiver) = TaskCallback::new();
    audio_player_service
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
pub async fn load_audio_track_from_metadata(
    deck_id: u32,
    metadata_id: i64,
    audio_player_service: State<'_, AddressableService<AudioPlayerTask>>,
) -> AppResult<()> {
    audio_player_service
        .send(AudioPlayerTask::LoadTrack {
            deck_id,
            target: TrackTarget::Metadata { metadata_id },
        })
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn load_audio_track_from_variation(
    deck_id: u32,
    file_variation_id: i64,
    audio_player_service: State<'_, AddressableService<AudioPlayerTask>>,
) -> AppResult<()> {
    audio_player_service
        .send(AudioPlayerTask::LoadTrack {
            deck_id,
            target: TrackTarget::FileVariation { file_variation_id },
        })
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn set_deck_file_status(
    deck_id: u32,
    status: dto::DeckFileStatus,
    audio_player_service: State<'_, AddressableService<AudioPlayerTask>>,
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
    audio_player_service
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
pub async fn extract_stems(
    metadata_id: i64,
    worker: State<'_, AddressableService<WorkerTask>>,
) -> AppResult<()> {
    let task = TaskPayload::FileAnalysis(FileAnalysisTask { metadata_id });
    worker.send(WorkerTask::Schedule(task)).await?;
    Ok(())
}

#[tauri::command]
pub async fn get_file_variations_for_metadata(
    metadata_id: i64,
    database: State<'_, Database>,
) -> AppResult<Vec<dto::FileVariation>> {
    let file = database.get_file_from_metadata_id(metadata_id).await?;
    let variations = database.get_file_variations(file.id).await?;
    Ok(variations
        .into_iter()
        .map(dto::FileVariation::from)
        .collect())
}
