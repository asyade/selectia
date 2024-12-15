use std::{borrow::Cow, result};

use dasp::{signal::rms::SignalRms, Sample, Signal};
use selectia_audio_file::{audio_file::{AudioFilePayload, EncodedAudioFile}, error::AudioFileResult};
use spleeter::{AudioData, Stem};
use sqlx::database;
use wavision::prelude::*;

use crate::{analyser::bpm_analyser::{BpmAnalyser, BpmAnalyserOptions}, prelude::*};


#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub struct FileAnalysisTask {
    pub metadata_id: i64,
}

impl FileAnalysisTask {
    #[instrument(skip(context))]
    pub async fn process<T: ServiceHostContext>(&self, context: &T) -> Result<()> {
        use fundsp::prelude::*;
        const MIN_ANALYSIS_DURATION: f64 = 180.0;

        let database = context.get_singleton::<Database>().await?;

        let file = database
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

        //Demux
        let spleeter_model =
            spleeter::get_models_from_index(PathBuf::from("../../../models/spleeter/index.json"))
                .unwrap();
        let model = spleeter_model
            .iter()
            .find(|m| m.info.name == "4stems")
            .unwrap();
        let result = spleeter::split_pcm_audio(
            &AudioData {
                sample_rate: payload.sample_rate as usize,
                nb_channels: payload.channels as usize,
                samples: Cow::Borrowed(&payload.buffer.buffer),
            },
            &model,
        )
        .unwrap();

        let drums_stem = result.iter().find(|s| s.name == "drums").cloned().unwrap().data;
        let payload = AudioFilePayload::from_interleaved_samples(
            payload.sample_rate,
            payload.channels,
            drums_stem.samples.to_vec(),
        )?;
        let payload = payload.into_mono()?;

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
        let (payload, onesets) = tokio::task::spawn_blocking(move || {
            let onesets = payload.detect_onesets(512, 256)?;
            AudioFileResult::Ok((payload, onesets))
        }).await??;

        let rms_buffer = dasp::ring_buffer::Fixed::from(vec![0.0; 512]);
        let rms = dasp::signal::from_iter(payload.buffer.buffer.iter().copied()).rms(rms_buffer).until_exhausted().collect::<Vec<_>>();
        let rms = rms.chunks(22000).map(|e| {
            e.iter().copied().sum::<f32>() / 22000.0
        }).collect::<Vec<_>>();


        // let onesets = onesets.into_iter().filter(|oneset| {
        //     let rms_value = rms[oneset.offset / 22000];
        //     rms_value > 0.3
        // }).collect::<Vec<_>>();

        // Force borrow checker to corelate lifetimes (can be done cleaner with better code separation)
        fn graph_for_stems<'a>(sample_rate: u32, stems: &'a [Stem<'a>], image_generator: &mut GraphGenerator<'a>)  {
            for stem in stems {
                image_generator.layer(SamplesLayer::new(SamplesLayerStyle {
                    color: match stem.name.as_str() {
                        "vocals" => Rgb([255, 0, 0]),
                        "drums" => Rgb([123, 58, 236]),
                        "bass" => Rgb([236, 141, 58]),
                        "other" => Rgb([255, 255, 255]),
                        _ => Rgb([255, 255, 255]),
                    },
                }, &stem.data.samples, stem.data.nb_channels));
            }
        }

        let mut image_generator = GraphGenerator::new(payload.sample_rate as usize, 64, 64000);
        
        let cues = onesets.iter().map(|oneset| (oneset.offset, Cue {
            start: oneset.offset,
            end: oneset.offset + oneset.duration,
        })).collect();
        let oneset_layer = CuesLayer::new(CuesLayerStyle {}, 96000, cues);
        image_generator.layer(oneset_layer);
        let input_file_path = PathBuf::from(&file.path);
        graph_for_stems(payload.sample_rate as u32, &result, &mut image_generator);
        match image_generator.generate() {
            Ok(image) => {
                image.save("C:\\Users\\corbe\\Desktop\\wave.png").unwrap();
            }
            Err(e) => {
                error!("Error generating image: {}", e);
            }
        }

        // for stem in result {
            // image_generator.layer(SamplesLayer::new(SamplesLayerStyle {}, &stem.data.samples, stem.data.nb_channels as u32));
        // }


        let bpm_analyser = BpmAnalyser::new(
            BpmAnalyserOptions {
                range: (80.0, 280.0),
            },
            onesets,
        )
        .get_result()
        .unwrap();
        info!(bpm_analyser=?bpm_analyser, "BPM analysis completed");


        // let _ = context.database.delete_metadata_tag_by_tag_name_id(self.metadata_id, TagName::TEMPO_ID);
        // context.database.set_metadata_tag_by_tag_name_id(self.metadata_id, TagName::TEMPO_ID, bpm_analysis_result.average_bpm.unwrap().bpm.to_string())?;

        Ok(())
    }
}
