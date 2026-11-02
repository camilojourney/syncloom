#![allow(
    dead_code,
    reason = "Each integration target uses a different subset of the shared test fixtures."
)]
use crate::fake::Fake;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use syncloom_core::{
    Clock, PostRequest,
    social::{SocialConfig, SocialPlatforms},
};
use time::{OffsetDateTime, macros::datetime};
pub struct TestClock(pub Mutex<OffsetDateTime>);
impl Clock for TestClock {
    #[expect(
        clippy::unwrap_used,
        reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
    )]
    fn now(&self) -> OffsetDateTime {
        *self.0.lock().unwrap()
    }
}
pub fn clock() -> Arc<TestClock> {
    Arc::new(TestClock(Mutex::new(datetime!(2026-10-31 02:00 UTC))))
}
#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
pub fn social(
    fake: &Fake,
    store: Arc<dyn syncloom_core::TokenStore>,
    clock: Arc<TestClock>,
) -> SocialPlatforms {
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let config = SocialConfig::parse(|n| match n {
        "SYNCLOOM_BLUESKY_BASE_URL"
        | "SYNCLOOM_X_API_BASE_URL"
        | "SYNCLOOM_X_AUTH_BASE_URL"
        | "SYNCLOOM_THREADS_API_BASE_URL"
        | "SYNCLOOM_THREADS_AUTH_BASE_URL" => Some(fake.base().into()),
        "SYNCLOOM_X_CLIENT_ID" | "SYNCLOOM_THREADS_CLIENT_ID" => Some("test-client".into()),
        "SYNCLOOM_THREADS_CLIENT_SECRET" => Some("test-secret-canary".into()),
        "SYNCLOOM_X_REDIRECT_URL" => Some(format!("http://127.0.0.1:{port}/x/callback")),
        "SYNCLOOM_THREADS_REDIRECT_URL" => Some("https://redirect.invalid/callback".into()),
        _ => None,
    })
    .unwrap();
    SocialPlatforms::new(config, store, clock, Duration::from_secs(2))
}
pub fn text(platform: &dyn syncloom_core::Platform) -> PostRequest {
    PostRequest {
        approval_target: platform.posting_target().ok(),
        request_id: "dispatch-test".into(),
        text: "An approved test post.".into(),
        image: None,
        slides: Vec::new(),
    }
}
#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
pub async fn x_login(q: &SocialPlatforms) {
    let start = q.x.sign_in_begin().unwrap();
    let u = reqwest::Url::parse(&start.authorization_url).unwrap();
    let p: std::collections::HashMap<_, _> = u.query_pairs().into_owned().collect();
    assert_eq!(p["code_challenge_method"], "S256");
    assert_eq!(p["client_id"], "test-client");
    assert_eq!(p["response_type"], "code");
    assert_eq!(
        p["scope"],
        "tweet.read tweet.write users.read media.write offline.access"
    );
    assert_eq!(
        q.connection(syncloom_core::Channel::X).unwrap().status,
        syncloom_core::ConnectionStatus::AwaitingConsent
    );
    let cb = format!("{}?code=test-code&state={}", p["redirect_uri"], p["state"]);
    reqwest::get(cb).await.unwrap();
    q.x.sign_in_wait().await.unwrap();
}
#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
pub async fn threads_login(q: &SocialPlatforms) {
    let start = q.threads.sign_in_begin().unwrap();
    let u = reqwest::Url::parse(&start.authorization_url).unwrap();
    let pairs: std::collections::HashMap<_, _> = u.query_pairs().into_owned().collect();
    assert_eq!(pairs["client_id"], "test-client");
    assert_eq!(pairs["response_type"], "code");
    assert_eq!(pairs["redirect_uri"], "https://redirect.invalid/callback");
    assert_eq!(pairs["scope"], "threads_basic,threads_content_publish");
    assert_eq!(
        q.connection(syncloom_core::Channel::Threads)
            .unwrap()
            .status,
        syncloom_core::ConnectionStatus::AwaitingConsent
    );
    let state = u
        .query_pairs()
        .find(|(k, _)| k == "state")
        .unwrap()
        .1
        .into_owned();
    q.threads
        .complete(&format!(
            "https://redirect.invalid/callback?code=test-code&state={state}#_"
        ))
        .await
        .unwrap();
}
