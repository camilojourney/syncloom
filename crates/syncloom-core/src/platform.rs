//! The platform boundary: what a platform adapter receives for one dispatch attempt and the
//! outcomes it may answer (`dispatch-outbox`, `linkedin-integration`).
use std::{future::Future, pin::Pin};

/// One attempt to post one approved piece revision. `request_id` is the intent's id, the
/// studio's idempotency key; the adapter may use it where a platform accepts one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostRequest {
    pub request_id: String,
    /// Non-secret canonical target approved by the founder, never a credential-slot key.
    pub approval_target: Option<String>,
    pub text: String,
    pub image: Option<PostImage>,
    /// Ordered approved JPEG slides. Only carousel-capable adapters consume these.
    pub slides: Vec<PostImage>,
}

impl PostRequest {
    /// Check the exact credential snapshot before any media or posting side effect.
    pub fn require_target(&self, target: Result<String, PostFailure>) -> Result<(), PostFailure> {
        let target = target?;
        if self.approval_target.as_ref() != Some(&target) {
            return Err(PostFailure::ConsentRequired);
        }
        Ok(())
    }
}

/// Stable opaque identity; the provider adapter supplies observed canonical IDs and authority.
pub fn account_target(
    provider: &str,
    authority: &str,
    subject: &str,
) -> Result<String, PostFailure> {
    if provider.is_empty() || subject.is_empty() {
        return Err(PostFailure::ConsentRequired);
    }
    Ok(crate::piece::sha256_json(&serde_json::json!({
        "provider":provider, "authority":authority, "subject":subject
    })))
}

/// Verified image bytes, never a URL or caller-selected path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostImage {
    pub bytes: Vec<u8>,
    pub alt_text: String,
}

/// The platform's own id for the post and the URL where the founder can see it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Posted {
    pub post_id: String,
    pub url: String,
}

/// Why an attempt did not end in `posted`. Each maps to one intent status; the studio never
/// turns any of these into "posted".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PostFailure {
    /// No token, an expired token, or the platform refused the token (401).
    ConsentRequired,
    /// A credential exists, but its effective grant cannot post. Nothing is sent.
    PostingPermissionMissing,
    /// Effective permission evidence is absent or could not be verified. Nothing is sent.
    PostingPermissionUnverified,
    /// The platform refused the request for another reason (4xx other than 401 and 429).
    Refused,
    /// The platform asked to slow down (429).
    RateLimited,
    /// No connection could be made; the request was never sent.
    Unreachable,
    /// The request may have reached the platform (5xx, timeout, unreadable answer).
    Unknown,
}

pub(crate) fn transport_failure(error: reqwest::Error) -> PostFailure {
    if error.is_connect() {
        PostFailure::Unreachable
    } else {
        PostFailure::Unknown
    }
}

pub(crate) fn http_failure(status: reqwest::StatusCode) -> Option<PostFailure> {
    if status.is_success() {
        return None;
    }
    Some(match status.as_u16() {
        401 => PostFailure::ConsentRequired,
        429 => PostFailure::RateLimited,
        400..=499 => PostFailure::Refused,
        _ => PostFailure::Unknown,
    })
}

/// One owner for ordinary HTTP/JSON platform outcomes. Provider bodies never become errors.
pub(crate) async fn request_empty(builder: reqwest::RequestBuilder) -> Result<(), PostFailure> {
    let response = builder.send().await.map_err(transport_failure)?;
    http_failure(response.status()).map_or(Ok(()), Err)
}

pub(crate) async fn request_json<T: serde::de::DeserializeOwned>(
    builder: reqwest::RequestBuilder,
) -> Result<T, PostFailure> {
    let response = builder.send().await.map_err(transport_failure)?;
    if let Some(failure) = http_failure(response.status()) {
        return Err(failure);
    }
    crate::oauth::read_json(response)
        .await
        .ok_or(PostFailure::Unknown)
}

pub type PostFuture<'a> = Pin<Box<dyn Future<Output = Result<Posted, PostFailure>> + Send + 'a>>;

/// A platform adapter. One status-creation request per call; media preparation may precede it.
/// Dropping the future abandons it; the outbox treats the outcome as uncertain.
pub trait Platform: Send + Sync {
    /// Current resolved target. Unknown identity never grants posting authority.
    fn posting_target(&self) -> Result<String, PostFailure> {
        Err(PostFailure::ConsentRequired)
    }
    fn publish(&self, request: PostRequest) -> PostFuture<'_>;
}
