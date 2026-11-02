use super::*;
use syncloom_core::outbox::DispatchIntent;

pub(crate) async fn review(
    State(state): State<Arc<AppState>>,
    path: Result<Path<String>, PathRejection>,
) -> Result<Html<String>, Failure> {
    let Path(id) = path.map_err(|_| StudioError::PieceNotFound)?;
    let snapshot = state.studio.inspect_publish(&id)?;
    Ok(piece_page(
        &state,
        &snapshot.piece,
        snapshot.publish_intent.as_ref(),
        snapshot.approved_at.as_deref(),
    ))
}

pub(super) fn piece_page(
    state: &AppState,
    piece: &Piece,
    record: Option<&DispatchIntent>,
    approved_at: Option<&str>,
) -> Html<String> {
    let id = escape(&piece.piece_id);
    let channel = piece.channel.display_name();
    let publish_badge = record
        .map(|r| status::delivery(r.status.as_str(), "Publishing"))
        .unwrap_or_default();
    let receipt_html = record.map(receipt_view).unwrap_or_default();
    let last_operation = match piece.publish_status.as_deref() {
        // Only hide an exact duplicate of the visible banner, never separate export history.
        Some(s) if record.is_some_and(|r| r.status.as_str() == s) => String::new(),
        Some(s) => format!(
            r#"<p class="hint last-operation">Most recently: <strong>{}</strong>. {}</p><details class="operation-details"><summary>Latest action details</summary><p class="hint">Last operation: {}</p></details>"#,
            status::label(s), status::explanation(s), escape(s)
        ),
        None if record.is_some() => String::new(),
        None => "<p class=\"hint\">This version hasn't been sent. Approval alone does not connect an account or allow publishing.</p>".into(),
    };
    let blocked = record.is_some_and(|r| !r.status.allows_new_attempt());
    document(
        "Review piece",
        &format!(
            r#"<a class="back" href="/">All thoughts &amp; pieces</a><div class="review-heading"><div><h1>Review your piece.</h1><p class="channel-name">{channel}</p></div><div class="status-row">{}{publish_badge}</div></div>{receipt_html}<div class="review-layout"><div class="review-content"><section class="card local-preview-card" aria-labelledby="preview-title"><div class="section-heading"><h2 id="preview-title">{channel} preview</h2><a href="/review/{id}/edit">Edit draft</a></div><p class="hint preview-disclaimer">Approximate local preview. It may look different on the platform.</p><p class="preview-prose draft-text">{}</p>{}<details class="revision"><summary>Exact saved version</summary><p class="hint">Exact saved revision. Approval covers this version, not unsaved edits.</p><code class="hash">{}</code></details></section><section class="source-section"><h2>The source thought</h2><p class="source prose">{}</p>{}</section>{}</div><aside class="review-controls" aria-label="Review and publish"><section class="card approval"><h2>Your decision</h2>{}<hr>{}{last_operation}</section><details class="visual-config"><summary>Visual previews</summary><p class="hint">Optional: make slides from your text. Making new previews means you'll need to check and approve them again.</p><a class="button secondary" href="/review/{id}/render">Render previews</a></details></aside></div>"#,
            status::review(piece.status),
            escape(&piece.text),
            counts::summary(piece),
            escape(&piece.revision),
            escape(&piece.source.raw_input),
            routing(piece),
            previews::assets(piece),
            approval_controls(state, piece, approved_at),
            publish_controls(state, piece, blocked)
        ),
    )
}

fn routing(piece: &Piece) -> String {
    let Some(routing) = &piece.routing else {
        return "<p class=\"hint\">Suggested review target: not recorded</p>".into();
    };
    let products = routing.products.join(", ");
    format!(
        r#"<details class="routing"><summary>Suggested review target: {} ({})</summary><p class="hint">Derived from source language; not a connected account or publish permission.</p><dl><dt>Thesis</dt><dd>{}</dd><dt>Visual hook</dt><dd>{}</dd><dt>Products mentioned</dt><dd>{}</dd></dl></details>"#,
        escape(&routing.destination.handle),
        match routing.destination.language {
            syncloom_core::routing::Language::En => "EN",
            syncloom_core::routing::Language::Es => "ES",
        },
        escape(&routing.thesis),
        escape(&routing.visual_hook),
        if products.is_empty() {
            "None detected".into()
        } else {
            escape(&products)
        }
    )
}

fn approval_controls(state: &AppState, piece: &Piece, approved_at: Option<&str>) -> String {
    if piece.status == Status::Approved {
        let date = approved_at
            .map(|at| {
                format!(
                    "Approved on <time datetime=\"{}\">{}</time>.",
                    escape(at),
                    escape(at.split('T').next().unwrap_or(at))
                )
            })
            .unwrap_or_else(|| "Approved. Approval date is not recorded.".into());
        return format!("<p>{date} Publishing is a separate action.</p>");
    }
    if piece.status == Status::Rejected {
        return "<p>This piece is rejected. Edit it to start a new review.</p>".into();
    }
    let needs_previews = piece.visual.is_some() && piece.assets.is_none();
    format!(
        r#"<p>{}</p><form method="post" action="/review/{}/approve">{}{}<button type="submit"{}>Approve this version</button></form>"#,
        if needs_previews {
            "Make the current previews before approving."
        } else {
            "Read the saved text and any previews before approving this version."
        },
        escape(&piece.piece_id),
        revision_fields(state, piece),
        asset_fields(piece),
        if needs_previews { " disabled" } else { "" }
    )
}

#[expect(
    clippy::expect_used,
    reason = "An ordered list of hash strings always serializes to JSON."
)]
fn asset_fields(piece: &Piece) -> String {
    let Some(assets) = &piece.assets else {
        return String::new();
    };
    let mut fields = format!(
        "{}{}",
        hidden("image_sha256", &assets.image_sha256),
        hidden("carousel_sha256", &assets.carousel_sha256)
    );
    if !assets.slides_sha256.is_empty() {
        fields.push_str(&hidden(
            "slides_sha256",
            &serde_json::to_string(&assets.slides_sha256).expect("hash list"),
        ));
    }
    fields
}

/// Core intent policy, review approval and consent jointly control the visible action.
fn publish_controls(state: &AppState, piece: &Piece, blocked: bool) -> String {
    let id = escape(&piece.piece_id);
    let unavailable_image = piece.channel == Channel::Threads && piece.visual.is_some();
    let accepted_cost = state.cost_consents.accepted(piece);
    let disabled = if piece.status != Status::Approved
        || blocked
        || unavailable_image
        || (piece.channel == Channel::X && accepted_cost.is_none())
    {
        " disabled"
    } else {
        ""
    };
    let recovery = if blocked {
        "This version can't be sent again here. Check what happened above before doing anything else."
    } else if piece.status != Status::Approved {
        "Approve this saved version before publishing."
    } else if unavailable_image {
        "Threads supports text only here. Image delivery is unavailable."
    } else {
        "Approval does not connect your account or allow publishing. Your account must be set up and publishing allowed separately."
    };
    format!(
        r#"<h3>Publish separately</h3><p class="hint">{recovery}</p>{}<form method="post" action="/review/{id}/publish">{}<button class="publish" type="submit"{disabled}>Publish</button></form><p class="hint">If publishing isn't available, this stays saved on this laptop and isn't sent.</p>"#,
        cost_confirmation(state, piece, accepted_cost.is_some()),
        revision_fields(state, piece)
    )
}

fn cost_confirmation(state: &AppState, piece: &Piece, accepted: bool) -> String {
    if piece.channel != Channel::X {
        return String::new();
    }
    let cost = syncloom_core::social::estimate(&piece.text, piece.visual.is_some());
    let acceptance = if accepted {
        "<p class=\"hint\">Estimated cost accepted for this version.</p>".into()
    } else {
        format!(
            r#"<form class="cost-confirmation" method="post" action="/review/{}/accept-cost">{}{}{}<button class="secondary" type="submit">Accept estimated cost</button></form>"#,
            escape(&piece.piece_id),
            revision_fields(state, piece),
            hidden("accepted_cost_microusd", &cost.microusd.to_string()),
            hidden("pricing_as_of", &cost.pricing_as_of)
        )
    };
    format!(
        r#"<div class="cost-note"><p class="hint">X estimated cost: <strong>{}</strong>. Pricing as of <time class="pricing-date" datetime="{}">{}</time>. This is not an actual charge; URL-like text uses the link rate.</p>{acceptance}</div>"#,
        cost.usd(),
        escape(&cost.pricing_as_of),
        escape(&cost.pricing_as_of)
    )
}

/// Publish redirects here: one persisted snapshot owns both receipt and visible actions.
fn receipt_view(record: &DispatchIntent) -> String {
    let status = record.status.as_str();
    let attempts = record.attempts;
    let estimate = record.cost_estimate.as_ref().map(|cost| format!(
        r#"<p class="hint">Recorded estimate: {} · Pricing as of {}. Not an actual charge.</p>"#,
        escape(&cost.usd()), escape(&cost.pricing_as_of)
    )).unwrap_or_default();
    format!(
        r#"<section class="receipt" aria-label="Publishing result"><div class="receipt-heading"><h2>{}</h2><span class="hint">Publishing result</span></div><p>{}</p><details class="receipt-details"><summary>Publishing details</summary><p class="hint">Status: {status}. Request: <code class="hash">{}</code> · Attempts: {attempts}</p><p class="hint receipt-detail">Post id: {} · Post URL: {}</p>{estimate}</details></section>"#,
        status::label(status),
        status::explanation(status),
        escape(&record.request_id),
        escape(record.post_id.as_deref().unwrap_or("not recorded")),
        escape(record.post_url.as_deref().unwrap_or("not recorded"))
    )
}
