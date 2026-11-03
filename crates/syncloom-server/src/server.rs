//! Routes and handlers. Errors are returned as `Failure` values; the request scope renders
//! them. Bodies are read strictly: a body that is not exactly the documented object is
//! `invalid_request`.
use crate::{
    boundary::{self, Boundary},
    failure::Failure,
    page, request_scope,
};
use axum::{
    Router,
    body::Body,
    extract::{
        FromRequest, Path, Query, Request, State,
        rejection::{PathRejection, QueryRejection},
    },
    http::{StatusCode, header},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use http_body_util::BodyExt;
use serde::de::DeserializeOwned;
use serde_json::json;
use std::sync::Arc;
use syncloom_core::{
    PublishRequest, ReviewRequest, Studio, StudioError, ThoughtRequest, ThoughtResponse,
};
use tokio::net::TcpListener;
use tower_http::catch_panic::CatchPanicLayer;

/// Largest request body read. A thought or an edit is far smaller.
const MAX_BODY_BYTES: usize = 1024 * 1024;

pub(crate) struct AppState {
    pub(crate) studio: Arc<Studio>,
    pub(crate) boundary: Boundary,
    pub(crate) cost_consents: page::CostConsents,
}

pub async fn serve(listener: TcpListener, studio: Arc<Studio>) -> std::io::Result<()> {
    let address = listener.local_addr()?;
    if address.ip() != std::net::Ipv4Addr::LOCALHOST {
        return Err(std::io::Error::other("server listener must bind 127.0.0.1"));
    }
    let state = Arc::new(AppState {
        studio,
        boundary: Boundary::new(address.to_string()),
        cost_consents: page::CostConsents::default(),
    });
    let app = Router::new()
        .merge(crate::meta::routes())
        .merge(crate::social_routes::routes())
        .route("/", get(page::index))
        .route("/studio.css", get(page::css))
        .route("/capture", get(page::capture::entry))
        .route("/capture/suggest", post(page::capture::suggest))
        .route("/capture/preview", post(page::capture::preview))
        .route("/capture/confirm", post(page::capture::confirm))
        .route("/capture/result/{revision}", get(page::capture::result))
        .route("/v1/capture/suggest", post(crate::capture::suggest))
        .route("/v1/capture/preview", post(crate::capture::preview))
        .route("/v1/capture/confirm", post(crate::capture::confirm))
        .route("/review/{piece_id}", get(page::review))
        .route(
            "/review/{piece_id}/{action}",
            get(page::editor).post(page::action),
        )
        .route("/actions/thought", post(page::thought))
        .route("/v1/session", get(session))
        .route("/v1/pieces", get(list_pieces))
        .route("/v1/health", get(health))
        .route("/v1/thoughts", post(create_thought))
        .route("/v1/pieces/{piece_id}", get(show_piece).patch(review_piece))
        .route("/v1/pieces/{piece_id}/publish", post(publish_piece))
        .route("/v1/pieces/{piece_id}/export", post(export_piece))
        .route(
            "/v1/mastodon/sign-in",
            post(mastodon_sign_in_begin).get(mastodon_sign_in_wait),
        )
        .route(
            "/v1/mastodon/connection",
            get(mastodon_connection).delete(mastodon_sign_out),
        )
        .route("/v1/pieces/{piece_id}/render", post(render_piece))
        .route(
            "/v1/pieces/{piece_id}/assets/image",
            get(asset_handler(syncloom_core::AssetKind::Image)),
        )
        .route(
            "/v1/pieces/{piece_id}/assets/carousel",
            get(asset_handler(syncloom_core::AssetKind::Carousel)),
        )
        .route(
            "/v1/linkedin/sign-in",
            post(linkedin_sign_in_begin).get(linkedin_sign_in_wait),
        )
        .route(
            "/v1/linkedin/connection",
            get(linkedin_connection).delete(linkedin_sign_out),
        )
        .fallback(|| async { Failure::NotFound })
        .method_not_allowed_fallback(|| async { Failure::MethodNotAllowed })
        .layer(CatchPanicLayer::custom(|_| request_scope::internal_error()))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            boundary::middleware,
        ))
        .layer(middleware::from_fn(request_scope::middleware))
        .with_state(state);
    axum::serve(listener, app).await
}

async fn session(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    axum::Json(json!({"token": state.boundary.token()}))
}

async fn list_pieces(
    State(state): State<Arc<AppState>>,
    query: Result<Query<page::Cursor>, QueryRejection>,
) -> Result<Response, Failure> {
    let page = state.studio.list(page::cursor(query)?.after.as_deref())?;
    Ok(axum::Json(page).into_response())
}

async fn health() -> impl IntoResponse {
    axum::Json(json!({"status": "ok", "version": env!("CARGO_PKG_VERSION")}))
}

/// A JSON body that must be exactly the documented object (unknown fields are refused by the
/// request types). Any other body is `invalid_request`.
pub(crate) struct Strict<T>(pub(crate) T);

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

pub(crate) async fn read_bounded(mut body: Body) -> Result<Vec<u8>, Failure> {
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
    let response = state.studio.publish(&piece_id(path)?, request).await?;
    Ok(axum::Json(response).into_response())
}

async fn export_piece(
    State(state): State<Arc<AppState>>,
    path: PiecePath,
    Strict(request): Strict<syncloom_core::manual_export::ExportRequest>,
) -> Result<Response, Failure> {
    Ok(axum::Json(state.studio.export(&piece_id(path)?, request)?).into_response())
}
async fn mastodon_sign_in_begin(State(state): State<Arc<AppState>>) -> Result<Response, Failure> {
    Ok((
        StatusCode::ACCEPTED,
        axum::Json(state.studio.mastodon_sign_in_begin().await?),
    )
        .into_response())
}
async fn mastodon_sign_in_wait(State(state): State<Arc<AppState>>) -> Result<Response, Failure> {
    Ok(axum::Json(state.studio.mastodon_sign_in_wait().await?).into_response())
}
async fn mastodon_connection(State(state): State<Arc<AppState>>) -> Result<Response, Failure> {
    Ok(axum::Json(state.studio.mastodon_connection()?).into_response())
}
async fn mastodon_sign_out(State(state): State<Arc<AppState>>) -> Result<Response, Failure> {
    Ok(axum::Json(state.studio.mastodon_sign_out()?).into_response())
}

async fn render_piece(
    State(state): State<Arc<AppState>>,
    path: Result<Path<String>, PathRejection>,
    Strict(request): Strict<syncloom_core::RenderRequest>,
) -> Result<Response, Failure> {
    let piece = state.studio.render(&piece_id(path)?, request).await?;
    Ok(axum::Json(piece).into_response())
}

pub(crate) fn asset_response(
    state: &AppState,
    id: &str,
    kind: syncloom_core::AssetKind,
) -> Result<Response, Failure> {
    let (bytes, hash) = state.studio.view_asset(id, kind)?;
    Ok((
        [
            (header::CONTENT_TYPE, kind.mime().to_owned()),
            (header::CACHE_CONTROL, "no-store".to_owned()),
            (header::ETAG, format!("\"{hash}\"")),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_owned()),
        ],
        bytes,
    )
        .into_response())
}

/// Both fixed routes share the byte-response policy; only the asset kind differs.
type PiecePath = Result<Path<String>, PathRejection>;
type AssetResponse = std::future::Ready<Result<Response, Failure>>;

fn asset_handler(
    kind: syncloom_core::AssetKind,
) -> impl Fn(State<Arc<AppState>>, PiecePath) -> AssetResponse + Clone {
    move |State(state), path| {
        std::future::ready(piece_id(path).and_then(|id| asset_response(&state, &id, kind)))
    }
}

async fn linkedin_sign_in_begin(State(state): State<Arc<AppState>>) -> Result<Response, Failure> {
    let start = state.studio.linkedin_sign_in_begin()?;
    Ok((StatusCode::ACCEPTED, axum::Json(start)).into_response())
}

async fn linkedin_sign_in_wait(State(state): State<Arc<AppState>>) -> Result<Response, Failure> {
    let connection = state.studio.linkedin_sign_in_wait().await?;
    Ok(axum::Json(connection).into_response())
}

async fn linkedin_connection(State(state): State<Arc<AppState>>) -> Result<Response, Failure> {
    let connection = state.studio.linkedin_connection()?;
    Ok(axum::Json(connection).into_response())
}

async fn linkedin_sign_out(State(state): State<Arc<AppState>>) -> Result<Response, Failure> {
    let connection = state.studio.linkedin_sign_out()?;
    Ok(axum::Json(connection).into_response())
}
