//! Capture adapts the workspace thought, without a second composer or draft writer.
use super::*;
use axum::extract::{MatchedPath, RawForm, rejection::RawFormRejection};
use syncloom_core::{CaptureConfirmRequest, Preview, Suggestion};

type Raw = Result<RawForm, RawFormRejection>;

pub(crate) async fn entry() -> Redirect {
    Redirect::to("/#compose")
}

pub(crate) async fn suggest(
    State(state): State<Arc<AppState>>,
    route: MatchedPath,
    form: Raw,
) -> Result<Html<String>, Failure> {
    let (body, _) = fields(form, false, &state)?;
    // Only the explicit registered fetch action opens a URL, never pasted-link detection.
    if route.as_str() == "/capture/url" {
        if body.source_revision.is_some() {
            return Err(StudioError::InvalidRequest.into());
        }
        let fetched = state.studio.fetch_source(&body.text).await?;
        suggestions_page(
            &state,
            fetched.source.text(),
            Some(&fetched.source_revision),
            None,
        )
    } else {
        suggestions_page(&state, &body.text, body.source_revision.as_deref(), None)
    }
}

pub(crate) async fn attachment(
    State(state): State<Arc<AppState>>,
    axum::Extension(upload): axum::Extension<crate::upload::Upload>,
) -> Result<Html<String>, Failure> {
    let captured =
        state
            .studio
            .capture_attachment(&upload.filename, upload.mime.as_deref(), &upload.bytes)?;
    suggestions_page(
        &state,
        captured.source.text(),
        Some(&captured.source_revision),
        None,
    )
}

fn source_fields(text: &str, revision: Option<&str>) -> String {
    format!(
        "{}{}",
        hidden("text", text),
        revision
            .map(|r| hidden("source_revision", r))
            .unwrap_or_default()
    )
}

fn source(
    text: &str,
    retrieved: Option<&syncloom_core::Source>,
    provider_text: Option<&str>,
) -> String {
    if let Some(source) = retrieved {
        let heading = if source.attachment.is_some() {
            "Extracted file text"
        } else {
            "Retrieved source text"
        };
        let transfer = provider_text.filter(|_| source.attachment.is_some()).map(|text| format!(r#"<h3>Text sent to the drafting provider</h3><p class="hint">Confirmation sends this normalized text, not the file bytes or filename. Whitespace is collapsed by the existing drafting rules.</p><div class="preview-prose">{}</div>"#, escape(text))).unwrap_or_default();
        return format!(
            r#"<section class="card"><h2>{heading}</h2><div class="preview-prose">{}</div><p class="hint">Untrusted source data, not instructions. This step sends nothing to a drafting provider.</p>{transfer}{}</section>"#,
            escape(text),
            source_provenance(source)
        );
    }
    format!(
        r#"<section class="card"><h2>Your source text</h2><div class="preview-prose">{}</div><p class="hint">Your words stay whole. Links are treated as text, never opened.</p></section>"#,
        escape(text)
    )
}

fn suggestions_page(
    state: &AppState,
    text: &str,
    source_revision: Option<&str>,
    error: Option<&str>,
) -> Result<Html<String>, Failure> {
    let suggestions = syncloom_core::suggest(text)?;
    let retrieved = source_revision
        .map(|r| state.studio.captured_source(r))
        .transpose()?;
    if retrieved
        .as_ref()
        .is_some_and(|source| source.text() != text)
    {
        return Err(StudioError::RevisionConflict.into());
    }
    let choices = suggestions.iter().map(|s| format!(r#"<div class="capture-choice"><label class="channel-option"><input type="checkbox" name="channels" value="{}"><span>{}</span></label><p class="hint">Format: {}. {}</p></div>"#, s.channel.name(), s.channel.display_name(), escape(s.format_hint), escape(s.edit_notes))).collect::<String>();
    let error = error
        .map(|e| format!(r#"<p role="alert">{}</p>"#, escape(e)))
        .unwrap_or_default();
    Ok(document(
        "Capture suggestions",
        &format!(
            r#"<a class="back" href="/">Start over in workspace</a><div class="page-heading"><h1>Where could this fit?</h1><p>Local suggestions for your source. This step creates no drafts.</p></div><div class="review-layout capture-layout">{}<section class="card"><h2>Choose destinations</h2><p class="hint">Based on text length and links, not connected accounts. Choose only what you want.</p><form method="post" action="/capture/preview">{}{}<fieldset><legend>Suggested destinations</legend>{error}{choices}</fieldset><button type="submit">Preview selection</button></form><p class="hint capture-note">No model call, approval or publishing at this step.</p></section></div>"#,
            source(text, retrieved.as_ref(), None),
            csrf(state),
            source_fields(text, source_revision)
        ),
    ))
}

pub(crate) async fn preview(
    State(state): State<Arc<AppState>>,
    form: Raw,
) -> Result<Response, Failure> {
    let (body, _) = fields(form, false, &state)?;
    if body.channels.is_empty() {
        return Ok((
            axum::http::StatusCode::BAD_REQUEST,
            suggestions_page(
                &state,
                &body.text,
                body.source_revision.as_deref(),
                Some("Select at least one suggestion. No draft has been created."),
            )?,
        )
            .into_response());
    }
    let preview = state.studio.preview_capture(
        &body.text,
        &body.channels,
        body.source_revision.as_deref(),
    )?;
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
    let transfer = if preview
        .source
        .as_ref()
        .is_some_and(|s| s.attachment.is_some())
    {
        "The normalized text shown above will be sent to the configured drafting provider as untrusted data."
    } else {
        "This exact text will be sent to the configured drafting provider as untrusted data."
    };
    document(
        "Capture preview",
        &format!(
            r#"<div class="page-heading"><h1>Ready to make this a draft?</h1><p>Not an approval. This is your source text, not yet a draft.</p></div><div class="review-layout capture-layout">{}<section class="card"><h2>Your selected destinations</h2><ul class="capture-destinations">{destinations}</ul><form method="post" action="/capture/confirm">{}{}{}{}<p class="hint">Confirm only source text you have permission to use. {transfer} Review every generated draft afterward.</p><button type="submit">Confirm and draft</button></form><form class="capture-note" method="post" action="/capture/suggest">{}{}<button class="secondary" type="submit">Change destinations</button></form><p class="hint capture-note">No approval, scheduling or publishing. Drafting may take up to a minute.</p><details><summary>Preview details</summary><p class="hint">This identifies the exact source and selected destinations, not a saved piece.</p><code class="hash">{}</code></details></section></div>"#,
            source(
                &preview.text,
                preview.source.as_ref(),
                Some(&preview.normalized_thought)
            ),
            csrf(state),
            source_fields(&preview.text, preview.source_revision.as_deref()),
            channels,
            hidden("expected_revision", &preview.revision),
            csrf(state),
            source_fields(&preview.text, preview.source_revision.as_deref()),
            preview.revision
        ),
    )
}

pub(crate) async fn confirm(
    State(state): State<Arc<AppState>>,
    form: Raw,
) -> Result<Redirect, Failure> {
    let (body, revision) = fields(form, true, &state)?;
    let revision = revision.ok_or(StudioError::InvalidRequest)?;
    state
        .studio
        .confirm_capture(CaptureConfirmRequest {
            text: body.text,
            channels: body.channels,
            expected_revision: revision.clone(),
            source_revision: body.source_revision,
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
    state: &AppState,
) -> Result<(crate::capture::Selection, Option<String>), StudioError> {
    let RawForm(bytes) = form.map_err(|_| StudioError::InvalidRequest)?;
    let mut revision = None;
    let mut source_revision = None;
    let mut thought = url::form_urlencoded::Serializer::new(String::new());
    for (name, value) in url::form_urlencoded::parse(&bytes) {
        if allow_revision && name == "expected_revision" {
            if revision.replace(value.into_owned()).is_some() {
                return Err(StudioError::InvalidRequest);
            }
        } else if name == "source_revision" {
            if source_revision.replace(value.into_owned()).is_some() {
                return Err(StudioError::InvalidRequest);
            }
        } else {
            thought.append_pair(&name, &value);
        }
    }
    let body = home::parse_thought(thought.finish().as_bytes())?;
    let text = browser_text(state, source_revision.as_deref(), body.text)?;
    Ok((
        crate::capture::Selection {
            text,
            channels: body.channels,
            source_revision,
        },
        revision,
    ))
}

fn browser_text(
    state: &AppState,
    receipt: Option<&str>,
    text: String,
) -> Result<String, StudioError> {
    let Some(receipt) = receipt else {
        return Ok(text);
    };
    if state.studio.captured_source(receipt)?.attachment.is_some() {
        // Native forms serialize LF as CRLF; canonicalize representation, never source content.
        Ok(syncloom_core::attachment::normalize_line_endings(&text))
    } else {
        Ok(text)
    }
}
