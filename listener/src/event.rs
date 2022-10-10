use shared::models::backend::Backend;
use tokio::sync::oneshot;

/// Event is an enum that represents the different events that can be sent to the proxy
#[derive(Debug)]
pub enum Event {
    ListBackends(oneshot::Sender<anyhow::Result<Vec<Backend>>>),
    PutBackend(Backend, oneshot::Sender<anyhow::Result<()>>),
    DeleteBackend(Backend, oneshot::Sender<anyhow::Result<()>>),
}
