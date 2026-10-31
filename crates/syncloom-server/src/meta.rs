//! Meta transport handlers inherit the application boundary and fixed error envelope.
use crate::{failure::Failure, server::AppState};
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use std::sync::Arc;
pub(crate) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/meta/sign-in", post(begin).get(wait))
        .route("/v1/meta/connection", get(connection).delete(sign_out))
        .route("/v1/pieces/{piece_id}/assets/slides/{index}", get(slide))
}
async fn begin(State(state): State<Arc<AppState>>) -> Result<Response, Failure> {
    Ok((
        StatusCode::ACCEPTED,
        axum::Json(state.studio.meta()?.sign_in_begin()?),
    )
        .into_response())
}
async fn wait(State(state): State<Arc<AppState>>) -> Result<Response, Failure> {
    Ok(axum::Json(state.studio.meta()?.sign_in_wait().await?).into_response())
}
async fn connection(State(state): State<Arc<AppState>>) -> Result<Response, Failure> {
    Ok(axum::Json(state.studio.meta()?.connection()?).into_response())
}
async fn sign_out(State(state): State<Arc<AppState>>) -> Result<Response, Failure> {
    Ok(axum::Json(state.studio.meta()?.sign_out()?).into_response())
}
async fn slide(
    State(state): State<Arc<AppState>>,
    path: Result<Path<(String, usize)>, axum::extract::rejection::PathRejection>,
) -> Result<Response, Failure> {
    let Path((id, index)) = path.map_err(|_| syncloom_core::StudioError::AssetsRequired)?;
    crate::server::asset_response(&state, &id, syncloom_core::AssetKind::Slide(index))
}
