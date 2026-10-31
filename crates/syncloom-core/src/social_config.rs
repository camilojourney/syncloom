//! Configuration for the SocialPlatforms adapters. BaseUrl and RedirectUrl own URL policy.
use crate::{BaseUrl, Secret, StudioError};

#[derive(Clone, Debug)]
pub struct SocialConfig {
    pub(crate) bluesky: BaseUrl,
    pub(crate) x_api: BaseUrl,
    pub(crate) x_auth: BaseUrl,
    pub(crate) x_client: Option<String>,
    pub(crate) x_redirect: crate::RedirectUrl,
    pub(crate) threads_api: BaseUrl,
    pub(crate) threads_auth: BaseUrl,
    pub(crate) threads_client: Option<String>,
    pub(crate) threads_secret: Option<Secret>,
    pub(crate) threads_redirect: Option<String>,
}
impl SocialConfig {
    pub fn parse(get: impl Fn(&str) -> Option<String>) -> Result<Self, String> {
        let base = |n, d| BaseUrl::parse(n, get(n).as_deref(), d);
        let x_redirect = crate::oauth::RedirectUrl::parse(
            get("SYNCLOOM_X_REDIRECT_URL")
                .as_deref()
                .unwrap_or("http://127.0.0.1:8793/x/callback"),
        )
        .ok_or("SYNCLOOM_X_REDIRECT_URL must be a loopback HTTP URL with an explicit port")?;
        let threads_redirect = get("SYNCLOOM_THREADS_REDIRECT_URL")
            .filter(|s| !s.trim().is_empty())
            .map(threads_redirect)
            .transpose()
            .map_err(str::to_owned)?;
        Ok(Self {
            bluesky: base("SYNCLOOM_BLUESKY_BASE_URL", "https://bsky.social")?,
            x_api: base("SYNCLOOM_X_API_BASE_URL", "https://api.x.com")?,
            x_auth: base("SYNCLOOM_X_AUTH_BASE_URL", "https://x.com")?,
            x_client: nonblank(get("SYNCLOOM_X_CLIENT_ID")),
            x_redirect,
            threads_api: base("SYNCLOOM_THREADS_API_BASE_URL", "https://graph.threads.com")?,
            threads_auth: base("SYNCLOOM_THREADS_AUTH_BASE_URL", "https://threads.com")?,
            threads_client: nonblank(get("SYNCLOOM_THREADS_CLIENT_ID")),
            threads_secret: nonblank(get("SYNCLOOM_THREADS_CLIENT_SECRET")).map(Secret::new),
            threads_redirect,
        })
    }
}
fn threads_redirect(s: String) -> Result<String, &'static str> {
    const ERROR: &str = "SYNCLOOM_THREADS_REDIRECT_URL must be HTTPS and non-localhost";
    let u = crate::config::plain_url(&s).ok_or(ERROR)?;
    if u.scheme() != "https"
        || u.host_str().is_none_or(|h| {
            h == "localhost" || h.ends_with(".localhost") || h.parse::<std::net::IpAddr>().is_ok()
        })
    {
        return Err(ERROR);
    }
    Ok(u.to_string())
}
fn nonblank(s: Option<String>) -> Option<String> {
    s.filter(|s| !s.trim().is_empty())
}
pub(crate) fn configured<T>(value: Option<&T>) -> Result<&T, StudioError> {
    value.ok_or(StudioError::PlatformNotConfigured)
}
