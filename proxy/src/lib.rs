use std::{env, net::SocketAddr};

use anyhow::{anyhow, Result};
use tokio::net::{TcpListener, TcpStream};

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
pub struct Proxy {}

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

        loop {
            let (socket, remote_addr) = listener.accept().await?;
            log::debug!("serving incoming connection from {}", remote_addr);
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(socket, remote_addr).await {
                    log::error!("failed to handle connection: {}", e);
                }
            });
        }
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
    async fn handle_connection(socket: TcpStream, remote_addr: SocketAddr) -> Result<()> {
        // todo(iverly): handle the handshake packet

        // todo(iverly): read the handshake packet and determine the server address
        // for the moment, just forward the connection to a static server
        let server_addr = "127.0.0.1:25566";

        let client_stream = Stream::wrap(socket);
        client_stream.configure()?;

        let server_stream = Stream::from(server_addr).await?;
        // todo(iverly): handle server connection errors & send back to the client
        server_stream.configure()?;

        // todo(iverly): send the handshake packet to the server

        Self::copy_streams(client_stream, server_stream).await?;

        log::debug!("connection closed from {}", remote_addr);
        Ok(())
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
