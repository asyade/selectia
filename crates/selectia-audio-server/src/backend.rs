use cpal::{FromSample, Sample, SizedSample};
use cpal::traits::*;

use crate::prelude::*;

pub struct Backend {}

impl Backend {
    pub fn new(device: &cpal::Device, config: cpal::SupportedStreamConfig) -> AudioServerResult<Self> {
        match config.sample_format() {
            cpal::SampleFormat::I8 => run::<i8>(device, &config.into()),
            cpal::SampleFormat::I16 => run::<i16>(device, &config.into()),
            // cpal::SampleFormat::I24 => run::<I24>(device, &config.into()),
            cpal::SampleFormat::I32 => run::<i32>(device, &config.into()),
            // cpal::SampleFormat::I48 => run::<I48>(device, &config.into()),
            cpal::SampleFormat::I64 => run::<i64>(device, &config.into()),
            cpal::SampleFormat::U8 => run::<u8>(device, &config.into()),
            cpal::SampleFormat::U16 => run::<u16>(device, &config.into()),
            // cpal::SampleFormat::U24 => run::<U24>(device, &config.into()),
            cpal::SampleFormat::U32 => run::<u32>(device, &config.into()),
            // cpal::SampleFormat::U48 => run::<U48>(device, &config.into()),
            cpal::SampleFormat::U64 => run::<u64>(device, &config.into()),
            cpal::SampleFormat::F32 => run::<f32>(device, &config.into()),
            cpal::SampleFormat::F64 => run::<f64>(device, &config.into()),
            sample_format => Err(AudioServerError::UnsupportedSampleFormat { format: sample_format }),
        }?;
        
        Ok(Self {})
    }
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> AudioServerResult<()>
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
            dbg!("data");
            write_data(data, channels, &mut next_value)
        },
        err_fn,
        None,
    )?;
    stream.play()?;

    std::thread::sleep(std::time::Duration::from_millis(10000));

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