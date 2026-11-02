//! Publish intents: one per piece, revision and operation, holding the attempt record and the
//! truthful outcome of dispatch (`dispatch-outbox`).
use crate::piece::sha256_json;
use serde::{Deserialize, Serialize};

pub const OPERATION_PUBLISH: &str = "publish";

/// Where an intent stands. Only `Posted` means the platform confirmed a post with its id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntentStatus {
    /// Reserved while no delivery grant names the platform; nothing left the machine.
    Contained,
    /// Local manual handoff completed. Never a remote post.
    Exported,
    /// Written before a platform request; found later, it means an unknown outcome.
    Posting,
    /// The platform answered with its post id.
    Posted,
    /// The platform refused the request (4xx other than 401 and 429).
    Refused,
    /// No usable token, or the platform refused it (401).
    ConsentRequired,
    /// 429 from the platform.
    RateLimited,
    /// No connection; nothing was sent.
    Unreachable,
    /// The request may have reached the platform; the founder must check before retrying.
    Unknown,
}

impl IntentStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            IntentStatus::Contained => "contained",
            IntentStatus::Exported => "exported",
            IntentStatus::Posting => "posting",
            IntentStatus::Posted => "posted",
            IntentStatus::Refused => "refused",
            IntentStatus::ConsentRequired => "consent_required",
            IntentStatus::RateLimited => "rate_limited",
            IntentStatus::Unreachable => "unreachable",
            IntentStatus::Unknown => "unknown",
        }
    }
    /// May a new attempt be made from this status? `Posted` never; `Posting` and `Unknown`
    /// never blindly (the outcome is not known); the rest yes.
    pub fn allows_new_attempt(self) -> bool {
        matches!(
            self,
            IntentStatus::Contained
                | IntentStatus::Refused
                | IntentStatus::ConsentRequired
                | IntentStatus::RateLimited
                | IntentStatus::Unreachable
        )
    }
}

/// What a platform integration receives.
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
    /// Target of the last claimed attempt, retained even after reconnect or restart.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_target: Option<String>,
    pub payload: Payload,
    pub status: IntentStatus,
    pub created_at: String,
    /// Dispatch attempts made so far; slice 1 records have none.
    #[serde(default)]
    pub attempts: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempted_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_estimate: Option<crate::social::CostEstimate>,
}

/// One intent per operation, piece, revision and exact target. Unbound/manual keys
/// retain their historical encoding so uncertain legacy attempts cannot be bypassed.
pub fn intent_key(operation: &str, piece_id: &str, revision: &str, target: Option<&str>) -> String {
    let mut value = serde_json::json!({
        "operation": operation, "piece_id": piece_id, "revision": revision
    });
    if let Some(target) = target {
        value["target"] = serde_json::json!(target);
    }
    sha256_json(&value)
}

/// `dispatch-` plus the first 32 characters of [`intent_key`].
pub fn request_id(operation: &str, piece_id: &str, revision: &str, target: Option<&str>) -> String {
    let key = intent_key(operation, piece_id, revision, target);
    format!("dispatch-{}", &key[..32])
}
