//! Mastodon v1 apps/statuses, v2 instance/media. Official docs:
//! https://docs.joinmastodon.org/methods/{apps,oauth,instance,media,statuses}/
//! No status retries: the durable outbox owns uncertainty and operator-initiated attempts.
use crate::oauth::{Connection, Session, SignInStart, read_json};
use crate::platform::{http_failure, request_json, transport_failure};
use crate::{
    BaseUrl, Clock, Platform, PostFailure, PostFuture, PostRequest, Posted, Secret, StudioError,
    Token, TokenStore,
};
use serde::Deserialize;
use std::{sync::Arc, time::Duration};

pub const INSTANCE_ENV: &str = "SYNCLOOM_MASTODON_INSTANCE";
pub const REDIRECT_ENV: &str = "SYNCLOOM_MASTODON_REDIRECT_URL";
pub const SCOPES: &str = "write:statuses write:media";
const DEFAULT_REDIRECT: &str = "http://127.0.0.1:8792/mastodon/callback";
#[derive(Clone, Debug)]
pub struct MastodonConfig {
    instance: Option<BaseUrl>,
    redirect: crate::RedirectUrl,
}
impl MastodonConfig {
    #[expect(
        clippy::unwrap_used,
        reason = "BaseUrl has already parsed and validated this instance URL."
    )]
    pub fn parse(instance: Option<&str>, redirect: Option<&str>) -> Result<Self, String> {
        let instance = instance
            .filter(|s| !s.trim().is_empty())
            .map(|s| BaseUrl::parse(INSTANCE_ENV, Some(s), ""))
            .transpose()?;
        if instance
            .as_ref()
            .is_some_and(|b| reqwest::Url::parse(b.as_str()).unwrap().path() != "/")
        {
            return Err(format!("{INSTANCE_ENV} must be an origin without a path"));
        }
        let redirect = crate::oauth::RedirectUrl::parse(redirect.unwrap_or(DEFAULT_REDIRECT))
            .ok_or_else(|| {
                format!("{REDIRECT_ENV} must be a loopback HTTP URL with an explicit port")
            })?;
        Ok(Self { instance, redirect })
    }
}
#[derive(Deserialize)]
struct App {
    client_id: String,
    client_secret: Secret,
}
#[derive(Deserialize)]
struct Issued {
    access_token: Secret,
    scope: String,
}
pub struct Mastodon {
    config: MastodonConfig,
    client: reqwest::Client,
    session: Session,
    app: tokio::sync::Mutex<Option<App>>,
}
impl Mastodon {
    pub fn new(
        config: MastodonConfig,
        store: Arc<dyn TokenStore>,
        clock: Arc<dyn Clock>,
        deadline: Duration,
    ) -> Self {
        let account = account_for(&config);
        Self {
            session: Session::new(config.redirect.clone(), store, account, clock, deadline),
            config,
            client: crate::config::http_client(),
            app: tokio::sync::Mutex::new(None),
        }
    }
    fn instance(&self) -> Result<&str, StudioError> {
        self.config
            .instance
            .as_ref()
            .map(BaseUrl::as_str)
            .ok_or(StudioError::MastodonNotConfigured)
    }
    /// Stable, private account key. Different instance origins cannot share a token.
    pub fn account(&self) -> String {
        account_for(&self.config)
    }
    #[expect(
        clippy::expect_used,
        reason = "The instance is a validated BaseUrl and the authorization suffix is fixed."
    )]
    pub async fn sign_in_begin(&self) -> Result<SignInStart, StudioError> {
        let instance = self.instance()?;
        let mut slot = self.app.lock().await;
        if self.session.active() {
            return Err(StudioError::SignInInProgress);
        }
        let response = tokio::time::timeout(
            self.session.deadline(),
            self.client
                .post(format!("{instance}/api/v1/apps"))
                .form(&[
                    ("client_name", "Syncloom"),
                    ("redirect_uris", self.config.redirect.as_str()),
                    ("scopes", SCOPES),
                ])
                .send(),
        )
        .await
        .map_err(|_| StudioError::SignInTimedOut)?
        .map_err(|_| StudioError::SignInRejected)?;
        let app: App = read_json(response)
            .await
            .ok_or(StudioError::SignInRejected)?;
        if app.client_id.is_empty() || app.client_secret.expose().is_empty() {
            return Err(StudioError::SignInRejected);
        }
        let mut url =
            reqwest::Url::parse(&format!("{instance}/oauth/authorize")).expect("checked instance");
        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", &app.client_id)
            .append_pair("redirect_uri", self.config.redirect.as_str())
            .append_pair("scope", SCOPES);
        let start = self.session.begin(url, false)?;
        *slot = Some(app);
        Ok(start)
    }
    pub async fn sign_in_wait(&self) -> Result<Connection, StudioError> {
        // Take credentials before waiting. A second waiter cannot consume or exchange them.
        let app = self
            .app
            .lock()
            .await
            .take()
            .ok_or(StudioError::SignInNotStarted)?;
        let (code, _, generation) = self.session.wait().await?;
        let exchange = async {
            let response = self
                .client
                .post(format!("{}/oauth/token", self.instance()?))
                .form(&[
                    ("grant_type", "authorization_code"),
                    ("client_id", app.client_id.as_str()),
                    ("client_secret", app.client_secret.expose()),
                    ("redirect_uri", self.config.redirect.as_str()),
                    ("code", &code),
                ])
                .send()
                .await
                .map_err(|_| StudioError::SignInRejected)?;
            read_json::<Issued>(response)
                .await
                .ok_or(StudioError::SignInRejected)
        };
        let issued = tokio::time::timeout(self.session.deadline(), exchange)
            .await
            .map_err(|_| StudioError::SignInTimedOut)??;
        if issued.access_token.expose().is_empty()
            || !SCOPES
                .split_whitespace()
                .all(|s| issued.scope.split_whitespace().any(|got| s == got))
        {
            return Err(StudioError::SignInRejected);
        }
        #[derive(Deserialize)]
        struct Identity {
            id: String,
        }
        let identity: Identity = crate::oauth::exchange_json(
            self.client
                .get(format!(
                    "{}/api/v1/accounts/verify_credentials",
                    self.instance()?
                ))
                .bearer_auth(issued.access_token.expose()),
        )
        .await?;
        if !valid_id(&identity.id) {
            return Err(StudioError::SignInRejected);
        }
        let token = Token {
            access_token: issued.access_token,
            expires_at: None,
            identity_authority: None,
            page_id: None,
            member_id: identity.id,
            scope: issued.scope,
            obtained_at: self.session.now_rfc3339(),
        };
        self.session.save_token(generation, &token)?;
        self.connection()
    }
    pub fn connection(&self) -> Result<Connection, StudioError> {
        self.instance()?;
        self.session.connection()
    }
    pub fn sign_out(&self) -> Result<Connection, StudioError> {
        self.instance()?;
        self.session.sign_out()
    }
    #[expect(
        clippy::expect_used,
        reason = "The fixed image/png MIME literal is valid."
    )]
    async fn upload_image(
        &self,
        token: &Secret,
        image: crate::platform::PostImage,
    ) -> Result<String, PostFailure> {
        let base = self.instance().map_err(|_| PostFailure::ConsentRequired)?;
        let part = reqwest::multipart::Part::bytes(image.bytes)
            .file_name("image.png")
            .mime_str("image/png")
            .expect("fixed MIME");
        let form = reqwest::multipart::Form::new()
            .part("file", part)
            .text("description", image.alt_text);
        let media: Media = request_json(
            self.client
                .post(format!("{base}/api/v2/media"))
                .bearer_auth(token.expose())
                .multipart(form),
        )
        .await?;
        if !valid_id(&media.id) {
            return Err(PostFailure::Unknown);
        }
        self.wait_for_media(token, media).await
    }

    async fn wait_for_media(
        &self,
        token: &Secret,
        mut media: Media,
    ) -> Result<String, PostFailure> {
        let base = self.instance().map_err(|_| PostFailure::ConsentRequired)?;
        let mut remaining = 20;
        while !media.ready() && remaining > 0 {
            remaining -= 1;
            tokio::time::sleep(Duration::from_millis(50)).await;
            let response = self
                .client
                .get(format!("{base}/api/v1/media/{}", media.id))
                .bearer_auth(token.expose())
                .send()
                .await
                .map_err(transport_failure)?;
            if response.status() == reqwest::StatusCode::PARTIAL_CONTENT {
                continue;
            }
            if let Some(failure) = http_failure(response.status()) {
                return Err(failure);
            }
            let ready: Media = read_json(response).await.ok_or(PostFailure::Unknown)?;
            if ready.id != media.id {
                return Err(PostFailure::Unknown);
            }
            media = ready;
        }
        if media.ready() {
            Ok(media.id)
        } else {
            Err(PostFailure::Unknown)
        }
    }

    #[expect(
        clippy::unwrap_used,
        reason = "The configured instance base has already passed BaseUrl validation."
    )]
    async fn create_status(
        &self,
        token: &Secret,
        request: PostRequest,
        ids: Vec<String>,
    ) -> Result<Posted, PostFailure> {
        let base = self.instance().map_err(|_| PostFailure::ConsentRequired)?;
        let status: Status = request_json(self.client.post(format!("{base}/api/v1/statuses"))
            .bearer_auth(token.expose()).header("Idempotency-Key", request.request_id)
            .json(&serde_json::json!({"status": request.text, "media_ids": ids, "visibility": "public"}))).await?;
        if !valid_id(&status.id) || status.url.is_empty() {
            return Err(PostFailure::Unknown);
        }
        let url = crate::config::plain_url(&status.url).ok_or(PostFailure::Unknown)?;
        if url.origin() != reqwest::Url::parse(base).unwrap().origin() {
            return Err(PostFailure::Unknown);
        }
        Ok(Posted {
            post_id: status.id,
            url: status.url,
        })
    }
}

impl Platform for Mastodon {
    fn posting_target(&self) -> Result<String, PostFailure> {
        let token = self.session.usable_token()?;
        let base = self.instance().map_err(|_| PostFailure::ConsentRequired)?;
        crate::platform::account_target("mastodon", base, &token.member_id)
    }
    fn publish(&self, mut request: PostRequest) -> PostFuture<'_> {
        Box::pin(async move {
            let base = self.instance().map_err(|_| PostFailure::ConsentRequired)?;
            let token = self.session.usable_token()?;
            request.require_target(crate::platform::account_target(
                "mastodon",
                base,
                &token.member_id,
            ))?;
            let instance: Instance =
                request_json(self.client.get(format!("{base}/api/v2/instance"))).await?;
            instance.configuration.check_text(&request.text)?;
            let mut ids = Vec::new();
            if let Some(image) = request.image.take() {
                instance.configuration.check_image(&image)?;
                ids.push(self.upload_image(&token.access_token, image).await?);
            }
            self.create_status(&token.access_token, request, ids).await
        })
    }
}
fn account_for(config: &MastodonConfig) -> String {
    format!(
        "mastodon:{}",
        crate::visual::digest(
            config
                .instance
                .as_ref()
                .map(BaseUrl::as_str)
                .unwrap_or("")
                .as_bytes()
        )
    )
}
fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
#[derive(Deserialize)]
struct Instance {
    configuration: Limits,
}
#[derive(Deserialize)]
struct Limits {
    statuses: StatusLimits,
    media_attachments: MediaLimits,
}
impl Limits {
    fn check_text(&self, text: &str) -> Result<(), PostFailure> {
        if text.chars().count() > self.statuses.max_characters || text.is_empty() {
            Err(PostFailure::Refused)
        } else {
            Ok(())
        }
    }
    fn check_image(&self, image: &crate::platform::PostImage) -> Result<(), PostFailure> {
        if self.statuses.max_media_attachments < 1
            || image.bytes.len() > self.media_attachments.image_size_limit
            || !self
                .media_attachments
                .supported_mime_types
                .iter()
                .any(|s| s == "image/png")
            || image.alt_text.trim().is_empty()
        {
            Err(PostFailure::Refused)
        } else {
            Ok(())
        }
    }
}
#[derive(Deserialize)]
struct StatusLimits {
    max_characters: usize,
    max_media_attachments: usize,
}
#[derive(Deserialize)]
struct MediaLimits {
    image_size_limit: usize,
    supported_mime_types: Vec<String>,
}
#[derive(Deserialize)]
struct Media {
    id: String,
    url: Option<String>,
}
impl Media {
    fn ready(&self) -> bool {
        self.url.as_ref().is_some_and(|u| !u.is_empty())
    }
}
#[derive(Deserialize)]
struct Status {
    id: String,
    url: String,
}
