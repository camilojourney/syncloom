#[path = "support/fake_social.rs"]
mod fake;
mod support;
use fake::Fake;
use serde_json::json;
use std::{sync::Arc, time::Duration};
#[path = "support/social.rs"]
mod fixture;
use fixture::*;
use support::memory_store::MemoryStore;
use syncloom_core::{Platform, PostFailure, Secret, StudioError, TokenStore};

#[tokio::test]
async fn bluesky_password_session_refresh_and_image_use_only_configured_pds() {
    let fake = Fake::bluesky().await;
    let store = Arc::new(MemoryStore::default());
    let q = social(&fake, store.clone(), clock());
    q.bluesky
        .sign_in("test.invalid", Secret::new("app-password-canary"))
        .await
        .unwrap();
    assert_eq!(fake.calls().len(), 1);
    q.bluesky.refresh().await.unwrap();
    let mut request = text();
    request.image = Some(syncloom_core::platform::PostImage {
        bytes: b"\x89PNG\r\n\x1a\n".to_vec(),
        alt_text: "Test chart.".into(),
    });
    let posted = q.bluesky.publish(request).await.unwrap();
    assert_eq!(
        posted.post_id,
        "at://did:plc:testrepo/app.bsky.feed.post/testrecord"
    );
    let calls = fake.calls();
    assert_eq!(
        calls.iter().map(|c| c.path.as_str()).collect::<Vec<_>>(),
        [
            "/xrpc/com.atproto.server.createSession",
            "/xrpc/com.atproto.server.refreshSession",
            "/xrpc/com.atproto.repo.uploadBlob",
            "/xrpc/com.atproto.repo.createRecord"
        ]
    );
    let record = calls[3].json();
    assert_eq!(record["record"]["embed"]["images"][0]["alt"], "Test chart.");
    assert_eq!(calls[1].authorization, "Bearer refresh-bsky-1");
    assert_eq!(
        calls[3].authorization,
        format!("Bearer {}", fake::jwt(1793541600))
    );
    let serialized = serde_json::to_string(&q.bluesky.connection().unwrap()).unwrap();
    assert!(!serialized.contains("canary"));
    q.bluesky.sign_out().unwrap();
    assert_eq!(
        q.bluesky.publish(text()).await,
        Err(PostFailure::ConsentRequired)
    );
    assert_eq!(fake.calls().len(), 4);
}

#[tokio::test]
async fn x_pkce_refresh_rotation_and_current_v2_image_contract() {
    use base64::Engine;
    use sha2::{Digest, Sha256};
    let fake = Fake::x().await;
    let q = social(&fake, Arc::new(MemoryStore::default()), clock());
    let start = q.x.sign_in_begin().unwrap();
    let u = reqwest::Url::parse(&start.authorization_url).unwrap();
    let p: std::collections::HashMap<_, _> = u.query_pairs().into_owned().collect();
    reqwest::get(format!(
        "{}?code=test-code&state={}",
        p["redirect_uri"], p["state"]
    ))
    .await
    .unwrap();
    q.x.sign_in_wait().await.unwrap();
    let form = fake.calls()[0].form();
    assert_eq!(
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(Sha256::digest(form["code_verifier"].as_bytes())),
        p["code_challenge"]
    );
    assert!(!form.contains_key("client_secret"));
    fake.reply("/2/oauth2/token",200,json!({"access_token":"x-access-2","refresh_token":"x-refresh-2","expires_in":7200,"scope":"tweet.read tweet.write users.read media.write offline.access","token_type":"bearer"}));
    q.refresh(syncloom_core::Channel::X).await.unwrap();
    q.refresh(syncloom_core::Channel::X).await.unwrap();
    assert_eq!(fake.calls()[1].form()["refresh_token"], "x-refresh-1");
    assert_eq!(fake.calls()[2].form()["refresh_token"], "x-refresh-2");
    let mut request = text();
    request.image = Some(syncloom_core::platform::PostImage {
        bytes: b"\x89PNG\r\n\x1a\n".to_vec(),
        alt_text: "Test chart.".into(),
    });
    assert_eq!(q.x.publish(request).await.unwrap().post_id, "202");
    let calls = fake.calls();
    assert_eq!(
        calls[3].json(),
        json!({"total_bytes":8,"media_type":"image/png","media_category":"tweet_image"})
    );
    assert!(calls[4].content_type.starts_with("multipart/form-data"));
    assert_eq!(calls[5].path, "/2/media/upload/101/finalize");
    assert_eq!(
        calls[6].json(),
        json!({"id":"101","metadata":{"alt_text":{"text":"Test chart."}}})
    );
    assert_eq!(calls[7].json()["media"]["media_ids"], json!(["101"]));
    assert_eq!(calls[7].authorization, "Bearer x-access-2");
}

#[tokio::test]
async fn threads_external_state_long_lived_exchange_refresh_window_and_text() {
    let fake = Fake::threads().await;
    let c = clock();
    let store = Arc::new(MemoryStore::default());
    let q = social(&fake, store.clone(), c.clone());
    threads_login(&q).await;
    assert_eq!(
        fake.calls()[0].form()["redirect_uri"],
        "https://redirect.invalid/callback"
    );
    assert!(
        fake.calls()[1]
            .query
            .contains("grant_type=th_exchange_token")
    );
    let query: std::collections::HashMap<_, _> =
        url::form_urlencoded::parse(fake.calls()[2].query.as_bytes())
            .into_owned()
            .collect();
    assert_eq!(fake.calls()[2].path, "/v1.0/debug_token");
    assert_eq!(query["input_token"], "threads-long-1");
    assert_eq!(query["access_token"], "TH|test-client|test-secret-canary");
    let saved = store
        .load(&q.account(syncloom_core::Channel::Threads).unwrap())
        .unwrap()
        .unwrap();
    assert!(saved.scope_verified);
    assert_eq!(saved.scope, "threads_basic threads_content_publish");
    let count = fake.calls().len();
    assert_eq!(
        q.refresh(syncloom_core::Channel::Threads).await,
        Err(StudioError::SignInRejected)
    );
    assert_eq!(fake.calls().len(), count);
    *c.0.lock().unwrap() += time::Duration::days(1);
    q.refresh(syncloom_core::Channel::Threads).await.unwrap();
    assert!(
        fake.calls()[3]
            .query
            .contains("access_token=threads-long-1")
    );
    assert_eq!(q.threads.publish(text()).await.unwrap().post_id, "505");
    let query: std::collections::HashMap<_, _> =
        url::form_urlencoded::parse(fake.calls()[4].query.as_bytes())
            .into_owned()
            .collect();
    assert_eq!(fake.calls()[4].path, "/v1.0/debug_token");
    assert_eq!(query["input_token"], "threads-long-2");
    assert_eq!(fake.calls()[5].form()["media_type"], "TEXT");
    assert_eq!(fake.calls()[7].form()["creation_id"], "404");
    assert_eq!(fake.calls()[7].authorization, "Bearer threads-long-2");
    assert_eq!(fake.calls().len(), 9);
    *c.0.lock().unwrap() += time::Duration::days(60);
    let count = fake.calls().len();
    assert_eq!(
        q.refresh(syncloom_core::Channel::Threads).await,
        Err(StudioError::SignInRejected)
    );
    assert_eq!(
        q.threads.publish(text()).await,
        Err(PostFailure::ConsentRequired)
    );
    assert_eq!(fake.calls().len(), count);
}

#[tokio::test]
async fn oauth_state_redirect_scope_and_refresh_failures_never_store_or_post() {
    let fake = Fake::threads().await;
    let store = Arc::new(MemoryStore::default());
    let c = clock();
    let q = social(&fake, store.clone(), c.clone());
    q.threads.sign_in_begin().unwrap();
    assert_eq!(
        q.threads
            .complete("https://wrong.invalid/callback?code=c&state=wrong")
            .await,
        Err(StudioError::SignInStateMismatch)
    );
    assert!(fake.calls().is_empty());
    let start = q.threads.sign_in_begin().unwrap();
    let state = reqwest::Url::parse(&start.authorization_url)
        .unwrap()
        .query_pairs()
        .find(|(k, _)| k == "state")
        .unwrap()
        .1
        .into_owned();
    assert_eq!(
        q.threads
            .complete(&format!(
                "https://redirect.invalid/callback?code=c&state={state}&state={state}"
            ))
            .await,
        Err(StudioError::SignInStateMismatch)
    );
    assert!(fake.calls().is_empty());
    threads_login(&q).await;
    let before = store
        .load(&q.account(syncloom_core::Channel::Threads).unwrap())
        .unwrap();
    fake.reply(
        "/refresh_access_token",
        400,
        json!({"error":{"code":190,"message":"secret-canary"}}),
    );
    *c.0.lock().unwrap() += time::Duration::days(1);
    assert_eq!(
        q.refresh(syncloom_core::Channel::Threads).await,
        Err(StudioError::SignInRejected)
    );
    assert_eq!(
        store
            .load(&q.account(syncloom_core::Channel::Threads).unwrap())
            .unwrap(),
        before
    );
}

#[tokio::test]
async fn signing_out_during_refresh_cannot_resurrect_credentials() {
    let fake = Fake::x().await;
    let store = Arc::new(MemoryStore::default());
    let q = Arc::new(social(&fake, store.clone(), clock()));
    x_login(&q).await;
    let hold = fake.hold("/2/oauth2/token");
    let active = q.clone();
    let refreshing = tokio::spawn(async move { active.refresh(syncloom_core::Channel::X).await });
    fake.wait_calls(2).await;
    q.x.sign_out().unwrap();
    hold.notify_one();
    assert_eq!(refreshing.await.unwrap(), Err(StudioError::SignInRejected));
    assert!(
        store
            .load(&q.account(syncloom_core::Channel::X).unwrap())
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn sign_out_during_loopback_token_exchange_cannot_resurrect_credentials() {
    let fake = Fake::x().await;
    let store = MemoryStore::new();
    let q = Arc::new(social(&fake, store.clone(), clock()));
    let start = q.x.sign_in_begin().unwrap();
    let url = reqwest::Url::parse(&start.authorization_url).unwrap();
    let pairs: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
    reqwest::get(format!(
        "{}?code=c&state={}",
        pairs["redirect_uri"], pairs["state"]
    ))
    .await
    .unwrap();
    let hold = fake.hold("/2/oauth2/token");
    let active = q.clone();
    let exchange = tokio::spawn(async move { active.x.sign_in_wait().await });
    fake.wait_calls(1).await;
    q.sign_out(syncloom_core::Channel::X).unwrap();
    hold.notify_one();
    assert_eq!(exchange.await.unwrap(), Err(StudioError::SignInRejected));
    assert!(
        store
            .load(&q.account(syncloom_core::Channel::X).unwrap())
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn platform_failures_and_partial_media_success_never_claim_posted_or_retry() {
    for (status, expected) in [
        (401, PostFailure::ConsentRequired),
        (403, PostFailure::Refused),
        (429, PostFailure::RateLimited),
        (500, PostFailure::Unknown),
        (599, PostFailure::Unknown),
    ] {
        let fake = Fake::x().await;
        let q = social(&fake, Arc::new(MemoryStore::default()), clock());
        x_login(&q).await;
        fake.reply("/2/tweets", status, json!({"error":"secret-canary"}));
        assert_eq!(
            tokio::time::timeout(Duration::from_millis(30), q.x.publish(text()))
                .await
                .unwrap_or(Err(PostFailure::Unknown)),
            Err(expected)
        );
        assert_eq!(
            fake.calls()
                .iter()
                .filter(|c| c.path == "/2/tweets")
                .count(),
            1
        );
    }
    let fake = Fake::x().await;
    let q = social(&fake, Arc::new(MemoryStore::default()), clock());
    x_login(&q).await;
    fake.reply("/2/media/metadata", 400, json!({"error":"refused"}));
    let mut request = text();
    request.image = Some(syncloom_core::platform::PostImage {
        bytes: b"\x89PNG\r\n\x1a\n".to_vec(),
        alt_text: "Test chart.".into(),
    });
    assert_eq!(q.x.publish(request).await, Err(PostFailure::Refused));
    assert!(!fake.calls().iter().any(|c| c.path == "/2/tweets"));
}
