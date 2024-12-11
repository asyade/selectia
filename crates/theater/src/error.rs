
use thiserror::Error;
#[derive(Debug, Error)]
pub enum TheaterError {
    #[error("The requested service is not registered")]
    ServiceNotRegistered,
    #[error("The requested service exist but does not match the requested type")]
    ServiceTypeMismatch,
    #[error("The callback has already been resolved")]
    CallbackAlreadyResolved,
    #[error("The callback sender has been dropped")]
    CallbackSenderDropped,
    #[error("The callback owner has dropped the callback without resolving it")]
    CallbackOwnerDropped,
    #[error("Service not alive")]
    ServiceNotAlive,
}

pub type TheaterResult<T> = Result<T, TheaterError>;