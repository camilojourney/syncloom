//! Threads consent uses an explicitly configured HTTPS redirect, never a pretend local host.
use crate::oauth_context::OAuthLifecycle;
use crate::{Platform, PostFailure, PostFuture, PostRequest, Secret, StudioError};
use crate::{
    oauth::{Connection, SignInStart},
    oauth_context::OAuthContext,
    social_config::configured,
};
use serde::Deserialize;
use std::{sync::Mutex, time::Instant};
struct Pending {
    state: Secret,
    deadline: Instant,
}
pub struct Threads {
    pub(crate) context: OAuthContext,
    pending: Mutex<Option<Pending>>,
}
impl Threads {
    pub(crate) fn new(context: OAuthContext) -> Self {
        Self {
            context,
            pending: Mutex::new(None),
        }
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Poisoned OAuth pending state cannot be safely reused; preserve fail-closed behavior."
    )]
    pub fn sign_in_begin(&self) -> Result<SignInStart, StudioError> {
        configured(self.context.config.threads_secret.as_ref())?;
        let redirect = configured(self.context.config.threads_redirect.as_ref())?;
        let mut slot = self.pending.lock().unwrap();
        if slot.as_ref().is_some_and(|p| p.deadline > Instant::now()) {
            return Err(StudioError::SignInInProgress);
        }
        let state = crate::oauth::random_token();
        let mut url = self.context.authorization_url(
            self.context.config.threads_client.as_ref(),
            self.context.config.threads_auth.as_str(),
            "/oauth/authorize",
            redirect,
            "threads_basic,threads_content_publish",
        )?;
        url.query_pairs_mut().append_pair("state", state.expose());
        self.context.credentials.invalidate();
        *slot = Some(Pending {
            state,
            deadline: Instant::now() + self.context.deadline,
        });
        Ok(SignInStart {
            status: crate::oauth::ConnectionStatus::AwaitingConsent,
            authorization_url: url.to_string(),
            expires_at: crate::clock::rfc3339(
                self.context.credentials.clock.now() + self.context.deadline,
            ),
        })
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Poisoned OAuth pending state cannot be safely reused; preserve fail-closed behavior."
    )]
    fn take_code(&self, callback: &str) -> Result<String, StudioError> {
        let pending = self
            .pending
            .lock()
            .unwrap()
            .take()
            .ok_or(StudioError::SignInNotStarted)?;
        if pending.deadline <= Instant::now() {
            return Err(StudioError::SignInTimedOut);
        }
        callback_code(
            callback,
            configured(self.context.config.threads_redirect.as_ref())?,
            &pending.state,
        )
    }
    pub async fn complete(&self, callback: &str) -> Result<Connection, StudioError> {
        let _guard = self.context.credentials.auth.lock().await;
        let generation = self.context.credentials.generation();
        let code = self.take_code(callback)?;
        let client = configured(self.context.config.threads_client.as_ref())?;
        let secret = configured(self.context.config.threads_secret.as_ref())?;
        let redirect = configured(self.context.config.threads_redirect.as_ref())?;
        let base = self.context.config.threads_api.as_str();
        let short: Short = crate::threads_publish::request(
            self.context
                .client
                .post(format!("{base}/oauth/access_token"))
                .form(&[
                    ("client_id", client.as_str()),
                    ("client_secret", secret.expose()),
                    ("grant_type", "authorization_code"),
                    ("redirect_uri", redirect.as_str()),
                    ("code", code.as_str()),
                ]),
        )
        .await
        .map_err(|_| StudioError::SignInRejected)?;
        let member = short.member()?;
        self.finish(
            self.context
                .client
                .get(format!("{base}/access_token"))
                .query(&[
                    ("grant_type", "th_exchange_token"),
                    ("client_secret", secret.expose()),
                    ("access_token", short.access_token.expose()),
                ]),
            member,
            generation,
        )
        .await
    }
    async fn finish(
        &self,
        request: reqwest::RequestBuilder,
        member: String,
        generation: u64,
    ) -> Result<Connection, StudioError> {
        let long = crate::threads_publish::request(request)
            .await
            .map_err(|_| StudioError::SignInRejected)?;
        self.save(long, member, generation).await?;
        self.connection()
    }
    async fn save(&self, long: Long, member: String, generation: u64) -> Result<(), StudioError> {
        if long.access_token.expose().is_empty() {
            return Err(StudioError::SignInRejected);
        }
        #[derive(Deserialize)]
        struct Identity {
            id: String,
        }
        let identity: Identity = crate::oauth::exchange_json(
            self.context
                .client
                .get(format!(
                    "{}/v1.0/me",
                    self.context.config.threads_api.as_str()
                ))
                .query(&[("fields", "id")])
                .bearer_auth(long.access_token.expose()),
        )
        .await?;
        if identity.id != member {
            self.context.credentials.clear()?;
            return Err(StudioError::SignInRejected);
        }
        if !crate::social::numeric_id(&identity.id) {
            return Err(StudioError::SignInRejected);
        }
        self.context.credentials.commit_token(
            generation,
            &crate::Token {
                access_token: long.access_token,
                expires_at: Some(crate::social::expiry(
                    self.context.credentials.clock.as_ref(),
                    long.expires_in,
                )?),
                member_id: member,
                identity_authority: self.context.config.threads_client.clone(),
                page_id: None,
                scope: "threads_basic,threads_content_publish".into(),
                obtained_at: self.context.credentials.clock.now_rfc3339(),
            },
        )
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Poisoned OAuth pending state cannot be safely reused; preserve fail-closed behavior."
    )]
    pub fn sign_out(&self) -> Result<Connection, StudioError> {
        self.pending.lock().unwrap().take();
        self.context.credentials.clear()
    }
}
impl crate::oauth_context::OAuthLifecycle for Threads {
    type Issued = Long;
    const REFRESH_RULE: crate::credential_session::RefreshRule =
        crate::credential_session::RefreshRule::UnexpiredAfter(time::Duration::days(1));
    fn oauth(&self) -> &OAuthContext {
        &self.context
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Poisoned OAuth pending state cannot be safely reused; preserve fail-closed behavior."
    )]
    fn pending(&self) -> bool {
        self.pending
            .lock()
            .unwrap()
            .as_ref()
            .is_some_and(|p| p.deadline > Instant::now())
    }
    fn refresh_request(
        &self,
        token: &crate::Token,
    ) -> Result<reqwest::RequestBuilder, StudioError> {
        Ok(self
            .context
            .client
            .get(format!(
                "{}/refresh_access_token",
                self.context.config.threads_api.as_str()
            ))
            .query(&[
                ("grant_type", "th_refresh_token"),
                ("access_token", token.access_token.expose()),
            ]))
    }
    async fn save_refreshed(
        &self,
        long: Long,
        old: &crate::Token,
        generation: u64,
    ) -> Result<(), StudioError> {
        self.save(long, old.member_id.clone(), generation).await
    }
}
impl Platform for Threads {
    fn posting_target(&self) -> Result<String, PostFailure> {
        let token = self.context.credentials.usable()?;
        token.app_target("threads")
    }
    fn publish(&self, request: PostRequest) -> PostFuture<'_> {
        Box::pin(async move {
            crate::social::check_request(crate::Channel::Threads, &request, 0)?;
            let token = self.context.credentials.usable()?;
            request.require_target(token.app_target("threads"))?;
            if !crate::social::numeric_id(&token.member_id) {
                return Err(PostFailure::ConsentRequired);
            }
            crate::threads_publish::publish(
                &self.context.client,
                self.context.config.threads_api.as_str(),
                &token,
                request,
            )
            .await
        })
    }
}
#[derive(Deserialize)]
struct Short {
    access_token: Secret,
    user_id: serde_json::Value,
}
#[derive(Deserialize)]
pub(crate) struct Long {
    access_token: Secret,
    expires_in: u64,
}

fn callback_code(
    callback: &str,
    redirect: &str,
    state_expected: &Secret,
) -> Result<String, StudioError> {
    let mut url = reqwest::Url::parse(callback).map_err(|_| StudioError::SignInStateMismatch)?;
    let pairs: Vec<_> = url.query_pairs().into_owned().collect();
    url.set_query(None);
    url.set_fragment(None);
    let state: Vec<_> = pairs.iter().filter(|(k, _)| k == "state").collect();
    if url.as_str() != redirect || state.len() != 1 || !state_expected.matches(&state[0].1) {
        return Err(StudioError::SignInStateMismatch);
    }
    if pairs.iter().any(|(k, _)| k == "error") {
        return Err(StudioError::SignInDenied);
    }
    let codes: Vec<_> = pairs.into_iter().filter(|(k, _)| k == "code").collect();
    if codes.len() != 1 || codes[0].1.is_empty() {
        return Err(StudioError::SignInDenied);
    }
    Ok(codes[0].1.clone())
}
impl Short {
    fn member(&self) -> Result<String, StudioError> {
        let member = match &self.user_id {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            _ => return Err(StudioError::SignInRejected),
        };
        if !crate::social::numeric_id(&member) || self.access_token.expose().is_empty() {
            return Err(StudioError::SignInRejected);
        }
        Ok(member)
    }
}
