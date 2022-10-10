use std::io::Error;

use async_trait::async_trait;
use log::{debug, error};
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
        debug!("received request: {:?}", request);

        let (tx, rx) = oneshot::channel::<Result<Vec<Backend>, Error>>();

        self.sender.send(Event::ListBackends(tx)).await.unwrap();

        let backends = rx.await.unwrap().map_err(|e| {
            error!("failed to list backends: {}", e);
            Status::internal("Internal server error")
        })?;
        let (tx, rx) = mpsc::channel::<Result<Backend, Status>>(4);

        tokio::spawn(async move {
            for backend in backends {
                tx.send(Ok(backend)).await.unwrap();
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
        debug!("received request: {:?}", request);

        let (tx, rx) = oneshot::channel::<Result<(), Error>>();

        self.sender
            .send(Event::PutBackend(request.into_inner(), tx))
            .await
            .unwrap();

        rx.await.unwrap().map_or_else(
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
        debug!("received request: {:?}", request);

        let (tx, rx) = oneshot::channel::<Result<(), Error>>();

        self.sender
            .send(Event::PutBackend(request.into_inner(), tx))
            .await
            .unwrap();

        rx.await.unwrap().map_or_else(
            |e| {
                error!("failed to delete backend: {}", e);
                Err(Status::internal("Internal server error"))
            },
            |_| Ok(Response::new(())),
        )
    }
}
