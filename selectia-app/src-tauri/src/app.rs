use interactive_list_context::InteractiveListContext;
use tauri::State;

use crate::prelude::*;
use std::sync::RwLock;

#[derive(Clone)]
pub struct App {
    pub(crate) database: Database,
    pub(crate) state_machine: StateMachine,
    pub(crate) file_loader: FileLoader,
    pub(crate) interactive_list_context: ContextProvider<InteractiveListContext>,
}

pub struct AppState(pub(crate) Arc<RwLock<App>>);

pub type AppArg<'a> = State<'a, AppState>;
