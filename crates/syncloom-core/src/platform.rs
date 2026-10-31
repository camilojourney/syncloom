//! The platform boundary: what a platform adapter receives for one dispatch attempt and the
//! outcomes it may answer (`dispatch-outbox`, `linkedin-integration`).
use std::{future::Future, pin::Pin};

/// One attempt to post one approved piece revision. `request_id` is the intent's id, the
/// studio's idempotency key; the adapter may use it where a platform accepts one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostRequest {
    pub request_id: String,
    pub text: String,
    pub image: Option<PostImage>,
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
    fn publish(&self, request: PostRequest) -> PostFuture<'_>;
}
