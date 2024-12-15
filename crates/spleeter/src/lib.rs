use crate::prelude::*;
use std::{borrow::Cow, path::PathBuf};
use tensorflow::{Graph, SavedModelBundle, SessionOptions, SessionRunArgs, Tensor};

mod error;
pub mod prelude;

#[derive(Debug, Serialize, Deserialize)]
pub struct SpleeterModelInfo {
    pub name: String,
    pub output_count: usize,
    pub output_names: Vec<String>,
    /// Generated output from the model
    pub track_names: Vec<String>,
}

#[derive(Debug)]
pub struct SpleeterModel {
    pub info: SpleeterModelInfo,
    pub path: PathBuf,
}

#[derive(Clone)]
pub struct Stem<'a> {
    pub name: String,
    pub data: AudioData<'a>,
}

#[derive(Clone)]
pub struct AudioData<'a> {
    pub sample_rate: usize,
    pub nb_channels: usize,
    pub samples: Cow<'a, [f32]>,
}

impl std::fmt::Debug for AudioData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AudioData {{ sample_rate: {}, nb_channels: {}, samples: [{}...] }}", self.sample_rate, self.nb_channels, self.samples.len())
    }
}

pub fn get_models_from_index(index_path: impl AsRef<Path>) -> SpleeterResult<Vec<SpleeterModel>> {
    let index_directory = index_path.as_ref().parent().to_owned().ok_or_else(|| {
        error!(
            "Malformated models index file path: {}",
            index_path.as_ref().display()
        );
        SpleeterError::MalformatedModelsIndex
    })?;
    let index_file = std::fs::File::open(index_path.as_ref())?;
    let index: Vec<SpleeterModelInfo> = serde_json::from_reader(index_file).map_err(|e| {
        error!("Malformated models index file: {}", e);
        SpleeterError::MalformatedModelsIndex
    })?;
    Ok(index
        .into_iter()
        .map(|info| {
            let path = index_directory.join(&info.name);
            SpleeterModel { info, path }
        })
        .collect())
}

#[instrument]
pub fn split_pcm_audio(audio: &AudioData, model: &SpleeterModel) -> SpleeterResult<Vec<Stem<'static>>> {
    let tensorflow_version = tensorflow::version().unwrap();
    info!(?tensorflow_version);

    let slice_length = audio.sample_rate * 30;
    let extend_length = audio.sample_rate * 5;
    let nb_channels = audio.nb_channels;

    let mut transformed_samples = vec![vec![]; model.info.output_count];

    info!("Loading model...");
    let mut graph = Graph::new();
    let session = SavedModelBundle::load(&SessionOptions::new(), ["serve"], &mut graph, &model.path)
        .map_err(|e| SpleeterError::SessionLoadError)?
        .session;

    let input_samples_count_per_channel = audio.samples.len() / audio.nb_channels;
    let segment_count = (input_samples_count_per_channel + (slice_length - 1)) / slice_length;

    for i in 0..segment_count {
        let current_offset = slice_length * i;
        let extend_length_at_begin = if i == 0 { 0 } else { extend_length };
        let extend_length_at_end = if i == (segment_count - 1) {
            0
        } else {
            extend_length
        };

        let useful_start = extend_length_at_begin;
        let useful_length = if i == (segment_count - 1) {
            input_samples_count_per_channel - current_offset
        } else {
            slice_length
        };

        let process_start = current_offset - extend_length_at_begin;
        let process_length = (useful_length + extend_length_at_begin + extend_length_at_end)
            .min(input_samples_count_per_channel - process_start);

        info!(
            "processing: [{}, {}), using [{}, {})",
            process_start,
            process_start + process_length,
            current_offset,
            current_offset + useful_length
        );

        let oper = graph
            .operation_by_name("Placeholder")
            .map_err(|e| SpleeterError::ProcessAudioError("no placeholder found (failed to get operation)"))?
            .ok_or_else(|| SpleeterError::ProcessAudioError("no placeholder found (operation is empty)"))?;

        let input_dims = [process_length as u64, nb_channels as u64];

        let input_data_length = process_length * nb_channels;
        let input_data_begin = process_start * nb_channels;
        let input_data =
            &audio.samples[input_data_begin..input_data_begin + input_data_length];

        let input_tensors = Tensor::new(&input_dims)
            .with_values(input_data)
            .map_err(|e| SpleeterError::ProcessAudioError("get tensor failed"))?;

        let mut output_tokens = Vec::new();

        let mut run_args = SessionRunArgs::new();
        run_args.add_feed(&oper, 0, &input_tensors);

        for i in 0..model.info.output_count {
            let oper = graph
                .operation_by_name(&model.info.output_names[i])
                .map_err(|e| SpleeterError::ProcessAudioError("get operation failed"))?
                .ok_or_else(|| SpleeterError::ProcessAudioError("get empty operation"))?;
            let fetch_token = run_args.request_fetch(&oper, 0);
            output_tokens.push(fetch_token);
        }

        session
            .run(&mut run_args)
            .map_err(|_e| SpleeterError::ProcessAudioError("run session failed"))?;

        for i in 0..model.info.output_count {
            let data: Tensor<f32> = run_args
                .fetch(output_tokens[i])
                .map_err(|_e| SpleeterError::ProcessAudioError("get output failed"))?;
            let begin = useful_start * nb_channels;
            let len = useful_length * nb_channels;
            transformed_samples[i].extend_from_slice(&data.as_ref()[begin..begin + len]);
        }
        info!("{}/{} done...", i + 1, segment_count);
    }
    Ok(transformed_samples.into_iter().enumerate().map(|(i, s)| {
        let stem = Stem {
            name: model.info.track_names[i].clone(),
            data: AudioData {
                sample_rate: audio.sample_rate,
                nb_channels: audio.nb_channels,
                samples: Cow::Owned(s),
            },
        };
        stem
    }).collect())
}
