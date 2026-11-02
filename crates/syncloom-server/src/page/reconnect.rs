use super::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReconnectForm {
    csrf: String,
    callback_url: Option<String>,
}

impl ReconnectForm {
    fn resolve(
        state: &AppState,
        path: Result<Path<String>, PathRejection>,
        form: Result<Form<Self>, FormRejection>,
    ) -> Result<(syncloom_core::Piece, Self), StudioError> {
        let Path(id) = path.map_err(|_| StudioError::PieceNotFound)?;
        let Form(form) = form.map_err(|_| StudioError::InvalidRequest)?;
        // The shared boundary authenticated this bounded native form before resolving a piece.
        let _ = &form.csrf;
        Ok((state.studio.show(&id)?, form))
    }
}

/// Native browser adapter only: the existing integration owns consent, identity and storage.
pub(crate) async fn begin(
    State(state): State<Arc<AppState>>,
    path: Result<Path<String>, PathRejection>,
    form: Result<Form<ReconnectForm>, FormRejection>,
) -> Result<Html<String>, Failure> {
    let (piece, _) = ReconnectForm::resolve(&state, path, form)?;
    let id = &piece.piece_id;
    let start = match piece.channel {
        Channel::LinkedinText => state.studio.linkedin_sign_in_begin(),
        Channel::Mastodon => state.studio.mastodon_sign_in_begin().await,
        Channel::X => state.studio.social()?.x.sign_in_begin(),
        Channel::Threads => state.studio.social()?.threads.sign_in_begin(),
        Channel::FacebookPage | Channel::Instagram => state.studio.meta()?.sign_in_begin(),
        _ => Err(StudioError::UnsupportedChannel),
    }?;
    let callback = if piece.channel == Channel::Threads {
        "<label for=\"callback-url\">Returned callback URL</label><input id=\"callback-url\" name=\"callback_url\" type=\"url\" required autocomplete=\"off\"><p class=\"hint\">After signing in, paste the full returned URL here. It is used only to finish this sign-in.</p>"
    } else {
        ""
    };
    Ok(document(
        "Reconnect account",
        &format!(
            r#"<a class="back" href="/review/{}">Back to piece</a><section class="card"><h1>Reconnect {}.</h1><p>This does not approve or publish your piece.</p><p>Sign in in the new tab, then return here to finish reconnecting.</p><a class="button secondary" id="provider-sign-in" href="{}" target="_blank" rel="noreferrer">Continue to sign in</a><form method="post" action="/review/{}/reconnect/complete">{}{callback}<button type="submit">Finish reconnecting</button></form></section>"#,
            escape(id),
            piece.channel.display_name(),
            escape(&start.authorization_url),
            escape(id),
            csrf(&state)
        ),
    ))
}

pub(crate) async fn complete(
    State(state): State<Arc<AppState>>,
    path: Result<Path<String>, PathRejection>,
    form: Result<Form<ReconnectForm>, FormRejection>,
) -> Result<Redirect, Failure> {
    let (piece, form) = ReconnectForm::resolve(&state, path, form)?;
    let id = &piece.piece_id;
    match piece.channel {
        Channel::LinkedinText => state.studio.linkedin_sign_in_wait().await,
        Channel::Mastodon => state.studio.mastodon_sign_in_wait().await,
        Channel::X => state.studio.social()?.x.sign_in_wait().await,
        Channel::Threads => {
            state
                .studio
                .social()?
                .threads
                .complete(
                    form.callback_url
                        .as_deref()
                        .ok_or(StudioError::InvalidRequest)?,
                )
                .await
        }
        Channel::FacebookPage | Channel::Instagram => state.studio.meta()?.sign_in_wait().await,
        _ => Err(StudioError::UnsupportedChannel),
    }?;
    Ok(Redirect::to(&format!("/review/{id}")))
}
