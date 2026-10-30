//! Routes and handlers. Errors are returned as `Failure` values; the request scope renders them.
use crate::{failure::Failure, request_scope};
use axum::{Router, middleware};
use std::sync::Arc;
use syncloom_core::Studio;
use tokio::net::TcpListener;

pub async fn serve(listener: TcpListener, studio: Arc<Studio>) -> std::io::Result<()> {
    let _ = studio;
    let app = Router::new()
        .fallback(|| async { Failure::NotFound })
        .layer(middleware::from_fn(request_scope::middleware));
    axum::serve(listener, app).await
}
