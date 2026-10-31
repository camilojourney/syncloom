//! One request's identity and log line.
//!
//! The middleware here is the outermost layer. It owns request-id validation, rendering the
//! error envelope (only it knows the final id) and the drop guard that writes the single log
//! line. A request whose future is dropped before a response exists, because the client left,
//! never calls `finish`, so its line says `cancelled`. Route labels are fixed strings, so a
//! path carrying an id never reaches the log.
use crate::failure::Failure;
use axum::{
    extract::Request,
    http::{HeaderMap, HeaderValue},
    middleware::Next,
    response::Response,
};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

#[derive(Clone)]
struct Scope(Arc<Inner>);

struct Inner {
    id: String,
    route: &'static str,
    started: Instant,
    /// Status and error code of the response; `None` until a response exists.
    finished: Mutex<Option<(u16, Option<&'static str>)>>,
}

impl Scope {
    fn finish(&self, status: u16, code: Option<&'static str>) {
        *self
            .0
            .finished
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some((status, code));
    }
}

/// Writes the request's log line when dropped. Fields that do not apply are `None`, which
/// `tracing` omits, so they are absent from the line and never written as 0.
struct LogGuard(Scope);
impl Drop for LogGuard {
    fn drop(&mut self) {
        let inner = &self.0.0;
        let finished = *inner
            .finished
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let (status, error_code, outcome) = match finished {
            Some((status, code)) => (Some(status), code, "completed"),
            None => (None, None, "cancelled"),
        };
        let total_ms: u64 = inner
            .started
            .elapsed()
            .as_millis()
            .try_into()
            .unwrap_or(u64::MAX);
        tracing::info!(
            request_id = inner.id.as_str(),
            route = inner.route,
            status,
            error_code,
            total_ms,
            outcome,
            "request_finished"
        );
    }
}

/// Keep a valid `X-Request-Id`, otherwise generate a UUID v4.
fn request_id(headers: &HeaderMap) -> String {
    let valid = |id: &str| {
        (1..=128).contains(&id.len())
            && id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b':' | b'-'))
    };
    headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .filter(|id| valid(id))
        .map_or_else(|| uuid::Uuid::new_v4().to_string(), str::to_owned)
}

/// A fixed label per documented route, never the raw path.
fn route_label(path: &str) -> &'static str {
    let segments: Vec<&str> = path.trim_matches('/').split('/').collect();
    match segments.as_slice() {
        ["v1", "health"] => "/v1/health",
        ["v1", "thoughts"] => "/v1/thoughts",
        ["v1", "pieces", _] => "/v1/pieces/{piece_id}",
        ["v1", "pieces", _, "publish"] => "/v1/pieces/{piece_id}/publish",
        ["v1", "pieces", _, "export"] => "/v1/pieces/{piece_id}/export",
        ["v1", "mastodon", "sign-in"] => "/v1/mastodon/sign-in",
        ["v1", "mastodon", "connection"] => "/v1/mastodon/connection",
        ["v1", "pieces", _, "render"] => "/v1/pieces/{piece_id}/render",
        ["v1", "pieces", _, "assets", "image"] => "/v1/pieces/{piece_id}/assets/image",
        ["v1", "pieces", _, "assets", "carousel"] => "/v1/pieces/{piece_id}/assets/carousel",
        ["v1", "linkedin", "sign-in"] => "/v1/linkedin/sign-in",
        ["v1", "linkedin", "connection"] => "/v1/linkedin/connection",
        _ => "unmatched",
    }
}

pub(crate) async fn middleware(request: Request, next: Next) -> Response {
    let scope = Scope(Arc::new(Inner {
        id: request_id(request.headers()),
        route: route_label(request.uri().path()),
        started: Instant::now(),
        finished: Mutex::new(None),
    }));
    let _log = LogGuard(scope.clone());
    let mut response = next.run(request).await;
    match response.extensions_mut().remove::<Failure>() {
        Some(failure) => {
            scope.finish(failure.status().as_u16(), Some(failure.code()));
            response = failure.envelope(&scope.0.id);
        }
        None => scope.finish(response.status().as_u16(), None),
    }
    response.headers_mut().insert(
        "x-request-id",
        HeaderValue::from_str(&scope.0.id).expect("request ids are visible ASCII"),
    );
    response
}

/// Used by the panic layer: a response the middleware renders as `internal_error`.
pub(crate) fn internal_error() -> Response {
    axum::response::IntoResponse::into_response(Failure::Studio(
        syncloom_core::StudioError::Internal,
    ))
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
