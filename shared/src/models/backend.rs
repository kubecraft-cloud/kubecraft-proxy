#[derive(Debug, Clone)]
pub struct Backend {
    pub hostname: String,
    pub redirect_ip: String,
    pub redirect_port: u16,
}
