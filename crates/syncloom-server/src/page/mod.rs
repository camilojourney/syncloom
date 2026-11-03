//! Script-free browser adapters. Studio owns content, revisions and dispatch policy.
mod actions;
pub(crate) mod capture;
mod consent;
mod counts;
mod editor;
mod home;
mod previews;
mod reconnect;
mod review;
mod status;

pub(crate) use actions::{PieceForm, action};
pub(crate) use consent::CostConsents;
pub(crate) use editor::editor;
pub(crate) use home::{Cursor, cursor, index, thought};
pub(crate) use reconnect::{begin as reconnect, complete as reconnect_complete};
pub(crate) use review::review;

use crate::{failure::Failure, server::AppState};
use axum::{
    Form,
    extract::{
        Path, Query, State,
        rejection::{FormRejection, PathRejection, QueryRejection},
    },
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use std::sync::Arc;
use syncloom_core::html::escape;
use syncloom_core::{
    AssetHashes, Channel, Piece, PublishRequest, RenderRequest, ReviewRequest, Status, StudioError,
    ThoughtRequest,
};

fn document(title: &str, content: &str) -> Html<String> {
    Html(format!(
        r##"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>{} | Syncloom</title><link rel="stylesheet" href="/studio.css"></head><body><a class="skip-link" href="#main">Skip to content</a><header class="topbar"><a class="brand" href="/">syncloom<span class="brand-dot">.</span></a><a class="workspace-link" href="/">Workspace</a><span class="local">Local studio</span></header><main id="main" tabindex="-1">{content}</main><footer><span>Your words. Your decision.</span><span>Saved on this laptop</span></footer></body></html>"##,
        escape(title)
    ))
}

pub(crate) fn error(failure: Failure, request_id: &str) -> Response {
    (failure.status(), document("Something needs attention", &format!(
        r#"<section class="error card"><h1>Something needs attention.</h1><p>{}</p><details class="error-details"><summary>Details</summary><p>{}</p><p class="error-code"><code>{}</code></p><p class="hint">Request <code class="hash">{}</code></p></details><a class="button" href="/">Back to studio</a><p class="hint">Before trying again, open the piece to check what was saved and whether it was sent.</p></section>"#,
        error_message(failure), escape(failure.message()), failure.code(), escape(request_id)
    ))).into_response()
}

/// Browser copy is separate from stable API error codes and diagnostic messages.
fn error_message(failure: Failure) -> &'static str {
    match failure {
        Failure::Studio(StudioError::PieceNotFound) => {
            "We couldn't find that piece. It may have been deleted."
        }
        Failure::Studio(StudioError::ThoughtTooShort) => {
            "Add a little more to your thought: at least 8 characters that aren't spaces."
        }
        Failure::Studio(StudioError::RevisionConflict | StudioError::AssetConflict) => {
            "This version no longer matches. Check saved pieces before trying again; a capture may already have created drafts."
        }
        Failure::Studio(StudioError::ApprovalRequired) => {
            "Review and approve this saved version before publishing."
        }
        Failure::Studio(StudioError::CostConfirmationRequired) => {
            "Check and accept the current X cost estimate before publishing."
        }
        Failure::Studio(StudioError::AssetsRequired | StudioError::AssetIntegrity) => {
            "The current previews aren't ready to approve. Make them again, then check every slide."
        }
        Failure::Studio(
            StudioError::ProviderError
            | StudioError::ProviderTimeout
            | StudioError::ProviderNotConfigured,
        ) => "We couldn't draft your text. Check the drafting setup and try again.",
        Failure::SessionRequired | Failure::Studio(StudioError::InvalidRequest) => {
            "We couldn't use that form. Reload the studio and try again."
        }
        Failure::NotFound => "We couldn't find that page. Return to the studio to continue.",
        _ => {
            "We couldn't finish that action. Check the saved piece before trying again; more information is in Details."
        }
    }
}

pub(crate) async fn css() -> impl IntoResponse {
    (
        [("content-type", "text/css; charset=utf-8")],
        include_str!("../studio.css"),
    )
}

fn csrf(state: &AppState) -> String {
    hidden("csrf", state.boundary.token())
}
fn hidden(name: &str, value: &str) -> String {
    format!(
        r#"<input type="hidden" name="{name}" value="{}">"#,
        escape(value)
    )
}
fn revision_fields(state: &AppState, piece: &Piece) -> String {
    format!(
        "{}{}",
        csrf(state),
        hidden("expected_revision", &piece.revision)
    )
}
