// Allow unused imports to silence warnings for early development
#![allow(unused_imports)]

#![feature(path_file_prefix)]

use crate::prelude::*;

pub mod database;
pub mod prelude;
pub mod result;
pub mod services;
pub mod tasks;
pub mod ext;
pub mod analyser;

pub use selectia_audio_file as audio_file;
