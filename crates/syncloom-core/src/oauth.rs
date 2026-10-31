//! Shared loopback OAuth boundary. Platform adapters own registration and token exchange.
use crate::{Clock, Secret, StudioError, Token};
use axum::{
    Router,
    extract::{Query, Request, State},
    http::{StatusCode, header},
    middleware,
    response::{IntoResponse, Response},
    routing::get,
};
use base64::Engine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::oneshot;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RedirectUrl {
    url: String,
    bind: SocketAddr,
    path: String,
}
impl RedirectUrl {
    pub fn as_str(&self) -> &str {
        &self.url
    }
    pub fn bind_addr(&self) -> SocketAddr {
        self.bind
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub(crate) fn parse(raw: &str) -> Option<Self> {
        let url = crate::config::plain_url(raw)?;
        if url.scheme() != "http"
            || !matches!(url.host_str(), Some("127.0.0.1") | Some("localhost"))
        {
            return None;
        }
        let port = url.port()?;
        if port == 0 {
            return None;
        }
        Some(Self {
            url: url.to_string(),
            bind: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port),
            path: url.path().into(),
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionStatus {
    SignedOut,
    AwaitingConsent,
    SignedIn,
    Expired,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Connection {
    pub status: ConnectionStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}
impl Connection {
    pub(crate) fn from_token(token: Option<Token>, now: &str, pending: bool) -> Self {
        if pending {
            return Self {
                status: ConnectionStatus::AwaitingConsent,
                expires_at: None,
                scope: None,
            };
        }
        match token {
            None => Self {
                status: ConnectionStatus::SignedOut,
                expires_at: None,
                scope: None,
            },
            Some(token) => Self {
                status: if token.is_expired_at(now) {
                    ConnectionStatus::Expired
                } else {
                    ConnectionStatus::SignedIn
                },
                expires_at: token.expires_at,
                scope: Some(token.scope),
            },
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignInStart {
    pub status: ConnectionStatus,
    pub authorization_url: String,
    pub expires_at: String,
}
#[derive(Deserialize)]
struct Callback {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}
struct Pending {
    state: Secret,
    verifier: Option<Secret>,
    deadline: Instant,
    receiver: Mutex<Option<oneshot::Receiver<Callback>>>,
    listener: Mutex<Option<tokio::task::JoinHandle<()>>>,
}
impl Pending {
    fn stop(&self) -> Option<tokio::task::JoinHandle<()>> {
        let task = self.listener.lock().unwrap().take();
        if let Some(task) = &task {
            task.abort();
        }
        task
    }
}
impl Drop for Pending {
    fn drop(&mut self) {
        self.stop();
    }
}
#[derive(Default)]
pub(crate) struct Session {
    pending: Mutex<Option<Arc<Pending>>>,
}
impl Session {
    pub fn cancel(&self) {
        if let Some(pending) = self.pending.lock().unwrap().take() {
            pending.stop();
        }
    }
    pub fn active(&self) -> bool {
        self.pending
            .lock()
            .unwrap()
            .as_ref()
            .is_some_and(|p| Instant::now() < p.deadline)
    }
    pub fn begin(
        &self,
        redirect: &RedirectUrl,
        mut url: reqwest::Url,
        pkce: bool,
        clock: &dyn Clock,
        deadline: Duration,
    ) -> Result<SignInStart, StudioError> {
        // Hold one guard through bind and reservation, including across OS threads.
        let mut slot = self.pending.lock().unwrap();
        if slot.as_ref().is_some_and(|p| Instant::now() < p.deadline) {
            return Err(StudioError::SignInInProgress);
        }
        if let Some(old) = slot.take() {
            old.stop();
        }
        let listener = std::net::TcpListener::bind(redirect.bind_addr())
            .map_err(|_| StudioError::SignInRejected)?;
        listener
            .set_nonblocking(true)
            .map_err(|_| StudioError::SignInRejected)?;
        let listener =
            tokio::net::TcpListener::from_std(listener).map_err(|_| StudioError::SignInRejected)?;
        let state = random_token();
        let verifier = pkce.then(random_token);
        url.query_pairs_mut().append_pair("state", state.expose());
        if let Some(v) = &verifier {
            url.query_pairs_mut()
                .append_pair("code_challenge_method", "S256")
                .append_pair("code_challenge", &s256(v.expose()));
        }
        let (sender, receiver) = oneshot::channel();
        let app = callback_app(redirect, sender);
        let task = tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        *slot = Some(Arc::new(Pending {
            state,
            verifier,
            deadline: Instant::now() + deadline,
            receiver: Mutex::new(Some(receiver)),
            listener: Mutex::new(Some(task)),
        }));
        Ok(SignInStart {
            status: ConnectionStatus::AwaitingConsent,
            authorization_url: url.to_string(),
            expires_at: crate::clock::rfc3339(clock.now() + deadline),
        })
    }
    pub async fn wait(&self) -> Result<(String, Option<Secret>), StudioError> {
        let pending = self
            .pending
            .lock()
            .unwrap()
            .clone()
            .ok_or(StudioError::SignInNotStarted)?;
        let receiver = pending
            .receiver
            .lock()
            .unwrap()
            .take()
            .ok_or(StudioError::SignInInProgress)?;
        let result = match tokio::time::timeout(
            pending.deadline.saturating_duration_since(Instant::now()),
            receiver,
        )
        .await
        {
            Ok(Ok(callback)) => accepted(&pending, callback),
            _ => Err(StudioError::SignInTimedOut),
        };
        // Give the fixed page time to flush before closing the listener.
        tokio::time::sleep(Duration::from_millis(100)).await;
        // Abort is only a request. Await completion before promising that the port is free.
        if let Some(task) = pending.stop() {
            let _ = task.await;
        }
        let mut slot = self.pending.lock().unwrap();
        if slot.as_ref().is_some_and(|p| Arc::ptr_eq(p, &pending)) {
            *slot = None;
        }
        result.map(|code| (code, pending.verifier.clone()))
    }
}
/// One owner of callback routing, Host validation and fixed error responses.
fn callback_app(redirect: &RedirectUrl, sender: oneshot::Sender<Callback>) -> Router {
    let host = reqwest::Url::parse(redirect.as_str()).expect("checked redirect");
    let expected = format!("{}:{}", host.host_str().unwrap(), host.port().unwrap());
    Router::new().route(redirect.path(),get(callback))
        .fallback(|| async { fixed_error(StatusCode::NOT_FOUND) })
        .method_not_allowed_fallback(|| async { fixed_error(StatusCode::METHOD_NOT_ALLOWED) })
        .layer(middleware::from_fn(move |request:Request,next:middleware::Next| {
        let expected=expected.clone();
        async move {
            let mut hosts=request.headers().get_all(header::HOST).iter();
            if matches!((hosts.next(),hosts.next()),(Some(h),None) if h.as_bytes()==expected.as_bytes()) { next.run(request).await }
            else { fixed_error(StatusCode::FORBIDDEN) }
        }
        })).with_state(Arc::new(Mutex::new(Some(sender))))
}
fn accepted(p: &Pending, callback: Callback) -> Result<String, StudioError> {
    if !callback
        .state
        .as_deref()
        .is_some_and(|s| constant_time_eq(s.as_bytes(), p.state.expose().as_bytes()))
    {
        return Err(StudioError::SignInStateMismatch);
    }
    if callback.error.is_some() {
        return Err(StudioError::SignInDenied);
    }
    callback
        .code
        .filter(|c| !c.is_empty())
        .ok_or(StudioError::SignInDenied)
}
fn fixed_error(status: StatusCode) -> Response {
    (status,axum::Json(serde_json::json!({"error":{"code":"sign_in_rejected","message":"Sign-in callback refused."}}))).into_response()
}
async fn callback(
    State(sender): State<Arc<Mutex<Option<oneshot::Sender<Callback>>>>>,
    query: Result<Query<Callback>, axum::extract::rejection::QueryRejection>,
) -> Response {
    let Ok(Query(callback)) = query else {
        return fixed_error(StatusCode::BAD_REQUEST);
    };
    if let Some(sender) = sender.lock().unwrap().take() {
        let _ = sender.send(callback);
    }
    ([(header::CONTENT_TYPE,"text/html; charset=utf-8")],"<!doctype html><html><head><title>Syncloom</title></head><body><p>Sign-in answer received. Return to the studio.</p></body></html>").into_response()
}
fn random_token() -> Secret {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("OS randomness");
    Secret::new(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes))
}
fn s256(verifier: &str) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Bounded JSON answers shared by adapters; never include provider bodies in errors.
pub(crate) async fn read_json<T: serde::de::DeserializeOwned>(
    mut response: reqwest::Response,
) -> Option<T> {
    if !response.status().is_success() {
        return None;
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.ok()? {
        if bytes.len() + chunk.len() > 64 * 1024 {
            return None;
        }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes).ok()
}
