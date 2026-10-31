//! The macOS Keychain token store (platform-credentials "Token stored only in the Keychain",
//! "Test-only service name"), against the real Keychain under a test-only service name that
//! every test deletes.
mod support;
use std::process::Command;
use support::{FIXED_AT, TOKEN_EXPIRES_AT, stored_token};
use syncloom_core::{KeychainStore, Secret, Token, TokenStore};

/// A service name unique to this process, deleted when dropped even when the test panics.
struct TestService(String);
impl TestService {
    fn new(tag: &str) -> Self {
        Self(format!(
            "syncloom-test-{}-{tag}-{}",
            std::process::id(),
            uuid::Uuid::new_v4().simple()
        ))
    }
}
impl Drop for TestService {
    fn drop(&mut self) {
        let _ = KeychainStore::new(self.0.clone()).delete("linkedin");
    }
}

/// What the Keychain itself says about the item, through Apple's own tool.
fn keychain_has(service: &str) -> bool {
    Command::new("/usr/bin/security")
        .args(["find-generic-password", "-s", service, "-a", "linkedin"])
        .output()
        .unwrap()
        .status
        .success()
}

#[test]
fn a_token_round_trips_through_the_keychain_and_is_deleted_without_a_trace() {
    let service = TestService::new("roundtrip");
    let store = KeychainStore::new(service.0.clone());
    assert_eq!(
        store.load("linkedin").unwrap(),
        None,
        "fresh service holds nothing"
    );
    assert!(!keychain_has(&service.0));

    let token = stored_token(TOKEN_EXPIRES_AT);
    store.save("linkedin", &token).unwrap();
    assert!(
        keychain_has(&service.0),
        "the item is in the Keychain itself"
    );
    assert_eq!(store.load("linkedin").unwrap(), Some(token.clone()));
    // A second handle on the same service reads the same item: it is the Keychain, not memory.
    assert_eq!(
        KeychainStore::new(service.0.clone())
            .load("linkedin")
            .unwrap(),
        Some(token)
    );

    let renewed = Token {
        access_token: Secret::new("second-token"),
        expires_at: Some("2027-01-01T00:00:00Z".to_owned()),
        member_id: "M2".to_owned(),
        scope: "w_member_social".to_owned(),
        obtained_at: FIXED_AT.to_owned(),
    };
    store.save("linkedin", &renewed).unwrap();
    assert_eq!(
        store.load("linkedin").unwrap(),
        Some(renewed.clone()),
        "save overwrites"
    );
    let nonexpiring = Token {
        expires_at: None,
        ..renewed
    };
    store.save("linkedin", &nonexpiring).unwrap();
    assert_eq!(store.load("linkedin").unwrap(), Some(nonexpiring.clone()));
    assert!(!nonexpiring.is_expired_at("2099-01-01T00:00:00Z"));

    store.delete("linkedin").unwrap();
    assert_eq!(store.load("linkedin").unwrap(), None);
    assert!(
        !keychain_has(&service.0),
        "deleted from the Keychain itself"
    );
    store
        .delete("linkedin")
        .unwrap_or_else(|e| panic!("deleting an absent item is fine: {e}"));
}

#[test]
fn the_secret_is_redacted_in_debug_output() {
    let token = stored_token(TOKEN_EXPIRES_AT);
    let debug = format!("{token:?}");
    assert!(debug.contains("[redacted]"), "{debug}");
    assert!(!debug.contains("fake-access-token"), "{debug}");
    assert_eq!(token.access_token.expose(), "fake-access-token-zebrafish");
}

#[test]
fn a_token_is_expired_at_or_before_its_expiry_instant() {
    let token = stored_token("2026-12-29T18:00:00Z");
    assert!(!token.is_expired_at("2026-12-29T17:59:59Z"));
    assert!(token.is_expired_at("2026-12-29T18:00:00Z"));
    assert!(token.is_expired_at("2026-12-29T18:00:01Z"));
}
