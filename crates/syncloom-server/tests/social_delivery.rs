#[path = "../../syncloom-core/tests/support/fake_social.rs"]
mod fake;
#[path = "../../syncloom-core/tests/support/social.rs"]
mod fixture;
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
) -> (TestServer, Arc<SocialPlatforms>, Arc<fixture::TestClock>) {
    let clock = fixture::clock();
    let q = Arc::new(fixture::social(
        fake,
        memory_store::MemoryStore::new(),
        clock.clone(),
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
    (server, q, clock)
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
        let (server, _, clock) = setup(&fake, platform, false).await;
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
        assert_eq!(fake.calls().len(), calls + 1 + identity_reads);
        assert_eq!(fake.calls()[calls].path, path);
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
        assert_eq!(fake.calls().len(), calls + 2 + identity_reads);
        assert_eq!(get(&server, &connection).await.body, refreshed.body);
        fake.reply(path, 200, renewed);
        let retry = post_empty(&server, &refresh).await;
        assert_eq!(retry.status, 200, "{platform}: {}", retry.raw);
        assert_eq!(fake.calls().len(), calls + 3 + 2 * identity_reads);
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
    let (server, _, _) = setup(&fake, "bluesky", false).await;
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
    let (server, _, _) = setup(&fake, "x", false).await;
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
async fn posted_and_unknown_intents_stay_with_their_target_after_fresh_approval() {
    for initial_status in [200, 500] {
        let fake = fake::Fake::bluesky().await;
        let (server, _, _) = setup(&fake, "bluesky", true).await;
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
    let (server, _, _) = setup(&fake, "bluesky", true).await;
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
    let (server, _, _) = setup(&fake, "bluesky", true).await;
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
    let (server, _, _) = setup(&fake, "bluesky", true).await;
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
        let (server, _, _) = setup(&fake, "bluesky", true).await;
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
        let (server, _, _) = setup(&fake, platform, true).await;
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
        let (server, _, _) = setup(&fake, "bluesky", true).await;
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
        let (server, _, _) = setup(&fake, "bluesky", true).await;
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
