use crate::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioServerError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    DevicesError(#[from] cpal::DevicesError),
    #[error(transparent)]
    StreamError(#[from] cpal::StreamError),
    #[error(transparent)]
    ConfigError(#[from] cpal::SupportedStreamConfigsError),
    #[error("Host not available")]
    HostUnavailable,
    #[error(transparent)]
    DefaultConfigError(#[from] cpal::DefaultStreamConfigError),
    #[error(transparent)]
    BuildStreamError(#[from] cpal::BuildStreamError),
    #[error(transparent)]
    PlayStreamError(#[from] cpal::PlayStreamError),
    #[error("Unsupported sample format")]
    UnsupportedSampleFormat {
        format: cpal::SampleFormat,
    },
}

pub type AudioServerResult<T> = Result<T, AudioServerError>;
