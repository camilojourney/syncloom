#[path = "support/fake_social.rs"]
mod fake;
#[path = "support/social.rs"]
mod fixture;
mod support;
use fake::Fake;
use fixture::*;
use serde_json::json;
use support::memory_store::MemoryStore;
use syncloom_core::{Platform, PostFailure};

#[tokio::test]
async fn x_does_not_claim_success_with_empty_non_numeric_or_oversized_ids() {
    let fake = Fake::x().await;
    let q = social(&fake, MemoryStore::new(), clock());
    x_login(&q).await;
    for id in ["", "bad-id", "12345678901234567890"] {
        fake.reply("/2/tweets", 200, json!({"data":{"id":id}}));
        assert_eq!(
            q.x.publish(text(q.x.as_ref())).await,
            Err(PostFailure::Unknown),
            "{id}"
        );
    }
    assert_eq!(
        fake.calls()
            .iter()
            .filter(|c| c.path == "/2/tweets")
            .count(),
        3
    );
}
