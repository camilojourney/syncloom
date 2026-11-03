//! Thin JSON adapters for the core capture owner.
use crate::{
    failure::Failure,
    server::{AppState, Strict},
};
use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use syncloom_core::{CaptureConfirmRequest, ThoughtRequest, ThoughtResponse};

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
pub(crate) async fn preview(
    Strict(body): Strict<ThoughtRequest>,
) -> Result<Json<syncloom_core::Preview>, Failure> {
    Ok(Json(syncloom_core::preview(&body.text, &body.channels)?))
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
