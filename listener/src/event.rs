use std::io::Error;

use proto::proxy::Backend;
use tokio::sync::oneshot;

/// Event is an enum that represents the different events that can be sent to the proxy
#[derive(Debug)]
pub enum Event {
    ListBackends(oneshot::Sender<Result<Vec<Backend>, Error>>),
    PutBackend(Backend, oneshot::Sender<Result<(), Error>>),
    DeleteBackend(Backend, oneshot::Sender<Result<(), Error>>),
}
