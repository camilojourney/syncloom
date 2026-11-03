//! Real loopback HTTP capture journey, with a scripted provider and temporary store.
mod support;
use serde_json::json;
use support::{
    scripted::{Scripted, text},
    *,
};

#[tokio::test]
async fn capture_suggest_preview_confirm_and_repeat_never_approve_or_publish() {
    let provider = Scripted::new(text("A draft that still needs the founder's review."));
    let server = start(provider.clone()).await;
    let suggestion = post(&server, "/v1/capture/suggest", json!({"text":THOUGHT})).await;
    assert_eq!(suggestion.status, 200, "{}", suggestion.raw);
    assert_eq!(suggestion.body["suggestions"][0]["channel"], "x");
    let selected = json!({"text": THOUGHT,"channels":["linkedin_text"]});
    let preview = post(&server, "/v1/capture/preview", selected.clone()).await;
    assert_eq!(preview.status, 200, "{}", preview.raw);
    assert_eq!(preview.body["text"], THOUGHT);
    assert!(preview.body["approval"].is_null());
    assert!(preview.body["publish_status"].is_null());
    assert!(provider.requests().is_empty());
    assert!(
        get(&server, "/v1/pieces").await.body["pieces"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let mut confirm = selected.clone();
    confirm["expected_revision"] = json!("a".repeat(64));
    let stale = post(&server, "/v1/capture/confirm", confirm.clone()).await;
    assert_eq!(stale.code(), "revision_conflict");
    assert!(provider.requests().is_empty());
    confirm["expected_revision"] = preview.body["revision"].clone();
    let (first, retry) = tokio::join!(
        post(&server, "/v1/capture/confirm", confirm.clone()),
        post(&server, "/v1/capture/confirm", confirm.clone())
    );
    let mut statuses = [first.status, retry.status];
    statuses.sort_unstable();
    assert_eq!(statuses, [200, 201]);
    assert_eq!(first.body, retry.body);
    assert_eq!(provider.requests().len(), 1);
    let piece = &first.body["pieces"][0];
    assert_eq!(piece["status"], "pending_review");
    assert!(piece["review_decision_id"].is_null());
    assert!(piece["publish_status"].is_null());
    assert_eq!(piece["source"]["raw_input"], THOUGHT);
    let result = get(
        &server,
        &format!(
            "/capture/result/{}",
            preview.body["revision"].as_str().unwrap()
        ),
    )
    .await;
    assert_eq!(result.status, 200);
    assert!(result.raw.contains(piece["piece_id"].as_str().unwrap()));
    assert!(!result.raw.contains("Approve this version"));
    assert!(!result.raw.contains(">Publish<"));
    let events =
        std::fs::read_to_string(server.dir.path().join("provenance/events.jsonl")).unwrap();
    assert_eq!(events.lines().count(), 1);
    assert!(!events.contains("piece_reviewed"));
    assert_eq!(
        std::fs::read_dir(server.dir.path().join("outbox"))
            .unwrap()
            .count(),
        0
    );
}

#[tokio::test]
async fn capture_rejects_malformed_selections_and_auth_before_any_provider_call() {
    let provider = Scripted::new(text("Never called."));
    let server = start(provider.clone()).await;
    for suffix in ["suggest", "preview", "confirm"] {
        let path = format!("/v1/capture/{suffix}");
        for body in [
            json!(null),
            json!([]),
            json!({"text":123}),
            json!({"text":THOUGHT,"attachment_url":"https://example.invalid"}),
        ] {
            assert_eq!(post(&server, &path, body).await.code(), "invalid_request");
        }
        for (header, value, code) in [
            ("host", "foreign.invalid", "host_not_allowed"),
            ("origin", "https://foreign.invalid", "origin_not_allowed"),
            ("sec-fetch-site", "cross-site", "origin_not_allowed"),
            ("sec-fetch-site", "same-site", "origin_not_allowed"),
            ("x-syncloom-token", "stale", "session_required"),
        ] {
            let response = answer(
                client()
                    .post(server.url(&path))
                    .header(header, value)
                    .json(&json!({"text":THOUGHT}))
                    .send()
                    .await
                    .unwrap(),
            )
            .await;
            assert_eq!(response.status, 403, "{path} {header}");
            assert_eq!(response.code(), code);
            assert!(
                !response
                    .headers
                    .keys()
                    .any(|name| name.as_str().starts_with("access-control-"))
            );
        }
    }
    for (channels, code) in [
        (json!([]), "invalid_request"),
        (json!(["x", "x"]), "invalid_request"),
        (json!(["instagram"]), "invalid_request"),
        (json!(["tiktok_video"]), "unsupported_channel"),
    ] {
        assert_eq!(
            post(
                &server,
                "/v1/capture/preview",
                json!({"text":THOUGHT,"channels":channels})
            )
            .await
            .code(),
            code
        );
    }
    assert_eq!(
        post(
            &server,
            "/v1/capture/suggest",
            json!({"text":"a b c d e f g"})
        )
        .await
        .code(),
        "thought_too_short"
    );
    assert!(provider.requests().is_empty());
}

#[tokio::test]
async fn browser_capture_reuses_the_composer_and_preserves_selected_text() {
    let provider = Scripted::new(text("A reviewable fake draft."));
    let server = start(provider.clone()).await;
    let entry = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
        .get(server.url("/capture"))
        .send()
        .await
        .unwrap();
    assert_eq!(entry.status(), 303);
    assert_eq!(entry.headers()["location"], "/#compose");
    let home = get(&server, "/").await;
    assert!(home.raw.contains("formaction=\"/capture/suggest\""));
    let fields = [
        ("csrf", server.token.as_str()),
        ("text", THOUGHT),
        ("channels", "linkedin_text"),
    ];
    let suggest = answer(
        client()
            .post(server.url("/capture/suggest"))
            .form(&fields)
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(suggest.status, 200, "{}", suggest.raw);
    assert!(
        !suggest.raw.contains("<textarea"),
        "Do not duplicate the thought composer"
    );
    assert!(
        !suggest.raw.contains(" checked"),
        "The founder selects the suggestions"
    );
    let preview = answer(
        client()
            .post(server.url("/capture/preview"))
            .form(&fields)
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(preview.status, 200, "{}", preview.raw);
    assert!(preview.raw.contains("Not an approval"));
    assert!(preview.raw.contains("not yet a draft"));
    assert!(preview.raw.contains("Confirm and draft"));
    assert!(provider.requests().is_empty());
    let empty = answer(
        client()
            .post(server.url("/capture/preview"))
            .form(&fields[..2])
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(empty.status, 400);
    assert!(empty.raw.contains("Select at least one suggestion"));
    for path in ["/capture/suggest", "/capture/preview", "/capture/confirm"] {
        let denied = answer(
            client()
                .post(server.url(path))
                .form(&[("text", THOUGHT)])
                .send()
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(denied.status, 403);
        assert!(denied.raw.contains("session_required"));
    }
}
