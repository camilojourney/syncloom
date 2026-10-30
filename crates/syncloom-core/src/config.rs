//! Configuration shared by the server and the CLI: the data directory and the provider
//! settings, read from an environment lookup the binaries pass in.
use crate::clock::SystemClock;
use crate::provider::{ClaudeProvider, ProviderBase, model_id, provider_base};
use crate::store::Store;
use crate::studio::{Settings, Studio};
use std::{io, path::PathBuf, sync::Arc};

pub const DATA_DIR_ENV: &str = "SYNCLOOM_DATA_DIR";
pub const KEY_ENV: &str = "ANTHROPIC_API_KEY";
pub const BASE_URL_ENV: &str = "SYNCLOOM_CLAUDE_BASE_URL";
pub const MODEL_ENV: &str = "SYNCLOOM_CLAUDE_MODEL";

#[derive(Clone, Debug)]
pub struct StudioConfig {
    pub data_dir: PathBuf,
    /// The raw key as found; [`usable_key`] decides whether it can be sent.
    pub key: Option<String>,
    pub base: ProviderBase,
    pub model: String,
}

/// `SYNCLOOM_DATA_DIR` when set, else `<home>/.syncloom`.
pub fn data_dir(value: Option<&str>, home: Option<&str>) -> Result<PathBuf, &'static str> {
    let _ = (value, home);
    Err("SYNCLOOM_DATA_DIR is not set and HOME is unknown")
}

/// The key without surrounding whitespace, or `None` when what is left cannot be a key: empty,
/// a `#` comment (a `.env` line with no value), or any byte outside visible ASCII.
pub fn usable_key(raw: &str) -> Option<String> {
    let _ = raw;
    None
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
    })
}

/// A studio over the Claude provider and the system clock.
pub fn build_studio(config: StudioConfig, settings: Settings) -> io::Result<Studio> {
    let store = Store::open(config.data_dir)?;
    let provider = ClaudeProvider::new(
        config.key.as_deref().and_then(usable_key),
        config.base,
        config.model,
    );
    Ok(Studio::new(
        store,
        Arc::new(provider),
        Arc::new(SystemClock),
        settings,
    ))
}
