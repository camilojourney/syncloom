//! Protected connection handlers; route registration lives in social_routes.
use crate::{
    failure::Failure,
    server::{AppState, Strict},
};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use std::sync::Arc;
use syncloom_core::{Channel, Secret};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Password {
    identifier: String,
    app_password: Secret,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Callback {
    callback_url: Secret,
}
pub(crate) async fn bluesky_sign_in(
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
pub(crate) async fn x_begin(State(s): State<Arc<AppState>>) -> Result<Response, Failure> {
    Ok(crate::connection_routes::started(
        s.studio.social()?.x.sign_in_begin()?,
    ))
}
pub(crate) async fn x_wait(State(s): State<Arc<AppState>>) -> Result<Response, Failure> {
    Ok(axum::Json(s.studio.social()?.x.sign_in_wait().await?).into_response())
}
pub(crate) async fn threads_begin(State(s): State<Arc<AppState>>) -> Result<Response, Failure> {
    Ok(crate::connection_routes::started(
        s.studio.social()?.threads.sign_in_begin()?,
    ))
}
pub(crate) async fn threads_complete(
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
pub(crate) async fn connection(
    State(s): State<Arc<AppState>>,
    path: TargetPath,
) -> Result<Response, Failure> {
    Ok(axum::Json(s.studio.social()?.connection(target(path)?)?).into_response())
}
pub(crate) async fn sign_out(
    State(s): State<Arc<AppState>>,
    path: TargetPath,
) -> Result<Response, Failure> {
    let signed_out = s.studio.social()?.sign_out(target(path)?)?;
    Ok(axum::Json(signed_out).into_response())
}
pub(crate) async fn refresh(
    State(s): State<Arc<AppState>>,
    path: TargetPath,
) -> Result<Response, Failure> {
    let refreshed = s.studio.social()?.refresh(target(path)?).await?;
    Ok(axum::Json(refreshed).into_response())
}
