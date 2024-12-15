use crate::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioFileError {
    #[error("Audio separation failed")]
    AudioSeparationFailed,
    #[error("Empty container")]
    EmptyContainer,
    #[error("Unsupported sample format")]
    UnsupportedSampleFormat,
    #[error("No default track found")]
    NoDefaultTrack,
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Symphonia(#[from] symphonia::core::errors::Error),
    #[error(transparent)]
    Hound(#[from] hound::Error),
    #[error("Index out of bounds")]
    OutOfBounds,
    #[error("Invalid sample rate")]
    InvalidSampleRate,
    #[error("Invalid channel count, expected {expected}, got {got}")]
    InvalidChannelCount { expected: u32, got: u32 },
}

pub type AudioFileResult<T> = Result<T, AudioFileError>;
