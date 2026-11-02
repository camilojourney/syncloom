#[path = "../../syncloom-core/tests/support/fake_meta.rs"]
mod fake_meta;
#[path = "../../syncloom-core/tests/support/meta_fixture.rs"]
mod meta_fixture;
mod support;
use meta_fixture::*;
use serde_json::json;
use std::sync::Arc;
use support::{
    scripted::{Scripted, text},
    *,
};
use syncloom_core::{Settings, Store, Studio, meta::MetaTarget};

// Independent JPEG frame-header check: Chromium must keep the specified portrait geometry.
fn jpeg_dimensions(bytes: &[u8]) -> Option<(u16, u16)> {
    let mut index = 2;
    while index + 9 < bytes.len() {
        if bytes[index] != 255 {
            return None;
        }
        if [192, 193, 194].contains(&bytes[index + 1]) {
            return Some((
                u16::from_be_bytes([bytes[index + 7], bytes[index + 8]]),
                u16::from_be_bytes([bytes[index + 5], bytes[index + 6]]),
            ));
        }
        let size = u16::from_be_bytes([bytes[index + 2], bytes[index + 3]]) as usize;
        if size < 2 {
            return None;
        }
        index += size + 2;
    }
    None
}
async fn start_meta(
    fake: &fake_meta::FakeMeta,
    granted: bool,
) -> (TestServer, Arc<syncloom_core::meta::Meta>, Arc<Media>) {
    let media = Arc::new(Media::default());
    let adapter = adapter(fake, memory_store::MemoryStore::new(), media.clone());
    let a = adapter.clone();
    let server = start_studio(
        Scripted::new(text("Build with measured evidence.")),
        Settings::default(),
        move |s| {
            let s = s.with_meta(a.clone());
            if granted {
                s.with_delivery("facebook", a.target(MetaTarget::Facebook))
                    .with_delivery("instagram", a.target(MetaTarget::Instagram))
            } else {
                s
            }
        },
    )
    .await;
    (server, adapter, media)
}
#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
async fn sign_in(server: &TestServer) {
    let start = post_empty(server, "/v1/meta/sign-in").await;
    assert_eq!(start.status, 202, "{}", start.raw);
    client()
        .get(start.body["authorization_url"].as_str().unwrap())
        .send()
        .await
        .unwrap();
    let finish = get(server, "/v1/meta/sign-in").await;
    assert_eq!(finish.body["status"], "signed_in", "{}", finish.raw);
    assert!(finish.body.get("expires_at").is_none());
    assert!(!finish.raw.contains(fake_meta::TOKEN));
}
async fn draft(server: &TestServer, channel: &str, unique: usize) -> serde_json::Value {
    let r=post(server,"/v1/thoughts",json!({"text":format!("Build with measured evidence version {unique}."),"channels":[channel]})).await;
    assert_eq!(r.status, 201, "{}", r.raw);
    r.body["pieces"][0].clone()
}
#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
async fn rendered(server: &TestServer, p: &serde_json::Value, count: usize) -> serde_json::Value {
    let path = format!("/v1/pieces/{}/render", p["piece_id"].as_str().unwrap());
    let r=post(server,&path,json!({"expected_revision":p["revision"],"template":"paper","slides":(0..count).map(|i|format!("Evidence slide {}",i+1)).collect::<Vec<_>>()})).await;
    assert_eq!(r.status, 200, "{}", r.raw);
    r.body
}
fn hashes(p: &serde_json::Value) -> serde_json::Value {
    json!({"image_sha256":p["assets"]["image_sha256"],"carousel_sha256":p["assets"]["carousel_sha256"],"slides_sha256":p["assets"]["slides_sha256"]})
}
#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
async fn approve_meta(server: &TestServer, p: &serde_json::Value) -> Answer {
    let mut body = json!({"status":"approved","expected_revision":p["revision"]});
    if !p["assets"].is_null() {
        body["expected_assets"] = hashes(p);
    }
    send_json(
        server,
        reqwest::Method::PATCH,
        &format!("/v1/pieces/{}", p["piece_id"].as_str().unwrap()),
        body,
    )
    .await
}
#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
async fn publish_meta(server: &TestServer, p: &serde_json::Value) -> Answer {
    post(
        server,
        &format!("/v1/pieces/{}/publish", p["piece_id"].as_str().unwrap()),
        json!({"expected_revision":p["revision"]}),
    )
    .await
}

#[tokio::test]
async fn meta_channels_follow_the_end_user_create_path() {
    let server = start(Scripted::new(text("Build with measured evidence."))).await;
    for channel in ["facebook_page", "instagram"] {
        let result = post(
            &server,
            "/v1/thoughts",
            json!({
                "text":THOUGHT, "channels":[channel]
            }),
        )
        .await;
        assert_eq!(result.status, 201, "{}", result.raw);
        assert_eq!(result.body["pieces"][0]["channel"], channel);
    }
}

#[tokio::test]
async fn http_consent_page_text_photo_instagram_image_carousel_and_restart_repeat_are_fake_only() {
    let fake = fake_meta::start(Default::default()).await;
    let (server, adapter, media) = start_meta(&fake, true).await;
    sign_in(&server).await;
    for (n, (channel, count)) in [
        ("facebook_page", 0),
        ("facebook_page", 1),
        ("instagram", 1),
        ("instagram", 2),
    ]
    .into_iter()
    .enumerate()
    {
        let mut p = draft(&server, channel, n).await;
        assert_eq!(publish_meta(&server, &p).await.code(), "approval_required");
        if count > 0 {
            p = rendered(&server, &p, count).await;
            let a = &p["assets"];
            assert_eq!(a["slides_sha256"].as_array().unwrap().len(), count);
            for i in 0..count {
                let bytes = client()
                    .get(server.url(&format!(
                        "/v1/pieces/{}/assets/slides/{i}",
                        p["piece_id"].as_str().unwrap()
                    )))
                    .header("x-syncloom-token", &server.token)
                    .send()
                    .await
                    .unwrap()
                    .bytes()
                    .await
                    .unwrap();
                assert!(bytes.starts_with(&[255, 216]));
                assert_eq!(jpeg_dimensions(&bytes), Some((1080, 1350)));
                assert_eq!(
                    syncloom_core::visual::digest(&bytes),
                    a["slides_sha256"][i].as_str().unwrap()
                );
                assert!(bytes.len() < 8 * 1024 * 1024);
            }
            for index in [
                count.to_string(),
                "-1".into(),
                "invalid".into(),
                "18446744073709551616".into(),
            ] {
                let refused = get(
                    &server,
                    &format!("/v1/pieces/{}/assets/slides/{index}", id_of(&p)),
                )
                .await;
                assert_eq!((refused.status, refused.code()), (409, "assets_required"));
            }
            let mut wrong = hashes(&p);
            wrong["slides_sha256"] = json!([]);
            let denied=send_json(&server,reqwest::Method::PATCH,&format!("/v1/pieces/{}",p["piece_id"].as_str().unwrap()),json!({"expected_revision":p["revision"],"status":"approved","expected_assets":wrong})).await;
            assert_eq!(denied.code(), "asset_conflict");
            let review = get(
                &server,
                &format!("/review/{}", p["piece_id"].as_str().unwrap()),
            )
            .await;
            assert!(review.raw.contains("Meta / JPEG"));
            assert!(review.raw.contains("class=\"preview-grid meta-slides\""));
        }
        assert_eq!(approve_meta(&server, &p).await.status, 200);
        let published = publish_meta(&server, &p).await;
        assert_eq!(published.body["status"], "posted", "{}", published.raw);
        assert_eq!(
            published.body["post_id"],
            if channel == "instagram" {
                "303"
            } else {
                "101_404"
            }
        );
        let calls = fake.all().len();
        let reopened = Studio::new(
            Store::open(server.dir.path()).unwrap(),
            Scripted::new(text("unused")),
            fixed_clock(),
            Settings::default(),
        )
        .with_delivery("facebook", adapter.target(MetaTarget::Facebook))
        .with_delivery("instagram", adapter.target(MetaTarget::Instagram));
        let again = reopened
            .publish(
                p["piece_id"].as_str().unwrap(),
                syncloom_core::PublishRequest {
                    expected_revision: p["revision"].as_str().unwrap().into(),
                    dry_run: false,
                    accepted_cost_microusd: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(again.status, "posted");
        assert!(!again.created);
        assert_eq!(fake.all().len(), calls);
    }
    assert_eq!(fake.calls("101/feed").len(), 1);
    assert_eq!(fake.calls("101/photos").len(), 1);
    let photo = &fake.calls("101/photos")[0];
    assert_eq!(photo.body["alt_text_custom"], "Evidence slide 1");
    assert_eq!(photo.body["caption"], "Build with measured evidence.");
    assert_eq!(fake.calls("202/media_publish").len(), 2);
    let images = media.images.lock().unwrap();
    assert_eq!(images.len(), 4);
    assert_eq!(images[0].0, "image/png");
    assert!(images[0].1.starts_with(b"\x89PNG"));
    let records =
        std::fs::read_to_string(server.dir.path().join("provenance/events.jsonl")).unwrap();
    for secret in [
        fake_meta::TOKEN,
        "fake-user-short",
        "fake-user-long",
        "fake-app-secret",
        "Build with measured evidence.",
    ] {
        assert!(!records.contains(secret));
    }
}

#[tokio::test]
async fn meta_connection_read_and_repeated_sign_out_use_only_the_store() {
    let fake = fake_meta::start(Default::default()).await;
    let (server, _, _) = start_meta(&fake, false).await;
    let empty = get(&server, "/v1/meta/connection").await;
    assert_eq!(empty.status, 200);
    assert_eq!(empty.body["status"], "signed_out");
    assert!(fake.all().is_empty());
    sign_in(&server).await;
    let calls = fake.all().len();
    let connected = get(&server, "/v1/meta/connection").await;
    assert_eq!(connected.status, 200);
    assert_eq!(connected.body["status"], "signed_in");
    assert!(!connected.raw.contains(fake_meta::TOKEN));
    for _ in 0..2 {
        let deleted = send_json(
            &server,
            reqwest::Method::DELETE,
            "/v1/meta/connection",
            serde_json::Value::Null,
        )
        .await;
        assert_eq!(deleted.status, 200);
        assert_eq!(deleted.body["status"], "signed_out");
    }
    assert_eq!(get(&server, "/v1/meta/connection").await.body, empty.body);
    assert_eq!(fake.all().len(), calls);
}

#[tokio::test]
async fn contained_and_dry_run_meta_intents_never_deliver() {
    let fake = fake_meta::start(Default::default()).await;
    let (server, _, media) = start_meta(&fake, false).await;
    for (n, c) in ["facebook_page", "instagram"].into_iter().enumerate() {
        let p = draft(&server, c, n).await;
        let dry = post(
            &server,
            &format!("/v1/pieces/{}/publish", p["piece_id"].as_str().unwrap()),
            json!({"expected_revision":p["revision"],"dry_run":true}),
        )
        .await;
        assert_eq!(dry.body["status"], "dry_run");
        assert_eq!(approve_meta(&server, &p).await.status, 200);
        assert_eq!(publish_meta(&server, &p).await.body["status"], "contained");
    }
    assert!(fake.all().is_empty());
    assert!(media.images.lock().unwrap().is_empty());
}

#[tokio::test]
async fn meta_routes_inherit_host_origin_and_session_guards() {
    let server = start(Scripted::new(text("unused"))).await;
    for (method, path) in [
        (reqwest::Method::POST, "/v1/meta/sign-in"),
        (reqwest::Method::GET, "/v1/meta/sign-in"),
        (reqwest::Method::GET, "/v1/meta/connection"),
        (reqwest::Method::DELETE, "/v1/meta/connection"),
        (reqwest::Method::GET, "/v1/pieces/invalid/assets/slides/0"),
    ] {
        let base = client().request(method, server.url(path));
        let absent = answer(base.try_clone().unwrap().send().await.unwrap()).await;
        if !matches!(
            base.try_clone().unwrap().build().unwrap().method(),
            &reqwest::Method::GET
        ) {
            assert_eq!(absent.code(), "session_required");
        } else {
            assert_eq!(absent.status, if path.ends_with("/0") { 404 } else { 503 });
        }
        for (header, value, code) in [
            ("host", "evil.invalid", "host_not_allowed"),
            ("origin", "https://evil.invalid", "origin_not_allowed"),
            ("sec-fetch-site", "cross-site", "origin_not_allowed"),
        ] {
            let denied = answer(
                base.try_clone()
                    .unwrap()
                    .header("x-syncloom-token", &server.token)
                    .header(header, value)
                    .send()
                    .await
                    .unwrap(),
            )
            .await;
            assert_eq!(denied.status, 403);
            assert_eq!(denied.code(), code);
            assert!(!denied.headers.contains_key("access-control-allow-origin"));
        }
    }
}

#[tokio::test]
async fn lost_instagram_publish_response_is_unknown_and_never_retried() {
    let fake = fake_meta::start(Default::default()).await;
    let (server, _, _) = start_meta(&fake, true).await;
    sign_in(&server).await;
    let p = rendered(&server, &draft(&server, "instagram", 0).await, 1).await;
    assert_eq!(approve_meta(&server, &p).await.status, 200);
    fake.configure(fake_meta::Config {
        fault_path: Some("202/media_publish".into()),
        fault_status: 500,
        ..Default::default()
    });
    assert_eq!(publish_meta(&server, &p).await.body["status"], "unknown");
    assert_eq!(publish_meta(&server, &p).await.body["status"], "unknown");
    assert_eq!(fake.calls("202/media_publish").len(), 1);
}

#[tokio::test]
async fn tampered_jpeg_and_reordered_approval_hashes_never_authorize_delivery() {
    let fake = fake_meta::start(Default::default()).await;
    let (server, _, media) = start_meta(&fake, true).await;
    let p = rendered(&server, &draft(&server, "instagram", 0).await, 2).await;
    let mut wrong = hashes(&p);
    wrong["slides_sha256"].as_array_mut().unwrap().reverse();
    let denied = send_json(
        &server,
        reqwest::Method::PATCH,
        &format!("/v1/pieces/{}", p["piece_id"].as_str().unwrap()),
        json!({"expected_revision":p["revision"],"status":"approved","expected_assets":wrong}),
    )
    .await;
    assert_eq!(denied.code(), "asset_conflict");
    assert_eq!(approve_meta(&server, &p).await.status, 200);
    let hash = p["assets"]["slides_sha256"][0].as_str().unwrap();
    std::fs::write(server.dir.path().join("assets").join(hash), b"tampered").unwrap();
    assert_eq!(publish_meta(&server, &p).await.code(), "asset_integrity");
    assert_eq!(approve_meta(&server, &p).await.code(), "asset_integrity");
    assert!(fake.all().is_empty());
    assert!(media.images.lock().unwrap().is_empty());
}
