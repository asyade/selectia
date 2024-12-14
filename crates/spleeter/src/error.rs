#[derive(thiserror::Error, Debug)]
pub enum SpleeterError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("Model not found")]
    ModelNotFound,
    #[error("Cannot load session")]
    SessionLoadError,
    #[error("Malformated models index file")]
    MalformatedModelsIndex,
    #[error("Cannot process audio: {0}")]
    ProcessAudioError(&'static str),
}

pub type SpleeterResult<T> = Result<T, SpleeterError>;
