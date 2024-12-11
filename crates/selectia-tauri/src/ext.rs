use crate::prelude::*;
use selectia_tauri_dto::events::Events;
use tauri::{AppHandle, Emitter};

pub trait IdentifiedEventEmitter {
    fn emit_identified_event<T: Into<Events>>(&self, id: u32, event: T) -> eyre::Result<()>;
    fn emit_event<T: Into<Events>>(&self, event: T) -> eyre::Result<()>;
}

impl IdentifiedEventEmitter for AppHandle {
    fn emit_event<T: Into<Events>>(&self, event: T) -> eyre::Result<()> {
        let event: Events = event.into();
        self.emit(event.name(), event)?;
        Ok(())
    }

    fn emit_identified_event<T: Into<Events>>(&self, id: u32, event: T) -> eyre::Result<()> {
        let event: Events = event.into();
        self.emit(&format!("{}:{}", event.name(), id), event)?;
        Ok(())
    }
}