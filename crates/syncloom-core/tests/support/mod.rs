//! Shared test support for the core crate.
#![allow(dead_code)]
pub mod fake_claude;
pub mod fake_linkedin;
pub mod memory_store;
pub mod scripted;
pub mod scripted_platform;

use std::sync::Arc;
use syncloom_core::{
    AuthFlow, BaseUrl, FixedClock, Linkedin, LinkedinConfig, ModelProvider, Secret, Settings,
    Store, Studio, Token, TokenStore, redirect_url,
};
use tempfile::TempDir;
use time::macros::datetime;

/// 2026-10-30T18:00:00Z, the fixed instant of every test.
pub fn fixed_clock() -> Arc<FixedClock> {
    Arc::new(FixedClock(datetime!(2026-10-30 18:00:00 UTC)))
}

pub const FIXED_AT: &str = "2026-10-30T18:00:00Z";
/// FIXED_AT plus LinkedIn's 60-day `expires_in` of 5184000 seconds.
pub const TOKEN_EXPIRES_AT: &str = "2026-12-29T18:00:00Z";
pub const CLIENT_SECRET_CANARY: &str = "client-secret-axolotl";

/// A studio over a fresh temporary directory.
pub fn studio(dir: &TempDir, provider: Arc<dyn ModelProvider>, settings: Settings) -> Studio {
    Studio::new(
        Store::open(dir.path()).unwrap(),
        provider,
        fixed_clock(),
        settings,
    )
}

pub fn fast() -> Settings {
    Settings {
        provider_deadline: std::time::Duration::from_millis(200),
        platform_deadline: std::time::Duration::from_millis(200),
        sign_in_deadline: std::time::Duration::from_millis(300),
    }
}

/// A port nobody listens on right now, for a fixed loopback redirect URL.
pub fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// A LinkedIn configuration pointing both bases at the fake, with or without app credentials.
pub fn linkedin_config(
    fake_base: &str,
    port: u16,
    flow: AuthFlow,
    configured: bool,
) -> LinkedinConfig {
    LinkedinConfig {
        client_id: configured.then(|| "fake-client-id".to_owned()),
        client_secret: configured.then(|| Secret::new(CLIENT_SECRET_CANARY)),
        redirect: redirect_url(Some(&format!("http://127.0.0.1:{port}/linkedin/callback")))
            .unwrap(),
        auth_base: BaseUrl::parse("SYNCLOOM_LINKEDIN_AUTH_BASE_URL", Some(fake_base), "").unwrap(),
        api_base: BaseUrl::parse("SYNCLOOM_LINKEDIN_API_BASE_URL", Some(fake_base), "").unwrap(),
        flow,
    }
}

pub fn linkedin(fake_base: &str, store: Arc<dyn TokenStore>, configured: bool) -> Arc<Linkedin> {
    Arc::new(Linkedin::new(
        linkedin_config(fake_base, free_port(), AuthFlow::Standard, configured),
        store,
        fixed_clock(),
        fast().sign_in_deadline,
    ))
}

/// A token as a completed sign-in would have stored it.
pub fn stored_token(expires_at: &str) -> Token {
    Token {
        access_token: Secret::new(fake_linkedin::DEFAULT_TOKEN),
        expires_at: expires_at.to_owned(),
        member_id: fake_linkedin::MEMBER_ID.to_owned(),
        scope: "openid,profile,w_member_social".to_owned(),
        obtained_at: FIXED_AT.to_owned(),
    }
}

/// The browser: follows the authorization URL through the fake's redirect to the callback
/// listener and returns the callback page.
pub async fn browser(url: &str) -> reqwest::Response {
    reqwest::Client::builder()
        .no_proxy()
        .build()
        .unwrap()
        .get(url)
        .send()
        .await
        .unwrap()
}

/// A thought with a canary word that must never reach logs or events.
pub const CANARY: &str = "zebrafish";
pub const THOUGHT: &str =
    "  Agents write better when the  spec names one owner per rule; zebrafish.  ";
pub const NORMALIZED: &str =
    "Agents write better when the spec names one owner per rule; zebrafish.";

pub fn events(dir: &TempDir) -> Vec<serde_json::Value> {
    let path = dir.path().join("provenance").join("events.jsonl");
    if !path.exists() {
        return Vec::new();
    }
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

pub fn events_raw(dir: &TempDir) -> String {
    std::fs::read_to_string(dir.path().join("provenance").join("events.jsonl")).unwrap_or_default()
}

pub fn files(dir: &TempDir, sub: &str) -> Vec<String> {
    let path = dir.path().join(sub);
    if !path.exists() {
        return Vec::new();
    }
    let mut names: Vec<String> = std::fs::read_dir(path)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// The one intent in the outbox, parsed.
pub fn intent(dir: &TempDir) -> serde_json::Value {
    let names = files(dir, "outbox");
    assert_eq!(names.len(), 1, "expected exactly one intent: {names:?}");
    serde_json::from_str(
        &std::fs::read_to_string(dir.path().join("outbox").join(&names[0])).unwrap(),
    )
    .unwrap()
}

/// Every file under the data directory as one string, for canary checks.
pub fn all_data(dir: &TempDir) -> String {
    let mut out = String::new();
    for sub in ["pieces", "outbox", "provenance"] {
        for name in files(dir, sub) {
            out.push_str(&std::fs::read_to_string(dir.path().join(sub).join(name)).unwrap());
        }
    }
    out
}
