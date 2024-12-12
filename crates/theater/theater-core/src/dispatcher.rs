//! Responsible of dispatching events from a service to its listeners.
//! Basically a multiple receiver signle sender channel where event are cloned and sent to all listeners.

use std::marker::PhantomData;

use crate::prelude::*;

#[derive(Clone)]
pub struct EventDispatcher<T> {
    dispatcher: sync::mpsc::Sender<T>,
    listeners: Arc<RwLock<Vec<sync::mpsc::Sender<T>>>>,
}

impl<T: Event> EventDispatcher<T> {
    pub fn new() -> Self {
        let (dispatcher, proxy_recv) = sync::mpsc::channel(4096);
        let listeners = Arc::new(RwLock::new(vec![]));
        let _background_handle = tokio::spawn(Self::proxy(proxy_recv, listeners.clone()));
        Self {
            dispatcher,
            listeners,
        }
    }

    pub async fn dispatch(&self, event: T) -> TheaterResult<()> {
        self.dispatcher
            .send(event)
            .await
            .map_err(|_| TheaterError::ServiceNotAlive)?;
        Ok(())
    }

    pub fn dispatch_blocking(&self, event: T) -> TheaterResult<()> {
        self.dispatcher
            .blocking_send(event)
            .map_err(|_| TheaterError::ServiceNotAlive)?;
        Ok(())
    }

    async fn proxy(
        mut receiver: sync::mpsc::Receiver<T>,
        listeners: Arc<RwLock<Vec<sync::mpsc::Sender<T>>>>,
    ) -> TheaterResult<()> {
        while let Some(event) = receiver.recv().await {
            let listeners = listeners.read().await;
            if listeners.is_empty() {
                trace!("Dispatched event has no listeners");
            }
            for listener in listeners.iter() {
                if let Err(e) = listener.send(event.clone()).await {
                    warn!("Failed to send event to listener: {}", e);
                }
            }
        }
        Ok(())
    }

    pub async fn register(&self, listener: impl Into<sync::mpsc::Sender<T>>) {
        self.listeners.write().await.push(listener.into());
    }
}


/// Helper function to handle events dispatched by a dispatcher
pub fn channel_iterator<IT: Task, F: FnMut(IT) -> () + Send + 'static>(
    mut f: F,
) -> tokio::sync::mpsc::Sender<IT> {
    let (sender, mut receiver) = tokio::sync::mpsc::channel(4096);
    let sender_clone = sender.clone();
    tokio::spawn(async move {
        while let Some(event) = receiver.recv().await {
            f(event);
        }
    });
    sender_clone
}


/// Helper function to handle events dispatched by a dispatcher, similar to `channel_iterator` but with async handlers
pub fn async_channel_iterator<
    IT: Task,
    F: FnMut(IT) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
>(
    mut f: F,
) -> tokio::sync::mpsc::Sender<IT> {
    let (sender, mut receiver) = tokio::sync::mpsc::channel(4096);
    let sender_clone = sender.clone();
    tokio::spawn(async move {
        while let Some(event) = receiver.recv().await {
            f(event).await;
        }
    });
    sender_clone
}
