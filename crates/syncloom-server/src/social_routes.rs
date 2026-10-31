//! SocialPlatforms connection operations behind the server's single boundary and fixed error envelope.
use crate::{
    failure::Failure,
    server::{AppState, Strict},
};
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use std::sync::Arc;
use syncloom_core::{Channel, Secret};

pub(crate) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/bluesky/sign-in", post(bluesky_sign_in))
        .route("/v1/x/sign-in", post(x_begin).get(x_wait))
        .route("/v1/threads/sign-in", post(threads_begin))
        .route("/v1/threads/complete", post(threads_complete))
        .route(
            "/v1/{platform}/connection",
            get(connection).delete(sign_out),
        )
        .route("/v1/{platform}/refresh", post(refresh))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Password {
    identifier: String,
    app_password: Secret,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Callback {
    callback_url: Secret,
}
async fn bluesky_sign_in(
    State(s): State<Arc<AppState>>,
    Strict(p): Strict<Password>,
) -> Result<Response, Failure> {
    Ok(axum::Json(
        s.studio
            .social()?
            .bluesky
            .sign_in(&p.identifier, p.app_password)
            .await?,
    )
    .into_response())
}
async fn x_begin(State(s): State<Arc<AppState>>) -> Result<Response, Failure> {
    Ok((
        StatusCode::ACCEPTED,
        axum::Json(s.studio.social()?.x.sign_in_begin()?),
    )
        .into_response())
}
async fn x_wait(State(s): State<Arc<AppState>>) -> Result<Response, Failure> {
    Ok(axum::Json(s.studio.social()?.x.sign_in_wait().await?).into_response())
}
async fn threads_begin(State(s): State<Arc<AppState>>) -> Result<Response, Failure> {
    Ok((
        StatusCode::ACCEPTED,
        axum::Json(s.studio.social()?.threads.sign_in_begin()?),
    )
        .into_response())
}
async fn threads_complete(
    State(s): State<Arc<AppState>>,
    Strict(c): Strict<Callback>,
) -> Result<Response, Failure> {
    Ok(axum::Json(
        s.studio
            .social()?
            .threads
            .complete(c.callback_url.expose())
            .await?,
    )
    .into_response())
}
type TargetPath = Result<Path<String>, axum::extract::rejection::PathRejection>;
fn target(path: TargetPath) -> Result<Channel, Failure> {
    let Path(platform) = path.map_err(|_| Failure::NotFound)?;
    match Channel::parse(&platform) {
        Some(c @ (Channel::Bluesky | Channel::X | Channel::Threads)) => Ok(c),
        _ => Err(Failure::NotFound),
    }
}
async fn connection(State(s): State<Arc<AppState>>, path: TargetPath) -> Result<Response, Failure> {
    Ok(axum::Json(s.studio.social()?.connection(target(path)?)?).into_response())
}
async fn sign_out(State(s): State<Arc<AppState>>, path: TargetPath) -> Result<Response, Failure> {
    let signed_out = s.studio.social()?.sign_out(target(path)?)?;
    Ok(axum::Json(signed_out).into_response())
}
async fn refresh(State(s): State<Arc<AppState>>, path: TargetPath) -> Result<Response, Failure> {
    let refreshed = s.studio.social()?.refresh(target(path)?).await?;
    Ok(axum::Json(refreshed).into_response())
}
