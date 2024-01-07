use proto::proxy::Backend;

use tokio::sync::oneshot;

pub mod handlers;

#[derive(Debug)]
pub enum Event {
    // Backend events
    ListBackend(oneshot::Sender<Result<Vec<Backend>, tonic::Status>>),
    PutBackend(Backend, oneshot::Sender<Result<(), tonic::Status>>),
    DeleteBackend(Backend, oneshot::Sender<Result<(), tonic::Status>>),
}

/// It takes a `proto::proxy::Backend` and returns a `proxy::backend::Backend`
///
/// Arguments:
///
/// * `backend`: proto::proxy::Backend
///
/// Returns:
///
/// A proxy::backend::Backend struct
pub fn proxy_backend_from_tonic(backend: Backend) -> shared::models::backend::Backend {
    shared::models::backend::Backend::new(
        backend.hostname,
        backend.redirect_ip,
        backend.redirect_port as u16,
    )
}

/// It takes a `proxy::backend::Backend` and returns a `proto::proxy::Backend`
///
/// Arguments:
///
/// * `backend`: proxy::backend::Backend
///
/// Returns:
///
/// A proto::proxy::Backend struct
pub fn tonic_backend_from_proxy(backend: shared::models::backend::Backend) -> Backend {
    Backend {
        hostname: backend.hostname().to_string(),
        redirect_ip: backend.redirect_ip().to_string(),
        redirect_port: backend.redirect_port() as u32,
    }
}
