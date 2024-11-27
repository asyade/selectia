use crate::prelude::*;
use selectia::services::worker::tasks::TaskStatus as SelectiaTaskStatus;

#[derive(TS)]
#[ts(export)]
#[derive(Serialize, Clone)]
pub struct AudioDeckCreatedEvent {
    pub id: u32,
}

#[derive(TS)]
#[ts(export)]
#[derive(Serialize, Clone)]
pub struct AudioDeckUpdatedEvent {
    pub id: u32,
    pub file: Option<DeckFileView>,
}

#[derive(TS)]
#[ts(export)]
#[derive(Serialize, Clone)]
pub struct WorkerQueueTaskCreatedEvent {
    pub task: WorkerQueueTask,
}


#[derive(TS)]
#[ts(export)]
#[derive(Serialize, Clone)]
pub struct WorkerQueueTaskUpdatedEvent {
    pub id: i64,
    pub task: Option<WorkerQueueTask>,
}

#[derive(TS)]
#[ts(export)]
#[derive(Deserialize, Serialize, Clone)]
pub struct WorkerQueueTask {
    pub id: i64,
    pub status: TaskStatus,
}

#[derive(TS)]
#[ts(export)]
#[derive(Deserialize, Serialize, Clone)]
pub enum TaskStatus {
    Queued,
    Processing,
    Done,
}

#[derive(TS)]
#[ts(export)]
#[derive(Serialize, Clone)]
pub struct DeckView {
    pub file: Option<DeckFileView>,
    pub id: u32,
}

#[derive(TS)]
#[ts(export)]
#[derive(Serialize, Clone)]
pub struct DeckFileView {
    pub title: String,
    pub length: f32,
    pub offset: f32,
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