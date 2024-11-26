use std::env;
use std::fs::File;
use std::path::Path;

use tracing::instrument;
use std::path::PathBuf;

use selectia_audio_file::prelude::*;

#[derive(Debug)]
pub struct FileAnalyser {
    path: PathBuf,
}

#[derive(Debug)]
pub struct FileAnalyserResult {}

impl FileAnalyser {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    #[instrument]
    pub fn analyse(&self) -> eyre::Result<FileAnalyserResult> {
        let mut audio_file = AudioFile::open(&self.path)?;
        audio_file.decode()?;
        Ok(FileAnalyserResult {})
    }
}
