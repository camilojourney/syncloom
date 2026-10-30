//! Claude over the Messages API (`POST {base}/v1/messages`), raw HTTPS.
use super::{Draft, DraftFuture, DraftRequest, ModelProvider, ProviderError};

/// The pinned Claude Sonnet model id (decision D4). UNVERIFIED until the live check runs.
pub const DEFAULT_MODEL: &str = "claude-sonnet-5-5";
pub const DEFAULT_PROVIDER_BASE: &str = "https://api.anthropic.com";

/// A provider base URL that passed [`provider_base`]: `https`, or `http://127.0.0.1:<port>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderBase(String);
impl ProviderBase {
    /// The URL without a trailing slash.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub fn provider_base(value: Option<&str>) -> Result<ProviderBase, &'static str> {
    let _ = value;
    Err("SYNCLOOM_CLAUDE_BASE_URL must be an https URL or http://127.0.0.1:<port>")
}

/// The model id: the environment override when set and not blank, else [`DEFAULT_MODEL`].
pub fn model_id(value: Option<&str>) -> String {
    let _ = value;
    String::new()
}

pub struct ClaudeProvider {
    key: Option<String>,
    base: ProviderBase,
    model: String,
}

impl ClaudeProvider {
    pub fn new(key: Option<String>, base: ProviderBase, model: String) -> Self {
        Self { key, base, model }
    }
}

impl ModelProvider for ClaudeProvider {
    fn draft(&self, request: DraftRequest) -> DraftFuture<'_> {
        let _ = (request, &self.key, &self.base, &self.model);
        Box::pin(async { Err(ProviderError::Rejected) })
    }
}

#[allow(dead_code)]
fn unused_draft() -> Draft {
    Draft {
        text: String::new(),
        usage: None,
    }
}
