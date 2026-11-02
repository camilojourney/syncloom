#[path = "../../syncloom-core/tests/support/fake_social.rs"]
mod fake;
#[path = "../../syncloom-core/tests/support/social.rs"]
mod fixture;
#[path = "social_delivery/rotation.rs"]
mod rotation;
mod support;
use serde_json::{Value, json};
use std::sync::Arc;
use support::{
    scripted::{Scripted, text},
    *,
};
use syncloom_core::{Platform, PublishRequest, Settings, Store, Studio, social::SocialPlatforms};

async fn setup(
    fake: &fake::Fake,
    platform: &str,
    granted: bool,
) -> (
    TestServer,
    Arc<SocialPlatforms>,
    Arc<fixture::TestClock>,
    Arc<memory_store::MemoryStore>,
) {
    let clock = fixture::clock();
    let store = memory_store::MemoryStore::new();
    let q = Arc::new(fixture::social(fake, store.clone(), clock.clone()));
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
    (server, q, clock, store)
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
    let signed = complete_oauth(server, platform).await.unwrap();
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
async fn approve_piece(server: &TestServer, p: &Value) {
    let result = approve(server, p).await;
    assert_eq!(result.status, 200, "{}", result.raw);
}
#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
fn publish_path(p: &Value) -> String {
    format!("/v1/pieces/{}/publish", p["piece_id"].as_str().unwrap())
}

// HTTP refresh was previously tested only below the router. Exercise the same real
// server as delivery, with a mutable clock only at the external time boundary.
#[tokio::test]
async fn connection_refresh_and_idempotent_sign_out_cross_real_http_for_each_platform() {
    for platform in ["bluesky", "x", "threads"] {
        let fake = match platform {
            "bluesky" => fake::Fake::bluesky().await,
            "x" => fake::Fake::x().await,
            _ => fake::Fake::threads().await,
        };
        let (server, _, clock, _) = setup(&fake, platform, false).await;
        let connection = format!("/v1/{platform}/connection");
        let refresh = format!("/v1/{platform}/refresh");
        let signed_out = get(&server, &connection).await;
        assert_eq!(signed_out.status, 200);
        assert_eq!(signed_out.body["status"], "signed_out");
        let absent = post_empty(&server, &refresh).await;
        assert_eq!((absent.status, absent.code()), (502, "sign_in_rejected"));
        assert!(fake.calls().is_empty());

        login(&server, platform).await;
        let calls = fake.calls().len();
        let signed_in = get(&server, &connection).await;
        assert_eq!(signed_in.status, 200);
        assert_eq!(signed_in.body["status"], "signed_in");
        assert_eq!(
            fake.calls().len(),
            calls,
            "connection never calls a platform"
        );
        if platform == "threads" {
            let young = post_empty(&server, &refresh).await;
            assert_eq!((young.status, young.code()), (502, "sign_in_rejected"));
            assert_eq!(fake.calls().len(), calls);
            *clock.0.lock().unwrap() += time::Duration::days(1);
        }
        let (path, renewed) = match platform {
            "bluesky" => (
                "/xrpc/com.atproto.server.refreshSession",
                json!({
                    "accessJwt":fake::jwt(1793541600), "refreshJwt":"refresh-bsky-2",
                    "did":"did:plc:testrepo", "handle":"test.invalid"
                }),
            ),
            "x" => (
                "/2/oauth2/token",
                json!({
                    "access_token":"x-access-2", "refresh_token":"x-refresh-2", "expires_in":7200,
                    "scope":"tweet.read tweet.write users.read media.write offline.access", "token_type":"bearer"
                }),
            ),
            _ => (
                "/refresh_access_token",
                json!({
                    "access_token":"threads-long-2", "expires_in":5184000, "token_type":"bearer"
                }),
            ),
        };
        fake.reply(path, 200, renewed.clone());
        let refreshed = post_empty(&server, &refresh).await;
        assert_eq!(refreshed.status, 200, "{platform}: {}", refreshed.raw);
        assert_eq!(refreshed.body["status"], "signed_in");
        let identity_reads = usize::from(platform != "bluesky");
        let renewal_calls = 1 + identity_reads + usize::from(platform == "threads");
        assert_eq!(fake.calls().len(), calls + renewal_calls);
        assert_eq!(fake.calls()[calls].path, path);
        if platform == "threads" {
            assert_eq!(fake.calls()[calls + 1].path, "/v1.0/debug_token");
            assert!(
                fake.calls()[calls + 1]
                    .query
                    .contains("input_token=threads-long-2")
            );
        }
        if platform != "bluesky" {
            assert_eq!(
                fake.calls().last().unwrap().path,
                if platform == "x" {
                    "/2/users/me"
                } else {
                    "/v1.0/me"
                }
            );
        }
        assert_eq!(get(&server, &connection).await.body, refreshed.body);

        // A failed renewal must retain the last successful credentials, not the original
        // credentials and not a partially decoded replacement. No delivery is authorized.
        if platform == "threads" {
            *clock.0.lock().unwrap() += time::Duration::days(1);
        }
        fake.reply(path, 400, json!({"error":"renewal-secret-canary"}));
        let failed = post_empty(&server, &refresh).await;
        assert_eq!((failed.status, failed.code()), (502, "sign_in_rejected"));
        assert!(!failed.raw.contains("renewal-secret-canary"));
        assert_eq!(fake.calls().len(), calls + renewal_calls + 1);
        assert_eq!(get(&server, &connection).await.body, refreshed.body);
        fake.reply(path, 200, renewed);
        let retry = post_empty(&server, &refresh).await;
        assert_eq!(retry.status, 200, "{platform}: {}", retry.raw);
        assert_eq!(fake.calls().len(), calls + renewal_calls * 2 + 1);
        let last = fake
            .calls()
            .into_iter()
            .rev()
            .find(|call| call.path == path)
            .unwrap();
        match platform {
            "bluesky" => assert_eq!(last.authorization, "Bearer refresh-bsky-2"),
            "x" => assert_eq!(last.form()["refresh_token"], "x-refresh-2"),
            _ => assert!(last.query.contains("access_token=threads-long-2")),
        }
        for canary in [
            "refresh-bsky",
            "x-access",
            "x-refresh",
            "threads-long",
            "password-canary",
        ] {
            assert!(!retry.raw.contains(canary));
        }
        let calls = fake.calls().len();
        for _ in 0..2 {
            let deleted =
                send_json(&server, reqwest::Method::DELETE, &connection, Value::Null).await;
            assert_eq!(deleted.status, 200);
            assert_eq!(deleted.body["status"], "signed_out");
        }
        assert_eq!(get(&server, &connection).await.body["status"], "signed_out");
        let absent = post_empty(&server, &refresh).await;
        assert_eq!((absent.status, absent.code()), (502, "sign_in_rejected"));
        assert_eq!(
            fake.calls().len(),
            calls,
            "sign-out and absent renewal send nothing"
        );
        assert!(files(&server.dir, "pieces").is_empty());
        assert!(files(&server.dir, "outbox").is_empty());
    }
}

#[tokio::test]
async fn malformed_credentials_and_callback_refusals_never_reach_a_platform() {
    let fake = fake::Fake::bluesky().await;
    let (server, _, _, _) = setup(&fake, "bluesky", false).await;
    for body in [
        json!({}),
        json!({"identifier":"test.invalid"}),
        json!({"identifier":42,"app_password":"synthetic-password"}),
        json!({"identifier":"","app_password":"synthetic-password"}),
        json!({"identifier":"test.invalid","app_password":" "}),
        json!({"identifier":"synthetic@example.invalid","app_password":"synthetic-password"}),
        json!({"identifier":"test.invalid","app_password":"synthetic-password","extra":true}),
    ] {
        let refused = post(&server, "/v1/bluesky/sign-in", body).await;
        assert_eq!((refused.status, refused.code()), (400, "invalid_request"));
    }
    for body in [
        json!({}),
        json!({"callback_url":42}),
        json!({"callback_url":"synthetic", "extra":true}),
    ] {
        let refused = post(&server, "/v1/threads/complete", body).await;
        assert_eq!((refused.status, refused.code()), (400, "invalid_request"));
    }
    let not_started = post(
        &server,
        "/v1/threads/complete",
        json!({"callback_url":"synthetic"}),
    )
    .await;
    assert_eq!(
        (not_started.status, not_started.code()),
        (409, "sign_in_not_started")
    );
    let not_started = get(&server, "/v1/x/sign-in").await;
    assert_eq!(
        (not_started.status, not_started.code()),
        (409, "sign_in_not_started")
    );
    for suffix in ["wrong-state", "missing-code", "duplicate-state"] {
        let started = post_empty(&server, "/v1/threads/sign-in").await;
        assert_eq!(started.status, 202);
        let again = post_empty(&server, "/v1/threads/sign-in").await;
        assert_eq!((again.status, again.code()), (409, "sign_in_in_progress"));
        let url = reqwest::Url::parse(started.body["authorization_url"].as_str().unwrap()).unwrap();
        let state = url
            .query_pairs()
            .find(|(k, _)| k == "state")
            .unwrap()
            .1
            .into_owned();
        let (query, code) = match suffix {
            "wrong-state" => (
                "code=synthetic&state=wrong".to_owned(),
                "sign_in_state_mismatch",
            ),
            "missing-code" => (format!("state={state}"), "sign_in_denied"),
            _ => (
                format!("code=synthetic&state={state}&state={state}"),
                "sign_in_state_mismatch",
            ),
        };
        let refused = post(
            &server,
            "/v1/threads/complete",
            json!({"callback_url":format!("https://redirect.invalid/callback?{query}")}),
        )
        .await;
        assert_eq!((refused.status, refused.code()), (403, code));
        assert!(!refused.raw.contains(&state));
    }
    assert!(fake.calls().is_empty());
    assert_eq!(
        get(&server, "/v1/bluesky/connection").await.body["status"],
        "signed_out"
    );
    assert_eq!(
        get(&server, "/v1/threads/connection").await.body["status"],
        "signed_out"
    );
}

#[tokio::test]
async fn dynamic_connection_and_refresh_routes_refuse_unsupported_targets() {
    let fake = fake::Fake::x().await;
    let (server, _, _, _) = setup(&fake, "x", false).await;
    for target in ["unknown", "facebook", "bluesky_text", "%FF"] {
        for (method, operation) in [
            (reqwest::Method::GET, "connection"),
            (reqwest::Method::DELETE, "connection"),
            (reqwest::Method::POST, "refresh"),
        ] {
            let path = format!("/v1/{target}/{operation}");
            let refused = send_json(&server, method, &path, Value::Null).await;
            assert_eq!(
                (refused.status, refused.code()),
                (404, "not_found"),
                "{path}: {}",
                refused.raw
            );
        }
    }
    assert!(fake.calls().is_empty());
}

#[tokio::test]
async fn each_platform_text_journey_obeys_grant_approval_and_durable_repeat_guard() {
    for platform in ["bluesky", "x", "threads"] {
        let fake = match platform {
            "bluesky" => fake::Fake::bluesky().await,
            "x" => fake::Fake::x().await,
            _ => fake::Fake::threads().await,
        };
        let (server, q, _, _) = setup(&fake, platform, true).await;
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
async fn posted_and_unknown_intents_stay_with_their_target_after_fresh_approval() {
    for initial_status in [200, 500] {
        let fake = fake::Fake::bluesky().await;
        let (server, _, _, _) = setup(&fake, "bluesky", true).await;
        login(&server, "bluesky").await;
        let p = draft(&server, "bluesky", false).await;
        approve_piece(&server, &p).await;
        if initial_status == 500 {
            fake.reply(
                "/xrpc/com.atproto.repo.createRecord",
                500,
                json!({"error":"lost outcome"}),
            );
        }
        let a = post(
            &server,
            &publish_path(&p),
            json!({"expected_revision":p["revision"]}),
        )
        .await;
        assert_eq!(
            a.body["status"],
            if initial_status == 500 {
                "unknown"
            } else {
                "posted"
            }
        );
        send_json(
            &server,
            reqwest::Method::DELETE,
            "/v1/bluesky/connection",
            Value::Null,
        )
        .await;
        fake.reply("/xrpc/com.atproto.server.createSession", 200,
            json!({"accessJwt":fake::jwt(1793455200),"refreshJwt":"synthetic-second-session","did":"did:plc:second","handle":"second.invalid"}));
        fake.reply(
            "/xrpc/com.atproto.repo.createRecord",
            200,
            json!({"uri":"at://did:plc:second/app.bsky.feed.post/secondrecord","cid":fake::CID}),
        );
        login(&server, "bluesky").await;
        approve_piece(&server, &p).await;
        let b = post(
            &server,
            &publish_path(&p),
            json!({"expected_revision":p["revision"]}),
        )
        .await;
        let targets: Vec<_> = fake
            .calls()
            .into_iter()
            .filter(|c| c.path.ends_with("createRecord"))
            .map(|c| c.json()["repo"].clone())
            .collect();
        assert_eq!(
            targets,
            vec![json!("did:plc:testrepo"), json!("did:plc:second")],
            "{}",
            b.raw
        );
        assert_eq!(b.body["status"], "posted");
        assert_ne!(a.body["request_id"], b.body["request_id"]);
        assert_ne!(a.body["approval_target"], b.body["approval_target"]);
        assert_eq!(
            std::fs::read_dir(server.dir.path().join("outbox"))
                .unwrap()
                .count(),
            2
        );
    }
}

#[tokio::test]
async fn identity_mismatched_refresh_signs_out_without_rebinding_approval() {
    let fake = fake::Fake::bluesky().await;
    let (server, _, _, _) = setup(&fake, "bluesky", true).await;
    login(&server, "bluesky").await;
    fake.reply("/xrpc/com.atproto.server.refreshSession", 200,
        json!({"accessJwt":fake::jwt(1793541600),"refreshJwt":"synthetic-other","did":"did:plc:other","handle":"other.invalid"}));
    assert_eq!(
        post_empty(&server, "/v1/bluesky/refresh").await.code(),
        "sign_in_rejected"
    );
    assert_eq!(
        get(&server, "/v1/bluesky/connection").await.body["status"],
        "signed_out"
    );
}

#[tokio::test]
async fn approval_must_name_the_account_the_reviewer_saw() {
    let fake = fake::Fake::bluesky().await;
    let (server, _, _, _) = setup(&fake, "bluesky", true).await;
    login(&server, "bluesky").await;
    let p = draft(&server, "bluesky", false).await;
    let path = format!("/v1/pieces/{}", p["piece_id"].as_str().unwrap());
    let unnamed = send_json(
        &server,
        reqwest::Method::PATCH,
        &path,
        json!({"expected_revision":p["revision"],"status":"approved"}),
    )
    .await;
    assert_eq!(
        unnamed.code(),
        "account_approval_required",
        "{}",
        unnamed.raw
    );
    let observed = get(&server, &path).await.body["current_target"].clone();
    send_json(
        &server,
        reqwest::Method::DELETE,
        "/v1/bluesky/connection",
        Value::Null,
    )
    .await;
    second_bluesky(&fake);
    login(&server, "bluesky").await;
    let stale = send_json(
        &server,
        reqwest::Method::PATCH,
        &path,
        json!({"expected_revision":p["revision"],"expected_target":observed,"status":"approved"}),
    )
    .await;
    assert_eq!(stale.code(), "account_approval_required");
    assert_eq!(get(&server, &path).await.body["status"], "pending_review");
    approve_piece(&server, &p).await;
}

fn second_bluesky(fake: &fake::Fake) {
    fake.reply("/xrpc/com.atproto.server.createSession", 200,
        json!({"accessJwt":fake::jwt(1793455300),"refreshJwt":"synthetic-second-session","did":"did:plc:second","handle":"second.invalid"}));
}

#[tokio::test]
async fn a_claimed_media_attempt_keeps_its_target_during_reconnect_and_fresh_review()
-> Result<(), Box<dyn std::error::Error>> {
    let fake = fake::Fake::bluesky().await;
    let (server, _, _, _) = setup(&fake, "bluesky", true).await;
    login(&server, "bluesky").await;
    let p = draft(&server, "bluesky", true).await;
    approve_piece(&server, &p).await;
    let before = get(&server, &format!("/v1/pieces/{}", id_of(&p)))
        .await
        .body["approval_target"]
        .clone();
    let hold = fake.hold("/xrpc/com.atproto.repo.uploadBlob");
    let count = fake.calls().len();
    let url = server.url(&publish_path(&p));
    let token = server.token.clone();
    let revision = p["revision"].clone();
    let publishing = tokio::spawn(async move {
        client()
            .post(url)
            .header("x-syncloom-token", token)
            .json(&json!({"expected_revision":revision}))
            .send()
            .await
    });
    fake.wait_calls(count + 1).await;
    send_json(
        &server,
        reqwest::Method::DELETE,
        "/v1/bluesky/connection",
        Value::Null,
    )
    .await;
    second_bluesky(&fake);
    login(&server, "bluesky").await;
    approve_piece(&server, &p).await;
    let reviewed = get(&server, &format!("/v1/pieces/{}", id_of(&p))).await;
    assert_ne!(reviewed.body["approval_target"], before);
    hold.notify_one();
    let result: Value = publishing.await??.json().await?;
    assert_eq!(result["status"], "posted");
    assert_eq!(
        result["approval_target"], before,
        "receipt describes A, not concurrent B review"
    );
    let posts: Vec<_> = fake
        .calls()
        .into_iter()
        .filter(|c| c.path.ends_with("createRecord"))
        .collect();
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].json()["repo"], "did:plc:testrepo");
    assert_eq!(
        posts[0].authorization,
        format!("Bearer {}", fake::jwt(1793455200))
    );
    let events: Vec<Value> =
        std::fs::read_to_string(server.dir.path().join("provenance/events.jsonl"))?
            .lines()
            .map(serde_json::from_str)
            .collect::<Result<_, _>>()?;
    assert_eq!(
        events.last().ok_or("missing outcome")?["approval_target"],
        before
    );
    Ok(())
}

#[tokio::test]
async fn legacy_unbound_approval_requires_review_and_uncertain_history_cannot_be_bypassed()
-> Result<(), Box<dyn std::error::Error>> {
    for historical in [None, Some("unknown"), Some("posting"), Some("posted")] {
        let fake = fake::Fake::bluesky().await;
        let (server, _, _, _) = setup(&fake, "bluesky", true).await;
        login(&server, "bluesky").await;
        let p = draft(&server, "bluesky", false).await;
        approve_piece(&server, &p).await;
        if historical.is_some() {
            assert_eq!(
                post(
                    &server,
                    &publish_path(&p),
                    json!({"expected_revision":p["revision"]})
                )
                .await
                .body["status"],
                "posted"
            );
            let entry = std::fs::read_dir(server.dir.path().join("outbox"))?
                .next()
                .ok_or("missing intent")??;
            let mut old: Value = serde_json::from_slice(&std::fs::read(entry.path())?)?;
            old.as_object_mut()
                .ok_or("invalid intent")?
                .remove("approval_target");
            old["status"] = json!(historical);
            let key =
                syncloom_core::outbox::intent_key("publish", &id_of(&p), &revision_of(&p), None);
            std::fs::remove_file(entry.path())?;
            std::fs::write(
                server.dir.path().join("outbox").join(format!("{key}.json")),
                serde_json::to_vec(&old)?,
            )?;
        }
        let piece_path = server
            .dir
            .path()
            .join("pieces")
            .join(format!("{}.json", id_of(&p)));
        let mut old: Value = serde_json::from_slice(&std::fs::read(&piece_path)?)?;
        old.as_object_mut()
            .ok_or("invalid piece")?
            .remove("approval_target");
        std::fs::write(&piece_path, serde_json::to_vec(&old)?)?;
        let history = std::fs::read(server.dir.path().join("provenance/events.jsonl"))?;
        send_json(
            &server,
            reqwest::Method::DELETE,
            "/v1/bluesky/connection",
            Value::Null,
        )
        .await;
        second_bluesky(&fake);
        login(&server, "bluesky").await;
        let count = fake.calls().len();
        let page = get(&server, &format!("/review/{}", id_of(&p))).await;
        assert!(page.raw.contains("Review: Needs fresh approval"));
        assert!(page.raw.contains("Historical approval on <time"));
        assert!(!page.raw.contains("badge-approved\">Review: Approved"));
        assert_eq!(
            std::fs::read(server.dir.path().join("provenance/events.jsonl"))?,
            history
        );
        let refused = post(
            &server,
            &publish_path(&p),
            json!({"expected_revision":p["revision"]}),
        )
        .await;
        if let Some(status) = historical {
            assert_eq!(
                refused.body["status"],
                if status == "posting" {
                    "unknown"
                } else {
                    status
                }
            );
        } else {
            assert_eq!(refused.code(), "account_approval_required");
        }
        assert_eq!(fake.calls().len(), count);
        approve_piece(&server, &p).await;
        if let Some(status) = historical {
            let repeated = post(
                &server,
                &publish_path(&p),
                json!({"expected_revision":p["revision"]}),
            )
            .await;
            assert_eq!(
                repeated.body["status"],
                if status == "posting" {
                    "unknown"
                } else {
                    status
                }
            );
            assert!(
                repeated.body.get("approval_target").is_none(),
                "never fabricate legacy receipt identity"
            );
            assert_eq!(fake.calls().len(), count);
            assert_eq!(
                std::fs::read_dir(server.dir.path().join("outbox"))?.count(),
                1
            );
        }
        assert!(
            std::fs::read(server.dir.path().join("provenance/events.jsonl"))?.starts_with(&history)
        );
    }
    Ok(())
}

#[tokio::test]
async fn x_account_switch_requires_fresh_approval() {
    social_account_switch("x").await;
}
#[tokio::test]
async fn threads_account_switch_requires_fresh_approval() {
    social_account_switch("threads").await;
}
async fn social_account_switch(platform: &str) {
    {
        let fake = if platform == "x" {
            fake::Fake::x().await
        } else {
            fake::Fake::threads().await
        };
        let (server, _, _, _) = setup(&fake, platform, true).await;
        fake.principal("x-access-1", "111");
        fake.principal("threads-long-1", "303");
        login(&server, platform).await;
        let p = draft(&server, platform, false).await;
        approve_piece(&server, &p).await;
        send_json(
            &server,
            reqwest::Method::DELETE,
            &format!("/v1/{platform}/connection"),
            Value::Null,
        )
        .await;
        if platform == "x" {
            fake.reply("/2/oauth2/token", 200, json!({"access_token":"synthetic-x-second","refresh_token":"synthetic-refresh","expires_in":7200,"token_type":"bearer","scope":"tweet.read tweet.write users.read media.write offline.access"}));
            fake.reply("/2/users/me", 200, json!({"data":{"id":"222"}}));
            fake.principal("synthetic-x-second", "222");
        } else {
            fake.reply(
                "/oauth/access_token",
                200,
                json!({"access_token":"synthetic-short","user_id":606}),
            );
            fake.reply(
                "/access_token",
                200,
                json!({"access_token":"synthetic-threads-second","expires_in":5184000}),
            );
            fake.principal("synthetic-threads-second", "606");
            fake.reply("/v1.0/me", 200, json!({"id":"606"}));
            fake.reply("/v1.0/606/threads", 200, json!({"id":"404"}));
            fake.reply("/v1.0/606/threads_publish", 200, json!({"id":"505"}));
        }
        login(&server, platform).await;
        let published = post(
            &server,
            &publish_path(&p),
            json!({"expected_revision":p["revision"],"accepted_cost_microusd":15000}),
        )
        .await;
        let posts: Vec<_> = fake
            .calls()
            .into_iter()
            .filter(|c| c.path == "/2/tweets" || c.path.ends_with("/threads_publish"))
            .map(|c| (c.path, c.target_account))
            .collect();
        assert_eq!(
            published.code(),
            "account_approval_required",
            "{platform}: stale approval sent to {posts:?}; {}",
            published.raw
        );
        assert!(posts.is_empty());
        approve_piece(&server, &p).await;
        let fresh = post(
            &server,
            &publish_path(&p),
            json!({"expected_revision":p["revision"],"accepted_cost_microusd":15000}),
        )
        .await;
        assert_eq!(fresh.body["status"], "posted");
        let targets: Vec<_> = fake
            .calls()
            .into_iter()
            .filter(|c| c.path == "/2/tweets" || c.path.ends_with("/threads_publish"))
            .map(|c| c.target_account)
            .collect();
        assert_eq!(
            targets,
            vec![Some(if platform == "x" { "222" } else { "606" }.into())]
        );
    }
}

// Native forms must echo the displayed account and inspect the same durable intent as publish.
#[tokio::test]
async fn frontend_account_approval_and_target_scoped_receipts_survive_reconnect()
-> Result<(), Box<dyn std::error::Error>> {
    for posted_a in [false, true] {
        let fake = fake::Fake::bluesky().await;
        let (server, _, _, _) = setup(&fake, "bluesky", true).await;
        login(&server, "bluesky").await;
        let p = draft(&server, "bluesky", false).await;
        let id = p["piece_id"].as_str().ok_or("piece id")?;
        let revision = p["revision"].as_str().ok_or("revision")?;
        let path = format!("/review/{id}");
        let a = get(&server, &format!("/v1/pieces/{id}")).await;
        let target_a = a.body["current_target"].as_str().ok_or("A target")?;
        let page = get(&server, &path).await;
        assert!(
            page.raw
                .contains(&format!("name=\"expected_target\" value=\"{target_a}\"")),
            "displayed account must be submitted"
        );
        let approve_fields = [
            ("csrf", server.token.as_str()),
            ("expected_revision", revision),
            ("expected_target", target_a),
        ];
        let approved = answer(
            client()
                .post(server.url(&format!("{path}/approve")))
                .header("origin", server.url(""))
                .form(&approve_fields)
                .send()
                .await?,
        )
        .await;
        assert_eq!(approved.status, 200);
        assert!(approved.raw.contains("Approved on <time"));
        assert!(approved.raw.contains("badge-approved\">Review: Approved"));
        assert!(!approved.raw.contains("Historical approval"));
        assert!(!approved.raw.contains(&format!("action=\"{path}/approve\"")));
        let publish_fields = [
            ("csrf", server.token.as_str()),
            ("expected_revision", revision),
        ];
        let mut first_request = None;
        if posted_a {
            let sent = answer(
                client()
                    .post(server.url(&format!("{path}/publish")))
                    .header("origin", server.url(""))
                    .form(&publish_fields)
                    .send()
                    .await?,
            )
            .await;
            assert!(sent.raw.contains("<h2>Posted</h2>"));
            first_request = Some(
                post(
                    &server,
                    &publish_path(&p),
                    json!({"expected_revision":revision}),
                )
                .await
                .body["request_id"]
                    .clone(),
            );
        }
        send_json(
            &server,
            reqwest::Method::DELETE,
            "/v1/bluesky/connection",
            Value::Null,
        )
        .await;
        fake.reply("/xrpc/com.atproto.server.createSession", 200, json!({"accessJwt":fake::jwt(1793455200),"refreshJwt":"synthetic-second-session","did":"did:plc:second","handle":"second.invalid"}));
        fake.reply(
            "/xrpc/com.atproto.repo.createRecord",
            200,
            json!({"uri":"at://did:plc:second/app.bsky.feed.post/secondrecord","cid":fake::CID}),
        );
        login(&server, "bluesky").await;
        let stale = answer(
            client()
                .post(server.url(&format!("{path}/approve")))
                .header("origin", server.url(""))
                .form(&approve_fields)
                .send()
                .await?,
        )
        .await;
        assert_eq!(stale.status, 409);
        assert!(stale.raw.contains("account_approval_required"));
        let b = get(&server, &format!("/v1/pieces/{id}")).await;
        let target_b = b.body["current_target"].as_str().ok_or("B target")?;
        assert_ne!(target_a, target_b);
        assert_eq!(b.body["approval_target"], target_a);
        let page = get(&server, &path).await;
        assert!(
            page.raw
                .contains("badge-pending\">Review: Needs fresh approval")
        );
        assert!(!page.raw.contains("badge-approved\">Review: Approved"));
        assert!(page.raw.contains("Historical approval on <time"));
        assert!(!page.raw.contains("<p>Approved on <time"));
        let explanation = page
            .raw
            .find("A fresh approval is needed for the connected account.")
            .ok_or("missing plain account explanation")?;
        let details = page
            .raw
            .find("<details class=\"account-details\">")
            .ok_or("missing technical account details")?;
        assert!(
            explanation < details,
            "plain explanation precedes closed technical details"
        );
        assert!(page.raw.contains(target_b));
        assert!(
            page.raw
                .contains("A new approval will apply to this connected account fingerprint.")
        );
        assert!(
            !page
                .raw
                .contains("Approval is for this exact account fingerprint.")
        );
        assert!(page.raw.contains("Approve this version"));
        assert!(
            page.raw
                .contains("class=\"publish\" type=\"submit\" disabled>Publish")
        );
        let before = fake
            .calls()
            .into_iter()
            .filter(|c| c.path.ends_with("createRecord"))
            .count();
        if !posted_a {
            let denied = answer(
                client()
                    .post(server.url(&format!("{path}/publish")))
                    .header("origin", server.url(""))
                    .form(&publish_fields)
                    .send()
                    .await?,
            )
            .await;
            assert_eq!(denied.status, 409);
        }
        assert_eq!(
            fake.calls()
                .into_iter()
                .filter(|c| c.path.ends_with("createRecord"))
                .count(),
            before
        );
        let fresh_fields = [
            ("csrf", server.token.as_str()),
            ("expected_revision", revision),
            ("expected_target", target_b),
        ];
        let fresh = answer(
            client()
                .post(server.url(&format!("{path}/approve")))
                .header("origin", server.url(""))
                .form(&fresh_fields)
                .send()
                .await?,
        )
        .await;
        assert_eq!(fresh.status, 200);
        assert!(fresh.raw.contains("2026-10-30</time>"));
        assert!(fresh.raw.contains("badge-approved\">Review: Approved"));
        assert!(fresh.raw.contains("<p>Approved on <time"));
        assert!(!fresh.raw.contains("Historical approval"));
        assert!(!fresh.raw.contains(&format!("action=\"{path}/approve\"")));
        if let Some(old) = first_request {
            assert!(
                !fresh.raw.contains(old.as_str().ok_or("A request")?),
                "new target must not inherit A receipt"
            );
        }
        let sent = answer(
            client()
                .post(server.url(&format!("{path}/publish")))
                .header("origin", server.url(""))
                .form(&publish_fields)
                .send()
                .await?,
        )
        .await;
        assert_eq!(sent.status, 200);
        assert!(sent.raw.contains("<h2>Posted</h2>"));
        let repeated = post(
            &server,
            &publish_path(&p),
            json!({"expected_revision":revision}),
        )
        .await;
        let request = repeated.body["request_id"].as_str().ok_or("B request")?;
        assert!(sent.raw.contains(request));
        assert!(sent.raw.contains("disabled>Publish"));
        let targets: Vec<_> = fake
            .calls()
            .into_iter()
            .filter(|c| c.path.ends_with("createRecord"))
            .map(|c| c.json()["repo"].clone())
            .collect();
        let mut expected = Vec::new();
        if posted_a {
            expected.push(json!("did:plc:testrepo"));
        }
        expected.push(json!("did:plc:second"));
        assert_eq!(targets, expected);
        post(
            &server,
            &format!("/v1/pieces/{id}/export"),
            json!({"expected_revision":revision,"target":"reddit"}),
        )
        .await;
        let exported = get(&server, &path).await;
        assert!(exported.raw.contains(request));
        assert!(exported.raw.contains("<h2>Posted</h2>"));
        assert!(exported.raw.contains("disabled>Publish"));
    }
    Ok(())
}

// The only counterfactual is the authenticated DID; content and revision stay fixed.
#[tokio::test]
async fn bluesky_account_switch_requires_fresh_approval_before_any_post() {
    for switched in [false, true] {
        let fake = fake::Fake::bluesky().await;
        let (server, _, _, _) = setup(&fake, "bluesky", true).await;
        login(&server, "bluesky").await;
        let p = draft(&server, "bluesky", false).await;
        approve_piece(&server, &p).await;
        if switched {
            let removed = send_json(
                &server,
                reqwest::Method::DELETE,
                "/v1/bluesky/connection",
                Value::Null,
            )
            .await;
            assert_eq!(removed.body["status"], "signed_out");
            fake.reply("/xrpc/com.atproto.server.createSession", 200,
                json!({"accessJwt":fake::jwt(1793455200),"refreshJwt":"synthetic-second-session","did":"did:plc:second","handle":"second.invalid"}));
            fake.reply("/xrpc/com.atproto.repo.createRecord", 200,
                json!({"uri":"at://did:plc:second/app.bsky.feed.post/secondrecord","cid":fake::CID}));
            let signed = post(
                &server,
                "/v1/bluesky/sign-in",
                json!({"identifier":"second.invalid","app_password":"synthetic-second-password"}),
            )
            .await;
            assert_eq!(signed.body["status"], "signed_in");
        }
        let published = post(
            &server,
            &publish_path(&p),
            json!({"expected_revision":p["revision"]}),
        )
        .await;
        let posts: Vec<_> = fake
            .calls()
            .into_iter()
            .filter(|c| c.path.ends_with("createRecord"))
            .map(|c| c.json()["repo"].clone())
            .collect();
        if switched {
            assert_eq!(
                published.code(),
                "account_approval_required",
                "old account approval must refuse; actual targets={posts:?}; response={}",
                published.raw
            );
            assert!(posts.is_empty());
            approve_piece(&server, &p).await;
            let fresh = post(
                &server,
                &publish_path(&p),
                json!({"expected_revision":p["revision"]}),
            )
            .await;
            assert_eq!(fresh.body["status"], "posted");
            let targets: Vec<_> = fake
                .calls()
                .into_iter()
                .filter(|c| c.path.ends_with("createRecord"))
                .map(|c| c.json()["repo"].clone())
                .collect();
            assert_eq!(targets, vec![json!("did:plc:second")]);
        } else {
            assert_eq!(published.body["status"], "posted");
            assert_eq!(posts, vec![json!("did:plc:testrepo")]);
        }
    }
}

#[tokio::test]
async fn x_cost_is_visible_before_delivery_requires_exact_confirmation_and_survives_outcome() {
    let fake = fake::Fake::x().await;
    let (server, _, _, _) = setup(&fake, "x", true).await;
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
    let (server, _, _, _) = setup(&fake, "x", true).await;
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
        let (server, _, _, _) = setup(&fake, platform, true).await;
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
    let (server, _, _, _) = setup(&fake, "threads", true).await;
    let p = draft(&server, "threads", true).await;
    approve_piece(&server, &p).await;
    let page = get(
        &server,
        &format!("/review/{}", p["piece_id"].as_str().unwrap()),
    )
    .await;
    assert_eq!(page.status, 200);
    assert!(page.raw.contains("Connect the intended account"));
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
        let (server, _, _, _) = setup(&fake, platform, false).await;
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

async fn complete_oauth(
    server: &TestServer,
    platform: &str,
) -> Result<Answer, Box<dyn std::error::Error>> {
    let start = post_empty(server, &format!("/v1/{platform}/sign-in")).await;
    assert_eq!(start.status, 202, "{}", start.raw);
    let url = reqwest::Url::parse(
        start.body["authorization_url"]
            .as_str()
            .ok_or("missing authorization URL")?,
    )?;
    let pairs: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
    let callback = format!(
        "{}?code=test-code&state={}",
        pairs.get("redirect_uri").ok_or("missing redirect")?,
        pairs.get("state").ok_or("missing state")?
    );
    if platform == "x" {
        client().get(callback).send().await?;
        Ok(get(server, "/v1/x/sign-in").await)
    } else {
        Ok(post(
            server,
            "/v1/threads/complete",
            json!({"callback_url":callback}),
        )
        .await)
    }
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
        let (server, q, _, store) = setup(&fake, "threads", true).await;
        let completed = complete_oauth(&server, "threads").await.unwrap();
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
    let (server, q, _, store) = setup(&fake, "threads", true).await;
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
    let (server, _, _, _) = setup(&fake, "x", true).await;
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
    let (server, q, _, store) = setup(&fake, "threads", true).await;
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
    let (server, q, _, store) = setup(&fake, "x", true).await;
    store
        .save(
            &q.account(Channel::X).unwrap(),
            &Token {
                access_token: Secret::new(
                    r#"{"access_token":"reduced-access","refresh_token":"reduced-refresh"}"#,
                ),
                expires_at: Some("2026-12-29T18:00:00Z".into()),
                member_id: "111".into(),
                identity_authority: None,
                page_id: None,
                scope: "tweet.read".into(),
                scope_verified: true,
                identity_confirmed: true,
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

async fn assert_permission_review(
    scope: &str,
    verified: bool,
    expected: &str,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use syncloom_core::{Channel, TokenStore};
    let fake = fake::Fake::threads().await;
    let (server, q, _, store) = setup(&fake, "threads", true).await;
    login(&server, "threads").await;
    let piece = draft(&server, "threads", false).await;
    approve_piece(&server, &piece).await;
    let page_path = format!(
        "/review/{}",
        piece["piece_id"].as_str().ok_or("missing piece")?
    );
    let disabled = |html: &str| {
        html.split_once("<button class=\"publish\"")
            .and_then(|(_, tail)| tail.split_once('>'))
            .map(|(attrs, _)| attrs.contains(" disabled"))
    };
    let full = get(&server, &page_path).await;
    assert_eq!(full.status, 200);
    assert_eq!(
        disabled(&full.raw),
        Some(false),
        "full verified grant stays publish-ready"
    );
    let account = q.account(Channel::Threads)?;
    let mut token = store.load(&account)?.ok_or("missing initial grant")?;
    token.scope = scope.into();
    token.scope_verified = verified;
    store.save(&account, &token)?;
    let before = fake.calls().len();
    let denied = get(&server, &page_path).await;
    assert_eq!(denied.status, 200);
    assert_eq!(disabled(&denied.raw), Some(true));
    assert!(denied.raw.contains("connected, cannot post:"));
    assert!(denied.raw.contains("Sign in again to reconnect."));
    let published = post(
        &server,
        &publish_path(&piece),
        json!({"expected_revision":piece["revision"]}),
    )
    .await;
    assert_eq!(published.body["status"], expected);
    let receipt = get(&server, &page_path).await;
    assert_eq!(disabled(&receipt.raw), Some(true));
    assert!(receipt.raw.contains(label));
    assert!(
        !receipt.raw.contains("Sending not confirmed"),
        "permission refusal is not an unknown effect"
    );
    assert_eq!(
        fake.calls().len(),
        before,
        "UI inspection and denied publish send no platform request"
    );
    Ok(())
}

#[tokio::test]
async fn permission_missing_review_disables_publish_and_explains_saved_refusal()
-> Result<(), Box<dyn std::error::Error>> {
    assert_permission_review(
        "threads_basic",
        true,
        "posting_permission_missing",
        "Posting permission missing",
    )
    .await
}

#[tokio::test]
async fn permission_unverified_review_disables_publish_without_claiming_an_unknown_effect()
-> Result<(), Box<dyn std::error::Error>> {
    assert_permission_review(
        "threads_basic threads_content_publish",
        false,
        "posting_permission_unverified",
        "Posting permission unverified",
    )
    .await
}

async fn assert_x_scope_case(
    scope: Option<Value>,
    refresh: bool,
    expected: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use syncloom_core::{Channel, TokenStore};
    let fake = fake::Fake::x().await;
    let (server, q, _, store) = setup(&fake, "x", true).await;
    if refresh {
        login(&server, "x").await;
    }
    let account = q.account(Channel::X)?;
    let prior = store.load(&account)?;
    let mut issued = json!({"access_token":"scope-new-access","refresh_token":"scope-new-refresh","expires_in":7200,"token_type":"bearer"});
    if let Some(value) = scope.clone() {
        issued["scope"] = value;
    }
    fake.reply("/2/oauth2/token", 200, issued);
    let response = if refresh {
        post_empty(&server, "/v1/x/refresh").await
    } else {
        complete_oauth(&server, "x").await?
    };
    let connection = get(&server, "/v1/x/connection").await;
    let saved = store.load(&account)?;
    let piece = draft(&server, "x", false).await;
    approve_piece(&server, &piece).await;
    let before_write = fake.calls().len();
    let published = post(
        &server,
        &publish_path(&piece),
        json!({"expected_revision":piece["revision"],"accepted_cost_microusd":15000}),
    )
    .await;
    let writes = fake.calls().len() - before_write;
    let ready = expected == "posted";
    let connection_status = if ready {
        "signed_in"
    } else if refresh {
        "cannot_post"
    } else {
        "signed_out"
    };
    let publish_status = if ready {
        "posted"
    } else if refresh {
        expected
    } else {
        "consent_required"
    };
    assert_eq!(
        (
            response.code(),
            connection.body["status"].as_str(),
            published.body["status"].as_str(),
            writes
        ),
        (
            if ready { "" } else { expected },
            Some(connection_status),
            Some(publish_status),
            usize::from(ready)
        ),
        "scope={scope:?} refresh={refresh} response={} connection={} publish={}",
        response.raw,
        connection.raw,
        published.raw
    );
    assert_eq!(response.status, if ready { 200 } else { 403 });
    assert_eq!(connection.status, 200);
    assert_eq!(
        fake.calls()
            .iter()
            .filter(|call| call.path == "/2/oauth2/token")
            .count(),
        if refresh { 2 } else { 1 }
    );
    if ready || refresh {
        let token = saved.ok_or("missing actual rotated credential")?;
        let bundle: Value = serde_json::from_str(token.access_token.expose())?;
        assert_eq!(bundle["access_token"], "scope-new-access");
        assert_eq!(bundle["refresh_token"], "scope-new-refresh");
        if expected == "posting_permission_unverified" {
            assert!(token.scope.is_empty());
            assert!(!token.scope_verified);
            assert_eq!(connection.body["reason"], expected);
            assert_eq!(connection.body["action"], "sign_in");
        } else {
            let expected_scope = scope
                .as_ref()
                .and_then(Value::as_str)
                .or_else(|| prior.as_ref().map(|token| token.scope.as_str()))
                .ok_or("missing known scope")?;
            assert_eq!(token.scope, expected_scope);
            assert!(token.scope_verified);
        }
    } else {
        assert!(saved.is_none());
    }
    if ready {
        assert_eq!(
            fake.calls().last().ok_or("missing post")?.authorization,
            "Bearer scope-new-access"
        );
    }
    Ok(())
}

#[tokio::test]
async fn x_scope_unknown_prior_is_refused_before_another_refresh()
-> Result<(), Box<dyn std::error::Error>> {
    use syncloom_core::{Channel, TokenStore};
    let fake = fake::Fake::x().await;
    let (server, q, _, store) = setup(&fake, "x", true).await;
    login(&server, "x").await;
    fake.reply("/2/oauth2/token", 200, json!({"access_token":"unknown-first","refresh_token":"unknown-refresh-first","expires_in":7200,"token_type":"bearer","scope":null}));
    let first = post_empty(&server, "/v1/x/refresh").await;
    assert_eq!(first.code(), "posting_permission_unverified");
    fake.reply("/2/oauth2/token", 200, json!({"access_token":"unknown-second","refresh_token":"unknown-refresh-second","expires_in":7200,"token_type":"bearer"}));
    let second = post_empty(&server, "/v1/x/refresh").await;
    let connection = get(&server, "/v1/x/connection").await;
    let piece = draft(&server, "x", false).await;
    approve_piece(&server, &piece).await;
    let writes_before = fake.calls().len();
    let published = post(
        &server,
        &publish_path(&piece),
        json!({"expected_revision":piece["revision"],"accepted_cost_microusd":15000}),
    )
    .await;
    assert_eq!(fake.calls().len(), writes_before);
    assert_eq!(
        (
            second.code(),
            connection.body["reason"].as_str(),
            published.body["status"].as_str()
        ),
        (
            "posting_permission_unverified",
            Some("posting_permission_unverified"),
            Some("posting_permission_unverified")
        )
    );
    assert_eq!(second.status, 403);
    let token = store
        .load(&q.account(Channel::X)?)?
        .ok_or("missing rotated token")?;
    assert!(token.scope.is_empty());
    assert!(!token.scope_verified);
    let bundle: Value = serde_json::from_str(token.access_token.expose())?;
    assert_eq!(bundle["access_token"], "unknown-first");
    assert_eq!(bundle["refresh_token"], "unknown-refresh-first");
    assert_eq!(
        fake.calls()
            .iter()
            .filter(|call| call.path == "/2/oauth2/token")
            .count(),
        2
    );
    Ok(())
}

#[tokio::test]
async fn x_scope_refresh_absent() -> Result<(), Box<dyn std::error::Error>> {
    assert_x_scope_case(None, true, "posted").await
}

#[tokio::test]
async fn x_scope_fresh_absent() -> Result<(), Box<dyn std::error::Error>> {
    assert_x_scope_case(None, false, "posting_permission_unverified").await
}

#[tokio::test]
async fn x_scope_fresh_null() -> Result<(), Box<dyn std::error::Error>> {
    assert_x_scope_case(Some(json!(null)), false, "posting_permission_unverified").await
}

#[tokio::test]
async fn x_scope_fresh_empty() -> Result<(), Box<dyn std::error::Error>> {
    assert_x_scope_case(Some(json!("")), false, "posting_permission_unverified").await
}

#[tokio::test]
async fn x_scope_fresh_malformed() -> Result<(), Box<dyn std::error::Error>> {
    assert_x_scope_case(
        Some(json!(["tweet.write", "users.read"])),
        false,
        "posting_permission_unverified",
    )
    .await
}

#[tokio::test]
async fn x_scope_refresh_null() -> Result<(), Box<dyn std::error::Error>> {
    assert_x_scope_case(Some(json!(null)), true, "posting_permission_unverified").await
}

#[tokio::test]
async fn x_scope_refresh_empty() -> Result<(), Box<dyn std::error::Error>> {
    assert_x_scope_case(Some(json!("")), true, "posting_permission_unverified").await
}

#[tokio::test]
async fn x_scope_refresh_malformed() -> Result<(), Box<dyn std::error::Error>> {
    assert_x_scope_case(
        Some(json!(["tweet.write", "users.read"])),
        true,
        "posting_permission_unverified",
    )
    .await
}

#[tokio::test]
async fn x_scope_refresh_number() -> Result<(), Box<dyn std::error::Error>> {
    assert_x_scope_case(Some(json!(42)), true, "posting_permission_unverified").await
}

#[tokio::test]
async fn x_scope_refresh_object() -> Result<(), Box<dyn std::error::Error>> {
    assert_x_scope_case(
        Some(json!({"scope":"tweet.write users.read"})),
        true,
        "posting_permission_unverified",
    )
    .await
}

#[tokio::test]
async fn x_scope_refresh_whitespace() -> Result<(), Box<dyn std::error::Error>> {
    assert_x_scope_case(Some(json!("   ")), true, "posting_permission_unverified").await
}

#[tokio::test]
async fn x_scope_refresh_control_separator() -> Result<(), Box<dyn std::error::Error>> {
    assert_x_scope_case(
        Some(json!("tweet.write\nusers.read")),
        true,
        "posting_permission_unverified",
    )
    .await
}

#[tokio::test]
async fn x_scope_refresh_comma_separator() -> Result<(), Box<dyn std::error::Error>> {
    assert_x_scope_case(
        Some(json!("tweet.write,users.read")),
        true,
        "posting_permission_unverified",
    )
    .await
}

#[tokio::test]
async fn x_scope_refresh_present_full() -> Result<(), Box<dyn std::error::Error>> {
    assert_x_scope_case(Some(json!("tweet.write users.read")), true, "posted").await
}

#[tokio::test]
async fn x_scope_refresh_present_reduced() -> Result<(), Box<dyn std::error::Error>> {
    assert_x_scope_case(
        Some(json!("tweet.read")),
        true,
        "posting_permission_missing",
    )
    .await
}

#[tokio::test]
async fn fresh_x_text_grant_does_not_need_a_refresh_token_or_offline_permission() {
    let fake = fake::Fake::x().await;
    fake.reply("/2/oauth2/token", 200, json!({"access_token":"text-grant","expires_in":7200,"token_type":"bearer","scope":"tweet.read tweet.write users.read"}));
    let (server, _, _, _) = setup(&fake, "x", true).await;
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
    let (server, q, _, store) = setup(&fake, "x", true).await;
    store
        .save(
            &q.account(Channel::X).unwrap(),
            &Token {
                access_token: Secret::new(
                    r#"{"access_token":"text-only-access","refresh_token":"text-only-refresh"}"#,
                ),
                expires_at: Some("2026-12-29T18:00:00Z".into()),
                member_id: "111".into(),
                identity_authority: None,
                page_id: None,
                scope: "tweet.read tweet.write users.read".into(),
                scope_verified: true,
                identity_confirmed: true,
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
