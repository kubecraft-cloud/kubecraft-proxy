use std::collections::BTreeMap;

use anyhow::Result;
use shared::models::backend::Backend;

/// The storage is responsible for storing the backends
#[derive(Debug, Default)]
pub struct Storage {
    backends: BTreeMap<String, Backend>,
}

impl Storage {
    /// Creates a new instance of the `Storage` struct
    ///
    /// Returns:
    ///
    /// A new instance of the struct.
    pub fn new() -> Self {
        Self::default()
    }

    /// It adds a new backend to the storage
    ///
    /// Arguments:
    ///
    /// * `backend` - The backend to add
    ///
    /// Returns:
    ///
    /// A Result<()>
    pub fn add_backend(&mut self, backend: Backend) -> Result<()> {
        self.backends
            .insert(backend.hostname().to_string(), backend);
        Ok(())
    }

    /// It removes a backend from the storage
    ///
    /// Arguments:
    ///
    /// * `host` - The host of the backend to remove
    ///
    /// Returns:
    ///
    /// A Result<()>
    pub fn remove_backend(&mut self, host: &str) -> Result<()> {
        self.backends.remove(host);
        Ok(())
    }

    /// It returns the backend with the specified host
    ///
    /// Arguments:
    ///
    /// * `host` - The host of the backend
    ///
    /// Returns:
    ///
    /// The backend with the specified host
    pub fn get_backend(&self, host: &str) -> Option<&Backend> {
        self.backends.get(host)
    }

    /// It returns all the backends
    ///
    /// Returns:
    ///
    /// All the backends
    pub fn get_backends(&self) -> &BTreeMap<String, Backend> {
        &self.backends
    }
}
