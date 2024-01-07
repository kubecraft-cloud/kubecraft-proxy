use std::sync::Arc;

use proto::proxy::Backend;
use storage::Storage;
use tokio::sync::{oneshot, Mutex};

use crate::tonic_backend_from_proxy;

pub struct ListBackendHandler {}

impl ListBackendHandler {
    /// It handles the `ListBackend` event.
    ///
    /// Arguments:
    ///
    /// * `storage`: Arc<Mutex<Storage>> - the storage object that holds all the backends
    /// * `backend`: The backend to add to the storage.
    /// * `tx`: This is the channel that the client is listening on.
    pub async fn handle(
        storage: Arc<Mutex<Storage>>,
        tx: oneshot::Sender<Result<Vec<Backend>, tonic::Status>>,
    ) {
        let storage = storage.lock().await;

        let backends = storage
            .get_backends()
            .clone()
            .into_values()
            .map(tonic_backend_from_proxy)
            .collect();

        let _ = tx.send(Ok(backends));
    }
}
