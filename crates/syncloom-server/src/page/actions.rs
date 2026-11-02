use super::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PieceForm {
    csrf: String,
    pub(super) expected_revision: String,
    expected_target: Option<String>,
    text: Option<String>,
    template: Option<String>,
    slides: Option<String>,
    image_sha256: Option<String>,
    carousel_sha256: Option<String>,
    slides_sha256: Option<String>,
    pub(super) accepted_cost_microusd: Option<u64>,
    pub(super) pricing_as_of: Option<String>,
}

impl PieceForm {
    fn review(self, action: PieceAction) -> Result<ReviewRequest, StudioError> {
        let (status, text, expected_assets) = if matches!(action, PieceAction::Edit) {
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
            expected_revision: self.expected_revision,
            expected_target: self.expected_target,
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

/// One vocabulary for GET editors and submitted actions: an unknown action is not
/// a known operation with the wrong method, and must never reach a core operation.
#[derive(Clone, Copy)]
pub(super) enum PieceAction {
    Edit,
    Render,
    Approve,
    Publish,
    AcceptCost,
}
impl PieceAction {
    pub(super) fn parse(action: &str) -> Result<Self, Failure> {
        match action {
            "edit" => Ok(Self::Edit),
            "render" => Ok(Self::Render),
            "approve" => Ok(Self::Approve),
            "publish" => Ok(Self::Publish),
            "accept-cost" => Ok(Self::AcceptCost),
            _ => Err(Failure::NotFound),
        }
    }
}

/// Transport parsing produces the same core request types used by JSON and CLI callers.
enum SubmittedAction {
    Review(ReviewRequest),
    Render(RenderRequest),
    Publish(PublishRequest),
    AcceptCost(PieceForm),
}

fn submitted_action(action: PieceAction, form: PieceForm) -> Result<SubmittedAction, Failure> {
    let _ = &form.csrf; // Authenticated by the boundary before any adapter runs.
    match action {
        PieceAction::Edit | PieceAction::Approve => {
            Ok(SubmittedAction::Review(form.review(action)?))
        }
        PieceAction::Render => Ok(SubmittedAction::Render(form.rendering()?)),
        PieceAction::Publish => Ok(SubmittedAction::Publish(PublishRequest {
            expected_revision: form.expected_revision,
            accepted_cost_microusd: form.accepted_cost_microusd,
            dry_run: false,
        })),
        PieceAction::AcceptCost => Ok(SubmittedAction::AcceptCost(form)),
    }
}

pub(crate) async fn action(
    State(state): State<Arc<AppState>>,
    path: Result<Path<(String, String)>, PathRejection>,
    form: Result<Form<PieceForm>, FormRejection>,
) -> Result<Response, Failure> {
    let Path((id, action)) = path.map_err(|_| StudioError::PieceNotFound)?;
    let Form(form) = form.map_err(|_| StudioError::InvalidRequest)?;
    match submitted_action(PieceAction::parse(&action)?, form)? {
        SubmittedAction::Review(request) => review_action(&state, &id, request),
        SubmittedAction::Render(request) => render_action(&state, &id, request).await,
        SubmittedAction::Publish(request) => publish_action(&state, &id, request).await,
        SubmittedAction::AcceptCost(form) => {
            let piece = state.studio.show(&id)?;
            state.cost_consents.accept(&piece, &form)?;
            saved_piece_response(Ok(piece))
        }
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
    saved_piece_response(Ok(receipt.piece))
}
