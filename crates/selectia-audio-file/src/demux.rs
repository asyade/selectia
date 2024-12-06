pub struct DemuxedAudioFile {
    pub format: Format,
    pub decoded: Option<AudioFilePayload>,
    pub preview: Option<AudioFilePayload>,
}
