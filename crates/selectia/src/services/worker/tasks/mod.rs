use crate::prelude::*;
use eyre::bail;
use models::Task;

mod file_analysis_task;
mod stem_extraction_task;

pub use file_analysis_task::FileAnalysisTask;
pub use stem_extraction_task::StemExtractionTask;

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
    StemExtraction(StemExtractionTask),
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
    pub async fn process<T: ServiceHostContext>(&self, context: T) -> Result<()> {
        match &self.payload {
            TaskPayload::FileAnalysis(task) => task.process(&context).await,
            TaskPayload::StemExtraction(task) => task.process(&context).await,
        }
    }
}
