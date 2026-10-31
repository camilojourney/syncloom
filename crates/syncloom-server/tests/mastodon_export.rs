#[path = "../../syncloom-core/tests/support/fake_mastodon.rs"]
mod fake_mastodon;
mod support;
use serde_json::json;
use std::{sync::Arc, time::Duration};
use support::{
    answer, client,
    scripted::{Scripted, text},
};
use syncloom_core::{
    Settings, Store, Studio,
    mastodon::{Mastodon, MastodonConfig},
};

#[tokio::test]
async fn full_mastodon_consent_render_approve_publish_repeat_and_export_are_fake_only() {
    let fake = fake_mastodon::start(fake_mastodon::Config {
        async_media: true,
        ..Default::default()
    })
    .await;
    let tokens = support::memory_store::MemoryStore::new();
    let adapter = Arc::new(Mastodon::new(
        MastodonConfig::parse(
            Some(fake.base()),
            Some(&format!(
                "http://127.0.0.1:{}/mastodon/callback",
                support::free_port()
            )),
        )
        .unwrap(),
        tokens,
        support::fixed_clock(),
        Duration::from_secs(3),
    ));
    let a = adapter.clone();
    let server = support::start_studio(
        Scripted::new(text("Build with evidence.")),
        Settings::default(),
        move |s| s.with_mastodon(a.clone()).with_delivery("mastodon", a),
    )
    .await;
    let start = answer(
        client()
            .post(server.url("/v1/mastodon/sign-in"))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(start.status, 202);
    client()
        .get(start.body["authorization_url"].as_str().unwrap())
        .send()
        .await
        .unwrap();
    let signed = answer(
        client()
            .get(server.url("/v1/mastodon/sign-in"))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(signed.body["status"], "signed_in");
    assert!(signed.body.get("expires_at").is_none());
    let created = answer(
        client()
            .post(server.url("/v1/thoughts"))
            .json(&json!({"text":"Build with evidence.","channels":["mastodon"]}))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(created.status, 201);
    let p = &created.body["pieces"][0];
    let id = p["piece_id"].as_str().unwrap();
    let rendered=answer(client().post(server.url(&format!("/v1/pieces/{id}/render"))).json(&json!({"expected_revision":p["revision"],"template":"midnight","slides":["Build with evidence.","Review exact bytes."]})).send().await.unwrap()).await;
    assert_eq!(rendered.status, 200);
    let rev = rendered.body["revision"].as_str().unwrap();
    let approved=answer(client().patch(server.url(&format!("/v1/pieces/{id}"))).json(&json!({"expected_revision":rev,"status":"approved","expected_assets":{"image_sha256":rendered.body["assets"]["image_sha256"],"carousel_sha256":rendered.body["assets"]["carousel_sha256"]}})).send().await.unwrap()).await;
    assert_eq!(approved.status, 200);
    let published = answer(
        client()
            .post(server.url(&format!("/v1/pieces/{id}/publish")))
            .json(&json!({"expected_revision":rev}))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(published.body["status"], "posted");
    assert_eq!(published.body["post_id"], "status-1");
    let media = fake.calls("media");
    assert_eq!(media.len(), 1);
    assert_eq!(
        media[0].headers["authorization"],
        format!("Bearer {}", fake_mastodon::TOKEN)
    );
    assert!(media[0].body.windows(8).any(|w| w == b"\x89PNG\r\n\x1a\n"));
    assert!(String::from_utf8_lossy(&media[0].body).contains("Build with evidence."));
    let reopened = Studio::new(
        Store::open(server.dir.path()).unwrap(),
        Scripted::new(text("unused")),
        support::fixed_clock(),
        Settings::default(),
    )
    .with_delivery("mastodon", adapter);
    let repeat = reopened
        .publish(
            id,
            syncloom_core::PublishRequest {
                expected_revision: rev.into(),
                dry_run: false,
            },
        )
        .await
        .unwrap();
    assert_eq!(repeat.post_id.as_deref(), Some("status-1"));
    assert!(!repeat.created);
    assert_eq!(fake.calls("statuses").len(), 1);
    assert_eq!(fake.calls("media").len(), 1);
    let exported = answer(
        client()
            .post(server.url(&format!("/v1/pieces/{id}/export")))
            .json(&json!({"expected_revision":rev,"target":"reddit"}))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(exported.body["status"], "exported");
    let path = std::path::Path::new(exported.body["bundle"].as_str().unwrap());
    assert!(path.join("image.png").is_file());
    assert!(path.join("carousel.pdf").is_file());
    let records =
        std::fs::read_to_string(server.dir.path().join("provenance/events.jsonl")).unwrap();
    assert!(!records.contains(fake_mastodon::TOKEN));
    assert!(!records.contains("Build with evidence."));
    let signed_out = answer(
        client()
            .delete(server.url("/v1/mastodon/connection"))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(signed_out.body["status"], "signed_out");
}

#[tokio::test]
async fn mastodon_connection_without_an_instance_is_an_explicit_configuration_error() {
    let server = support::start(Scripted::new(text("A draft about evidence."))).await;
    let result = answer(
        client()
            .get(server.url("/v1/mastodon/connection"))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(result.status, 503);
    assert_eq!(result.code(), "mastodon_not_configured");
}

#[tokio::test]
async fn export_requires_approval_and_returns_a_real_local_bundle_never_a_post() {
    let server = support::start(Scripted::new(text("A draft about evidence."))).await;
    let created = answer(
        client()
            .post(server.url("/v1/thoughts"))
            .json(&json!({"text":"Build with evidence.","channels":["linkedin_text"]}))
            .send()
            .await
            .unwrap(),
    )
    .await;
    let piece = &created.body["pieces"][0];
    let id = piece["piece_id"].as_str().unwrap();
    let revision = piece["revision"].as_str().unwrap();
    let url = server.url(&format!("/v1/pieces/{id}/export"));
    let request = json!({"expected_revision":revision,"target":"reddit"});
    let refused = answer(client().post(&url).json(&request).send().await.unwrap()).await;
    assert_eq!(refused.status, 409);
    assert_eq!(refused.code(), "approval_required");
    let approved = answer(
        client()
            .patch(server.url(&format!("/v1/pieces/{id}")))
            .json(&json!({"expected_revision":revision,"status":"approved"}))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(approved.status, 200);
    let result = answer(client().post(&url).json(&request).send().await.unwrap()).await;
    assert_eq!(result.status, 200);
    assert_eq!(result.body["status"], "exported");
    assert!(result.body.get("post_id").is_none());
    let bundle = std::path::Path::new(result.body["bundle"].as_str().unwrap());
    assert!(bundle.starts_with(server.dir.path().join("exports")));
    assert_eq!(
        std::fs::read_to_string(bundle.join("text.txt")).unwrap(),
        "A draft about evidence."
    );
    assert!(
        std::fs::read_to_string(bundle.join("notes.md"))
            .unwrap()
            .contains("subreddit")
    );
    let again = answer(client().post(&url).json(&request).send().await.unwrap()).await;
    assert_eq!(again.body["request_id"], result.body["request_id"]);
    assert_eq!(again.body["created"], false);
    let stale = answer(
        client()
            .post(&url)
            .json(&json!({"expected_revision":"old","target":"reddit"}))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(stale.code(), "revision_conflict");
}
