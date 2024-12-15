use models::FileVariationMetadata;

use crate::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub struct StemExtractionTask {
    pub metadata_id: i64,
}


impl StemExtractionTask {
    pub async fn process<T: ServiceHostContext>(&self, context: &T) -> Result<()> {
        info!("Processing stem extraction task: {:?}", self);

        let database = context.get_singleton::<Database>().await?;
        let demuxer = context.get_singleton_address::<Demuxer>().await?;

        let file = database
            .get_file_from_metadata_id(self.metadata_id)
            .await?;

        let input_path = PathBuf::from(&file.path);
        let mut output_path = PathBuf::from(&file.path);
        output_path.set_extension("stems");

        let (callback, recv) = TaskCallback::new();
        demuxer
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
            match database
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
