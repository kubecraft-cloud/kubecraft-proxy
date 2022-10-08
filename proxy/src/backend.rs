/// A backend is a Minecraft server that the proxy can connect to.
///
/// Properties:
///
/// * `host`: The hostname of the backend server.
/// * `port`: The port that the backend server is listening on.
#[derive(Debug, Clone)]
pub struct Backend {
    hostname: String,
    redirect_ip: String,
    redirect_port: u16,
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
    pub fn new(hostname: String, redirect_ip: String, redirect_port: u16) -> Self {
        Self {
            hostname,
            redirect_ip,
            redirect_port,
        }
    }

    /// It returns the host of the backend
    ///
    /// Returns:
    ///
    /// The host of the backend
    pub fn hostname(&self) -> &str {
        self.hostname.as_str()
    }

    /// It returns the ip of the backend
    ///
    /// Returns:
    ///
    /// The ip of the backend
    pub fn redirect_ip(&self) -> &str {
        self.redirect_ip.as_str()
    }

    /// It returns the port of the backend
    ///
    /// Returns:
    ///
    /// The port of the backend
    pub fn redirect_port(&self) -> u16 {
        self.redirect_port
    }

    /// It returns the address of the backend
    ///
    /// Returns:
    ///
    /// The address of the backend
    pub fn addr(&self) -> String {
        self.redirect_ip.clone() + ":" + &self.redirect_port.to_string()
    }
}
