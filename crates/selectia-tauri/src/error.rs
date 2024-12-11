use selectia_tauri_dto::models::AppError as AppErrorDto;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::ipc::{InvokeError, InvokeResponseBody, IpcResponse};
use theater::error::TheaterError;
pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Theater(#[from] TheaterError),
    #[error(transparent)]
    Eyre(#[from] eyre::Error),
    #[error("Unhandled error: {0:?}")]
    Unhandled(Option<String>),
}

impl AppError {
    pub fn id(&self) -> u32 {
        match self {
            AppError::Unhandled(_) => 0,
            AppError::Eyre(_) => 1,
            AppError::Theater(_) => 2,
        }
    }
}

impl Into<AppErrorDto> for AppError {
    fn into(self) -> AppErrorDto {
        match self {
            AppError::Eyre(_) => AppErrorDto {
                message: self.to_string(),
                id: self.id(),
            },
            _ => AppErrorDto {
                message: self.to_string(),
                id: self.id(),
            },
        }
    }
}

impl From<String> for AppError {
    fn from(value: String) -> Self {
        AppError::Unhandled(Some(value))
    }
}

impl Into<InvokeError> for AppError {
    fn into(self) -> InvokeError {
        let error: AppErrorDto = self.into();
        let serialized = serde_json::to_value(&error)
            .unwrap_or_else(|_| json!({"id": 0, "message": "Failed to serialize error"}));
        InvokeError(serialized)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "result")]
pub enum SerializedAppResult<T> {
    Err(AppErrorDto),
    Ok(T),
}
