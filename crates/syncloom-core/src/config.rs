//! Configuration shared by the server and the CLI: the data directory, the provider settings,
//! the delivery grant, the Keychain service and the LinkedIn settings, read from an
//! environment lookup the binaries pass in.
use crate::channel::Channel;
use crate::clock::SystemClock;
use crate::credentials::KeychainStore;
use crate::linkedin::{Linkedin, LinkedinConfig, linkedin_config};
use crate::provider::{ClaudeProvider, model_id, provider_base};
use crate::store::Store;
use crate::studio::{Settings, Studio};
use std::{collections::BTreeSet, io, path::PathBuf, sync::Arc};

pub const DATA_DIR_ENV: &str = "SYNCLOOM_DATA_DIR";
pub const KEY_ENV: &str = "ANTHROPIC_API_KEY";
pub const BASE_URL_ENV: &str = "SYNCLOOM_CLAUDE_BASE_URL";
pub const MODEL_ENV: &str = "SYNCLOOM_CLAUDE_MODEL";
/// Comma-separated platform names whose intents are dispatched; unset means all contained.
pub const DELIVERY_ENV: &str = "SYNCLOOM_DELIVERY";
pub const KEYCHAIN_SERVICE_ENV: &str = "SYNCLOOM_KEYCHAIN_SERVICE";
pub const DEFAULT_KEYCHAIN_SERVICE: &str = "syncloom";
const DEFAULT_DATA_DIR_NAME: &str = ".syncloom";

/// A base URL that passed [`BaseUrl::parse`]: `https://<host>` or `http://127.0.0.1:<port>`,
/// without credentials, query or fragment. The one owner of that rule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaseUrl(String);

impl BaseUrl {
    /// `value` when set, else `default`; the error names `variable`.
    pub fn parse(variable: &str, value: Option<&str>, default: &str) -> Result<BaseUrl, String> {
        let refused = || format!("{variable} must be an https URL or http://127.0.0.1:<port>");
        let url = plain_url(value.unwrap_or(default)).ok_or_else(refused)?;
        let allowed = match url.scheme() {
            "https" => url.host_str().is_some(),
            // `Url::port` is None for the scheme's default port, so `:80` is refused with no port.
            "http" => url.host_str() == Some("127.0.0.1") && url.port().is_some(),
            _ => false,
        };
        if !allowed {
            return Err(refused());
        }
        Ok(BaseUrl(url.as_str().trim_end_matches('/').to_owned()))
    }
    /// The URL without a trailing slash.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A URL with no credentials, query or fragment, or `None`. The one owner of that rule for
/// every URL the studio is configured with.
pub(crate) fn plain_url(raw: &str) -> Option<reqwest::Url> {
    let url = reqwest::Url::parse(raw).ok()?;
    let plain = url.username().is_empty()
        && url.password().is_none()
        && url.query().is_none()
        && url.fragment().is_none();
    plain.then_some(url)
}

/// The one outbound HTTP client policy: a 5 s connect timeout and no redirect following, so
/// a provider or platform cannot send the studio anywhere else.
pub(crate) fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("TLS client configuration is valid")
}

#[derive(Clone, Debug)]
pub struct StudioConfig {
    pub data_dir: PathBuf,
    /// The raw key as found; [`usable_key`] decides whether it can be sent.
    pub key: Option<String>,
    pub base: BaseUrl,
    pub model: String,
    /// Platforms granted delivery; every other platform's intents stay contained.
    pub delivery: BTreeSet<String>,
    pub keychain_service: String,
    pub linkedin: LinkedinConfig,
    pub renderer: crate::visual::RendererConfig,
}

/// `SYNCLOOM_DATA_DIR` when set and not blank, else `<home>/.syncloom`.
pub fn data_dir(value: Option<&str>, home: Option<&str>) -> Result<PathBuf, &'static str> {
    if let Some(value) = value.map(str::trim).filter(|v| !v.is_empty()) {
        return Ok(PathBuf::from(value));
    }
    home.map(str::trim)
        .filter(|h| !h.is_empty())
        .map(|home| PathBuf::from(home).join(DEFAULT_DATA_DIR_NAME))
        .ok_or("SYNCLOOM_DATA_DIR is not set and HOME is unknown")
}

/// The key without surrounding whitespace, or `None` when what is left cannot be a key: empty,
/// a `#` comment (a `.env` line with no value), or any byte outside visible ASCII. Such a value
/// is "not configured"; sending it would fail as a malformed header or reach the provider as junk.
pub fn usable_key(raw: &str) -> Option<String> {
    let key = raw.trim();
    let visible = key.bytes().all(|byte| byte.is_ascii_graphic());
    (!key.is_empty() && !key.starts_with('#') && visible).then(|| key.to_owned())
}

/// The platforms named in `SYNCLOOM_DELIVERY`: comma-separated, each a platform of the channel
/// table. Unset or blank grants nothing.
pub fn delivery_grant(value: Option<&str>) -> Result<BTreeSet<String>, String> {
    let mut granted = BTreeSet::new();
    for name in value
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|n| !n.is_empty())
    {
        let known = [Channel::LinkedinText]
            .iter()
            .any(|channel| channel.platform() == name);
        if !known {
            return Err(format!(
                "{DELIVERY_ENV} names an unknown platform; known platforms: linkedin"
            ));
        }
        granted.insert(name.to_owned());
    }
    Ok(granted)
}

/// The Keychain service name: the override when set and not blank, else `syncloom`.
pub fn keychain_service(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or(DEFAULT_KEYCHAIN_SERVICE)
        .to_owned()
}

/// Read the shared configuration through `get` (normally `std::env::var`).
pub fn studio_config(get: impl Fn(&str) -> Option<String>) -> Result<StudioConfig, String> {
    let data_dir = data_dir(get(DATA_DIR_ENV).as_deref(), get("HOME").as_deref())?;
    let base = provider_base(get(BASE_URL_ENV).as_deref())?;
    Ok(StudioConfig {
        data_dir,
        key: get(KEY_ENV),
        base,
        model: model_id(get(MODEL_ENV).as_deref()),
        delivery: delivery_grant(get(DELIVERY_ENV).as_deref())?,
        keychain_service: keychain_service(get(KEYCHAIN_SERVICE_ENV).as_deref()),
        linkedin: linkedin_config(&get)?,
        renderer: crate::visual::renderer_config(&get),
    })
}

/// A studio over the Claude provider, the system clock, the Keychain and LinkedIn; LinkedIn
/// delivers only when the grant names it.
pub fn build_studio(config: StudioConfig, settings: Settings) -> io::Result<Studio> {
    let store = Store::open(config.data_dir)?;
    let provider = ClaudeProvider::new(config.key, config.base, config.model);
    let clock = Arc::new(SystemClock);
    let linkedin = Arc::new(Linkedin::new(
        config.linkedin,
        Arc::new(KeychainStore::new(config.keychain_service)),
        clock.clone(),
        settings.sign_in_deadline,
    ));
    let mut studio = Studio::new(store, Arc::new(provider), clock, settings)
        .with_linkedin(linkedin.clone())
        .with_renderer(config.renderer);
    if config.delivery.contains(crate::linkedin::PLATFORM) {
        studio = studio.with_delivery(crate::linkedin::PLATFORM, linkedin);
    }
    Ok(studio)
}
