use crate::prelude::*;
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
            status: dbg!(TaskStatus::try_from(task.status.as_str())?),
            payload: dbg!(serde_json::from_str(&task.payload)?),
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
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        Ok(())
    }
}
