use std::path::{Path, PathBuf};

const AUDIO_EXTENSIONS: [&str; 6] = ["mp3", "flac", "wav", "m4a", "aac", "ogg"];

pub trait PathExt {
    fn is_audio_file(&self) -> bool;
}

impl PathExt for Path {
    fn is_audio_file(&self) -> bool {
        self.extension().map_or(false, |ext| AUDIO_EXTENSIONS.contains(&ext.to_str().unwrap().to_lowercase().as_str()))
    }
}

impl PathExt for PathBuf {
    fn is_audio_file(&self) -> bool {
        self.as_path().is_audio_file()
    }
}
