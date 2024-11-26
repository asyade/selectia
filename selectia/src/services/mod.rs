use crate::prelude::*;
use tokio::net::unix::pipe::Sender;

pub use addresable_service::*;
pub use addresable_service_with_dispatcher::{AddressableServiceWithDispatcher, dispatcher::*};
pub use threaded_service::*;

pub mod embedding;
pub mod file_loader;
pub mod state_machine;
pub mod worker;
pub mod audio_server;

pub trait Service<T> {
    fn blocking_send(&self, message: T) -> Result<()>;
    fn send(&self, message: T) -> impl Future<Output = Result<()>> + Send;
    fn join(&self) -> impl Future<Output = Result<()>> + Send;
}

pub trait ChannelService<T> : Service<T> {
    fn sender(&self) -> &sync::mpsc::Sender<T>;
}

pub trait CancelableTask: Sized + Send + Clone + 'static {
    fn cancel() -> Self;
}

mod addresable_service {
    use crate::prelude::*;

    #[derive(Clone)]
    pub struct AddressableService<T> {
        pub(super) sender: sync::mpsc::Sender<T>,
        pub(super) background_handle: Arc<Mutex<Option<task::JoinHandle<Result<()>>>>>,
    }

    impl<T: CancelableTask> Service<T> for AddressableService<T> {
        async fn send(&self, message: T) -> Result<()> {
            self.sender
                .send(message)
                .await
                .map_err(|_| eyre!("Failed to send message"))
        }

        fn blocking_send(&self, message: T) -> Result<()> {
            self.sender.blocking_send(message).map_err(|_| eyre!("Failed to send message"))
        }

        async fn join(&self) -> Result<()> {
            self.sender
                .send(T::cancel())
                .await
                .map_err(|_| eyre!("Failed to send cancel message"))?;
            self.background_handle
                .lock()
                .await
                .take()
                .unwrap()
                .await??;
            Ok(())
        }
    }

    impl<T: CancelableTask> ChannelService<T> for AddressableService<T> {
        fn sender(&self) -> &sync::mpsc::Sender<T> {
            &self.sender
        }
    }

    impl<T: CancelableTask> AddressableService<T> {
        pub fn new<Fut, F>(background_task: F) -> Self
        where
            Fut: Future<Output = Result<()>> + Send + 'static,
            F: FnOnce(sync::mpsc::Receiver<T>, sync::mpsc::Sender<T>) -> Fut,
        {
            let (sender, receiver) = sync::mpsc::channel(4096);
            let background_handle = tokio::spawn(background_task(receiver, sender.clone()));
            Self {
                sender,
                background_handle: Arc::new(Mutex::new(Some(background_handle))),
            }
        }
    }
}

mod addresable_service_with_dispatcher {
    use super::addresable_service::AddressableService;
    use crate::prelude::*;
    use dispatcher::EventDispatcher;

    #[derive(Clone)]
    pub struct AddressableServiceWithDispatcher<T, R> {
        service: AddressableService<T>,
        dispatcher: EventDispatcher<R>,
    }

    impl<T: CancelableTask, R: CancelableTask> AddressableServiceWithDispatcher<T, R> {
        pub fn new<Fut, F>(background_task: F) -> Self
        where
            Fut: Future<Output = Result<()>> + Send + 'static,
            F: FnOnce(sync::mpsc::Receiver<T>, sync::mpsc::Sender<T>, EventDispatcher<R>) -> Fut,
        {
            let dispatcher = EventDispatcher::new();
            let (sender, receiver) = sync::mpsc::channel(4096);
            let background_handle = tokio::spawn(background_task(receiver, sender.clone(), dispatcher.clone()));
            Self {
                dispatcher,
                service: AddressableService {
                    sender,
                    background_handle: Arc::new(Mutex::new(Some(background_handle))),
                },
            }
        }

        pub async fn register_channel(&self, listener: sync::mpsc::Sender<R>) {
            self.dispatcher.register(listener).await;
        }
    }

    impl<T: CancelableTask, R: CancelableTask> ChannelService<T> for AddressableServiceWithDispatcher<T, R> {
        fn sender(&self) -> &sync::mpsc::Sender<T> {
            &self.service.sender
        }
    }

    impl<T: CancelableTask + Send + 'static, R: CancelableTask> Service<T>
        for AddressableServiceWithDispatcher<T, R>
    {
        async fn send(&self, message: T) -> Result<()> {
            self.service.send(message).await
        }

        fn blocking_send(&self, message: T) -> Result<()> {
            self.service.sender.blocking_send(message).map_err(|_| eyre!("Failed to send message"))
        }

        async fn join(&self) -> Result<()> {
            self.dispatcher
                .background_handle
                .lock()
                .await
                .take()
                .unwrap()
                .await??;
            self.service.join().await
        }
    }

    pub (super)mod dispatcher {
        use crate::prelude::*;

        #[derive(Clone)]
        pub struct EventDispatcher<T> {
            dispatcher: sync::mpsc::Sender<T>,
            listeners: Arc<RwLock<Vec<sync::mpsc::Sender<T>>>>,
            pub (super)background_handle: Arc<Mutex<Option<task::JoinHandle<Result<()>>>>>,
        }

        impl<T: CancelableTask> EventDispatcher<T> {
            pub fn new() -> Self {
                let (dispatcher, proxy_recv) = sync::mpsc::channel(4096);
                let listeners = Arc::new(RwLock::new(vec![]));
                let background_handle = tokio::spawn(Self::proxy(proxy_recv, listeners.clone()));
                Self {
                    dispatcher,
                    listeners,
                    background_handle: Arc::new(Mutex::new(Some(background_handle))),
                }
            }

            pub async fn dispatch(&self, event: T) -> Result<()> {
                self.dispatcher
                    .send(event)
                    .await
                    .map_err(|_| eyre!("Failed to send event to dispatcher"))
            }

            async fn proxy(
                mut receiver: sync::mpsc::Receiver<T>,
                listeners: Arc<RwLock<Vec<sync::mpsc::Sender<T>>>>,
            ) -> Result<()> {
                while let Some(event) = receiver.recv().await {
                    let listeners = listeners.read().await;
                    for listener in listeners.iter() {
                        listener
                            .send(event.clone())
                            .await
                            .map_err(|_| eyre!("Failed to send event to listener"))?;
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
    use crate::prelude::*;

    #[derive(Clone)]
    pub struct ThreadedService<T> {
        pub(super) sender: sync::mpsc::Sender<T>,
        pub(super) background_handle: Arc<Mutex<Option<std::thread::JoinHandle<Result<()>>>>>,
    }

    impl<T: CancelableTask> ThreadedService<T> {
        pub fn new<F>(background_task: F) -> Self
        where
            F: FnOnce(sync::mpsc::Receiver<T>) -> Result<()> + Send + 'static,
        {
            let (sender, receiver) = sync::mpsc::channel(4096);
            let background_handle = std::thread::spawn(move || background_task(receiver));
            Self {
                sender,
                background_handle: Arc::new(Mutex::new(Some(background_handle))),
            }
        }
    }

    impl<T: CancelableTask> ChannelService<T> for ThreadedService<T> {
        fn sender(&self) -> &sync::mpsc::Sender<T> {
            &self.sender
        }
    }

    impl<T: CancelableTask> Service<T> for ThreadedService<T> {
        async fn send(&self, message: T) -> Result<()> {
            self.sender
                .send(message)
                .await
                .map_err(|_| eyre!("Failed to send message"))
        }

        fn blocking_send(&self, message: T) -> Result<()> {
            self.sender.blocking_send(message).map_err(|_| eyre!("Failed to send message"))
        }

        async fn join(&self) -> Result<()> {
            self.sender.send(T::cancel()).await.map_err(|_| eyre!("Failed to send cancel message"))?;
            self.background_handle
                .lock()
                .await
                .take()
                .unwrap()
                .join()
                .map_err(|_| eyre!("Failed to join background thread"))??;
            Ok(())
        }
    }
}


pub fn channel_iterator<
    IT: CancelableTask,
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