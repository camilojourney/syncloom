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
    fn oauth(&self) -> &OAuthContext;
    fn pending(&self) -> bool;
    fn refresh_request(&self, token: &Token) -> Result<reqwest::RequestBuilder, StudioError>;
    fn save_refreshed(
        &self,
        issued: Self::Issued,
        old: &Token,
        generation: u64,
    ) -> Result<(), StudioError>;
    fn connection(&self) -> Result<Connection, StudioError> {
        self.oauth().credentials.connection(self.pending())
    }
    async fn refresh(&self) -> Result<Connection, StudioError> {
        self.oauth()
            .refresh(
                Self::REFRESH_RULE,
                |token| self.refresh_request(token),
                |issued, old, generation| {
                    self.save_refreshed(issued, old, generation)?;
                    self.connection()
                },
            )
            .await
    }
}
impl OAuthContext {
    pub async fn refresh<T, Request, Commit>(
        &self,
        rule: RefreshRule,
        request: Request,
        commit: Commit,
    ) -> Result<Connection, StudioError>
    where
        T: serde::de::DeserializeOwned,
        Request: FnOnce(&Token) -> Result<reqwest::RequestBuilder, StudioError>,
        Commit: FnOnce(T, &Token, u64) -> Result<Connection, StudioError>,
    {
        let renewal = self.credentials.renewal(rule).await?;
        let issued = crate::oauth::exchange_json(request(&renewal.token)?).await?;
        commit(issued, &renewal.token, renewal.generation)
    }

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
