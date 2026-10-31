use crate::{BaseUrl, Secret, StudioError, oauth::RedirectUrl};
#[derive(Clone, Debug)]
pub struct MetaConfig {
    pub client_id: Option<String>,
    pub client_secret: Option<Secret>,
    pub config_id: Option<String>,
    pub page_id: Option<String>,
    pub redirect: RedirectUrl,
    pub graph_base: BaseUrl,
    pub auth_base: BaseUrl,
}
impl MetaConfig {
    pub fn parse(get: impl Fn(&str) -> Option<String>) -> Result<Self, String> {
        let value = |name| get(name).filter(|s| !s.trim().is_empty());
        let page_id = value("SYNCLOOM_META_PAGE_ID");
        if page_id.as_deref().is_some_and(|s| !numeric_id(s)) {
            return Err("SYNCLOOM_META_PAGE_ID must be numeric".into());
        }
        let redirect = value("SYNCLOOM_META_REDIRECT_URL")
            .unwrap_or_else(|| "http://127.0.0.1:8913/meta/callback".into());
        Ok(Self {
            client_id: value("SYNCLOOM_META_CLIENT_ID"),
            client_secret: value("SYNCLOOM_META_CLIENT_SECRET").map(Secret::new),
            config_id: value("SYNCLOOM_META_CONFIG_ID"),
            page_id,
            redirect: RedirectUrl::parse(&redirect).ok_or(
                "SYNCLOOM_META_REDIRECT_URL must be a loopback callback with an explicit port",
            )?,
            graph_base: BaseUrl::parse(
                "SYNCLOOM_META_GRAPH_BASE",
                value("SYNCLOOM_META_GRAPH_BASE").as_deref(),
                "https://graph.facebook.com",
            )?,
            auth_base: BaseUrl::parse(
                "SYNCLOOM_META_AUTH_BASE",
                value("SYNCLOOM_META_AUTH_BASE").as_deref(),
                "https://www.facebook.com",
            )?,
        })
    }
    pub(super) fn require(&self) -> Result<(&str, &Secret, &str, &str), StudioError> {
        match (
            &self.client_id,
            &self.client_secret,
            &self.config_id,
            &self.page_id,
        ) {
            (Some(id), Some(secret), Some(config), Some(page)) => Ok((id, secret, config, page)),
            _ => Err(StudioError::MetaNotConfigured),
        }
    }
}
pub(super) fn numeric_id(s: &str) -> bool {
    !s.is_empty() && s.len() <= 128 && s.bytes().all(|b| b.is_ascii_digit())
}
pub(super) fn post_id(s: &str) -> bool {
    !s.is_empty() && s.len() <= 256 && s.bytes().all(|b| b.is_ascii_digit() || b == b'_')
}
