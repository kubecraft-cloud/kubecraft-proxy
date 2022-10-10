use std::{net::SocketAddr, str::FromStr};

use anyhow::{anyhow, Ok};
use log::error;
use proto::proxy::proxy_service_server::ProxyServiceServer;
use tokio::sync::mpsc;
use tonic::transport::Server;

use crate::{event::Event, listeners::proxy::ProxyListener};

pub mod event;
pub mod listeners;

pub struct Listener {
    addr: String,
}

impl Listener {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }

    /// It creates a gRPC server that listens on the address specified in the configuration, and sends
    /// events to the event loop
    ///
    /// Arguments:
    ///
    /// * `tx`: mpsc::Sender<Event>
    ///
    /// Returns:
    ///
    /// A JoinHandle<()>
    pub async fn start(&self, tx: mpsc::Sender<Event>) -> anyhow::Result<()> {
        let addr = SocketAddr::from_str(&self.addr).map_err(|e| {
            error!("failed to parse address: {}", e);
            anyhow!("failed to parse address: {}", e)
        })?;

        let proxy_listener = ProxyListener { sender: tx };

        Server::builder()
            .add_service(ProxyServiceServer::new(proxy_listener))
            .serve(addr)
            .await
            .map_err(|e| anyhow!("server exited with error {}", e))?;

        Ok(())
    }
}
