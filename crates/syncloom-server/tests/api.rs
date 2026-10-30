//! The HTTP boundary (studio-api) and the operations it exposes over the core.
mod support;
use serde_json::{Value, json};
use support::*;

// studio-api: Host check, Rebinding host refused and Every path is guarded
#[tokio::test]
async fn a_wrong_host_is_refused_on_every_path_without_touching_the_provider() {
    let provider = Scripted::new(text("A draft."));
    let server = start(provider.clone()).await;
    let port = server.addr.port();
    let hosts = [
        format!("Host: evil.example:{port}"),
        format!("Host: localhost:{port}"),
        "Host: 127.0.0.1".to_owned(),
        format!("Host: [::1]:{port}"),
        format!("Host: 127.0.0.1:{}", port.wrapping_add(1)),
    ];
    let paths = [
        "GET /v1/health HTTP/1.1",
        "POST /v1/thoughts HTTP/1.1",
        "GET /v1/pieces/thought-000000000000-linkedin_text HTTP/1.1",
        "GET /missing HTTP/1.1",
    ];
    for host in &hosts {
        for path in paths {
            let answer = raw(server.addr, &head_with_hosts(path, &[host], &[])).await;
            assert_eq!(answer.status, 403, "{host} {path}: {}", answer.raw);
            assert_eq!(answer.code(), "host_not_allowed", "{host} {path}");
        }
    }
    // No Host header at all.
    let answer = raw(server.addr, &head_with_hosts("GET /v1/health HTTP/1.1", &[], &[])).await;
    assert_eq!((answer.status, answer.code()), (403, "host_not_allowed"), "{}", answer.raw);
    assert!(provider.requests().is_empty());
    assert!(files(&server.dir, "pieces").is_empty());
}

// studio-api: Host check, Two Host headers
#[tokio::test]
async fn two_host_headers_are_refused_even_when_one_is_right() {
    let server = start(Scripted::new(text("A draft."))).await;
    let right = format!("Host: {}", server.host());
    let answer = raw(
        server.addr,
        &head_with_hosts("GET /v1/health HTTP/1.1", &[&right, "Host: evil.example"], &[]),
    )
    .await;
    assert_eq!((answer.status, answer.code()), (403, "host_not_allowed"), "{}", answer.raw);
}

// studio-api: No cross-origin access
#[tokio::test]
async fn no_cors_header_is_ever_sent() {
    let server = start(Scripted::new(text("A draft."))).await;
    let with_origin = client()
        .get(server.url("/v1/health"))
        .header("Origin", "http://evil.example")
        .send()
        .await
        .unwrap();
    let preflight = client()
        .request(reqwest::Method::OPTIONS, server.url("/v1/thoughts"))
        .header("Origin", "http://evil.example")
        .header("Access-Control-Request-Method", "POST")
        .send()
        .await
        .unwrap();
    for response in [with_origin, preflight] {
        for name in response.headers().keys() {
            assert!(
                !name.as_str().starts_with("access-control-"),
                "CORS header {name} was sent"
            );
        }
    }
}

// studio-api: Health
#[tokio::test]
async fn health_answers_ok_with_the_crate_version() {
    let server = start(Scripted::new(Reply::Unconfigured)).await;
    let answer = get(&server, "/v1/health").await;
    assert_eq!(answer.status, 200);
    assert_eq!(
        answer.body,
        json!({"status": "ok", "version": env!("CARGO_PKG_VERSION")})
    );
}

// studio-api: Request id, Valid id kept and Invalid id replaced
#[tokio::test]
async fn a_valid_request_id_is_kept_and_an_invalid_one_is_replaced_by_a_uuid() {
    let server = start(Scripted::new(text("A draft."))).await;
    let long = "a".repeat(128);
    for valid in ["a", "Req.1:a_b-2", long.as_str()] {
        let answer = answer(
            client()
                .get(server.url("/missing"))
                .header("X-Request-Id", valid)
                .send()
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(answer.request_id(), valid);
        assert_eq!(answer.body["error"]["request_id"], valid);
    }
    let too_long = "a".repeat(129);
    let invalid: Vec<Option<&str>> = vec![
        None,
        Some(""),
        Some(too_long.as_str()),
        Some("has space"),
        Some("a/b"),
        Some("tab\there"),
    ];
    for value in invalid {
        let mut request = client().get(server.url("/v1/health"));
        if let Some(value) = value {
            request = request.header("X-Request-Id", value);
        }
        let answer = answer(request.send().await.unwrap()).await;
        let id = answer.request_id();
        assert_eq!(id.len(), 36, "{value:?} gave {id:?}");
        assert_eq!(id.matches('-').count(), 4);
        assert_eq!(&id[14..15], "4", "{value:?}: not a UUID v4");
    }
    let raw_answer = raw(
        server.addr,
        &head_with_hosts(
            "GET /v1/health HTTP/1.1",
            &[&format!("Host: {}", server.host())],
            &[("X-Request-Id", "caf\u{e9}")],
        ),
    )
    .await;
    assert_eq!(raw_answer.request_id().len(), 36, "non-ASCII id must be replaced");
}

// studio-api: Error envelope, Message never echoes the request
#[tokio::test]
async fn error_messages_are_fixed_and_never_echo_request_content() {
    let server = start(Scripted::new(Reply::Fail)).await;
    let canary = "canary-9f1e";
    let cases: Vec<(Answer, &str)> = vec![
        (
            get(&server, &format!("/v1/pieces/{canary}")).await,
            "piece_not_found",
        ),
        (
            post(&server, "/v1/thoughts", json!({"text": canary, "channels": ["linkedin_text"]})).await,
            "thought_too_short",
        ),
        (
            post(&server, "/v1/thoughts", json!({"text": format!("{canary} and more words"), "channels": [canary]})).await,
            "unsupported_channel",
        ),
        (
            post(&server, "/v1/thoughts", json!({"text": format!("{canary} and more words"), "channels": ["linkedin_text"]})).await,
            "provider_error",
        ),
        (get(&server, &format!("/{canary}")).await, "not_found"),
        (
            answer(
                client()
                    .post(server.url("/v1/thoughts"))
                    .header("Content-Type", "application/json")
                    .header("X-Canary", canary)
                    .body(format!("{{\"text\": \"{canary}\" and junk"))
                    .send()
                    .await
                    .unwrap(),
            )
            .await,
            "invalid_request",
        ),
    ];
    for (answer, code) in cases {
        assert_eq!(answer.code(), code, "{}", answer.raw);
        let expected = fixture(&format!("errors/{code}.json"));
        assert_eq!(answer.body["error"]["message"], expected["body"]["error"]["message"]);
        assert_eq!(answer.status, expected["status"].as_u64().unwrap() as u16);
        assert!(!answer.raw.contains(canary), "{code}: {}", answer.raw);
        assert_eq!(
            answer.body["error"].as_object().map(|o| o.len()),
            Some(3),
            "the envelope has exactly code, message and request_id"
        );
        assert_eq!(answer.body["error"]["request_id"], answer.request_id());
    }
}

// studio-api: Unknown path, Wrong method
#[tokio::test]
async fn unknown_paths_and_wrong_methods_have_their_codes() {
    let server = start(Scripted::new(text("A draft."))).await;
    let answer = get(&server, "/missing").await;
    assert_eq!((answer.status, answer.code()), (404, "not_found"));
    let answer = get(&server, "/v1/thoughts").await;
    assert_eq!((answer.status, answer.code()), (405, "method_not_allowed"));
    let answer = post(&server, "/v1/health", json!({})).await;
    assert_eq!((answer.status, answer.code()), (405, "method_not_allowed"));
}

// studio-api: Invalid request body; thought-drafting: Repeated channel
#[tokio::test]
async fn malformed_thought_bodies_are_invalid_requests_without_a_provider_call() {
    let provider = Scripted::new(text("A draft."));
    let server = start(provider.clone()).await;
    let bodies = [
        json!({"channels": ["linkedin_text"]}),
        json!({"text": 42, "channels": ["linkedin_text"]}),
        json!({"text": THOUGHT, "channels": ["linkedin_text"], "extra": 1}),
        json!({"text": THOUGHT, "channels": []}),
        json!({"text": THOUGHT}),
        json!({"text": THOUGHT, "channels": ["linkedin_text", "linkedin_text"]}),
        json!([1, 2]),
    ];
    for body in bodies {
        let answer = post(&server, "/v1/thoughts", body.clone()).await;
        assert_eq!((answer.status, answer.code()), (400, "invalid_request"), "{body}: {}", answer.raw);
    }
    let answer = answer(
        client()
            .post(server.url("/v1/thoughts"))
            .header("Content-Type", "text/plain")
            .body("not json")
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!((answer.status, answer.code()), (400, "invalid_request"));
    assert!(provider.requests().is_empty());
}

// thought-drafting: Create pieces from a thought (through the API)
#[tokio::test]
async fn a_thought_is_drafted_into_a_pending_piece() {
    let server = start(Scripted::new(text("  A draft about the thought.  "))).await;
    let answer = create(&server, THOUGHT).await;
    let piece = piece_of(&answer);
    let group = answer.body["group_id"].as_str().unwrap();
    assert_eq!(group.len(), 32);
    assert_eq!(piece["piece_id"], format!("thought-{}-linkedin_text", &group[..12]));
    assert_eq!(piece["group_id"], group);
    assert_eq!(piece["channel"], "linkedin_text");
    assert_eq!(piece["platform"], "linkedin");
    assert_eq!(piece["status"], "pending_review");
    assert_eq!(piece["text"], "A draft about the thought.");
    assert_eq!(piece["source"], json!({"type": "text", "raw_input": THOUGHT}));
    assert_eq!(piece["review_decision_id"], Value::Null);
    assert_eq!(piece["publish_status"], Value::Null);
    assert_eq!(piece["fit"], json!({"verdict": "PASS", "char_count": 26, "min_chars": 20, "max_chars": 3000}));
    assert_eq!(piece["created_at"], "2026-10-30T18:00:00Z");
    assert_eq!(piece["updated_at"], piece["created_at"]);
    assert_eq!(piece["revision"].as_str().unwrap().len(), 64);
    let keys: Vec<&str> = piece.as_object().unwrap().keys().map(String::as_str).collect();
    assert_eq!(
        keys,
        [
            "piece_id", "group_id", "channel", "platform", "text", "revision", "status",
            "review_decision_id", "publish_status", "fit", "source", "created_at", "updated_at"
        ]
    );
}

// thought-drafting: Thought too short and Unsupported channel (status codes through the API)
#[tokio::test]
async fn too_short_thoughts_and_unknown_channels_are_422() {
    let server = start(Scripted::new(text("A draft."))).await;
    let answer = create(&server, "a b c d e f g").await;
    assert_eq!((answer.status, answer.code()), (422, "thought_too_short"));
    let answer = create(&server, "a b c d e f g h").await;
    assert_eq!(answer.status, 201, "{}", answer.raw);
    let answer = post(
        &server,
        "/v1/thoughts",
        json!({"text": THOUGHT, "channels": ["linkedin_text", "tiktok_video"]}),
    )
    .await;
    assert_eq!((answer.status, answer.code()), (422, "unsupported_channel"));
}

// thought-drafting: Provider failures persist nothing (status codes through the API)
#[tokio::test]
async fn provider_failures_have_their_statuses_and_leave_no_record() {
    let cases = [
        (Reply::Empty, 502, "provider_error"),
        (Reply::Fail, 502, "provider_error"),
        (Reply::Hang, 504, "provider_timeout"),
        (Reply::Unconfigured, 503, "provider_not_configured"),
    ];
    for (reply, status, code) in cases {
        let server = start_with(Scripted::new(reply.clone()), fast()).await;
        let answer = create(&server, THOUGHT).await;
        assert_eq!((answer.status, answer.code()), (status, code), "{reply:?}: {}", answer.raw);
        assert!(files(&server.dir, "pieces").is_empty(), "{reply:?}");
        assert!(events(&server.dir).is_empty(), "{reply:?}");
    }
}

// studio-api: Internal failure
#[tokio::test]
async fn a_provider_panic_is_an_internal_error_and_the_server_keeps_serving() {
    let provider = Scripted::new(Reply::Panic);
    let server = start(provider.clone()).await;
    let answer = create(&server, THOUGHT).await;
    assert_eq!((answer.status, answer.code()), (500, "internal_error"), "{}", answer.raw);
    assert_eq!(answer.body["error"]["message"], fixture("errors/internal_error.json")["body"]["error"]["message"]);
    provider.set(text("A draft."));
    assert_eq!(create(&server, THOUGHT).await.status, 201);
}

// draft-review: Show a piece
#[tokio::test]
async fn show_returns_the_piece_and_404_for_unknown_ids() {
    let server = start(Scripted::new(text("A draft."))).await;
    let piece = piece_of(&create(&server, THOUGHT).await);
    let answer = get(&server, &format!("/v1/pieces/{}", id_of(&piece))).await;
    assert_eq!(answer.status, 200);
    assert_eq!(answer.body, piece);
    let answer = get(&server, "/v1/pieces/thought-000000000000-linkedin_text").await;
    assert_eq!((answer.status, answer.code()), (404, "piece_not_found"));
    let answer = get(&server, "/v1/pieces/..%2F..%2Fetc").await;
    assert_eq!((answer.status, answer.code()), (404, "piece_not_found"));
}

// draft-review: Statuses, Status transitions, Approval bound to a revision, Stale revision
#[tokio::test]
async fn review_moves_along_the_table_binds_approval_and_refuses_stale_revisions() {
    let server = start(Scripted::new(text("A draft."))).await;
    let piece = piece_of(&create(&server, THOUGHT).await);
    let path = format!("/v1/pieces/{}", id_of(&piece));
    let revision = revision_of(&piece);

    let answer = patch(&server, &path, json!({"status": "draft", "expected_revision": revision})).await;
    assert_eq!((answer.status, answer.code()), (400, "invalid_request"));
    let answer = patch(&server, &path, json!({"status": "approved", "expected_revision": "0".repeat(64)})).await;
    assert_eq!((answer.status, answer.code()), (409, "revision_conflict"));
    assert_eq!(get(&server, &path).await.body, piece, "a refused request changes nothing");

    let approved = approve(&server, &piece).await;
    assert_eq!(approved.status, 200, "{}", approved.raw);
    assert_eq!(approved.body["status"], "approved");
    assert_eq!(
        approved.body["review_decision_id"],
        format!("review-{}-{}", id_of(&piece), &revision[..16])
    );
    let again = approve(&server, &piece).await;
    assert_eq!(again.body, approved.body, "repeating the status changes nothing");

    let answer = patch(&server, &path, json!({"status": "pending_review", "expected_revision": revision})).await;
    assert_eq!((answer.status, answer.code()), (409, "invalid_status_transition"));
    assert_eq!(get(&server, &path).await.body, approved.body);

    let rejected = patch(&server, &path, json!({"status": "rejected", "expected_revision": revision})).await;
    assert_eq!(rejected.body["status"], "rejected");
    let answer = patch(&server, &path, json!({"status": "approved", "expected_revision": revision})).await;
    assert_eq!((answer.status, answer.code()), (409, "invalid_status_transition"));
    let pending = patch(&server, &path, json!({"status": "pending_review", "expected_revision": revision})).await;
    assert_eq!(pending.body["status"], "pending_review");
}

// draft-review: Edit creates a revision and voids approval; Edit body rules; platform-rules
#[tokio::test]
async fn an_edit_voids_approval_and_a_publish_then_needs_a_new_approval() {
    let server = start(Scripted::new(text("A draft."))).await;
    let piece = piece_of(&create(&server, THOUGHT).await);
    let path = format!("/v1/pieces/{}", id_of(&piece));
    let revision = revision_of(&piece);
    approve(&server, &piece).await;

    let long = "y".repeat(3001);
    let edited = patch(&server, &path, json!({"text": format!(" {long} "), "expected_revision": revision})).await;
    assert_eq!(edited.status, 200, "{}", edited.raw);
    assert_eq!(edited.body["text"], long);
    assert_ne!(edited.body["revision"], revision);
    assert_eq!(edited.body["status"], "pending_review");
    assert_eq!(edited.body["review_decision_id"], Value::Null);
    assert_eq!(edited.body["fit"]["verdict"], "REVIEW");
    assert_eq!(edited.body["fit"]["char_count"], 3001);

    let answer = post(
        &server,
        &format!("{path}/publish"),
        json!({"expected_revision": edited.body["revision"]}),
    )
    .await;
    assert_eq!((answer.status, answer.code()), (409, "approval_required"));

    let new_revision = revision_of(&edited.body);
    for body in [
        json!({"status": "approved", "text": "x", "expected_revision": new_revision}),
        json!({"expected_revision": new_revision}),
        json!({"text": "  \n", "expected_revision": new_revision}),
        json!({"text": "fine", "expected_revision": new_revision, "unknown": true}),
        json!({"text": "fine"}),
    ] {
        let answer = patch(&server, &path, body.clone()).await;
        assert_eq!((answer.status, answer.code()), (400, "invalid_request"), "{body}: {}", answer.raw);
    }
    let answer = patch(&server, "/v1/pieces/thought-000000000000-linkedin_text", json!({"status": "approved", "expected_revision": new_revision})).await;
    assert_eq!((answer.status, answer.code()), (404, "piece_not_found"));
}

// dispatch-outbox: Publish operation, Dry run, Approval required, Stale, Contained, Repeat
#[tokio::test]
async fn publish_dry_runs_then_reserves_one_contained_intent_and_repeats_it() {
    let provider = Scripted::new(text("A draft."));
    let server = start(provider.clone()).await;
    let piece = piece_of(&create(&server, THOUGHT).await);
    let path = format!("/v1/pieces/{}/publish", id_of(&piece));
    let revision = revision_of(&piece);

    let dry = post(&server, &path, json!({"expected_revision": revision, "dry_run": true})).await;
    assert_eq!(dry.status, 200, "{}", dry.raw);
    assert_eq!(dry.body["status"], "dry_run");
    assert_eq!(dry.body["request_id"], Value::Null);
    assert_eq!(dry.body["created"], false);
    assert_eq!(dry.body["payload"], json!({"content": "A draft.", "platforms": ["linkedin"], "style": "raw"}));
    assert_eq!(dry.body["piece"], piece);
    assert!(files(&server.dir, "outbox").is_empty());
    assert_eq!(events(&server.dir).len(), 1);

    let answer = post(&server, &path, json!({"expected_revision": revision})).await;
    assert_eq!((answer.status, answer.code()), (409, "approval_required"));
    let answer = post(&server, &path, json!({"expected_revision": "0".repeat(64), "dry_run": true})).await;
    assert_eq!((answer.status, answer.code()), (409, "revision_conflict"));
    let answer = post(&server, &path, json!({"dry_run": true})).await;
    assert_eq!((answer.status, answer.code()), (400, "invalid_request"));
    let answer = post(&server, "/v1/pieces/thought-000000000000-linkedin_text/publish", json!({"expected_revision": revision})).await;
    assert_eq!((answer.status, answer.code()), (404, "piece_not_found"));

    approve(&server, &piece).await;
    let calls_before = provider.requests().len();
    let first = post(&server, &path, json!({"expected_revision": revision})).await;
    assert_eq!(first.status, 200, "{}", first.raw);
    assert_eq!(first.body["status"], "contained");
    assert_eq!(first.body["created"], true);
    let request_id = first.body["request_id"].as_str().unwrap().to_owned();
    assert!(request_id.starts_with("dispatch-") && request_id.len() == 41, "{request_id}");
    assert_eq!(first.body["piece"]["publish_status"], "contained");
    assert_eq!(first.body["piece"]["status"], "approved");
    assert_eq!(files(&server.dir, "outbox").len(), 1);
    assert_eq!(provider.requests().len(), calls_before, "publishing makes no outbound call");
    let keys: Vec<&str> = first.body.as_object().unwrap().keys().map(String::as_str).collect();
    assert_eq!(keys, ["piece", "status", "request_id", "created", "payload"]);

    let second = post(&server, &path, json!({"expected_revision": revision})).await;
    assert_eq!(second.body["request_id"], request_id);
    assert_eq!(second.body["created"], false);
    assert_eq!(files(&server.dir, "outbox").len(), 1);
    let kinds: Vec<String> = events(&server.dir)
        .iter()
        .map(|e| e["event"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(kinds, ["piece_created", "piece_reviewed", "intent_reserved"]);
    assert_eq!(get(&server, &format!("/v1/pieces/{}", id_of(&piece))).await.body, first.body["piece"]);
}
