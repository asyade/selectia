#![allow(unused_variables)]
use symphonia::core::audio::Channels;

use crate::prelude::*;

#[derive(Debug, Clone)]
pub enum AudioServerTask {
    EnqueueAudioBuffer{
        samples: Vec<f32>,
        samples_rate: u32,
        channels: Channels,
    },
}

pub type AudioServerService = AddressableService<AudioServerTask>;

pub fn audio_server(state_machine: StateMachine) -> AudioServerService {
    AddressableService::new(move |mut receiver, _| async move {
        while let Some(task) = receiver.recv().await {
            match task {
                AudioServerTask::EnqueueAudioBuffer { samples, samples_rate, channels } => {
                    info!("EnqueueAudioBuffer");
                }
            }
        }
        Ok(())
    })
}

impl Task for AudioServerTask {
}
