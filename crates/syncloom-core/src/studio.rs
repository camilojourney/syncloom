//! The studio use cases: create pieces from a thought, show, review, publish. Shared by the
//! HTTP API and the CLI so both return the same results and errors.
//!
//! Invariants: a request persists either all of its pieces or none (drafts are generated
//! before anything is written); every persisted state change appends one provenance event
//! after the record it describes; nothing is written while the model provider is running.
use crate::channel::{Channel, fit};
use crate::clock::Clock;
use crate::error::StudioError;
use crate::outbox::{DispatchIntent, OPERATION_PUBLISH, Payload, STATUS_CONTAINED, intent_key};
use crate::piece::{Piece, Source, Status, piece_id, review_decision_id, revision};
use crate::provenance::{Event, EventKind, SCHEMA_VERSION, Usage};
use crate::provider::{Draft, DraftRequest, ModelProvider, ProviderError};
use crate::store::{Lock, Store};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc, time::Duration};

/// Fewest non-space characters a thought needs (retained from the old pipeline).
pub const MIN_THOUGHT_CHARS: usize = 8;
pub const PROVIDER_DEADLINE: Duration = Duration::from_secs(60);
/// Piece ids are generated here and used as file names; anything else is "not found".
const MAX_PIECE_ID_LEN: usize = 128;

#[derive(Clone, Copy, Debug)]
pub struct Settings {
    /// One budget for the whole provider call.
    pub provider_deadline: Duration,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            provider_deadline: PROVIDER_DEADLINE,
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
}

pub struct Studio {
    store: Store,
    provider: Arc<dyn ModelProvider>,
    clock: Arc<dyn Clock>,
    settings: Settings,
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

fn publish_response(
    piece: Piece,
    status: String,
    request_id: Option<String>,
    created: bool,
    payload: Payload,
) -> PublishResponse {
    PublishResponse {
        piece,
        status,
        request_id,
        created,
        payload,
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
        }
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
            self.record(&lock, EventKind::PieceCreated, piece, draft.usage, None)?;
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
        self.record(&lock, event, &piece, None, None)?;
        Ok(piece)
    }

    pub fn publish(
        &self,
        piece_id: &str,
        request: PublishRequest,
    ) -> Result<PublishResponse, StudioError> {
        let lock = self.store.lock()?;
        let mut piece = self.load_at(&lock, piece_id, &request.expected_revision)?;
        if request.dry_run {
            let payload = payload_for(&piece);
            return Ok(publish_response(
                piece,
                "dry_run".to_owned(),
                None,
                false,
                payload,
            ));
        }
        if piece.status != Status::Approved || piece.review_decision_id.is_none() {
            return Err(StudioError::ApprovalRequired);
        }
        let key = intent_key(OPERATION_PUBLISH, &piece.piece_id, &piece.revision);
        match lock.load_intent(&key)? {
            Some(existing) => Ok(publish_response(
                piece,
                existing.status,
                Some(existing.request_id),
                false,
                existing.payload,
            )),
            None => self.reserve_intent(&lock, &key, &mut piece),
        }
    }

    /// Record the one intent for this piece revision. No platform integration exists, so
    /// every intent is contained and nothing leaves the machine.
    fn reserve_intent(
        &self,
        lock: &Lock<'_>,
        key: &str,
        piece: &mut Piece,
    ) -> Result<PublishResponse, StudioError> {
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
            status: STATUS_CONTAINED.to_owned(),
            created_at: now.clone(),
        };
        lock.save_intent(key, &intent)?;
        piece.publish_status = Some(STATUS_CONTAINED.to_owned());
        piece.updated_at = now;
        lock.save_piece(piece)?;
        self.record(
            lock,
            EventKind::IntentReserved,
            piece,
            None,
            Some(intent.request_id.clone()),
        )?;
        Ok(publish_response(
            piece.clone(),
            intent.status,
            Some(intent.request_id),
            true,
            intent.payload,
        ))
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

    /// One event per state change, holding ids, hashes and statuses only.
    fn record(
        &self,
        lock: &Lock<'_>,
        kind: EventKind,
        piece: &Piece,
        usage: Option<Usage>,
        request_id: Option<String>,
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
            usage,
            request_id,
        };
        Ok(lock.append_event(&event)?)
    }
}
