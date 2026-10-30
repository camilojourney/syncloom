//! Platform credentials: the token bundle, the store it lives in, and the macOS Keychain
//! store (`platform-credentials`).
//!
//! Invariant: a token is serialized only by a `TokenStore`; it never reaches a response, an
//! event, a log line or an error message. `Secret` has no `Display` and a redacted `Debug`.
use serde::{Deserialize, Serialize};
use std::fmt;

/// A secret string. `Debug` prints `[redacted]`; read it only through [`Secret::expose`].
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Secret(String);

impl Secret {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[redacted]")
    }
}

/// What a sign-in leaves behind: the access token, when it expires (RFC 3339 UTC), the member
/// id the platform needs as author, the granted scope and when it was obtained.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    pub access_token: Secret,
    pub expires_at: String,
    pub member_id: String,
    pub scope: String,
    pub obtained_at: String,
}

impl Token {
    /// Expired at or before `now` (both RFC 3339 UTC with whole seconds, so strings compare).
    pub fn is_expired_at(&self, now: &str) -> bool {
        self.expires_at.as_str() <= now
    }
}

/// The store could not be used. The kind is logged by the caller; never a secret.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoreError(pub String);

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "credential store failed: {}", self.0)
    }
}
impl std::error::Error for StoreError {}

/// Where tokens live, keyed by the platform's account name.
pub trait TokenStore: Send + Sync {
    fn load(&self, account: &str) -> Result<Option<Token>, StoreError>;
    fn save(&self, account: &str, token: &Token) -> Result<(), StoreError>;
    fn delete(&self, account: &str) -> Result<(), StoreError>;
}

/// The macOS Keychain: one generic-password item per account under `service`.
pub struct KeychainStore {
    service: String,
}

impl KeychainStore {
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
        }
    }
    pub fn service(&self) -> &str {
        &self.service
    }
}

impl TokenStore for KeychainStore {
    fn load(&self, _account: &str) -> Result<Option<Token>, StoreError> {
        Err(StoreError("not implemented".to_owned()))
    }
    fn save(&self, _account: &str, _token: &Token) -> Result<(), StoreError> {
        Err(StoreError("not implemented".to_owned()))
    }
    fn delete(&self, _account: &str) -> Result<(), StoreError> {
        Err(StoreError("not implemented".to_owned()))
    }
}
