//! Mastodon v1 apps/statuses, v2 instance/media. Official docs:
//! https://docs.joinmastodon.org/methods/{apps,oauth,instance,media,statuses}/
//! No status retries: the durable outbox owns uncertainty and operator-initiated attempts.
use crate::oauth::{Connection, Session, SignInStart, read_json};
use crate::platform::{http_failure, transport_failure};
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
    store: Arc<dyn TokenStore>,
    clock: Arc<dyn Clock>,
    deadline: Duration,
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
        Self {
            config,
            store,
            clock,
            deadline,
            client: crate::config::http_client(),
            session: Session::default(),
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
        format!(
            "mastodon:{}",
            crate::visual::digest(
                self.config
                    .instance
                    .as_ref()
                    .map(BaseUrl::as_str)
                    .unwrap_or("")
                    .as_bytes()
            )
        )
    }
    pub async fn sign_in_begin(&self) -> Result<SignInStart, StudioError> {
        let instance = self.instance()?;
        let mut slot = self.app.lock().await;
        if self.session.active() {
            return Err(StudioError::SignInInProgress);
        }
        let response = tokio::time::timeout(
            self.deadline,
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
        let start = self.session.begin(
            &self.config.redirect,
            url,
            false,
            self.clock.as_ref(),
            self.deadline,
        )?;
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
        let (code, _) = self.session.wait().await?;
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
        let issued = tokio::time::timeout(self.deadline, exchange)
            .await
            .map_err(|_| StudioError::SignInTimedOut)??;
        if issued.access_token.expose().is_empty()
            || !SCOPES
                .split_whitespace()
                .all(|s| issued.scope.split_whitespace().any(|got| s == got))
        {
            return Err(StudioError::SignInRejected);
        }
        let token = Token {
            access_token: issued.access_token,
            expires_at: None,
            member_id: String::new(),
            scope: issued.scope,
            obtained_at: self.clock.now_rfc3339(),
        };
        self.store.save(&self.account(), &token)?;
        self.connection()
    }
    pub fn connection(&self) -> Result<Connection, StudioError> {
        self.instance()?;
        Ok(Connection::from_token(
            self.store.load(&self.account())?,
            &self.clock.now_rfc3339(),
            self.session.active(),
        ))
    }
    pub fn sign_out(&self) -> Result<Connection, StudioError> {
        self.instance()?;
        self.session.cancel();
        self.store.delete(&self.account())?;
        self.connection()
    }
    async fn json<T: serde::de::DeserializeOwned>(
        &self,
        builder: reqwest::RequestBuilder,
    ) -> Result<T, PostFailure> {
        let response = builder.send().await.map_err(transport_failure)?;
        if let Some(f) = http_failure(response.status()) {
            return Err(f);
        }
        read_json(response).await.ok_or(PostFailure::Unknown)
    }
    async fn post(&self, request: PostRequest) -> Result<Posted, PostFailure> {
        let base = self.instance().map_err(|_| PostFailure::ConsentRequired)?;
        let token = self
            .store
            .load(&self.account())
            .map_err(|_| PostFailure::Unknown)?
            .ok_or(PostFailure::ConsentRequired)?;
        if token.is_expired_at(&self.clock.now_rfc3339()) {
            return Err(PostFailure::ConsentRequired);
        }
        let instance: Instance = self
            .json(self.client.get(format!("{base}/api/v2/instance")))
            .await?;
        let limits = instance.configuration;
        if request.text.chars().count() > limits.statuses.max_characters || request.text.is_empty()
        {
            return Err(PostFailure::Refused);
        }
        let mut ids = Vec::new();
        if let Some(image) = request.image {
            if limits.statuses.max_media_attachments < 1
                || image.bytes.len() > limits.media_attachments.image_size_limit
                || !limits
                    .media_attachments
                    .supported_mime_types
                    .iter()
                    .any(|s| s == "image/png")
                || image.alt_text.trim().is_empty()
            {
                return Err(PostFailure::Refused);
            }
            let part = reqwest::multipart::Part::bytes(image.bytes)
                .file_name("image.png")
                .mime_str("image/png")
                .expect("fixed MIME");
            let form = reqwest::multipart::Form::new()
                .part("file", part)
                .text("description", image.alt_text);
            let mut media: Media = self
                .json(
                    self.client
                        .post(format!("{base}/api/v2/media"))
                        .bearer_auth(token.access_token.expose())
                        .multipart(form),
                )
                .await?;
            if !valid_id(&media.id) {
                return Err(PostFailure::Unknown);
            }
            for _ in 0..20 {
                if media.url.as_ref().is_some_and(|u| !u.is_empty()) {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
                let response = self
                    .client
                    .get(format!("{base}/api/v1/media/{}", media.id))
                    .bearer_auth(token.access_token.expose())
                    .send()
                    .await
                    .map_err(transport_failure)?;
                if response.status() == reqwest::StatusCode::PARTIAL_CONTENT {
                    continue;
                }
                if let Some(f) = http_failure(response.status()) {
                    return Err(f);
                }
                let ready: Media = read_json(response).await.ok_or(PostFailure::Unknown)?;
                if ready.id != media.id {
                    return Err(PostFailure::Unknown);
                }
                media = ready;
            }
            if !media.url.as_ref().is_some_and(|u| !u.is_empty()) {
                return Err(PostFailure::Unknown);
            }
            ids.push(media.id);
        }
        let status:Status=self.json(self.client.post(format!("{base}/api/v1/statuses")).bearer_auth(token.access_token.expose()).header("Idempotency-Key",request.request_id).json(&serde_json::json!({"status":request.text,"media_ids":ids,"visibility":"public"}))).await?;
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
    fn publish(&self, request: PostRequest) -> PostFuture<'_> {
        Box::pin(self.post(request))
    }
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
#[derive(Deserialize)]
struct Status {
    id: String,
    url: String,
}
