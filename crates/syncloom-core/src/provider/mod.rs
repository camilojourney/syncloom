//! The model provider boundary: what a provider receives and may answer.
pub mod claude;

pub use claude::{
    ClaudeProvider, DEFAULT_MODEL, DEFAULT_PROVIDER_BASE, ProviderBase, model_id, provider_base,
};

use crate::channel::Channel;
use crate::provenance::Usage;
use std::{future::Future, pin::Pin};

/// What a provider is asked for: the channel and the normalized thought, as data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DraftRequest {
    pub channel: Channel,
    pub thought: String,
}

/// Raw provider output. The studio trims it and decides that empty text is a failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Draft {
    pub text: String,
    pub usage: Option<Usage>,
}

/// Why a provider produced no draft. The studio maps these to error codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderError {
    /// No usable key: absent, blank, or refused by the provider (401, 403).
    Unconfigured,
    /// Any other failure: 429, 5xx, other 4xx, unreachable, or an unusable answer.
    Rejected,
}

pub type DraftFuture<'a> = Pin<Box<dyn Future<Output = Result<Draft, ProviderError>> + Send + 'a>>;

/// A model provider. Dropping the future abandons the call.
pub trait ModelProvider: Send + Sync {
    fn draft(&self, request: DraftRequest) -> DraftFuture<'_>;
}
