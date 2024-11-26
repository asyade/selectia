#![allow(unused_imports)]

mod backend;
mod error;
mod prelude;
mod spec;

use prelude::*;

pub struct AudioServer {}

impl AudioServer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_hosts(&self) -> AudioServerResult<Vec<HostSpec>> {
        let default_host = cpal::default_host().id();
        let hosts = cpal::available_hosts();

        hosts
            .into_iter()
            .map(|h| HostSpec::evaluate(h, default_host))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use backend::Backend;
    use cpal::{Sample, SampleRate, SupportedStreamConfig};

    use super::*;

    #[test]
    fn it_works() {
        // let server = AudioServer::new();
        // let hosts = server.get_hosts().unwrap();
        // for host in hosts {
        //     for device in host.output_devices.iter() {
        //         let config = device.spec.device.default_output_config().unwrap();
        //         match Backend::new(&device.spec.device, config) {
        //             Ok(backend) => {
        //                 dbg!(device.spec.device.name());
        //                 dbg!("@@@@@@@@@@@@@@@@@@@@@");
        //             }
        //             Err(e) => {
        //                 dbg!("Error creating backend: {:?}", e);
        //             }
        //         }
        //     }
        // }

        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let config = device.default_output_config().unwrap();
        Backend::new(&device, config).unwrap();
    }
}

impl std::fmt::Debug for DeviceSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DeviceSpec {{ name: {} }}", self.name)
    }
}

impl std::fmt::Debug for InputDeviceSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InputDeviceSpec {{ spec: {:?} }}", self.spec)
    }
}

impl std::fmt::Debug for OutputDeviceSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OutputDeviceSpec {{ spec: {:?} }}", self.spec)
    }
}
