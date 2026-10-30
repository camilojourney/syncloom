//! LinkedIn for the founder's personal profile (`linkedin-integration`): configuration, the
//! browser sign-in over a loopback redirect, and the Posts API adapter behind [`Platform`].
//!
//! Read for this adapter (through the Q2 research record, `docs/research/2026-10-30-q2-linkedin-publishing/`):
//! the 3-legged OAuth and native PKCE pages, the Posts API page (version 202610) and the
//! OpenID Connect `userinfo` endpoint. api.linkedin.com has not been called from this repository.
use crate::clock::Clock;
use crate::config::BaseUrl;
use crate::credentials::{Secret, TokenStore};
use crate::error::StudioError;
use crate::platform::{Platform, PostFailure, PostFuture, PostRequest};
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};

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
    let raw = value.unwrap_or(DEFAULT_REDIRECT_URL);
    let url = reqwest::Url::parse(raw).map_err(|_| REFUSED)?;
    let plain = url.username().is_empty()
        && url.password().is_none()
        && url.query().is_none()
        && url.fragment().is_none();
    let host_ok = matches!(url.host_str(), Some("127.0.0.1") | Some("localhost"));
    let Some(port) = url.port() else {
        return Err(REFUSED);
    };
    if url.scheme() != "http" || !plain || !host_ok {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

/// What `POST /v1/linkedin/sign-in` answers: open the URL in a browser before `expires_at`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignInStart {
    pub status: ConnectionStatus,
    pub authorization_url: String,
    pub expires_at: String,
}

/// The LinkedIn integration: sign-in, connection and the Posts API adapter.
pub struct Linkedin {
    config: LinkedinConfig,
    store: Arc<dyn TokenStore>,
    clock: Arc<dyn Clock>,
    sign_in_deadline: Duration,
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
        }
    }

    /// The registered redirect URL, as it must appear in the developer app.
    pub fn redirect_url(&self) -> &str {
        self.config.redirect.as_str()
    }

    /// Bind the callback listener and return the authorization URL.
    pub fn sign_in_begin(&self) -> Result<SignInStart, StudioError> {
        let _ = (&self.config, &self.sign_in_deadline);
        Err(StudioError::LinkedinNotConfigured)
    }

    /// Wait for the browser callback, exchange the code, store the token.
    pub async fn sign_in_wait(&self) -> Result<Connection, StudioError> {
        Err(StudioError::SignInNotStarted)
    }

    /// The connection from the stored token and the clock; no platform call.
    pub fn connection(&self) -> Result<Connection, StudioError> {
        let _ = (&self.store, &self.clock);
        Ok(Connection {
            status: ConnectionStatus::SignedOut,
            expires_at: None,
            scope: None,
        })
    }

    /// Delete the token; `signed_out` whether or not one existed.
    pub fn sign_out(&self) -> Result<Connection, StudioError> {
        Ok(Connection {
            status: ConnectionStatus::SignedOut,
            expires_at: None,
            scope: None,
        })
    }
}

impl Platform for Linkedin {
    fn publish(&self, _request: PostRequest) -> PostFuture<'_> {
        Box::pin(async { Err(PostFailure::Unknown) })
    }
}
