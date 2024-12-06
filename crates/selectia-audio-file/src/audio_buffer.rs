use dasp::{sample::FromSample as DaspFromSample, signal::Signal};
use std::borrow::Cow;
use symphonia::core::{
    audio::{AudioBuffer, AudioBufferRef, Signal as SymphoniaSignal, SignalSpec},
    conv::{FromSample, IntoSample},
};

#[derive(Clone)]
pub enum AnySampleBuffer<'a> {
    S16(Cow<'a, InterleveadSampleBuffer<i16>>),
    S32(Cow<'a, InterleveadSampleBuffer<i32>>),
    F32(Cow<'a, InterleveadSampleBuffer<f32>>),
}

pub enum SampleFormat {
    S16,
    F32,
}

#[derive(Clone)]
pub struct InterleveadSampleBuffer<T> {
    pub buffer: Vec<T>,
    pub rate: u32,
    pub channels: u32,
}

impl<T: symphonia::core::sample::Sample> InterleveadSampleBuffer<T> {
    pub fn from_samples(rate: u32, channels: u32, samples: Vec<T>) -> Self {
        Self {
            buffer: samples,
            rate,
            channels,
        }
    }

    /// Copy the interleaved samples from the source buffer to the end of the sample buffer.
    pub fn append_interleaved_typed<F>(&mut self, src: &AudioBuffer<F>) -> usize
    where
        F: symphonia::core::sample::Sample + IntoSample<T>,
    {
        let n_channels = src.spec().channels.count();
        let n_samples = src.frames() * n_channels;

        // Ensure that the capacity of the sample buffer is greater than or equal to the number
        // of samples that will be copied from the source buffer.
        let base_offset = self.buffer.len();
        self.buffer.resize(self.buffer.len() + n_samples, T::MID);
        for ch in 0..n_channels {
            let ch_slice = src.chan(ch);
            let offset = ch + base_offset;
            for (dst, src) in self.buffer[offset..]
                .iter_mut()
                .step_by(n_channels)
                .zip(ch_slice)
            {
                *dst = (*src).into_sample();
            }
        }
        self.buffer.len() - base_offset
    }
}

impl<'a> AnySampleBuffer<'a> {
    pub fn new(rate: u32, channels: u32, format: SampleFormat) -> Self {
        match format {
            SampleFormat::F32 => Self::F32(Cow::Owned(InterleveadSampleBuffer::from_samples(
                rate,
                channels,
                vec![],
            ))),
            SampleFormat::S16 => Self::S16(Cow::Owned(InterleveadSampleBuffer::from_samples(
                rate,
                channels,
                vec![],
            ))),
        }
    }

    pub fn into_f32_buffer(self) -> InterleveadSampleBuffer<f32> {
        match self {
            Self::F32(buf) => buf.into_owned(),
            _ => todo!(),
        }
    }

    pub fn append_interleaved(&mut self, src: &AudioBufferRef) -> usize {
        match src {
            AudioBufferRef::F32(buf) => self.append_interleaved_typed(buf.as_ref()),
            AudioBufferRef::S16(buf) => self.append_interleaved_typed(buf.as_ref()),
            AudioBufferRef::S32(buf) => self.append_interleaved_typed(buf.as_ref()),
            _ => match src {
                AudioBufferRef::U8(buf) => panic!("Unsupported sample format: U8"),
                AudioBufferRef::U16(buf) => panic!("Unsupported sample format: U16"),
                AudioBufferRef::U24(buf) => panic!("Unsupported sample format: U24"),
                AudioBufferRef::U32(buf) => panic!("Unsupported sample format: U32"),
                AudioBufferRef::S8(buf) => panic!("Unsupported sample format: S8"),
                AudioBufferRef::S24(buf) => panic!("Unsupported sample format: S24"),
                _ => panic!("Unsupported sample format"),
            },
        }
    }

    pub fn append_interleaved_typed<F>(&mut self, src: &AudioBuffer<F>) -> usize
    where
        F: symphonia::core::sample::Sample + IntoSample<f32> + IntoSample<i16> + IntoSample<i32>,
    {
        match self {
            Self::S32(buf) => buf.to_mut().append_interleaved_typed(src),
            Self::F32(buf) => buf.to_mut().append_interleaved_typed(src),
            Self::S16(buf) => buf.to_mut().append_interleaved_typed(src),
        }
    }

    pub fn rate(&self) -> u32 {
        match self {
            Self::S32(buf) => buf.rate,
            Self::F32(buf) => buf.rate,
            Self::S16(buf) => buf.rate,
        }
    }

    pub fn channels(&self) -> u32 {
        match self {
            Self::S32(buf) => buf.channels,
            Self::F32(buf) => buf.channels,
            Self::S16(buf) => buf.channels,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::S32(buf) => buf.buffer.len(),
            Self::F32(buf) => buf.buffer.len(),
            Self::S16(buf) => buf.buffer.len(),
        }
    }
}

pub trait FromSamples<'a, T> {
    fn from_samples(rate: u32, channels: u32, samples: Vec<T>) -> AnySampleBuffer<'a>;
}

impl<'a> FromSamples<'a, f32> for AnySampleBuffer<'a> {
    fn from_samples(rate: u32, channels: u32, samples: Vec<f32>) -> AnySampleBuffer<'a> {
        AnySampleBuffer::F32(Cow::Owned(InterleveadSampleBuffer::from_samples(
            rate, channels, samples,
        )))
    }
}

impl<'a> FromSamples<'a, i16> for AnySampleBuffer<'a> {
    fn from_samples(rate: u32, channels: u32, samples: Vec<i16>) -> AnySampleBuffer<'a> {
        AnySampleBuffer::S16(Cow::Owned(InterleveadSampleBuffer::from_samples(
            rate, channels, samples,
        )))
    }
}

impl<'a> FromSamples<'a, i32> for AnySampleBuffer<'a> {
    fn from_samples(rate: u32, channels: u32, samples: Vec<i32>) -> AnySampleBuffer<'a> {
        AnySampleBuffer::S32(Cow::Owned(InterleveadSampleBuffer::from_samples(
            rate, channels, samples,
        )))
    }
}

impl SampleFormat {
    pub fn from_audio_buffer_ref<'a>(buffer: &AudioBufferRef<'a>) -> Self {
        match buffer {
            AudioBufferRef::F32(_) => SampleFormat::F32,
            AudioBufferRef::S16(_) => SampleFormat::S16,
            _ => panic!("Unsupported sample format"),
        }
    }
}
