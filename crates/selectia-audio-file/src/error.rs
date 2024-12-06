use crate::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioFileError {
    #[error("Unsupported sample format")]
    UnsupportedSampleFormat,
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Symphonia(#[from] symphonia::core::errors::Error),
    #[error(transparent)]
    Hound(#[from] hound::Error),
}

pub type AudioFileResult<T> = Result<T, AudioFileError>;
