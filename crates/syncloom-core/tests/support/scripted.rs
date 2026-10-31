//! An in-process fake model provider with scripted replies, recording every request.
#![allow(
    dead_code,
    reason = "Each integration target uses a different subset of the shared test fixtures."
)]
use std::sync::{Arc, Mutex};
use syncloom_core::{Draft, DraftFuture, DraftRequest, ModelProvider, ProviderError, Usage};

#[derive(Clone, Debug)]
pub enum Reply {
    /// Answer this text with usage 321 in, 123 out.
    Text(String),
    /// Answer whitespace only.
    Empty,
    /// Fail as rejected.
    Fail,
    /// Fail as unconfigured (no key).
    Unconfigured,
    /// Never answer.
    Hang,
    /// Panic inside the provider.
    Panic,
}

pub fn text(value: &str) -> Reply {
    Reply::Text(value.to_owned())
}

pub struct Scripted {
    reply: Mutex<Reply>,
    requests: Mutex<Vec<DraftRequest>>,
}

impl Scripted {
    pub fn new(reply: Reply) -> Arc<Self> {
        Arc::new(Self {
            reply: Mutex::new(reply),
            requests: Mutex::new(Vec::new()),
        })
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
    )]
    pub fn set(&self, reply: Reply) {
        *self.reply.lock().unwrap() = reply;
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
    )]
    pub fn requests(&self) -> Vec<DraftRequest> {
        self.requests.lock().unwrap().clone()
    }
}

impl ModelProvider for Scripted {
    #[expect(
        clippy::panic,
        clippy::unwrap_used,
        reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
    )]
    fn draft(&self, request: DraftRequest) -> DraftFuture<'_> {
        self.requests.lock().unwrap().push(request);
        let reply = self.reply.lock().unwrap().clone();
        Box::pin(async move {
            match reply {
                Reply::Text(text) => Ok(Draft {
                    text,
                    usage: Some(Usage {
                        input_tokens: 321,
                        output_tokens: 123,
                    }),
                }),
                Reply::Empty => Ok(Draft {
                    text: "  \n ".to_owned(),
                    usage: None,
                }),
                Reply::Fail => Err(ProviderError::Rejected),
                Reply::Unconfigured => Err(ProviderError::Unconfigured),
                Reply::Hang => std::future::pending().await,
                Reply::Panic => panic!("scripted provider panic"),
            }
        })
    }
}
