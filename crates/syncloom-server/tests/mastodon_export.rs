#[path = "../../syncloom-core/tests/support/fake_mastodon.rs"]
mod fake_mastodon;
mod support;
use serde_json::json;
use std::{sync::Arc, time::Duration};
use support::{
    scripted::{Scripted, text},
    *,
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
    let tokens = memory_store::MemoryStore::new();
    let adapter = Arc::new(Mastodon::new(
        MastodonConfig::parse(
            Some(fake.base()),
            Some(&format!(
                "http://127.0.0.1:{}/mastodon/callback",
                free_port()
            )),
        )
        .unwrap(),
        tokens,
        fixed_clock(),
        Duration::from_secs(3),
    ));
    let a = adapter.clone();
    let server = start_studio(
        Scripted::new(text("Build with evidence.")),
        Settings::default(),
        move |s| s.with_mastodon(a.clone()).with_delivery("mastodon", a),
    )
    .await;
    let start = post_empty(&server, "/v1/mastodon/sign-in").await;
    assert_eq!(start.status, 202, "{}", start.raw);
    client()
        .get(start.body["authorization_url"].as_str().unwrap())
        .send()
        .await
        .unwrap();
    let signed = get(&server, "/v1/mastodon/sign-in").await;
    assert_eq!(signed.body["status"], "signed_in");
    assert!(signed.body.get("expires_at").is_none());
    let created = post(
        &server,
        "/v1/thoughts",
        json!({"text":"Build with evidence.","channels":["mastodon"]}),
    )
    .await;
    assert_eq!(created.status, 201, "{}", created.raw);
    let p = &created.body["pieces"][0];
    let id = p["piece_id"].as_str().unwrap();
    let path = format!("/v1/pieces/{id}");
    let rendered = post(
        &server,
        &format!("{path}/render"),
        json!({
            "expected_revision":p["revision"],"template":"midnight",
            "slides":["Build with evidence.","Review exact bytes."]
        }),
    )
    .await;
    assert_eq!(rendered.status, 200, "{}", rendered.raw);
    let rev = rendered.body["revision"].as_str().unwrap();
    let approved = send_json(
        &server,
        reqwest::Method::PATCH,
        &path,
        json!({
            "expected_revision":rev,"status":"approved","expected_assets":{
                "image_sha256":rendered.body["assets"]["image_sha256"],
                "carousel_sha256":rendered.body["assets"]["carousel_sha256"]
            }
        }),
    )
    .await;
    assert_eq!(approved.status, 200, "{}", approved.raw);
    let published = post(
        &server,
        &format!("{path}/publish"),
        json!({"expected_revision":rev}),
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
        fixed_clock(),
        Settings::default(),
    )
    .with_delivery("mastodon", adapter);
    let repeat = reopened
        .publish(
            id,
            syncloom_core::PublishRequest {
                expected_revision: rev.into(),
                accepted_cost_microusd: None,
                dry_run: false,
            },
        )
        .await
        .unwrap();
    assert_eq!(repeat.post_id.as_deref(), Some("status-1"));
    assert!(!repeat.created);
    assert_eq!(fake.calls("statuses").len(), 1);
    assert_eq!(fake.calls("media").len(), 1);
    let exported = post(
        &server,
        &format!("{path}/export"),
        json!({"expected_revision":rev,"target":"reddit"}),
    )
    .await;
    assert_eq!(exported.body["status"], "exported");
    let bundle = std::path::Path::new(exported.body["bundle"].as_str().unwrap());
    assert!(bundle.join("image.png").is_file());
    assert!(bundle.join("carousel.pdf").is_file());
    let records =
        std::fs::read_to_string(server.dir.path().join("provenance/events.jsonl")).unwrap();
    assert!(!records.contains(fake_mastodon::TOKEN));
    assert!(!records.contains("Build with evidence."));
    let signed_out = send_json(
        &server,
        reqwest::Method::DELETE,
        "/v1/mastodon/connection",
        serde_json::Value::Null,
    )
    .await;
    assert_eq!(signed_out.body["status"], "signed_out");
}

#[tokio::test]
async fn mastodon_connection_without_an_instance_is_an_explicit_configuration_error() {
    let server = start(Scripted::new(text("A draft about evidence."))).await;
    let result = get(&server, "/v1/mastodon/connection").await;
    assert_eq!(result.status, 503);
    assert_eq!(result.code(), "mastodon_not_configured");
}

#[tokio::test]
async fn export_requires_approval_and_returns_a_real_local_bundle_never_a_post() {
    let server = start(Scripted::new(text("A draft about evidence."))).await;
    let created = create(&server, THOUGHT).await;
    assert_eq!(created.status, 201, "{}", created.raw);
    let piece = &created.body["pieces"][0];
    let id = piece["piece_id"].as_str().unwrap();
    let revision = piece["revision"].as_str().unwrap();
    let path = format!("/v1/pieces/{id}/export");
    let request = json!({"expected_revision":revision,"target":"reddit"});
    let refused = post(&server, &path, request.clone()).await;
    assert_eq!(refused.status, 409);
    assert_eq!(refused.code(), "approval_required");
    let approved = approve(&server, piece).await;
    assert_eq!(approved.status, 200);
    let result = post(&server, &path, request.clone()).await;
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
    let again = post(&server, &path, request).await;
    assert_eq!(again.body["request_id"], result.body["request_id"]);
    assert_eq!(again.body["created"], false);
    let stale = post(
        &server,
        &path,
        json!({"expected_revision":"old","target":"reddit"}),
    )
    .await;
    assert_eq!(stale.code(), "revision_conflict");
}

#[tokio::test]
async fn saved_new_channels_are_not_mislabeled_as_linkedin_in_the_browser() {
    let server = start(Scripted::new(text("A saved draft about evidence."))).await;
    for channel in ["mastodon", "manual_export"] {
        let created = post(
            &server,
            "/v1/thoughts",
            json!({
                "text":THOUGHT,"channels":[channel]
            }),
        )
        .await;
        assert_eq!(created.status, 201, "{}", created.raw);
        let id = created.body["pieces"][0]["piece_id"].as_str().unwrap();
        let home = get(&server, "/").await;
        assert!(home.raw.contains(&format!("<span>{channel}</span>")));
        let review = get(&server, &format!("/review/{id}")).await;
        assert_eq!(review.status, 200);
        assert!(
            review
                .raw
                .contains(&format!("class=\"eyebrow\">{channel}</p>"))
        );
        assert!(review.raw.contains(&format!("{channel} preview")));
        assert!(!review.raw.contains("Visual uploads remain contained."));
    }
}

#[tokio::test]
async fn mastodon_and_export_routes_inherit_session_host_and_origin_guards() {
    let provider = Scripted::new(text("unused"));
    let server = start(provider.clone()).await;
    for (method, path) in [
        (reqwest::Method::POST, "/v1/mastodon/sign-in"),
        (reqwest::Method::DELETE, "/v1/mastodon/connection"),
        (reqwest::Method::POST, "/v1/pieces/invalid/export"),
    ] {
        let request = client().request(method, server.url(path));
        let missing = answer(request.try_clone().unwrap().send().await.unwrap()).await;
        assert_eq!(missing.code(), "session_required");
        for (header, value, code) in [
            ("host", "evil.invalid", "host_not_allowed"),
            ("origin", "https://evil.invalid", "origin_not_allowed"),
            ("sec-fetch-site", "cross-site", "origin_not_allowed"),
        ] {
            let refused = answer(
                request
                    .try_clone()
                    .unwrap()
                    .header("x-syncloom-token", &server.token)
                    .header(header, value)
                    .send()
                    .await
                    .unwrap(),
            )
            .await;
            assert_eq!(refused.status, 403);
            assert_eq!(refused.code(), code);
            assert!(!refused.headers.contains_key("access-control-allow-origin"));
        }
    }
    assert!(provider.requests().is_empty());
}
