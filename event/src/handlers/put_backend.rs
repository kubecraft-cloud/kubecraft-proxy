use std::sync::Arc;

use anyhow::{anyhow, Result};
use shared::models::backend::Backend;
use storage::Storage;
use tokio::sync::{oneshot, Mutex};

pub struct PutBackendHandler {}

impl PutBackendHandler {
    /// It handles the `PutBackend` event.
    ///
    /// Arguments:
    ///
    /// * `storage`: Arc<Mutex<Storage>> - the storage object that holds all the backends
    /// * `backend`: The backend to add to the storage.
    /// * `tx`: This is the channel that the client is listening on.
    pub async fn handle(
        storage: Arc<Mutex<Storage>>,
        backend: Backend,
        tx: oneshot::Sender<Result<()>>,
    ) {
        let mut storage = storage.lock().await;

        let result = storage
            .add_backend(backend)
            .map_err(|e| anyhow!(format!("Failed to add backend: {}", e)));

        let _ = tx.send(result);
    }
}
