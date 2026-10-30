//! Routes and handlers. Errors are returned as `Failure` values; the request scope renders
//! them. Bodies are read strictly: a body that is not exactly the documented object is
//! `invalid_request`.
use crate::{failure::Failure, request_scope};
use axum::{
    Router,
    body::Body,
    extract::{FromRequest, Path, Request, State, rejection::PathRejection},
    http::{StatusCode, header},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use http_body_util::BodyExt;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use syncloom_core::{
    PublishRequest, ReviewRequest, Studio, StudioError, ThoughtRequest, ThoughtResponse,
};
use tokio::net::TcpListener;
use tower_http::catch_panic::CatchPanicLayer;

/// Largest request body read. A thought or an edit is far smaller.
const MAX_BODY_BYTES: usize = 1024 * 1024;

struct AppState {
    studio: Arc<Studio>,
    /// The only Host header value accepted: `127.0.0.1:<bound port>`.
    host: String,
}

pub async fn serve(listener: TcpListener, studio: Arc<Studio>) -> std::io::Result<()> {
    let state = Arc::new(AppState {
        studio,
        host: listener.local_addr()?.to_string(),
    });
    let app = Router::new()
        .route("/v1/health", get(health))
        .route("/v1/thoughts", post(create_thought))
        .route("/v1/pieces/{piece_id}", get(show_piece).patch(review_piece))
        .route("/v1/pieces/{piece_id}/publish", post(publish_piece))
        .fallback(|| async { Failure::NotFound })
        .method_not_allowed_fallback(|| async { Failure::MethodNotAllowed })
        .layer(CatchPanicLayer::custom(|_| request_scope::internal_error()))
        .layer(middleware::from_fn_with_state(state.clone(), host_check))
        .layer(middleware::from_fn(request_scope::middleware))
        .with_state(state);
    axum::serve(listener, app).await
}

/// DNS-rebinding guard: a page that resolves its own name to 127.0.0.1 keeps its own Host.
async fn host_check(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: middleware::Next,
) -> Response {
    let mut hosts = request.headers().get_all(header::HOST).iter();
    let accepted = match (hosts.next(), hosts.next()) {
        (Some(host), None) => host.as_bytes() == state.host.as_bytes(),
        _ => false,
    };
    if accepted {
        next.run(request).await
    } else {
        Failure::HostNotAllowed.into_response()
    }
}

async fn health() -> impl IntoResponse {
    axum::Json(serde_json::json!({"status": "ok", "version": env!("CARGO_PKG_VERSION")}))
}

/// A JSON body that must be exactly the documented object (unknown fields are refused by the
/// request types). Any other body is `invalid_request`.
struct Strict<T>(T);

impl<S, T> FromRequest<S> for Strict<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = Failure;

    async fn from_request(request: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let bytes = read_bounded(request.into_body()).await?;
        serde_json::from_slice(&bytes)
            .map(Strict)
            .map_err(|_| Failure::Studio(StudioError::InvalidRequest))
    }
}

async fn read_bounded(mut body: Body) -> Result<Vec<u8>, Failure> {
    let mut bytes = Vec::new();
    while let Some(frame) = body.frame().await {
        let frame = frame.map_err(|_| Failure::Studio(StudioError::InvalidRequest))?;
        if let Ok(data) = frame.into_data() {
            if bytes.len() + data.len() > MAX_BODY_BYTES {
                return Err(Failure::Studio(StudioError::InvalidRequest));
            }
            bytes.extend_from_slice(&data);
        }
    }
    Ok(bytes)
}

/// A piece id from the path; an undecodable path is "not found", never a transport error.
fn piece_id(path: Result<Path<String>, PathRejection>) -> Result<String, Failure> {
    path.map(|Path(id)| id)
        .map_err(|_| Failure::Studio(StudioError::PieceNotFound))
}

async fn create_thought(
    State(state): State<Arc<AppState>>,
    Strict(request): Strict<ThoughtRequest>,
) -> Result<(StatusCode, axum::Json<ThoughtResponse>), Failure> {
    let response = state.studio.create_thought(request).await?;
    Ok((StatusCode::CREATED, axum::Json(response)))
}

async fn show_piece(
    State(state): State<Arc<AppState>>,
    path: Result<Path<String>, PathRejection>,
) -> Result<Response, Failure> {
    let piece = state.studio.show(&piece_id(path)?)?;
    Ok(axum::Json(piece).into_response())
}

async fn review_piece(
    State(state): State<Arc<AppState>>,
    path: Result<Path<String>, PathRejection>,
    Strict(request): Strict<ReviewRequest>,
) -> Result<Response, Failure> {
    let piece = state.studio.review(&piece_id(path)?, request)?;
    Ok(axum::Json(piece).into_response())
}

async fn publish_piece(
    State(state): State<Arc<AppState>>,
    path: Result<Path<String>, PathRejection>,
    Strict(request): Strict<PublishRequest>,
) -> Result<Response, Failure> {
    let response = state.studio.publish(&piece_id(path)?, request)?;
    Ok(axum::Json(response).into_response())
}
