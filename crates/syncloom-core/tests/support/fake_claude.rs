//! A fake Claude Messages endpoint on loopback: scripted answers, recorded requests.
//! Included by the provider tests, the CLI tests and the `fake_claude` example.
#![allow(dead_code)]
use axum::{
    Router,
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

#[derive(Clone, Debug)]
pub enum Mode {
    /// Answer this text with usage 321 in, 123 out.
    Ok(String),
    /// Answer an empty text block.
    Empty,
    /// Answer 200 with a body that has no text block.
    NoText,
    /// Wait a long time before answering.
    Slow,
    /// Answer this status with a JSON error body.
    Status(u16),
}

impl Mode {
    /// Parse the example's `--mode` value.
    pub fn parse(value: &str, text: &str) -> Result<Mode, String> {
        Ok(match value {
            "ok" => Mode::Ok(text.to_owned()),
            "empty" => Mode::Empty,
            "notext" => Mode::NoText,
            "slow" => Mode::Slow,
            status => Mode::Status(
                status
                    .parse()
                    .map_err(|_| format!("unknown mode {value}"))?,
            ),
        })
    }
}

/// One recorded request: whether a key was sent (never its value), the version header, the body.
#[derive(Clone, Debug)]
pub struct Recorded {
    pub has_api_key: bool,
    pub api_key_is: Option<String>,
    pub anthropic_version: Option<String>,
    pub body: serde_json::Value,
}

#[derive(Clone)]
struct Shared {
    mode: Arc<Mutex<Mode>>,
    requests: Arc<Mutex<Vec<Recorded>>>,
}

pub struct FakeClaude {
    pub addr: SocketAddr,
    shared: Shared,
}

impl FakeClaude {
    pub fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }
    pub fn set(&self, mode: Mode) {
        *self.shared.mode.lock().unwrap() = mode;
    }
    pub fn requests(&self) -> Vec<Recorded> {
        self.shared.requests.lock().unwrap().clone()
    }
}

pub async fn start(mode: Mode) -> FakeClaude {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let shared = Shared {
        mode: Arc::new(Mutex::new(mode)),
        requests: Arc::new(Mutex::new(Vec::new())),
    };
    let app = Router::new()
        .route("/v1/messages", post(messages))
        .with_state(shared.clone());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    FakeClaude { addr, shared }
}

async fn messages(State(shared): State<Shared>, headers: HeaderMap, body: Bytes) -> Response {
    let key = headers
        .get("x-api-key")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    shared.requests.lock().unwrap().push(Recorded {
        has_api_key: key.is_some(),
        api_key_is: key,
        anthropic_version: headers
            .get("anthropic-version")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned),
        body: serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null),
    });
    let mode = shared.mode.lock().unwrap().clone();
    match mode {
        Mode::Ok(text) => answer(&text),
        Mode::Empty => answer(""),
        Mode::NoText => axum::Json(serde_json::json!({"id": "msg_1", "type": "message", "content": []}))
            .into_response(),
        Mode::Slow => {
            tokio::time::sleep(Duration::from_secs(30)).await;
            answer("late")
        }
        Mode::Status(status) => (
            StatusCode::from_u16(status).unwrap(),
            axum::Json(serde_json::json!({"type": "error", "error": {"type": "fake", "message": "fake provider body"}})),
        )
            .into_response(),
    }
}

fn answer(text: &str) -> Response {
    axum::Json(serde_json::json!({
        "id": "msg_fake",
        "type": "message",
        "role": "assistant",
        "model": "fake",
        "content": [{"type": "text", "text": text}],
        "stop_reason": "end_turn",
        "usage": {"input_tokens": 321, "output_tokens": 123}
    }))
    .into_response()
}
