use models::File;

use crate::prelude::*;

pub type StateMachine = AddressableServiceWithDispatcher<StateMachineTask, StateMachineEvent>;

#[derive(Clone, Debug)]
pub enum StateMachineEvent {
    FileIngested {
        file: File,
        new: bool,
    },
}

impl Task for StateMachineEvent {
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct StateMachineTask {
    owner: TaskOwner,
    payload: StateMachineTaskPayload,
}

#[derive(Clone, Debug)]
pub enum TaskOwner {
    System,
    User,
}

#[derive(Clone, Debug)]
pub enum StateMachineTaskPayload {
    IngestFile(IngestFileTask),
    SetTag(SetTagTask),
}

#[derive(Clone)]
pub struct SetTagTask {
    /// Name of the tag used to identify the tag kind (i.e lookup in the `tag_name` table)
    pub name: String,
    /// Value of the tag
    pub value: String,
    /// Metadata id to bind the tag to (optional, a tag is not required to be bound to anything to exist)
    pub metadata_id: Option<i64>,
    pub callback: Arc<Mutex<Option<Box<dyn FnOnce() -> Result<()> + Send + Sync + 'static>>>>,
}

impl std::fmt::Debug for SetTagTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SetTagTask {{ name: {:?}, value: {:?}, metadata_id: {:?} }}",
            self.name, self.value, self.metadata_id
        )
    }
}

#[derive(Clone, Debug)]
pub struct IngestFileTask {
    pub path: PathBuf,
    pub hash: String,
}

pub async fn state_machine(ctx: TheaterContext) -> StateMachine {
    AddressableServiceWithDispatcher::new(ctx, move |ctx, receiver, _sender, dispatcher| async move{
        let database = ctx.get_service::<Database>().await?;
        state_machine_task(database, receiver, dispatcher).await
    }).await
}

async fn state_machine_task(
    database: Database,
    mut receiver: sync::mpsc::Receiver<StateMachineTask>,
    dispatcher: EventDispatcher<StateMachineEvent>,
) -> Result<()> {
    while let Some(task) = receiver.recv().await {
        match handle_task(database.clone(), task, dispatcher.clone()).await {
            Ok(true) => (),
            Ok(false) => break,
            Err(e) => error!("Error handling task: {}", e),
        }
    }
    Ok(())
}

#[instrument(skip(database, dispatcher))]
async fn handle_task(
    database: Database,
    task: StateMachineTask,
    dispatcher: EventDispatcher<StateMachineEvent>,
) -> Result<bool> {
    match task.payload {
        StateMachineTaskPayload::IngestFile(ingest_file_event) => {
            let (metadata, created) = database
                .get_or_create_metadata(&ingest_file_event.hash)
                .await?;
            let file = database
                .create_or_replace_file(&ingest_file_event.path, metadata.id)
                .await?;
            let directory = ingest_file_event
                .path
                .parent()
                .unwrap()
                .to_string_lossy()
                .to_string();
            let _tag_id = database
                .set_metadata_tag_by_tag_name_id(metadata.id, TagName::DIRECTORY_ID, directory)
                .await?;
            let file_name = ingest_file_event
                .path
                .file_prefix()
                .unwrap()
                .to_string_lossy()
                .to_string();
            let _tag_id = database
                .set_metadata_tag_by_tag_name_id(metadata.id, TagName::FILE_NAME_ID, file_name)
                .await?;
            
            dispatcher
                .dispatch(StateMachineEvent::FileIngested { file, new: created })
                .await?;
            Ok(true)
        }
        StateMachineTaskPayload::SetTag(set_tag_event) => {
            let tag_id = database
                .set_tag(&set_tag_event.name, set_tag_event.value)
                .await?;
            if let Some(metadata_id) = set_tag_event.metadata_id {
                database.set_metadata_tag(metadata_id, tag_id).await?;
            }
            info!(tag_id, "Tag set");
            Ok(true)
        }
    }
}

impl Task for StateMachineTask {
}

impl StateMachineTask {
    pub fn ingest_file(path: PathBuf, hash: String) -> Self {
        Self {
            owner: TaskOwner::User,
            payload: StateMachineTaskPayload::IngestFile(IngestFileTask { path, hash }),
        }
    }

    pub fn set_tag(name: String, value: String, metadata_id: Option<i64>) -> Self {
        Self {
            owner: TaskOwner::User,
            payload: StateMachineTaskPayload::SetTag(SetTagTask {
                name,
                value,
                metadata_id,
                callback: Arc::new(Mutex::new(None)),
            }),
        }
    }

    pub fn set_tag_with_callback<F>(name: String, value: String, metadata_id: Option<i64>, callback: F) -> Self
    where
        F: FnOnce() -> Result<()> + Send + Sync + 'static,
    {
        Self {
            owner: TaskOwner::User,
            payload: StateMachineTaskPayload::SetTag(SetTagTask {
                name,
                value,
                metadata_id,
                callback: Arc::new(Mutex::new(Some(Box::new(callback)))),
            }),
        }
    }
}
