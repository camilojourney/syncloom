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

/// The macOS Keychain: one generic-password item per account under `service`, holding the
/// token bundle as JSON. Read for this store: `keyring-core` 1.0.1 (`Entry`, `Error::NoEntry`)
/// and `apple-native-keyring-store` 1.0.2 (`keychain::Store`, the login keychain).
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

    #[cfg(target_os = "macos")]
    fn entry(&self, account: &str) -> Result<keyring_core::Entry, StoreError> {
        let store = apple_native_keyring_store::keychain::Store::new().map_err(kind)?;
        // The store's `build` needs no process-wide default; one store per call keeps the
        // Keychain domain explicit and avoids global state.
        use keyring_core::api::CredentialStoreApi;
        store.build(&self.service, account, None).map_err(kind)
    }
}

/// The error's kind as the store reports it; the message never carries the secret.
#[cfg(target_os = "macos")]
fn kind(error: keyring_core::Error) -> StoreError {
    let name = match error {
        keyring_core::Error::PlatformFailure(_) => "platform_failure",
        keyring_core::Error::NoStorageAccess(_) => "no_storage_access",
        keyring_core::Error::NoEntry => "no_entry",
        keyring_core::Error::BadEncoding(_) => "bad_encoding",
        keyring_core::Error::BadDataFormat(_, _) => "bad_data_format",
        keyring_core::Error::BadStoreFormat(_) => "bad_store_format",
        keyring_core::Error::TooLong(_, _) => "too_long",
        keyring_core::Error::Invalid(_, _) => "invalid",
        keyring_core::Error::Ambiguous(_) => "ambiguous",
        keyring_core::Error::NoDefaultStore => "no_default_store",
        keyring_core::Error::NotSupportedByStore(_) => "not_supported",
        _ => "other",
    };
    StoreError(name.to_owned())
}

#[cfg(target_os = "macos")]
impl TokenStore for KeychainStore {
    fn load(&self, account: &str) -> Result<Option<Token>, StoreError> {
        match self.entry(account)?.get_secret() {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map(Some)
                .map_err(|_| StoreError("bad_token_bundle".to_owned())),
            Err(keyring_core::Error::NoEntry) => Ok(None),
            Err(error) => Err(kind(error)),
        }
    }
    fn save(&self, account: &str, token: &Token) -> Result<(), StoreError> {
        let bytes =
            serde_json::to_vec(token).map_err(|_| StoreError("token_serialize".to_owned()))?;
        self.entry(account)?.set_secret(&bytes).map_err(kind)
    }
    fn delete(&self, account: &str) -> Result<(), StoreError> {
        match self.entry(account)?.delete_credential() {
            Ok(()) | Err(keyring_core::Error::NoEntry) => Ok(()),
            Err(error) => Err(kind(error)),
        }
    }
}

/// No Keychain on this platform: the store reports itself unsupported rather than pretending.
#[cfg(not(target_os = "macos"))]
impl TokenStore for KeychainStore {
    fn load(&self, _account: &str) -> Result<Option<Token>, StoreError> {
        Err(StoreError("keychain_unsupported_platform".to_owned()))
    }
    fn save(&self, _account: &str, _token: &Token) -> Result<(), StoreError> {
        Err(StoreError("keychain_unsupported_platform".to_owned()))
    }
    fn delete(&self, _account: &str) -> Result<(), StoreError> {
        Err(StoreError("keychain_unsupported_platform".to_owned()))
    }
}
