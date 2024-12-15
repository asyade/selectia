use crate::prelude::*;
use image::ImageError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WavisionError {

    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("export error: {0}")]
    Export(ImageError),
}

pub type WavisionResult<T> = Result<T, WavisionError>;
