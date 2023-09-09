use std::{env, sync::Arc};

use anyhow::{anyhow, Ok, Result};
use storage::Storage;
use tokio::{join, net::TcpListener, sync::Mutex};

use crate::stream::Stream;

pub mod backend;
pub mod storage;
pub mod stream;

/// The proxy is responsible for accepting connections from the client and
/// forwarding them to the correct server.
///
/// The proxy reads the handshake packet from the client and determines the
/// server to connect to. It then connects to the server and forwards the
/// handshake packet to the server. The proxy then forwards all packets between
/// the client and the server.
///
/// The proxy is responsible for keeping track of the server's state and
/// forwarding packets to the correct client.
#[derive(Debug, Default)]
pub struct Proxy {
    storage: Arc<Mutex<Storage>>,
}

impl Proxy {
    /// Creates a new instance of the `Proxy` struct
    ///
    /// Returns:
    ///
    /// A new instance of the struct.
    pub fn new() -> Self {
        Self::default()
    }

    /// It listens for incoming connections on the port specified by the `PROXY_PORT` environment
    /// variable, and spawns a new task to handle each connection
    ///
    /// Returns:
    ///
    /// A Result<()>
    pub async fn start(&self) -> Result<()> {
        let port = env::var("PROXY_PORT").unwrap_or_else(|_| "25565".to_string());
        let addr = format!("0.0.0.0:{}", port);

        log::info!("Starting proxy on {}", addr);
        let listener = TcpListener::bind(addr.clone())
            .await
            .map_err(|e| anyhow!("Failed to bind proxy to {}: {}", addr, e))?;

        let results = join!(Self::handle_connections(listener, self.storage.clone()));

        results
            .0
            .unwrap_or_else(|e| log::error!("proxy connection handler exited with error: {}", e));

        Ok(())
    }

    /// It reads the handshake packet from the client, connects to the server, and then forwards all
    /// data between the client and the server
    ///
    /// Arguments:
    ///
    /// * `socket`: The socket that the client connected to.
    /// * `remote_addr`: The address of the client that connected to the proxy
    ///
    /// Returns:
    ///
    /// A Result<()>
    async fn handle_connections(listener: TcpListener, storage: Arc<Mutex<Storage>>) -> Result<()> {
        loop {
            let (socket, remote_addr) = listener.accept().await?;
            log::debug!("serving incoming connection from {}", remote_addr);

            let storage = storage.clone();

            // Handle connection in parallel
            tokio::spawn(async move {
                let mut client_stream = Stream::wrap(socket);
                client_stream.configure().map_err(|e| {
                    let err_msg = format!(
                        "failed to configure client stream for {}: {}",
                        remote_addr, e
                    );
                    log::error!("{}", err_msg);
                    anyhow!(err_msg)
                })?;

                let mut handshake = client_stream.read_handshake().await.map_err(|e| {
                    let err_msg = format!(
                        "failed to read handshake packet from client {}: {}",
                        remote_addr, e
                    );
                    log::error!("{}", err_msg);
                    anyhow!(err_msg)
                })?;

                log::debug!(
                    "client {} trying to connect to {}",
                    remote_addr,
                    handshake.hostname()
                );

                let (backend_addr, backend_host) = match storage
                    .lock()
                    .await
                    .get_backend(handshake.hostname().as_str())
                {
                    Some(backend) => (backend.addr(), backend.redirect_ip().to_string()),
                    None => {
                        client_stream
                            .kick_backend_not_found(handshake.next_state())
                            .await
                            .map_err(|e| {
                                let err_msg =
                                    format!("failed to kick client {}: {}", remote_addr, e);
                                log::error!("{}", err_msg);
                                anyhow!(err_msg)
                            })?;
                        return Err(anyhow!(
                            "failed to handle connection, unable to find hostname: {}",
                            handshake.hostname()
                        ));
                    }
                };

                log::debug!("forwarding client packets to {}", backend_addr);

                let mut server_stream = Stream::from(&backend_addr).await?;
                server_stream.configure().map_err(|e| {
                    let err_msg = format!(
                        "failed to configure server stream for {}: {}",
                        backend_addr, e
                    );
                    log::error!("{}", err_msg);
                    anyhow!(err_msg)
                })?;

                // rewrite handshake packet to use the backend's IP
                handshake.set_hostname(backend_host);
                server_stream
                    .write_handshake(&handshake)
                    .await
                    .map_err(|e| {
                        let err_msg = format!(
                            "failed to write handshake packet to server {}: {}",
                            backend_addr, e
                        );
                        log::error!("{}", err_msg);
                        anyhow!(err_msg)
                    })?;

                Self::copy_streams(client_stream, server_stream)
                    .await
                    .map_err(|e| {
                        let err_msg = format!(
                            "failed to copy streams between client {} and server {}: {}",
                            remote_addr, backend_addr, e
                        );
                        log::error!("{}", err_msg);
                        anyhow!(err_msg)
                    })?;

                log::debug!("connection closed from {}", remote_addr);

                Ok(())
            });
        }
    }

    /// It copies data from the client to the server and vice versa
    ///
    /// Arguments:
    ///
    /// * `client_stream`: The stream that the client is connected to.
    /// * `server_stream`: The stream to the server.
    ///
    /// Returns:
    ///
    /// A future that resolves to a Result<()>
    async fn copy_streams(client_stream: Stream, server_stream: Stream) -> Result<()> {
        let mut client_tcp_stream = client_stream.tcp_stream();
        let mut server_tcp_stream = server_stream.tcp_stream();

        tokio::io::copy_bidirectional(&mut client_tcp_stream, &mut server_tcp_stream)
            .await
            .map_err(|e| anyhow!("failed to copy data between client and server: {}", e))?;

        Ok(())
    }
}
