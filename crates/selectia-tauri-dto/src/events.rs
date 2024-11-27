use crate::prelude::*;

#[derive(Serialize, Clone, TS)]
#[ts(export_to = "events.ts")]
pub struct AudioDeckCreatedEvent {
    pub id: u32,
}

#[derive(Serialize, Clone, TS)]
#[ts(export_to = "events.ts")]
pub struct AudioDeckUpdatedEvent {
    pub id: u32,
    pub file: Option<DeckFileView>,
}

#[derive(Serialize, Clone, TS)]
#[ts(export_to = "events.ts")]
pub struct WorkerQueueTaskCreatedEvent {
    pub task: WorkerQueueTask,
}


#[derive(Serialize, Clone, TS)]
#[ts(export_to = "events.ts")]
pub struct WorkerQueueTaskUpdatedEvent {
    pub id: i64,
    pub task: Option<WorkerQueueTask>,
}

#[derive(Serialize, Clone, TS)]
#[ts(export_to = "events.ts")]
pub struct TagListChangedEvent {}

#[derive(Serialize, Clone, TS)]
#[ts(export_to = "events.ts")]
pub struct EntryListChangedEvent {}

#[derive(Serialize, Clone, TS)]
#[ts(export_to = "events.ts")]
pub struct EntryChangedEvent {
    pub entry: EntryView,
}

#[derive(Serialize, Clone, TS, From)]
#[ts(export_to = "events.ts")]
#[serde(tag = "type")]
pub enum Events {
    AudioDeckCreated(AudioDeckCreatedEvent),
    AudioDeckUpdated(AudioDeckUpdatedEvent),
    WorkerQueueTaskCreated(WorkerQueueTaskCreatedEvent),
    WorkerQueueTaskUpdated(WorkerQueueTaskUpdatedEvent),
    TagListChanged(TagListChangedEvent),
    EntryChanged(EntryChangedEvent),
    EntryListChanged(EntryListChangedEvent),
}

impl Events {
    pub fn name(&self) -> &'static str {
        match self {
            Events::AudioDeckCreated(_) => "AudioDeckCreated",
            Events::AudioDeckUpdated(_) => "AudioDeckUpdated",
            Events::WorkerQueueTaskCreated(_) => "WorkerQueueTaskCreated",
            Events::WorkerQueueTaskUpdated(_) => "WorkerQueueTaskUpdated",
            Events::TagListChanged(_) => "TagListChanged",
            Events::EntryChanged(_) => "EntryChanged",
            Events::EntryListChanged(_) => "EntryListChanged",
        }
    }
}