use std::{
    any::{Any, TypeId},
    collections::HashMap,
    future::Future,
    ops::Deref,
    sync::{Arc, MutexGuard, RwLockReadGuard}, task::Context,
};

use sync::{Mutex, RwLock, OwnedRwLockWriteGuard};

use crate::prelude::*;

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
        OwnedTheaterContext { context, lock: Arc::new(Mutex::new(Some(lock))) }
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
    pub(crate) async fn is_ready(&self) -> bool {
        drop(self.status.read().await);
        true
    }

    pub async fn register_service<T: Any + Send + Sync>(&self, actor: T) {
        self.actors
            .write()
            .await
            .insert(TypeId::of::<T>(), Box::new(actor));
    }

    pub async fn get_service<T: Any + Clone + Send + Sync>(&self) -> TheaterResult<T> {
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
}
