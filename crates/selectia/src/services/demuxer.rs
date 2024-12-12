use std::ops::Deref;

use demucs::{backend::DemuxResult, Demucs};

use crate::prelude::*;

#[derive(Debug, Clone, Task)]
pub enum DemuxerTask {
    Demux {
        input: PathBuf,
        output: PathBuf,
        callback: TaskCallback<DemuxResult>,
    },
    StatusUpdate {
        status: DemuxerStatus,
    },
}

#[derive(Debug, Clone, Task)]
pub enum DemuxerEvent {
    StatusUpdate { status: DemuxerStatus },
}

#[derive(Clone)]
pub enum DemuxerStatus {
    None,
    Loading,
    Installing,
    Ready {
        demucs: Demucs,
    },
    NotInstalled,
}

impl std::fmt::Debug for DemuxerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DemuxerStatus::?")
    }
}

impl PartialEq for DemuxerStatus {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DemuxerStatus::None, DemuxerStatus::None) => true,
            (DemuxerStatus::Loading, DemuxerStatus::Loading) => true,
            (DemuxerStatus::Installing, DemuxerStatus::Installing) => true,
            (DemuxerStatus::Ready { .. }, DemuxerStatus::Ready { .. }) => true,
            (DemuxerStatus::NotInstalled, DemuxerStatus::NotInstalled) => true,
            _ => false,
        }
    }
}

#[singleton_service(DemuxerSingleton)]
pub async fn demuxer(ctx: ServiceContext, mut rx: ServiceReceiver<DemuxerTask>, dispatcher: EventDispatcher<DemuxerEvent>,  data_path: PathBuf) -> Result<()> {
    let mut current_status = DemuxerStatus::None;

    // Trigger status update to perform first check
    let introspect_address = ctx.get_singleton_address::<DemuxerSingleton>().await?;
    introspect_address.send(DemuxerTask::StatusUpdate {
        status: DemuxerStatus::None,
    }).await?;

    while let Some(task) = rx.recv().await {
        match task {
            DemuxerTask::StatusUpdate { status } => {
                current_status = status;
                if current_status == DemuxerStatus::NotInstalled {
                    info!("Demuxer not installed, installing it ...");
                    tokio::spawn(install_demuxer(data_path.clone(), introspect_address.clone()));
                } else if current_status == DemuxerStatus::None {
                    info!("Demuxer ready, loading it ...");
                    load_demuxer(data_path.clone(), introspect_address.clone()).await?;
                }
            }
            DemuxerTask::Demux {
                input,
                output,
                callback,
            } => {
                match &current_status {
                    DemuxerStatus::Ready { demucs } => {
                        tokio::fs::create_dir_all(&output).await?;
                        let result = demucs.demux(input, output.clone()).await?;
                        callback.resolve(result).await?;
                    }
                    _ => {
                        error!("Demuxer not ready, failed to process demux task");
                    }
                }
            }
        }
    }
    Ok(())
}

async fn load_demuxer(data_path: PathBuf, sender: AddressableService<DemuxerTask>) -> Result<()> {
    sender
        .send(DemuxerTask::StatusUpdate {
            status: DemuxerStatus::Loading,
        })
        .await?;
    info!("Initializing demuxer");
    match demucs::Demucs::new(data_path).await {
        Ok(demucs) => {
            if let Err(e) = demucs.init().await {
                warn!("Failed to initialize demuxer: {:?}", e);
            }
            let status = demucs.status.read().await;
            match &*status {
                demucs::Status::Ready { backend } => {
                    sender
                        .send(DemuxerTask::StatusUpdate {
                            status: DemuxerStatus::Ready {
                                demucs: demucs.clone(),
                            }
                        })
                        .await?
                }
                _ => {
                    sender
                        .send(DemuxerTask::StatusUpdate {
                            status: DemuxerStatus::NotInstalled,
                        })
                        .await?
                }
            }
        }
        Err(e) => {
            warn!("Failed to initialize demuxer: {:?}", e);
            sender
                .send(DemuxerTask::StatusUpdate {
                    status: DemuxerStatus::NotInstalled,
                })
                .await?;
        }
    }
    Ok(())
}

async fn install_demuxer(
    data_path: PathBuf,
    sender: AddressableService<DemuxerTask>,
) -> Result<()> {
    sender
        .send(DemuxerTask::StatusUpdate {
            status: DemuxerStatus::Installing,
        })
        .await?;
    info!("Initializing demuxer");
    let Ok(demucs) = demucs::Demucs::new(data_path).await else {
        info!("Failed to initialize demuxer !");
        sender
            .send(DemuxerTask::StatusUpdate {
                status: DemuxerStatus::NotInstalled,
            })
            .await?;
        return Ok(());
    };
    match demucs.install().await {
        Ok(()) => {
            // Trigger reload
            sender
                .send(DemuxerTask::StatusUpdate {
                    status: DemuxerStatus::None,
                })
                .await?
        }
        _ => {
            sender
                .send(DemuxerTask::StatusUpdate {
                    status: DemuxerStatus::NotInstalled,
                })
                .await?
        }
    }
    Ok(())
}
