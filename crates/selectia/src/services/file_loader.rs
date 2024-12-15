use views::entry_view::EntryView;

use crate::prelude::*;

const MAX_CONCURRENT_LOADS: usize = 4;

#[derive(Clone, Debug, Task)]
pub enum FileLoaderTask {
    LoadFile {
        /// Path to the file to load
        path: PathBuf,
        /// Callback that will be resolved with the metadata's id of the ingested file
        callback: Option<TaskCallback<i64>>,
    },
}

#[singleton_service(FileLoader)]
pub async fn file_loader(ctx: ServiceContext, rx: ServiceReceiver<FileLoaderTask>) -> Result<()> {
    let stream = futures::stream::unfold(rx, |mut recv| async move {
        recv.recv().await.map(|task| (task, recv))
    })
    .map(|file| async {
        match file {
            FileLoaderTask::LoadFile { path, callback } => {
                trace!(path = ?path, "Loading file ...");
                match loader::LoadedFile::new(path).await {
                    Ok(loaded_file) => {
                        let database = ctx.get_singleton::<Database>().await.expect("database service");
                        match ingest_file(database, &loaded_file.path, &loaded_file.hash).await {
                            Ok(metadata_id) => {
                                if let Some(callback) = callback {
                                    let _ = callback.resolve(metadata_id).await;
                                }
                            }
                            Err(e) => {
                                error!(error = ?e, "Error ingesting file");
                            }
                        }
                    }
                    Err(e) => {
                        error!(error = ?e, "Error loading file");
                    }
                }
                true
            }
        }
    })
    .buffer_unordered(MAX_CONCURRENT_LOADS);

    let mut stream = Box::pin(stream);
    while let Some(should_continue) = stream.as_mut().next().await {
        if !should_continue {
            break;
        }
    }
    Ok(())
}

async fn ingest_file(database: Database, path: &Path, hash: &str) -> Result<i64> {
    let (metadata, _created) = database
        .get_or_create_metadata(hash)
        .await?;
    let _file = database
        .create_or_replace_file(path, metadata.id)
        .await?;
    let directory = path
        .parent()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let _tag_id = database
        .set_metadata_tag_by_tag_name_id(metadata.id, TagName::DIRECTORY_ID, directory)
        .await?;
    let file_name = path
        .file_prefix()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let _tag_id = database
        .set_metadata_tag_by_tag_name_id(metadata.id, TagName::FILE_NAME_ID, file_name)
        .await?;

    Ok(metadata.id)
}

mod loader {
    use base64ct::{Base64, Encoding};
    use sha2::{Digest, Sha256};
    use tokio::io::{AsyncReadExt, AsyncSeekExt};

    use crate::prelude::*;

    pub struct LoadedFile {
        pub path: PathBuf,
        pub hash: String,
    }

    impl LoadedFile {
        #[instrument]
        pub async fn new(path: PathBuf) -> Result<Self> {
            const BLOCK_SIZE: u64 = 1024 * 128;

            let mut file = tokio::fs::File::open(&path).await?;
            let mut input_buffer = Vec::with_capacity(BLOCK_SIZE as usize);

            let metadata = file.metadata().await?;
            let size = metadata.len();

            let mut hasher = Sha256::new();

            hasher.update(&size.to_be_bytes());

            if size >= BLOCK_SIZE {
                input_buffer.resize(BLOCK_SIZE as usize, 0);
                file.read_exact(&mut input_buffer).await?;
                hasher.update(&input_buffer);

                if size >= 2 * BLOCK_SIZE {
                    file.seek(std::io::SeekFrom::End(-(BLOCK_SIZE as i64)))
                        .await?;
                    file.read_exact(&mut input_buffer).await?;
                    hasher.update(&input_buffer);
                }

                if size >= 4 * BLOCK_SIZE {
                    let offset = size / 2 - (BLOCK_SIZE / 2);
                    file.seek(std::io::SeekFrom::Start(offset)).await?;
                    file.read_exact(&mut input_buffer).await?;
                    hasher.update(&input_buffer);
                }
            } else {
                input_buffer.resize(size as usize, 0);
                file.read_exact(&mut input_buffer).await?;
                hasher.update(&input_buffer);
            }

            let hash = hasher.finalize();
            let hash = Base64::encode_string(&hash);
            trace!(path = ?path, hash, "File hash computed");
            Ok(Self { path, hash })
        }
    }
}
