//! The studio use cases: create pieces from a thought, show, review, publish. Shared by the
//! HTTP API and the CLI so both return the same results and errors.
use crate::clock::Clock;
use crate::error::StudioError;
use crate::outbox::Payload;
use crate::piece::{Piece, Status};
use crate::provider::ModelProvider;
use crate::store::Store;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};

/// Fewest non-space characters a thought needs (retained from the old pipeline).
pub const MIN_THOUGHT_CHARS: usize = 8;
pub const PROVIDER_DEADLINE: Duration = Duration::from_secs(60);

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
    let _ = raw;
    String::new()
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

    pub async fn create_thought(
        &self,
        request: ThoughtRequest,
    ) -> Result<ThoughtResponse, StudioError> {
        let _ = (request, &self.store, &self.provider, &self.clock, &self.settings);
        Err(StudioError::Internal)
    }

    pub fn show(&self, piece_id: &str) -> Result<Piece, StudioError> {
        let _ = piece_id;
        Err(StudioError::Internal)
    }

    pub fn review(&self, piece_id: &str, request: ReviewRequest) -> Result<Piece, StudioError> {
        let _ = (piece_id, request);
        Err(StudioError::Internal)
    }

    pub fn publish(
        &self,
        piece_id: &str,
        request: PublishRequest,
    ) -> Result<PublishResponse, StudioError> {
        let _ = (piece_id, request);
        Err(StudioError::Internal)
    }
}
