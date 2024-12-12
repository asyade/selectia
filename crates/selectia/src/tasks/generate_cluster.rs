use crate::prelude::*;

pub struct GenerateClusterTask {
}

impl GenerateClusterTask {
    pub async fn new(database: Database) -> Result<Self> {
        let _tags = database.list_files().await?;

        let barier = Arc::new(tokio::sync::Mutex::new(()));
        let lock = barier.clone().lock_owned().await;
        let _handle = std::thread::spawn(move || {

            // let mut config: TokenClassificationConfig = Default::default();
            // // config.strip_accents = Some(true);

            // let ner_model = NERModel::new(config).unwrap();

            // let inputs = tags.iter().map(|file| {
            //     let path = PathBuf::from(file.`path.clone());
            //     let ret = path.file_name().unwrap().to_str().unwrap().split(".").next().unwrap();

                

            //     // ret.to_lowercase()
            //     dbg!(ret.to_string().to_lowercase())
            // }).take(4).collect::<Vec<_>>();
            // let results = ner_model.predict(&inputs);

            // for result in results {
            //     println!("{:#?}", result);
            // }


            drop(lock);
        });
        let _ = barier.lock().await;
        Ok(Self {})
    }
}
