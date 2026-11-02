//! Rust-rendered browser adapters. Studio owns every content, revision and dispatch rule.
use crate::{failure::Failure, server::AppState};
use axum::{
    Form,
    extract::{
        Path, Query, State,
        rejection::{FormRejection, PathRejection, QueryRejection},
    },
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use syncloom_core::html::escape;
use syncloom_core::{
    AssetHashes, Piece, PublishRequest, PublishResponse, RenderRequest, ReviewRequest, Status,
    StudioError, ThoughtRequest,
};

/// Browser-only form state. Restart requires fresh acceptance; core dispatch still checks cost.
#[derive(Default)]
pub(crate) struct CostConsents(Mutex<HashMap<String, CostConsent>>);

#[derive(PartialEq, Eq)]
struct CostConsent {
    revision: String,
    estimate: syncloom_core::social::CostEstimate,
}

impl CostConsent {
    fn current(piece: &Piece) -> Self {
        Self {
            revision: piece.revision.clone(),
            estimate: syncloom_core::social::estimate(&piece.text, piece.visual.is_some()),
        }
    }
}

impl CostConsents {
    #[expect(
        clippy::expect_used,
        reason = "Poisoned cost-consent state cannot safely authorize spend; preserve fail-closed behavior."
    )]
    fn accepted(&self, piece: &Piece) -> Option<u64> {
        let current = CostConsent::current(piece);
        let mut records = self.0.lock().expect("cost consent lock");
        if piece.channel == syncloom_core::Channel::X
            && records.get(&piece.piece_id) == Some(&current)
        {
            Some(current.estimate.microusd)
        } else {
            // Never reuse a stale acceptance after content or pricing changes.
            records.remove(&piece.piece_id);
            None
        }
    }

    #[expect(
        clippy::expect_used,
        reason = "Poisoned cost-consent state cannot safely authorize spend; preserve fail-closed behavior."
    )]
    fn accept(&self, piece: &Piece, form: &PieceForm) -> Result<(), StudioError> {
        if piece.channel != syncloom_core::Channel::X {
            return Err(StudioError::InvalidRequest);
        }
        if piece.revision != form.expected_revision {
            return Err(StudioError::RevisionConflict);
        }
        let current = CostConsent::current(piece);
        if form.accepted_cost_microusd != Some(current.estimate.microusd)
            || form.pricing_as_of.as_deref() != Some(current.estimate.pricing_as_of.as_str())
        {
            return Err(StudioError::CostConfirmationRequired);
        }
        self.0
            .lock()
            .expect("cost consent lock")
            .insert(piece.piece_id.clone(), current);
        Ok(())
    }
}

fn document(title: &str, content: &str) -> Html<String> {
    Html(format!(
        r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>{} | Syncloom</title><link rel="stylesheet" href="/studio.css"></head><body><header class="topbar"><a class="brand" href="/">syncloom<span class="brand-dot">.</span></a><span class="local">Laptop only <span aria-hidden="true">·</span> 127.0.0.1</span></header><main>{content}</main><footer>One thought. Your voice. Your decision.<span>Local studio</span></footer></body></html>"#,
        escape(title)
    ))
}

pub(crate) fn error(failure: Failure, request_id: &str) -> Response {
    (failure.status(), document("Something needs attention", &format!(r#"<section class="error card"><p class="eyebrow">Nothing was assumed</p><h1>Something needs attention.</h1><p>{}</p><p><code>{}</code></p><p class="muted">Request <code>{}</code></p><a class="button" href="/">Back to studio</a><p class="muted">Reopen the piece to see its current saved revision before trying again.</p></section>"#, failure.message(), failure.code(), escape(request_id)))).into_response()
}

pub(crate) async fn css() -> impl IntoResponse {
    (
        [("content-type", "text/css; charset=utf-8")],
        include_str!("studio.css"),
    )
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Cursor {
    pub(crate) after: Option<String>,
}

pub(crate) fn cursor(query: Result<Query<Cursor>, QueryRejection>) -> Result<Cursor, Failure> {
    query
        .map(|Query(query)| query)
        .map_err(|_| StudioError::InvalidRequest.into())
}

pub(crate) async fn index(
    State(state): State<Arc<AppState>>,
    query: Result<Query<Cursor>, QueryRejection>,
) -> Result<Html<String>, Failure> {
    let page = state.studio.list(cursor(query)?.after.as_deref())?;
    let mut saved = String::new();
    for piece in page.pieces {
        saved.push_str(&format!(r#"<a class="piece-card" href="/review/{}"><div class="card-meta"><span>{}</span><span class="status">{}</span></div><h3>{}</h3><p class="thought-excerpt">{}</p><span class="open-piece">Review piece ↗</span></a>"#, escape(&piece.piece_id), piece.channel.display_name(), status(piece.status), escape(&piece.text), escape(&piece.source.raw_input)));
    }
    if saved.is_empty() {
        saved.push_str("<div class=\"empty card\"><h3>No pieces yet</h3><p>Your next piece starts with a thought. Add one to draft for LinkedIn.</p></div>");
    }
    if let Some(after) = page.next_after {
        saved.push_str(&format!(
            r#"<a class="button secondary" href="/?after={}">Next page</a>"#,
            escape(&after)
        ));
    }
    Ok(document(
        "Studio",
        &format!(
            r#"<section class="intro"><p class="eyebrow">The founder's studio</p><h1>Make the thought<br>worth sharing.</h1><p class="lede">Draft with intention. Review the exact version. Publish only when you decide.</p></section><div class="home-grid"><section class="card composer"><span class="step">01 / Start here</span><h2>What's on your mind?</h2><form method="post" action="/actions/thought">{}<label for="thought">Your raw thought</label><textarea id="thought" name="text" rows="8" required placeholder="An observation, a lesson, a point of view…"></textarea><p class="hint">One LinkedIn text draft. The model may take up to a minute; nothing publishes here.</p><button type="submit">Draft a piece <span aria-hidden="true">↗</span></button></form></section><section aria-labelledby="saved"><div class="section-heading"><h2 id="saved">Thoughts &amp; pieces</h2><span class="muted">Saved on this laptop</span></div><div class="pieces">{saved}</div></section></div>"#,
            csrf(&state)
        ),
    ))
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
fn status(status: Status) -> &'static str {
    match status {
        Status::PendingReview => "Pending review",
        Status::Approved => "Approved",
        Status::Rejected => "Rejected",
    }
}

pub(crate) async fn review(
    State(state): State<Arc<AppState>>,
    path: Result<Path<String>, PathRejection>,
) -> Result<Html<String>, Failure> {
    let Path(id) = path.map_err(|_| StudioError::PieceNotFound)?;
    let piece = state.studio.show(&id)?;
    Ok(piece_page(&state, &piece, None))
}

pub(crate) async fn editor(
    State(state): State<Arc<AppState>>,
    path: Result<Path<(String, String)>, PathRejection>,
) -> Result<Html<String>, Failure> {
    let Path((id, action)) = path.map_err(|_| StudioError::PieceNotFound)?;
    if !matches!(action.as_str(), "edit" | "render") {
        return Err(Failure::MethodNotAllowed);
    }
    let piece = state.studio.show(&id)?;
    if action == "render" {
        return Ok(visual_editor(&state, &piece));
    }
    Ok(document(
        "Edit draft",
        &format!(
            r#"<a class="back" href="/review/{id}">← Cancel and return to saved draft</a><section class="card edit-card"><p class="eyebrow">Edit the next revision</p><h1>Keep it in your voice.</h1><p>Saving a change clears approval and rendered assets. Review and approve the new version afterward.</p><form method="post" action="/review/{id}/edit">{}<label for="draft">Draft text</label><textarea id="draft" name="text" rows="14" required>{}</textarea><button type="submit">Save changes</button></form></section>"#,
            revision_fields(&state, &piece),
            escape(&piece.text)
        ),
    ))
}

fn visual_editor(state: &AppState, piece: &Piece) -> Html<String> {
    let slides = piece
        .visual
        .as_ref()
        .map(|v| v.slides.join("\n"))
        .unwrap_or_else(|| piece.text.clone());
    let paper = if piece.visual.as_ref().is_some_and(|v| v.template == "paper") {
        " selected"
    } else {
        ""
    };
    let id = escape(&piece.piece_id);
    document(
        "Render previews",
        &format!(
            r#"<a class="back" href="/review/{id}">← Cancel and return to saved draft</a><section class="card edit-card"><p class="eyebrow">Configure the next revision</p><h1>Give the thought a shape.</h1><p>Rendering clears earlier approval. Return to the review page to inspect the saved assets before approving.</p><form method="post" action="/review/{id}/render">{}<label for="template">Template</label><select id="template" name="template"><option value="midnight">Midnight</option><option value="paper"{paper}>Paper</option></select><label for="slides">Slides, one per line</label><textarea id="slides" name="slides" rows="8" required>{}</textarea><p class="hint">Creates a PNG and a PDF carousel. Rendering may take 30 seconds.</p><button type="submit">Render previews</button></form></section>"#,
            revision_fields(state, piece),
            escape(&slides)
        ),
    )
}

fn approval_note(piece: &Piece) -> &'static str {
    match piece.status {
        Status::Approved => "This saved revision is approved. Publishing is a separate action.",
        Status::Rejected => "This piece is rejected. Edit it to start a new review.",
        Status::PendingReview if piece.visual.is_some() && piece.assets.is_none() => {
            "Render current assets before approval."
        }
        Status::PendingReview => {
            "Approval applies only to this saved revision and the exact asset hashes shown."
        }
    }
}

#[expect(
    clippy::expect_used,
    reason = "The hash list contains only strings, which always serialize to JSON."
)]
fn piece_page(state: &AppState, piece: &Piece, receipt: Option<&PublishResponse>) -> Html<String> {
    let id = escape(&piece.piece_id);
    let channel = piece.channel.display_name();
    // The displayed account and form name the same observation, even during reconnect.
    let target = state.studio.review_target(piece).ok().flatten();
    let fields = format!(
        "{}{}",
        revision_fields(state, piece),
        target
            .as_deref()
            .map(|target| hidden("expected_target", target))
            .unwrap_or_default()
    );
    let mut hashes = String::new();
    if let Some(assets) = &piece.assets {
        hashes = format!(
            "{}{}",
            hidden("image_sha256", &assets.image_sha256),
            hidden("carousel_sha256", &assets.carousel_sha256)
        );
        if !assets.slides_sha256.is_empty() {
            hashes.push_str(&hidden(
                "slides_sha256",
                &serde_json::to_string(&assets.slides_sha256).expect("hash list"),
            ));
        }
    }
    let needs_account = state
        .studio
        .account_approval_required(piece, target.as_deref());
    let approval_disabled = if (piece.status != Status::PendingReview
        && !(piece.status == Status::Approved && needs_account))
        || (piece.visual.is_some() && piece.assets.is_none())
    {
        " disabled"
    } else {
        ""
    };
    let approval_note = if needs_account {
        "Account changed or approval is unbound. Connect, review and approve this account before publishing."
    } else {
        approval_note(piece)
    };
    let account = target
        .map(|target| format!("<p class=\"label\">Connected account fingerprint</p><code class=\"hash\">{}</code>", escape(&target)))
        .unwrap_or_default();
    let publish_controls = publish_controls(state, piece, needs_account);
    let receipt = receipt.map(publish_receipt).unwrap_or_default();
    let last_outcome = piece
        .publish_status
        .as_deref()
        .map(|s| {
            format!(
                "<p class=\"muted\">Last saved publish outcome: <strong>{}</strong></p>",
                escape(s)
            )
        })
        .unwrap_or_default();
    document(
        "Review piece",
        &format!(
            r#"<a class="back" href="/">← All thoughts &amp; pieces</a><div class="review-heading"><div><p class="eyebrow">{channel}</p><h1>Make it yours.</h1></div><span class="status prominent">{}</span></div>{receipt}<div class="review-layout"><div class="review-controls"><section class="card"><span class="step">01 / The thought</span><p class="source">{}</p></section><section class="card"><div class="section-heading"><span class="step">02 / Saved draft</span><a href="/review/{id}/edit">Edit draft ↗</a></div><p class="draft-text">{}</p><p class="hint">{} {} / {} maximum · {:?}</p><p class="label">Exact revision</p><code class="hash">{}</code></section><section class="card"><span class="step">03 / Visuals</span><h2>Render previews</h2><p class="hint">Configure slides on a separate screen, then return to inspect and approve the saved bytes. Rendering clears earlier approval.</p><a class="button secondary" href="/review/{id}/render">Render previews ↗</a></section><section class="card approval"><span class="step">04 / Your decision</span><h2>Review. Then approve.</h2><p>{approval_note}</p>{account}<form method="post" action="/review/{id}/approve">{fields}{hashes}<button type="submit"{approval_disabled}>Approve exact revision</button></form><hr>{publish_controls}{last_outcome}</section></div><section class="previews" aria-label="Phone-sized previews"><p class="eyebrow">A smaller screen. The same content.</p><div class="section-heading preview-heading"><h2>Phone previews</h2><span class="muted">360 px</span></div><p class="hint">Preview on this laptop, not a phone connection.</p>{}</section></div>"#,
            status(piece.status),
            escape(&piece.source.raw_input),
            escape(&piece.text),
            piece.fit.char_count,
            piece.channel.count_unit(),
            piece.fit.max_chars,
            piece.fit.verdict,
            escape(&piece.revision),
            format_args!("{}{}", previews(piece), jpeg_previews(piece))
        ),
    )
}

/// Keep delivery availability, cost acceptance and the Publish control together.
fn publish_controls(state: &AppState, piece: &Piece, needs_account: bool) -> String {
    let id = escape(&piece.piece_id);
    let fields = revision_fields(state, piece);
    let unavailable_image =
        piece.channel == syncloom_core::Channel::Threads && piece.visual.is_some();
    let delivery_note = delivery_note(piece, unavailable_image);
    let accepted_cost = state.cost_consents.accepted(piece);
    let publish_disabled = if needs_account
        || piece.status != Status::Approved
        || unavailable_image
        || (piece.channel == syncloom_core::Channel::X && accepted_cost.is_none())
    {
        " disabled"
    } else {
        ""
    };
    let cost_confirmation = if piece.channel == syncloom_core::Channel::X {
        let cost = syncloom_core::social::estimate(&piece.text, piece.visual.is_some());
        let acceptance = if accepted_cost.is_some() {
            r#"<p class="hint">Estimated cost accepted for this revision.</p>"#.to_owned()
        } else {
            format!(
                r#"<form class="cost-confirmation" method="post" action="/review/{id}/accept-cost">{fields}{}{}<button class="secondary" type="submit">Accept estimated cost</button></form>"#,
                hidden("accepted_cost_microusd", &cost.microusd.to_string()),
                hidden("pricing_as_of", &cost.pricing_as_of)
            )
        };
        format!(
            r#"<p class="hint">X estimated cost: <strong>{}</strong>. Pricing as of <time class="pricing-date" datetime="{}">{}</time>. This is not an actual charge; URL-like text uses the link rate.</p>{acceptance}"#,
            cost.usd(),
            escape(&cost.pricing_as_of),
            escape(&cost.pricing_as_of)
        )
    } else {
        String::new()
    };
    format!(
        r#"<p class="hint">Without a delivery grant, nothing leaves this laptop. {delivery_note}</p>{cost_confirmation}<form method="post" action="/review/{id}/publish">{fields}<button class="publish" type="submit"{publish_disabled}>Publish</button></form>"#
    )
}

fn delivery_note(piece: &Piece, unavailable_image: bool) -> &'static str {
    use syncloom_core::Channel;
    if unavailable_image {
        return "Image delivery is unavailable for Threads until a public URL host is selected. Export or use a text-only piece.";
    }
    match piece.channel {
        Channel::LinkedinText => {
            "LinkedIn visuals stay contained; only text delivery is available."
        }
        Channel::Mastodon | Channel::Bluesky | Channel::X => {
            "Granted delivery sends text and the approved PNG. PDF carousels stay local."
        }
        Channel::Threads => "Threads supports text only. Image hosting is not configured.",
        Channel::ManualExport => "Manual export is a local handoff, never a remote post.",
        Channel::FacebookPage | Channel::Instagram => {
            "Meta media delivery needs an explicitly approved public host; none is configured."
        }
    }
}

fn previews(piece: &Piece) -> String {
    let id = escape(&piece.piece_id);
    let channel = piece.channel.display_name();
    match &piece.assets {
        Some(assets) => format!(
            r#"<div class="preview-grid"><article class="preview-card"><div class="preview-label">Image / PNG</div><img class="asset-image" src="/v1/pieces/{id}/assets/image" alt="Rendered first slide for this exact revision"><div class="preview-details"><a href="/v1/pieces/{id}/assets/image" target="_blank" rel="noreferrer">Open verified PNG ↗</a><p class="label">Image SHA-256</p><code class="hash">{}</code></div></article><article class="preview-card"><div class="preview-label">Carousel / PDF · {} pages</div><div class="pdf-preview"><iframe title="Rendered PDF carousel" src="/v1/pieces/{id}/assets/carousel#toolbar=0&amp;navpanes=0&amp;view=FitH"></iframe></div><div class="preview-details"><a href="/v1/pieces/{id}/assets/carousel" target="_blank" rel="noreferrer">Open verified PDF ↗</a><p class="hint">Scroll inside the PDF to inspect every page. If your browser cannot embed PDFs, use the link above.</p><p class="label">Carousel SHA-256</p><code class="hash">{}</code></div></article></div>"#,
            escape(&assets.image_sha256),
            assets.pages,
            escape(&assets.carousel_sha256)
        ),
        None => format!(
            r#"<article class="text-phone"><div class="preview-label">{channel} preview</div><p class="draft-text">{}</p></article><div class="empty visual-empty"><h3>No current visual assets</h3><p>Render previews to inspect the image and carousel before approving.</p></div>"#,
            escape(&piece.text)
        ),
    }
}

fn jpeg_previews(piece: &Piece) -> String {
    let Some(assets) = &piece.assets else {
        return String::new();
    };
    let id = escape(&piece.piece_id);
    let slides:String=assets.slides_sha256.iter().enumerate().map(|(index,hash)|format!(
        r#"<article class="preview-card"><div class="preview-label">Meta / JPEG · slide {}</div><img class="asset-image" src="/v1/pieces/{id}/assets/slides/{index}" alt="Rendered slide {}"><div class="preview-details"><p class="label">JPEG SHA-256</p><code class="hash">{}</code></div></article>"#,
        index+1,index+1,escape(hash))).collect();
    if slides.is_empty() {
        slides
    } else {
        format!(r#"<div class="preview-grid meta-slides">{slides}</div>"#)
    }
}

fn publish_receipt(receipt: &PublishResponse) -> String {
    let explanation = match receipt.status.as_str() {
        "contained" => "Nothing was sent. The publish intent is saved in the local outbox.",
        "posted" => "The platform returned a post id. This is the recorded delivery outcome.",
        "unknown" => {
            "Delivery is unknown. Do not retry blindly; check the platform before taking action."
        }
        "refused" => "The platform refused the request. Nothing is claimed as posted.",
        "consent_required" => "Sign-in or consent is required. Nothing is claimed as posted.",
        "rate_limited" => "The platform rate-limited the request. Nothing is claimed as posted.",
        "unreachable" => "The platform was unreachable. Nothing is claimed as posted.",
        _ => "See the core outcome below. No delivery is assumed.",
    };
    let cost = receipt
        .cost_estimate
        .as_ref()
        .map(|c| {
            format!(
                "<p>Recorded estimate: {} (not an actual charge).</p>",
                c.usd()
            )
        })
        .unwrap_or_default();
    format!(
        r#"<section class="receipt" aria-label="Publish receipt"><p class="eyebrow">Publish receipt</p><h2>{}</h2><p>{explanation}</p><p>Request: <code>{}</code> · Attempts: {}</p><p>Post id: {} · Post URL: {}</p>{cost}</section>"#,
        escape(&receipt.status),
        escape(receipt.request_id.as_deref().unwrap_or("none")),
        receipt.attempts,
        escape(receipt.post_id.as_deref().unwrap_or("none")),
        escape(receipt.post_url.as_deref().unwrap_or("none"))
    )
}

// Transport data only: all domain validation remains in Studio and its existing request types.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ThoughtForm {
    csrf: String,
    text: String,
}

pub(crate) async fn thought(
    State(state): State<Arc<AppState>>,
    form: Result<Form<ThoughtForm>, FormRejection>,
) -> Result<Redirect, Failure> {
    let Form(form) = form.map_err(|_| StudioError::InvalidRequest)?;
    let _ = form.csrf; // The boundary already authenticated it before parsing this adapter.
    let response = state
        .studio
        .create_thought(ThoughtRequest {
            text: form.text,
            channels: vec!["linkedin_text".into()],
        })
        .await?;
    Ok(Redirect::to(&format!(
        "/review/{}",
        response.pieces[0].piece_id
    )))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PieceForm {
    csrf: String,
    expected_revision: String,
    expected_target: Option<String>,
    text: Option<String>,
    template: Option<String>,
    slides: Option<String>,
    image_sha256: Option<String>,
    carousel_sha256: Option<String>,
    slides_sha256: Option<String>,
    accepted_cost_microusd: Option<u64>,
    pricing_as_of: Option<String>,
}

impl PieceForm {
    fn review(self, action: &str) -> Result<ReviewRequest, StudioError> {
        let (status, text, expected_assets) = if action == "edit" {
            (
                None,
                Some(self.text.ok_or(StudioError::InvalidRequest)?),
                None,
            )
        } else {
            let hashes = match (self.image_sha256, self.carousel_sha256) {
                (None, None) => None,
                (Some(image_sha256), Some(carousel_sha256)) => Some(AssetHashes {
                    image_sha256,
                    carousel_sha256,
                    slides_sha256: self
                        .slides_sha256
                        .as_deref()
                        .map(serde_json::from_str)
                        .transpose()
                        .map_err(|_| StudioError::InvalidRequest)?
                        .unwrap_or_default(),
                }),
                _ => return Err(StudioError::InvalidRequest),
            };
            (Some(Status::Approved), None, hashes)
        };
        Ok(ReviewRequest {
            status,
            text,
            expected_assets,
            expected_target: self.expected_target,
            expected_revision: self.expected_revision,
        })
    }

    fn rendering(self) -> Result<RenderRequest, StudioError> {
        Ok(RenderRequest {
            expected_revision: self.expected_revision,
            template: self.template.ok_or(StudioError::InvalidRequest)?,
            slides: self
                .slides
                .ok_or(StudioError::InvalidRequest)?
                .lines()
                .map(str::to_owned)
                .collect(),
        })
    }
}

/// Transport parsing produces the same core request types used by JSON and CLI callers.
enum SubmittedAction {
    Review(ReviewRequest),
    Render(RenderRequest),
    Publish(PublishRequest),
}

fn submitted_action(action: &str, form: PieceForm) -> Result<SubmittedAction, Failure> {
    let _ = &form.csrf; // Authenticated by the boundary before any adapter runs.
    match action {
        "edit" | "approve" => Ok(SubmittedAction::Review(form.review(action)?)),
        "render" => Ok(SubmittedAction::Render(form.rendering()?)),
        "publish" => Ok(SubmittedAction::Publish(PublishRequest {
            expected_revision: form.expected_revision,
            accepted_cost_microusd: form.accepted_cost_microusd,
            dry_run: false,
        })),
        _ => Err(Failure::NotFound),
    }
}

pub(crate) async fn action(
    State(state): State<Arc<AppState>>,
    path: Result<Path<(String, String)>, PathRejection>,
    form: Result<Form<PieceForm>, FormRejection>,
) -> Result<Response, Failure> {
    let Path((id, action)) = path.map_err(|_| StudioError::PieceNotFound)?;
    let Form(form) = form.map_err(|_| StudioError::InvalidRequest)?;
    if action == "accept-cost" {
        let piece = state.studio.show(&id)?;
        state.cost_consents.accept(&piece, &form)?;
        return saved_piece_response(Ok(piece));
    }
    match submitted_action(&action, form)? {
        SubmittedAction::Review(request) => review_action(&state, &id, request),
        SubmittedAction::Render(request) => render_action(&state, &id, request).await,
        SubmittedAction::Publish(request) => publish_action(&state, &id, request).await,
    }
}

fn saved_piece_response(piece: Result<Piece, StudioError>) -> Result<Response, Failure> {
    Ok(Redirect::to(&format!("/review/{}", piece?.piece_id)).into_response())
}

fn review_action(state: &AppState, id: &str, request: ReviewRequest) -> Result<Response, Failure> {
    saved_piece_response(state.studio.review(id, request))
}

async fn render_action(
    state: &AppState,
    id: &str,
    request: RenderRequest,
) -> Result<Response, Failure> {
    saved_piece_response(state.studio.render(id, request).await)
}

async fn publish_action(
    state: &AppState,
    id: &str,
    mut request: PublishRequest,
) -> Result<Response, Failure> {
    // Browser publish uses only server-recorded consent, never a forged form amount.
    let piece = state.studio.show(id)?;
    if piece.revision != request.expected_revision {
        return Err(StudioError::RevisionConflict.into());
    }
    request.accepted_cost_microusd = state.cost_consents.accepted(&piece);
    if piece.channel == syncloom_core::Channel::X && request.accepted_cost_microusd.is_none() {
        return Err(StudioError::CostConfirmationRequired.into());
    }
    let receipt = state.studio.publish(id, request).await?;
    Ok((
        StatusCode::OK,
        piece_page(state, &receipt.piece, Some(&receipt)),
    )
        .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn piece() -> Piece {
        serde_json::from_value(serde_json::json!({
            "piece_id":"thought-test-x", "group_id":"test", "channel":"x",
            "platform":"x", "text":"A draft.", "revision":"revision-1",
            "status":"approved", "review_decision_id":"review-1", "publish_status":null,
            "fit":{"verdict":"PASS", "char_count":8,"min_chars":1,"max_chars":280},
            "source":{"type":"text","raw_input":"A thought."},
            "created_at":"2026-10-31T00:00:00Z", "updated_at":"2026-10-31T00:00:00Z"
        }))
        .unwrap()
    }

    // [utest->req~studio-api.browser-x-cost-consent~1]
    #[test]
    fn changed_estimate_or_date_invalidates_consent_even_for_the_same_revision() {
        let piece = piece();
        for change in ["amount", "date"] {
            let records = CostConsents::default();
            let mut old = CostConsent::current(&piece);
            if change == "amount" {
                old.estimate.microusd = 14999;
            } else {
                old.estimate.pricing_as_of = "2026-10-30".into();
            }
            records
                .0
                .lock()
                .unwrap()
                .insert(piece.piece_id.clone(), old);
            assert_eq!(records.accepted(&piece), None, "{change}");
            assert!(
                records.0.lock().unwrap().is_empty(),
                "stale record must be cleared"
            );
        }
    }

    // [utest->req~studio-api.browser-x-cost-consent~1]
    #[test]
    fn consent_is_piece_and_session_scoped() {
        let piece = piece();
        let records = CostConsents::default();
        records
            .0
            .lock()
            .unwrap()
            .insert(piece.piece_id.clone(), CostConsent::current(&piece));
        assert_eq!(records.accepted(&piece), Some(15000));
        let mut other = piece.clone();
        other.piece_id = "thought-other-x".into();
        assert_eq!(records.accepted(&other), None);
        assert_eq!(
            CostConsents::default().accepted(&piece),
            None,
            "new session requires consent"
        );
    }
}
