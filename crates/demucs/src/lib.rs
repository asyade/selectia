//! This crate is a wrapper around the demucs library (https://github.com/facebookresearch/demucs)
//! It provides a way to install the demucs library and use it to demux audio files.
//! 
//! ** WIP **
//! In his current state the crate is not ready for production use.
//! Actually the crate does not implement inference of the model, it simply install a full python environment and run a python script in a subprocess.
//! This approach work fine but is terribly ineficient.
//! In the future the crate may fully implement the demucs model.

#![allow(unused_imports)]

use backend::{Backend, DemuxResult, FromBackendRequest, ToProcessRequest};
use tokio::sync::Mutex;
use tracing::{error, info, instrument};

use crate::prelude::*;

pub mod backend;
pub mod prelude;

const CONDA_ENV_FILE: &str = include_str!("../env.yml");
const BACKEND_SCRIPT: &str = include_str!("../backend.py");

const ENVIRONMENT_NAME: &str = "demucs";

#[derive(Clone)]
pub struct Demucs {
    conda_env_file: PathBuf,
    backend_script: PathBuf,
    environment: Arc<RwLock<macromamba::Environment>>,
    pub status: Arc<RwLock<Status>>,
}

#[derive(Clone)]
pub enum Status {
    Initializing,
    NotInstalled,
    Ready {
        backend: Arc<Mutex<Backend>>,
    },
    Exited,
}

impl Demucs {
    pub async fn new(data_path: PathBuf) -> Result<Self> {
        let environment = macromamba::Environment::new(data_path.join("mamba"));

        let conda_env_file = data_path.join("env.yml");
        let backend_script = data_path.join("backend.py");

        let instance = Self {
            conda_env_file,
            backend_script,
            environment: Arc::new(RwLock::new(environment)),
            status: Arc::new(RwLock::new(Status::Initializing)),
        };
        instance.create_required_files().await?;
        Ok(instance)
    }

    pub async fn install(&self) -> Result<()> {
        self.environment.read().await.install(&self.conda_env_file).await?;
        self.environment.read().await.activate_env(ENVIRONMENT_NAME).await?;
        Ok(())
    }

    async fn create_required_files(&self) -> Result<()> {
        if !self.conda_env_file.parent().unwrap().exists() {
            std::fs::create_dir_all(self.conda_env_file.parent().unwrap())?;
        }
        if !self.backend_script.parent().unwrap().exists() {
            std::fs::create_dir_all(self.backend_script.parent().unwrap())?;
        }
        std::fs::write(&self.conda_env_file, CONDA_ENV_FILE)?;
        tracing::info!(
            "Created conda environment file at {}",
            self.conda_env_file.display()
        );
        std::fs::write(&self.backend_script, BACKEND_SCRIPT)?;
        tracing::info!(
            "Created backend script at {}",
            self.backend_script.display()
        );
        Ok(())
    }

    pub async fn init(&self) -> Result<()> {
        info!("Initializing environment");
        {
            let environ = self.environment.read().await;
            if let Err(e) = environ.load(ENVIRONMENT_NAME).await {
                error!("Demucs environment is not installed: {}", e);
                *self.status.write().await = Status::NotInstalled;
                return Ok(())
            }
        }
        let (backend, mut recv) = Backend::new(self.environment.clone(), self.backend_script.clone()).await?;
        let status = self.status.clone();
        let backend_clone = backend.clone();
        tokio::spawn(async move {
            while let Some(request) = recv.recv().await {
                match request {
                    FromBackendRequest::PythonBackendDroped | FromBackendRequest::RustBackendDroped => {
                        error!("Backend dropped");
                        *status.write().await = Status::Exited;
                        break;
                    },
                    FromBackendRequest::PythonBackendConnected => {
                        let version = backend_clone.version().await.unwrap();
                        info!("Backend connected, demucs version: {}, torch device: {}", version.version, version.torch_device);
                    },
                }
            }
        });

        *self.status.write().await = Status::Ready {
            backend: Arc::new(Mutex::new(backend)),
        };
        Ok(())
    }

    pub async fn demux(&self, input: PathBuf, output: PathBuf) -> Result<DemuxResult> {
        match &*self.status.read().await {
            Status::Ready { backend } => {
                info!("Demuxing {} to {}", input.display(), output.display());
                let result = backend.lock().await.demux(input, output).await?;
                Ok(result)
            }
            _ => {
                error!("Demucs not ready, failed to process demux task");
                eyre::bail!("Demucs not ready, failed to process demux task");
            }
        }
    }
}

impl std::fmt::Debug for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Status::?")
    }
}
