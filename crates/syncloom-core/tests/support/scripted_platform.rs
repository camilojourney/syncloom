//! An in-process fake platform with scripted outcomes, recording every request.
#![allow(dead_code)]
use std::sync::{Arc, Mutex};
use syncloom_core::{Platform, PostFailure, PostFuture, PostRequest, Posted};
use tokio::sync::Notify;

#[derive(Clone, Debug)]
pub enum Outcome {
    /// Answer posted with this id and URL.
    Posted(String, String),
    /// Answer this failure.
    Fail(PostFailure),
    /// Never answer.
    Hang,
    /// Answer posted once [`ScriptedPlatform::release`] is called.
    Hold,
}

pub fn posted() -> Outcome {
    Outcome::Posted(
        "urn:li:share:7000000000000000001".to_owned(),
        "https://www.linkedin.com/feed/update/urn:li:share:7000000000000000001/".to_owned(),
    )
}

pub struct ScriptedPlatform {
    outcome: Mutex<Outcome>,
    requests: Mutex<Vec<PostRequest>>,
    gate: Notify,
}

impl ScriptedPlatform {
    pub fn new(outcome: Outcome) -> Arc<Self> {
        Arc::new(Self {
            outcome: Mutex::new(outcome),
            requests: Mutex::new(Vec::new()),
            gate: Notify::new(),
        })
    }
    pub fn set(&self, outcome: Outcome) {
        *self.outcome.lock().unwrap() = outcome;
    }
    pub fn requests(&self) -> Vec<PostRequest> {
        self.requests.lock().unwrap().clone()
    }
    pub fn release(&self) {
        self.gate.notify_one();
    }
}

impl Platform for ScriptedPlatform {
    fn publish(&self, request: PostRequest) -> PostFuture<'_> {
        self.requests.lock().unwrap().push(request);
        let outcome = self.outcome.lock().unwrap().clone();
        Box::pin(async move {
            match outcome {
                Outcome::Posted(post_id, url) => Ok(Posted { post_id, url }),
                Outcome::Fail(failure) => Err(failure),
                Outcome::Hang => std::future::pending().await,
                Outcome::Hold => {
                    self.gate.notified().await;
                    let posted = posted();
                    match posted {
                        Outcome::Posted(post_id, url) => Ok(Posted { post_id, url }),
                        _ => unreachable!(),
                    }
                }
            }
        })
    }
}
