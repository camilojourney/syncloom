use super::*;
use syncloom_core::outbox::DispatchIntent;

pub(crate) async fn review(
    State(state): State<Arc<AppState>>,
    path: Result<Path<String>, PathRejection>,
) -> Result<Html<String>, Failure> {
    let Path(id) = path.map_err(|_| StudioError::PieceNotFound)?;
    let snapshot = state.studio.inspect_publish(&id)?;
    Ok(piece_page(&state, &snapshot))
}

pub(super) fn piece_page(
    state: &AppState,
    snapshot: &syncloom_core::studio::ReviewSnapshot,
) -> Html<String> {
    let piece = &snapshot.piece;
    let record = snapshot.publish_intent.as_ref();
    let current_target = snapshot.current_target.as_deref();
    let account_required = state
        .studio
        .account_approval_required(piece, current_target);
    let id = escape(&piece.piece_id);
    let channel = piece.channel.display_name();
    let review_badge = if piece.status == Status::Approved && account_required {
        status::badge("pending", "Review: Needs fresh approval")
    } else {
        status::review(piece.status)
    };
    let publish_badge = record
        .map(|r| status::delivery(r.status.as_str(), "Publishing"))
        .unwrap_or_default();
    let receipt_html = record
        .map(|record| receipt_view(record, channel))
        .unwrap_or_default();
    let last_operation = match piece.publish_status.as_deref() {
        // Only hide an exact duplicate of the visible banner, never separate export history.
        Some(s) if record.is_some_and(|r| r.status.as_str() == s) => String::new(),
        Some(s) => format!(
            r#"<p class="hint last-operation">Most recently: <strong>{}</strong>. {}</p><details class="operation-details"><summary>Latest action details</summary><p class="hint">Last operation: {}</p></details>"#,
            status::label(s), status::explanation(s, channel), escape(s)
        ),
        None if record.is_some() => String::new(),
        None => "<p class=\"hint\">This version hasn't been sent. Approval alone does not connect an account or allow publishing.</p>".into(),
    };
    let blocked = record.is_some_and(|r| !r.status.allows_new_attempt());
    document(
        "Review piece",
        &format!(
            r#"<a class="back" href="/">All thoughts &amp; pieces</a><div class="review-heading"><div><h1>Review your piece.</h1><p class="channel-name">{channel}</p></div><div class="status-row">{}{publish_badge}</div></div>{receipt_html}<div class="review-layout"><div class="review-content"><section class="card local-preview-card" aria-labelledby="preview-title"><div class="section-heading"><h2 id="preview-title">{channel} preview</h2><a href="/review/{id}/edit">Edit draft</a></div><p class="hint preview-disclaimer">Approximate local preview. It may look different on the platform.</p><p class="preview-prose draft-text">{}</p>{}<details class="revision"><summary>Exact saved version</summary><p class="hint">Exact saved revision. Approval covers this version, not unsaved edits.</p><code class="hash">{}</code></details></section><section class="source-section"><h2>The source thought</h2><p class="source prose">{}</p>{}</section>{}</div><aside class="review-controls" aria-label="Review and publish"><section class="card approval"><h2>Your decision</h2>{}<hr>{}{last_operation}</section><details class="visual-config"><summary>Visual previews</summary><p class="hint">Optional: make slides from your text. Making new previews means you'll need to check and approve them again.</p><a class="button secondary" href="/review/{id}/render">Render previews</a></details></aside></div>"#,
            review_badge,
            escape(&piece.text),
            counts::summary(piece),
            escape(&piece.revision),
            escape(piece.source.text()),
            format_args!("{}{}", source_provenance(&piece.source), routing(piece)),
            previews::assets(piece),
            approval_controls(state, snapshot, account_required),
            publish_controls(state, piece, blocked, account_required)
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

fn approval_controls(
    state: &AppState,
    snapshot: &syncloom_core::studio::ReviewSnapshot,
    account_required: bool,
) -> String {
    let piece = &snapshot.piece;
    let account = snapshot.current_target.as_deref().map(|target| format!(
        "<details class=\"account-details\"><summary>Connected account</summary><p class=\"hint\">A new approval will apply to this connected account fingerprint.</p><code class=\"hash\">{}</code></details>", escape(target)
    )).unwrap_or_default();
    let approved = if piece.status == Status::Approved {
        let label = if account_required {
            "Historical approval"
        } else {
            "Approved"
        };
        let date = snapshot
            .approved_at
            .as_deref()
            .map(|at| {
                format!(
                    "{label} on <time datetime=\"{}\">{}</time>.",
                    escape(at),
                    escape(at.split('T').next().unwrap_or(at))
                )
            })
            .unwrap_or_else(|| format!("{label}. Approval date is not recorded."));
        format!("<p>{date} Publishing is a separate action.</p>")
    } else {
        String::new()
    };
    if piece.status == Status::Approved && !account_required {
        return format!("{approved}{account}");
    }
    if piece.status == Status::Rejected {
        return "<p>This piece is rejected. Edit it to start a new review.</p>".into();
    }
    let needs_previews = piece.visual.is_some() && piece.assets.is_none();
    format!(
        r#"{approved}<p>{}</p>{account}<form method="post" action="/review/{}/approve">{}{}{}<button type="submit"{}>Approve this version</button></form>"#,
        if account_required && piece.status == Status::Approved {
            "A fresh approval is needed for the connected account. The saved approval is for a different account or has no account recorded."
        } else if needs_previews {
            "Make the current previews before approving."
        } else {
            "Read the saved text and any previews before approving this version."
        },
        escape(&piece.piece_id),
        revision_fields(state, piece),
        asset_fields(piece),
        snapshot
            .current_target
            .as_deref()
            .map(|target| hidden("expected_target", target))
            .unwrap_or_default(),
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
fn publish_controls(
    state: &AppState,
    piece: &Piece,
    blocked: bool,
    account_required: bool,
) -> String {
    let id = escape(&piece.piece_id);
    let unavailable_image = piece.channel == Channel::Threads && piece.visual.is_some();
    let accepted_cost = state.cost_consents.accepted(piece);
    let cannot_post = cannot_post_note(state, piece);
    let disabled = if piece.status != Status::Approved
        || blocked
        || account_required
        || unavailable_image
        || cannot_post.is_some()
        || (piece.channel == Channel::X && accepted_cost.is_none())
    {
        " disabled"
    } else {
        ""
    };
    let recovery = if blocked {
        "This version can't be sent again here. Check what happened above before doing anything else."
    } else if account_required {
        "Connect the intended account, then approve this version for that account before publishing."
    } else if piece.status != Status::Approved {
        "Approve this saved version before publishing."
    } else {
        "Approval does not connect your account or allow publishing. Your account must be set up and publishing allowed separately."
    };
    let image_notice = if unavailable_image {
        "<p class=\"hint\">Threads supports text only here. Image delivery is unavailable.</p>"
    } else {
        ""
    };
    let cannot_post = cannot_post.unwrap_or_default();
    format!(
        r#"<h3>Publish separately</h3><p class="hint">{recovery}</p>{image_notice}{cannot_post}{}<form method="post" action="/review/{id}/publish">{}<button class="publish" type="submit"{disabled}>Publish</button></form><p class="hint">If publishing isn't available, this stays saved on this laptop and isn't sent.</p>"#,
        cost_confirmation(state, piece, accepted_cost.is_some()),
        revision_fields(state, piece)
    )
}

fn cannot_post_note(state: &AppState, piece: &Piece) -> Option<String> {
    use syncloom_core::{Channel, ConnectionStatus};
    let connection = match piece.channel {
        Channel::LinkedinText => state.studio.linkedin_connection().ok(),
        Channel::Mastodon => state
            .studio
            .mastodon_connection_for(piece.visual.is_some())
            .ok(),
        Channel::FacebookPage | Channel::Instagram => state.studio.meta().ok().and_then(|meta| {
            meta.connection_for(piece.channel, piece.visual.is_some())
                .ok()
        }),
        Channel::Bluesky | Channel::X | Channel::Threads => {
            state.studio.social().ok().and_then(|social| {
                social
                    .connection_for(piece.channel, piece.visual.is_some())
                    .ok()
            })
        }
        Channel::ManualExport => None,
    }?;
    if connection.status != ConnectionStatus::CannotPost {
        return None;
    }
    let platform = piece.channel.display_name();
    let sentence = connection
        .reason
        .as_deref()
        .and_then(|reason| status::readiness_sentence(reason, platform))
        .or_else(|| status::readiness_sentence("posting_permission_missing", platform))?;
    Some(format!(
        r#"<p class="hint">{}</p><form method="post" action="/review/{}/reconnect">{}<button class="secondary" type="submit">Reconnect account</button></form>"#,
        escape(&sentence),
        escape(&piece.piece_id),
        csrf(state)
    ))
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
fn receipt_view(record: &DispatchIntent, platform: &str) -> String {
    let status = record.status.as_str();
    let attempts = record.attempts;
    let estimate = record.cost_estimate.as_ref().map(|cost| format!(
        r#"<p class="hint">Recorded estimate: {} · Pricing as of {}. Not an actual charge.</p>"#,
        escape(&cost.usd()), escape(&cost.pricing_as_of)
    )).unwrap_or_default();
    format!(
        r#"<section class="receipt" aria-label="Publishing result"><div class="receipt-heading"><h2>{}</h2><span class="hint">Publishing result</span></div><p>{}</p><details class="receipt-details"><summary>Publishing details</summary><p class="hint">Status: {status}. Request: <code class="hash">{}</code> · Attempts: {attempts}</p><p class="hint receipt-detail">Post id: {} · Post URL: {}</p>{estimate}</details></section>"#,
        status::label(status),
        status::explanation(status, platform),
        escape(&record.request_id),
        escape(record.post_id.as_deref().unwrap_or("not recorded")),
        escape(record.post_url.as_deref().unwrap_or("not recorded"))
    )
}
