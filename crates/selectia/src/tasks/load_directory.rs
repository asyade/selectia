use crate::prelude::*;

pub struct LoadDirectory {
    file_loader: FileLoader,
    directory: PathBuf,
}

impl LoadDirectory {
    pub fn new(file_loader: FileLoader, directory: PathBuf) -> Result<Self> {
        if !directory.exists() {
            return Err(eyre!("Directory does not exist"));
        }
        Ok(Self { file_loader, directory })
    }

    pub async fn load(&self) -> Result<()> {
        let mut to_be_processed = VecDeque::new();
        to_be_processed.push_back(self.directory.clone());
        while let Some(path) = to_be_processed.pop_front() {
            info!(path = ?path, "Loading directory ...");
            let mut entries = fs::read_dir(path).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_dir() {
                    to_be_processed.push_back(path);
                } else if path.is_audio_file() {
                    if let Err(e) = self.file_loader.send(file_loader::FileLoaderTask::LoadFile(path.to_path_buf())).await {
                        error!(path = ?path, error = ?e, "Error loading file");
                    }
                }
            }
        }
        Ok(())
    }
}
