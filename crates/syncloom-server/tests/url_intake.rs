//! URL intake through the running loopback server, never mocked handlers.
#[cfg(feature = "test-source")]
#[path = "support/source.rs"]
mod source;
mod support;
use serde_json::json;
use support::{
    scripted::{Scripted, text},
    *,
};

#[tokio::test]
async fn url_intake_is_disabled_without_an_explicit_host_allowlist() {
    let provider = Scripted::new(text("Never called."));
    let server = start(provider.clone()).await;
    let response = post(
        &server,
        "/v1/capture/url",
        json!({"url":"https://fixture.test/article"}),
    )
    .await;
    assert_eq!(response.status, 503, "{}", response.raw);
    assert_eq!(response.code(), "source_not_configured");
    let browser = answer(
        client()
            .post(server.url("/capture/url"))
            .form(&[
                ("csrf", server.token.as_str()),
                ("text", "https://fixture.test/article"),
            ])
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(browser.status, 503);
    assert!(browser.raw.contains("No draft was created by this fetch"));
    assert!(!browser.raw.contains("open the piece to check"));
    assert!(provider.requests().is_empty());
    assert!(
        get(&server, "/v1/pieces").await.body["pieces"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn workspace_exposes_explicit_fetch_separately_from_drafting() {
    let server = start(Scripted::new(text("Never called."))).await;
    let response = get(&server, "/").await;
    assert!(response.raw.contains("formaction=\"/capture/url\""));
    assert!(response.raw.contains("Fetch URL only"));
}

#[cfg(feature = "test-source")]
mod controlled {
    use super::*;
    use syncloom_core::{Settings, Store, Studio, source::SourceClient};

    #[expect(
        clippy::unwrap_used,
        reason = "Controlled fixture setup fails immediately on an invalid test listener."
    )]
    async fn configured(
        provider: std::sync::Arc<Scripted>,
        fixture: &source::Fixture,
    ) -> TestServer {
        start_studio(provider, Settings::default(), |studio| {
            studio.with_sources(SourceClient::fixture(fixture.address).unwrap())
        })
        .await
    }
    fn selection(fetched: &serde_json::Value) -> serde_json::Value {
        json!({"text":fetched["source"]["retrieval"]["extracted_text"], "channels":["linkedin_text"], "source_revision":fetched["source_revision"]})
    }
    async fn fetch(server: &TestServer, path: &str) -> Answer {
        post(
            server,
            "/v1/capture/url",
            json!({"url":format!("http://fixture.test{path}")}),
        )
        .await
    }

    #[tokio::test]
    async fn fetched_snapshot_preview_confirm_duplicate_and_reopened_result_keep_provenance() {
        let fixture = source::start().await;
        let provider = Scripted::new(text("A pending controlled draft."));
        let server = configured(provider.clone(), &fixture).await;
        assert_eq!(
            post(
                &server,
                "/v1/capture/suggest",
                json!({"text":"http://fixture.test/article"})
            )
            .await
            .status,
            200
        );
        let hint = answer(
            client()
                .post(server.url("/capture/suggest"))
                .form(&[
                    ("csrf", server.token.as_str()),
                    ("text", "http://fixture.test/article"),
                ])
                .send()
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(hint.status, 200);
        assert!(
            fixture.seen.0.lock().unwrap().is_empty(),
            "suggestions never open links"
        );
        let fetched = fetch(&server, "/redirect").await;
        assert_eq!(fetched.status, 200, "{}", fetched.raw);
        assert_eq!(fetched.body["source"]["type"], "url");
        assert_eq!(
            fetched.body["source"]["raw_input"],
            "http://fixture.test/redirect"
        );
        assert_eq!(
            fetched.body["source"]["retrieval"]["final_url"],
            "http://fixture.test/article"
        );
        assert_eq!(
            fetched.body["source"]["retrieval"]["extracted_text"],
            source::ARTICLE
        );
        assert_eq!(
            fetched.body["source"]["retrieval"]["fetched_at"],
            "2026-10-30T18:00:00Z"
        );
        assert_eq!(
            fetched.body["source"]["retrieval"]["content_sha256"],
            syncloom_core::visual::digest(source::HTML.as_bytes())
        );
        assert!(provider.requests().is_empty());
        let selected = selection(&fetched.body);
        let preview = post(&server, "/v1/capture/preview", selected.clone()).await;
        assert_eq!(preview.status, 200, "{}", preview.raw);
        assert!(preview.body["approval"].is_null());
        let mut confirm = selected.clone();
        confirm["expected_revision"] = preview.body["revision"].clone();
        for field in ["text", "source_revision", "expected_revision"] {
            let mut changed = confirm.clone();
            changed[field] = json!("a".repeat(64));
            assert_eq!(
                post(&server, "/v1/capture/confirm", changed).await.code(),
                "revision_conflict"
            );
        }
        let mut changed = confirm.clone();
        changed["channels"] = json!(["facebook_page"]);
        assert_eq!(
            post(&server, "/v1/capture/confirm", changed).await.code(),
            "revision_conflict"
        );
        assert!(provider.requests().is_empty());
        let (one, two) = tokio::join!(
            post(&server, "/v1/capture/confirm", confirm.clone()),
            post(&server, "/v1/capture/confirm", confirm.clone())
        );
        let mut statuses = [one.status, two.status];
        statuses.sort_unstable();
        assert_eq!(statuses, [200, 201]);
        assert_eq!(one.body, two.body);
        assert_eq!(provider.requests().len(), 1);
        assert_eq!(provider.requests()[0].thought, source::ARTICLE);
        let piece = &one.body["pieces"][0];
        assert_eq!(piece["source"], fetched.body["source"]);
        assert_eq!(piece["status"], "pending_review");
        assert!(piece["publish_status"].is_null() && piece["review_decision_id"].is_null());
        assert_eq!(
            std::fs::read_dir(server.dir.path().join("outbox"))
                .unwrap()
                .count(),
            0
        );
        let events =
            std::fs::read_to_string(server.dir.path().join("provenance/events.jsonl")).unwrap();
        assert_eq!(events.lines().count(), 1);
        for forbidden in [
            "zebrafish",
            "fixture.test",
            "A pending controlled draft.",
            "evil.invalid",
        ] {
            assert!(!events.contains(forbidden));
        }
        {
            let seen = fixture.seen.0.lock().unwrap();
            assert_eq!(seen.len(), 2, "preview and confirm must not refetch");
            for (_, headers) in seen.iter() {
                for name in [
                    "authorization",
                    "cookie",
                    "referer",
                    "from",
                    "x-syncloom-token",
                ] {
                    assert!(!headers.contains_key(name));
                }
                assert_eq!(headers["user-agent"], "Syncloom/0.1");
                assert_eq!(headers["accept-encoding"], "identity");
            }
        }
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let reopened = Studio::new(
            Store::open(server.dir.path()).unwrap(),
            provider.clone(),
            fixed_clock(),
            Settings::default(),
        );
        let task = tokio::spawn(async move {
            syncloom_server::serve(listener, std::sync::Arc::new(reopened))
                .await
                .unwrap()
        });
        let path = format!(
            "/capture/result/{}",
            preview.body["revision"].as_str().unwrap()
        );
        let result = answer(
            client()
                .get(format!("http://{addr}{path}"))
                .send()
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(result.status, 200);
        assert!(result.raw.contains(piece["piece_id"].as_str().unwrap()));
        let denied = answer(
            client()
                .post(format!("http://{addr}/v1/capture/confirm"))
                .header("x-syncloom-token", &server.token)
                .json(&confirm)
                .send()
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(denied.code(), "session_required");
        assert_eq!(provider.requests().len(), 1);
        task.abort();
    }

    #[tokio::test]
    async fn source_policy_redirect_content_and_limit_errors_are_fixed_and_do_not_draft() {
        let fixture = source::start().await;
        let provider = Scripted::new(text("Never called."));
        let server = configured(provider.clone(), &fixture).await;
        for (path, code, status) in [
            ("/escape", "source_not_allowed", 422),
            ("/mixed", "source_not_allowed", 422),
            ("/foreign", "source_not_allowed", 422),
            ("/loop", "source_fetch_failed", 502),
            ("/missing-location", "source_fetch_failed", 502),
            ("/bad-location", "source_fetch_failed", 502),
            ("/denied", "source_fetch_failed", 502),
            ("/large-body", "source_fetch_failed", 502),
            ("/large-text", "source_fetch_failed", 502),
            ("/json", "source_fetch_failed", 502),
            ("/charset", "source_fetch_failed", 502),
            ("/utf8", "source_fetch_failed", 502),
            ("/compressed", "source_fetch_failed", 502),
            ("/hidden-mismatch", "thought_too_short", 422),
            ("/short", "thought_too_short", 422),
            ("/empty", "thought_too_short", 422),
            ("/slow", "source_timeout", 504),
        ] {
            let response = fetch(&server, path).await;
            assert_eq!(response.status, status, "{path}: {}", response.raw);
            assert_eq!(response.code(), code, "{path}");
            assert!(!response.raw.contains("fixture.test"));
            let mut expected = support::fixture(&format!("errors/{code}.json"));
            expected["body"]["error"]["request_id"] = response.body["error"]["request_id"].clone();
            assert_eq!(response.as_fixture(), expected);
        }
        for path in ["/plain", "/text-limit", "/body-limit"] {
            assert_eq!(fetch(&server, path).await.status, 200, "{path}");
        }
        for url in [
            "file:///etc/passwd",
            "ftp://fixture.test/x",
            "http://127.0.0.1/x",
            "http://2130706433/x",
            "http://[::1]/x",
            "http://sub.fixture.test/x",
            "http://fixture.test./x",
            "http://user:pass@fixture.test/x",
            "http://fixture.test:8080/x",
            "http://fixture.test/x?token=zebrafish",
            "http://fixture.test/x#secret",
        ] {
            assert_eq!(
                post(&server, "/v1/capture/url", json!({"url":url}))
                    .await
                    .code(),
                "source_not_allowed",
                "{url}"
            );
        }
        assert!(provider.requests().is_empty());
        assert!(
            get(&server, "/v1/pieces").await.body["pieces"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        let seen = fixture.seen.0.lock().unwrap();
        assert!(!seen.iter().any(|(p, _)| p == "/private"));
        assert_eq!(seen.iter().filter(|(p, _)| p == "/loop").count(), 4);
        assert_eq!(
            seen.iter().filter(|(p, _)| p == "/denied").count(),
            1,
            "no retries"
        );
    }

    #[tokio::test]
    async fn all_url_and_receipt_endpoints_reject_foreign_sessions_and_malformed_input_before_fetching()
     {
        let fixture = source::start().await;
        let provider = Scripted::new(text("Never called."));
        let server = configured(provider.clone(), &fixture).await;
        for path in [
            "/v1/capture/url",
            "/v1/capture/preview",
            "/v1/capture/confirm",
        ] {
            for (header, value, code) in [
                ("host", "foreign.invalid", "host_not_allowed"),
                ("origin", "null", "origin_not_allowed"),
                ("sec-fetch-site", "cross-site", "origin_not_allowed"),
                ("x-syncloom-token", "foreign-session", "session_required"),
            ] {
                let response = answer(
                    client()
                        .post(server.url(path))
                        .header("x-syncloom-token", &server.token)
                        .header(header, value)
                        .json(&json!({"url":"http://fixture.test/article"}))
                        .send()
                        .await
                        .unwrap(),
                )
                .await;
                assert_eq!(response.status, 403, "{path} {header}: {}", response.raw);
                assert_eq!(response.code(), code);
            }
            for body in [
                json!(null),
                json!({"url":42}),
                json!({"url":"http://fixture.test/article","source":{"type":"url"}}),
            ] {
                assert_eq!(post(&server, path, body).await.code(), "invalid_request");
            }
        }
        for path in [
            "/capture/url",
            "/capture/suggest",
            "/capture/preview",
            "/capture/confirm",
        ] {
            let response = answer(
                client()
                    .post(server.url(path))
                    .form(&[("csrf", "wrong"), ("text", "http://fixture.test/article")])
                    .send()
                    .await
                    .unwrap(),
            )
            .await;
            assert_eq!(response.status, 403);
        }
        assert!(fixture.seen.0.lock().unwrap().is_empty());
        let other_provider = Scripted::new(text("Never called by another session."));
        let other = configured(other_provider.clone(), &fixture).await;
        let denied = answer(
            client()
                .post(other.url("/v1/capture/url"))
                .header("x-syncloom-token", &server.token)
                .json(&json!({"url":"http://fixture.test/article"}))
                .send()
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(denied.code(), "session_required");
        assert!(fixture.seen.0.lock().unwrap().is_empty());
        let fetched = fetch(&server, "/article").await;
        assert_eq!(fetched.status, 200);
        assert_eq!(
            post(&other, "/v1/capture/preview", selection(&fetched.body))
                .await
                .code(),
            "revision_conflict"
        );
        for owner in [&server, &other] {
            assert!(
                get(owner, "/v1/pieces").await.body["pieces"]
                    .as_array()
                    .unwrap()
                    .is_empty()
            );
        }
        assert!(provider.requests().is_empty());
        assert!(other_provider.requests().is_empty());
        assert_eq!(fixture.seen.0.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn browser_receipt_fields_are_inert_and_provider_failure_can_retry_without_refetch() {
        let fixture = source::start().await;
        let provider = Scripted::new(support::scripted::Reply::Fail);
        let server = configured(provider.clone(), &fixture).await;
        let fetched = fetch(&server, "/article").await;
        let selected = selection(&fetched.body);
        let revision = fetched.body["source_revision"].as_str().unwrap();
        let fields = [
            ("csrf", server.token.as_str()),
            ("text", source::ARTICLE),
            ("source_revision", revision),
            ("channels", "linkedin_text"),
        ];
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
        for expected in [
            "Source provenance",
            "Untrusted source data",
            "configured drafting provider",
            "Not an approval",
            "Confirm and draft",
        ] {
            assert!(preview.raw.contains(expected), "{expected}");
        }
        assert!(!preview.raw.contains("<script") && !preview.raw.contains("<textarea"));
        let preview = post(&server, "/v1/capture/preview", selected.clone()).await;
        let mut confirm = selected;
        confirm["expected_revision"] = preview.body["revision"].clone();
        assert_eq!(
            post(&server, "/v1/capture/confirm", confirm.clone())
                .await
                .code(),
            "provider_error"
        );
        assert!(
            get(&server, "/v1/pieces").await.body["pieces"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        provider.set(text("Recovered fake draft."));
        assert_eq!(
            post(&server, "/v1/capture/confirm", confirm).await.status,
            201
        );
        assert_eq!(fixture.seen.0.lock().unwrap().len(), 1);
    }
}
