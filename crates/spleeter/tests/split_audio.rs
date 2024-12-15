use std::borrow::Cow;

use spleeter::prelude::*;
use selectia_audio_file::{audio_file::{AudioFilePayload, EncodedAudioFile}, prelude::*};

const INDEX_FILE_PATH: &str = "../../models/spleeter/index.json";
const INPUT_FILE_PATH: &str = "../../dataset/128-bpm.wav";
#[test]
pub fn test_split_audio() {
    tracing_subscriber::fmt()
        .with_env_filter("trace")
        .pretty()
        .with_file(true)
        .fmt_fields(tracing_subscriber::fmt::format::PrettyFields::new())
        .init();

    let models = get_models_from_index(INDEX_FILE_PATH).unwrap();
    let model = models.iter().find(|m| m.info.name == "4stems").unwrap();

    let encoded = EncodedAudioFile::from_file(Path::new(INPUT_FILE_PATH)).unwrap();
    let payload = encoded.read_into_payload().unwrap();
    let payload = payload.resample(44100.0).unwrap();
    dbg!(&payload.channels);
    let audio = AudioData {
        sample_rate: payload.sample_rate as usize,
        nb_channels: payload.channels as usize,
        samples: Cow::Owned(payload.buffer.buffer.to_vec()),
    };
    let result = split_pcm_audio(&audio, &model).unwrap();

    for (i, result) in result.into_iter().enumerate() {
        let encoded = AudioFilePayload::from_interleaved_samples(payload.sample_rate, payload.channels, result.data.samples.to_vec()).unwrap();
        let encoded = encoded.wav_export(payload.sample_rate as u32, PathBuf::from(format!("C:\\Users\\corbe\\Desktop\\test{}.wav", &result.name)));
    }
}
