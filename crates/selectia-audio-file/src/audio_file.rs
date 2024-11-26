use symphonia::core::audio::SignalSpec;
use symphonia::core::formats::{FormatReader, Track};
use symphonia::core::io::MediaSource;

use crate::prelude::*;
use std::fs::File;

pub struct AudioFile {
    format: Box<dyn FormatReader>,
    samples: Option<Vec<f32>>,
}

impl AudioFile {

    pub fn from_source(source: Box<dyn MediaSource>, path: impl AsRef<Path>) -> AudioFileResult<Self> {
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
            samples: None,
        })
    }

    /// Open an audio file from a path.
    /// 
    /// ** Note ** Do not use this function in async contexts, use `from_source` instead.
    pub fn open<T: AsRef<Path>>(path: T) -> AudioFileResult<Self> {
        let file = Box::new(File::open(&path)?);
        Self::from_source(file, path)
    }

    pub fn decode(&mut self) -> AudioFileResult<()> {
        let decoder_opts: DecoderOptions = Default::default();
        // Get the default track.
        let track = self.format.default_track().expect("No default track found");
        info!("Track: {:?}", self.format.tracks());
        // Create a decoder for the track.
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &decoder_opts)
            .expect("Failed to create decoder");

        // Store the track identifier, we'll use it to filter packets.
        let track_id = track.id;

        let mut sample_buf = None;

        let mut decoded_samples = vec![];
        loop {
            // Get the next packet from the format reader.
            // @TODO: Handle ResetRequired error
            let Ok(packet) = self.format.next_packet() else {
                tracing::warn!("Unexpected error while getting next packet");
                break;
            };

            // If the packet does not belong to the selected track, skip it.
            if packet.track_id() != track_id {
                continue;
            }

            // Decode the packet into audio samples, ignoring any decode errors.
            match decoder.decode(&packet) {
                Ok(audio_buf) => {
                    // The decoded audio samples may now be accessed via the audio buffer if per-channel
                    // slices of samples in their native decoded format is desired. Use-cases where
                    // the samples need to be accessed in an interleaved order or converted into
                    // another sample format, or a byte buffer is required, are covered by copying the
                    // audio buffer into a sample buffer or raw sample buffer, respectively. In the
                    // example below, we will copy the audio buffer into a sample buffer in an
                    // interleaved order while also converting to a f32 sample format.

                    // If this is the *first* decoded packet, create a sample buffer matching the
                    // decoded audio buffer format.
                    if sample_buf.is_none() {
                        // Get the audio buffer specification.
                        let spec = *audio_buf.spec();
                        // Get the capacity of the decoded buffer. Note: This is capacity, not length!
                        let duration = audio_buf.capacity() as u64;
                        // Create the f32 sample buffer.
                        sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
                    }

                    // Copy the decoded audio buffer into the sample buffer in an interleaved format.
                    if let Some(buf) = &mut sample_buf {
                        buf.copy_interleaved_ref(audio_buf);

                        // The samples may now be access via the `samples()` function.
                        decoded_samples.extend_from_slice(buf.samples());
                    }
                }
                Err(Error::DecodeError(_)) => (),
                Err(_) => break,
            }
        }
        tracing::info!("Decoded {} samples", decoded_samples.len());
        self.samples = Some(decoded_samples);
        Ok(())
    }
}
