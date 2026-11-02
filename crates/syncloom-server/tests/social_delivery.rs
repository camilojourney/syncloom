#[path = "../../syncloom-core/tests/support/fake_social.rs"]
mod fake;
mod support;
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use support::{
    scripted::{Scripted, text},
    *,
};
use syncloom_core::{
    Platform, PublishRequest, Settings, Store, Studio,
    social::{SocialConfig, SocialPlatforms},
};

#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
async fn setup(
    fake: &fake::Fake,
    platform: &str,
    granted: bool,
) -> (
    TestServer,
    Arc<SocialPlatforms>,
    Arc<memory_store::MemoryStore>,
) {
    let config = SocialConfig::parse(|n| match n {
        "SYNCLOOM_BLUESKY_BASE_URL"
        | "SYNCLOOM_X_API_BASE_URL"
        | "SYNCLOOM_X_AUTH_BASE_URL"
        | "SYNCLOOM_THREADS_API_BASE_URL"
        | "SYNCLOOM_THREADS_AUTH_BASE_URL" => Some(fake.base().into()),
        "SYNCLOOM_X_CLIENT_ID" | "SYNCLOOM_THREADS_CLIENT_ID" => Some("test-client".into()),
        "SYNCLOOM_THREADS_CLIENT_SECRET" => Some("test-secret-canary".into()),
        "SYNCLOOM_X_REDIRECT_URL" => Some(format!("http://127.0.0.1:{}/x/callback", free_port())),
        "SYNCLOOM_THREADS_REDIRECT_URL" => Some("https://redirect.invalid/callback".into()),
        _ => None,
    })
    .unwrap();
    let store = memory_store::MemoryStore::new();
    let q = Arc::new(SocialPlatforms::new(
        config,
        store.clone(),
        fixed_clock(),
        Duration::from_secs(2),
    ));
    let adapter = adapter(&q, platform);
    let attached = q.clone();
    let server = start_studio(
        Scripted::new(text("Build with evidence.")),
        Settings::default(),
        |s| {
            let s = s.with_social(attached);
            if granted {
                s.with_delivery(platform, adapter)
            } else {
                s
            }
        },
    )
    .await;
    (server, q, store)
}
#[expect(
    clippy::panic,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
fn adapter(q: &SocialPlatforms, platform: &str) -> Arc<dyn Platform> {
    match platform {
        "bluesky" => q.bluesky.clone(),
        "x" => q.x.clone(),
        "threads" => q.threads.clone(),
        _ => panic!("fixture platform"),
    }
}
#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
async fn login(server: &TestServer, platform: &str) {
    if platform == "bluesky" {
        let signed = post(
            server,
            "/v1/bluesky/sign-in",
            json!({"identifier":"test.invalid","app_password":"password-canary"}),
        )
        .await;
        assert_eq!(signed.body["status"], "signed_in", "{}", signed.raw);
        return;
    }
    let start = post_empty(server, &format!("/v1/{platform}/sign-in")).await;
    assert_eq!(start.status, 202, "{}", start.raw);
    let u = reqwest::Url::parse(start.body["authorization_url"].as_str().unwrap()).unwrap();
    let pairs: std::collections::HashMap<_, _> = u.query_pairs().into_owned().collect();
    let callback = format!(
        "{}?code=test-code&state={}",
        pairs["redirect_uri"], pairs["state"]
    );
    let signed = if platform == "x" {
        client().get(callback).send().await.unwrap();
        get(server, "/v1/x/sign-in").await
    } else {
        post(
            server,
            "/v1/threads/complete",
            json!({"callback_url":callback}),
        )
        .await
    };
    assert_eq!(signed.body["status"], "signed_in", "{}", signed.raw);
    if platform == "threads" {
        assert_eq!(
            signed.body["scope"], "threads_basic threads_content_publish",
            "Threads scope must come from verified token-debug evidence"
        );
    }
}
#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
async fn draft(server: &TestServer, platform: &str, image: bool) -> Value {
    let created = post(
        server,
        "/v1/thoughts",
        json!({"text":"Build with evidence.","channels":[platform]}),
    )
    .await;
    assert_eq!(created.status, 201, "{}", created.raw);
    let p = created.body["pieces"][0].clone();
    if !image {
        return p;
    }
    let rendered=post(server,&format!("/v1/pieces/{}/render",p["piece_id"].as_str().unwrap()),json!({"expected_revision":p["revision"],"template":"midnight","slides":["A chart with evidence."]})).await;
    assert_eq!(rendered.status, 200, "{}", rendered.raw);
    rendered.body
}
#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
async fn approve_piece(server: &TestServer, p: &Value) {
    let result=send_json(server,reqwest::Method::PATCH,&format!("/v1/pieces/{}",p["piece_id"].as_str().unwrap()),json!({"expected_revision":p["revision"],"status":"approved","expected_assets":p.get("assets").map(|a|json!({"image_sha256":a["image_sha256"],"carousel_sha256":a["carousel_sha256"]}))})).await;
    assert_eq!(result.status, 200, "{}", result.raw);
}
#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
fn publish_path(p: &Value) -> String {
    format!("/v1/pieces/{}/publish", p["piece_id"].as_str().unwrap())
}

#[tokio::test]
async fn each_platform_text_journey_obeys_grant_approval_and_durable_repeat_guard() {
    for platform in ["bluesky", "x", "threads"] {
        let fake = match platform {
            "bluesky" => fake::Fake::bluesky().await,
            "x" => fake::Fake::x().await,
            _ => fake::Fake::threads().await,
        };
        let (server, q, _) = setup(&fake, platform, true).await;
        login(&server, platform).await;
        let p = draft(&server, platform, false).await;
        let request = json!({"expected_revision":p["revision"],"accepted_cost_microusd":15000});
        let unapproved = post(&server, &publish_path(&p), request.clone()).await;
        assert_eq!(unapproved.code(), "approval_required");
        approve_piece(&server, &p).await;
        let result = post(&server, &publish_path(&p), request.clone()).await;
        assert_eq!(
            result.body["status"], "posted",
            "{platform}: {}",
            result.raw
        );
        let count = fake.calls().len();
        let reopened = Studio::new(
            Store::open(server.dir.path()).unwrap(),
            Scripted::new(text("unused")),
            fixed_clock(),
            Settings::default(),
        )
        .with_social(q.clone())
        .with_delivery(platform, adapter(&q, platform));
        let repeat = reopened
            .publish(
                p["piece_id"].as_str().unwrap(),
                PublishRequest {
                    expected_revision: p["revision"].as_str().unwrap().into(),
                    dry_run: false,
                    accepted_cost_microusd: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(repeat.status, "posted");
        assert!(!repeat.created);
        assert_eq!(repeat.attempts, 1);
        assert_eq!(fake.calls().len(), count);
        let events =
            std::fs::read_to_string(server.dir.path().join("provenance/events.jsonl")).unwrap();
        for canary in [
            "password-canary",
            "test-secret-canary",
            "x-access-1",
            "threads-long-1",
            "Build with evidence.",
        ] {
            assert!(!events.contains(canary));
        }
        let signed_out = send_json(
            &server,
            reqwest::Method::DELETE,
            &format!("/v1/{platform}/connection"),
            Value::Null,
        )
        .await;
        assert_eq!(signed_out.body["status"], "signed_out");
    }
}

#[tokio::test]
async fn x_cost_is_visible_before_delivery_requires_exact_confirmation_and_survives_outcome() {
    let fake = fake::Fake::x().await;
    let (server, _, _) = setup(&fake, "x", true).await;
    login(&server, "x").await;
    let p = draft(&server, "x", false).await;
    approve_piece(&server, &p).await;
    let page = client()
        .get(server.url(&format!("/review/{}", p["piece_id"].as_str().unwrap())))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(
        page.contains("USD 0.015"),
        "estimate must be shown before submit"
    );
    assert!(page.contains("accepted_cost_microusd"));
    let count = fake.calls().len();
    for value in [Value::Null, json!(14999), json!(15001)] {
        let denied = post(
            &server,
            &publish_path(&p),
            json!({"expected_revision":p["revision"],"accepted_cost_microusd":value}),
        )
        .await;
        assert_eq!(denied.code(), "cost_confirmation_required");
    }
    assert_eq!(fake.calls().len(), count);
    assert_eq!(
        std::fs::read_dir(server.dir.path().join("outbox"))
            .unwrap()
            .count(),
        0
    );
    let preview = post(
        &server,
        &publish_path(&p),
        json!({"expected_revision":p["revision"],"dry_run":true}),
    )
    .await;
    assert_eq!(preview.body["cost_estimate"]["microusd"], 15000);
    fake.reply("/2/tweets", 500, json!({"error":"lost outcome"}));
    let outcome = post(
        &server,
        &publish_path(&p),
        json!({"expected_revision":p["revision"],"accepted_cost_microusd":15000}),
    )
    .await;
    assert_eq!(outcome.body["status"], "unknown");
    assert_eq!(outcome.body["cost_estimate"], preview.body["cost_estimate"]);
    let count = fake.calls().len();
    let repeated = post(
        &server,
        &publish_path(&p),
        json!({"expected_revision":p["revision"]}),
    )
    .await;
    assert_eq!(repeated.body["status"], "unknown");
    assert_eq!(fake.calls().len(), count);
    let file = std::fs::read_dir(server.dir.path().join("outbox"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let saved: Value = serde_json::from_slice(&std::fs::read(file).unwrap()).unwrap();
    assert_eq!(saved["cost_estimate"]["microusd"], 15000);
}

#[tokio::test]
async fn numeric_domain_links_show_the_higher_quote_in_review_and_dry_run() {
    let fake = fake::Fake::x().await;
    let (server, _, _) = setup(&fake, "x", true).await;
    let p = draft(&server, "x", false).await;
    let id = p["piece_id"].as_str().unwrap();
    let edited = send_json(
        &server,
        reqwest::Method::PATCH,
        &format!("/v1/pieces/{id}"),
        json!({"expected_revision":p["revision"],"text":"Visit 123.example"}),
    )
    .await;
    assert_eq!(edited.status, 200);
    let page = client()
        .get(server.url(&format!("/review/{id}")))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(
        page.contains("USD 0.2"),
        "numeric domain must use the link estimate"
    );
    let preview = post(
        &server,
        &publish_path(&edited.body),
        json!({"expected_revision":edited.body["revision"],"dry_run":true}),
    )
    .await;
    assert_eq!(preview.body["cost_estimate"]["microusd"], 200000);
    assert!(fake.calls().is_empty());
}

#[tokio::test]
async fn bluesky_and_x_rendered_image_journeys_use_approved_bytes_and_alt_text() {
    for platform in ["bluesky", "x"] {
        let fake = if platform == "bluesky" {
            fake::Fake::bluesky().await
        } else {
            fake::Fake::x().await
        };
        let (server, _, _) = setup(&fake, platform, true).await;
        login(&server, platform).await;
        let p = draft(&server, platform, true).await;
        let refused = send_json(
            &server,
            reqwest::Method::PATCH,
            &format!("/v1/pieces/{}", p["piece_id"].as_str().unwrap()),
            json!({"expected_revision":p["revision"],"status":"approved"}),
        )
        .await;
        assert_eq!(refused.code(), "assets_required");
        approve_piece(&server, &p).await;
        let result = post(
            &server,
            &publish_path(&p),
            json!({"expected_revision":p["revision"],"accepted_cost_microusd":20000}),
        )
        .await;
        assert_eq!(
            result.body["status"], "posted",
            "{platform}: {}",
            result.raw
        );
        let calls = fake.calls();
        if platform == "bluesky" {
            let upload = calls
                .iter()
                .find(|c| c.path.ends_with("uploadBlob"))
                .unwrap();
            assert_eq!(upload.content_type, "image/png");
            assert!(upload.body.starts_with(b"\x89PNG\r\n\x1a\n"));
            let record = calls
                .iter()
                .find(|c| c.path.ends_with("createRecord"))
                .unwrap()
                .json();
            assert_eq!(
                record["record"]["embed"]["images"][0]["alt"],
                "A chart with evidence."
            );
        } else {
            let append = calls.iter().find(|c| c.path.ends_with("/append")).unwrap();
            assert!(append.body.windows(8).any(|w| w == b"\x89PNG\r\n\x1a\n"));
            assert_eq!(result.body["cost_estimate"]["microusd"], 20000);
        }
    }
}

#[tokio::test]
async fn threads_visual_review_disables_unavailable_image_publishing_visibly() {
    let fake = fake::Fake::threads().await;
    let (server, _, _) = setup(&fake, "threads", true).await;
    let p = draft(&server, "threads", true).await;
    approve_piece(&server, &p).await;
    let page = get(
        &server,
        &format!("/review/{}", p["piece_id"].as_str().unwrap()),
    )
    .await;
    assert!(page.raw.contains("Image delivery is unavailable"));
    assert!(
        page.raw
            .contains("class=\"publish\" type=\"submit\" disabled")
    );
    assert!(page.raw.contains("UTF-8 bytes"));
    assert!(fake.calls().is_empty());
}

#[tokio::test]
async fn absent_grants_and_bad_session_or_origin_never_reach_any_social_platform() {
    for platform in ["bluesky", "x", "threads"] {
        let fake = fake::Fake::x().await;
        let (server, _, _) = setup(&fake, platform, false).await;
        let p = draft(&server, platform, false).await;
        approve_piece(&server, &p).await;
        let outcome = post(
            &server,
            &publish_path(&p),
            json!({"expected_revision":p["revision"]}),
        )
        .await;
        assert_eq!(outcome.body["status"], "contained");
        assert!(fake.calls().is_empty());
        let denied = answer(
            client()
                .post(server.url(&format!("/v1/{platform}/refresh")))
                .send()
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(denied.code(), "session_required");
        let origin = answer(
            client()
                .post(server.url(&format!("/v1/{platform}/refresh")))
                .header("origin", "https://wrong.invalid")
                .header("x-syncloom-token", &server.token)
                .send()
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(origin.code(), "origin_not_allowed");
        assert!(fake.calls().is_empty());
    }
}

async fn complete_threads(server: &TestServer) -> Result<Answer, Box<dyn std::error::Error>> {
    let start = post_empty(server, "/v1/threads/sign-in").await;
    assert_eq!(start.status, 202);
    let url = reqwest::Url::parse(
        start.body["authorization_url"]
            .as_str()
            .ok_or("missing authorization URL")?,
    )?;
    let state = url
        .query_pairs()
        .find(|(k, _)| k == "state")
        .ok_or("missing state")?
        .1
        .into_owned();
    Ok(post(server, "/v1/threads/complete", json!({"callback_url":format!("https://redirect.invalid/callback?code=test&state={state}")})).await)
}

#[tokio::test]
async fn threads_fresh_exchange_requires_actual_effective_posting_permission() {
    for (debug, code) in [
        (
            json!({"data":{"is_valid":true,"user_id":"303","scopes":["threads_basic"]}}),
            "posting_permission_missing",
        ),
        (
            json!({"data":{"is_valid":true,"user_id":"303","scopes":["threads_content_publish_extra"]}}),
            "posting_permission_missing",
        ),
        (
            json!({"data":{"is_valid":true,"user_id":"303"}}),
            "posting_permission_unverified",
        ),
        (
            json!({"data":{"is_valid":false,"user_id":"303","scopes":["threads_content_publish"]}}),
            "posting_permission_unverified",
        ),
    ] {
        let fake = fake::Fake::threads().await;
        fake.reply("/v1.0/debug_token", 200, debug);
        let (server, q, store) = setup(&fake, "threads", true).await;
        let completed = complete_threads(&server).await.unwrap();
        assert_eq!(completed.code(), code, "{}", completed.raw);
        assert_eq!(
            fake.calls()
                .iter()
                .filter(|c| c.path == "/v1.0/debug_token")
                .count(),
            1
        );
        use syncloom_core::TokenStore;
        assert!(
            store
                .load(&q.account(syncloom_core::Channel::Threads).unwrap())
                .unwrap()
                .is_none()
        );
        assert!(
            !fake
                .calls()
                .iter()
                .any(|c| c.method == "POST" && c.path.contains("/v1.0/"))
        );
        for secret in ["test-secret-canary", "threads-long-1", "threads-short"] {
            assert!(!completed.raw.contains(secret));
        }
    }
}

#[tokio::test]
async fn saved_legacy_threads_requested_scopes_are_unverified_and_send_nothing() {
    use syncloom_core::{Channel, Token, TokenStore};
    let fake = fake::Fake::threads().await;
    let (server, q, store) = setup(&fake, "threads", true).await;
    // Deserialize the exact legacy shape: it contains no verified grant evidence.
    let token: Token = serde_json::from_value(json!({
        "access_token":"legacy-threads", "member_id":"303",
        "expires_at":"2026-12-29T18:00:00Z", "obtained_at":"2026-10-29T18:00:00Z",
        "scope":"threads_basic,threads_content_publish"
    }))
    .unwrap();
    store
        .save(&q.account(Channel::Threads).unwrap(), &token)
        .unwrap();
    let connection = get(&server, "/v1/threads/connection").await;
    let piece = draft(&server, "threads", false).await;
    approve_piece(&server, &piece).await;
    let published = post(
        &server,
        &publish_path(&piece),
        json!({"expected_revision":piece["revision"]}),
    )
    .await;
    assert_eq!(
        (
            connection.body["status"].as_str(),
            published.body["status"].as_str()
        ),
        (Some("cannot_post"), Some("posting_permission_unverified")),
        "connection={} publish={}",
        connection.raw,
        published.raw
    );
    assert_eq!(connection.body["reason"], "posting_permission_unverified");
    assert_eq!(connection.body["action"], "sign_in");
    assert!(fake.calls().is_empty());
    assert!(store.load(&q.account(Channel::Threads).unwrap()).unwrap() == Some(token));
}

#[tokio::test]
async fn narrowed_x_refresh_cannot_leave_the_old_full_grant_publish_ready() {
    let fake = fake::Fake::x().await;
    let (server, _, _) = setup(&fake, "x", true).await;
    login(&server, "x").await;
    fake.reply("/2/oauth2/token", 200, json!({"access_token":"reduced-rotation","refresh_token":"refresh-rotated","expires_in":7200,"token_type":"bearer","scope":"tweet.read users.read"}));
    let refreshed = post_empty(&server, "/v1/x/refresh").await;
    assert_eq!(refreshed.code(), "posting_permission_missing");
    let connection = get(&server, "/v1/x/connection").await;
    assert_eq!(
        connection.body["status"], "cannot_post",
        "{}",
        connection.raw
    );
    let piece = draft(&server, "x", false).await;
    approve_piece(&server, &piece).await;
    let before = fake.calls().len();
    let published = post(
        &server,
        &publish_path(&piece),
        json!({"expected_revision":piece["revision"],"accepted_cost_microusd":15000}),
    )
    .await;
    assert_eq!(published.body["status"], "posting_permission_missing");
    assert_eq!(fake.calls().len(), before);
}

async fn assert_threads_rotation_grant(
    debug: Value,
    expected: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use syncloom_core::{Channel, TokenStore};
    let fake = fake::Fake::threads().await;
    let (server, q, store) = setup(&fake, "threads", true).await;
    login(&server, "threads").await;
    let account = q.account(Channel::Threads)?;
    let mut old = store
        .load(&account)?
        .ok_or("missing connected credential")?;
    old.obtained_at = "2026-10-29T18:00:00Z".into();
    store.save(&account, &old)?;
    fake.reply("/v1.0/debug_token", 200, debug);
    let refreshed = post_empty(&server, "/v1/threads/refresh").await;
    assert_eq!(refreshed.code(), expected, "{}", refreshed.raw);
    let connection = get(&server, "/v1/threads/connection").await;
    assert_eq!(
        connection.body["status"], "cannot_post",
        "{}",
        connection.raw
    );
    assert_eq!(connection.body["reason"], expected);
    let piece = draft(&server, "threads", false).await;
    approve_piece(&server, &piece).await;
    let before = fake.calls().len();
    let published = post(
        &server,
        &publish_path(&piece),
        json!({"expected_revision":piece["revision"]}),
    )
    .await;
    assert_eq!(published.body["status"], expected);
    assert_eq!(fake.calls().len(), before);
    assert!(!refreshed.raw.contains("threads-long"));
    Ok(())
}

#[tokio::test]
async fn narrowed_threads_refresh_cannot_leave_the_old_full_grant_publish_ready() {
    assert_threads_rotation_grant(
        json!({"data":{"is_valid":true,"user_id":"303","scopes":["threads_basic"]}}),
        "posting_permission_missing",
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn unverified_threads_rotation_cannot_reuse_the_old_verified_grant() {
    assert_threads_rotation_grant(
        json!({"data":{"is_valid":true,"user_id":"303"}}),
        "posting_permission_unverified",
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn saved_x_grant_without_tweet_write_cannot_publish() {
    use syncloom_core::{Channel, Secret, Token, TokenStore};
    let fake = fake::Fake::x().await;
    let (server, q, store) = setup(&fake, "x", true).await;
    store
        .save(
            &q.account(Channel::X).unwrap(),
            &Token {
                access_token: Secret::new(
                    r#"{"access_token":"reduced-access","refresh_token":"reduced-refresh"}"#,
                ),
                expires_at: Some("2026-12-29T18:00:00Z".into()),
                member_id: String::new(),
                scope: "tweet.read".into(),
                scope_verified: true,
                obtained_at: "2026-10-30T18:00:00Z".into(),
            },
        )
        .unwrap();
    let connection = get(&server, "/v1/x/connection").await;
    let piece = draft(&server, "x", false).await;
    approve_piece(&server, &piece).await;
    let before = fake.calls().len();
    let published = post(
        &server,
        &publish_path(&piece),
        json!({"expected_revision": piece["revision"], "accepted_cost_microusd": 15000}),
    )
    .await;
    assert_eq!(
        (
            connection.body["status"].as_str(),
            published.body["status"].as_str(),
            fake.calls().len() - before
        ),
        (Some("cannot_post"), Some("posting_permission_missing"), 0),
        "connection={} publish={}",
        connection.raw,
        published.raw
    );
    assert_eq!(connection.body["reason"], "posting_permission_missing");
    assert_eq!(connection.body["action"], "sign_in");
}

#[tokio::test]
async fn fresh_x_text_grant_does_not_need_a_refresh_token_or_offline_permission() {
    let fake = fake::Fake::x().await;
    fake.reply("/2/oauth2/token", 200, json!({"access_token":"text-grant","expires_in":7200,"token_type":"bearer","scope":"tweet.read tweet.write users.read"}));
    let (server, _, _) = setup(&fake, "x", true).await;
    login(&server, "x").await;
    let piece = draft(&server, "x", false).await;
    approve_piece(&server, &piece).await;
    let published = post(
        &server,
        &publish_path(&piece),
        json!({"expected_revision":piece["revision"],"accepted_cost_microusd":15000}),
    )
    .await;
    assert_eq!(published.body["status"], "posted", "{}", published.raw);
    let before = fake.calls().len();
    let refreshed = post_empty(&server, "/v1/x/refresh").await;
    assert_eq!(refreshed.code(), "sign_in_rejected");
    assert_eq!(fake.calls().len(), before);
}

#[tokio::test]
async fn x_text_posting_does_not_require_media_or_offline_scope() {
    use syncloom_core::{Channel, Secret, Token, TokenStore};
    let fake = fake::Fake::x().await;
    let (server, q, store) = setup(&fake, "x", true).await;
    store
        .save(
            &q.account(Channel::X).unwrap(),
            &Token {
                access_token: Secret::new(
                    r#"{"access_token":"text-only-access","refresh_token":"text-only-refresh"}"#,
                ),
                expires_at: Some("2026-12-29T18:00:00Z".into()),
                member_id: String::new(),
                scope: "tweet.read tweet.write users.read".into(),
                scope_verified: true,
                obtained_at: "2026-10-30T18:00:00Z".into(),
            },
        )
        .unwrap();
    let connection = get(&server, "/v1/x/connection").await;
    assert_eq!(connection.body["status"], "signed_in", "{}", connection.raw);
    let piece = draft(&server, "x", false).await;
    approve_piece(&server, &piece).await;
    let published = post(
        &server,
        &publish_path(&piece),
        json!({"expected_revision": piece["revision"], "accepted_cost_microusd": 15000}),
    )
    .await;
    assert_eq!(published.body["status"], "posted", "{}", published.raw);
    let image = draft(&server, "x", true).await;
    approve_piece(&server, &image).await;
    let before = fake.calls().len();
    let refused = post(
        &server,
        &publish_path(&image),
        json!({"expected_revision": image["revision"], "accepted_cost_microusd": 20000}),
    )
    .await;
    assert_eq!(
        refused.body["status"], "posting_permission_missing",
        "{}",
        refused.raw
    );
    assert_eq!(fake.calls().len(), before);
}
