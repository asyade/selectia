use dasp::signal::interpolate::Converter;
use dasp::{sample, Sample as _, Signal};
use symphonia::core::audio::{
    AudioBuffer, AudioBufferRef, Channels, Signal as SymphoniaSignal, SignalSpec,
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

pub struct AudioFile {
    format: Box<dyn FormatReader>,
    decoded: Option<AudioFilePayload>,
    preview: Option<AudioFilePayload>,
}

#[derive(Clone)]
pub struct AudioFilePayload {
    pub duration: f64,
    pub sample_rate: u32,
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
        let probed =
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
            let preview = payload.resample(11025/10).unwrap();
            self.preview = Some(preview);
        }
        self.preview.as_ref()
    }

    pub fn preview(&self) -> Option<&AudioFilePayload> {
        self.preview.as_ref()
    }

    pub fn detect_tempo(&mut self) -> AudioFileResult<f32> {
        const WIN_SIZE: usize = 1024;
        const HOP_SIZE: usize = 512;
        let payload = self.payload().unwrap();
        let resampled = payload.resample(44100).unwrap().into_mono().unwrap();


        resampled.wav_export("resampled.wav").unwrap();
        let mut tempo = aubio_rs::Tempo::new(aubio_rs::OnsetMode::SpecDiff, WIN_SIZE, HOP_SIZE, resampled.sample_rate).unwrap();


        let mut output = vec![0.0; 1];
        let mut hop_buffer = vec![0.0; HOP_SIZE];

        let mut beats = vec![];
        resampled.buffer.buffer.chunks(HOP_SIZE).for_each(|chunk| {
            if chunk.len() < HOP_SIZE {
                return;
            }
            hop_buffer.copy_from_slice(chunk);  
            let c_source = aubio_rs::vec::FVec::from(&hop_buffer);
            let c_output = aubio_rs::vec::FVecMut::from(&mut output);
            tempo.do_(c_source, c_output).unwrap();

            if output[0] != 0.0 {
                beats.push(tempo.get_last_ms());
            }
        });

        let interval = beats.windows(2).map(|w| w[1] - w[0]).collect::<Vec<_>>();

        let avg_interval = interval.iter().sum::<f32>() / interval.len() as f32;

        let filtered_interval = interval.clone().into_iter().filter(|&i|  (i - avg_interval).abs() < 30.0).collect::<Vec<_>>();

        let avg_filtered_interval = filtered_interval.iter().sum::<f32>() / filtered_interval.len() as f32;

        let bpm = 60.0 / avg_interval * 1000.0;
        let bpm_filtered = 60.0 / avg_filtered_interval * 1000.0;
        dbg!(bpm);
        dbg!(bpm_filtered);

        Ok(bpm)
    }

    pub fn decode(&mut self) -> AudioFileResult<&AudioFilePayload> {
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
                            spec.rate,
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

impl AudioFilePayload {
    pub fn into_mono(self) -> AudioFileResult<AudioFilePayload> {
        if self.channels == 1 {
            return Ok(self);
        }

        if self.channels == 2 {
            let samples = self.buffer.buffer.chunks(2).filter_map(|chunk| {
                if chunk.len() < 2 {
                    return None;
                }
                Some((chunk[0] + chunk[1]) / 2.0)
            }).collect();
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
    pub fn resample(&self, sample_rate: u32) -> AudioFileResult<AudioFilePayload> {
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
    pub fn wav_export(&self, path: impl AsRef<Path>) -> AudioFileResult<()> {
        let spec = hound::WavSpec {
            channels: self.channels as u16,
            sample_rate: self.sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let file = File::create(path)?;
        let mut writer = hound::WavWriter::new(file, spec)?;
        let mut sample_writer = writer.get_i16_writer(self.buffer.buffer.len() as u32);
        for sample in self.buffer.buffer.iter() {
            let next = sample.to_sample::<i16>();
            sample_writer.write_sample(next);
        }
        sample_writer.flush().unwrap();
        writer.finalize()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_1_FILE: &str = "../../dataset/drums-160.flac";

    #[test]
    fn downsample() {
        let mut audio_file = AudioFile::open(TEST_1_FILE).unwrap();
        audio_file.decode().unwrap();
        let payload = audio_file.payload().unwrap().clone().into_mono().unwrap();

        let tempo = audio_file.detect_tempo().unwrap();
        println!("Tempo: {}", tempo);
    
        // let mut planner = FftPlanner::new();
        // let fft = planner.plan_fft_forward(samples.len());
        // let mut buffer: Vec<Complex<f32>> = samples.iter()
        //     .map(|&x| Complex { re: x, im: 0.0 })
        //     .collect();
        // fft.process(&mut buffer);
        
        // let spectrum: Vec<f32> = buffer.iter().map(|c| c.norm()).collect();

        // let fft_size = samples.len();
        // let bin_width = payload.sample_rate as f32 / fft_size as f32; // Δf = Fs / N
    
        // for (i, freq_component) in buffer.iter().enumerate() {
        //     let frequency = i as f32 * bin_width; // Frequency in Hz
        //     let magnitude = freq_component.norm(); // Magnitude of the frequency component
        //     println!("Bin {}: Frequency = {:.2} Hz, Magnitude = {:.2}", i, frequency, magnitude);
        // }
    
    }
}
