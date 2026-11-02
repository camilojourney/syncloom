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
    /// Shared secret comparison for OAuth state and the local HTTP session.
    pub fn matches(&self, candidate: &str) -> bool {
        use subtle::ConstantTimeEq;
        bool::from(self.0.as_bytes().ct_eq(candidate.as_bytes()))
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    pub member_id: String,
    /// App namespace captured with an app-scoped subject at authenticated resolution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity_authority: Option<String>,
    /// Meta's provider-confirmed Page, not the configured selection or Keychain slot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_id: Option<String>,
    pub scope: String,
    pub obtained_at: String,
}

impl Token {
    pub(crate) fn app_target(&self, provider: &str) -> Result<String, crate::PostFailure> {
        let authority = self
            .identity_authority
            .as_deref()
            .filter(|s| !s.is_empty())
            .ok_or(crate::PostFailure::ConsentRequired)?;
        crate::platform::account_target(provider, authority, &self.member_id)
    }
    /// Expired at or before `now` (both RFC 3339 UTC with whole seconds, so strings compare).
    pub fn is_expired_at(&self, now: &str) -> bool {
        self.expires_at
            .as_deref()
            .is_some_and(|expiry| expiry <= now)
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

    /// Shared dispatch credential policy. Missing and expired tokens send nothing.
    fn usable_token(&self, account: &str, now: &str) -> Result<Token, crate::PostFailure> {
        let token = self
            .load(account)
            .map_err(|_| crate::PostFailure::Unknown)?
            .ok_or(crate::PostFailure::ConsentRequired)?;
        if token.is_expired_at(now) {
            return Err(crate::PostFailure::ConsentRequired);
        }
        Ok(token)
    }
}

/// Serializes credential commits with sign-out. Network operations hold only a generation,
/// never this lock, so an old exchange cannot resurrect deleted credentials.
#[derive(Default)]
pub(crate) struct CredentialWrites(std::sync::Mutex<u64>);
impl CredentialWrites {
    #[expect(
        clippy::unwrap_used,
        reason = "Poisoned credential-generation state cannot be safely reused without risking stale credential commits."
    )]
    pub fn generation(&self) -> u64 {
        *self.0.lock().unwrap()
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Poisoned credential-generation state cannot be safely reused without risking stale credential commits."
    )]
    pub fn invalidate(&self) {
        *self.0.lock().unwrap() += 1;
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Poisoned credential-generation state cannot be safely reused without risking stale credential commits."
    )]
    pub fn save(
        &self,
        generation: u64,
        store: &dyn TokenStore,
        account: &str,
        token: &Token,
    ) -> Result<(), crate::StudioError> {
        let current = self.0.lock().unwrap();
        if *current != generation {
            return Err(crate::StudioError::SignInRejected);
        }
        store.save(account, token)?;
        Ok(())
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Poisoned credential-generation state cannot be safely reused without risking stale credential commits."
    )]
    pub fn delete(&self, store: &dyn TokenStore, account: &str) -> Result<(), crate::StudioError> {
        let mut current = self.0.lock().unwrap();
        *current += 1;
        store.delete(account)?;
        Ok(())
    }
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
    use keyring_core::Error::*;
    let name = match error {
        PlatformFailure(_) => "platform_failure",
        NoStorageAccess(_) => "no_storage_access",
        NoEntry => "no_entry",
        BadEncoding(_) | BadDataFormat(_, _) | BadStoreFormat(_) => "bad_format",
        TooLong(_, _) | Invalid(_, _) | Ambiguous(_) => "invalid",
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
