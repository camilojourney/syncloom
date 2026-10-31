//! Meta Graph v26.0. Consent is shared; targets differ only in their publish protocol.
mod auth;
mod config;
mod graph;
mod media;
mod publish;
use crate::{Clock, Platform, PostFuture, PostRequest, TokenStore, oauth::Session};
pub use config::MetaConfig;
pub use media::{MediaDelivery, MediaFuture, PublicMediaUrl, UnconfiguredMedia};
use std::{sync::Arc, time::Duration};
pub const GRAPH_VERSION: &str = "v26.0";
pub const SCOPES: &[&str] = &[
    "pages_show_list",
    "pages_read_engagement",
    "pages_manage_posts",
    "instagram_basic",
    "instagram_content_publish",
];

pub struct Meta {
    config: MetaConfig,
    session: Session,
    client: reqwest::Client,
    media: Arc<dyn MediaDelivery>,
}
impl Meta {
    pub fn new(
        config: MetaConfig,
        store: Arc<dyn TokenStore>,
        clock: Arc<dyn Clock>,
        deadline: Duration,
        media: Arc<dyn MediaDelivery>,
    ) -> Self {
        let account = format!("meta:{}", config.page_id.as_deref().unwrap_or(""));
        Self {
            config: config.clone(),
            media,
            client: crate::config::http_client(),
            session: Session::new(config.redirect, store, account, clock, deadline),
        }
    }
    fn configured_session(&self) -> Result<&Session, crate::StudioError> {
        self.config.require()?;
        Ok(&self.session)
    }
    pub fn target(self: &Arc<Self>, target: MetaTarget) -> Arc<dyn Platform> {
        Arc::new(Delivery {
            meta: self.clone(),
            target,
        })
    }
}
#[derive(Clone, Copy)]
pub enum MetaTarget {
    Facebook,
    Instagram,
}
struct Delivery {
    meta: Arc<Meta>,
    target: MetaTarget,
}
impl Platform for Delivery {
    fn publish(&self, request: PostRequest) -> PostFuture<'_> {
        Box::pin(self.meta.publish(self.target, request))
    }
}
