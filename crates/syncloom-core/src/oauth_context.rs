//! Common HTTP and credential environment for the two OAuth protocols.
use crate::{
    StudioError, Token,
    credential_session::{CredentialSession, RefreshRule},
    oauth::Connection,
    social_config::SocialConfig,
};
use std::time::Duration;

pub(crate) struct OAuthContext {
    pub config: SocialConfig,
    pub credentials: std::sync::Arc<CredentialSession>,
    pub deadline: Duration,
    pub client: reqwest::Client,
}
pub(crate) trait OAuthLifecycle: Sync {
    type Issued: serde::de::DeserializeOwned;
    const REFRESH_RULE: RefreshRule;
    const CHANNEL: crate::Channel;
    fn oauth(&self) -> &OAuthContext;
    fn pending(&self) -> bool;
    async fn observed_identity(&self, token: &Token) -> Result<String, StudioError>;

    /// The auth lock serializes rotation and confirmation; the generation also excludes sign-out.
    async fn confirm_token(&self, token: &mut Token, generation: u64) -> Result<(), StudioError> {
        let observed = self
            .observed_identity(token)
            .await
            .map_err(|_| StudioError::IdentityUnconfirmed)?;
        if !crate::social::numeric_id(&observed) {
            return Err(StudioError::IdentityUnconfirmed);
        }
        self.oauth()
            .credentials
            .confirm_identity(Some(&token.member_id), &observed)?;
        token.identity_confirmed = true;
        self.oauth().credentials.commit_token(generation, token)
    }

    async fn save_issued(
        &self,
        mut token: Token,
        generation: u64,
        rotating: bool,
    ) -> Result<(), StudioError> {
        let permission = token
            .posting_grant(Self::CHANNEL, false)
            .map_err(|e| e.error());
        if !rotating {
            permission?;
            let observed = self.observed_identity(&token).await?;
            if !crate::social::numeric_id(&observed) {
                return Err(StudioError::SignInRejected);
            }
            // Threads' short-token subject is already an expected identity; X resolves its first subject here.
            self.oauth().credentials.confirm_identity(
                (!token.member_id.is_empty()).then_some(token.member_id.as_str()),
                &observed,
            )?;
            token.member_id = observed;
            token.identity_confirmed = true;
            return self.oauth().credentials.commit_token(generation, &token);
        }
        // Issuance has spent the old refresh bundle. Persist the actual new grant and
        // expected identity without confirmation before any fallible identity request.
        self.oauth().credentials.commit_token(generation, &token)?;
        let identity = self.confirm_token(&mut token, generation).await;
        if identity == Err(StudioError::IdentityUnconfirmed) {
            permission?;
        }
        identity?;
        permission
    }

    async fn dispatch_token(&self, image: bool) -> Result<Token, crate::PostFailure> {
        let credentials = &self.oauth().credentials;
        let _guard = credentials.auth.lock().await;
        let generation = credentials.generation();
        let mut token = credentials.usable()?;
        token
            .posting_grant(Self::CHANNEL, image)
            .map_err(|e| e.failure())?;
        if !token.identity_confirmed {
            self.confirm_token(&mut token, generation)
                .await
                .map_err(|_| crate::PostFailure::ConsentRequired)?;
        }
        token
            .posting_permission(Self::CHANNEL, image)
            .map_err(|e| e.failure())?;
        Ok(token)
    }
    fn refresh_request(&self, token: &Token) -> Result<reqwest::RequestBuilder, StudioError>;
    async fn save_refreshed(
        &self,
        issued: Self::Issued,
        old: &Token,
        generation: u64,
    ) -> Result<(), StudioError>;
    fn connection(&self) -> Result<Connection, StudioError> {
        self.connection_for(false)
    }
    fn connection_for(&self, image: bool) -> Result<Connection, StudioError> {
        self.oauth()
            .credentials
            .connection_for(self.pending(), Self::CHANNEL, image)
    }
    async fn refresh(&self) -> Result<Connection, StudioError> {
        let renewal = self.oauth().credentials.renewal(Self::REFRESH_RULE).await?;
        renewal
            .token
            .posting_permission(Self::CHANNEL, false)
            .map_err(|e| e.error())?;
        let issued = crate::oauth::exchange_json(self.refresh_request(&renewal.token)?).await?;
        self.save_refreshed(issued, &renewal.token, renewal.generation)
            .await?;
        self.connection()
    }
}
impl OAuthContext {
    #[expect(
        clippy::expect_used,
        reason = "Callers supply validated authorization bases and fixed platform paths."
    )]
    pub fn authorization_url(
        &self,
        client: Option<&String>,
        base: &str,
        path: &str,
        redirect: &str,
        scopes: &str,
    ) -> Result<reqwest::Url, crate::StudioError> {
        let client = crate::social_config::configured(client)?;
        let mut url =
            reqwest::Url::parse(&format!("{base}{path}")).expect("checked authorization base");
        url.query_pairs_mut()
            .append_pair("client_id", client)
            .append_pair("redirect_uri", redirect)
            .append_pair("scope", scopes)
            .append_pair("response_type", "code");
        Ok(url)
    }

    pub fn new(config: SocialConfig, credentials: CredentialSession, deadline: Duration) -> Self {
        Self {
            config,
            credentials: std::sync::Arc::new(credentials),
            deadline,
            client: crate::config::http_client(),
        }
    }
}
