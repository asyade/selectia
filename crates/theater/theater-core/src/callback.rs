use crate::prelude::*;

#[derive(Clone)]
pub struct TaskCallback<T> {
    pub sender: Arc<RwLock<Option<sync::oneshot::Sender<T>>>>,
}

impl<T> std::fmt::Debug for TaskCallback<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TaskCallback")
    }
}

pub struct TaskCallbackReceiver<T> {
    pub receiver: sync::oneshot::Receiver<T>,
}

impl<T: Send + 'static> TaskCallback<T> {
    pub fn new() -> (Self, TaskCallbackReceiver<T>) {
        let (sender, receiver) = sync::oneshot::channel();
        (
            Self {
                sender: Arc::new(RwLock::new(Some(sender))),
            },
            TaskCallbackReceiver { receiver },
        )
    }

    pub async fn resolve(&self, value: T) -> TheaterResult<()> {
        let sender = {
            let mut lock = self.sender.write().await;
            lock.take().ok_or(TheaterError::CallbackAlreadyResolved)?
        };
        sender
            .send(value)
            .map_err(|_| TheaterError::CallbackSenderDropped)?;
        Ok(())
    }
}

impl<T: Send + 'static> TaskCallbackReceiver<T> {
    pub async fn wait(self) -> TheaterResult<T> {
        Ok(self
            .receiver
            .await
            .map_err(|_| TheaterError::CallbackOwnerDropped)?)
    }
}