use crate::prelude::*;
use crate::services::{CancelableTask, AddressableService};

const MAX_CONCURRENT_LOADS: usize = 4;

#[derive(Clone, Debug)]
pub enum FileLoaderTask {
    LoadFile(PathBuf),
    Exit,
}

pub type FileLoader = AddressableService<FileLoaderTask>;

pub fn file_loader(state_machine: StateMachine) -> FileLoader {
    AddressableService::new(move |receiver, _| file_loader_task(state_machine, receiver))
}

async fn file_loader_task(state_machine: StateMachine, receiver: sync::mpsc::Receiver<FileLoaderTask>) -> Result<()> {
    let stream = futures::stream::unfold(receiver, |mut recv| async move {
        recv.recv().await.map(|task| (task, recv))
    }).map(|file| async {
        match file {
            FileLoaderTask::LoadFile(path) => {
                trace!(path = ?path, "Loading file ...");
                match loader::LoadedFile::new(path).await {
                    Ok(loaded_file) => {
                        if let Err(e) = state_machine.send(StateMachineTask::ingest_file(loaded_file.path, loaded_file.hash)).await {
                            error!(error = ?e, "Error sending file to state machine");
                        }
                    }
                    Err(e) => {
                        error!(error = ?e, "Error loading file");
                    }
                }
                true
            }
            FileLoaderTask::Exit => {
                false
            }
        }
    }).buffer_unordered(MAX_CONCURRENT_LOADS);

    let mut stream = Box::pin(stream);
    while let Some(should_continue) = stream.as_mut().next().await {
        if !should_continue {
            break;
        }
    }
    Ok(())
}

impl CancelableTask for FileLoaderTask {
    fn cancel() -> Self {
        Self::Exit
    }
}

mod loader {
    use sha2::{Digest, Sha256};
    use base64ct::{Base64, Encoding};
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
                    file.seek(std::io::SeekFrom::End(-(BLOCK_SIZE as i64))).await?;
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
            Ok(Self {
                path,
                hash,
            })
        }
    }
}
