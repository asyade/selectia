use crate::prelude::*;
use selectia::services::worker::tasks::TaskStatus as SelectiaTaskStatus;

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export_to = "models.ts")]
pub struct AppError {
    pub message: String,
    pub id: u32,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export_to = "models.ts")]
pub struct ContextId(i64);

#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export_to = "models.ts")]
pub struct WorkerQueueTask {
    pub id: i64,
    pub status: TaskStatus,
}

#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export_to = "models.ts")]
pub enum TaskStatus {
    Queued,
    Processing,
    Done,
}

#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export_to = "models.ts")]
pub struct DeckView {
    pub file: Option<DeckFileView>,
    pub id: u32,
}

#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export_to = "models.ts")]
#[serde(tag = "kind")]
pub enum DeckFileStatus {
    Loading {
        progress: u32,
    },
    Playing {
        offset: u32,
    },
    Paused {
        offset: u32,
    },
}

#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export_to = "models.ts")]
pub struct DeckFileView {
    pub metadata: Option<DeckFileMetadataSnapshot>,
    pub payload: Option<DeckFilePayloadSnapshot>,
    pub status: DeckFileStatus,
}

#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export_to = "models.ts")]
pub struct DeckFileMetadataSnapshot {
    pub title: String,
}

#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export_to = "models.ts")]
pub struct DeckFilePayloadSnapshot {
    pub duration: f64,
    pub sample_rate: u32,
    pub channels_count: usize,
    pub samples_count: usize,
    pub preview: Option<DeckFilePreview>,
}

#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export_to = "models.ts")]
pub struct DeckFilePreview {
    pub sample_rate: u32,
    pub original_sample_rate: u32,
    pub channels_count: usize,
    pub samples: Vec<f32>,
}

#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export_to = "models.ts")]
pub struct TagSelection {
    pub id: i64,
    pub value: String,
    pub selected: bool,
}

#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export_to = "models.ts")]
pub struct FilterSelection {
    pub directories: Vec<String>,
    pub tags: HashMap<i32, Vec<TagSelection>>,
}

#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export_to = "models.ts")]
pub struct EntryView {
    pub metadata_id: i64,
    pub metadata_hash: String,
    pub tags: Vec<MetadataTagView>,
}

#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export_to = "models.ts")]
pub struct MetadataTagView {
    pub tag_id: i64,
    pub metadata_tag_id: i64,
    pub tag_name_id: i64,
    pub tag_value: String,
    pub metadata_id: i64,
}

#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export_to = "models.ts")]
pub struct TagName {
    pub id: i64,
    pub name: String,
    pub use_for_filtering: bool,
}

#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export_to = "models.ts")]
pub struct TagView {
    pub id: i64,
    pub value: String,
    pub name_id: i64,
}

#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export_to = "models.ts")]
pub enum Models {
    DeckFileMetadataSnapshot(DeckFileMetadataSnapshot),
    DeckFilePayloadSnapshot(DeckFilePayloadSnapshot),
    DeckFileStatus(DeckFileStatus),
    AppError(AppError),
    ContextId(ContextId),
    WorkerQueueTask(WorkerQueueTask),
    TaskStatus(TaskStatus),
    DeckView(DeckView),
    DeckFileView(DeckFileView),
    TagSelection(TagSelection),
    FilterSelection(FilterSelection),
    EntryView(EntryView),
    MetadataTagView(MetadataTagView),
    TagName(TagName),
    TagView(TagView),
}

impl From<SelectiaTaskStatus> for TaskStatus {
    fn from(status: SelectiaTaskStatus) -> Self {
        match status {
            SelectiaTaskStatus::Queued => TaskStatus::Queued,
            SelectiaTaskStatus::Processing => TaskStatus::Processing,
            SelectiaTaskStatus::Done => TaskStatus::Done,
        }
    }
}

impl From<selectia::database::views::entry_view::EntryView> for EntryView {
    fn from(entry: selectia::database::views::entry_view::EntryView) -> Self {
        EntryView {
            metadata_id: entry.metadata_id,
            metadata_hash: entry.metadata_hash,
            tags: entry.tags.0.into_iter().map(|e| e.into()).collect(),
        }
    }
}

impl From<selectia::database::views::entry_view::MetadataTagView> for MetadataTagView {
    fn from(tag: selectia::database::views::entry_view::MetadataTagView) -> Self {
        MetadataTagView { tag_id: tag.tag_id, metadata_tag_id: tag.metadata_tag_id, tag_name_id: tag.tag_name_id, tag_value: tag.tag_value, metadata_id: tag.metadata_id }
    }
}

impl From<selectia::services::audio_player::DeckFileStatus> for DeckFileStatus {
    fn from(status: selectia::services::audio_player::DeckFileStatus) -> Self {
        match status {
            selectia::services::audio_player::DeckFileStatus::Loading { progress } => DeckFileStatus::Loading { progress },
            selectia::services::audio_player::DeckFileStatus::Playing { offset } => DeckFileStatus::Playing { offset },
            selectia::services::audio_player::DeckFileStatus::Paused { offset } => DeckFileStatus::Paused { offset },
        }
    }
}

impl Into<selectia::services::audio_player::DeckFileStatus> for DeckFileStatus {
    fn into(self) -> selectia::services::audio_player::DeckFileStatus {
        match self {
            DeckFileStatus::Loading { progress } => selectia::services::audio_player::DeckFileStatus::Loading { progress },
            DeckFileStatus::Playing { offset } => selectia::services::audio_player::DeckFileStatus::Playing { offset },
            DeckFileStatus::Paused { offset } => selectia::services::audio_player::DeckFileStatus::Paused { offset },
        }
    }
}

impl From<selectia::services::audio_player::DeckFilePayloadSnapshot> for DeckFilePayloadSnapshot {
    fn from(payload: selectia::services::audio_player::DeckFilePayloadSnapshot) -> Self {
        DeckFilePayloadSnapshot { duration: payload.duration, sample_rate: payload.sample_rate, channels_count: payload.channels_count, samples_count: payload.samples_count, preview: payload.preview.map(|e| e.into()) }
    }
}

impl From<selectia::services::audio_player::DeckFileMetadataSnapshot> for DeckFileMetadataSnapshot {
    fn from(metadata: selectia::services::audio_player::DeckFileMetadataSnapshot) -> Self {
        DeckFileMetadataSnapshot { title: metadata.title }
    }
}

impl From<selectia::services::audio_player::DeckFilePreview> for DeckFilePreview {
    fn from(preview: selectia::services::audio_player::DeckFilePreview) -> Self {
        DeckFilePreview { sample_rate: preview.sample_rate, original_sample_rate: preview.original_sample_rate, channels_count: preview.channels_count, samples: preview.samples }
    }
}


