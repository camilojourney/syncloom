//! Shared support for the server tests: a server over the scripted provider in a temporary
//! directory, raw socket requests for Host header control, and body helpers.
#![allow(dead_code)]
#[path = "../../../syncloom-core/tests/support/scripted.rs"]
pub mod scripted;

use serde_json::{Value, json};
use std::{net::SocketAddr, sync::Arc};
use syncloom_core::{FixedClock, Settings, Store, Studio};
use tempfile::TempDir;
use time::macros::datetime;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[allow(unused_imports)]
pub use scripted::{Reply, Scripted, text};

pub const FIXTURE_ID: &str = "fixture-request-id";
pub const CANARY: &str = "zebrafish";
pub const THOUGHT: &str = "Agents write better when the spec names one owner per rule; zebrafish.";

pub struct TestServer {
    pub addr: SocketAddr,
    pub dir: TempDir,
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
    }
}

pub async fn start_with(provider: Arc<Scripted>, settings: Settings) -> TestServer {
    let dir = TempDir::new().unwrap();
    let studio = Studio::new(
        Store::open(dir.path()).unwrap(),
        provider,
        Arc::new(FixedClock(datetime!(2026-10-30 18:00:00 UTC))),
        settings,
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        syncloom_server::serve(listener, Arc::new(studio))
            .await
            .unwrap()
    });
    TestServer { addr, dir }
}

pub async fn start(provider: Arc<Scripted>) -> TestServer {
    start_with(provider, Settings::default()).await
}

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

pub async fn get(server: &TestServer, path: &str) -> Answer {
    answer(client().get(server.url(path)).send().await.unwrap()).await
}

pub async fn send_json(
    server: &TestServer,
    method: reqwest::Method,
    path: &str,
    body: Value,
) -> Answer {
    answer(
        client()
            .request(method, server.url(path))
            .json(&body)
            .send()
            .await
            .unwrap(),
    )
    .await
}

pub async fn post(server: &TestServer, path: &str, body: Value) -> Answer {
    send_json(server, reqwest::Method::POST, path, body).await
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

pub fn id_of(piece: &Value) -> String {
    piece["piece_id"].as_str().unwrap().to_owned()
}

pub fn revision_of(piece: &Value) -> String {
    piece["revision"].as_str().unwrap().to_owned()
}

pub async fn approve(server: &TestServer, piece: &Value) -> Answer {
    patch(
        server,
        &format!("/v1/pieces/{}", id_of(piece)),
        json!({"status": "approved", "expected_revision": revision_of(piece)}),
    )
    .await
}

/// Write one raw HTTP/1.1 request and read the whole response, for Host header control.
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

pub fn fixture(name: &str) -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts/studio-api")
        .join(name);
    serde_json::from_str(
        &std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}")),
    )
    .unwrap()
}

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
