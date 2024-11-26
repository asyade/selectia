use crate::prelude::*;

use worker::tasks::TaskStatus;

#[derive(Serialize, Clone)]
pub struct WorkerQueueTaskCreatedEvent {
    pub task: WorkerQueueTask,
}

#[derive(Serialize, Clone)]
pub struct WorkerQueueTaskUpdatedEvent {
    pub id: i64,
    pub task: Option<WorkerQueueTask>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct WorkerQueueTask {
    pub id: i64,
    pub status: TaskStatus,
}

pub struct DeckView {
    pub file: Option<DeckFileView>,
    pub id: u32,
}

pub struct DeckFileView {
    pub title: String,
    pub length: f32,
    pub offset: f32,
}
