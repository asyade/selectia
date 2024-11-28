#![allow(dead_code)]
use cpal::{traits::*, StreamConfig, SupportedBufferSize};
use cpal::{FromSample, Sample, SizedSample};

use crate::prelude::*;

pub type BufferSender = mpsc::Sender<Vec<f32>>;
pub type BufferReceiver = mpsc::Receiver<Vec<f32>>;

pub struct BackendHandle {
    buffer_sender: BufferSender,
    handle: tokio::task::JoinHandle<AudioServerResult<()>>,
}

pub struct Backend {
    buffer_receiver: BufferReceiver,
    config: cpal::SupportedStreamConfig,
    device: cpal::Device,
}

impl Backend {
    pub fn new(
        device: cpal::Device,
        config: cpal::SupportedStreamConfig,
    ) -> AudioServerResult<BackendHandle> {
        let (buffer_sender, buffer_receiver) = mpsc::channel::<Vec<f32>>(10);

        let backend = Self {
            buffer_receiver,
            config,
            device,
        };
        let handle = backend.spawn();
        Ok(BackendHandle {
            buffer_sender,
            handle,
        })
    }

    pub fn spawn(self) -> tokio::task::JoinHandle<AudioServerResult<()>> {
        tokio::task::spawn_blocking(move || {
            let device = &self.device;
            let config: StreamConfig = self.config.clone().into();
            match self.config.sample_format() {
                cpal::SampleFormat::I8 => run::<i8>(&device, &config, self.buffer_receiver),
                cpal::SampleFormat::I16 => run::<i16>(&device, &config, self.buffer_receiver),
                // cpal::SampleFormat::I24 => run::<I24>(device, &config.into()),
                cpal::SampleFormat::I32 => run::<i32>(&device, &config, self.buffer_receiver),
                // cpal::SampleFormat::I48 => run::<I48>(device, &config.into()),
                cpal::SampleFormat::I64 => run::<i64>(&device, &config, self.buffer_receiver),
                cpal::SampleFormat::U8 => run::<u8>(&device, &config, self.buffer_receiver),
                cpal::SampleFormat::U16 => run::<u16>(&device, &config, self.buffer_receiver),
                // cpal::SampleFormat::U24 => run::<U24>(device, &config.into()),
                cpal::SampleFormat::U32 => run::<u32>(&device, &config, self.buffer_receiver),
                // cpal::SampleFormat::U48 => run::<U48>(device, &config.into()),
                cpal::SampleFormat::U64 => run::<u64>(&device, &config, self.buffer_receiver),
                cpal::SampleFormat::F32 => run::<f32>(&device, &config, self.buffer_receiver),
                cpal::SampleFormat::F64 => run::<f64>(&device, &config, self.buffer_receiver),
                sample_format => Err(AudioServerError::UnsupportedSampleFormat {
                    format: sample_format,
                }),
            }
        })
    }
}

fn run<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut recv: BufferReceiver,
) -> AudioServerResult<()>
where
    T: SizedSample + FromSample<f32>,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    // Produce a sinusoid of maximum amplitude.
    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin()
    };

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            info!("writing data");
            write_data(data, channels, &mut next_value)
        },
        err_fn,
        None,
    )?;
    stream.play()?;

    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
    Ok(())
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let value: T = T::from_sample(next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
