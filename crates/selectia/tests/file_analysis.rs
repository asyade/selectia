use std::path::PathBuf;

use selectia::{database::Database, prelude::*, test_utils::TmpDatabase};
use theater::prelude::*;
use worker::tasks::FileAnalysisTask;

const TEST_FILE_128_BPM: &str = "../../dataset/128-bpm.wav";

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
pub async fn test_file_analysis() {
    tracing_subscriber::fmt::init();

    let theater = OwnedTheaterContext::new().await;
    let database = TmpDatabase::new().await;

    theater.register_singleton(database.clone()).await.unwrap();
    let file_loader_addr = FileLoader::spawn(&theater).await.unwrap();
    let worker_addr = Worker::spawn(&theater).await.unwrap();

    theater.ready().await;


    let (callback, resolve) = TaskCallback::new();
    let ingest_task = FileLoaderTask::LoadFile {
        path: PathBuf::from(TEST_FILE_128_BPM),
        callback: Some(callback),
    };
    file_loader_addr.send(ingest_task).await.unwrap();
    let metadata_id = resolve.wait().await.unwrap();

    let file_analysis_task = FileAnalysisTask { metadata_id };
    worker_addr
        .send(WorkerTask::Schedule(
            worker::tasks::TaskPayload::FileAnalysis(file_analysis_task),
        ))
        .await
        .unwrap();

    drop(database);
}
