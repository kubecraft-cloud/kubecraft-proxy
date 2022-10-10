use std::{net::SocketAddr, str::FromStr, sync::Arc};

use anyhow::{anyhow, Ok, Result};
use log::{debug, error, info};
use proto::proxy::proxy_service_server::ProxyServiceServer;
use tokio::{sync::mpsc, task::JoinHandle};
use tonic::transport::Server;

use crate::{event::Event, listeners::proxy::ProxyListener};

pub mod event;
pub mod listeners;

/// Listener configuration
/// 
/// Properties:
/// 
/// * `host`: The host of the server.
/// * `port`: The port that the server will listen on.
pub struct Config {
    pub host: String,
    pub port: u16,
}

/// The listener is responsible for handling gRPC requests to configure the proxy
/// 
/// All the requests are forwarded to the proxy via a channel.
pub struct Listener {
    config: Arc<Config>,
}

impl Listener {
    /// Creates a new instance of the `Listener` struct
    /// 
    /// Arguments:
    /// 
    /// * `config`: Arc<Config>
    /// 
    /// Returns:
    /// 
    /// A new instance of the `Config` struct.
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
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
    pub fn create_grpc_server(&self, tx: mpsc::Sender<Event>) -> Result<JoinHandle<()>> {
        info!("creating grpc server ...");
        let addr = SocketAddr::from_str(&format!("{}:{}", self.config.host, self.config.port))
            .map_err(|e| {
                error!("failed to parse address: {}", e);
                anyhow!("failed to parse address: {}", e)
            })?;

        debug!("creating proxy listener ...");
        let proxy_listener = ProxyListener { sender: tx };

        Ok(tokio::spawn(async move {
            Server::builder()
                .add_service(ProxyServiceServer::new(proxy_listener))
                .serve(addr)
                .await
                .unwrap();
        }))
    }
}
