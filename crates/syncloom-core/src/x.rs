//! X public native OAuth 2.0 PKCE and v2 publishing. No paid call is implicit.
use crate::oauth_context::OAuthLifecycle;
use crate::{
    Platform, PostFailure, PostFuture, PostRequest, Posted, Secret, StudioError, Token,
    oauth_context::OAuthContext,
};
use crate::{
    oauth::{Connection, Session, SignInStart},
    social::{decode, encode},
    social_config::configured,
};
use serde::{Deserialize, Serialize};
pub const SCOPES: &str = "tweet.read tweet.write users.read media.write offline.access";
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CostEstimate {
    pub currency: String,
    pub microusd: u64,
    pub includes_link: bool,
    pub metadata_microusd: u64,
    pub pricing_as_of: String,
    pub kind: String,
}
impl CostEstimate {
    pub fn usd(&self) -> String {
        let fraction = format!("{:06}", self.microusd % 1_000_000);
        format!(
            "USD {}.{}",
            self.microusd / 1_000_000,
            fraction.trim_end_matches('0')
        )
    }
}
/// Integer micro-USD, no rounding. Conservative URL detection may overestimate, never a bill.
pub fn estimate(text: &str, image: bool) -> CostEstimate {
    let includes_link = text.split_whitespace().any(|word| {
        let word = word.to_lowercase().replace(['。', '．', '｡'], ".");
        word.contains("://")
            || word.split_once('.').is_some_and(|(a, b)| {
                a.chars().any(char::is_alphanumeric) && b.chars().any(char::is_alphanumeric)
            })
    });
    let metadata_microusd = if image { 5000 } else { 0 };
    CostEstimate {
        currency: "USD".into(),
        microusd: if includes_link { 200000 } else { 15000 } + metadata_microusd,
        includes_link,
        metadata_microusd,
        pricing_as_of: "2026-10-31".into(),
        kind: "estimate_not_actual_charge".into(),
    }
}
#[derive(Serialize, Deserialize)]
struct Bundle {
    access_token: Secret,
    refresh_token: Option<Secret>,
}
#[derive(Default)]
enum ResponseScope {
    #[default]
    Absent,
    Null,
    Present(serde_json::Value),
}
impl<'de> Deserialize<'de> for ResponseScope {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(match serde_json::Value::deserialize(deserializer)? {
            serde_json::Value::Null => Self::Null,
            value => Self::Present(value),
        })
    }
}
impl ResponseScope {
    fn resolve(self, old: Option<&Token>) -> Option<String> {
        match self {
            Self::Absent => old.map(|token| token.scope.clone()),
            Self::Present(serde_json::Value::String(scope)) if valid_response_scope(&scope) => {
                Some(scope)
            }
            _ => None,
        }
    }
}
fn valid_response_scope(scope: &str) -> bool {
    !scope.trim().is_empty()
        && scope.bytes().all(|byte| {
            byte == b' ' || (byte.is_ascii_graphic() && !matches!(byte, b',' | b'"' | b'\\'))
        })
}
#[derive(Deserialize)]
pub(crate) struct Issued {
    access_token: Secret,
    refresh_token: Option<Secret>,
    expires_in: u64,
    #[serde(default)]
    scope: ResponseScope,
    token_type: String,
}
pub struct X {
    pub(crate) context: OAuthContext,
    session: Session,
}
impl X {
    pub(crate) fn new(context: OAuthContext) -> Self {
        let session = Session::with_credentials(
            context.config.x_redirect.clone(),
            context.credentials.clone(),
            context.deadline,
        );
        Self { context, session }
    }
    pub fn sign_in_begin(&self) -> Result<SignInStart, StudioError> {
        let url = self.context.authorization_url(
            self.context.config.x_client.as_ref(),
            self.context.config.x_auth.as_str(),
            "/i/oauth2/authorize",
            self.context.config.x_redirect.as_str(),
            SCOPES,
        )?;
        self.session.begin(url, true)
    }
    pub async fn sign_in_wait(&self) -> Result<Connection, StudioError> {
        let _guard = self.context.credentials.auth.lock().await;
        let (code, verifier, generation) = self.session.wait().await?;
        let client = configured(self.context.config.x_client.as_ref())?;
        let verifier = verifier.ok_or(StudioError::SignInRejected)?;
        let issued = self
            .issue(&[
                ("grant_type", "authorization_code"),
                ("client_id", client),
                ("redirect_uri", self.context.config.x_redirect.as_str()),
                ("code", &code),
                ("code_verifier", verifier.expose()),
            ])
            .await?;
        self.save(issued, None, generation)?;
        self.connection()
    }
    fn token_request(&self, form: &[(&str, &str)]) -> reqwest::RequestBuilder {
        self.context
            .client
            .post(format!(
                "{}/2/oauth2/token",
                self.context.config.x_api.as_str()
            ))
            .form(form)
    }
    async fn issue(&self, form: &[(&str, &str)]) -> Result<Issued, StudioError> {
        crate::oauth::exchange_json(self.token_request(form)).await
    }
    fn save(
        &self,
        issued: Issued,
        old: Option<&Token>,
        generation: u64,
    ) -> Result<(), StudioError> {
        let (scope, scope_verified) = issued
            .scope
            .resolve(old)
            .map(|scope| (scope, true))
            .unwrap_or_default();
        let invalid_credential = !issued.token_type.eq_ignore_ascii_case("bearer")
            || issued.access_token.expose().is_empty()
            || issued
                .refresh_token
                .as_ref()
                .is_some_and(|token| token.expose().is_empty());
        let refresh_token = issued
            .refresh_token
            .or_else(|| {
                old.and_then(|t| decode::<Bundle>(t).ok())
                    .and_then(|b| b.refresh_token)
            })
            .filter(|s| !s.expose().is_empty());
        let mut token = Token {
            access_token: encode(&Bundle {
                access_token: issued.access_token,
                refresh_token,
            })?,
            expires_at: None,
            member_id: String::new(),
            scope,
            scope_verified,
            obtained_at: self.context.credentials.clock.now_rfc3339(),
        };
        let permission = token
            .posting_permission(crate::Channel::X, false)
            .map_err(|e| e.error());
        // Preserve fresh consent-error precedence before credential/expiry validation.
        if permission.is_err() && old.is_none() {
            return permission;
        }
        if invalid_credential {
            return Err(StudioError::SignInRejected);
        }
        token.expires_at = Some(crate::social::expiry(
            self.context.credentials.clock.as_ref(),
            issued.expires_in,
        )?);
        // A valid rotation replaces token and evidence together, even if evidence is unknown.
        self.context.credentials.commit_token(generation, &token)?;
        permission
    }
    pub fn sign_out(&self) -> Result<Connection, StudioError> {
        self.session.sign_out()
    }
}
impl crate::oauth_context::OAuthLifecycle for X {
    type Issued = Issued;
    const CHANNEL: crate::Channel = crate::Channel::X;
    const REFRESH_RULE: crate::credential_session::RefreshRule =
        crate::credential_session::RefreshRule::Any;
    fn oauth(&self) -> &OAuthContext {
        &self.context
    }
    fn pending(&self) -> bool {
        self.session.active()
    }
    fn refresh_request(&self, token: &Token) -> Result<reqwest::RequestBuilder, StudioError> {
        let bundle: Bundle = decode(token)?;
        let refresh_token = bundle.refresh_token.ok_or(StudioError::SignInRejected)?;
        let client = configured(self.context.config.x_client.as_ref())?;
        Ok(self.token_request(&[
            ("grant_type", "refresh_token"),
            ("client_id", client),
            ("refresh_token", refresh_token.expose()),
        ]))
    }
    async fn save_refreshed(
        &self,
        issued: Issued,
        old: &Token,
        generation: u64,
    ) -> Result<(), StudioError> {
        self.save(issued, Some(old), generation)
    }
}
impl Platform for X {
    fn publish(&self, mut request: PostRequest) -> PostFuture<'_> {
        Box::pin(async move {
            crate::social::check_request(crate::Channel::X, &request, 5_000_000)?;
            let image = request.image.is_some();
            let token = self
                .context
                .credentials
                .usable_for_post(crate::Channel::X, image)?;
            let bundle: Bundle = decode(&token).map_err(|_| PostFailure::ConsentRequired)?;
            let mut body = serde_json::json!({"text":request.text});
            if let Some(image) = request.image.take() {
                let id = crate::x_media::upload(
                    &self.context.client,
                    self.context.config.x_api.as_str(),
                    &bundle.access_token,
                    image,
                )
                .await?;
                body["media"] = serde_json::json!({"media_ids":[id]});
            }
            let result: PostResponse = crate::platform::request_json(
                self.context
                    .client
                    .post(format!("{}/2/tweets", self.context.config.x_api.as_str()))
                    .bearer_auth(bundle.access_token.expose())
                    .json(&body),
            )
            .await?;
            if !crate::social::numeric_id(&result.data.id) {
                return Err(PostFailure::Unknown);
            }
            Ok(Posted {
                url: format!("https://x.com/i/web/status/{}", result.data.id),
                post_id: result.data.id,
            })
        })
    }
}
#[derive(Deserialize)]
struct PostResponse {
    data: PostId,
}
#[derive(Deserialize)]
struct PostId {
    id: String,
}
