use std::sync::Arc;

use proto::proxy::Backend;
use storage::Storage;
use tokio::sync::{oneshot, Mutex};

pub struct DeleteBackendHandler {}

impl DeleteBackendHandler {
    /// It handles the `DeleteBackend` event.
    ///
    /// Arguments:
    ///
    /// * `storage`: Arc<Mutex<Storage>> - the storage object that holds all the backends
    /// * `backend`: The backend to add to the storage.
    /// * `tx`: This is the channel that the client is listening on.
    pub async fn handle(
        storage: Arc<Mutex<Storage>>,
        backend: Backend,
        tx: oneshot::Sender<Result<(), tonic::Status>>,
    ) {
        let mut storage = storage.lock().await;

        let result = storage
            .remove_backend(&backend.hostname)
            .map_err(|e| tonic::Status::internal(format!("Failed to delete backend: {}", e)));

        let _ = tx.send(result);
    }
}