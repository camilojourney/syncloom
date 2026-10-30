//! A piece: one channel's draft of a thought, its revision and review state (`draft-review`).
use crate::channel::{Channel, Fit};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    PendingReview,
    Approved,
    Rejected,
}

impl Status {
    /// The retained transition table: pending_review to approved or rejected, approved to
    /// rejected, rejected to pending_review; the same status is always allowed and changes nothing.
    pub fn can_move_to(self, next: Status) -> bool {
        let _ = next;
        false
    }
}

/// Where the thought came from. Slice 1 knows text only.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    #[serde(rename = "type")]
    pub kind: String,
    pub raw_input: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Piece {
    pub piece_id: String,
    pub group_id: String,
    pub channel: Channel,
    pub platform: String,
    pub text: String,
    pub revision: String,
    pub status: Status,
    pub review_decision_id: Option<String>,
    pub publish_status: Option<String>,
    pub fit: Fit,
    pub source: Source,
    pub created_at: String,
    pub updated_at: String,
}

/// `thought-<first 12 of group id>-<channel>`, the shape the old queue used.
pub fn piece_id(group_id: &str, channel: Channel) -> String {
    let _ = (group_id, channel);
    String::new()
}

/// Lowercase hex SHA-256 of the compact JSON object with sorted keys `channel`, `piece_id`, `text`.
pub fn revision(channel: Channel, piece_id: &str, text: &str) -> String {
    let _ = (channel, piece_id, text);
    String::new()
}

/// `review-<piece id>-<first 16 of revision>`.
pub fn review_decision_id(piece_id: &str, revision: &str) -> String {
    let _ = (piece_id, revision);
    String::new()
}
