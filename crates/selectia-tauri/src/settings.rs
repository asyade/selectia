use crate::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub database_path: PathBuf,
    pub demuxer_data_path: PathBuf,
    pub worker_threads: u32,
}

impl Settings {
    pub async fn load() -> eyre::Result<Self> {
        let data_dir = match std::env::var("SELECTIA_DATA_DIR").ok() {
            Some(path) => {
                info!("Using data directory from environment variable: {}", path);
                PathBuf::from(path)
            }
            None => dirs::data_local_dir().map(|path| path.join("selectia")).unwrap_or_else(|| {
                warn!("No data directory found, using current directory");
                PathBuf::from(".")
            }),
        };
        tokio::fs::create_dir_all(&data_dir).await?;
        let settings_file_path = data_dir.join("settings.json");

        match Self::load_stored_settings(&settings_file_path) {
            Ok(settings) => Ok(settings),
            Err(e) => {
                error!("Failed to load settings: {}", e);
                Self::create_default_settings_file(&settings_file_path, &data_dir)?;
                Self::load_stored_settings(&settings_file_path)
            }
        }
    }

    fn create_default_settings_file(
        settings_file_path: &PathBuf,
        data_dir: &PathBuf,
    ) -> eyre::Result<()> {
        info!(
            "Creating default settings file at {}",
            settings_file_path.display()
        );
        let settings = Settings {
            database_path: data_dir.join("database.db"),
            demuxer_data_path: data_dir.join("demuxer"),
            worker_threads: 4,
        };
        let settings_str = serde_json::to_string(&settings)?;
        std::fs::write(settings_file_path, settings_str)?;
        Ok(())
    }

    fn load_stored_settings(settings_file_path: &PathBuf) -> eyre::Result<Settings> {
        let settings_str = std::fs::read_to_string(settings_file_path)?;
        let settings: Settings = serde_json::from_str(&settings_str)?;
        Ok(settings)
    }
}
