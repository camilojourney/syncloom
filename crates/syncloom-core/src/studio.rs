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

/// Exactly one of `status` or `text`, plus the revision the reviewer saw.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub expected_revision: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublishRequest {
    pub expected_revision: String,
    #[serde(default)]
    pub dry_run: bool,
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
}

/// Surrounding whitespace removed and inner whitespace runs collapsed to one space.
pub fn normalize_thought(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Ids the studio generated: ASCII letters, digits, `-` and `_`, bounded length. Anything
/// else never reaches the file system.
fn valid_piece_id(piece_id: &str) -> bool {
    (1..=MAX_PIECE_ID_LEN).contains(&piece_id.len())
        && piece_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

/// What the reviewer asked for, once the body has been checked.
enum ReviewAction {
    Status(Status),
    Text(String),
}

fn review_action(request: &ReviewRequest) -> Result<ReviewAction, StudioError> {
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

fn parse_channels(names: &[String]) -> Result<Vec<Channel>, StudioError> {
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
        }
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
        let thought = normalize_thought(&request.text);
        if thought.chars().filter(|c| !c.is_whitespace()).count() < MIN_THOUGHT_CHARS {
            return Err(StudioError::ThoughtTooShort);
        }
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
                    platform: channel.platform().to_owned(),
                    revision,
                    status: Status::PendingReview,
                    review_decision_id: None,
                    publish_status: None,
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

    pub fn show(&self, piece_id: &str) -> Result<Piece, StudioError> {
        let lock = self.store.lock()?;
        self.load(&lock, piece_id)
    }

    pub fn review(&self, piece_id: &str, request: ReviewRequest) -> Result<Piece, StudioError> {
        let action = review_action(&request)?;
        let lock = self.store.lock()?;
        let mut piece = self.load_at(&lock, piece_id, &request.expected_revision)?;
        let event = match action {
            ReviewAction::Status(next) => {
                if !piece.status.can_move_to(next) {
                    return Err(StudioError::InvalidStatusTransition);
                }
                if next == piece.status {
                    return Ok(piece);
                }
                piece.status = next;
                piece.review_decision_id = (next == Status::Approved)
                    .then(|| review_decision_id(&piece.piece_id, &piece.revision));
                EventKind::PieceReviewed
            }
            ReviewAction::Text(text) => {
                if text == piece.text {
                    return Ok(piece);
                }
                piece.revision = revision(piece.channel, &piece.piece_id, &text);
                piece.fit = fit(piece.channel, &text);
                piece.text = text;
                piece.status = Status::PendingReview;
                piece.review_decision_id = None;
                // The new revision has no intent yet; the old one's stays in the outbox.
                piece.publish_status = None;
                EventKind::PieceEdited
            }
        };
        piece.updated_at = self.clock.now_rfc3339();
        lock.save_piece(&piece)?;
        self.record(&lock, event, &piece, Extra::default())?;
        Ok(piece)
    }

    pub async fn publish(
        &self,
        piece_id: &str,
        request: PublishRequest,
    ) -> Result<PublishResponse, StudioError> {
        let (key, piece, intent, created) = {
            let lock = self.store.lock()?;
            let mut piece = self.load_at(&lock, piece_id, &request.expected_revision)?;
            if request.dry_run {
                let payload = payload_for(&piece);
                return Ok(PublishResponse {
                    piece,
                    status: "dry_run".to_owned(),
                    request_id: None,
                    created: false,
                    payload,
                    post_id: None,
                    post_url: None,
                    attempts: 0,
                });
            }
            if piece.status != Status::Approved || piece.review_decision_id.is_none() {
                return Err(StudioError::ApprovalRequired);
            }
            let key = intent_key(OPERATION_PUBLISH, &piece.piece_id, &piece.revision);
            match lock.load_intent(&key)? {
                Some(existing) => (key, piece, existing, false),
                None => {
                    let intent = self.reserve_intent(&lock, &key, &mut piece)?;
                    (key, piece, intent, true)
                }
            }
        };
        let Some(platform) = self.delivery.get(&piece.platform).cloned() else {
            return Ok(response_from(piece, &intent, created));
        };
        match intent.status {
            IntentStatus::Posted => Ok(response_from(piece, &intent, false)),
            IntentStatus::Posting => self.settle_unknown(&key, piece, intent),
            IntentStatus::Unknown => Ok(response_from(piece, &intent, false)),
            _ => self.attempt(&key, piece, intent, created, platform).await,
        }
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
        };
        lock.save_intent(key, &intent)?;
        piece.publish_status = Some(IntentStatus::Contained.as_str().to_owned());
        piece.updated_at = now;
        lock.save_piece(piece)?;
        self.record(
            lock,
            EventKind::IntentReserved,
            piece,
            Extra {
                request_id: Some(intent.request_id.clone()),
                ..Extra::default()
            },
        )?;
        Ok(intent)
    }

    /// One platform attempt: `posting` written first, the lock released for the call, the
    /// outcome written after. A deadline overrun is `unknown`: the request may have landed.
    async fn attempt(
        &self,
        key: &str,
        mut piece: Piece,
        mut intent: DispatchIntent,
        created: bool,
        platform: Arc<dyn Platform>,
    ) -> Result<PublishResponse, StudioError> {
        {
            let lock = self.store.lock()?;
            let now = self.clock.now_rfc3339();
            intent.status = IntentStatus::Posting;
            intent.attempts += 1;
            intent.attempted_at = Some(now.clone());
            intent.finished_at = None;
            lock.save_intent(key, &intent)?;
            piece.publish_status = Some(IntentStatus::Posting.as_str().to_owned());
            piece.updated_at = now;
            lock.save_piece(&piece)?;
            self.record(
                &lock,
                EventKind::DispatchAttempted,
                &piece,
                Extra {
                    request_id: Some(intent.request_id.clone()),
                    ..Extra::default()
                },
            )?;
        }
        let request = PostRequest {
            request_id: intent.request_id.clone(),
            text: intent.payload.content.clone(),
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

    /// A `posting` record found later: its attempt's outcome is unknown and stays so.
    fn settle_unknown(
        &self,
        key: &str,
        piece: Piece,
        mut intent: DispatchIntent,
    ) -> Result<PublishResponse, StudioError> {
        intent.status = IntentStatus::Unknown;
        self.finish(key, &piece.piece_id, intent, false)
    }

    /// Write the outcome to the intent and, when the piece is still at this revision, to the
    /// piece; append `dispatch_finished`.
    fn finish(
        &self,
        key: &str,
        piece_id: &str,
        mut intent: DispatchIntent,
        created: bool,
    ) -> Result<PublishResponse, StudioError> {
        let lock = self.store.lock()?;
        let now = self.clock.now_rfc3339();
        intent.finished_at = Some(now.clone());
        lock.save_intent(key, &intent)?;
        let mut piece = self.load(&lock, piece_id)?;
        if piece.revision == intent.revision {
            piece.publish_status = Some(intent.status.as_str().to_owned());
            piece.updated_at = now;
            lock.save_piece(&piece)?;
        }
        self.record(
            &lock,
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
        };
        Ok(lock.append_event(&event)?)
    }
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
    }
}
