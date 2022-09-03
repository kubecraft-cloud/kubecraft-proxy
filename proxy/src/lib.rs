use std::{env, net::SocketAddr};

use anyhow::{anyhow, Result};
use tokio::net::{TcpListener, TcpStream};

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

        let mut client_stream = socket;
        let mut server_stream = Self::connect_to_server(server_addr).await?;
        // todo(iverly): handle server connection errors & send back to the client

        Self::set_stream_nodelay(&mut client_stream)?;
        Self::set_stream_nodelay(&mut server_stream)?;

        // todo(iverly): send the handshake packet to the server

        tokio::io::copy_bidirectional(&mut client_stream, &mut server_stream)
            .await
            .map_err(|e| anyhow!("failed to copy data between client and server: {}", e))?;

        log::debug!("connection closed from {}", remote_addr);
        Ok(())
    }

    /// It connects to a server and returns a `TcpStream` if successful
    ///
    /// Arguments:
    ///
    /// * `server_addr`: The address of the server to connect to.
    ///
    /// Returns:
    ///
    /// A Result<TcpStream>
    async fn connect_to_server(server_addr: &str) -> Result<TcpStream> {
        TcpStream::connect(server_addr)
            .await
            .map_err(|e| anyhow!("Failed to connect to {}: {}", server_addr, e))
    }

    /// `set_stream_nodelay` sets the TCP_NODELAY option on a TCP stream
    ///
    /// Arguments:
    ///
    /// * `stream`: The stream to set nodelay on.
    ///
    /// Returns:
    ///
    /// Result<()>
    fn set_stream_nodelay(stream: &mut TcpStream) -> Result<()> {
        stream
            .set_nodelay(true)
            .map_err(|e| anyhow!("Failed to set nodelay on stream: {}", e))
    }
}
