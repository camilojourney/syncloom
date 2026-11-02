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
