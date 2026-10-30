//! HTTP failures: the transport's own codes plus the core's error table, each with one status.
//! Messages come from the core table or are fixed here; none carries request content.
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use syncloom_core::StudioError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Failure {
    HostNotAllowed,
    NotFound,
    MethodNotAllowed,
    Studio(StudioError),
}

impl From<StudioError> for Failure {
    fn from(error: StudioError) -> Self {
        Failure::Studio(error)
    }
}

impl Failure {
    pub(crate) fn status(self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
    pub(crate) fn code(self) -> &'static str {
        ""
    }
    pub(crate) fn message(self) -> &'static str {
        ""
    }
    /// The standard envelope, carrying the final request id.
    pub(crate) fn envelope(self, request_id: &str) -> Response {
        let body = serde_json::json!({
            "error": {"code": self.code(), "message": self.message(), "request_id": request_id}
        });
        (self.status(), axum::Json(body)).into_response()
    }
}

/// Handlers return this; the request scope renders the envelope because only it knows the id.
impl IntoResponse for Failure {
    fn into_response(self) -> Response {
        let mut response = self.status().into_response();
        response.extensions_mut().insert(self);
        response
    }
}
