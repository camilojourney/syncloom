//! A piece: one channel's draft of a thought, its revision and review state (`draft-review`).
use crate::channel::{Channel, Fit};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
        use Status::*;
        self == next
            || matches!(
                (self, next),
                (PendingReview, Approved)
                    | (PendingReview, Rejected)
                    | (Approved, Rejected)
                    | (Rejected, PendingReview)
            )
    }
}

/// Where the thought came from. Historical text records need no retrieval metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    #[serde(rename = "type")]
    pub kind: String,
    pub raw_input: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retrieval: Option<crate::source::Retrieval>,
}

impl Source {
    pub fn text(&self) -> &str {
        self.retrieval
            .as_ref()
            .map_or(&self.raw_input, |r| &r.extracted_text)
    }
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
    /// Absent on historical or offline content-only approvals; never backfilled implicitly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_target: Option<String>,
    pub publish_status: Option<String>,
    pub fit: Fit,
    pub source: Source,
    /// Source-derived review target. Older records predate language routing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub routing: Option<crate::routing::ThoughtContract>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visual: Option<crate::visual::VisualContent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assets: Option<crate::visual::Assets>,
}

impl Piece {
    /// Every content change or rerender voids approval and the previous visual bytes.
    pub(crate) fn invalidate_review(&mut self) {
        self.status = Status::PendingReview;
        self.review_decision_id = None;
        self.approval_target = None;
        self.publish_status = None;
        self.assets = None;
    }
}

/// `thought-<first 12 of group id>-<channel>`, the shape the old queue used.
pub fn piece_id(group_id: &str, channel: Channel) -> String {
    let prefix: String = group_id.chars().take(12).collect();
    format!("thought-{prefix}-{}", channel.name())
}

/// Lowercase hex SHA-256 of a compact JSON value. `serde_json` sorts object keys, so the
/// encoding is canonical.
pub(crate) fn sha256_json(value: &serde_json::Value) -> String {
    format!("{:x}", Sha256::digest(value.to_string().as_bytes()))
}

/// Lowercase hex SHA-256 of the compact JSON object with sorted keys `channel`, `piece_id`, `text`.
pub fn revision(channel: Channel, piece_id: &str, text: &str) -> String {
    revision_with_visual(channel, piece_id, text, None)
}

/// One owner of piece revision identity; absent visual content preserves the old encoding.
#[expect(
    clippy::expect_used,
    reason = "VisualContent serializes only strings and string vectors, so hashing cannot encounter unsupported JSON values."
)]
pub(crate) fn revision_with_visual(
    channel: Channel,
    piece_id: &str,
    text: &str,
    visual: Option<&crate::visual::VisualContent>,
) -> String {
    let mut value =
        serde_json::json!({"channel": channel.name(), "piece_id":piece_id, "text":text});
    if let Some(visual) = visual {
        value["visual"] = serde_json::to_value(visual).expect("visual content serializes");
    }
    sha256_json(&value)
}

/// `review-<piece id>-<first 16 of revision>`.
pub fn review_decision_id(piece_id: &str, revision: &str) -> String {
    let prefix: String = revision.chars().take(16).collect();
    format!("review-{piece_id}-{prefix}")
}
