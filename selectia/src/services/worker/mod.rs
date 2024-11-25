use models::Task;
use tasks::{BackgroundTask, TaskPayload};

use crate::prelude::*;
use crate::services::{CancelableTask, AddressableService};

pub mod tasks;

const DEFAULT_WORKER_POOL_SIZE: usize = 4;

#[derive(Clone, Debug)]
pub enum WorkerTask {
    TaskDone {
        id: i64,
        success: bool,
    },
    Schedule(TaskPayload),
    Exit,
}

#[derive(Clone, Debug)]
pub enum WorkerEvent {
    QueueUpdated,
    /// TODO: refactor event dispatcher to remove this
    Exit,
}

pub type Worker = AddressableServiceWithDispatcher<WorkerTask, WorkerEvent>;

pub fn worker(database: Database) -> Worker {
    AddressableServiceWithDispatcher::new(move |receiver, sender, dispatcher| worker_task(database, receiver, sender, dispatcher))
}

async fn worker_task(database: Database, mut receiver: sync::mpsc::Receiver<WorkerTask>, sender: sync::mpsc::Sender<WorkerTask>, dispatcher: EventDispatcher<WorkerEvent>) -> Result<()> {
    let mut pool = WorkerPool::new(DEFAULT_WORKER_POOL_SIZE, sender);
    
    while let Some(task) = receiver.recv().await {
        info!("Worker received task: {:?}", task);
        match task {
            WorkerTask::TaskDone { id, success } => {
                pool.done(id).await;
                if success {
                    info!("Task done: {}", id);
                } else {
                    error!("Task failed: {}", id);
                }
                database.delete_task(id).await?;
            }
            WorkerTask::Schedule(task) => {
                if let Err(e) = database.enqueue_task(task).await {
                    error!("Error enqueuing task: {:?}", e);
                }
                // dispatcher.dispatch(WorkerEvent::QueueUpdated).await?;
            }
            WorkerTask::Exit => {
                pool.join(true).await?;
                break;
            }
        }

        if pool.has_empty_slots() {
            if let Some(task) = database.dequeue_task().await? {
                if let Err(e) = pool.spawn(task, database.clone()).await {
                    error!("Error spawning task: {:?}", e);
                }
            } else  {
                info!("No task to process, waiting for a worker to finish");
            }
            // dispatcher.dispatch(WorkerEvent::QueueUpdated).await?;
        }
    }
    Ok(())
}


struct WorkerPool {
    max_size: usize,
    notify: sync::mpsc::Sender<WorkerTask>,
    background_handles: HashMap<i64, (tokio::task::JoinHandle<Result<()>>, BackgroundTask)>,
}

impl WorkerPool {
    pub fn new(nbr_worker: usize, notify: sync::mpsc::Sender<WorkerTask>) -> Self {
        Self {
            max_size: nbr_worker,
            notify,
            background_handles: HashMap::new(),
        }
    }

    pub async fn done(&mut self, id: i64) {
        match self.background_handles.remove(&id) {
            Some((handle, task)) => {
                if let Err(e) = handle.await {
                    //TODO: retry task here based on task.retries or similar
                    error!(task_id = task.id, "Task failed: {:?}", e);
                }
            }
            None => {
                error!("Worker pool: task not found: {}", id);
            }
        };
    }

    pub async fn spawn(&mut self, task: Task, database: Database) -> Result<()> {
        info!("Worker spawning task: {:?}", task);
        let task = BackgroundTask::try_from(task)?;
        let handle = tokio::spawn(process_task(database, task.clone(), self.notify.clone()));
        self.background_handles.insert(task.id, (handle, task));
        Ok(())
    }

    pub fn has_empty_slots(&self) -> bool {
        self.background_handles.len() < self.max_size
    }

    /// TODO : better error handling
    pub async fn join(self, cancel: bool) -> Result<()> {
        if cancel {
            for (_, handle) in self.background_handles {
                handle.0.abort();
                let _ = handle.0.await;
            }
        } else {
            for (_, handle) in self.background_handles {
                let _ = handle.0.await;
            }
        }
        Ok(())
    }
}

async fn process_task(database: Database, task: BackgroundTask, notify: sync::mpsc::Sender<WorkerTask>) -> Result<()> {
    info!("Processing task: {:?}", task);
    if let Err(e) = task.process(database).await {
        error!("Error processing task: {:?}", e);
        notify.send(WorkerTask::TaskDone { id: task.id, success: false }).await?;
    } else {
        notify.send(WorkerTask::TaskDone { id: task.id, success: true }).await?;
    }
    Ok(())
}

impl CancelableTask for WorkerTask {
    fn cancel() -> Self {
        Self::Exit
    }
}

impl CancelableTask for WorkerEvent {
    fn cancel() -> Self {
        Self::Exit
    }
}

pub trait WorkerDatabaseExt {
    fn enqueue_task(&self, task: TaskPayload) -> impl Future<Output = Result<i64>> + Send;
}

impl WorkerDatabaseExt for Database {
    async fn enqueue_task(&self, task: TaskPayload) -> Result<i64> {
        let payload = serde_json::to_string(&task)?;
        self.create_task(payload).await
    }
}
