use std::fmt::Debug;

use anyhow::{anyhow, Result};
use protocol::packets::{
    clientbound,
    serverbound::{self, handshake::NextState},
};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpStream, ToSocketAddrs},
};

#[derive(Debug)]
pub struct Stream {
    tcp_stream: TcpStream,
}

impl Stream {
    /// This function takes a TcpStream and returns a Stream.
    ///
    /// Arguments:
    ///
    /// * `tcp_stream`: The TcpStream that we want to wrap.
    ///
    /// Returns:
    ///
    /// A new instance of the `TcpStreamWrapper` struct.
    pub fn wrap(tcp_stream: TcpStream) -> Self {
        Self { tcp_stream }
    }

    /// It connects to a server, and returns a `TcpStream` wrapped in a `Stream` that can be used to
    /// send and receive messages
    ///
    /// Arguments:
    ///
    /// * `server_addr`: The address of the server to connect to.
    ///
    /// Returns:
    ///
    /// A `Result<Self>`
    pub async fn from<A: ToSocketAddrs + Debug + Clone>(server_addr: A) -> Result<Self> {
        let tcp_stream = TcpStream::connect(server_addr.clone())
            .await
            .map_err(|e| anyhow!("Failed to connect to {:?}: {}", server_addr, e))?;

        Ok(Self::wrap(tcp_stream))
    }

    /// Configure the TCP stream.
    ///
    /// The first thing we do is call `set_nodelay` on the stream. This is a method that comes from the
    /// `TcpStream` type. It returns a `Result` that we can use to check if the call succeeded
    ///
    /// Returns:
    ///
    /// A Result<()>
    pub fn configure(&self) -> Result<()> {
        self.tcp_stream
            .set_nodelay(true)
            .map_err(|e| anyhow!("Failed to set nodelay on stream: {}", e))
    }

    /// It returns the tcp stream
    ///
    /// Returns:
    ///
    /// A TcpStream
    pub fn tcp_stream(self) -> TcpStream {
        self.tcp_stream
    }

    /// It reads a handshake from the stream
    ///
    /// Returns:
    ///
    /// A Result<Handshake>
    pub async fn read_handshake(&mut self) -> Result<serverbound::handshake::Handshake> {
        serverbound::handshake::Handshake::read(&mut self.tcp_stream).await
    }

    /// It writes a handshake to the stream
    ///
    /// Arguments:
    ///
    /// * `handshake`: The handshake to write to the stream.
    ///
    /// Returns:
    ///
    /// A Result<()>
    pub async fn write_handshake(
        &mut self,
        handshake: &serverbound::handshake::Handshake,
    ) -> Result<()> {
        handshake.write(&mut self.tcp_stream).await
    }

    /// It kicks the user with the message "Backend not found"
    ///
    /// Arguments:
    ///
    /// * `next_state`: Next state of the handshake
    ///
    /// Returns:
    ///
    /// A Result<()>
    pub async fn kick_backend_not_found(&mut self, next_state: NextState) -> Result<()> {
        // todo(iverly): get the message from the env for each next_state
        self.kick("Backend not found".to_string(), next_state).await
    }

    /// It kicks the user with the reason, then shuts down the TCP stream
    ///
    /// Arguments:
    ///
    /// * `reason`: The reason for the kick.
    /// * `next_state`: The next state the client will be in.
    ///
    /// Returns:
    ///
    /// Result<()>
    async fn kick(&mut self, reason: String, next_state: NextState) -> Result<()> {
        let status = clientbound::status::Status::from_error(reason);

        match next_state {
            NextState::Login => status.write_as_text(&mut self.tcp_stream).await,
            NextState::Status => status.write_as_motd(&mut self.tcp_stream).await,
        }?;

        self.tcp_stream.shutdown().await?;
        Ok(())
    }
}
