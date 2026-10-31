//! LinkedIn for the founder's personal profile (`linkedin-integration`): configuration, the
//! browser sign-in over a loopback redirect, and the Posts API adapter behind [`Platform`].
//!
//! Read for this adapter (through the Q2 research record, `docs/research/2026-10-30-q2-linkedin-publishing/`):
//! the 3-legged OAuth and native PKCE pages, the Posts API page (version 202610) and the
//! OpenID Connect `userinfo` endpoint. api.linkedin.com has not been called from this repository.
//!
//! Invariants: the access token, the client secret, the authorization code, the `state` and
//! the code verifier never leave this module except through the [`TokenStore`]; exactly one
//! Posts API request is sent per [`Platform::publish`] call; a callback whose `state` differs
//! from the one issued exchanges nothing.
use crate::clock::{Clock, rfc3339};
use crate::config::BaseUrl;
use crate::credentials::{Secret, Token, TokenStore};
use crate::error::StudioError;
use crate::platform::{Platform, PostFailure, PostFuture, PostRequest, Posted};
use axum::{
    Router,
    extract::{Query, State},
    http::header,
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

pub const PLATFORM: &str = "linkedin";
/// The Keychain account name of the LinkedIn token.
pub const ACCOUNT: &str = "linkedin";
pub const CLIENT_ID_ENV: &str = "SYNCLOOM_LINKEDIN_CLIENT_ID";
pub const CLIENT_SECRET_ENV: &str = "SYNCLOOM_LINKEDIN_CLIENT_SECRET";
pub const REDIRECT_URL_ENV: &str = "SYNCLOOM_LINKEDIN_REDIRECT_URL";
pub const AUTH_BASE_ENV: &str = "SYNCLOOM_LINKEDIN_AUTH_BASE_URL";
pub const API_BASE_ENV: &str = "SYNCLOOM_LINKEDIN_API_BASE_URL";
pub const FLOW_ENV: &str = "SYNCLOOM_LINKEDIN_AUTH_FLOW";
pub const DEFAULT_REDIRECT_URL: &str = "http://127.0.0.1:8791/linkedin/callback";
pub const DEFAULT_AUTH_BASE: &str = "https://www.linkedin.com";
pub const DEFAULT_API_BASE: &str = "https://api.linkedin.com";
/// The versioned REST API month; Active until 15 Oct 2027 per the Q2 migration table.
pub const API_VERSION: &str = "202610";
/// `openid profile` for the member id (`userinfo.sub`), `w_member_social` to post.
pub const SCOPES: &str = "openid profile w_member_social";
/// How long the callback listener stays up after the callback so the page reaches the browser.
const CALLBACK_PAGE_GRACE: Duration = Duration::from_millis(100);
/// Largest token or userinfo answer read.
const MAX_ANSWER_BYTES: usize = 64 * 1024;
/// The page the browser lands on. Fixed text: nothing from the callback is echoed.
const CALLBACK_PAGE: &str = "<!doctype html><html><head><meta charset=\"utf-8\"><title>Syncloom</title></head>\
<body><p>Syncloom received LinkedIn's answer. You can close this tab and return to the studio.</p></body></html>";

/// Which authorization endpoint and token proof the sign-in uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthFlow {
    /// `/oauth/v2/authorization`, client secret at the token endpoint. The self-serve default.
    Standard,
    /// `/oauth/native-pkce/authorization`, S256 code challenge, no secret. LinkedIn enables it per app.
    NativePkce,
}

impl AuthFlow {
    /// `standard` (default when unset or blank) or `native_pkce`.
    pub fn parse(value: Option<&str>) -> Result<AuthFlow, &'static str> {
        match value.map(str::trim).filter(|v| !v.is_empty()) {
            None | Some("standard") => Ok(AuthFlow::Standard),
            Some("native_pkce") => Ok(AuthFlow::NativePkce),
            Some(_) => Err("SYNCLOOM_LINKEDIN_AUTH_FLOW must be standard or native_pkce"),
        }
    }
    fn authorization_path(self) -> &'static str {
        match self {
            AuthFlow::Standard => "/oauth/v2/authorization",
            AuthFlow::NativePkce => "/oauth/native-pkce/authorization",
        }
    }
}

/// The registered redirect URL and the loopback socket the callback listener binds for it.
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
}

/// `http://127.0.0.1:<port>/<path>` or `http://localhost:<port>/<path>`, no credentials, query
/// or fragment. Either host binds `127.0.0.1`.
pub fn redirect_url(value: Option<&str>) -> Result<RedirectUrl, &'static str> {
    const REFUSED: &str = "SYNCLOOM_LINKEDIN_REDIRECT_URL must be http://127.0.0.1:<port>/<path> or http://localhost:<port>/<path>";
    let url = crate::config::plain_url(value.unwrap_or(DEFAULT_REDIRECT_URL)).ok_or(REFUSED)?;
    let host_ok = matches!(url.host_str(), Some("127.0.0.1") | Some("localhost"));
    let Some(port) = url.port() else {
        return Err(REFUSED);
    };
    if url.scheme() != "http" || !host_ok {
        return Err(REFUSED);
    }
    Ok(RedirectUrl {
        url: url.as_str().to_owned(),
        bind: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port),
        path: url.path().to_owned(),
    })
}

#[derive(Clone, Debug)]
pub struct LinkedinConfig {
    /// Needed only to sign in; `None` or blank means "not configured" at that moment.
    pub client_id: Option<String>,
    pub client_secret: Option<Secret>,
    pub redirect: RedirectUrl,
    pub auth_base: BaseUrl,
    pub api_base: BaseUrl,
    pub flow: AuthFlow,
}

/// Read the LinkedIn configuration through `get` (normally `std::env::var`).
pub fn linkedin_config(get: impl Fn(&str) -> Option<String>) -> Result<LinkedinConfig, String> {
    let blank_is_none = |value: Option<String>| value.filter(|v| !v.trim().is_empty());
    Ok(LinkedinConfig {
        client_id: blank_is_none(get(CLIENT_ID_ENV)).map(|v| v.trim().to_owned()),
        client_secret: blank_is_none(get(CLIENT_SECRET_ENV)).map(|v| Secret::new(v.trim())),
        redirect: redirect_url(get(REDIRECT_URL_ENV).as_deref())?,
        auth_base: BaseUrl::parse(
            AUTH_BASE_ENV,
            get(AUTH_BASE_ENV).as_deref(),
            DEFAULT_AUTH_BASE,
        )?,
        api_base: BaseUrl::parse(API_BASE_ENV, get(API_BASE_ENV).as_deref(), DEFAULT_API_BASE)?,
        flow: AuthFlow::parse(get(FLOW_ENV).as_deref())?,
    })
}

/// Where the founder sees a post: `https://www.linkedin.com/feed/update/<urn>/`.
pub fn post_url(post_id: &str) -> String {
    format!("https://www.linkedin.com/feed/update/{post_id}/")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionStatus {
    SignedOut,
    AwaitingConsent,
    SignedIn,
    Expired,
}

/// The connection as reported to the founder: never the token.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Connection {
    pub status: ConnectionStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

impl Connection {
    fn signed_out() -> Self {
        Self {
            status: ConnectionStatus::SignedOut,
            expires_at: None,
            scope: None,
        }
    }
}

/// What `POST /v1/linkedin/sign-in` answers: open the URL in a browser before `expires_at`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignInStart {
    pub status: ConnectionStatus,
    pub authorization_url: String,
    pub expires_at: String,
}

/// What the browser brought back to the callback listener.
#[derive(Clone, Debug, Deserialize)]
struct Callback {
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

/// One sign-in waiting for the browser.
struct Pending {
    state: Secret,
    code_verifier: Option<Secret>,
    deadline: Instant,
    /// Taken by the one `sign_in_wait` that consumes the callback.
    callback: Mutex<Option<oneshot::Receiver<Callback>>>,
    /// The listener task; aborting it and awaiting the abort frees the bound port.
    listener: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl Pending {
    /// Stop the listener. The returned handle completes once the port is released.
    fn stop(&self) -> Option<tokio::task::JoinHandle<()>> {
        let handle = self
            .listener
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
        if let Some(handle) = &handle {
            handle.abort();
        }
        handle
    }
}

/// The LinkedIn integration: sign-in, connection and the Posts API adapter.
pub struct Linkedin {
    config: LinkedinConfig,
    store: Arc<dyn TokenStore>,
    clock: Arc<dyn Clock>,
    sign_in_deadline: Duration,
    client: reqwest::Client,
    pending: Mutex<Option<Arc<Pending>>>,
}

impl Linkedin {
    pub fn new(
        config: LinkedinConfig,
        store: Arc<dyn TokenStore>,
        clock: Arc<dyn Clock>,
        sign_in_deadline: Duration,
    ) -> Self {
        Self {
            config,
            store,
            clock,
            sign_in_deadline,
            client: crate::config::http_client(),
            pending: Mutex::new(None),
        }
    }

    /// The registered redirect URL, as it must appear in the developer app.
    pub fn redirect_url(&self) -> &str {
        self.config.redirect.as_str()
    }

    fn pending(&self) -> Option<Arc<Pending>> {
        self.pending
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Forget the pending sign-in if it is `finished` and wait for its listener to close, so
    /// the next sign-in can bind the same port.
    async fn clear_pending(&self, finished: &Arc<Pending>) {
        let handle = {
            let mut slot = self
                .pending
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if slot.as_ref().is_some_and(|p| Arc::ptr_eq(p, finished)) {
                *slot = None;
            }
            finished.stop()
        };
        if let Some(handle) = handle {
            let _ = handle.await;
        }
    }

    /// The app credentials, needed only to sign in.
    fn app(&self) -> Result<(&str, &Secret), StudioError> {
        match (&self.config.client_id, &self.config.client_secret) {
            (Some(id), Some(secret)) => Ok((id.as_str(), secret)),
            _ => Err(StudioError::LinkedinNotConfigured),
        }
    }

    /// Bind the callback listener and return the authorization URL.
    pub fn sign_in_begin(&self) -> Result<SignInStart, StudioError> {
        let (client_id, _) = self.app()?;
        let mut slot = self
            .pending
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(pending) = slot.as_ref() {
            if Instant::now() < pending.deadline {
                return Err(StudioError::SignInInProgress);
            }
            // Nobody waited for this one; its listener goes. Until the runtime drops it the
            // bind below may still fail, which is reported as the port being taken.
            pending.stop();
        }
        let std_listener = std::net::TcpListener::bind(self.config.redirect.bind_addr())
            .map_err(|_| StudioError::LinkedinNotConfigured)?;
        std_listener
            .set_nonblocking(true)
            .map_err(|_| StudioError::LinkedinNotConfigured)?;
        let listener = tokio::net::TcpListener::from_std(std_listener)
            .map_err(|_| StudioError::LinkedinNotConfigured)?;

        let state = random_token();
        let code_verifier = (self.config.flow == AuthFlow::NativePkce).then(random_token);
        let (sender, receiver) = oneshot::channel();
        let app = Router::new()
            .route(self.config.redirect.path(), get(callback))
            .with_state(Arc::new(Mutex::new(Some(sender))));
        let listener = tokio::spawn(async move {
            if let Err(error) = axum::serve(listener, app).await {
                tracing::error!(kind = ?error.kind(), "callback_listener_failed");
            }
        });

        let mut url = reqwest::Url::parse(&format!(
            "{}{}",
            self.config.auth_base.as_str(),
            self.config.flow.authorization_path()
        ))
        .expect("the base URL and a fixed path form a URL");
        {
            let mut query = url.query_pairs_mut();
            query
                .append_pair("response_type", "code")
                .append_pair("client_id", client_id)
                .append_pair("redirect_uri", self.config.redirect.as_str())
                .append_pair("state", state.expose())
                .append_pair("scope", SCOPES);
            if let Some(verifier) = &code_verifier {
                query
                    .append_pair("code_challenge_method", "S256")
                    .append_pair("code_challenge", &s256(verifier.expose()));
            }
        }
        let expires_at = rfc3339(self.clock.now() + self.sign_in_deadline);
        *slot = Some(Arc::new(Pending {
            state,
            code_verifier,
            deadline: Instant::now() + self.sign_in_deadline,
            callback: Mutex::new(Some(receiver)),
            listener: Mutex::new(Some(listener)),
        }));
        Ok(SignInStart {
            status: ConnectionStatus::AwaitingConsent,
            authorization_url: url.to_string(),
            expires_at,
        })
    }

    /// Wait for the browser callback, exchange the code, store the token.
    pub async fn sign_in_wait(&self) -> Result<Connection, StudioError> {
        let pending = self.pending().ok_or(StudioError::SignInNotStarted)?;
        let receiver = pending
            .callback
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
            .ok_or(StudioError::SignInInProgress)?;
        let remaining = pending.deadline.saturating_duration_since(Instant::now());
        let callback = match tokio::time::timeout(remaining, receiver).await {
            Ok(Ok(callback)) => callback,
            Ok(Err(_)) | Err(_) => {
                self.clear_pending(&pending).await;
                return Err(StudioError::SignInTimedOut);
            }
        };
        let result = self.complete(&pending, callback).await;
        // The browser's page is still being written; keep the listener a moment.
        tokio::time::sleep(CALLBACK_PAGE_GRACE).await;
        self.clear_pending(&pending).await;
        result
    }

    /// Check the state, then exchange the code for a token and read the member id.
    async fn complete(
        &self,
        pending: &Pending,
        callback: Callback,
    ) -> Result<Connection, StudioError> {
        let code = accepted_code(pending, callback)?;
        let issued = self.exchange(pending, &code).await?;
        let member_id = self.member_id(&issued.access_token).await?;
        let now = self.clock.now();
        let token = Token {
            access_token: Secret::new(issued.access_token),
            expires_at: rfc3339(now + Duration::from_secs(issued.expires_in)),
            member_id,
            scope: issued.scope,
            obtained_at: rfc3339(now),
        };
        self.store.save(ACCOUNT, &token)?;
        Ok(Connection {
            status: ConnectionStatus::SignedIn,
            expires_at: Some(token.expires_at),
            scope: Some(token.scope),
        })
    }

    /// One token request for the code, proving the app with the secret or the verifier as
    /// the flow demands.
    async fn exchange(&self, pending: &Pending, code: &str) -> Result<Issued, StudioError> {
        let (client_id, client_secret) = self.app()?;
        let mut form: Vec<(&str, &str)> = vec![
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", self.config.redirect.as_str()),
            ("client_id", client_id),
        ];
        match &pending.code_verifier {
            Some(verifier) => form.push(("code_verifier", verifier.expose())),
            None => form.push(("client_secret", client_secret.expose())),
        }
        let response = self
            .client
            .post(format!(
                "{}/oauth/v2/accessToken",
                self.config.auth_base.as_str()
            ))
            .form(&form)
            .send()
            .await
            .map_err(|_| StudioError::SignInRejected)?;
        read_json(response).await.ok_or(StudioError::SignInRejected)
    }

    /// `GET /v2/userinfo`: the member id (`sub`) the Posts API needs as author.
    async fn member_id(&self, access_token: &str) -> Result<String, StudioError> {
        let response = self
            .client
            .get(format!("{}/v2/userinfo", self.config.api_base.as_str()))
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|_| StudioError::SignInRejected)?;
        let info: UserInfo = read_json(response)
            .await
            .ok_or(StudioError::SignInRejected)?;
        Ok(info.sub)
    }

    /// The connection from the stored token and the clock; no platform call.
    pub fn connection(&self) -> Result<Connection, StudioError> {
        if self
            .pending()
            .is_some_and(|pending| Instant::now() < pending.deadline)
        {
            return Ok(Connection {
                status: ConnectionStatus::AwaitingConsent,
                expires_at: None,
                scope: None,
            });
        }
        let Some(token) = self.store.load(ACCOUNT)? else {
            return Ok(Connection::signed_out());
        };
        let status = if token.is_expired_at(&self.clock.now_rfc3339()) {
            ConnectionStatus::Expired
        } else {
            ConnectionStatus::SignedIn
        };
        Ok(Connection {
            status,
            expires_at: Some(token.expires_at),
            scope: Some(token.scope),
        })
    }

    /// Delete the token; `signed_out` whether or not one existed.
    pub fn sign_out(&self) -> Result<Connection, StudioError> {
        self.store.delete(ACCOUNT)?;
        Ok(Connection::signed_out())
    }

    /// Exactly one `POST /rest/posts`. The caller (the studio) applies the deadline.
    async fn post(&self, request: PostRequest) -> Result<Posted, PostFailure> {
        let token = self.usable_token()?;
        let response = self
            .client
            .post(format!("{}/rest/posts", self.config.api_base.as_str()))
            .bearer_auth(token.access_token.expose())
            .header("LinkedIn-Version", API_VERSION)
            .header("X-Restli-Protocol-Version", "2.0.0")
            .json(&post_body(&token.member_id, &request.text))
            .send()
            .await
            .map_err(transport_failure)?;
        outcome(response)
    }

    /// The stored token when present and not expired; otherwise consent is required and
    /// nothing is sent.
    fn usable_token(&self) -> Result<Token, PostFailure> {
        let token = self
            .store
            .load(ACCOUNT)
            .map_err(|_| PostFailure::Unknown)?
            .ok_or(PostFailure::ConsentRequired)?;
        if token.is_expired_at(&self.clock.now_rfc3339()) {
            return Err(PostFailure::ConsentRequired);
        }
        Ok(token)
    }
}

impl Platform for Linkedin {
    fn publish(&self, request: PostRequest) -> PostFuture<'_> {
        Box::pin(self.post(request))
    }
}

/// The token endpoint's answer.
#[derive(Deserialize)]
struct Issued {
    access_token: String,
    expires_in: u64,
    #[serde(default)]
    scope: String,
}

/// The `userinfo` answer: only the subject is kept.
#[derive(Deserialize)]
struct UserInfo {
    sub: String,
}

/// The callback's code, once its `state` matches the pending sign-in and LinkedIn reported no
/// error. A mismatch refuses before anything is exchanged.
fn accepted_code(pending: &Pending, callback: Callback) -> Result<String, StudioError> {
    let state_ok = callback
        .state
        .as_deref()
        .is_some_and(|state| pending.state.matches(state));
    if !state_ok {
        return Err(StudioError::SignInStateMismatch);
    }
    if callback.error.is_some() {
        return Err(StudioError::SignInDenied);
    }
    callback.code.ok_or(StudioError::SignInDenied)
}

/// The Posts API body for a text post on the member's own feed.
fn post_body(member_id: &str, text: &str) -> serde_json::Value {
    serde_json::json!({
        "author": format!("urn:li:person:{member_id}"),
        "commentary": text,
        "visibility": "PUBLIC",
        "distribution": {
            "feedDistribution": "MAIN_FEED",
            "targetEntities": [],
            "thirdPartyDistributionChannels": []
        },
        "lifecycleState": "PUBLISHED",
        "isReshareDisabledByAuthor": false
    })
}

/// A failure before any answer arrived: unreachable when the connection failed, else unknown
/// because the request may have landed.
fn transport_failure(error: reqwest::Error) -> PostFailure {
    if error.is_connect() {
        PostFailure::Unreachable
    } else {
        PostFailure::Unknown
    }
}

/// LinkedIn's answer as one outcome: 2xx with `x-restli-id` is posted; a 2xx without it is
/// unknown; 401 needs consent, 429 is rate limited, any other 4xx refused, anything else unknown.
fn outcome(response: reqwest::Response) -> Result<Posted, PostFailure> {
    let status = response.status();
    if !status.is_success() {
        return Err(match status.as_u16() {
            401 => PostFailure::ConsentRequired,
            429 => PostFailure::RateLimited,
            400..=499 => PostFailure::Refused,
            _ => PostFailure::Unknown,
        });
    }
    let post_id = response
        .headers()
        .get("x-restli-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
        .filter(|id| !id.is_empty())
        .ok_or(PostFailure::Unknown)?;
    Ok(Posted {
        url: post_url(&post_id),
        post_id,
    })
}

/// The callback handler: hand the first callback to the waiting sign-in, answer the fixed page.
async fn callback(
    State(sender): State<Arc<Mutex<Option<oneshot::Sender<Callback>>>>>,
    Query(callback): Query<Callback>,
) -> Response {
    if let Some(sender) = sender
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .take()
    {
        let _ = sender.send(callback);
    }
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        CALLBACK_PAGE,
    )
        .into_response()
}

/// A bounded 2xx JSON answer, or `None` for any other status, size or shape.
async fn read_json<T: serde::de::DeserializeOwned>(mut response: reqwest::Response) -> Option<T> {
    if !response.status().is_success() {
        return None;
    }
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.ok()? {
        if body.len() + chunk.len() > MAX_ANSWER_BYTES {
            return None;
        }
        body.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&body).ok()
}

/// 32 random bytes as URL-safe base64 without padding: 43 characters.
fn random_token() -> Secret {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("the OS random source is available");
    Secret::new(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes))
}

/// The S256 code challenge of a verifier.
fn s256(verifier: &str) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}
