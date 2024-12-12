use crate::prelude::*;
use std::{fmt::Debug, marker::PhantomData, ops::Deref};

pub use addresable_service::*;

pub type ServiceSender<T> = sync::mpsc::Sender<T>;
pub type ServiceReceiver<T> = sync::mpsc::Receiver<T>;

pub trait Service {
    type Task: Task;

    fn blocking_send(&self, message: Self::Task) -> TheaterResult<()>;
    fn send(&self, message: Self::Task) -> impl Future<Output = TheaterResult<()>> + Send;

    fn spawn_entrypoint<
        Fut: Future<Output = Result<(), E>> + Send + 'static,
        E: Send + 'static + Debug,
    >(
        &self,
        ctx: ServiceContext,
        task: Fut,
    ) {
        let is_ready = ctx.is_ready();
        let _handle = tokio::task::spawn(async move {
            is_ready.await;
            if let Err(e) = task.await {
                error!("task failed: {:?}", e);
            }
            if let Err(e) = ctx.destroy().await {
                error!("failed to cleanup service context: {:?}", e);
            }
        });
    }
}

pub trait Task: Sized + Send + Clone + 'static {}

pub trait Event: Task + Clone + 'static {}

impl<T: Task + Clone + 'static> Event for T {}

/// A wraper around `EventDispatcher` that allows registration as a singleton and identification based on the service/event type.
pub struct SingletonServiceDispatcher<T: Task, R: Event, S: SingletonService<Task = T>> {
    dispatcher: EventDispatcher<R>,
    _phantom: PhantomData<(S, T)>,
}

impl<T: Task, R: Event, S: SingletonService<Task = T>> Into<EventDispatcher<R>>
    for SingletonServiceDispatcher<T, R, S>
{
    fn into(self) -> EventDispatcher<R> {
        self.dispatcher
    }
}

impl<T: Task, R: Event, S: SingletonService<Task = T>> SingletonServiceDispatcher<T, R, S> {
    pub fn new() -> Self {
        Self {
            dispatcher: EventDispatcher::new(),
            _phantom: PhantomData,
        }
    }
}

impl<T: Task, R: Event, S: SingletonService<Task = T>> Clone for SingletonServiceDispatcher<T, R, S> {
    fn clone(&self) -> Self {
        Self {
            dispatcher: self.dispatcher.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T: Task, R: Event, S: SingletonService<Task = T>> Deref for SingletonServiceDispatcher<T, R, S> {
    type Target = EventDispatcher<R>;

    fn deref(&self) -> &Self::Target {
        &self.dispatcher
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

    impl<T: Task> Service for AddressableService<T> {
        type Task = T;

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

    impl<T: Task> AddressableService<T> {
        pub async fn new<Fut, F, E>(ctx: &ServiceContext, background_task: F) -> Self
        where
            Fut: Future<Output = Result<(), E>> + Send + 'static,
            F: FnOnce(ServiceContext, sync::mpsc::Receiver<T>) -> Fut,
            E: Debug + Send + 'static,
        {
            let (sender, receiver) = sync::mpsc::channel(4096);
            let instance = Self {
                sender: sender.clone(),
            };
            let background_task = background_task(ctx.clone(), receiver);
            instance.spawn_entrypoint(ctx.clone(), background_task);
            instance
        }
    }

    impl <T: Task> Into<sync::mpsc::Sender<T>> for AddressableService<T> {
        fn into(self) -> sync::mpsc::Sender<T> {
            self.sender
        }
    }
}
