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
pub enum Events {
    AudioDeckCreated(AudioDeckCreatedEvent),
    AudioDeckUpdated(AudioDeckUpdatedEvent),
    WorkerQueueTaskCreated(WorkerQueueTaskCreatedEvent),
    WorkerQueueTaskUpdated(WorkerQueueTaskUpdatedEvent),
}
