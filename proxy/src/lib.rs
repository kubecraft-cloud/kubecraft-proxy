use std::{env, sync::Arc};

use anyhow::{anyhow, Ok, Result};
use listener::{event::Event, Listener};
use log::debug;
use storage::Storage;
use tokio::{
    join,
    net::TcpListener,
    sync::{mpsc::Receiver, Mutex},
};

use crate::stream::Stream;

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
        let proxy_port = env::var("PROXY_PORT").unwrap_or_else(|_| "25565".to_string());
        let proxy_addr = format!("0.0.0.0:{}", proxy_port);

        log::info!("Starting proxy on {}", proxy_addr);
        let tcp_listener = TcpListener::bind(proxy_addr.clone())
            .await
            .map_err(|e| anyhow!("Failed to bind proxy to {}: {}", proxy_addr, e))?;

        let listener_port = env::var("LISTENER_PORT").unwrap_or_else(|_| "65535".to_string());
        let listener_addr = format!("0.0.0.0:{}", listener_port);

        log::info!("Starting listener on {}", listener_addr);
        let listener = Listener::new(listener_addr);

        // Start listener and pass it a channel to send events to the proxy
        let (tx, rx) = tokio::sync::mpsc::channel::<Event>(16);

        // Create the joins that will run in parallel
        let results = join!(
            Self::handle_connections(tcp_listener, self.storage.clone()),
            Self::handle_listener_events(rx, self.storage.clone()),
            listener.start(tx)
        );

        results
            .0
            .unwrap_or_else(|e| log::error!("proxy connection handler exited with error: {}", e));
        results
            .1
            .unwrap_or_else(|e| log::error!("listener event handler exited with error: {}", e));
        results
            .2
            .unwrap_or_else(|e| log::error!("listener exited with error: {}", e));

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

    async fn handle_listener_events(
        mut rx: Receiver<Event>,
        storage: Arc<Mutex<Storage>>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await.ok_or(anyhow!("failed to receive event"))?;
            debug!("handling event: {:?}", event);

            let storage = storage.clone();

            tokio::spawn(async move {
                match event {
                    Event::DeleteBackend(backend, tx) => {
                        tx.send(storage.lock().await.remove_backend(&backend.hostname))
                            .map_err(|_| {
                                log::error!("failed to send delete backend response");
                                anyhow!("failed to send delete backend response")
                            })?;
                    }
                    Event::PutBackend(backend, tx) => {
                        tx.send(storage.lock().await.add_backend(
                            shared::models::backend::Backend::new(
                                backend.hostname,
                                backend.redirect_ip,
                                backend.redirect_port,
                            ),
                        ))
                        .map_err(|_| {
                            log::error!("failed to send put backend response");
                            anyhow!("failed to send put backend response")
                        })?;
                    }
                    Event::ListBackends(tx) => {
                        tx.send(Ok(storage
                            .lock()
                            .await
                            .get_backends()
                            .iter()
                            .map(|backend| shared::models::backend::Backend {
                                hostname: backend.1.hostname().to_string(),
                                redirect_ip: backend.1.redirect_ip().to_string(),
                                redirect_port: backend.1.redirect_port(),
                            })
                            .collect::<Vec<_>>()))
                            .map_err(|_| {
                                log::error!("failed to send list backends response");
                                anyhow!("failed to send list backends response")
                            })?;
                    }
                }
                Ok(())
            });
        }
    }
}
