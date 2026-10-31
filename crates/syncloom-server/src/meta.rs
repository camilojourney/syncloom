//! Meta transport handlers inherit the application boundary and fixed error envelope.
use crate::{
    connection_routes::{AppRouter, register},
    failure::Failure,
    server::AppState,
};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use std::sync::Arc;
pub(crate) fn routes() -> AppRouter {
    register([
        ("/v1/meta/sign-in", post(begin).get(wait)),
        ("/v1/meta/connection", get(connection).delete(sign_out)),
        ("/v1/pieces/{piece_id}/assets/slides/{index}", get(slide)),
    ])
}
async fn begin(State(state): State<Arc<AppState>>) -> Result<Response, Failure> {
    Ok(crate::connection_routes::started(
        state.studio.meta()?.sign_in_begin()?,
    ))
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
