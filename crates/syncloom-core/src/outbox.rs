//! Publish intents: one per piece, revision and operation, contained until a platform
//! integration exists (`dispatch-outbox`).
use serde::{Deserialize, Serialize};

pub const OPERATION_PUBLISH: &str = "publish";
pub const STATUS_CONTAINED: &str = "contained";

/// What a platform integration would receive.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Payload {
    pub content: String,
    pub platforms: Vec<String>,
    pub style: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchIntent {
    pub request_id: String,
    pub operation: String,
    pub piece_id: String,
    pub revision: String,
    pub payload: Payload,
    pub status: String,
    pub created_at: String,
}

/// Hex SHA-256 of the compact sorted JSON of `operation`, `piece_id` and `revision`; the
/// intent's file name.
pub fn intent_key(operation: &str, piece_id: &str, revision: &str) -> String {
    let _ = (operation, piece_id, revision);
    String::new()
}

/// `dispatch-` plus the first 32 characters of [`intent_key`].
pub fn request_id(operation: &str, piece_id: &str, revision: &str) -> String {
    let _ = (operation, piece_id, revision);
    String::new()
}
