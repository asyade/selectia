use std::thread;

use crate::{analyser::file_analyser::FileAnalyser, prelude::*};
use chrono::{DateTime, Utc};
use eyre::bail;
use models::Task;

#[derive(Clone, Debug)]
pub struct BackgroundTask {
    pub id: i64,
    pub status: TaskStatus,
    pub payload: TaskPayload,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Queued,
    Processing,
    Done,
}

impl TryFrom<&str> for TaskStatus {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "queued" => Ok(TaskStatus::Queued),
            "processing" => Ok(TaskStatus::Processing),
            "done" => Ok(TaskStatus::Done),
            _ => bail!("Invalid task status: {}", value),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum TaskPayload {
    FileAnalysis(FileAnalysisTask),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub struct FileAnalysisTask {
    pub metadata_id: i64,
}

impl TryFrom<Task> for BackgroundTask {
    type Error = eyre::Error;

    fn try_from(task: Task) -> Result<Self> {
        Ok(Self {
            id: task.id,
            status: TaskStatus::try_from(task.status.as_str())?,
            payload: serde_json::from_str(&task.payload)?,
        })
    }
}

impl BackgroundTask {
    pub async fn process(&self, database: Database) -> Result<()> {
        match &self.payload {
            TaskPayload::FileAnalysis(task) => task.process(database).await,
        }
    }
}

impl FileAnalysisTask {
    pub async fn process(&self, database: Database) -> Result<()> {
        info!("Processing file analysis task: {:?}", self);
        let file = database.get_file_from_metadata_id(self.metadata_id).await?;
        let analyser = FileAnalyser::new(PathBuf::from(file.path));

        let (tx, rx) = tokio::sync::oneshot::channel();
        let _thread = thread::spawn(move || {
            let result = analyser.analyse();
            tx.send(result).unwrap();
        });
        let _result = rx.await?.unwrap();
        Ok(())
    }
}
