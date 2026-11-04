//! Thin JSON adapters for the core capture owner.
use crate::{
    failure::Failure,
    server::{AppState, Strict},
};
use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use syncloom_core::{CaptureConfirmRequest, ThoughtResponse};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Text {
    text: String,
}

pub(crate) async fn suggest(Strict(body): Strict<Text>) -> Result<Json<Value>, Failure> {
    Ok(Json(
        json!({"suggestions":syncloom_core::suggest(&body.text)?}),
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Selection {
    pub text: String,
    pub channels: Vec<String>,
    #[serde(default)]
    pub source_revision: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Url {
    pub url: String,
}

pub(crate) async fn fetch(
    State(state): State<Arc<AppState>>,
    Strict(body): Strict<Url>,
) -> Result<Json<syncloom_core::capture::UrlCapture>, Failure> {
    Ok(Json(state.studio.fetch_source(&body.url).await?))
}

pub(crate) async fn preview(
    State(state): State<Arc<AppState>>,
    Strict(body): Strict<Selection>,
) -> Result<Json<syncloom_core::Preview>, Failure> {
    Ok(Json(state.studio.preview_capture(
        &body.text,
        &body.channels,
        body.source_revision.as_deref(),
    )?))
}
pub(crate) async fn confirm(
    State(state): State<Arc<AppState>>,
    Strict(body): Strict<CaptureConfirmRequest>,
) -> Result<(StatusCode, Json<ThoughtResponse>), Failure> {
    let result = state.studio.confirm_capture(body).await?;
    Ok((
        if result.created {
            StatusCode::CREATED
        } else {
            StatusCode::OK
        },
        Json(result.thought),
    ))
}
