//! The platform boundary: what a platform adapter receives for one dispatch attempt and the
//! outcomes it may answer (`dispatch-outbox`, `linkedin-integration`).
use std::{future::Future, pin::Pin};

/// One attempt to post one approved piece revision. `request_id` is the intent's id, the
/// studio's idempotency key; the adapter may use it where a platform accepts one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostRequest {
    pub request_id: String,
    pub text: String,
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

pub type PostFuture<'a> = Pin<Box<dyn Future<Output = Result<Posted, PostFailure>> + Send + 'a>>;

/// A platform adapter. Exactly one platform request per call; dropping the future abandons it.
pub trait Platform: Send + Sync {
    fn publish(&self, request: PostRequest) -> PostFuture<'_>;
}
