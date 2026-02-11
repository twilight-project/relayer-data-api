use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use hyper::{Body, Request, Response};
use tower::{Layer, Service};

tokio::task_local! {
    pub static REQUEST_HEADERS: HashMap<String, Option<String>>;
}

/// Build a Meta from the current request's task-local headers.
/// Falls back to Meta::default() if no task-local is set (e.g. WebSocket).
pub fn meta_from_headers() -> relayer_core::relayer::Meta {
    REQUEST_HEADERS
        .try_with(|headers| relayer_core::relayer::Meta {
            metadata: headers.clone(),
        })
        .unwrap_or_default()
}

#[derive(Clone)]
pub struct HeaderExtractLayer;

impl<S> Layer<S> for HeaderExtractLayer {
    type Service = HeaderExtractService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        HeaderExtractService { inner }
    }
}

#[derive(Clone)]
pub struct HeaderExtractService<S> {
    inner: S,
}

impl<S, ResBody> Service<Request<Body>> for HeaderExtractService<S>
where
    S: Service<Request<Body>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Error: Send,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let mut headers_map = HashMap::new();
        if let Some(val) = req.headers().get("Twilight-Address") {
            headers_map.insert(
                "Twilight-Address".to_string(),
                val.to_str().ok().map(|s| s.to_string()),
            );
        }
        if let Some(val) = req.headers().get("Relayer") {
            headers_map.insert(
                "Relayer".to_string(),
                val.to_str().ok().map(|s| s.to_string()),
            );
        }
        let mut inner = self.inner.clone();
        Box::pin(REQUEST_HEADERS.scope(headers_map, async move {
            inner.call(req).await
        }))
    }
}
