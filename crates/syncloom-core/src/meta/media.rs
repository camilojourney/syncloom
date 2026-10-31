use crate::PostFailure;
use std::{future::Future, pin::Pin};
/// Public fetch location supplied only by a trusted operator-installed MediaDelivery.
/// This app does not fetch this URL; the provider does. Never accept it from a publish body.
#[derive(Clone, Debug)]
pub struct PublicMediaUrl(String);
impl PublicMediaUrl {
    pub fn parse(raw: &str) -> Option<Self> {
        let url = reqwest::Url::parse(raw).ok()?;
        let host = url.domain()?;
        if url.scheme() != "https"
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
            || !host.contains('.')
            || host.ends_with(".local")
            || host.ends_with(".localhost")
            || host.ends_with(".internal")
            || host.ends_with('.')
        {
            return None;
        }
        Some(Self(url.to_string()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
pub type MediaFuture<'a> =
    Pin<Box<dyn Future<Output = Result<PublicMediaUrl, PostFailure>> + Send + 'a>>;
/// Must expose these exact approved bytes, without transformation, until Graph finishes fetching.
/// A real host, access lifetime and cleanup policy require a separate approved implementation.
pub trait MediaDelivery: Send + Sync {
    fn deliver<'a>(&'a self, bytes: Vec<u8>, mime: &'a str) -> MediaFuture<'a>;
}
pub struct UnconfiguredMedia;
impl MediaDelivery for UnconfiguredMedia {
    fn deliver<'a>(&'a self, _bytes: Vec<u8>, _mime: &'a str) -> MediaFuture<'a> {
        Box::pin(async { Err(PostFailure::Refused) })
    }
}
