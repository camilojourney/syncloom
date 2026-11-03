use super::*;
use axum::extract::{RawForm, rejection::RawFormRejection};

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
    let mut saved = page.pieces.iter().map(piece_card).collect::<String>();
    if saved.is_empty() {
        saved.push_str(r#"<div class="empty"><h3>No pieces yet</h3><p>Start with an observation, a lesson, or a point of view. Your drafts will live here until you decide what to share.</p></div>"#);
    }
    if let Some(after) = page.next_after {
        saved.push_str(&format!(r#"<nav class="pagination" aria-label="Saved pieces"><a class="button secondary" href="/?after={}">Next page</a><a href="/">Start of list</a></nav>"#, escape(&after)));
    }
    Ok(document(
        "Workspace",
        &format!(
            r#"<div class="page-heading"><h1>Your writing, in one place.</h1><p>Start with a thought. Make it yours before it goes anywhere.</p></div><div class="home-grid"><section class="card composer" aria-labelledby="compose"><h2 id="compose">What's on your mind?</h2><form method="post" action="/actions/thought">{}<label for="thought">Your thought</label><textarea id="thought" name="text" rows="7" required minlength="8" aria-describedby="thought-hint" placeholder="Something you noticed. Something worth sharing."></textarea><p class="hint" id="thought-hint">At least 8 characters, not counting spaces. Drafting may take up to a minute; nothing publishes here.</p><fieldset><legend>Draft for</legend><div class="channel-options">{}</div></fieldset><button type="submit">Draft a piece</button><button class="secondary capture-note" type="submit" formaction="/capture/suggest">Suggest destinations first</button></form><p class="composer-note">Every draft stays in review.<br>Publishing is always a separate decision.</p></section><section class="saved-section" aria-labelledby="saved"><div class="section-heading"><h2 id="saved">Saved pieces</h2><span class="hint">On this laptop</span></div><p class="hint">Approved doesn't mean published. Open a piece to see whether it was sent.</p><div class="pieces">{saved}</div></section></div>"#,
            csrf(&state),
            channel_options()
        ),
    ))
}

fn channel_options() -> String {
    // This is presentation order only; Channel owns identities, names and validation.
    [Channel::LinkedinText, Channel::X, Channel::Bluesky, Channel::Threads,
     Channel::Mastodon, Channel::FacebookPage, Channel::Instagram, Channel::ManualExport]
        .iter().map(|channel| format!(
            r#"<label class="channel-option"><input type="checkbox" name="channels" value="{}"{}><span>{}</span></label>"#,
            channel.name(), if *channel == Channel::LinkedinText { " checked" } else { "" }, channel.display_name()
        )).collect()
}

fn piece_card(piece: &Piece) -> String {
    let target = piece
        .routing
        .as_ref()
        .map(|r| escape(&r.destination.handle))
        .unwrap_or_else(|| "Target not recorded".into());
    let outcome = piece
        .publish_status
        .as_deref()
        .map(|s| status::delivery(s, "Most recently"))
        .unwrap_or_default();
    format!(
        r#"<article class="piece-card"><div class="card-meta"><span>{}</span><span class="hint">{target}</span></div><h3><a href="/review/{}">{}</a></h3><div class="status-row">{}{outcome}</div>{}<a class="review-link" href="/review/{}">Review piece</a><details><summary>Source thought</summary><p class="hint">{}</p></details></article>"#,
        piece.channel.display_name(),
        escape(&piece.piece_id),
        escape(&piece.text),
        status::review(piece.status),
        counts::summary(piece),
        escape(&piece.piece_id),
        escape(&piece.source.raw_input)
    )
}

/// Repeated channels are ordered values, not a map; scalars and unknown names stay strict.
pub(super) fn parse_thought(bytes: &[u8]) -> Result<ThoughtRequest, StudioError> {
    let mut text = None;
    let mut csrf_seen = false;
    let mut channels = Vec::new();
    for (name, value) in url::form_urlencoded::parse(bytes) {
        match name.as_ref() {
            "csrf" if !csrf_seen => csrf_seen = true, // Authenticated once by Boundary.
            "text" if text.is_none() => text = Some(value.into_owned()),
            "channels" => channels.push(value.into_owned()),
            _ => return Err(StudioError::InvalidRequest),
        }
    }
    Ok(ThoughtRequest {
        text: text.ok_or(StudioError::InvalidRequest)?,
        channels,
    })
}

pub(crate) async fn thought(
    State(state): State<Arc<AppState>>,
    form: Result<RawForm, RawFormRejection>,
) -> Result<Redirect, Failure> {
    let RawForm(bytes) = form.map_err(|_| StudioError::InvalidRequest)?;
    let response = state.studio.create_thought(parse_thought(&bytes)?).await?;
    Ok(Redirect::to(&format!(
        "/review/{}",
        response.pieces[0].piece_id
    )))
}
