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

fn source_provenance(source: &syncloom_core::Source) -> String {
    let Some(retrieval) = &source.retrieval else {
        return String::new();
    };
    format!(
        r#"<details open><summary>Source provenance</summary><dl><dt>Requested URL</dt><dd>{}</dd><dt>Final URL</dt><dd>{}</dd><dt>Retrieved at (UTC)</dt><dd>{}</dd><dt>Response SHA-256</dt><dd><code class="hash">{}</code></dd></dl></details>"#,
        escape(&source.raw_input),
        escape(&retrieval.final_url),
        escape(&retrieval.fetched_at),
        escape(&retrieval.content_sha256)
    )
}

fn document(title: &str, content: &str) -> Html<String> {
    Html(format!(
        r##"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>{} | Syncloom</title><link rel="stylesheet" href="/studio.css"></head><body><a class="skip-link" href="#main">Skip to content</a><header class="topbar"><a class="brand" href="/">syncloom<span class="brand-dot">.</span></a><a class="workspace-link" href="/">Workspace</a><span class="local">Local studio</span></header><main id="main" tabindex="-1">{content}</main><footer><span>Your words. Your decision.</span><span>Saved on this laptop</span></footer></body></html>"##,
        escape(title)
    ))
}

pub(crate) fn error(failure: Failure, request_id: &str) -> Response {
    let (message, recovery) = error_copy(failure);
    (failure.status(), document("Something needs attention", &format!(
        r#"<section class="error card"><h1>Something needs attention.</h1><p>{}</p><details class="error-details"><summary>Details</summary><p>{}</p><p class="error-code"><code>{}</code></p><p class="hint">Request <code class="hash">{}</code></p></details><a class="button" href="/">Back to studio</a><p class="hint">{recovery}</p></section>"#,
        message, escape(failure.message()), failure.code(), escape(request_id)
    ))).into_response()
}

/// Browser copy is separate from stable API error codes and diagnostic messages.
fn error_copy(failure: Failure) -> (&'static str, &'static str) {
    use Failure::Studio as Domain;
    use StudioError::*;
    if [
        Domain(SourceNotConfigured),
        Domain(SourceNotAllowed),
        Domain(SourceFetchFailed),
        Domain(SourceTimeout),
    ]
    .contains(&failure)
    {
        return (
            failure.message(),
            "No draft was created by this fetch. Return to the workspace to try an allowed URL or paste your own text.",
        );
    }
    // Browser translations are data; core owns the stable API messages.
    const MESSAGES: &[(&[Failure], &str)] = &[
        (
            &[Domain(PieceNotFound)],
            "We couldn't find that piece. It may have been deleted.",
        ),
        (
            &[Domain(ThoughtTooShort)],
            "Add a little more to your thought: at least 8 characters that aren't spaces.",
        ),
        (
            &[Domain(RevisionConflict), Domain(AssetConflict)],
            "This version no longer matches. Check saved pieces before trying again; a capture may already have created drafts.",
        ),
        (
            &[Domain(ApprovalRequired)],
            "Review and approve this saved version before publishing.",
        ),
        (
            &[Domain(CostConfirmationRequired)],
            "Check and accept the current X cost estimate before publishing.",
        ),
        (
            &[Domain(AssetsRequired), Domain(AssetIntegrity)],
            "The current previews aren't ready to approve. Make them again, then check every slide.",
        ),
        (
            &[
                Domain(ProviderError),
                Domain(ProviderTimeout),
                Domain(ProviderNotConfigured),
            ],
            "We couldn't draft your text. Check the drafting setup and try again.",
        ),
        (
            &[Failure::SessionRequired, Domain(InvalidRequest)],
            "We couldn't use that form. Reload the studio and try again.",
        ),
        (
            &[Failure::NotFound],
            "We couldn't find that page. Return to the studio to continue.",
        ),
    ];
    let message = MESSAGES.iter().find(|(kinds,_)| kinds.contains(&failure)).map_or(
        "We couldn't finish that action. Check the saved piece before trying again; more information is in Details.",
        |(_, message)| *message,
    );
    (
        message,
        "Before trying again, open the piece to check what was saved and whether it was sent.",
    )
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
