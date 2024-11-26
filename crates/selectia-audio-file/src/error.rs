use crate::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioFileError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Symphonia(#[from] symphonia::core::errors::Error),
}

pub type AudioFileResult<T> = Result<T, AudioFileError>;
