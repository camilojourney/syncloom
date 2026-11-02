// Shared by core and HTTP test binaries, which use different subsets of these fixtures.
#![allow(
    dead_code,
    reason = "Each integration target uses a different subset of the shared test fixtures."
)]
use super::{fake_meta, memory_store::MemoryStore};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use syncloom_core::{
    FixedClock, Secret, Token,
    meta::{MediaDelivery, MediaFuture, Meta, MetaConfig, PublicMediaUrl},
};
pub fn clock() -> Arc<FixedClock> {
    Arc::new(FixedClock(time::macros::datetime!(2026-10-31 00:00 UTC)))
}
#[derive(Default)]
pub struct Media {
    pub images: Mutex<Vec<(String, Vec<u8>)>>,
    pub refused: bool,
}
impl MediaDelivery for Media {
    #[expect(
        clippy::unwrap_used,
        reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
    )]
    fn deliver<'a>(&'a self, bytes: Vec<u8>, mime: &'a str) -> MediaFuture<'a> {
        Box::pin(async move {
            if self.refused {
                return Err(syncloom_core::PostFailure::Refused);
            }
            let mut images = self.images.lock().unwrap();
            images.push((mime.into(), bytes));
            PublicMediaUrl::parse(&format!("https://media.invalid/{}.jpg", images.len()))
                .ok_or(syncloom_core::PostFailure::Refused)
        })
    }
}
#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
pub fn config(fake: &fake_meta::FakeMeta) -> MetaConfig {
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    MetaConfig::parse(|name| match name {
        "SYNCLOOM_META_CLIENT_ID" => Some("app-1".into()),
        "SYNCLOOM_META_CLIENT_SECRET" => Some("fake-app-secret".into()),
        "SYNCLOOM_META_CONFIG_ID" => Some("config-1".into()),
        "SYNCLOOM_META_PAGE_ID" => Some("101".into()),
        "SYNCLOOM_META_REDIRECT_URL" => Some(format!("http://127.0.0.1:{port}/meta/callback")),
        "SYNCLOOM_META_GRAPH_BASE" | "SYNCLOOM_META_AUTH_BASE" => Some(fake.base().into()),
        _ => None,
    })
    .unwrap()
}
pub fn adapter(
    fake: &fake_meta::FakeMeta,
    store: Arc<MemoryStore>,
    media: Arc<Media>,
) -> Arc<Meta> {
    Arc::new(Meta::new(
        config(fake),
        store,
        clock(),
        Duration::from_secs(1),
        media,
    ))
}
pub fn stored() -> Token {
    Token {
        access_token: Secret::new(fake_meta::TOKEN),
        expires_at: None,
        identity_authority: None,
        page_id: Some("101".into()),
        member_id: "202".into(),
        scope: fake_meta::SCOPES.join(" "),
        scope_verified: true,
        identity_confirmed: true,
        obtained_at: "2026-10-31T00:00:00Z".into(),
    }
}
#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
pub async fn consent(meta: &Meta) {
    let start = meta.sign_in_begin().unwrap();
    reqwest::Client::new()
        .get(start.authorization_url)
        .send()
        .await
        .unwrap();
    assert_eq!(
        meta.sign_in_wait().await.unwrap().status,
        syncloom_core::ConnectionStatus::SignedIn
    );
}
