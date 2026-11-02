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
pub use crate::oauth::{Connection, ConnectionStatus, RedirectUrl, SignInStart};
use crate::oauth::{Session, read_json};
use crate::platform::{Platform, PostFailure, PostFuture, PostRequest, Posted};
use serde::Deserialize;
use std::{sync::Arc, time::Duration};

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
/// LinkedIn's returned token `scope` is the effective grant. Requested scopes are not.
fn posting_granted(scope: &str) -> bool {
    crate::credentials::posting_scope(crate::Channel::LinkedinText, scope, false)
}
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

/// The loopback redirect accepted by the shared OAuth boundary.
pub fn redirect_url(value: Option<&str>) -> Result<RedirectUrl, &'static str> {
    RedirectUrl::parse(value.unwrap_or(DEFAULT_REDIRECT_URL)).ok_or("SYNCLOOM_LINKEDIN_REDIRECT_URL must be http://127.0.0.1:<port>/<path> or http://localhost:<port>/<path>")
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

/// The LinkedIn integration: sign-in, connection and the Posts API adapter.
pub struct Linkedin {
    config: LinkedinConfig,
    client: reqwest::Client,
    session: Session,
}

impl Linkedin {
    pub fn new(
        config: LinkedinConfig,
        store: Arc<dyn TokenStore>,
        clock: Arc<dyn Clock>,
        sign_in_deadline: Duration,
    ) -> Self {
        Self {
            session: Session::new(
                config.redirect.clone(),
                store,
                ACCOUNT.into(),
                clock,
                sign_in_deadline,
            ),
            config,
            client: crate::config::http_client(),
        }
    }

    /// The registered redirect URL, as it must appear in the developer app.
    pub fn redirect_url(&self) -> &str {
        self.config.redirect.as_str()
    }

    /// The app credentials, needed only to sign in.
    fn app(&self) -> Result<(&str, &Secret), StudioError> {
        match (&self.config.client_id, &self.config.client_secret) {
            (Some(id), Some(secret)) => Ok((id.as_str(), secret)),
            _ => Err(StudioError::LinkedinNotConfigured),
        }
    }

    /// Bind the shared callback listener and return the platform-specific authorization URL.
    #[expect(
        clippy::expect_used,
        reason = "BaseUrl validates the authorization base and authorization_path supplies a fixed path."
    )]
    pub fn sign_in_begin(&self) -> Result<SignInStart, StudioError> {
        let (client_id, _) = self.app()?;
        let mut url = reqwest::Url::parse(&format!(
            "{}{}",
            self.config.auth_base.as_str(),
            self.config.flow.authorization_path()
        ))
        .expect("checked base");
        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", client_id)
            .append_pair("redirect_uri", self.config.redirect.as_str())
            .append_pair("scope", SCOPES);
        self.session
            .begin(url, self.config.flow == AuthFlow::NativePkce)
    }

    pub async fn sign_in_wait(&self) -> Result<Connection, StudioError> {
        let (code, verifier, generation) = self.session.wait().await?;
        let issued = self.exchange(verifier.as_ref(), &code).await?;
        if !posting_granted(&issued.scope) {
            return Err(StudioError::PostingPermissionMissing);
        }
        let member_id = self.member_id(&issued.access_token).await?;
        let now = self.session.now();
        let token = Token {
            access_token: Secret::new(issued.access_token),
            expires_at: Some(rfc3339(now + Duration::from_secs(issued.expires_in))),
            identity_authority: self.config.client_id.clone(),
            page_id: None,
            member_id,
            scope: issued.scope,
            scope_verified: true,
            obtained_at: rfc3339(now),
        };
        self.session.save_token(generation, &token)?;
        self.connection()
    }

    /// One token request for the code, proving the app with the secret or the verifier as
    /// the flow demands.
    async fn exchange(&self, verifier: Option<&Secret>, code: &str) -> Result<Issued, StudioError> {
        let (client_id, client_secret) = self.app()?;
        let mut form: Vec<(&str, &str)> = vec![
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", self.config.redirect.as_str()),
            ("client_id", client_id),
        ];
        match verifier {
            Some(verifier) => form.push(("code_verifier", verifier.expose())),
            None => form.push(("client_secret", client_secret.expose())),
        }
        crate::oauth::exchange_json(
            self.client
                .post(format!(
                    "{}/oauth/v2/accessToken",
                    self.config.auth_base.as_str()
                ))
                .form(&form),
        )
        .await
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
        self.session
            .connection_for(crate::Channel::LinkedinText, false)
    }

    /// Delete only this platform's token.
    pub fn sign_out(&self) -> Result<Connection, StudioError> {
        self.session.sign_out()
    }

    /// Exactly one `POST /rest/posts`. The caller (the studio) applies the deadline.
    async fn post(&self, request: PostRequest) -> Result<Posted, PostFailure> {
        let token = self
            .session
            .usable_for_post(crate::Channel::LinkedinText, false)?;
        request.require_target(token.app_target("linkedin"))?;
        let response = self
            .client
            .post(format!("{}/rest/posts", self.config.api_base.as_str()))
            .bearer_auth(token.access_token.expose())
            .header("LinkedIn-Version", API_VERSION)
            .header("X-Restli-Protocol-Version", "2.0.0")
            .json(&post_body(&token.member_id, &request.text))
            .send()
            .await
            .map_err(crate::platform::transport_failure)?;
        outcome(response)
    }
}

impl Platform for Linkedin {
    fn posting_target(&self) -> Result<String, PostFailure> {
        let token = self.session.usable_token()?;
        token.app_target("linkedin")
    }
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

/// LinkedIn's answer as one outcome: 2xx with `x-restli-id` is posted; a 2xx without it is
/// unknown; 401 needs consent, 429 is rate limited, any other 4xx refused, anything else unknown.
#[expect(
    clippy::expect_used,
    reason = "The preceding non-success branch guarantees an HTTP failure classification."
)]
fn outcome(response: reqwest::Response) -> Result<Posted, PostFailure> {
    let status = response.status();
    if !status.is_success() {
        return Err(crate::platform::http_failure(status).expect("not a success"));
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
