use std::{net::IpAddr, sync::atomic::AtomicUsize};

use crate::{prelude::*, ENVIRONMENT_NAME};
use futures_util::StreamExt;
use macromamba::Environment;
use tokio::{
    io::AsyncWriteExt,
    net::TcpListener,
    sync::{
        mpsc::{Receiver, Sender},
        OwnedRwLockWriteGuard,
    },
    task::JoinHandle,
};
use tokio_util::codec::{BytesCodec, FramedRead};
use tracing::{error, info, warn};

pub enum FromBackendRequest {
    RustBackendDroped,
    PythonBackendConnected,
    PythonBackendDroped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "id")]
pub enum FromProcessRequest {
    Ack {
        request: serde_json::Value,
    },
    Log {
        message: String,
        level: String,
    },
    CallBack {
        call_id: usize,
        payload: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "id")]
pub enum ToProcessRequest {
    Call {
        procedure_id: &'static str,
        /// Id of the specific call
        call_id: usize,
        /// Payload passed to python procedure
        payload: serde_json::Value,
    },
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct MarshalledCall {
    /// Maintain the caller idle until the call is resolved
    lock: OwnedRwLockWriteGuard<serde_json::Value>,
    call_id: usize,
}

#[derive(Debug, Clone)]
pub struct Backend {
    last_call_id: Arc<AtomicUsize>,
    marshalled: Arc<RwLock<HashMap<usize, MarshalledCall>>>,
    // ipc_server_handle: JoinHandle<Result<()>>,
    // python_process_handle: JoinHandle<Result<()>>,
    pub proxy_send: Sender<ToProcessRequest>,
}

impl Backend {
    pub async fn new(
        environment: Arc<RwLock<Environment>>,
        backend_script: PathBuf,
    ) -> Result<(Self, Receiver<FromBackendRequest>)> {
        let (to_handler, from_backend) = tokio::sync::mpsc::channel(1024);
        let server: RemoteProcessServer = RemoteProcessServer::new(to_handler.clone()).await?;
        let port = server.port.to_string();
        let marshalled = server.marshalled.clone();
        let proxy_send = server.proxy_send.clone();
        let ipc_server_handle = tokio::spawn(async move { server.handle_connection().await });
        let to_handler_clone = to_handler.clone();
        let python_process_handle: JoinHandle<Result<()>> = tokio::spawn(async move {
            let mut cmd = environment
                .read()
                .await
                .run_script_within_env(ENVIRONMENT_NAME, backend_script.to_str().unwrap())
                .await?;
            cmd.env("PORT", &port);
            cmd.is_success().await?;
            info!("Python backend dropped");
            to_handler_clone.send(FromBackendRequest::PythonBackendDroped).await?;
            Ok(())
        });
        Ok((Self {
            last_call_id: Arc::new(AtomicUsize::new(0)),
            marshalled,
            proxy_send,
        }, from_backend))
    }

    pub async fn remote_call(
        &self,
        procedure_id: &'static str,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let lock = Arc::new(RwLock::new(serde_json::Value::Null));
        let guard = lock.clone().write_owned().await;
        let call_id = self.last_call_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.marshalled.write().await.insert(call_id, MarshalledCall { lock: guard, call_id });
        self.proxy_send.send(ToProcessRequest::Call { procedure_id, call_id, payload }).await?;
        info!("waiting for call {} to resolve", call_id);
        let value = lock.write().await;
        Ok(value.clone())
    }

    pub async fn version(&self) -> Result<VersionResult> {
        let result = self.remote_call("Version", serde_json::Value::Null).await?;
        Ok(serde_json::from_value(result)?)
    }
}

#[derive(Deserialize, Debug)]
pub struct VersionResult {
    pub version: String,
    pub torch_device: String,
}

struct RemoteProcessServer {
    marshalled: Arc<RwLock<HashMap<usize, MarshalledCall>>>,
    listener: TcpListener,
    to_handler: Sender<FromBackendRequest>,
    proxy_recv: Receiver<ToProcessRequest>,
    proxy_send: Sender<ToProcessRequest>,
    pub port: u16,
}

impl RemoteProcessServer {
    pub async fn new(to_handler: Sender<FromBackendRequest>) -> Result<Self> {
        let mut port = 8081;
        for _ in 0..10 {
            match tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await {
                Ok(listener) => {
                    let (proxy_send, proxy_recv) = tokio::sync::mpsc::channel(1024);
                    return Ok(Self {
                        marshalled: Arc::new(RwLock::new(HashMap::new())),
                        listener,
                        to_handler,
                        proxy_recv,
                        proxy_send,
                        port,
                    });
                }
                Err(e) => {
                    warn!("Failed to bind to port {}: {}", port, e);
                    port += 1;
                }
            }
        }
        Err(eyre::eyre!("failed to start IPC server"))
    }

    async fn handle_connection(mut self) -> Result<()> {
        info!(
            "Waiting for remote process connection on {}",
            self.listener.local_addr()?
        );
        let (mut connection, addr) = self.listener.accept().await?;
        info!("Remote process connected from {}", addr);
        self.to_handler.send(FromBackendRequest::PythonBackendConnected).await?;

        let (read, mut write) = connection.split();
        let mut framed = FramedRead::new(read, BytesCodec::new());
        let mut buffer: Vec<u8> = Vec::with_capacity(1024 * 1024);
        loop {
            tokio::select! {
                from_socket = framed.next() => {
                    match from_socket {
                        Some(Ok(bytes)) => {
                            tracing::trace!(bytes=bytes.len(), "received bytes from rpc socket");
                            buffer.extend(bytes);
                            while buffer.len() > 4 {
                                let packet_len = i32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);

                                if buffer.len() - 4 < packet_len as usize {
                                    continue;
                                }

                                let range = 4 .. (packet_len as usize + 4);

                                match serde_json::from_slice::<FromProcessRequest>(&buffer[range.clone()]) {
                                    Ok(packet) => {
                                        match self.handle_packet(packet).await {
                                            Ok(_) => (),
                                            Err(e) => error!("failed to handle packet: {:?}", e),
                                        }
                                    },
                                    Err(e) => {
                                        let raw = String::from_utf8_lossy(&buffer[range]).to_string();
                                        error!(packet=raw, packet_len=packet_len, "invalide packet: {:?}", e)
                                    }
                                }
                                buffer = Vec::from(&buffer[packet_len as usize + 4..]);
                            }
                        }
                        Some(Err(err)) => {
                            tracing::error!("Socket closed with error: {:?}", err);
                            self.to_handler.send(FromBackendRequest::RustBackendDroped).await?;
                            return Ok(())
                        },
                        _ => {}
                    }
                }
                from_proxy = self.proxy_recv.recv() => {
                    tracing::trace!("Received request from proxy, forwarding to python process");
                    match from_proxy {
                        Some(request) => {
                            let packet = serde_json::to_vec(&request).unwrap();
                            let packet: Vec<u8> = [&(packet.len() as i32).to_be_bytes(), &packet[..]].concat();
                            write.write_all(&packet[..]).await?;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    async fn handle_packet(&self, packet: FromProcessRequest) -> Result<()> {
        match packet {
            FromProcessRequest::Ack { .. } => Ok(()),
            FromProcessRequest::Log { message, level } => {
                if level.to_lowercase() == "error" {
                    error!(from = "demucs backend", level = level, "{}", message);
                } else {
                    info!(from = "demucs backend", level = level, "{}", message);
                }
                Ok(())
            }
            FromProcessRequest::CallBack { call_id, payload } => {
                info!(call_id = call_id, "got callback");
                match self.marshalled.write().await.remove(&call_id) {
                    Some(mut marshall) => {
                        info!(call_id = call_id, payload = ?payload, "marshalled callback resolved");
                        *marshall.lock = payload;
                    }
                    None => {
                        error!("retrive callback value but the marshalled call does not exists !");
                    }
                }

                Ok(())
            }
        }
    }
}
