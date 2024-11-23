#![allow(unused)]
pub (crate) use eyre::Result;
pub (crate) use thiserror::Error;
pub (crate) use std::path::{Path, PathBuf};
pub (crate) use eyre::eyre;

pub (crate) use crate::database::*;
pub (crate) use crate::services::*;
pub (crate) use tokio::{fs, sync, task};
pub (crate) use std::collections::{HashMap, HashSet, VecDeque};
pub (crate) use tokio::sync::{RwLock, Mutex};
pub (crate) use std::sync::Arc;
pub (crate) use tracing::*;
pub (crate) use std::sync::atomic::{AtomicBool, AtomicUsize};
pub (crate) use serde::{Serialize, Deserialize};
pub use crate::ext::*;
pub use crate::tasks::load_directory::LoadDirectory;
pub use crate::services::file_loader::{file_loader, FileLoader, FileLoaderTask};
pub use crate::services::state_machine::{state_machine, StateMachine, StateMachineTask, IngestFileTask};
pub use crate::services::embedding::{embedding, Embedding};
pub use futures::{Stream, StreamExt, FutureExt, Future};

pub use crate::models::{Tag, TagName};