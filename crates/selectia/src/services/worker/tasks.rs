use crate::analyser::bpm_analyser::{BpmAnalyser, BpmAnalyserOptions};
use crate::prelude::*;
use demucs::backend::DemuxResult;
use eyre::bail;
use models::{FileVariationMetadata, Task};
use selectia_audio_file::{
    audio_file::{AudioFilePayload, EncodedAudioFile},
    error::{AudioFileError, AudioFileResult},
};
use spleeter::AudioData;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct BackgroundTask {
    pub id: i64,
    pub status: TaskStatus,
    pub payload: TaskPayload,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Queued,
    Processing,
    Done,
}

impl TryFrom<&str> for TaskStatus {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "queued" => Ok(TaskStatus::Queued),
            "processing" => Ok(TaskStatus::Processing),
            "done" => Ok(TaskStatus::Done),
            _ => bail!("Invalid task status: {}", value),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum TaskPayload {
    FileAnalysis(FileAnalysisTask),
    StemExtraction(StemExtractionTask),
}

pub struct TaskContext {
    pub demuxer: AddressableService<DemuxerTask>,
    pub database: Database,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub struct FileAnalysisTask {
    pub metadata_id: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub struct StemExtractionTask {
    pub metadata_id: i64,
}

impl TryFrom<Task> for BackgroundTask {
    type Error = eyre::Error;

    fn try_from(task: Task) -> Result<Self> {
        Ok(Self {
            id: task.id,
            status: TaskStatus::try_from(task.status.as_str())?,
            payload: serde_json::from_str(&task.payload)?,
        })
    }
}

impl BackgroundTask {
    pub async fn process(&self, context: TaskContext) -> Result<()> {
        match &self.payload {
            TaskPayload::FileAnalysis(task) => task.process(context).await,
            TaskPayload::StemExtraction(task) => task.process(context).await,
        }
    }
}

impl FileAnalysisTask {
    #[instrument(skip(context))]
    pub async fn process(&self, context: TaskContext) -> Result<()> {
        use fundsp::prelude::*;
        const MIN_ANALYSIS_DURATION: f64 = 180.0;

        let file = context
            .database
            .get_file_from_metadata_id(self.metadata_id)
            .await?;
        let input_file_path = PathBuf::from(&file.path);

        let (temp_dir, payload) = tokio::task::spawn_blocking(move || {
            let dir = tempdir::TempDir::new("task_analysis").unwrap();
            let encoded_file = EncodedAudioFile::from_file(&input_file_path)?;
            let payload = encoded_file.read_into_payload()?;


            let export_dir = dir.path().join("export");
            info!(
                stem_path = export_dir.to_str().unwrap(),
                sample_rate = payload.sample_rate,
                duration = payload.duration,
                "Payload ready for stem extraction"
            );
            AudioFileResult::Ok((dir, payload))
        })
        .await??;


        let spleeter_model = spleeter::get_models_from_index(PathBuf::from("C:\\Users\\corbe\\Desktop\\spleeter\\index.json")).unwrap();
        let model = spleeter_model.iter().find(|m| m.info.name == "4stems").unwrap();
        let result = spleeter::split_pcm_audio(&AudioData {
            sample_rate: payload.sample_rate as usize,
            nb_channels: payload.channels as usize,
            samples: payload.buffer.buffer,
        }, &model).unwrap();

        let drums_stem = result.into_iter().find(|s| s.name == "drums").unwrap().data;
        let payload = AudioFilePayload::from_interleaved_samples(payload.sample_rate, payload.channels, drums_stem.samples)?;
        // let (callback, recv) = TaskCallback::new();
        // let task = DemuxerTask::Demux {
        //     input: analyzed_audio_file.clone(),
        //     output: temp_dir.path().join("stems"),
        //     callback,
        // };

        // let begin = Instant::now();
        // context.demuxer.send(task).await?;
        // let demux_result = recv.wait().await?;
        // info!(
        //     duration = begin.elapsed().as_secs_f32(),
        //     "Stem extraction task completed"
        // );

        // let drum_stem = demux_result
        //     .get_stem(DemuxResult::DRUMS)
        //     .ok_or(AudioFileError::AudioSeparationFailed)?;
        // let drum_file_path = PathBuf::from(drum_stem.path.as_str());

        let _bpm_analysis_result = tokio::task::spawn_blocking(move || {
            let wave = payload.wave();
            let mut wave = wave.filter(
                wave.duration(),
                &mut (highpass_hz(38.0, 0.7) >> lowpass_hz(1000.0, 0.7)),
            );
            wave.normalize();
            let payload = AudioFilePayload::from_wave(wave)?;
            let payload = payload.resample(44100.0)?;
            let onesets = payload.detect_onesets(512, 128)?;
            let bpm_analyser = BpmAnalyser::new(
                BpmAnalyserOptions {
                    range: (80.0, 280.0),
                },
                onesets,
            )
            .get_result()
            .unwrap();
            info!(bpm_analyser=?bpm_analyser, "BPM analysis completed");
            AudioFileResult::Ok(bpm_analyser)
        })
        .await??;

        // let _ = context.database.delete_metadata_tag_by_tag_name_id(self.metadata_id, TagName::TEMPO_ID);
        // context.database.set_metadata_tag_by_tag_name_id(self.metadata_id, TagName::TEMPO_ID, bpm_analysis_result.average_bpm.unwrap().bpm.to_string())?;

        Ok(())
    }
}

impl StemExtractionTask {
    pub async fn process(&self, context: TaskContext) -> Result<()> {
        info!("Processing stem extraction task: {:?}", self);
        let file = context
            .database
            .get_file_from_metadata_id(self.metadata_id)
            .await?;

        let input_path = PathBuf::from(&file.path);
        let mut output_path = PathBuf::from(&file.path);
        output_path.set_extension("stems");

        let (callback, recv) = TaskCallback::new();
        context
            .demuxer
            .send(DemuxerTask::Demux {
                input: PathBuf::from(file.path),
                output: output_path.clone(),
                callback,
            })
            .await?;
        info!("Waiting for stem extraction task to complete");
        let result = recv.wait().await?;
        info!("Stem extraction task completed, creating file variations");
        for variation in result.stems.iter() {
            let metadata = FileVariationMetadata {
                stem: Some(variation.stem.clone()),
                title: input_path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
            };
            match context
                .database
                .create_file_variation(file.id, &variation.path, metadata)
                .await
            {
                Ok(_) => info!("Created file variation: {}", variation.path),
                Err(e) => error!("Failed to create file variation: {}", e),
            }
        }
        Ok(())
    }
}
