// Allow unused imports to silence warnings for early development
#![allow(unused_imports)]

use std::clone;

use interactive_list_context::InteractiveListContext;
use selectia::database::models::TagName;
use selectia::database::views::{
    entry_view::{EntryView, EntryViewFilter},
    TagView,
};
use settings::Settings;
use tauri::{Manager, State};

use crate::commands::*;
use crate::prelude::*;

pub mod app;
pub use selectia_tauri_dto as dto;
pub mod commands;
pub mod error;
pub mod prelude;

mod settings;
mod context;
