//! The studio use cases: create pieces from a thought, show, review, publish. Shared by the
//! HTTP API and the CLI so both return the same results and errors.
//!
//! Invariants: a request persists either all of its pieces or none (drafts are generated
//! before anything is written); every persisted state change appends one provenance event
//! after the record it describes; nothing is written while the model provider is running;
//! a dispatch writes the intent as `posting` before the platform request and its outcome after,
//! and holds the store lock during neither the provider nor the platform call; a `posted`
//! intent is never sent again and a `posting` or `unknown` one is never retried blindly.
use crate::channel::{Channel, fit};
use crate::clock::Clock;
use crate::error::StudioError;
use crate::linkedin::Linkedin;
use crate::outbox::{DispatchIntent, IntentStatus, OPERATION_PUBLISH, Payload, intent_key};
use crate::piece::{Piece, Source, Status, piece_id, review_decision_id, revision};
use crate::platform::{Platform, PostFailure, PostRequest};
use crate::provenance::{Event, EventKind, SCHEMA_VERSION, Usage};
use crate::provider::{Draft, DraftRequest, ModelProvider, ProviderError};
use crate::store::{Lock, Store};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

/// Fewest non-space characters a thought needs (retained from the old pipeline).
pub const MIN_THOUGHT_CHARS: usize = 8;
pub const PROVIDER_DEADLINE: Duration = Duration::from_secs(60);
/// One budget for a platform request; past it the outcome is unknown.
pub const PLATFORM_DEADLINE: Duration = Duration::from_secs(30);
/// How long a sign-in waits for the browser.
pub const SIGN_IN_DEADLINE: Duration = Duration::from_secs(300);
/// Piece ids are generated here and used as file names; anything else is "not found".
const MAX_PIECE_ID_LEN: usize = 128;

#[derive(Clone, Copy, Debug)]
pub struct Settings {
    /// One budget for the whole provider call.
    pub provider_deadline: Duration,
    pub platform_deadline: Duration,
    pub sign_in_deadline: Duration,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            provider_deadline: PROVIDER_DEADLINE,
            platform_deadline: PLATFORM_DEADLINE,
            sign_in_deadline: SIGN_IN_DEADLINE,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThoughtRequest {
    pub text: String,
    pub channels: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThoughtResponse {
    pub group_id: String,
    pub pieces: Vec<Piece>,
}

/// Exactly one of `status` or `text`, the reviewed revision, and visual approval's byte hashes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub expected_revision: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_assets: Option<crate::visual::AssetHashes>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublishRequest {
    pub expected_revision: String,
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepted_cost_microusd: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishResponse {
    pub piece: Piece,
    pub status: String,
    pub request_id: Option<String>,
    pub created: bool,
    pub payload: Payload,
    pub post_id: Option<String>,
    pub post_url: Option<String>,
    pub attempts: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_estimate: Option<crate::social::CostEstimate>,
}

/// A bounded discovery page; the cursor is the last returned piece id.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PiecePage {
    pub pieces: Vec<Piece>,
    pub next_after: Option<String>,
}

/// Owned browser review data from one store lock; no delivery or schema change.
#[derive(Debug)]
pub struct ReviewSnapshot {
    pub piece: Piece,
    pub publish_intent: Option<DispatchIntent>,
    /// Actual provenance time for the current approval, unavailable in history-less records.
    pub approved_at: Option<String>,
}

pub struct Studio {
    store: Store,
    provider: Arc<dyn ModelProvider>,
    clock: Arc<dyn Clock>,
    settings: Settings,
    /// Platforms granted delivery, by platform name. Absent means contained.
    delivery: HashMap<String, Arc<dyn Platform>>,
    /// The LinkedIn integration for sign-in and connection, granted or not.
    linkedin: Option<Arc<Linkedin>>,
    mastodon: Option<Arc<crate::mastodon::Mastodon>>,
    meta: Option<Arc<crate::meta::Meta>>,
    social: Option<Arc<crate::social::SocialPlatforms>>,
    renderer: crate::visual::RendererConfig,
    /// Serializes confirm-to-draft so one process cannot draft the same preview twice.
    confirm_gate: tokio::sync::Mutex<()>,
}

/// Surrounding whitespace removed and inner whitespace runs collapsed to one space.
pub fn normalize_thought(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn checked_thought(raw: &str) -> Result<String, StudioError> {
    let thought = normalize_thought(raw);
    if thought.chars().filter(|c| !c.is_whitespace()).count() < MIN_THOUGHT_CHARS {
        return Err(StudioError::ThoughtTooShort);
    }
    Ok(thought)
}

/// Ids the studio generated: ASCII letters, digits, `-` and `_`, bounded length. Anything
/// else never reaches the file system.
fn valid_piece_id(piece_id: &str) -> bool {
    (1..=MAX_PIECE_ID_LEN).contains(&piece_id.len())
        && piece_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte))
}

/// What the reviewer asked for, once the body has been checked.
enum ReviewAction {
    Status(Status),
    Text(String),
}

fn review_action(request: &ReviewRequest) -> Result<ReviewAction, StudioError> {
    if request.expected_assets.is_some() && request.status != Some(Status::Approved) {
        return Err(StudioError::InvalidRequest);
    }
    match (request.status, request.text.as_deref()) {
        (Some(status), None) => Ok(ReviewAction::Status(status)),
        (None, Some(text)) => {
            let text = text.trim();
            if text.is_empty() {
                return Err(StudioError::InvalidRequest);
            }
            Ok(ReviewAction::Text(text.to_owned()))
        }
        _ => Err(StudioError::InvalidRequest),
    }
}

pub(crate) fn parse_channels(names: &[String]) -> Result<Vec<Channel>, StudioError> {
    if names.is_empty() {
        return Err(StudioError::InvalidRequest);
    }
    let mut seen = HashSet::new();
    let mut channels = Vec::with_capacity(names.len());
    for name in names {
        let channel = Channel::parse(name).ok_or(StudioError::UnsupportedChannel)?;
        if !seen.insert(channel) {
            return Err(StudioError::InvalidRequest);
        }
        channels.push(channel);
    }
    Ok(channels)
}

fn payload_for(piece: &Piece) -> Payload {
    Payload {
        content: piece.text.clone(),
        platforms: vec![piece.channel.platform().to_owned()],
        style: "raw".to_owned(),
    }
}

impl Studio {
    pub fn new(
        store: Store,
        provider: Arc<dyn ModelProvider>,
        clock: Arc<dyn Clock>,
        settings: Settings,
    ) -> Self {
        Self {
            store,
            provider,
            clock,
            settings,
            delivery: HashMap::new(),
            linkedin: None,
            mastodon: None,
            meta: None,
            social: None,
            renderer: crate::visual::renderer_config(|name| std::env::var(name).ok()),
            confirm_gate: tokio::sync::Mutex::new(()),
        }
    }

    /// Accept a matching preview into pending drafts. A repeat returns the same pieces.
    pub async fn confirm_capture(
        &self,
        request: crate::capture::CaptureConfirmRequest,
    ) -> Result<crate::CaptureConfirmResponse, StudioError> {
        let _guard = self.confirm_gate.lock().await;
        let preview = request.checked_preview()?;
        if let Some(thought) = self.claim_capture(&preview)? {
            return Ok(crate::CaptureConfirmResponse {
                created: false,
                thought,
            });
        }
        let thought = match self
            .create_thought(ThoughtRequest {
                text: request.text,
                channels: request.channels,
            })
            .await
        {
            Ok(thought) => thought,
            Err(
                error @ (StudioError::ProviderError
                | StudioError::ProviderTimeout
                | StudioError::ProviderNotConfigured),
            ) => {
                // These documented failures persist nothing. Other failures keep the claim.
                self.store.lock()?.remove_capture(&preview.revision)?;
                return Err(error);
            }
            Err(error) => return Err(error),
        };
        self.store
            .lock()?
            .save_capture(&crate::capture::CaptureRecord {
                revision: preview.revision,
                channels: preview
                    .channels
                    .iter()
                    .map(|channel| channel.name().to_owned())
                    .collect(),
                group_id: thought.group_id.clone(),
                piece_ids: thought
                    .pieces
                    .iter()
                    .map(|piece| piece.piece_id.clone())
                    .collect(),
            })?;
        Ok(crate::CaptureConfirmResponse {
            created: true,
            thought,
        })
    }

    fn claim_capture(
        &self,
        preview: &crate::capture::Preview,
    ) -> Result<Option<ThoughtResponse>, StudioError> {
        let lock = self.store.lock()?;
        if let Some(record) = lock.load_capture(&preview.revision)? {
            if record.revision != preview.revision {
                return Err(StudioError::RevisionConflict);
            }
            return self.capture_pieces(&lock, record).map(Some);
        }
        // Reserve before provider I/O. A crash or partial save cannot authorize a second draft.
        lock.save_capture(&crate::capture::CaptureRecord {
            revision: preview.revision.clone(),
            channels: preview
                .channels
                .iter()
                .map(|c| c.name().to_owned())
                .collect(),
            group_id: String::new(),
            piece_ids: Vec::new(),
        })?;
        Ok(None)
    }

    pub fn capture_result(&self, revision: &str) -> Result<Option<ThoughtResponse>, StudioError> {
        if !crate::capture::valid_revision(revision) {
            return Ok(None);
        }
        let lock = self.store.lock()?;
        let Some(record) = lock.load_capture(revision)? else {
            return Ok(None);
        };
        if record.revision != revision {
            return Err(StudioError::RevisionConflict);
        }
        self.capture_pieces(&lock, record).map(Some)
    }

    fn capture_pieces(
        &self,
        lock: &Lock<'_>,
        record: crate::capture::CaptureRecord,
    ) -> Result<ThoughtResponse, StudioError> {
        if record.piece_ids.is_empty() {
            return Err(StudioError::RevisionConflict);
        }
        let mut pieces = Vec::with_capacity(record.piece_ids.len());
        for id in &record.piece_ids {
            let piece = match self.load(lock, id) {
                Ok(piece) => piece,
                Err(StudioError::PieceNotFound) => return Err(StudioError::RevisionConflict),
                Err(error) => return Err(error),
            };
            let preview = crate::capture::preview(&piece.source.raw_input, &record.channels)?;
            if piece.group_id != record.group_id || preview.revision != record.revision {
                return Err(StudioError::RevisionConflict);
            }
            pieces.push(piece);
        }
        let mut found: Vec<&str> = pieces.iter().map(|piece| piece.channel.name()).collect();
        let mut expected: Vec<&str> = record.channels.iter().map(String::as_str).collect();
        found.sort_unstable();
        expected.sort_unstable();
        if found != expected {
            return Err(StudioError::RevisionConflict);
        }
        Ok(ThoughtResponse {
            group_id: record.group_id,
            pieces,
        })
    }

    pub fn with_social(mut self, adapters: Arc<crate::social::SocialPlatforms>) -> Self {
        self.social = Some(adapters);
        self
    }
    /// Integration service API, not credential or configuration state.
    pub fn social(&self) -> Result<&crate::social::SocialPlatforms, StudioError> {
        self.social
            .as_deref()
            .ok_or(StudioError::PlatformNotConfigured)
    }

    pub fn with_renderer(mut self, config: crate::visual::RendererConfig) -> Self {
        self.renderer = config;
        self
    }

    /// Configure first, render without the lock, and accept bytes only at that exact revision.
    pub async fn render(
        &self,
        piece_id: &str,
        request: crate::visual::RenderRequest,
    ) -> Result<Piece, StudioError> {
        let content = request.content()?;
        let configured = self.configure_visual(piece_id, &request, &content)?;
        let rendered = match self
            .renderer
            .render(
                &content,
                matches!(
                    configured.channel,
                    Channel::FacebookPage | Channel::Instagram
                ),
            )
            .await
        {
            Ok(rendered) => rendered,
            Err(error) => {
                let lock = self.store.lock()?;
                self.record(
                    &lock,
                    EventKind::RenderFailed,
                    &configured,
                    Extra {
                        outcome: Some(error.code().to_owned()),
                        ..Extra::default()
                    },
                )?;
                return Err(error);
            }
        };
        self.save_rendered(piece_id, &configured, &content, rendered)
    }

    fn configure_visual(
        &self,
        piece_id: &str,
        request: &crate::visual::RenderRequest,
        content: &crate::visual::VisualContent,
    ) -> Result<Piece, StudioError> {
        let lock = self.store.lock()?;
        let mut piece = self.load_at(&lock, piece_id, &request.expected_revision)?;
        let changed = piece.visual.as_ref() != Some(content);
        piece.revision = crate::piece::revision_with_visual(
            piece.channel,
            &piece.piece_id,
            &piece.text,
            Some(content),
        );
        piece.visual = Some(content.clone());
        // Starting any rerender invalidates approval and old bytes, even if it fails.
        piece.invalidate_review();
        self.save_change(
            &lock,
            if changed {
                EventKind::VisualConfigured
            } else {
                EventKind::AssetsInvalidated
            },
            &mut piece,
            Extra::default(),
        )?;
        Ok(piece)
    }

    fn save_rendered(
        &self,
        piece_id: &str,
        configured: &Piece,
        content: &crate::visual::VisualContent,
        rendered: crate::visual::Rendered,
    ) -> Result<Piece, StudioError> {
        use crate::visual::{Assets, TEMPLATE_VERSION};
        let lock = self.store.lock()?;
        let mut piece = self.load_at(&lock, piece_id, &configured.revision)?;
        let image_sha256 = lock.save_asset(&rendered.image)?;
        let carousel_sha256 = lock.save_asset(&rendered.carousel)?;
        piece.invalidate_review();
        piece.assets = Some(Assets {
            revision: piece.revision.clone(),
            template_version: TEMPLATE_VERSION.into(),
            image_sha256,
            carousel_sha256,
            slides_sha256: rendered
                .slides
                .iter()
                .map(|bytes| lock.save_asset(bytes))
                .collect::<Result<_, _>>()?,
            pages: content.slides.len(),
            renderer_version: self.renderer.version.clone(),
            font_sha256: self.renderer.font_sha256.clone(),
        });
        self.save_change(
            &lock,
            EventKind::AssetsRendered,
            &mut piece,
            Extra::default(),
        )?;
        Ok(piece)
    }

    fn checked_assets<'a>(
        &self,
        lock: &Lock<'_>,
        piece: &'a Piece,
    ) -> Result<&'a crate::visual::Assets, StudioError> {
        let assets = piece.assets.as_ref().ok_or(StudioError::AssetsRequired)?;
        if assets.revision != piece.revision
            || assets.template_version != crate::visual::TEMPLATE_VERSION
        {
            return Err(StudioError::AssetsRequired);
        }
        lock.load_asset(&assets.image_sha256)?;
        lock.load_asset(&assets.carousel_sha256)?;
        if !assets.slides_sha256.is_empty() && assets.slides_sha256.len() != assets.pages {
            return Err(StudioError::AssetIntegrity);
        }
        for hash in &assets.slides_sha256 {
            lock.load_asset(hash)?;
        }
        Ok(assets)
    }

    pub fn view_asset(
        &self,
        piece_id: &str,
        kind: crate::visual::AssetKind,
    ) -> Result<(Vec<u8>, String), StudioError> {
        let lock = self.store.lock()?;
        let piece = self.load(&lock, piece_id)?;
        let assets = self.checked_assets(&lock, &piece)?;
        let hash = kind.hash(assets)?.to_owned();
        let bytes = lock.load_asset(&hash)?;
        self.record(&lock, EventKind::AssetsViewed, &piece, Extra::default())?;
        Ok((bytes, hash))
    }

    /// Grant delivery to one platform: its intents are dispatched instead of contained.
    pub fn with_delivery(mut self, platform: &str, adapter: Arc<dyn Platform>) -> Self {
        self.delivery.insert(platform.to_owned(), adapter);
        self
    }

    /// Attach the LinkedIn integration for sign-in, connection and sign-out.
    pub fn with_linkedin(mut self, linkedin: Arc<Linkedin>) -> Self {
        self.linkedin = Some(linkedin);
        self
    }

    fn linkedin(&self) -> Result<&Linkedin, StudioError> {
        self.linkedin
            .as_deref()
            .ok_or(StudioError::LinkedinNotConfigured)
    }

    pub fn linkedin_sign_in_begin(&self) -> Result<crate::linkedin::SignInStart, StudioError> {
        self.linkedin()?.sign_in_begin()
    }
    pub async fn linkedin_sign_in_wait(&self) -> Result<crate::linkedin::Connection, StudioError> {
        self.linkedin()?.sign_in_wait().await
    }
    pub fn linkedin_connection(&self) -> Result<crate::linkedin::Connection, StudioError> {
        self.linkedin()?.connection()
    }
    pub fn linkedin_sign_out(&self) -> Result<crate::linkedin::Connection, StudioError> {
        self.linkedin()?.sign_out()
    }

    pub fn with_meta(mut self, adapter: Arc<crate::meta::Meta>) -> Self {
        self.meta = Some(adapter);
        self
    }
    pub fn meta(&self) -> Result<&crate::meta::Meta, StudioError> {
        self.meta.as_deref().ok_or(StudioError::MetaNotConfigured)
    }

    pub fn with_mastodon(mut self, adapter: Arc<crate::mastodon::Mastodon>) -> Self {
        self.mastodon = Some(adapter);
        self
    }
    fn mastodon(&self) -> Result<&crate::mastodon::Mastodon, StudioError> {
        self.mastodon
            .as_deref()
            .ok_or(StudioError::MastodonNotConfigured)
    }
    pub async fn mastodon_sign_in_begin(&self) -> Result<crate::oauth::SignInStart, StudioError> {
        self.mastodon()?.sign_in_begin().await
    }
    pub async fn mastodon_sign_in_wait(&self) -> Result<crate::oauth::Connection, StudioError> {
        self.mastodon()?.sign_in_wait().await
    }
    pub fn mastodon_connection(&self) -> Result<crate::oauth::Connection, StudioError> {
        self.mastodon()?.connection()
    }
    pub fn mastodon_sign_out(&self) -> Result<crate::oauth::Connection, StudioError> {
        self.mastodon()?.sign_out()
    }

    pub fn export(
        &self,
        piece_id: &str,
        request: crate::manual_export::ExportRequest,
    ) -> Result<crate::manual_export::ExportResponse, StudioError> {
        use crate::manual_export::{ExportResponse, Target};
        let target = Target::parse(&request.target)?;
        let lock = self.store.lock()?;
        let mut piece = self.load_at(&lock, piece_id, &request.expected_revision)?;
        self.require_approval(&lock, &piece)?;
        let operation = format!("export:{}", target.name());
        let key = intent_key(&operation, &piece.piece_id, &piece.revision);
        let existing = lock.load_intent(&key)?;
        let files = self.bundle_files(&lock, &piece, target)?;
        let bundle = lock.save_export(&key, &files, existing.is_some())?;
        let request_id = crate::outbox::request_id(&operation, &piece.piece_id, &piece.revision);
        if existing.is_none() {
            let now = self.clock.now_rfc3339();
            let intent = DispatchIntent {
                request_id: request_id.clone(),
                operation,
                piece_id: piece.piece_id.clone(),
                revision: piece.revision.clone(),
                payload: payload_for(&piece),
                status: IntentStatus::Exported,
                created_at: now.clone(),
                attempts: 0,
                attempted_at: None,
                finished_at: Some(now),
                post_id: None,
                post_url: None,
                cost_estimate: None,
            };
            self.record_export(&lock, &key, &mut piece, &intent)?;
        }
        Ok(ExportResponse {
            status: "exported".into(),
            request_id,
            bundle: bundle.to_string_lossy().into_owned(),
            created: existing.is_none(),
        })
    }

    /// One provider call under the deadline; the studio, not the adapter, decides that
    /// empty text is a failure.
    async fn draft(&self, channel: Channel, thought: &str) -> Result<Draft, StudioError> {
        let request = DraftRequest {
            channel,
            thought: thought.to_owned(),
        };
        let result = tokio::time::timeout(
            self.settings.provider_deadline,
            self.provider.draft(request),
        )
        .await;
        let draft = match result {
            Err(_) => return Err(StudioError::ProviderTimeout),
            Ok(Err(ProviderError::Unconfigured)) => return Err(StudioError::ProviderNotConfigured),
            Ok(Err(ProviderError::Rejected)) => return Err(StudioError::ProviderError),
            Ok(Ok(draft)) => draft,
        };
        let text = draft.text.trim().to_owned();
        if text.is_empty() {
            return Err(StudioError::ProviderError);
        }
        Ok(Draft {
            text,
            usage: draft.usage,
        })
    }

    pub async fn create_thought(
        &self,
        request: ThoughtRequest,
    ) -> Result<ThoughtResponse, StudioError> {
        let channels = parse_channels(&request.channels)?;
        let thought = checked_thought(&request.text)?;
        // All drafts first, then all writes: a provider failure persists nothing.
        let mut drafts = Vec::with_capacity(channels.len());
        for channel in &channels {
            drafts.push((*channel, self.draft(*channel, &thought).await?));
        }
        let group_id = uuid::Uuid::new_v4().simple().to_string();
        let now = self.clock.now_rfc3339();
        let pieces: Vec<Piece> = drafts
            .iter()
            .map(|(channel, draft)| {
                let piece_id = piece_id(&group_id, *channel);
                let revision = revision(*channel, &piece_id, &draft.text);
                Piece {
                    piece_id,
                    group_id: group_id.clone(),
                    channel: *channel,
                    routing: Some(crate::routing::analyze_thought(&thought, *channel)),
                    platform: channel.platform().to_owned(),
                    revision,
                    status: Status::PendingReview,
                    review_decision_id: None,
                    publish_status: None,
                    visual: None,
                    assets: None,
                    fit: fit(*channel, &draft.text),
                    text: draft.text.clone(),
                    source: Source {
                        kind: "text".to_owned(),
                        raw_input: request.text.clone(),
                    },
                    created_at: now.clone(),
                    updated_at: now.clone(),
                }
            })
            .collect();
        let lock = self.store.lock()?;
        for (piece, (_, draft)) in pieces.iter().zip(&drafts) {
            lock.save_piece(piece)?;
            self.record(
                &lock,
                EventKind::PieceCreated,
                piece,
                Extra {
                    usage: draft.usage,
                    ..Extra::default()
                },
            )?;
        }
        Ok(ThoughtResponse { group_id, pieces })
    }

    pub fn list(&self, after: Option<&str>) -> Result<PiecePage, StudioError> {
        const PAGE_SIZE: usize = 20;
        if after.is_some_and(|id| !valid_piece_id(id)) {
            return Err(StudioError::InvalidRequest);
        }
        let lock = self.store.lock()?;
        let mut pieces = lock.pieces_after(after, PAGE_SIZE + 1)?;
        if pieces.iter().any(|piece| !valid_piece_id(&piece.piece_id)) {
            return Err(StudioError::Internal);
        }
        let next_after = if pieces.len() > PAGE_SIZE {
            pieces.pop();
            pieces.last().map(|piece| piece.piece_id.clone())
        } else {
            None
        };
        Ok(PiecePage { pieces, next_after })
    }

    pub fn show(&self, piece_id: &str) -> Result<Piece, StudioError> {
        let lock = self.store.lock()?;
        self.load(&lock, piece_id)
    }

    /// Read one consistent piece/current-publish snapshot without dispatching or changing a
    /// persisted Posting marker. Exports have separate keys and cannot hide this intent.
    pub fn inspect_publish(&self, piece_id: &str) -> Result<ReviewSnapshot, StudioError> {
        let lock = self.store.lock()?;
        let piece = self.load(&lock, piece_id)?;
        let key = intent_key(OPERATION_PUBLISH, &piece.piece_id, &piece.revision);
        let publish_intent = lock.load_intent(&key)?;
        let approved_at = if piece.status == Status::Approved && piece.review_decision_id.is_some()
        {
            lock.approval_at(&piece.piece_id, &piece.revision)?
        } else {
            None
        };
        Ok(ReviewSnapshot {
            piece,
            publish_intent,
            approved_at,
        })
    }

    pub fn review(&self, piece_id: &str, request: ReviewRequest) -> Result<Piece, StudioError> {
        let action = review_action(&request)?;
        let lock = self.store.lock()?;
        let mut piece = self.load_at(&lock, piece_id, &request.expected_revision)?;
        let event = match action {
            ReviewAction::Status(next) => {
                if !self.review_status(&lock, &mut piece, next, request.expected_assets.as_ref())? {
                    return Ok(piece);
                }
                EventKind::PieceReviewed
            }
            ReviewAction::Text(text) => {
                if text == piece.text {
                    return Ok(piece);
                }
                piece.revision = crate::piece::revision_with_visual(
                    piece.channel,
                    &piece.piece_id,
                    &text,
                    piece.visual.as_ref(),
                );
                piece.fit = fit(piece.channel, &text);
                piece.text = text;
                // The old revision's intent stays in the outbox, but never authorizes this one.
                piece.invalidate_review();
                EventKind::PieceEdited
            }
        };
        self.save_change(&lock, event, &mut piece, Extra::default())?;
        Ok(piece)
    }

    fn require_visual_assets(&self, lock: &Lock<'_>, piece: &Piece) -> Result<(), StudioError> {
        if piece.visual.is_some() {
            self.checked_assets(lock, piece)?;
        }
        Ok(())
    }

    fn require_approval(&self, lock: &Lock<'_>, piece: &Piece) -> Result<(), StudioError> {
        if piece.status != Status::Approved || piece.review_decision_id.is_none() {
            return Err(StudioError::ApprovalRequired);
        }
        self.require_visual_assets(lock, piece)
    }

    fn approve_bytes(
        &self,
        lock: &Lock<'_>,
        piece: &Piece,
        expected: Option<&crate::visual::AssetHashes>,
    ) -> Result<(), StudioError> {
        if piece.visual.is_some() {
            let assets = self.checked_assets(lock, piece)?;
            let expected = expected.ok_or(StudioError::AssetsRequired)?;
            if expected.image_sha256 != assets.image_sha256
                || expected.carousel_sha256 != assets.carousel_sha256
                || expected.slides_sha256 != assets.slides_sha256
            {
                return Err(StudioError::AssetConflict);
            }
        } else if expected.is_some() {
            return Err(StudioError::InvalidRequest);
        }
        Ok(())
    }

    fn review_status(
        &self,
        lock: &Lock<'_>,
        piece: &mut Piece,
        next: Status,
        expected_assets: Option<&crate::visual::AssetHashes>,
    ) -> Result<bool, StudioError> {
        if next == Status::Approved {
            self.approve_bytes(lock, piece, expected_assets)?;
        }
        if !piece.status.can_move_to(next) {
            return Err(StudioError::InvalidStatusTransition);
        }
        if next == piece.status {
            return Ok(false);
        }
        piece.status = next;
        piece.review_decision_id = (next == Status::Approved)
            .then(|| review_decision_id(&piece.piece_id, &piece.revision));
        Ok(true)
    }

    pub async fn publish(
        &self,
        piece_id: &str,
        request: PublishRequest,
    ) -> Result<PublishResponse, StudioError> {
        let (key, piece, intent, created) = match self.reserve_or_load(piece_id, request)? {
            Reserved::DryRun(response) => return Ok(response),
            Reserved::Intent {
                key,
                piece,
                intent,
                created,
            } => (key, piece, intent, created),
        };
        let Some(platform) = self
            .delivery
            .get(&piece.platform)
            .filter(|_| {
                piece.channel != Channel::ManualExport
                    && (piece.visual.is_none()
                        || matches!(
                            piece.channel,
                            Channel::Mastodon
                                | Channel::FacebookPage
                                | Channel::Instagram
                                | Channel::Bluesky
                                | Channel::X
                                | Channel::Threads
                        ))
            })
            .cloned()
        else {
            return Ok(response_from(piece, &intent, created));
        };
        self.attempt(&key, piece, intent, created, platform).await
    }

    /// Under the lock: the revision check, the dry-run answer, the approval check, and the
    /// intent, loaded when it exists or reserved as contained when it does not.
    fn reserve_or_load(
        &self,
        piece_id: &str,
        request: PublishRequest,
    ) -> Result<Reserved, StudioError> {
        let lock = self.store.lock()?;
        let mut piece = self.load_at(&lock, piece_id, &request.expected_revision)?;
        if request.dry_run {
            let payload = payload_for(&piece);
            let cost_estimate = cost_for(&piece);
            return Ok(Reserved::DryRun(PublishResponse {
                piece,
                status: "dry_run".to_owned(),
                request_id: None,
                created: false,
                payload,
                post_id: None,
                post_url: None,
                attempts: 0,
                cost_estimate,
            }));
        }
        self.require_approval(&lock, &piece)?;
        let key = intent_key(OPERATION_PUBLISH, &piece.piece_id, &piece.revision);
        let existing = lock.load_intent(&key)?;
        self.require_cost_confirmation(&piece, existing.as_ref(), request.accepted_cost_microusd)?;
        let (intent, created) = match existing {
            Some(existing) => (existing, false),
            None => (self.reserve_intent(&lock, &key, &mut piece)?, true),
        };
        Ok(Reserved::Intent {
            key,
            piece,
            intent,
            created,
        })
    }

    fn require_cost_confirmation(
        &self,
        piece: &Piece,
        existing: Option<&DispatchIntent>,
        accepted: Option<u64>,
    ) -> Result<(), StudioError> {
        if self.delivery.contains_key(&piece.platform)
            && existing.is_none_or(|i| i.status.allows_new_attempt())
            && let Some(cost) = cost_for(piece)
            && accepted != Some(cost.microusd)
        {
            return Err(StudioError::CostConfirmationRequired);
        }
        Ok(())
    }

    /// Record the one intent for this piece revision as contained. Whether it is dispatched
    /// is decided by the delivery grant, afterwards.
    fn reserve_intent(
        &self,
        lock: &Lock<'_>,
        key: &str,
        piece: &mut Piece,
    ) -> Result<DispatchIntent, StudioError> {
        let now = self.clock.now_rfc3339();
        let intent = DispatchIntent {
            request_id: crate::outbox::request_id(
                OPERATION_PUBLISH,
                &piece.piece_id,
                &piece.revision,
            ),
            operation: OPERATION_PUBLISH.to_owned(),
            piece_id: piece.piece_id.clone(),
            revision: piece.revision.clone(),
            payload: payload_for(piece),
            status: IntentStatus::Contained,
            created_at: now.clone(),
            attempts: 0,
            attempted_at: None,
            finished_at: None,
            post_id: None,
            post_url: None,
            cost_estimate: cost_for(piece),
        };
        lock.save_intent(key, &intent)?;
        piece.publish_status = Some(IntentStatus::Contained.as_str().to_owned());
        piece.updated_at = now;
        lock.save_piece(piece)?;
        self.record_intent(lock, EventKind::IntentReserved, piece, &intent)?;
        Ok(intent)
    }

    fn record_export(
        &self,
        lock: &Lock<'_>,
        key: &str,
        piece: &mut Piece,
        intent: &DispatchIntent,
    ) -> Result<(), StudioError> {
        lock.save_intent(key, intent)?;
        piece.publish_status = Some("exported".into());
        self.save_change(
            lock,
            EventKind::PieceExported,
            piece,
            Extra {
                request_id: Some(intent.request_id.clone()),
                outcome: Some("exported".into()),
                ..Extra::default()
            },
        )
    }

    fn post_image(
        &self,
        lock: &Lock<'_>,
        piece: &Piece,
    ) -> Result<Option<crate::platform::PostImage>, StudioError> {
        let image = if piece.visual.is_some() {
            let assets = self.checked_assets(lock, piece)?;
            Some(crate::platform::PostImage {
                bytes: lock.load_asset(&assets.image_sha256)?,
                alt_text: piece
                    .visual
                    .as_ref()
                    .map(|v| v.image_alt_text())
                    .unwrap_or("")
                    .to_owned(),
            })
        } else {
            None
        };
        Ok(image)
    }

    fn post_slides(
        &self,
        lock: &Lock<'_>,
        piece: &Piece,
    ) -> Result<Vec<crate::platform::PostImage>, StudioError> {
        if piece.channel != Channel::Instagram || piece.visual.is_none() {
            return Ok(Vec::new());
        }
        let assets = self.checked_assets(lock, piece)?;
        let visual = piece.visual.as_ref().ok_or(StudioError::AssetsRequired)?;
        assets
            .slides_sha256
            .iter()
            .zip(&visual.slides)
            .map(|(hash, alt)| {
                Ok(crate::platform::PostImage {
                    bytes: lock.load_asset(hash)?,
                    alt_text: alt.clone(),
                })
            })
            .collect()
    }

    /// Persist the attempt marker before releasing the caller's claim lock.
    fn start_attempt(
        &self,
        lock: &Lock<'_>,
        key: &str,
        piece: &mut Piece,
        intent: &mut DispatchIntent,
    ) -> Result<(), StudioError> {
        let now = self.clock.now_rfc3339();
        intent.cost_estimate = cost_for(piece);
        intent.status = IntentStatus::Posting;
        intent.attempts += 1;
        intent.attempted_at = Some(now.clone());
        intent.finished_at = None;
        lock.save_intent(key, intent)?;
        piece.publish_status = Some(IntentStatus::Posting.as_str().to_owned());
        piece.updated_at = now;
        lock.save_piece(piece)?;
        self.record_intent(lock, EventKind::DispatchAttempted, piece, intent)?;
        Ok(())
    }

    fn bundle_files(
        &self,
        lock: &Lock<'_>,
        piece: &Piece,
        target: crate::manual_export::Target,
    ) -> Result<std::collections::BTreeMap<String, Vec<u8>>, StudioError> {
        let (image, carousel) = if let Some(assets) = &piece.assets {
            (
                Some(lock.load_asset(&assets.image_sha256)?),
                Some(lock.load_asset(&assets.carousel_sha256)?),
            )
        } else {
            (None, None)
        };
        Ok(crate::manual_export::files(piece, target, image, carousel))
    }

    fn post_request(
        &self,
        lock: &Lock<'_>,
        piece: &Piece,
        intent: &DispatchIntent,
    ) -> Result<PostRequest, StudioError> {
        Ok(PostRequest {
            request_id: intent.request_id.clone(),
            text: intent.payload.content.clone(),
            image: self.post_image(lock, piece)?,
            slides: self.post_slides(lock, piece)?,
        })
    }

    /// Atomically recheck approval and claim the durable intent before any network operation.
    fn claim_attempt(
        &self,
        key: &str,
        piece: &Piece,
        intent: &DispatchIntent,
    ) -> Result<Claim, StudioError> {
        let lock = self.store.lock()?;
        let mut piece = self.load_at(&lock, &piece.piece_id, &intent.revision)?;
        self.require_approval(&lock, &piece)?;
        let mut intent = lock.load_intent(key)?.ok_or(StudioError::Internal)?;
        if !intent.status.allows_new_attempt() {
            if intent.status == IntentStatus::Posting {
                intent.status = IntentStatus::Unknown;
                return self
                    .finish_locked(&lock, key, &piece.piece_id, intent, false)
                    .map(|response| Claim::Finished(Box::new(response)));
            }
            return Ok(Claim::Finished(Box::new(response_from(
                piece, &intent, false,
            ))));
        }
        let request = self.post_request(&lock, &piece, &intent)?;
        self.start_attempt(&lock, key, &mut piece, &mut intent)?;
        Ok(Claim::Send {
            intent: Box::new(intent),
            request,
        })
    }

    /// One platform attempt: `posting` written first, the lock released for the call, the
    /// outcome written after. A deadline overrun is `unknown`: the request may have landed.
    async fn attempt(
        &self,
        key: &str,
        piece: Piece,
        intent: DispatchIntent,
        created: bool,
        platform: Arc<dyn Platform>,
    ) -> Result<PublishResponse, StudioError> {
        let (mut intent, request) = match self.claim_attempt(key, &piece, &intent)? {
            Claim::Finished(response) => return Ok(*response),
            Claim::Send { intent, request } => (*intent, request),
        };
        let result =
            tokio::time::timeout(self.settings.platform_deadline, platform.publish(request)).await;
        let (status, posted) = match result {
            Ok(Ok(posted)) => (IntentStatus::Posted, Some(posted)),
            Ok(Err(failure)) => (status_of(failure), None),
            Err(_) => (IntentStatus::Unknown, None),
        };
        intent.status = status;
        intent.post_id = posted.as_ref().map(|p| p.post_id.clone());
        intent.post_url = posted.as_ref().map(|p| p.url.clone());
        self.finish(key, &piece.piece_id, intent, created)
    }

    /// Write the outcome to the intent and, when the piece is still at this revision, to the
    /// piece; append `dispatch_finished`.
    fn finish(
        &self,
        key: &str,
        piece_id: &str,
        intent: DispatchIntent,
        created: bool,
    ) -> Result<PublishResponse, StudioError> {
        let lock = self.store.lock()?;
        self.finish_locked(&lock, key, piece_id, intent, created)
    }

    fn finish_locked(
        &self,
        lock: &Lock<'_>,
        key: &str,
        piece_id: &str,
        mut intent: DispatchIntent,
        created: bool,
    ) -> Result<PublishResponse, StudioError> {
        let now = self.clock.now_rfc3339();
        intent.finished_at = Some(now.clone());
        lock.save_intent(key, &intent)?;
        let mut piece = self.load(lock, piece_id)?;
        if piece.revision == intent.revision {
            piece.publish_status = Some(intent.status.as_str().to_owned());
            piece.updated_at = now;
            lock.save_piece(&piece)?;
        }
        self.record(
            lock,
            EventKind::DispatchFinished,
            &piece,
            Extra {
                request_id: Some(intent.request_id.clone()),
                outcome: Some(intent.status.as_str().to_owned()),
                post_id: intent.post_id.clone(),
                usage: None,
            },
        )?;
        Ok(response_from(piece, &intent, created))
    }

    /// The piece at the revision the caller saw; any other revision is a conflict.
    fn load_at(
        &self,
        lock: &Lock<'_>,
        piece_id: &str,
        expected_revision: &str,
    ) -> Result<Piece, StudioError> {
        let piece = self.load(lock, piece_id)?;
        if expected_revision != piece.revision {
            return Err(StudioError::RevisionConflict);
        }
        Ok(piece)
    }

    fn load(&self, lock: &Lock<'_>, piece_id: &str) -> Result<Piece, StudioError> {
        if !valid_piece_id(piece_id) {
            return Err(StudioError::PieceNotFound);
        }
        lock.load_piece(piece_id)?.ok_or(StudioError::PieceNotFound)
    }

    /// An event about an intent: the piece's ids plus the request id.
    fn record_intent(
        &self,
        lock: &Lock<'_>,
        kind: EventKind,
        piece: &Piece,
        intent: &DispatchIntent,
    ) -> Result<(), StudioError> {
        self.record(
            lock,
            kind,
            piece,
            Extra {
                request_id: Some(intent.request_id.clone()),
                ..Extra::default()
            },
        )
    }

    /// Persist a piece change before its event; the event never describes an unwritten record.
    fn save_change(
        &self,
        lock: &Lock<'_>,
        kind: EventKind,
        piece: &mut Piece,
        extra: Extra,
    ) -> Result<(), StudioError> {
        piece.updated_at = self.clock.now_rfc3339();
        lock.save_piece(piece)?;
        self.record(lock, kind, piece, extra)
    }

    /// One event per state change, holding ids, hashes, statuses and outcomes only.
    fn record(
        &self,
        lock: &Lock<'_>,
        kind: EventKind,
        piece: &Piece,
        extra: Extra,
    ) -> Result<(), StudioError> {
        let event = Event {
            schema_version: SCHEMA_VERSION.to_owned(),
            event_id: uuid::Uuid::new_v4().to_string(),
            event: kind,
            at: self.clock.now_rfc3339(),
            group_id: piece.group_id.clone(),
            piece_id: piece.piece_id.clone(),
            channel: piece.channel,
            revision: piece.revision.clone(),
            status: piece.status,
            usage: extra.usage,
            request_id: extra.request_id,
            outcome: extra.outcome,
            post_id: extra.post_id,
            image_sha256: piece.assets.as_ref().map(|a| a.image_sha256.clone()),
            carousel_sha256: piece.assets.as_ref().map(|a| a.carousel_sha256.clone()),
        };
        Ok(lock.append_event(&event)?)
    }
}

enum Claim {
    Finished(Box<PublishResponse>),
    Send {
        intent: Box<DispatchIntent>,
        request: PostRequest,
    },
}

/// What the locked part of a publish produced.
enum Reserved {
    DryRun(PublishResponse),
    Intent {
        key: String,
        piece: Piece,
        intent: DispatchIntent,
        created: bool,
    },
}

/// The optional fields of an event.
#[derive(Default)]
struct Extra {
    usage: Option<Usage>,
    request_id: Option<String>,
    outcome: Option<String>,
    post_id: Option<String>,
}

/// The intent's status as the platform failure named it.
fn status_of(failure: PostFailure) -> IntentStatus {
    match failure {
        PostFailure::ConsentRequired => IntentStatus::ConsentRequired,
        PostFailure::Refused => IntentStatus::Refused,
        PostFailure::RateLimited => IntentStatus::RateLimited,
        PostFailure::Unreachable => IntentStatus::Unreachable,
        PostFailure::Unknown => IntentStatus::Unknown,
    }
}

fn response_from(piece: Piece, intent: &DispatchIntent, created: bool) -> PublishResponse {
    PublishResponse {
        piece,
        status: intent.status.as_str().to_owned(),
        request_id: Some(intent.request_id.clone()),
        created,
        payload: intent.payload.clone(),
        post_id: intent.post_id.clone(),
        post_url: intent.post_url.clone(),
        attempts: intent.attempts,
        cost_estimate: intent.cost_estimate.clone(),
    }
}

fn cost_for(piece: &Piece) -> Option<crate::social::CostEstimate> {
    (piece.channel == Channel::X)
        .then(|| crate::social::estimate(&piece.text, piece.visual.is_some()))
}
