use super::actions::PieceAction;
use super::*;

pub(crate) async fn editor(
    State(state): State<Arc<AppState>>,
    path: Result<Path<(String, String)>, PathRejection>,
) -> Result<Html<String>, Failure> {
    let Path((id, action)) = path.map_err(|_| StudioError::PieceNotFound)?;
    let action = PieceAction::parse(&action)?;
    if !matches!(action, PieceAction::Edit | PieceAction::Render) {
        return Err(Failure::MethodNotAllowed);
    }
    let piece = state.studio.show(&id)?;
    Ok(if matches!(action, PieceAction::Render) {
        visual_editor(&state, &piece)
    } else {
        text_editor(&state, &piece)
    })
}

fn text_editor(state: &AppState, piece: &Piece) -> Html<String> {
    let id = escape(&piece.piece_id);
    document(
        "Edit draft",
        &format!(
            r#"<div class="editor-layout"><a class="back" href="/review/{id}">Return to piece review</a><section class="card edit-card"><h1>Keep it in your voice.</h1><p class="editor-intro">Saving changes removes approval and any earlier previews. Read and approve the new version afterward.</p><p class="hint">{}</p>{}<form method="post" action="/review/{id}/edit">{}<label for="draft">Draft text</label><textarea id="draft" name="text" rows="14" required>{}</textarea><div class="form-actions"><button type="submit">Save changes</button><a href="/review/{id}">Cancel</a></div></form></section></div>"#,
            piece.channel.display_name(),
            counts::summary(piece),
            revision_fields(state, piece),
            escape(&piece.text)
        ),
    )
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
            r#"<div class="editor-layout"><a class="back" href="/review/{id}">Return to piece review</a><section class="card edit-card"><h1>Render previews</h1><p class="editor-intro">Making new previews removes the earlier approval. Check the new previews on the review page before approving them.</p>{}<form method="post" action="/review/{id}/render">{}<label for="template">Template</label><select id="template" name="template"><option value="midnight">Midnight</option><option value="paper"{paper}>Paper</option></select><label for="slides">Slides, one per line</label><textarea id="slides" name="slides" rows="8" required>{}</textarea><p class="hint">Makes an image and a PDF slideshow. This may take 30 seconds.</p><button type="submit">Render previews</button></form></section></div>"#,
            counts::summary(piece),
            revision_fields(state, piece),
            escape(&slides)
        ),
    )
}
