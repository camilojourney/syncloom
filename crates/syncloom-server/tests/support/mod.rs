//! Shared support for the server tests: a server over the scripted provider in a temporary
//! directory, raw socket requests for Host header control, and body helpers.
#![allow(
    dead_code,
    reason = "Each integration target uses a different subset of the shared test fixtures."
)]
#[path = "../../../syncloom-core/tests/support/fake_linkedin.rs"]
pub mod fake_linkedin;
#[path = "../../../syncloom-core/tests/support/memory_store.rs"]
pub mod memory_store;
#[path = "../../../syncloom-core/tests/support/scripted.rs"]
pub mod scripted;

use memory_store::MemoryStore;
use scripted::Scripted;
use serde_json::{Value, json};
use std::{net::SocketAddr, sync::Arc};
use syncloom_core::{
    AuthFlow, BaseUrl, FixedClock, Linkedin, LinkedinConfig, Secret, Settings, Store, Studio,
    Token, redirect_url,
};
use tempfile::TempDir;
use time::macros::datetime;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub const FIXTURE_ID: &str = "fixture-request-id";
pub const CANARY: &str = "zebrafish";
pub const THOUGHT: &str = "Agents write better when the spec names one owner per rule; zebrafish.";

pub struct TestServer {
    pub addr: SocketAddr,
    pub dir: TempDir,
    pub token: String,
}

impl TestServer {
    pub fn url(&self, path: &str) -> String {
        format!("http://{}{path}", self.addr)
    }
    pub fn host(&self) -> String {
        self.addr.to_string()
    }
}

pub fn fast() -> Settings {
    Settings {
        provider_deadline: std::time::Duration::from_millis(200),
        platform_deadline: std::time::Duration::from_millis(200),
        sign_in_deadline: std::time::Duration::from_millis(300),
    }
}

pub const CLIENT_SECRET_CANARY: &str = "client-secret-axolotl";

pub fn fixed_clock() -> Arc<FixedClock> {
    Arc::new(FixedClock(datetime!(2026-10-30 18:00:00 UTC)))
}

#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
pub fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// A token as a completed sign-in would have stored it.
pub fn stored_token(expires_at: &str) -> Token {
    Token {
        access_token: Secret::new(fake_linkedin::DEFAULT_TOKEN),
        expires_at: Some(expires_at.to_owned()),
        member_id: fake_linkedin::MEMBER_ID.to_owned(),
        scope: "openid,profile,w_member_social".to_owned(),
        obtained_at: "2026-10-30T18:00:00Z".to_owned(),
    }
}

/// A server whose studio has the LinkedIn integration over the fake, with app credentials when
/// `configured` and delivery granted when `granted`.
#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
pub async fn start_linkedin(
    provider: Arc<Scripted>,
    settings: Settings,
    fake: &fake_linkedin::FakeLinkedin,
    store: Arc<MemoryStore>,
    configured: bool,
    granted: bool,
) -> TestServer {
    let config = LinkedinConfig {
        client_id: configured.then(|| "fake-client-id".to_owned()),
        client_secret: configured.then(|| Secret::new(CLIENT_SECRET_CANARY)),
        redirect: redirect_url(Some(&format!(
            "http://127.0.0.1:{}/linkedin/callback",
            free_port()
        )))
        .unwrap(),
        auth_base: BaseUrl::parse("A", Some(&fake.base_url()), "").unwrap(),
        api_base: BaseUrl::parse("B", Some(&fake.base_url()), "").unwrap(),
        flow: AuthFlow::Standard,
    };
    let linkedin = Arc::new(Linkedin::new(
        config,
        store,
        fixed_clock(),
        settings.sign_in_deadline,
    ));
    start_studio(provider, settings, |studio| {
        let studio = studio.with_linkedin(linkedin.clone());
        if granted {
            studio.with_delivery("linkedin", linkedin.clone())
        } else {
            studio
        }
    })
    .await
}

pub async fn start_with(provider: Arc<Scripted>, settings: Settings) -> TestServer {
    start_studio(provider, settings, |studio| studio).await
}

#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
pub async fn start_studio(
    provider: Arc<Scripted>,
    settings: Settings,
    shape: impl FnOnce(Studio) -> Studio,
) -> TestServer {
    let dir = TempDir::new().unwrap();
    let studio = shape(Studio::new(
        Store::open(dir.path()).unwrap(),
        provider,
        fixed_clock(),
        settings,
    ));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        syncloom_server::serve(listener, Arc::new(studio))
            .await
            .unwrap()
    });
    let mut server = TestServer {
        addr,
        dir,
        token: String::new(),
    };
    let session = get(&server, "/v1/session").await;
    assert_eq!(session.status, 200, "{}", session.raw);
    server.token = session.body["token"].as_str().unwrap().to_owned();
    server
}

pub async fn start(provider: Arc<Scripted>) -> TestServer {
    start_with(provider, Settings::default()).await
}

#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
pub fn client() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().build().unwrap()
}

/// A response as (status, headers, JSON body or Null).
#[derive(Debug, Clone)]
pub struct Answer {
    pub status: u16,
    pub headers: reqwest::header::HeaderMap,
    pub body: Value,
    pub raw: String,
}

impl Answer {
    pub fn code(&self) -> &str {
        self.body["error"]["code"].as_str().unwrap_or("")
    }
    pub fn request_id(&self) -> &str {
        self.headers
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
    }
    /// The shape the fixtures use.
    pub fn as_fixture(&self) -> Value {
        json!({"status": self.status, "body": self.body})
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
pub async fn answer(response: reqwest::Response) -> Answer {
    let status = response.status().as_u16();
    let headers = response.headers().clone();
    let raw = response.text().await.unwrap();
    let body = serde_json::from_str(&raw).unwrap_or(Value::Null);
    Answer {
        status,
        headers,
        body,
        raw,
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
pub async fn get(server: &TestServer, path: &str) -> Answer {
    answer(client().get(server.url(path)).send().await.unwrap()).await
}

#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
pub async fn send_json(
    server: &TestServer,
    method: reqwest::Method,
    path: &str,
    body: Value,
) -> Answer {
    let request = client()
        .request(method, server.url(path))
        .header("x-syncloom-token", &server.token);
    let request = if body.is_null() {
        request
    } else {
        request.json(&body)
    };
    answer(request.send().await.unwrap()).await
}

pub async fn post(server: &TestServer, path: &str, body: Value) -> Answer {
    send_json(server, reqwest::Method::POST, path, body).await
}

/// A POST with no body, as the sign-in operation takes.
#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
pub async fn post_empty(server: &TestServer, path: &str) -> Answer {
    answer(
        client()
            .post(server.url(path))
            .header("x-syncloom-token", &server.token)
            .send()
            .await
            .unwrap(),
    )
    .await
}

pub async fn patch(server: &TestServer, path: &str, body: Value) -> Answer {
    send_json(server, reqwest::Method::PATCH, path, body).await
}

pub async fn create(server: &TestServer, text: &str) -> Answer {
    post(
        server,
        "/v1/thoughts",
        json!({"text": text, "channels": ["linkedin_text"]}),
    )
    .await
}

/// The first piece of a 201 answer.
pub fn piece_of(answer: &Answer) -> Value {
    assert_eq!(answer.status, 201, "{}", answer.raw);
    answer.body["pieces"][0].clone()
}

#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
pub fn id_of(piece: &Value) -> String {
    piece["piece_id"].as_str().unwrap().to_owned()
}

#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
pub fn revision_of(piece: &Value) -> String {
    piece["revision"].as_str().unwrap().to_owned()
}

pub async fn approve(server: &TestServer, piece: &Value) -> Answer {
    patch(
        server,
        &format!("/v1/pieces/{}", id_of(piece)),
        json!({"status": "approved", "expected_revision": revision_of(piece),
            "expected_assets": piece.get("assets").map(|a| json!({"image_sha256":a["image_sha256"],"carousel_sha256":a["carousel_sha256"]}))}),
    )
    .await
}

/// Write one raw HTTP/1.1 request and read the whole response, for Host header control.
#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
pub async fn raw(addr: SocketAddr, request: &str) -> Answer {
    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).await.unwrap();
    let text = String::from_utf8_lossy(&bytes).into_owned();
    let (head, body) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
    let status: u16 = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let mut headers = reqwest::header::HeaderMap::new();
    for line in head.lines().skip(1) {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        if let (Ok(name), Ok(value)) = (
            reqwest::header::HeaderName::from_bytes(name.trim().as_bytes()),
            reqwest::header::HeaderValue::from_str(value.trim()),
        ) {
            headers.append(name, value);
        }
    }
    Answer {
        status,
        headers,
        body: serde_json::from_str(body).unwrap_or(Value::Null),
        raw: text,
    }
}

/// A GET request head with the given Host lines (each full `Host: value`), closing the connection.
pub fn head_with_hosts(request_line: &str, host_lines: &[&str], extra: &[(&str, &str)]) -> String {
    let mut request = format!("{request_line}\r\n");
    for host in host_lines {
        request.push_str(host);
        request.push_str("\r\n");
    }
    for (name, value) in extra {
        request.push_str(&format!("{name}: {value}\r\n"));
    }
    request.push_str("Connection: close\r\n\r\n");
    request
}

#[expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
pub fn fixture(name: &str) -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts/studio-api")
        .join(name);
    serde_json::from_str(
        &std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}")),
    )
    .unwrap()
}

#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
pub fn events(dir: &TempDir) -> Vec<Value> {
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

#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
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
