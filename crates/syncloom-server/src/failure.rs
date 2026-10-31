//! HTTP failures: the transport's own codes plus the core's error table, each with one status.
//! Codes and messages for studio errors come from the core table (one owner); the three
//! transport codes and every status live here. `contracts/studio-api/errors/` holds the same
//! values and a test proves they agree.
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use syncloom_core::StudioError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Failure {
    HostNotAllowed,
    OriginNotAllowed,
    SessionRequired,
    NotFound,
    MethodNotAllowed,
    Studio(StudioError),
}

impl From<StudioError> for Failure {
    fn from(error: StudioError) -> Self {
        Failure::Studio(error)
    }
}

/// One status per core error, in the order of the `StudioError` variants (`StudioError::ALL`).
const STUDIO_STATUSES: [StatusCode; 29] = [
    StatusCode::BAD_REQUEST,           // invalid_request
    StatusCode::UNPROCESSABLE_ENTITY,  // thought_too_short
    StatusCode::UNPROCESSABLE_ENTITY,  // unsupported_channel
    StatusCode::NOT_FOUND,             // piece_not_found
    StatusCode::CONFLICT,              // invalid_status_transition
    StatusCode::CONFLICT,              // revision_conflict
    StatusCode::CONFLICT,              // approval_required
    StatusCode::SERVICE_UNAVAILABLE,   // provider_not_configured
    StatusCode::BAD_GATEWAY,           // provider_error
    StatusCode::GATEWAY_TIMEOUT,       // provider_timeout
    StatusCode::SERVICE_UNAVAILABLE,   // linkedin_not_configured
    StatusCode::CONFLICT,              // sign_in_in_progress
    StatusCode::CONFLICT,              // sign_in_not_started
    StatusCode::FORBIDDEN,             // sign_in_denied
    StatusCode::FORBIDDEN,             // sign_in_state_mismatch
    StatusCode::GATEWAY_TIMEOUT,       // sign_in_timed_out
    StatusCode::BAD_GATEWAY,           // sign_in_rejected
    StatusCode::SERVICE_UNAVAILABLE,   // renderer_missing
    StatusCode::SERVICE_UNAVAILABLE,   // renderer_version_mismatch
    StatusCode::SERVICE_UNAVAILABLE,   // renderer_font_mismatch
    StatusCode::GATEWAY_TIMEOUT,       // renderer_timeout
    StatusCode::BAD_GATEWAY,           // renderer_failed
    StatusCode::CONFLICT,              // assets_required
    StatusCode::CONFLICT,              // asset_integrity
    StatusCode::CONFLICT,              // asset_conflict
    StatusCode::SERVICE_UNAVAILABLE,   // mastodon_not_configured
    StatusCode::SERVICE_UNAVAILABLE,   // platform_not_configured
    StatusCode::CONFLICT,              // cost_confirmation_required
    StatusCode::INTERNAL_SERVER_ERROR, // internal_error
];

impl Failure {
    pub(crate) fn status(self) -> StatusCode {
        match self {
            Failure::HostNotAllowed | Failure::OriginNotAllowed | Failure::SessionRequired => {
                StatusCode::FORBIDDEN
            }
            Failure::NotFound => StatusCode::NOT_FOUND,
            Failure::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            Failure::Studio(error) => STUDIO_STATUSES[error as usize],
        }
    }
    pub(crate) fn code(self) -> &'static str {
        match self {
            Failure::HostNotAllowed => "host_not_allowed",
            Failure::OriginNotAllowed => "origin_not_allowed",
            Failure::SessionRequired => "session_required",
            Failure::NotFound => "not_found",
            Failure::MethodNotAllowed => "method_not_allowed",
            Failure::Studio(error) => error.code(),
        }
    }
    pub(crate) fn message(self) -> &'static str {
        match self {
            Failure::HostNotAllowed => "Requests must be sent to 127.0.0.1 on the server port.",
            Failure::OriginNotAllowed => "Requests must come from the local studio origin.",
            Failure::SessionRequired => {
                "Reload the local studio to obtain a current session token."
            }
            Failure::NotFound => "The requested endpoint does not exist.",
            Failure::MethodNotAllowed => "This HTTP method is not supported for this endpoint.",
            Failure::Studio(error) => error.message(),
        }
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
