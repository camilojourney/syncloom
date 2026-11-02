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

/// Common authenticated subject shape; each adapter owns its authority and ID validation.
#[derive(Deserialize)]
pub(crate) struct Subject {
    pub id: String,
}

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
/// Stable refusal when the effective grant cannot post. Not a second capability registry.
pub const POSTING_PERMISSION_MISSING: &str = "posting_permission_missing";
pub const POSTING_PERMISSION_MESSAGE: &str = "posting permission missing.";
pub const CANNOT_POST_MESSAGE: &str = "connected, cannot post: posting permission missing.";
/// Existing sign-in operation. Not a new credential-management page.
pub const RECONNECT_ACTION: &str = "sign_in";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionStatus {
    SignedOut,
    AwaitingConsent,
    SignedIn,
    Expired,
    /// A credential is stored, but it does not grant posting.
    CannotPost,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Connection {
    pub status: ConnectionStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Existing reconnect operation, present only when posting permission is missing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
}
impl Connection {
    fn absent(status: ConnectionStatus) -> Self {
        Self {
            status,
            expires_at: None,
            scope: None,
            reason: None,
            message: None,
            action: None,
        }
    }
    /// Readiness uses the credential owner's action policy, never requested scopes.
    pub(crate) fn from_token(
        token: Option<Token>,
        now: &str,
        pending: bool,
        permission: impl Fn(&Token) -> Result<(), crate::credentials::GrantError>,
    ) -> Self {
        if pending {
            return Self::absent(ConnectionStatus::AwaitingConsent);
        }
        let Some(token) = token else {
            return Self::absent(ConnectionStatus::SignedOut);
        };
        let expired = token.is_expired_at(now);
        let denied = (!expired).then(|| permission(&token).err()).flatten();
        Self {
            status: if expired {
                ConnectionStatus::Expired
            } else if denied.is_some() {
                ConnectionStatus::CannotPost
            } else {
                ConnectionStatus::SignedIn
            },
            expires_at: token.expires_at,
            scope: Some(token.scope),
            reason: denied.map(|e| e.error().code().to_owned()),
            message: denied.map(|e| format!("connected, cannot post: {}", e.error().message())),
            action: denied.map(|_| RECONNECT_ACTION.to_owned()),
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
    generation: u64,
    state: Secret,
    verifier: Option<Secret>,
    deadline: Instant,
    receiver: Mutex<Option<oneshot::Receiver<Callback>>>,
    listener: Mutex<Option<tokio::task::JoinHandle<()>>>,
}
impl Pending {
    #[expect(
        clippy::unwrap_used,
        reason = "Poisoned OAuth lifecycle state cannot be safely reused; preserve fail-closed behavior."
    )]
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
pub(crate) struct Session {
    pending: Mutex<Option<Arc<Pending>>>,
    redirect: RedirectUrl,
    credentials: Arc<crate::credential_session::CredentialSession>,
    deadline: Duration,
}
impl Session {
    pub fn new(
        redirect: RedirectUrl,
        store: Arc<dyn crate::TokenStore>,
        account: String,
        clock: Arc<dyn Clock>,
        deadline: Duration,
    ) -> Self {
        Self::with_credentials(
            redirect,
            Arc::new(crate::credential_session::CredentialSession::new(
                account, store, clock,
            )),
            deadline,
        )
    }
    pub fn with_credentials(
        redirect: RedirectUrl,
        credentials: Arc<crate::credential_session::CredentialSession>,
        deadline: Duration,
    ) -> Self {
        Self {
            pending: Mutex::new(None),
            redirect,
            credentials,
            deadline,
        }
    }
    pub fn deadline(&self) -> Duration {
        self.deadline
    }
    pub fn now(&self) -> time::OffsetDateTime {
        self.credentials.clock.now()
    }
    pub fn now_rfc3339(&self) -> String {
        self.credentials.clock.now_rfc3339()
    }
    pub fn save_token(&self, generation: u64, token: &Token) -> Result<(), StudioError> {
        self.credentials.commit_token(generation, token)
    }
    pub fn usable_token(&self) -> Result<Token, crate::PostFailure> {
        self.credentials.usable()
    }
    pub fn usable_for_post(
        &self,
        channel: crate::Channel,
        image: bool,
    ) -> Result<Token, crate::PostFailure> {
        self.credentials.usable_for_post(channel, image)
    }
    pub fn connection_for(
        &self,
        channel: crate::Channel,
        image: bool,
    ) -> Result<Connection, StudioError> {
        self.credentials
            .connection_for(self.active(), channel, image)
    }
    pub fn connection_if(
        &self,
        permission: impl Fn(&Token) -> Result<(), crate::credentials::GrantError>,
    ) -> Result<Connection, StudioError> {
        self.credentials.connection_if(self.active(), permission)
    }
    pub fn sign_out(&self) -> Result<Connection, StudioError> {
        self.cancel();
        self.credentials.clear()
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Poisoned OAuth lifecycle state cannot be safely reused; preserve fail-closed behavior."
    )]
    pub fn cancel(&self) {
        if let Some(pending) = self.pending.lock().unwrap().take() {
            pending.stop();
        }
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Poisoned OAuth lifecycle state cannot be safely reused; preserve fail-closed behavior."
    )]
    pub fn active(&self) -> bool {
        self.pending
            .lock()
            .unwrap()
            .as_ref()
            .is_some_and(|p| Instant::now() < p.deadline)
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Poisoned OAuth lifecycle state cannot be safely reused; preserve fail-closed behavior."
    )]
    pub fn begin(&self, mut url: reqwest::Url, pkce: bool) -> Result<SignInStart, StudioError> {
        let redirect = &self.redirect;
        let deadline = self.deadline;
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
        self.credentials.invalidate();
        *slot = Some(Arc::new(Pending {
            generation: self.credentials.generation(),
            state,
            verifier,
            deadline: Instant::now() + deadline,
            receiver: Mutex::new(Some(receiver)),
            listener: Mutex::new(Some(task)),
        }));
        Ok(SignInStart {
            status: ConnectionStatus::AwaitingConsent,
            authorization_url: url.to_string(),
            expires_at: crate::clock::rfc3339(self.now() + deadline),
        })
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Poisoned OAuth lifecycle state cannot be safely reused; preserve fail-closed behavior."
    )]
    pub async fn wait(&self) -> Result<(String, Option<Secret>, u64), StudioError> {
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
        result.map(|code| (code, pending.verifier.clone(), pending.generation))
    }
}
/// One owner of callback routing, Host validation and fixed error responses.
#[expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "RedirectUrl guarantees a parsed loopback URL with an explicit non-default port and host."
)]
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
        .is_some_and(|s| p.state.matches(s))
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
#[expect(
    clippy::unwrap_used,
    reason = "Poisoned OAuth lifecycle state cannot be safely reused; preserve fail-closed behavior."
)]
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
#[expect(
    clippy::expect_used,
    reason = "OS randomness failure is unrecoverable; never substitute predictable OAuth state or verifier bytes."
)]
pub(crate) fn random_token() -> Secret {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("OS randomness");
    Secret::new(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes))
}
fn s256(verifier: &str) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

/// Bounded JSON answers shared by adapters; never include provider bodies in errors.
pub(crate) async fn exchange_json<T: serde::de::DeserializeOwned>(
    builder: reqwest::RequestBuilder,
) -> Result<T, StudioError> {
    let response = builder
        .send()
        .await
        .map_err(|_| StudioError::SignInRejected)?;
    let value: serde_json::Value = read_json(response)
        .await
        .ok_or(StudioError::SignInRejected)?;
    if value.get("error").is_some() {
        return Err(StudioError::SignInRejected);
    }
    serde_json::from_value(value).map_err(|_| StudioError::SignInRejected)
}

pub(crate) async fn read_json<T: serde::de::DeserializeOwned>(
    response: reqwest::Response,
) -> Option<T> {
    if !response.status().is_success() {
        return None;
    }
    read_json_body(response).await
}

/// Bounded body decoding for APIs whose documented errors are JSON, too.
pub(crate) async fn read_json_body<T: serde::de::DeserializeOwned>(
    mut response: reqwest::Response,
) -> Option<T> {
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.ok()? {
        if bytes.len() + chunk.len() > 64 * 1024 {
            return None;
        }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes).ok()
}
