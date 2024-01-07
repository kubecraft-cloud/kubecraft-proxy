use async_trait::async_trait;
use log::{debug, error, trace};
use proto::proxy::{proxy_service_server::ProxyService, Backend};
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use crate::event::Event;

/// It is the gRPC server that handles the requests concerning the proxy configuration
///
/// Properties:
///
/// * `sender`: This is a channel that will be used to send events to the proxy.
pub struct ProxyListener {
    pub sender: mpsc::Sender<Event>,
}

#[async_trait]
impl ProxyService for ProxyListener {
    type ListBackendStream = ReceiverStream<Result<Backend, Status>>;

    /// Tt sends a message to the proxy to list all backend configurations and returns the response
    ///
    /// Arguments:
    ///
    /// * `request`: Request<()>
    ///
    /// Returns:
    ///
    /// A `Response` with a `ReceiverStream` of `Backend`s.
    async fn list_backend(
        &self,
        request: Request<()>,
    ) -> Result<Response<Self::ListBackendStream>, Status> {
        trace!("received request: {:?}", request);

        trace!("creating oneshot channel to communicate with the proxy");
        let (tx, rx) = oneshot::channel::<anyhow::Result<Vec<shared::models::backend::Backend>>>();

        debug!("sending backend list request");
        self.sender
            .send(Event::ListBackends(tx))
            .await
            .map_err(|e| {
                error!("failed to send list backends event: {}", e);
                Status::internal("Internal server error")
            })?;

        debug!("waiting for the response from the proxy");
        let backends = rx
            .await
            .map_err(|e| {
                error!("failed to receive list backends response: {}", e);
                Status::internal("Internal server error")
            })?
            .map_err(|e| {
                error!("failed to list backends: {}", e);
                Status::internal("Internal server error")
            })?;

        trace!("creating mpsc channel to stream backends");
        let (tx, rx) = mpsc::channel::<Result<Backend, Status>>(4);

        tokio::spawn(async move {
            debug!("streaming backends");
            for backend in backends {
                tx.send(Ok(Backend {
                    hostname: backend.hostname,
                    redirect_ip: backend.redirect_ip,
                    redirect_port: backend.redirect_port as u32,
                }))
                .await
                .map_err(|e| {
                    error!("failed to stream backend: {}", e);
                })
                .ok();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    /// It sends a message to the proxy to create a new backend configuration
    ///
    /// Arguments:
    ///
    /// * `request`: Request<Backend>
    ///
    /// Returns:
    ///
    /// A `Result<Response<()>, Status>`
    async fn put_backend(&self, request: Request<Backend>) -> Result<Response<()>, Status> {
        trace!("received request: {:?}", request);

        trace!("creating oneshot channel to communicate with the proxy");
        let (tx, rx) = oneshot::channel::<anyhow::Result<()>>();
        let backend = request.into_inner();

        debug!("sending backend creation request: {:?}", backend);
        self.sender
            .send(Event::PutBackend(
                shared::models::backend::Backend {
                    hostname: backend.hostname,
                    redirect_ip: backend.redirect_ip,
                    redirect_port: backend.redirect_port as u16,
                },
                tx,
            ))
            .await
            .map_err(|e| {
                error!("failed to send put backend event: {}", e);
                Status::internal("Internal server error")
            })?;

        debug!("waiting for the response from the proxy");
        rx.await
            .map_err(|e| {
                error!("failed to receive put backend response: {}", e);
                Status::internal("Internal server error")
            })?
            .map_or_else(
                |e| {
                    error!("failed to put backend: {}", e);
                    Err(Status::internal("Internal server error"))
                },
                |_| Ok(Response::new(())),
            )
    }

    /// It sends a message to the proxy to delete a backend configuration
    ///
    /// Arguments:
    ///
    /// * `request`: The request object that contains the backend to be deleted.
    ///
    /// Returns:
    ///
    /// A `Result<Response<()>, Status>`
    async fn delete_backend(&self, request: Request<Backend>) -> Result<Response<()>, Status> {
        trace!("received request: {:?}", request);

        trace!("creating oneshot channel to communicate with the proxy");
        let (tx, rx) = oneshot::channel::<anyhow::Result<()>>();
        let backend = request.into_inner();

        debug!("sending backend deletion request: {:?}", backend);
        self.sender
            .send(Event::DeleteBackend(
                shared::models::backend::Backend {
                    hostname: backend.hostname,
                    redirect_ip: backend.redirect_ip,
                    redirect_port: backend.redirect_port as u16,
                },
                tx,
            ))
            .await
            .map_err(|e| {
                error!("failed to send delete backend event: {}", e);
                Status::internal("Internal server error")
            })?;

        debug!("waiting for the response from the proxy");
        rx.await
            .map_err(|e| {
                error!("failed to receive delete backend response: {}", e);
                Status::internal("Internal server error")
            })?
            .map_or_else(
                |e| {
                    error!("failed to delete backend: {}", e);
                    Err(Status::internal("Internal server error"))
                },
                |_| Ok(Response::new(())),
            )
    }
}
