//! One registration owner for connection route tables. The application applies all guards.
use crate::server::AppState;
use axum::{
    Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::MethodRouter,
};
use std::sync::Arc;

pub(crate) type AppRouter = Router<Arc<AppState>>;
type Entry = (&'static str, MethodRouter<Arc<AppState>>);

pub(crate) fn started(start: syncloom_core::SignInStart) -> Response {
    (StatusCode::ACCEPTED, axum::Json(start)).into_response()
}

pub(crate) fn register<const N: usize>(entries: [Entry; N]) -> AppRouter {
    entries
        .into_iter()
        .fold(Router::new(), |router, (path, handler)| {
            router.route(path, handler)
        })
}
