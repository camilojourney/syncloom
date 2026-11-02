//! SocialPlatforms integration services and shared credential serialization, never a plaintext session file.
use crate::oauth_context::OAuthLifecycle;
use crate::{Channel, Clock, Secret, StudioError, Token, TokenStore, oauth::Connection};
pub use crate::{
    bluesky::Bluesky,
    social_config::SocialConfig,
    threads::Threads,
    x::{CostEstimate, X, estimate},
};
use serde::{Serialize, de::DeserializeOwned};
use std::{sync::Arc, time::Duration};

pub struct SocialPlatforms {
    pub bluesky: Arc<Bluesky>,
    pub x: Arc<X>,
    pub threads: Arc<Threads>,
}
impl SocialPlatforms {
    pub fn account(&self, channel: Channel) -> Result<String, StudioError> {
        match channel {
            Channel::Bluesky => Ok(self.bluesky.credentials.account()),
            Channel::X => Ok(self.x.context.credentials.account()),
            Channel::Threads => Ok(self.threads.context.credentials.account()),
            _ => Err(StudioError::UnsupportedChannel),
        }
    }
    pub fn connection(&self, channel: Channel) -> Result<Connection, StudioError> {
        self.connection_for(channel, false)
    }
    pub fn connection_for(&self, channel: Channel, image: bool) -> Result<Connection, StudioError> {
        match channel {
            Channel::Bluesky => self.bluesky.connection(),
            Channel::X => self.x.connection_for(image),
            Channel::Threads => self.threads.connection_for(image),
            _ => Err(StudioError::UnsupportedChannel),
        }
    }
    pub fn sign_out(&self, channel: Channel) -> Result<Connection, StudioError> {
        match channel {
            Channel::Bluesky => self.bluesky.sign_out(),
            Channel::X => self.x.sign_out(),
            Channel::Threads => self.threads.sign_out(),
            _ => Err(StudioError::UnsupportedChannel),
        }
    }
    pub async fn refresh(&self, channel: Channel) -> Result<Connection, StudioError> {
        match channel {
            Channel::Bluesky => self.bluesky.refresh().await,
            Channel::X => self.x.refresh().await,
            Channel::Threads => self.threads.refresh().await,
            _ => Err(StudioError::UnsupportedChannel),
        }
    }
    pub fn new(
        config: SocialConfig,
        store: Arc<dyn TokenStore>,
        clock: Arc<dyn Clock>,
        deadline: Duration,
    ) -> Self {
        let oauth = |name, base: &str, client: &str| {
            crate::oauth_context::OAuthContext::new(
                config.clone(),
                crate::credential_session::CredentialSession::new(
                    account(name, base, client),
                    store.clone(),
                    clock.clone(),
                ),
                deadline,
            )
        };
        Self {
            bluesky: Arc::new(Bluesky::new(
                config.bluesky.clone(),
                store.clone(),
                clock.clone(),
            )),
            x: Arc::new(X::new(oauth(
                "x",
                config.x_api.as_str(),
                config.x_client.as_deref().unwrap_or(""),
            ))),
            threads: Arc::new(Threads::new(oauth(
                "threads",
                config.threads_api.as_str(),
                config.threads_client.as_deref().unwrap_or(""),
            ))),
        }
    }
}
pub(crate) fn account(platform: &str, base: &str, client: &str) -> String {
    format!(
        "{platform}:{}",
        crate::visual::digest(format!("{base}\n{client}").as_bytes())
    )
}
pub(crate) fn decode<T: DeserializeOwned>(token: &Token) -> Result<T, StudioError> {
    serde_json::from_str(token.access_token.expose()).map_err(|_| StudioError::SignInRejected)
}
pub(crate) fn encode<T: Serialize>(bundle: &T) -> Result<Secret, StudioError> {
    serde_json::to_string(bundle)
        .map(Secret::new)
        .map_err(|_| StudioError::Internal)
}
pub(crate) fn expiry(clock: &dyn Clock, seconds: u64) -> Result<String, StudioError> {
    let seconds = i64::try_from(seconds)
        .ok()
        .filter(|n| *n > 0)
        .ok_or(StudioError::SignInRejected)?;
    clock
        .now()
        .checked_add(time::Duration::seconds(seconds))
        .map(crate::clock::rfc3339)
        .ok_or(StudioError::SignInRejected)
}
pub(crate) fn check_request(
    channel: crate::Channel,
    request: &crate::PostRequest,
    max_image: usize,
) -> Result<(), crate::PostFailure> {
    if crate::fit(channel, &request.text).verdict != crate::Verdict::Pass {
        return Err(crate::PostFailure::Refused);
    }
    if let Some(image) = &request.image
        && (image.bytes.len() > max_image
            || !image.bytes.starts_with(b"\x89PNG\r\n\x1a\n")
            || image.alt_text.trim().is_empty()
            || image.alt_text.chars().count() > 1000)
    {
        return Err(crate::PostFailure::Refused);
    }
    Ok(())
}
pub(crate) fn numeric_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 19 && id.bytes().all(|b| b.is_ascii_digit())
}
