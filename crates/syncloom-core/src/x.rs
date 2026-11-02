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
    refresh_token: Secret,
}
#[derive(Deserialize)]
pub(crate) struct Issued {
    access_token: Secret,
    refresh_token: Option<Secret>,
    expires_in: u64,
    scope: Option<String>,
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
        self.save(issued, None, generation).await?;
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
    async fn save(
        &self,
        issued: Issued,
        old: Option<&Token>,
        generation: u64,
    ) -> Result<(), StudioError> {
        #[derive(Deserialize)]
        struct Identity {
            data: User,
        }
        #[derive(Deserialize)]
        struct User {
            id: String,
        }
        let identity: Identity = crate::oauth::exchange_json(
            self.context
                .client
                .get(format!("{}/2/users/me", self.context.config.x_api.as_str()))
                .bearer_auth(issued.access_token.expose()),
        )
        .await?;
        if old.is_some_and(|old| old.member_id != identity.data.id) {
            self.context.credentials.clear()?;
            return Err(StudioError::SignInRejected);
        }
        if !crate::social::numeric_id(&identity.data.id) {
            return Err(StudioError::SignInRejected);
        }
        let scope = issued
            .scope
            .or_else(|| old.map(|t| t.scope.clone()))
            .ok_or(StudioError::SignInRejected)?;
        if !issued.token_type.eq_ignore_ascii_case("bearer")
            || issued.access_token.expose().is_empty()
            || !SCOPES
                .split_whitespace()
                .all(|s| scope.split_whitespace().any(|got| s == got))
        {
            return Err(StudioError::SignInRejected);
        }
        let refresh_token = issued
            .refresh_token
            .or_else(|| {
                old.and_then(|t| decode::<Bundle>(t).ok())
                    .map(|b| b.refresh_token)
            })
            .filter(|s| !s.expose().is_empty())
            .ok_or(StudioError::SignInRejected)?;
        self.context.credentials.commit(
            generation,
            encode(&Bundle {
                access_token: issued.access_token,
                refresh_token,
            })?,
            crate::social::expiry(self.context.credentials.clock.as_ref(), issued.expires_in)?,
            identity.data.id,
            scope,
        )
    }
    pub fn sign_out(&self) -> Result<Connection, StudioError> {
        self.session.sign_out()
    }
}
impl crate::oauth_context::OAuthLifecycle for X {
    type Issued = Issued;
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
        let client = configured(self.context.config.x_client.as_ref())?;
        Ok(self.token_request(&[
            ("grant_type", "refresh_token"),
            ("client_id", client),
            ("refresh_token", bundle.refresh_token.expose()),
        ]))
    }
    async fn save_refreshed(
        &self,
        issued: Issued,
        old: &Token,
        generation: u64,
    ) -> Result<(), StudioError> {
        self.save(issued, Some(old), generation).await
    }
}
impl Platform for X {
    fn posting_target(&self) -> Result<String, PostFailure> {
        let token = self.context.credentials.usable()?;
        crate::platform::account_target("x", "", &token.member_id)
    }
    fn publish(&self, mut request: PostRequest) -> PostFuture<'_> {
        Box::pin(async move {
            crate::social::check_request(crate::Channel::X, &request, 5_000_000)?;
            let token = self.context.credentials.usable()?;
            request.require_target(crate::platform::account_target("x", "", &token.member_id))?;
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
