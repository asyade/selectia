use crate::prelude::*;

pub struct AudioServer {
}

pub type AudioServerService = AddressableService<FileLoaderTask>;

pub fn file_loader(state_machine: StateMachine) -> AudioServerService {
    AddressableService::new(move |receiver, _| async move {

        Ok(())
    })
}

impl AudioServer {
    pub fn new() {
        
    }
}