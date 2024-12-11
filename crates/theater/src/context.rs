use std::{any::{Any, TypeId}, collections::HashMap, future::Future, ops::Deref, sync::{Arc, MutexGuard, RwLockReadGuard}};

use tokio::sync::RwLock;

use crate::prelude::*;

