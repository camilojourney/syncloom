#[path = "support/fake_social.rs"]
mod fake;
#[path = "support/social.rs"]
mod fixture;
mod support;
use fake::Fake;
use fixture::*;
use serde_json::json;
use support::memory_store::MemoryStore;
use syncloom_core::{Secret, StudioError, TokenStore};

#[tokio::test]
async fn bluesky_requires_handle_and_password_without_sending_private_identifiers() {
    let fake = Fake::bluesky().await;
    let q = social(&fake, MemoryStore::new(), clock());
    for (identifier, password) in [
        ("", "test"),
        ("test@example.invalid", "test"),
        ("test.invalid", " "),
    ] {
        assert_eq!(
            q.bluesky.sign_in(identifier, Secret::new(password)).await,
            Err(StudioError::InvalidRequest)
        );
    }
    assert!(fake.calls().is_empty());
}

#[tokio::test]
async fn bluesky_rejects_expired_or_unrefreshable_sessions_and_identity_drift() {
    for (exp, refresh) in [(1793412000, "refresh"), (1793455200, "")] {
        let fake = Fake::bluesky().await;
        let store = MemoryStore::new();
        let q = social(&fake, store.clone(), clock());
        fake.reply("/xrpc/com.atproto.server.createSession",200,json!({"accessJwt":fake::jwt(exp),"refreshJwt":refresh,"did":"did:plc:testrepo","handle":"test.invalid"}));
        assert_eq!(
            q.bluesky.sign_in("test.invalid", Secret::new("test")).await,
            Err(StudioError::SignInRejected)
        );
        assert!(
            store
                .load(&q.account(syncloom_core::Channel::Bluesky).unwrap())
                .unwrap()
                .is_none()
        );
    }
    let fake = Fake::bluesky().await;
    let store = MemoryStore::new();
    let q = social(&fake, store.clone(), clock());
    q.bluesky
        .sign_in("test.invalid", Secret::new("test"))
        .await
        .unwrap();
    let before = store
        .load(&q.account(syncloom_core::Channel::Bluesky).unwrap())
        .unwrap();
    fake.reply("/xrpc/com.atproto.server.refreshSession",200,json!({"accessJwt":fake::jwt(1793541600),"refreshJwt":"new-refresh","did":"did:plc:different","handle":"other.invalid"}));
    assert_eq!(q.bluesky.refresh().await, Err(StudioError::SignInRejected));
    assert_eq!(
        store
            .load(&q.account(syncloom_core::Channel::Bluesky).unwrap())
            .unwrap(),
        before
    );
}

#[tokio::test]
async fn x_rejects_invalid_issued_tokens_without_committing_credentials() {
    for (key, value) in [
        ("expires_in", json!(0)),
        ("expires_in", json!(u64::MAX)),
        ("expires_in", json!(i64::MAX)),
        ("access_token", json!("")),
        ("refresh_token", json!("")),
        ("token_type", json!("basic")),
        ("error", json!({"code":190})),
    ] {
        let fake = Fake::x().await;
        let store = MemoryStore::new();
        let q = social(&fake, store.clone(), clock());
        let mut issued = json!({"access_token":"token","refresh_token":"refresh","expires_in":7200,"scope":"tweet.read tweet.write users.read media.write offline.access","token_type":"bearer"});
        issued[key] = value;
        fake.reply("/2/oauth2/token", 200, issued);
        let start = q.x.sign_in_begin().unwrap();
        let u = reqwest::Url::parse(&start.authorization_url).unwrap();
        let pairs: std::collections::HashMap<_, _> = u.query_pairs().into_owned().collect();
        reqwest::get(format!(
            "{}?code=c&state={}",
            pairs["redirect_uri"], pairs["state"]
        ))
        .await
        .unwrap();
        assert_eq!(
            q.x.sign_in_wait().await,
            Err(StudioError::SignInRejected),
            "{key}"
        );
        assert!(
            store
                .load(&q.account(syncloom_core::Channel::X).unwrap())
                .unwrap()
                .is_none()
        );
    }
}
