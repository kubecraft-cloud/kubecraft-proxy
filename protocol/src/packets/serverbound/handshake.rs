use std::io::Cursor;

use anyhow::{anyhow, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::{read_string, read_var_int, write_string, write_var_int};

/// `Handshake` is a struct that contains a version, a host, a port, and a next state.
///
/// See [here](https://wiki.vg/Protocol#Serverbound) for more information.
///
/// Properties:
///
/// * `version`: The version of the protocol that the client is using.
/// * `host`: The hostname of the server.
/// * `port`: The port that the server is running on.
/// * `next_state`: This is the next state that the client will be in.
#[derive(Debug)]
pub struct Handshake {
    version: i32,
    hostname: String,
    port: u16,
    next_state: NextState,
}

impl Handshake {
    /// It reads the handshake packet from a stream and returns a `Handshake` struct
    ///
    /// Arguments:
    ///
    /// * `stream`: The stream to read from.
    ///
    /// Returns:
    ///
    /// A Result<Self>
    pub async fn read<T>(stream: &mut T) -> Result<Self>
    where
        T: AsyncReadExt + std::marker::Unpin,
    {
        let size = read_var_int(stream).await?;

        let mut data = vec![0u8; size as usize];
        stream.read_exact(&mut data).await?;
        let mut data = Cursor::new(data);

        let id = read_var_int(&mut data).await?;
        if id != 0 {
            return Err(anyhow!("invalid handshake packet id: {}", id));
        }

        let version = read_var_int(&mut data).await?;
        let hostname = read_string(&mut data).await?;
        let port = data.read_u16().await?;
        let next_state = NextState::from_i32(read_var_int(&mut data).await?)?;

        Ok(Self {
            version,
            hostname,
            port,
            next_state,
        })
    }

    /// It writes the packet to the stream
    ///
    /// Arguments:
    ///
    /// * `stream`: The stream to write to.
    ///
    /// Returns:
    ///
    /// A Result<()>
    pub async fn write(&self, stream: &mut TcpStream) -> Result<()> {
        let mut data = Vec::new();
        write_var_int(&mut data, 0).await?;
        write_var_int(&mut data, self.version).await?;
        write_string(&mut data, &self.hostname).await?;
        data.append(&mut self.port.to_be_bytes().to_vec());
        write_var_int(&mut data, self.next_state.to_i32()).await?;

        write_var_int(stream, data.len() as i32).await?;
        stream.write_all(&data).await?;

        Ok(())
    }

    /// It returns the version of the handshake packet
    ///
    /// Returns:
    ///
    /// The version of the object.
    pub fn version(&self) -> i32 {
        self.version
    }

    /// It returns the host of the handshake packet
    ///
    /// Returns:
    ///
    /// String
    pub fn hostname(&self) -> String {
        self.hostname.to_string()
    }

    /// It returns the port of the handshake packet
    ///
    /// Returns:
    ///
    /// The port number
    pub fn port(&self) -> u16 {
        self.port
    }

    /// It returns the `NextState` of the handshake packet
    ///
    /// Returns:
    ///
    /// The next state of the game.
    pub fn next_state(&self) -> NextState {
        self.next_state
    }
}

/// `NextState` is an enum that contains the next state of the game.
/// It can be either `Status` or `Login`.
///
/// See [here](https://wiki.vg/Protocol#Serverbound) for more information.
///
/// Properties:
///
/// * `Status`: The next state is the status state.
/// * `Login`: The next state is the login state.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NextState {
    Status,
    Login,
}

impl NextState {
    /// It converts an i32 to a `NextState`
    ///
    /// Arguments:
    ///
    /// * `num`: i32 - The number to convert to a NextState
    ///
    /// Returns:
    ///
    /// A Result<NextState>
    fn from_i32(num: i32) -> Result<NextState> {
        Ok(match num {
            1 => Self::Status,
            2 => Self::Login,
            _ => return Err(anyhow!("Cannot convert {} to NextState", num)),
        })
    }

    /// It converts a `NextState` to an i32
    ///
    /// Returns:
    ///
    /// i32 - The number that represents the `NextState`
    fn to_i32(self) -> i32 {
        match self {
            Self::Status => 1,
            Self::Login => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_read() {
        let mut stream = &b"\x0f\x00\x6e\x09\x6c\x6f\x63\x61\x6c\x68\x6f\x73\x74\x63\xdd\x01"[..];

        let handshake = Handshake::read(&mut stream).await.unwrap();

        assert_eq!(handshake.version(), 110);
        assert_eq!(handshake.hostname(), "localhost");
        assert_eq!(handshake.port(), 25565);
        assert_eq!(handshake.next_state(), NextState::Status);
    }
}
