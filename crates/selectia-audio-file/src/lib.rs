#![allow(unused_imports)]

pub mod audio_file;
pub mod error;
pub mod prelude;

pub use audio_file::AudioFile;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downsample() {
        let mut audio_file = AudioFile::open("C:\\Users\\corbe\\repos\\test.wav").unwrap();
        audio_file.decode().unwrap();
        let payload = audio_file.payload().unwrap();
        let downsampled = payload.downsampled(None, 4).unwrap();
        dbg!(&downsampled.samples[0..100]);
    }
}
