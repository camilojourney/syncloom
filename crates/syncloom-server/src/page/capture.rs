//! Capture adapts the workspace thought, without a second composer or draft writer.
use super::*;
use axum::extract::{RawForm, rejection::RawFormRejection};
use syncloom_core::{CaptureConfirmRequest, Preview, Suggestion};

type Raw = Result<RawForm, RawFormRejection>;

pub(crate) async fn entry() -> Redirect {
    Redirect::to("/#compose")
}

pub(crate) async fn suggest(
    State(state): State<Arc<AppState>>,
    form: Raw,
) -> Result<Html<String>, Failure> {
    let (body, _) = fields(form, false)?;
    suggestions_page(&state, &body.text, None)
}

fn source(text: &str) -> String {
    format!(
        r#"<section class="card"><h2>Your source text</h2><div class="preview-prose">{}</div><p class="hint">Your words stay whole. Links are treated as text, never opened.</p></section>"#,
        escape(text)
    )
}

fn suggestions_page(
    state: &AppState,
    text: &str,
    error: Option<&str>,
) -> Result<Html<String>, Failure> {
    let suggestions = syncloom_core::suggest(text)?;
    let choices = suggestions.iter().map(|s| format!(r#"<div class="capture-choice"><label class="channel-option"><input type="checkbox" name="channels" value="{}"><span>{}</span></label><p class="hint">Format: {}. {}</p></div>"#, s.channel.name(), s.channel.display_name(), escape(s.format_hint), escape(s.edit_notes))).collect::<String>();
    let error = error
        .map(|e| format!(r#"<p role="alert">{}</p>"#, escape(e)))
        .unwrap_or_default();
    Ok(document(
        "Capture suggestions",
        &format!(
            r#"<a class="back" href="/">Start over in workspace</a><div class="page-heading"><h1>Where could this fit?</h1><p>Local suggestions for your thought. Nothing is drafted yet.</p></div><div class="review-layout capture-layout">{}<section class="card"><h2>Choose destinations</h2><p class="hint">Based on text length and links, not connected accounts. Choose only what you want.</p><form method="post" action="/capture/preview">{}{}<fieldset><legend>Suggested destinations</legend>{error}{choices}</fieldset><button type="submit">Preview selection</button></form><p class="hint capture-note">No model call, approval or publishing at this step.</p></section></div>"#,
            source(text),
            csrf(state),
            hidden("text", text)
        ),
    ))
}

pub(crate) async fn preview(
    State(state): State<Arc<AppState>>,
    form: Raw,
) -> Result<Response, Failure> {
    let (body, _) = fields(form, false)?;
    if body.channels.is_empty() {
        return Ok((
            axum::http::StatusCode::BAD_REQUEST,
            suggestions_page(
                &state,
                &body.text,
                Some("Select at least one suggestion. No draft has been created."),
            )?,
        )
            .into_response());
    }
    let preview = syncloom_core::preview(&body.text, &body.channels)?;
    Ok(preview_page(&state, &preview).into_response())
}

fn destination(s: &Suggestion) -> String {
    format!(
        r#"<li><strong>{}</strong><p class="hint">Format: {}. {}</p></li>"#,
        s.channel.display_name(),
        escape(s.format_hint),
        escape(s.edit_notes)
    )
}
fn preview_page(state: &AppState, preview: &Preview) -> Html<String> {
    let channels = preview
        .channels
        .iter()
        .map(|c| hidden("channels", c.name()))
        .collect::<String>();
    let destinations = preview
        .suggestions
        .iter()
        .map(destination)
        .collect::<String>();
    document(
        "Capture preview",
        &format!(
            r#"<div class="page-heading"><h1>Ready to make this a draft?</h1><p>Not an approval. This is your source text, not yet a draft.</p></div><div class="review-layout capture-layout">{}<section class="card"><h2>Your selected destinations</h2><ul class="capture-destinations">{destinations}</ul><form method="post" action="/capture/confirm">{}{}{}{}<p class="hint">Confirm only your own text. It will be sent to the configured drafting provider. Review every generated draft afterward.</p><button type="submit">Confirm and draft</button></form><form class="capture-note" method="post" action="/capture/suggest">{}{}<button class="secondary" type="submit">Change destinations</button></form><p class="hint capture-note">No approval, scheduling or publishing. Drafting may take up to a minute.</p><details><summary>Preview details</summary><p class="hint">This identifies the exact source and selected destinations, not a saved piece.</p><code class="hash">{}</code></details></section></div>"#,
            source(&preview.text),
            csrf(state),
            hidden("text", &preview.text),
            channels,
            hidden("expected_revision", &preview.revision),
            csrf(state),
            hidden("text", &preview.text),
            preview.revision
        ),
    )
}

pub(crate) async fn confirm(
    State(state): State<Arc<AppState>>,
    form: Raw,
) -> Result<Redirect, Failure> {
    let (body, revision) = fields(form, true)?;
    let revision = revision.ok_or(StudioError::InvalidRequest)?;
    state
        .studio
        .confirm_capture(CaptureConfirmRequest {
            text: body.text,
            channels: body.channels,
            expected_revision: revision.clone(),
        })
        .await?;
    Ok(Redirect::to(&format!("/capture/result/{revision}")))
}

pub(crate) async fn result(
    State(state): State<Arc<AppState>>,
    path: Result<Path<String>, PathRejection>,
) -> Result<Html<String>, Failure> {
    let Path(revision) = path.map_err(|_| Failure::NotFound)?;
    let thought = state
        .studio
        .capture_result(&revision)?
        .ok_or(Failure::NotFound)?;
    let pieces = thought.pieces.iter().map(|p| format!(r#"<article class="piece-card"><h2>{}</h2><p class="prose">{}</p><a class="review-link" href="/review/{}">Review {}</a></article>"#,p.channel.display_name(),escape(&p.text),escape(&p.piece_id),p.channel.display_name())).collect::<String>();
    Ok(document(
        "Captured drafts",
        &format!(
            r#"<a class="back" href="/">Back to workspace</a><div class="page-heading"><h1>Your drafts are ready to review.</h1><p>Capture did not approve or publish anything. Open each piece to check its current saved version.</p></div><section class="editor-layout">{pieces}</section>"#
        ),
    ))
}

/// Reuse the strict thought form parser; only confirm adds one revision scalar.
fn fields(
    form: Raw,
    allow_revision: bool,
) -> Result<(ThoughtRequest, Option<String>), StudioError> {
    let RawForm(bytes) = form.map_err(|_| StudioError::InvalidRequest)?;
    let mut revision = None;
    let mut thought = url::form_urlencoded::Serializer::new(String::new());
    for (name, value) in url::form_urlencoded::parse(&bytes) {
        if allow_revision && name == "expected_revision" {
            if revision.replace(value.into_owned()).is_some() {
                return Err(StudioError::InvalidRequest);
            }
        } else {
            thought.append_pair(&name, &value);
        }
    }
    Ok((home::parse_thought(thought.finish().as_bytes())?, revision))
}
