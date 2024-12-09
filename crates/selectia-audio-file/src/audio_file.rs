use dasp::sample::ToSample;
use dasp::signal::interpolate::Converter;
use dasp::signal::rms::SignalRms as _;
use dasp::{ring_buffer, sample, Sample as _, Signal};
use fundsp::prelude::*;
use symphonia::core::audio::{
    self, AsAudioBufferRef, AudioBuffer, AudioBufferRef, Channels, Signal as SymphoniaSignal,
    SignalSpec,
};
use symphonia::core::conv::{ConvertibleSample, IntoSample};
use symphonia::core::formats::{FormatReader, Track};
use symphonia::core::io::MediaSource;

use crate::audio_buffer::*;
use crate::dc_blocker::DcBlocker;
use crate::prelude::*;
use std::any::Any;
use std::borrow::Cow;
use std::fs::File;
use std::ops::Range;
use std::panic::panic_any;

pub struct EncodedAudioFile {
    format: Box<dyn FormatReader>,
}

pub struct AudioFile {
    format: Box<dyn FormatReader>,
    decoded: Option<AudioFilePayload>,
    preview: Option<AudioFilePayload>,
}

#[derive(Clone)]
pub struct AudioFilePayload {
    pub duration: f64,
    pub sample_rate: f64,
    pub channels: u32,
    pub buffer: InterleveadSampleBuffer<f32>,
}

impl AudioFile {
    pub fn from_source(
        source: Box<dyn MediaSource>,
        path: impl AsRef<Path>,
    ) -> AudioFileResult<Self> {
        let mss = MediaSourceStream::new(source, Default::default());
        // Create a hint to help the format registry guess what format reader is appropriate. In this
        // example we'll leave it empty.
        let mut hint = Hint::new();
        if let Some(extension) = path.as_ref().extension() {
            hint.with_extension(extension.to_str().unwrap());
        }

        // Use the default options when reading and decoding.
        let format_opts: FormatOptions = Default::default();
        let metadata_opts: MetadataOptions = Default::default();

        // Probe the media source stream for a format.
        let probed: symphonia::core::probe::ProbeResult =
            symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts)?;

        // Get the format reader yielded by the probe operation.
        let format = probed.format;

        Ok(Self {
            format: format,
            decoded: None,
            preview: None,
        })
    }

    pub fn payload(&self) -> Option<&AudioFilePayload> {
        self.decoded.as_ref()
    }

    pub fn into_payload(self) -> Option<AudioFilePayload> {
        self.decoded
    }

    /// Open an audio file from a path.
    ///
    /// ** Note ** Do not use this function in async contexts, use `from_source` instead.
    pub fn open<T: AsRef<Path>>(path: T) -> AudioFileResult<Self> {
        let file = Box::new(File::open(&path)?);
        Self::from_source(file, path)
    }

    pub fn generate_preview(&mut self) -> Option<&AudioFilePayload> {
        if self.preview.is_none() {
            let payload = self.payload()?;
            let preview = payload.resample(10000.0).unwrap();
            self.preview = Some(preview);
        }
        self.preview.as_ref()
    }

    pub fn preview(&self) -> Option<&AudioFilePayload> {
        self.preview.as_ref()
    }

    pub fn decode_to_end(&mut self) -> AudioFileResult<&AudioFilePayload> {
        let decoder_opts: DecoderOptions = Default::default();
        // Get the default track.
        let track = self.format.default_track().expect("No default track found");
        info!("Track: {:?}", self.format.tracks());
        // Create a decoder for the track.
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &decoder_opts)
            .expect("Failed to create decoder");

        let track_id = track.id;
        let mut sample_buf = None;
        while let Ok(packet) = self.format.next_packet() {
            if packet.track_id() != track_id {
                continue;
            }
            match decoder.decode(&packet) {
                Ok(audio_buf) => {
                    let spec = audio_buf.spec();
                    let sample_buf = sample_buf.get_or_insert_with(|| {
                        AnySampleBuffer::new(
                            spec.rate as f64,
                            spec.channels.count() as u32,
                            SampleFormat::F32,
                        )
                    });
                    sample_buf.append_interleaved(&audio_buf);
                }
                Err(Error::DecodeError(_)) => (),
                Err(_) => break,
            }
        }

        let payload = sample_buf.expect("handle must be called at least once");
        let channels_count = payload.channels();
        let sample_rate = payload.rate();
        let decoded_samples = payload.len();
        let duration = decoded_samples as f64 / channels_count as f64 / sample_rate as f64;

        self.decoded = Some(AudioFilePayload {
            duration,
            sample_rate,
            channels: channels_count,
            buffer: payload.into_f32_buffer(),
        });
        Ok(self.payload().as_ref().unwrap())
    }
}

pub trait WaveExt {
    fn append_audio_buffer(&mut self, audio_buf: &AudioBuffer<f32>) -> AudioFileResult<()>;
    fn append_audio_buffer_mono(&mut self, audio_buf: &AudioBuffer<f32>) -> AudioFileResult<()>;
}

impl WaveExt for Wave {
    fn append_audio_buffer(&mut self, audio_buf: &AudioBuffer<f32>) -> AudioFileResult<()> {
        match self.channels() {
            1 => {
                let chann_0_buff = audio_buf.chan(0);
                for sample in chann_0_buff.iter().copied() {
                    self.push(sample);
                }
            }
            2 => {
                let chann_0_buff = audio_buf.chan(0);
                let chann_1_buff = audio_buf.chan(1);
                for (sample_0, sample_1) in chann_0_buff
                    .iter()
                    .copied()
                    .zip(chann_1_buff.iter().copied())
                {
                    self.push(sample_0);
                    self.push(sample_1);
                }
            }
            _ => todo!(),
        }
        Ok(())
    }

    fn append_audio_buffer_mono(&mut self, audio_buf: &AudioBuffer<f32>) -> AudioFileResult<()> {
        match self.channels() {
            1 => {
                let chann_0_buff = audio_buf.chan(0);
                for sample in chann_0_buff.iter().copied() {
                    self.push(sample);
                }
            }
            2 => {
                let chann_0_buff = audio_buf.chan(0);
                let chann_1_buff = audio_buf.chan(1);
                for (sample_0, sample_1) in chann_0_buff
                    .iter()
                    .copied()
                    .zip(chann_1_buff.iter().copied())
                {
                    self.push((sample_0 / 2.0) + (sample_1 / 2.0));
                }
            }
            _ => todo!(),
        }
        Ok(())
    }
}

impl EncodedAudioFile {
    pub fn from_source(
        source: Box<dyn MediaSource>,
        path: impl AsRef<Path>,
    ) -> AudioFileResult<Self> {
        let mss = MediaSourceStream::new(source, Default::default());
        let mut hint = Hint::new();
        if let Some(extension) = path.as_ref().extension() {
            hint.with_extension(extension.to_str().unwrap());
        }
        let probed = symphonia::default::get_probe().format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )?;
        let format = probed.format;
        Ok(Self { format: format })
    }

    pub fn from_file<T: AsRef<Path>>(path: T) -> AudioFileResult<Self> {
        let file = Box::new(File::open(&path)?);
        Self::from_source(file, path)
    }

    pub fn total_frames_count(mut self) -> AudioFileResult<u64> {
        let mut total = 0;

        let track = self
            .format
            .default_track()
            .ok_or(AudioFileError::NoDefaultTrack)?;
        let track_id = track.id;
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &Default::default())
            .expect("Failed to create decoder");
        while let Ok(packet) = self.format.next_packet() {
            if packet.track_id() != track_id {
                continue;
            }
            match decoder.decode(&packet) {
                Ok(audio_buf) => {
                    total += audio_buf.frames() as u64;
                }
                Err(Error::DecodeError(e)) => {
                    error!("Decode error: {:?}", e);
                }
                Err(_) => break,
            }
        }
        Ok(total)
    }

    pub fn read_wave_until<F: FnMut(&Wave) -> AudioFileResult<bool>>(
        mut self,
        mut callback: F,
    ) -> AudioFileResult<Wave> {
        let mut wave = None;
        self.decoded_iterator(|audio_buf| {
            if wave.is_none() {
                let spec = audio_buf.spec();
                wave = Some(Wave::new(spec.channels.count(), spec.rate as f64));
            }
            let wave = wave.as_mut().unwrap();
            wave.append_audio_buffer(audio_buf)?;
            Ok(callback(wave)?)
        })?;
        wave.ok_or(AudioFileError::EmptyContainer)
    }

    pub fn read_mono_wave_until<F: FnMut(&Wave) -> AudioFileResult<bool>>(
        mut self,
        mut callback: F,
    ) -> AudioFileResult<Wave> {
        let interpolator = dasp::interpolate::linear::Linear::new(0.0, 0.0);

        let mut wave: Option<Wave> = None;

        self.decoded_iterator(|audio_buf| {
            if wave.is_none() {
                let spec = audio_buf.spec();
                wave = Some(Wave::new(1, spec.rate as f64));
            }
            let wave = wave.as_mut().unwrap();
            wave.append_audio_buffer_mono(audio_buf)?;
            Ok(callback(wave)?)
        })?;
        wave.ok_or(AudioFileError::EmptyContainer)
    }

    pub fn read_into_payload(mut self) -> AudioFileResult<AudioFilePayload> {
        let mut sample_buf = None;
        self.decoded_iterator(|audio_buf| {
            let spec = audio_buf.spec();
            let sample_buf = sample_buf.get_or_insert_with(|| {
                AnySampleBuffer::new(spec.rate as f64, spec.channels.count() as u32, SampleFormat::F32)
            });
            sample_buf.append_interleaved(&audio_buf.as_audio_buffer_ref());
            Ok(true)
        })?;

        let sample_buf = sample_buf.ok_or(AudioFileError::EmptyContainer)?;
        let channels_count = sample_buf.channels();
        let sample_rate = sample_buf.rate();
        let decoded_samples = sample_buf.len();
        let duration = decoded_samples as f64 / channels_count as f64 / sample_rate as f64;

        let payload = AudioFilePayload {
            duration,
            sample_rate,
            channels: channels_count,
            buffer: sample_buf.into_f32_buffer(),
        };
        Ok(payload)
    }

    pub fn decoded_iterator<T: FnMut(&AudioBuffer<f32>) -> AudioFileResult<bool>>(
        &mut self,
        mut callback: T,
    ) -> AudioFileResult<bool> {
        let track = self
            .format
            .default_track()
            .ok_or(AudioFileError::NoDefaultTrack)?;
        let track_id = track.id;
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &Default::default())
            .expect("Failed to create decoder");

        let mut converted_buf: Option<AudioBuffer<f32>> = None;
        while let Ok(packet) = self.format.next_packet() {
            if packet.track_id() != track_id {
                continue;
            }
            match decoder.decode(&packet) {
                Ok(audio_buf) => {
                    if converted_buf.is_none() {
                        converted_buf = Some(AudioBuffer::new(
                            audio_buf.capacity() as u64,
                            audio_buf.spec().clone(),
                        ))
                    }

                    let converted_buf = converted_buf.as_mut().unwrap();
                    audio_buf.convert(converted_buf);
                    if !callback(converted_buf)? {
                        return Ok(false);
                    }
                }
                Err(Error::DecodeError(e)) => {
                    error!("Decode error: {:?}", e);
                }
                Err(_) => break,
            }
        }
        Ok(true)
    }
}

impl AudioFilePayload {
    pub fn from_wave(wave: Wave) -> AudioFileResult<Self> {
        match wave.channels() {
            1 => Ok(Self {
                duration: wave.duration(),
                sample_rate: wave.sample_rate(),
                channels: 1,
                buffer: InterleveadSampleBuffer::from_samples(
                    wave.sample_rate(),
                    1,
                    wave.channel(0).to_vec(),
                ),
            }),
            _ => todo!(),
        }
    }

    pub fn detect_onesets(&self, win_size: usize) -> AudioFileResult<Vec<AudioBeatOneset>> {
        // Aubio only supports integer sample rates
        if (self.sample_rate - self.sample_rate.round()).abs() > 0.0001 {
            return Err(AudioFileError::InvalidSampleRate);
        }
        let HOP_SIZE: usize = 128;

        let mut output = vec![0.0; 1];
        let mut hop_buffer = vec![0.0; HOP_SIZE];

        let mut tempo = aubio_rs::Tempo::new(
            aubio_rs::OnsetMode::SpecDiff,
            win_size,
            HOP_SIZE,
            self.sample_rate as u32,
        )
        .unwrap();

        let mut beats = vec![];
        self.buffer.buffer.chunks(HOP_SIZE).for_each(|chunk| {
            if chunk.len() < HOP_SIZE {
                return;
            }
            hop_buffer.copy_from_slice(chunk);
            let c_source = aubio_rs::vec::FVec::from(&hop_buffer);
            let c_output = aubio_rs::vec::FVecMut::from(&mut output);
            tempo.do_(c_source, c_output).unwrap();

            if output[0] != 0.0 {
                let detected_periode = tempo.get_period();
                let detected_confidence = tempo.get_confidence();
                let detected_at_offset = tempo.get_last();
                let bpm = tempo.get_bpm();
                beats.push(AudioBeatOneset {
                    offset: detected_at_offset,
                    duration: detected_periode,
                    confidence: detected_confidence,
                    bpm,
                });
            }
        });
        Ok(beats)
    }

    pub fn slice(&self, start_index: usize, end_index: usize) -> AudioFileResult<AudioFilePayload> {
        let start_index_in_buffer = start_index * self.channels as usize;
        let end_index_in_buffer = end_index * self.channels as usize;

        if start_index_in_buffer >= self.buffer.buffer.len()
            || end_index_in_buffer >= self.buffer.buffer.len()
        {
            error!(
                start_index_in_buffer,
                end_index_in_buffer,
                buffer_len = self.buffer.buffer.len(),
                "Index out of bounds"
            );
            return Err(AudioFileError::OutOfBounds);
        }

        let samples = self.buffer.buffer[start_index_in_buffer..end_index_in_buffer].to_vec();
        let duration = samples.len() as f64 / self.channels as f64 / self.sample_rate as f64;

        Ok(AudioFilePayload {
            duration,
            sample_rate: self.sample_rate,
            channels: self.channels,
            buffer: InterleveadSampleBuffer::from_samples(self.sample_rate, self.channels, samples),
        })
    }

    pub fn wave(&self) -> Wave {
        if self.channels != 1 {
            panic!("TODO: this function is broken for multi-channel audio");
        }
        let channel_size = self.buffer.buffer.len() / self.channels as usize;
        let mut channel_buffers = vec![Vec::with_capacity(channel_size); self.channels as usize];
        for samples in self.buffer.buffer.chunks(self.channels as usize) {
            for channel in 0..self.channels as usize {
                channel_buffers[channel].push(samples[channel]);
            }
        }

        let mut wave = Wave::new(0, self.sample_rate as f64);
        for channel in 0..self.channels as usize {
            let channel_buffer = channel_buffers.get(channel).unwrap();
            wave.push_channel(channel_buffer);
        }
        wave
    }

    pub fn into_processed_payload(
        &self,
        node: &mut dyn AudioUnit,
    ) -> AudioFileResult<AudioFilePayload> {
        let wave = self.wave();
        let mut wave = wave.filter(120.0, node);
        wave.normalize();
        let buffer = wave.channel(0).to_vec();

        Ok(AudioFilePayload {
            duration: self.duration,
            sample_rate: self.sample_rate,
            channels: self.channels,
            buffer: InterleveadSampleBuffer::from_samples(self.sample_rate, self.channels, buffer),
        })
    }

    pub fn into_mono(self) -> AudioFileResult<AudioFilePayload> {
        if self.channels == 1 {
            return Ok(self);
        }

        if self.channels == 2 {
            let samples = self
                .buffer
                .buffer
                .chunks(2)
                .filter_map(|chunk| {
                    if chunk.len() < 2 {
                        return None;
                    }
                    Some((chunk[0] + chunk[1]) / 2.0)
                })
                .collect();
            return Ok(AudioFilePayload {
                duration: self.duration,
                sample_rate: self.sample_rate,
                channels: 1,
                buffer: InterleveadSampleBuffer::from_samples(self.sample_rate, 1, samples),
            });
        }

        unimplemented!()
    }

    /// Resample the audio file to the given sample rate.
    /// The audio file will be converted to f32 samples (returned payload is always f32).
    pub fn resample(&self, sample_rate: f64) -> AudioFileResult<AudioFilePayload> {
        let interpolator = dasp::interpolate::linear::Linear::new(0.0, 0.0);
        let signal = dasp::signal::from_iter(self.buffer.buffer.iter().copied());
        let resampler =
            signal.from_hz_to_hz(interpolator, self.sample_rate as f64, sample_rate as f64);
        let samples: Vec<f32> = resampler.until_exhausted().collect();
        Ok(AudioFilePayload {
            duration: self.duration,
            sample_rate: sample_rate,
            channels: self.channels,
            buffer: InterleveadSampleBuffer::from_samples(sample_rate, self.channels, samples),
        })
    }

    /// *WIP*
    pub fn wav_export(&self, sample_rate: u32, path: impl AsRef<Path>) -> AudioFileResult<()> {
        let payload = self.resample(sample_rate as f64)?;
        let spec = hound::WavSpec {
            channels: self.channels as u16,
            sample_rate: sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let file = File::create(path)?;
        let mut writer = hound::WavWriter::new(file, spec)?;
        let mut sample_writer = writer.get_i16_writer(payload.buffer.buffer.len() as u32);
        for sample in payload.buffer.buffer.iter() {
            let next = sample.to_sample::<i16>();
            sample_writer.write_sample(next);
        }
        sample_writer.flush().unwrap();
        writer.finalize()?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct AudioBeatOneset {
    pub offset: usize,
    pub duration: usize,
    pub confidence: f32,
    pub bpm: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
}
