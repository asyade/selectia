use models::Task as TaskModel;
use tasks::{BackgroundTask, TaskPayload, TaskStatus};

use crate::prelude::*;

pub mod tasks;

const DEFAULT_WORKER_POOL_SIZE: usize = 1;

#[derive(Clone, Debug)]
pub enum WorkerTask {
    TaskDone { id: i64, success: bool },
    Poll,
    Schedule(TaskPayload),
}

#[derive(Clone, Debug)]
pub enum WorkerEvent {
    QueueTaskCreated {
        id: i64,
        status: TaskStatus,
    },
    QueueTaskUpdated {
        id: i64,
        status: TaskStatus,
        removed: bool,
    },
}

#[singleton_service(Worker)]
pub async fn worker(
    ctx: ServiceContext,
    mut rx: ServiceReceiver<WorkerTask>,
    dispatcher: EventDispatcher<WorkerEvent>,
) -> Result<()> {
    let database = ctx.get_singleton::<Database>().await?;
    let introspect_address = ctx.get_singleton_address::<Worker>().await?;
    let mut pool = WorkerPool::new(
        DEFAULT_WORKER_POOL_SIZE,
        introspect_address.clone(),
        dispatcher.clone(),
    );

    // Sanitize task status to ensure that no task is in processing status due to a crash
    let sanitized = database.sanitize_task_status().await?;
    if sanitized > 0 {
        warn!(
            "{} task(s) were in processing status and have been reset to queued",
            sanitized
        );
    }

    // Send a poll message to the dispatcher to wake up the main loop and check if there is a task store
    introspect_address.send(WorkerTask::Poll).await?;

    while let Some(task) = rx.recv().await {
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
                dispatcher
                    .dispatch(WorkerEvent::QueueTaskUpdated {
                        id,
                        status: TaskStatus::Done,
                        removed: true,
                    })
                    .await?;
            }
            WorkerTask::Schedule(task) => match database.enqueue_task(task).await {
                Ok(id) => {
                    dispatcher
                        .dispatch(WorkerEvent::QueueTaskCreated {
                            id,
                            status: TaskStatus::Queued,
                        })
                        .await?;
                }
                Err(e) => {
                    error!("Error enqueuing task: {:?}", e);
                }
            },
            WorkerTask::Poll => {}
        }

        while pool.has_empty_slots() {
            if let Some(task) = database.dequeue_task().await? {
                dispatcher
                    .dispatch(WorkerEvent::QueueTaskUpdated {
                        id: task.id,
                        status: TaskStatus::Processing,
                        removed: false,
                    })
                    .await?;
                if let Err(e) = pool
                    .spawn(
                        task,
                        ctx.clone(),
                    )
                    .await
                {
                    error!("Error spawning task: {:?}", e);
                }
            } else {
                info!("No task to process, waiting for a worker to finish");
                break;
            }
        }
    }
    Ok(())
}

#[allow(dead_code)]
struct WorkerPool {
    max_size: usize,
    notify: AddressableService<WorkerTask>,
    dispatcher: EventDispatcher<WorkerEvent>,
    background_handles: HashMap<i64, (tokio::task::JoinHandle<Result<()>>, BackgroundTask)>,
}

impl WorkerPool {
    pub fn new(
        nbr_worker: usize,
        notify: AddressableService<WorkerTask>,
        dispatcher: EventDispatcher<WorkerEvent>,
    ) -> Self {
        Self {
            max_size: nbr_worker,
            notify,
            dispatcher,
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

    pub async fn spawn(&mut self, task: TaskModel, context: ServiceContext) -> Result<()> {
        info!("Worker spawning task: {:?}", task);
        let task = BackgroundTask::try_from(task)?;
        let handle = tokio::spawn(process_task(context, task.clone(), self.notify.clone()));
        self.background_handles.insert(task.id, (handle, task));
        Ok(())
    }

    pub fn has_empty_slots(&self) -> bool {
        self.background_handles.len() < self.max_size
    }
}

async fn process_task(
    context: ServiceContext,
    task: BackgroundTask,
    notify: AddressableService<WorkerTask>,
) -> Result<()> {
    info!("Processing task: {:?}", task);
    if let Err(e) = task.process(context).await {
        error!("Error processing task: {:?}", e);
        notify
            .send(WorkerTask::TaskDone {
                id: task.id,
                success: false,
            })
            .await?;
    } else {
        notify
            .send(WorkerTask::TaskDone {
                id: task.id,
                success: true,
            })
            .await?;
    }
    Ok(())
}

impl Task for WorkerTask {}

impl Task for WorkerEvent {}

pub trait WorkerDatabaseExt {
    fn enqueue_task(&self, task: TaskPayload) -> impl Future<Output = Result<i64>> + Send;
}

impl WorkerDatabaseExt for Database {
    async fn enqueue_task(&self, task: TaskPayload) -> Result<i64> {
        let payload = serde_json::to_string(&task)?;
        self.create_task(payload).await
    }
}
