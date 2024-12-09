use std::fmt::Debug;

use crate::prelude::*;
pub use addresable_service::*;
pub use addresable_service_with_dispatcher::{dispatcher::*, AddressableServiceWithDispatcher};
pub use threaded_service::*;
use tokio::task::JoinHandle;

pub type ServiceSender<T> = sync::mpsc::Sender<T>;
pub type ServiceReceiver<T> = sync::mpsc::Receiver<T>;

pub trait Service<T> {
    fn blocking_send(&self, message: T) -> TheaterResult<()>;
    fn send(&self, message: T) -> impl Future<Output = TheaterResult<()>> + Send;

    fn spawn_task<
        Fut: Future<Output = Result<(), E>> + Send + 'static,
        E: Send + 'static + Debug,
    >(
        &self,
        task: Fut,
    ) {
        let _handle = tokio::task::spawn(async move {
            if let Err(e) = task.await {
                error!("task failed: {:?}", e);
            }
        });
    }
}

pub trait ChannelService<T>: Service<T> {
    fn sender(&self) -> &sync::mpsc::Sender<T>;
}

pub trait Task: Sized + Send + 'static {}

pub trait Event: Task + Clone + 'static {}

impl<T: Task + Clone + 'static> Event for T {}

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

mod addresable_service {
    use std::fmt::Debug;

    use crate::prelude::*;

    #[allow(unused_variables)]
    #[derive(Clone)]
    pub struct AddressableService<T> {
        pub(super) sender: sync::mpsc::Sender<T>,
    }

    impl<T: Task> Service<T> for AddressableService<T> {
        async fn send(&self, message: T) -> TheaterResult<()> {
            self.sender
                .send(message)
                .await
                .map_err(|_| TheaterError::ServiceNotAlive)?;
            Ok(())
        }

        fn blocking_send(&self, message: T) -> TheaterResult<()> {
            self.sender
                .blocking_send(message)
                .map_err(|_| TheaterError::ServiceNotAlive)?;
            Ok(())
        }
    }

    impl<T: Task> ChannelService<T> for AddressableService<T> {
        fn sender(&self) -> &sync::mpsc::Sender<T> {
            &self.sender
        }
    }

    impl<T: Task> AddressableService<T> {
        pub fn new<Fut, F, E>(background_task: F) -> Self
        where
            Fut: Future<Output = Result<(), E>> + Send + 'static,
            F: FnOnce(sync::mpsc::Receiver<T>, sync::mpsc::Sender<T>) -> Fut,
            E: Debug + Send + 'static,
        {
            let (sender, receiver) = sync::mpsc::channel(4096);
            let instance = Self {
                sender: sender.clone(),
            };
            instance.spawn_task(background_task(receiver, sender.clone()));
            instance
        }
    }
}

mod addresable_service_with_dispatcher {
    use std::{error::Error, marker::PhantomData};

    use super::{addresable_service::AddressableService, Event};
    use crate::prelude::*;
    use dispatcher::EventDispatcher;

    #[derive(Clone)]
    pub struct AddressableServiceWithDispatcher<T: Task, R: Event> {
        service: AddressableService<T>,
        dispatcher: EventDispatcher<R>,
    }

    impl<T: Task, R: Event> AddressableServiceWithDispatcher<T, R> {
        pub fn new<Fut, F, E: std::fmt::Debug + Send + 'static>(background_task: F) -> Self
        where
            Fut: Future<Output = Result<(), E>> + Send + 'static,
            F: FnOnce(sync::mpsc::Receiver<T>, sync::mpsc::Sender<T>, EventDispatcher<R>) -> Fut,
        {
            let dispatcher = EventDispatcher::new();
            let (sender, receiver) = sync::mpsc::channel(4096);
            let background_task = background_task(receiver, sender.clone(), dispatcher.clone());
            let instance = Self {
                dispatcher,
                service: AddressableService { sender },
            };
            instance.spawn_task(background_task);
            instance
        }

        pub async fn register_channel(&self, listener: sync::mpsc::Sender<R>) {
            self.dispatcher.register(listener).await;
        }
    }

    impl<T: Task, R: Event> ChannelService<T> for AddressableServiceWithDispatcher<T, R> {
        fn sender(&self) -> &sync::mpsc::Sender<T> {
            &self.service.sender
        }
    }

    impl<T: Task, R: Event> Service<T> for AddressableServiceWithDispatcher<T, R> {
        async fn send(&self, message: T) -> TheaterResult<()> {
            self.service.send(message).await
        }

        fn blocking_send(&self, message: T) -> TheaterResult<()> {
            self.service
                .sender
                .blocking_send(message)
                .map_err(|_| TheaterError::ServiceNotAlive)?;
            Ok(())
        }
    }

    pub(super) mod dispatcher {
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
                    for listener in listeners.iter() {
                        if let Err(e) = listener.send(event.clone()).await {
                            warn!("Failed to send event to listener: {}", e);
                        }
                    }
                }
                Ok(())
            }

            pub async fn register(&self, listener: sync::mpsc::Sender<T>) {
                self.listeners.write().await.push(listener);
            }
        }
    }
}

mod threaded_service {
    use std::fmt::Debug;

    use crate::prelude::*;

    #[derive(Clone)]
    pub struct ThreadedService<T> {
        pub(super) sender: sync::mpsc::Sender<T>,
    }

    impl<T: Task> ThreadedService<T> {
        pub fn new<F, E>(background_task: F) -> Self
        where
            F: FnOnce(sync::mpsc::Receiver<T>) -> Result<(), E> + Send + 'static,
            E: Debug + Send + 'static,
        {
            let (sender, receiver) = sync::mpsc::channel(4096);
            let _background_handle = std::thread::spawn(move || {
                if let Err(e) = background_task(receiver) {
                    error!("background task failed: {:?}", e);
                }
            });
            Self { sender }
        }
    }

    impl<T: Task> ChannelService<T> for ThreadedService<T> {
        fn sender(&self) -> &sync::mpsc::Sender<T> {
            &self.sender
        }
    }

    impl<T: Task> Service<T> for ThreadedService<T> {
        async fn send(&self, message: T) -> TheaterResult<()> {
            self.sender
                .send(message)
                .await
                .map_err(|_| TheaterError::ServiceNotAlive)?;
            Ok(())
        }

        fn blocking_send(&self, message: T) -> TheaterResult<()> {
            self.sender
                .blocking_send(message)
                .map_err(|_| TheaterError::ServiceNotAlive)?;
            Ok(())
        }
    }
}

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
