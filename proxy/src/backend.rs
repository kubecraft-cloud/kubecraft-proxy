use std::net::SocketAddr;

/// A backend is a Minecraft server that the proxy can connect to.
///
/// Properties:
///
/// * `host`: The hostname of the backend server.
/// * `port`: The port that the backend server is listening on.
#[derive(Debug, Clone)]
pub struct Backend {
    host: String,
    port: u16,
}

impl Backend {
    /// Creates a new instance of the `Backend` struct
    ///
    /// Arguments:
    ///
    /// * `host` - The host of the backend
    /// * `port` - The port of the backend
    ///
    /// Returns:
    ///
    /// A new instance of the struct.
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }

    /// It returns the host of the backend
    ///
    /// Returns:
    ///
    /// The host of the backend
    pub fn host(&self) -> &str {
        self.host.as_str()
    }

    /// It returns the port of the backend
    ///
    /// Returns:
    ///
    /// The port of the backend
    pub fn port(&self) -> u16 {
        self.port
    }

    /// It returns the address of the backend
    ///
    /// Returns:
    ///
    /// The address of the backend
    pub fn addr(&self) -> SocketAddr {
        SocketAddr::new(self.host.parse().unwrap(), self.port)
    }
}
