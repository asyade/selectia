use crate::prelude::*;

#[derive(Debug)]
pub struct HostSpec {
    pub id: cpal::HostId,
    pub is_default: bool,
    pub default_input: Option<InputDeviceSpec>,
    pub default_output: Option<OutputDeviceSpec>,
    pub input_devices: Vec<InputDeviceSpec>,
    pub output_devices: Vec<OutputDeviceSpec>,
}

pub struct DeviceSpec {
    pub name: String,
    pub device: cpal::Device,
}

pub struct InputDeviceSpec {
    pub spec: DeviceSpec,
    default_config: cpal::SupportedStreamConfig,
    supported_configs: Vec<cpal::SupportedStreamConfigRange>,
}

pub struct OutputDeviceSpec {
    pub spec: DeviceSpec,
    pub default_config: cpal::SupportedStreamConfig,
    pub supported_configs: Vec<cpal::SupportedStreamConfigRange>,
}

impl HostSpec {
    pub fn evaluate(host_id: cpal::HostId, default_host_id: cpal::HostId) -> AudioServerResult<Self> {
        let host = cpal::host_from_id(host_id).map_err(|_| AudioServerError::HostUnavailable)?;
        let input_devices = host.input_devices()?.collect::<Vec<_>>();
        let output_devices = host.output_devices()?.collect::<Vec<_>>();

        let default_input = host.default_input_device().and_then(|d| {
            InputDeviceSpec::new(d)
                .map_err(|e| error!("failed to load default input device: {}", e))
                .ok()
        });

        let default_output = host.default_output_device().and_then(|d| {
            OutputDeviceSpec::new(d)
                .map_err(|e| error!("failed to load default output device: {}", e))
                .ok()
        });

        Ok(Self {
            id: host_id,
            is_default: host_id == default_host_id,
            default_input,
            default_output,
            input_devices: input_devices
                .into_iter()
                .filter_map(|d| {
                    InputDeviceSpec::new(d)
                        .map_err(|e| {
                            error!("Failed to get input device: {}", e);
                        })
                        .ok()
                })
                .collect(),
            output_devices: output_devices
                .into_iter()
                .filter_map(|d| {
                    OutputDeviceSpec::new(d)
                        .map_err(|e| {
                            error!("Failed to get output device: {}", e);
                        })
                        .ok()
                })
                .collect(),
        })
    }
}

impl DeviceSpec {
    pub fn new(device: cpal::Device) -> Self {
        let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
        Self { name, device }
    }
}

impl InputDeviceSpec {
    pub fn new(device: cpal::Device) -> AudioServerResult<Self> {
        let spec = DeviceSpec::new(device);

        let default_config = spec.device.default_input_config()?;
        let supported_configs = spec.device.supported_input_configs()?;

        let mut configs = Vec::new();
        for config in supported_configs {
            dbg!(config);
            configs.push(config);
        }

        Ok(Self {
            spec,
            default_config,
            supported_configs: configs,
        })
    }
}

impl OutputDeviceSpec {
    pub fn new(device: cpal::Device) -> AudioServerResult<Self> {
        let spec = DeviceSpec::new(device);
        let supported_configs = spec.device.supported_output_configs()?;
        let default_config = spec.device.default_output_config()?;
        let mut configs = Vec::new();
        for config in supported_configs {
            dbg!(config);
            configs.push(config);
        }
        Ok(Self {
            spec,
            default_config,
            supported_configs: configs,
        })
    }
}
