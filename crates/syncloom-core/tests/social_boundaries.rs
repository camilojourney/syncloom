#[path = "support/fake_social.rs"]
mod fake;
#[path = "support/social.rs"]
mod fixture;
mod support;
use fake::Fake;
use fixture::*;
use serde_json::json;
use std::sync::Arc;
use support::memory_store::MemoryStore;
use syncloom_core::{Platform, PostFailure, Secret, StudioError, TokenStore, social::SocialConfig};

#[tokio::test]
async fn server_errors_are_uncertain_even_when_the_body_claims_retryable_auth_or_rate_errors() {
    let fake = Fake::threads().await;
    let q = social(&fake, MemoryStore::new(), clock());
    threads_login(&q).await;
    fake.reply("/v1.0/303/threads", 500, json!({"error":{"code":4}}));
    assert_eq!(
        q.threads.publish(text(q.threads.as_ref())).await,
        Err(PostFailure::Unknown)
    );
    let fake = Fake::bluesky().await;
    let q = social(&fake, MemoryStore::new(), clock());
    q.bluesky
        .sign_in("test.invalid", Secret::new("password"))
        .await
        .unwrap();
    fake.reply(
        "/xrpc/com.atproto.repo.createRecord",
        500,
        json!({"error":"ExpiredToken"}),
    );
    assert_eq!(
        q.bluesky.publish(text(q.bluesky.as_ref())).await,
        Err(PostFailure::Unknown)
    );
}

#[tokio::test]
async fn independent_threads_callback_and_x_scope_guards_reject_before_storage() {
    for wrong_origin in [true, false] {
        let fake = Fake::threads().await;
        let q = social(&fake, MemoryStore::new(), clock());
        let start = q.threads.sign_in_begin().unwrap();
        let state = reqwest::Url::parse(&start.authorization_url)
            .unwrap()
            .query_pairs()
            .find(|(k, _)| k == "state")
            .unwrap()
            .1
            .into_owned();
        let host = if wrong_origin {
            "wrong.invalid"
        } else {
            "redirect.invalid"
        };
        let state = if wrong_origin { state } else { "wrong".into() };
        assert_eq!(
            q.threads
                .complete(&format!("https://{host}/callback?code=c&state={state}"))
                .await,
            Err(StudioError::SignInStateMismatch)
        );
        assert!(fake.calls().is_empty());
    }
    let fake = Fake::x().await;
    let store = MemoryStore::new();
    let q = social(&fake, store.clone(), clock());
    fake.reply("/2/oauth2/token",200,json!({"access_token":"token","refresh_token":"refresh","expires_in":7200,"scope":"tweet.read","token_type":"bearer"}));
    let start = q.x.sign_in_begin().unwrap();
    let u = reqwest::Url::parse(&start.authorization_url).unwrap();
    let pairs: std::collections::HashMap<_, _> = u.query_pairs().into_owned().collect();
    reqwest::get(format!(
        "{}?code=c&state={}",
        pairs["redirect_uri"], pairs["state"]
    ))
    .await
    .unwrap();
    assert_eq!(q.x.sign_in_wait().await, Err(StudioError::SignInRejected));
    assert!(
        store
            .load(&q.account(syncloom_core::Channel::X).unwrap())
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn threads_permalink_failure_after_publication_is_unknown_not_retryable() {
    for status in [401, 403, 429, 500] {
        let fake = Fake::threads().await;
        let q = social(&fake, MemoryStore::new(), clock());
        threads_login(&q).await;
        fake.reply(
            "/v1.0/505",
            status,
            json!({"error":{"code":if status==401 {190}else{4}}}),
        );
        assert_eq!(
            q.threads.publish(text(q.threads.as_ref())).await,
            Err(PostFailure::Unknown)
        );
        assert_eq!(
            fake.calls()
                .iter()
                .filter(|c| c.path.ends_with("threads_publish"))
                .count(),
            1
        );
    }
}

#[tokio::test]
async fn bluesky_and_threads_report_all_http_failures_once_and_reject_malformed_success() {
    for (status, expected) in [
        (401, PostFailure::ConsentRequired),
        (403, PostFailure::Refused),
        (429, PostFailure::RateLimited),
        (500, PostFailure::Unknown),
    ] {
        let fake = Fake::bluesky().await;
        let q = social(&fake, MemoryStore::new(), clock());
        q.bluesky
            .sign_in("test.invalid", Secret::new("password"))
            .await
            .unwrap();
        fake.reply(
            "/xrpc/com.atproto.repo.createRecord",
            status,
            json!({"error":"rejected"}),
        );
        assert_eq!(
            q.bluesky.publish(text(q.bluesky.as_ref())).await,
            Err(expected)
        );
        assert_eq!(fake.calls().len(), 2);
        let fake = Fake::threads().await;
        let q = social(&fake, MemoryStore::new(), clock());
        threads_login(&q).await;
        fake.reply(
            "/v1.0/303/threads",
            status,
            json!({"error":{"code":if status==401 {190}else if status==429 {32}else{999}}}),
        );
        assert_eq!(
            q.threads.publish(text(q.threads.as_ref())).await,
            Err(expected)
        );
        assert_eq!(fake.calls().len(), 4);
    }
    let fake = Fake::bluesky().await;
    let q = social(&fake, MemoryStore::new(), clock());
    q.bluesky
        .sign_in("test.invalid", Secret::new("password"))
        .await
        .unwrap();
    fake.reply(
        "/xrpc/com.atproto.repo.createRecord",
        200,
        json!({"uri":"at://did:plc:other/app.bsky.feed.post/key","cid":fake::CID}),
    );
    assert_eq!(
        q.bluesky.publish(text(q.bluesky.as_ref())).await,
        Err(PostFailure::Unknown)
    );
    let fake = Fake::threads().await;
    let q = social(&fake, MemoryStore::new(), clock());
    threads_login(&q).await;
    fake.reply(
        "/v1.0/505",
        200,
        json!({"id":"505","permalink":"https://evil.invalid/post"}),
    );
    assert_eq!(
        q.threads.publish(text(q.threads.as_ref())).await,
        Err(PostFailure::Unknown)
    );
}

#[tokio::test]
async fn failed_refresh_keeps_both_jwts_and_expired_access_never_sends_a_post() {
    let fake = Fake::bluesky().await;
    let c = clock();
    let store = MemoryStore::new();
    let q = social(&fake, store.clone(), c.clone());
    q.bluesky
        .sign_in("test.invalid", Secret::new("password"))
        .await
        .unwrap();
    let before = store
        .load(&q.account(syncloom_core::Channel::Bluesky).unwrap())
        .unwrap();
    fake.reply(
        "/xrpc/com.atproto.server.refreshSession",
        401,
        json!({"error":"ExpiredToken"}),
    );
    assert_eq!(q.bluesky.refresh().await, Err(StudioError::SignInRejected));
    assert_eq!(
        store
            .load(&q.account(syncloom_core::Channel::Bluesky).unwrap())
            .unwrap(),
        before
    );
    *c.0.lock().unwrap() += time::Duration::days(3);
    let calls = fake.calls().len();
    assert_eq!(
        q.bluesky.publish(text(q.bluesky.as_ref())).await,
        Err(PostFailure::ConsentRequired)
    );
    assert_eq!(fake.calls().len(), calls);
    let fake = Fake::x().await;
    let c = clock();
    let store = MemoryStore::new();
    let q = social(&fake, store.clone(), c.clone());
    x_login(&q).await;
    let before = store
        .load(&q.account(syncloom_core::Channel::X).unwrap())
        .unwrap();
    fake.reply("/2/oauth2/token", 400, json!({"error":"invalid_grant"}));
    assert_eq!(
        q.refresh(syncloom_core::Channel::X).await,
        Err(StudioError::SignInRejected)
    );
    assert_eq!(
        store
            .load(&q.account(syncloom_core::Channel::X).unwrap())
            .unwrap(),
        before
    );
    *c.0.lock().unwrap() += time::Duration::hours(2);
    let count = fake.calls().len();
    assert_eq!(
        q.x.publish(text(q.x.as_ref())).await,
        Err(PostFailure::ConsentRequired)
    );
    assert_eq!(fake.calls().len(), count);
}

#[tokio::test]
async fn text_image_limits_and_unfinished_media_never_create_a_post() {
    let fake = Fake::x().await;
    let q = social(&fake, MemoryStore::new(), clock());
    x_login(&q).await;
    let calls = fake.calls().len();
    for (bytes, alt) in [
        (b"not PNG".to_vec(), "alt".to_owned()),
        (b"\x89PNG\r\n\x1a\n".to_vec(), " ".into()),
        (b"\x89PNG\r\n\x1a\n".to_vec(), "a".repeat(1001)),
        (
            {
                let mut b = b"\x89PNG\r\n\x1a\n".to_vec();
                b.resize(5_000_001, 0);
                b
            },
            "alt".into(),
        ),
    ] {
        let mut request = text(q.x.as_ref());
        request.image = Some(syncloom_core::platform::PostImage {
            bytes,
            alt_text: alt,
        });
        assert_eq!(q.x.publish(request).await, Err(PostFailure::Refused));
    }
    let mut request = text(q.x.as_ref());
    request.text = "x".repeat(281);
    assert_eq!(q.x.publish(request).await, Err(PostFailure::Refused));
    assert_eq!(fake.calls().len(), calls);
    fake.reply(
        "/2/media/upload/101/finalize",
        200,
        json!({"data":{"id":"101","processing_info":{"state":"pending"}}}),
    );
    let mut request = text(q.x.as_ref());
    request.image = Some(syncloom_core::platform::PostImage {
        bytes: b"\x89PNG\r\n\x1a\n".to_vec(),
        alt_text: "alt".into(),
    });
    assert_eq!(q.x.publish(request).await, Err(PostFailure::Unknown));
    assert!(!fake.calls().iter().any(|c| c.path == "/2/tweets"));
    let fake = Fake::threads().await;
    let q = social(&fake, MemoryStore::new(), clock());
    threads_login(&q).await;
    let calls = fake.calls().len();
    let mut request = text(q.threads.as_ref());
    request.image = Some(syncloom_core::platform::PostImage {
        bytes: b"\x89PNG\r\n\x1a\n".to_vec(),
        alt_text: "alt".into(),
    });
    assert_eq!(q.threads.publish(request).await, Err(PostFailure::Refused));
    assert_eq!(fake.calls().len(), calls);
    fake.reply("/v1.0/404", 200, json!({"id":"404","status":"IN_PROGRESS"}));
    assert_eq!(
        q.threads.publish(text(q.threads.as_ref())).await,
        Err(PostFailure::Unknown)
    );
    assert!(
        !fake
            .calls()
            .iter()
            .any(|c| c.path.ends_with("threads_publish"))
    );
}

#[test]
fn oauth_configuration_requires_real_redirect_shape_and_explicit_client_settings() {
    for redirect in [
        "http://localhost:8000/cb",
        "http://redirect.invalid/cb",
        "https://localhost/cb",
        "https://127.0.0.1/cb",
        "https://x.localhost/cb",
        "https://redirect.invalid/cb?code=c",
        "https://u:p@redirect.invalid/cb",
    ] {
        assert!(
            SocialConfig::parse(|n| (n == "SYNCLOOM_THREADS_REDIRECT_URL").then(|| redirect.into()))
                .is_err(),
            "{redirect}"
        );
    }
    assert!(
        SocialConfig::parse(|n| (n == "SYNCLOOM_THREADS_REDIRECT_URL")
            .then(|| "https://redirect.invalid/cb".into()))
        .is_ok()
    );
}

struct RejectWrites;
impl TokenStore for RejectWrites {
    fn load(&self, _: &str) -> Result<Option<syncloom_core::Token>, syncloom_core::StoreError> {
        Ok(None)
    }
    fn save(&self, _: &str, _: &syncloom_core::Token) -> Result<(), syncloom_core::StoreError> {
        Err(syncloom_core::StoreError("test-refused".into()))
    }
    fn delete(&self, _: &str) -> Result<(), syncloom_core::StoreError> {
        Ok(())
    }
}
#[tokio::test]
async fn rejected_credential_store_never_claims_signed_in() {
    let fake = Fake::bluesky().await;
    let q = social(&fake, Arc::new(RejectWrites), clock());
    assert_eq!(
        q.bluesky
            .sign_in("test.invalid", Secret::new("password"))
            .await,
        Err(StudioError::Internal)
    );
    assert_eq!(
        q.bluesky.publish(text(q.bluesky.as_ref())).await,
        Err(PostFailure::ConsentRequired)
    );
}
