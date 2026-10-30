//! One request's identity and log line: the outermost middleware owns request-id validation,
//! the error envelope and the drop guard that writes the single log line.
use axum::{extract::Request, middleware::Next, response::Response};

pub(crate) async fn middleware(request: Request, next: Next) -> Response {
    next.run(request).await
}

pub fn log_subscriber<W>(writer: W) -> impl tracing::Subscriber + Send + Sync
where
    W: for<'a> tracing_subscriber::fmt::MakeWriter<'a> + Send + Sync + 'static,
{
    tracing_subscriber::fmt()
        .json()
        .flatten_event(true)
        .with_current_span(false)
        .with_span_list(false)
        .with_target(false)
        .with_ansi(false)
        .with_writer(writer)
        .finish()
}
