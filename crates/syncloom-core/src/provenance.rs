//! Provenance events: ids, hashes and statuses, never text (`provenance-log`).
use crate::channel::Channel;
use crate::piece::Status;
use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: &str = "1.0";

/// Token usage as the provider reported it. Never a cost estimate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    PieceCreated,
    PieceReviewed,
    PieceEdited,
    IntentReserved,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub schema_version: String,
    pub event_id: String,
    pub event: EventKind,
    pub at: String,
    pub group_id: String,
    pub piece_id: String,
    pub channel: Channel,
    pub revision: String,
    pub status: Status,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}
