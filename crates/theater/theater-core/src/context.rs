use std::{
    any::{Any, TypeId},
    collections::HashMap,
    future::Future,
    ops::Deref,
    sync::{Arc, MutexGuard, RwLockReadGuard},
    task::Context,
};

use sync::{Mutex, OwnedRwLockWriteGuard, RwLock};

use crate::prelude::*;

pub trait ServiceHostContext: Clone + Send + Sync + 'static {
    fn is_ready(&self) -> impl Future<Output = bool> + Send + 'static;
    fn register_singleton<T: Any + Send + Sync>(
        &self,
        actor: T,
    ) -> impl Future<Output = TheaterResult<()>> + Send;
    fn get_singleton<T: Any + Clone + Send + Sync>(
        &self,
    ) -> impl Future<Output = TheaterResult<T>> + Send;

    fn map_singleton<T: Any + Send + Sync, F: FnOnce(&T) -> R, R>(
        &self,
        f: F,
    ) -> impl Future<Output = TheaterResult<R>>;

    fn get_singleton_address<Svc: Any + Send + Sync + SingletonService>(
        &self,
    ) -> impl Future<Output = TheaterResult<AddressableService<Svc::Task>>> {
        self.map_singleton::<Svc, _, _>(|svc| svc.address())
    }

    fn get_singleton_dispatcher<Svc: Any + Send + Sync + SingletonService, R: Event>(&self) -> impl Future<Output = TheaterResult<SingletonServiceDispatcher<Svc::Task, R, Svc>>> {
        self.get_singleton::<SingletonServiceDispatcher<Svc::Task, R, Svc>>()
    }
}

pub trait GlobalTheaterContext {
    fn as_global(&self) -> &TheaterContext;
}

pub trait SingletonService {
    type Task: Task + Send + Sync;
    fn address(&self) -> AddressableService<Self::Task>;
}

#[derive(Clone)]
pub struct OwnedTheaterContext {
    context: TheaterContext,
    lock: Arc<Mutex<Option<OwnedRwLockWriteGuard<TheaterStatus>>>>,
}

#[derive(Clone)]
pub struct TheaterContext {
    status: Arc<RwLock<TheaterStatus>>,
    actors: Arc<RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync + 'static>>>>,
}

#[derive(Clone)]
pub struct ServiceContext {
    components: Arc<Mutex<Vec<TypeId>>>,
    global: TheaterContext,
    singleton_type_id: TypeId,
}

pub enum TheaterStatus {
    Init,
    Playing,
}

impl OwnedTheaterContext {
    pub async fn new() -> Self {
        let context = TheaterContext {
            actors: Arc::new(RwLock::new(HashMap::new())),
            status: Arc::new(RwLock::new(TheaterStatus::Init)),
        };
        let lock = context.status.clone().write_owned().await;
        OwnedTheaterContext {
            context,
            lock: Arc::new(Mutex::new(Some(lock))),
        }
    }

    pub async fn ready(&self) {
        let _ = self.lock.lock().await.take().expect("ready called twice !");
    }
}

impl Deref for OwnedTheaterContext {
    type Target = TheaterContext;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl TheaterContext {
    #[instrument]
    pub(crate) async fn deregister_singleton(&self, actor: TypeId) {
        let found = {
            let mut lock = self.actors.write().await;
            lock.remove(&actor)
        };
        if found.is_some() {
            trace!("Singleton deregistered successfully");
        } else {
            error!("Unable to deregister non-existent singleton");
        }
    }
}

impl ServiceHostContext for TheaterContext {
    fn is_ready(&self) -> impl Future<Output = bool> + Send + 'static {
        let status = self.status.clone();
        Box::pin(async move {
            drop(status.read_owned().await);
            true
        })
    }

    #[instrument(skip(actor))]
    async fn register_singleton<T: Any + Send + Sync>(&self, actor: T) -> TheaterResult<()> {
        let mut lock = self.actors.write().await;

        match lock.entry(TypeId::of::<T>()) {
            std::collections::hash_map::Entry::Occupied(_entry) => {
                error!("Singleton already registered");
                return Err(TheaterError::ServiceAlreadyRegistered);
            }
            std::collections::hash_map::Entry::Vacant(vacant) => {
                vacant.insert(Box::new(actor));
                trace!("Singleton registered successfully");
            }
        }

        Ok(())
    }

    async fn get_singleton<T: Any + Clone + Send + Sync>(&self) -> TheaterResult<T> {
        self.actors
            .read()
            .await
            .get(&TypeId::of::<T>())
            .ok_or(TheaterError::ServiceNotRegistered)
            .and_then(|actor| {
                actor
                    .downcast_ref::<T>()
                    .ok_or(TheaterError::ServiceTypeMismatch)
            })
            .cloned()
    }

    async fn map_singleton<T: Any + Send + Sync, F: FnOnce(&T) -> R, R>(
        &self,
        f: F,
    ) -> TheaterResult<R> {
        self.actors
            .read()
            .await
            .get(&TypeId::of::<T>())
            .ok_or(TheaterError::ServiceNotRegistered)
            .and_then(|actor| {
                actor
                    .downcast_ref::<T>()
                    .ok_or(TheaterError::ServiceTypeMismatch)
            })
            .map(|actor| f(actor))
    }
}

impl ServiceContext {
    pub fn new<S: Any + Send + Sync>(global: TheaterContext) -> Self {
        let singleton_type_id = TypeId::of::<S>();
        Self {
            components: Arc::new(Mutex::new(vec![])),
            global,
            singleton_type_id,
        }
    }

    pub async fn destroy(&self) -> TheaterResult<()> {
        let components = {
            let mut lock = self.components.lock().await;
            lock.drain(..).collect::<Vec<_>>()
        };

        for component in components {
            self.global.deregister_singleton(component).await;
        }
        Ok(())
    }
}

impl GlobalTheaterContext for OwnedTheaterContext {
    fn as_global(&self) -> &TheaterContext {
        &self.context
    }
}

impl GlobalTheaterContext for TheaterContext {
    fn as_global(&self) -> &TheaterContext {
        self
    }
}

impl GlobalTheaterContext for ServiceContext {
    fn as_global(&self) -> &TheaterContext {
        &self.global
    }
}

impl ServiceHostContext for ServiceContext {
    /// TODO: Nothing guarantees that the service context is initialized before the service is spawned.
    ///       This may cause issues when spawning a service in an already running theater.
    ///       Actually i never triggered the issue.
    ///       I suspect that the code after the actual tokio::spawn() of the service task is always executed
    ///       before the actual task for because of tokio internals but nothing guarantes the behavior.
    fn is_ready(&self) -> impl Future<Output = bool> + Send + 'static {
        self.global.is_ready()
    }

    #[instrument(skip(actor))]
    async fn register_singleton<T: Any + Send + Sync>(&self, actor: T) -> TheaterResult<()> {
        let type_id = {
            let type_id = TypeId::of::<T>();
            let mut lock = self.components.lock().await;
            lock.push(type_id);
            type_id
        };
        if type_id == self.singleton_type_id {
            trace!(singleton_type_id=?type_id, "Singleton registered successfully as secondary singleton (component)");
        } else {
            trace!(singleton_type_id=?type_id, "Singleton registered successfully as the primary singleton (service)");
        }
        self.global.register_singleton(actor).await
    }

    fn get_singleton<T: Any + Clone + Send + Sync>(
        &self,
    ) -> impl Future<Output = TheaterResult<T>> + Send {
        self.global.get_singleton()
    }

    fn map_singleton<T: Any + Send + Sync, F: FnOnce(&T) -> R, R>(
        &self,
        f: F,
    ) -> impl Future<Output = TheaterResult<R>> {
        self.global.map_singleton(f)
    }
}

impl std::fmt::Debug for ServiceContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ServiceContext {{ singleton_type_id: {:?} }}", self.singleton_type_id)
    }
}

impl std::fmt::Display for ServiceContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ServiceContext(singleton_type_id: {:#?})", self.singleton_type_id)
    }
}

impl std::fmt::Debug for TheaterContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GlobalContext")
    }
}

impl std::fmt::Display for TheaterContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GlobalContext")
    }
}
