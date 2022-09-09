use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::{write_string, write_var_int};

#[derive(Debug, Default)]
pub struct Status {
    error: Option<String>,
}

impl Status {
    /// Creates a new instance of the `Status` struct
    ///
    /// Returns:
    ///
    /// A new instance of the struct.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new instance of the `Status` struct
    ///
    /// Arguments:
    ///
    /// * `error`: Option<String> - The error message
    ///
    /// Returns:
    ///
    /// A new instance of the struct.
    pub fn from_error(error: String) -> Self {
        Self { error: Some(error) }
    }

    /// It writes the status packet to a stream as a response to a handshake
    /// packet with login as the next state
    ///
    /// Arguments:
    ///
    /// * `stream`: The stream to write to.
    ///
    /// Returns:
    ///
    /// A Result<()>
    pub async fn write_as_text(&self, stream: &mut TcpStream) -> Result<()> {
        let error = self.error.clone().unwrap(); // todo(iverly): handle error

        let mut data = Vec::new();
        write_var_int(&mut data, 0).await?;
        write_string(&mut data, format!("{{\"text\": \"{}\"}}", error).as_str()).await?;

        write_var_int(stream, data.len() as i32).await?;
        stream.write_all(&data).await?;
        Ok(())
    }

    /// It writes the status packet to a stream as a response to a handshake
    /// packet with status as the next state
    ///
    /// Arguments:
    ///
    /// * `stream`: The stream to write to.
    ///
    /// Returns:
    ///
    /// A Result<()>
    pub async fn write_as_motd(&self, stream: &mut TcpStream) -> Result<()> {
        let error = self.error.clone().unwrap(); // todo(iverly): handle error

        let mut data = Vec::new();
        write_var_int(&mut data, 0).await?;
        write_string(
            &mut data,
            format!(
                "{{
                    \"version\": {{
                        \"name\": \"\",
                        \"protocol\": -1
                    }},
                    \"players\": {{
                        \"max\": 0,
                        \"online\": 0,
                        \"sample\": []
                    }},
                    \"description\": {{
                        \"text\": \"{}\"
                    }}
                }}",
                error
            )
            .as_str(),
        )
        .await?;

        write_var_int(stream, data.len() as i32).await?;
        stream.write_all(&data).await?;
        Ok(())
    }
}
