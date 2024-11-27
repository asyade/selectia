#![allow(unused_variables)]
use crate::prelude::*;

pub struct AudioServer {
}

#[derive(Debug, Clone)]
pub enum AudioServerTask {
}


pub type AudioServerService = AddressableService<AudioServerTask>;

pub fn audio_server(state_machine: StateMachine) -> AudioServerService {
    AddressableService::new(move |receiver, _| async move {
        Ok(())
    })
}

impl AudioServer {
    pub fn new() {
    }
}

impl Task for AudioServerTask {
}
