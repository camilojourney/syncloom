//! Browser boundary tests use real TCP and the same core as the CLI, never mocked handlers.
mod support;
use serde_json::{Value, json};
use support::{scripted::Scripted, *};

#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
async fn session(server: &TestServer) -> String {
    let response = get(server, "/v1/session").await;
    assert_eq!(response.status, 200, "{}", response.raw);
    response.body["token"].as_str().unwrap().to_owned()
}

#[expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
fn field(html: &str, name: &str) -> String {
    html.split_once(&format!("name=\"{name}\" value=\""))
        .unwrap_or_else(|| panic!("missing {name} in {html}"))
        .1
        .split('"')
        .next()
        .unwrap()
        .to_owned()
}

#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
async fn form(server: &TestServer, path: &str, fields: &[(&str, &str)]) -> Answer {
    answer(
        client()
            .post(server.url(path))
            .header("origin", server.url(""))
            .form(fields)
            .send()
            .await
            .unwrap(),
    )
    .await
}

#[tokio::test]
async fn empty_studio_is_accessible_and_has_no_script_runtime() {
    let server = start(Scripted::new(scripted::text("A draft."))).await;
    let page = get(&server, "/").await;
    assert_eq!(page.status, 200);
    assert!(page.raw.contains("No pieces yet"));
    assert!(page.raw.contains("<main id=\"main\" tabindex=\"-1\">"));
    assert!(page.raw.contains("<label for=\"thought\""));
    assert!(page.raw.contains("Draft a piece"));
    assert!(!page.raw.contains("<script"));
    assert_eq!(field(&page.raw, "csrf"), session(&server).await);
    assert!(
        get(&server, "/studio.css")
            .await
            .raw
            .contains(":focus-visible")
    );
}

#[tokio::test]
async fn origin_guard_precedes_every_route_and_never_invokes_the_provider() {
    let provider = Scripted::new(scripted::text("A draft."));
    let server = start(provider.clone()).await;
    for origin in [
        "https://evil.invalid",
        "null",
        "http://localhost:8790",
        "http://127.0.0.1:1",
        "http://127.0.0.1",
    ] {
        for path in ["/", "/v1/session", "/v1/health", "/missing"] {
            let r = answer(
                client()
                    .get(server.url(path))
                    .header("origin", origin)
                    .send()
                    .await
                    .unwrap(),
            )
            .await;
            assert_eq!(r.status, 403, "{origin} {path}");
            assert!(r.raw.contains("origin_not_allowed"));
            assert!(!r.raw.contains(origin));
        }
    }
    let r = answer(
        client()
            .get(server.url("/v1/health"))
            .header("origin", server.url(""))
            .header("origin", server.url(""))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(r.code(), "origin_not_allowed");
    let r = answer(
        client()
            .post(server.url("/v1/thoughts"))
            .header("origin", "https://evil.invalid")
            .json(&json!({"text":THOUGHT,"channels":["linkedin_text"]}))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(r.status, 403);
    assert!(provider.requests().is_empty());
    assert!(files(&server.dir, "pieces").is_empty());
}

#[tokio::test]
async fn fetch_site_guard_blocks_cross_site_reads_even_without_origin() {
    let server = start(Scripted::new(scripted::text("A draft."))).await;
    for site in ["cross-site", "same-site", "invalid"] {
        let r = answer(
            client()
                .get(server.url("/v1/health"))
                .header("sec-fetch-site", site)
                .send()
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(r.status, 403, "{site}");
        assert_eq!(r.code(), "origin_not_allowed");
    }
    for site in ["same-origin", "none"] {
        assert_eq!(
            client()
                .get(server.url("/v1/health"))
                .header("sec-fetch-site", site)
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
    }
}

#[tokio::test]
async fn mutations_require_exact_current_session_and_reject_duplicate_tokens() {
    let provider = Scripted::new(scripted::text("A draft."));
    let server = start(provider.clone()).await;
    let body = json!({"text":THOUGHT,"channels":["linkedin_text"]});
    let missing = answer(
        client()
            .post(server.url("/v1/thoughts"))
            .json(&body)
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(missing.status, 403);
    assert_eq!(missing.code(), "session_required");
    let token = session(&server).await;
    let other = start(Scripted::new(scripted::text("Other draft."))).await;
    let old = session(&other).await;
    assert_ne!(token, old);
    assert!(token.len() >= 64);
    for invalid in ["wrong", "", old.as_str()] {
        let r = answer(
            client()
                .post(server.url("/v1/thoughts"))
                .header("x-syncloom-token", invalid)
                .json(&body)
                .send()
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(r.code(), "session_required");
    }
    let repeated = answer(
        client()
            .post(server.url("/v1/thoughts"))
            .header("x-syncloom-token", &token)
            .header("x-syncloom-token", &token)
            .json(&body)
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(repeated.code(), "session_required");
    assert!(provider.requests().is_empty());
    assert!(files(&server.dir, "pieces").is_empty());
    let accepted = answer(
        client()
            .post(server.url("/v1/thoughts"))
            .header("x-syncloom-token", &token)
            .header("origin", server.url(""))
            .json(&body)
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(accepted.status, 201);
    let denied = answer(
        client()
            .post(server.url("/v1/thoughts"))
            .header("x-syncloom-token", &token)
            .header("origin", "https://evil.invalid")
            .json(&body)
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(denied.code(), "origin_not_allowed");
    assert_eq!(provider.requests().len(), 1);
}

#[tokio::test]
async fn security_headers_cover_pages_assets_sessions_and_errors() {
    let server = start(Scripted::new(scripted::text("A draft."))).await;
    for path in [
        "/",
        "/studio.css",
        "/v1/session",
        "/missing",
        "/v1/pieces/missing/assets/image",
    ] {
        let r = get(&server, path).await;
        assert_eq!(r.headers.get("cache-control").unwrap(), "no-store");
        assert_eq!(r.headers.get("x-content-type-options").unwrap(), "nosniff");
        // no-referrer makes Chromium send Origin: null on ordinary POST navigation.
        assert_eq!(r.headers.get("referrer-policy").unwrap(), "same-origin");
        assert_eq!(r.headers.get("x-frame-options").unwrap(), "DENY");
        let csp = r
            .headers
            .get("content-security-policy")
            .unwrap()
            .to_str()
            .unwrap();
        for rule in [
            "default-src 'none'",
            "script-src 'none'",
            "frame-ancestors 'none'",
            "form-action 'self'",
            "base-uri 'none'",
        ] {
            assert!(csp.contains(rule), "{csp}");
        }
        assert!(
            !r.headers
                .keys()
                .any(|h| h.as_str().starts_with("access-control-"))
        );
    }
}

#[tokio::test]
async fn discovery_pages_saved_thoughts_without_duplicates_and_rejects_bad_records() {
    let server = start(Scripted::new(scripted::text("A saved draft."))).await;
    let empty = get(&server, "/v1/pieces").await;
    assert_eq!(empty.status, 200);
    assert_eq!(empty.body, json!({"pieces":[],"next_after":null}));
    for _ in 0..21 {
        assert_eq!(create(&server, THOUGHT).await.status, 201);
    }
    std::fs::write(server.dir.path().join("pieces/.interrupted.tmp"), "partial").unwrap();
    let page = get(&server, "/v1/pieces").await;
    assert_eq!(page.body["pieces"].as_array().unwrap().len(), 20);
    let ids: Vec<_> = page.body["pieces"]
        .as_array()
        .unwrap()
        .iter()
        .map(id_of)
        .collect();
    assert!(ids.windows(2).all(|w| w[0] < w[1]));
    let after = page.body["next_after"].as_str().unwrap();
    let next = get(&server, &format!("/v1/pieces?after={after}")).await;
    assert_eq!(next.body["pieces"].as_array().unwrap().len(), 1);
    assert!(next.body["next_after"].is_null());
    assert!(!ids.contains(&id_of(&next.body["pieces"][0])));
    assert_eq!(
        get(&server, "/v1/pieces?after=..%2Fescape").await.code(),
        "invalid_request"
    );
    let page = get(&server, "/").await;
    assert!(page.raw.contains("Next page"));
    assert!(page.raw.contains(THOUGHT));
    std::fs::write(
        server.dir.path().join("pieces/000-corrupt.json"),
        "not json",
    )
    .unwrap();
    assert_eq!(get(&server, "/v1/pieces").await.code(), "internal_error");
}

#[tokio::test]
async fn discovery_refuses_symlinks_instead_of_reading_outside_the_store() {
    let server = start(Scripted::new(scripted::text("A saved draft."))).await;
    let outside = tempfile::NamedTempFile::new().unwrap();
    let mut record = piece_of(&create(&server, THOUGHT).await);
    record["piece_id"] = json!("000-link");
    record["text"] = json!("Outside data must not be listed.");
    std::fs::write(outside.path(), serde_json::to_vec(&record).unwrap()).unwrap();
    std::os::unix::fs::symlink(
        outside.path(),
        server.dir.path().join("pieces/000-link.json"),
    )
    .unwrap();
    let r = get(&server, "/v1/pieces").await;
    assert_eq!(r.code(), "internal_error");
}

#[tokio::test]
async fn browser_forms_edit_approve_publish_and_invalidate_without_another_rule_owner() {
    let server = start(Scripted::new(scripted::text("The draft to review."))).await;
    let token = session(&server).await;
    let page = form(
        &server,
        "/actions/thought",
        &[
            ("csrf", &token),
            ("text", THOUGHT),
            ("channels", "linkedin_text"),
        ],
    )
    .await;
    assert_eq!(page.status, 200, "{}", page.raw);
    let listed = get(&server, "/v1/pieces").await;
    let id = id_of(&listed.body["pieces"][0]);
    let path = format!("/review/{id}");
    let revision = field(&page.raw, "expected_revision");
    let approved = form(
        &server,
        &format!("{path}/approve"),
        &[("csrf", &token), ("expected_revision", &revision)],
    )
    .await;
    assert!(approved.raw.contains("Approved"));
    assert!(!approved.raw.contains(&format!("action=\"{path}/approve\"")));
    let approval_at = events(&server.dir)
        .into_iter()
        .find(|event| event["event"] == "piece_reviewed" && event["status"] == "approved")
        .unwrap()["at"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(
        approved
            .raw
            .contains(&format!("Approved on <time datetime=\"{approval_at}\""))
    );
    let receipt = form(
        &server,
        &format!("{path}/publish"),
        &[("csrf", &token), ("expected_revision", &revision)],
    )
    .await;
    assert!(receipt.raw.contains("contained"));
    assert!(receipt.raw.contains("Nothing was sent"));
    assert!(receipt.raw.contains("Attempts: 0"));
    assert!(receipt.raw.contains("Not sent - saved on this laptop"));
    assert!(
        receipt
            .raw
            .contains("<details class=\"receipt-details\"><summary>Details</summary>")
    );
    assert!(
        receipt
            .raw
            .contains(&format!("Approved on <time datetime=\"{approval_at}\""))
    );
    let intent = files(&server.dir, "outbox");
    let again = form(
        &server,
        &format!("{path}/publish"),
        &[("csrf", &token), ("expected_revision", &revision)],
    )
    .await;
    assert!(again.raw.contains("contained"));
    assert_eq!(files(&server.dir, "outbox"), intent);
    let edited = form(
        &server,
        &format!("{path}/edit"),
        &[
            ("csrf", &token),
            ("expected_revision", &revision),
            ("text", "A new exact revision."),
        ],
    )
    .await;
    assert_eq!(edited.status, 200);
    assert!(edited.raw.contains("Pending review"));
    assert!(edited.raw.contains(&format!("action=\"{path}/approve\"")));
    assert!(!edited.raw.contains("Approved on <time"));
    assert!(edited.raw.contains("disabled>Publish"));
    let new_revision = field(&edited.raw, "expected_revision");
    assert_ne!(revision, new_revision);
    let stale = form(
        &server,
        &format!("{path}/approve"),
        &[("csrf", &token), ("expected_revision", &revision)],
    )
    .await;
    assert_eq!(stale.status, 409);
    assert!(stale.raw.contains("revision_conflict"));
    assert!(stale.raw.contains("Back to studio"));
    let denied = form(
        &server,
        &format!("{path}/publish"),
        &[("csrf", &token), ("expected_revision", &new_revision)],
    )
    .await;
    assert_eq!(denied.status, 409);
    assert!(denied.raw.contains("approval_required"));
    let current = get(&server, &format!("/v1/pieces/{id}")).await;
    assert_eq!(current.body["status"], "pending_review");
    assert!(current.body["review_decision_id"].is_null());
}

#[tokio::test]
async fn browser_receipts_preserve_fake_platform_outcomes_and_repeat_safety() {
    use fake_linkedin::{AuthMode, PostMode, TokenMode};
    use memory_store::MemoryStore;
    for (mode, expected, heading) in [
        (
            PostMode::Status(403),
            "refused",
            "Platform declined this post",
        ),
        (
            PostMode::Status(401),
            "consent_required",
            "Account permission needed",
        ),
        (
            PostMode::Status(429),
            "rate_limited",
            "Platform limit reached",
        ),
        (PostMode::NoId, "unknown", "Sending not confirmed"),
        (
            PostMode::Posted("urn:li:share:7000000000000000001".into()),
            "posted",
            "Posted",
        ),
    ] {
        let fake = fake_linkedin::start(AuthMode::Consent, TokenMode::ok(), mode).await;
        let server = start_linkedin(
            Scripted::new(scripted::text("A reviewed draft.")),
            fast(),
            &fake,
            MemoryStore::with("linkedin", stored_token("2026-12-29T18:00:00Z")),
            false,
            true,
        )
        .await;
        let piece = piece_of(&create(&server, THOUGHT).await);
        assert_eq!(approve(&server, &piece).await.status, 200);
        let path = format!("/review/{}/publish", id_of(&piece));
        let revision = revision_of(&piece);
        let fields = [
            ("csrf", server.token.as_str()),
            ("expected_revision", revision.as_str()),
        ];
        let receipt = form(&server, &path, &fields).await;
        assert_eq!(receipt.status, 200);
        assert!(
            receipt.raw.contains(&format!("<h2>{heading}</h2>")),
            "{}",
            receipt.raw
        );
        assert!(receipt.raw.contains("Attempts: 1"));
        assert!(receipt.raw.contains(&format!("Status: {expected}.")));
        assert_eq!(fake.posts().len(), 1);
        if expected != "posted" {
            assert!(!receipt.raw.contains("<h2>Posted</h2>"));
            assert!(receipt.raw.contains("Post id: not recorded"));
        }
        if matches!(expected, "posted" | "unknown") {
            let repeated = form(&server, &path, &fields).await;
            assert!(repeated.raw.contains(&format!("<h2>{heading}</h2>")));
            assert_eq!(fake.posts().len(), 1);
        }
    }
}

#[tokio::test]
async fn malformed_page_ids_use_fixed_core_errors_and_recovery_links() {
    let server = start(Scripted::new(scripted::text("A saved version."))).await;
    for path in ["/review/%FF", "/review/%FF/edit", "/review/%FF/render"] {
        let page = get(&server, path).await;
        assert_eq!(page.status, 404, "{}", page.raw);
        assert!(page.raw.contains("piece_not_found"));
        assert!(page.raw.contains("Back to studio"));
        assert!(!page.raw.contains("%FF"));
    }
    let token = session(&server).await;
    let failed = form(
        &server,
        "/review/%FF/edit",
        &[
            ("csrf", &token),
            ("expected_revision", "old"),
            ("text", "A changed draft."),
        ],
    )
    .await;
    assert_eq!(failed.status, 404);
    assert!(failed.raw.contains("piece_not_found"));
}

#[tokio::test]
async fn editing_surfaces_do_not_offer_approval_of_unsaved_content() {
    let server = start(Scripted::new(scripted::text("A saved version."))).await;
    let piece = piece_of(&create(&server, THOUGHT).await);
    for action in ["edit", "render"] {
        let page = get(&server, &format!("/review/{}/{action}", id_of(&piece))).await;
        assert_eq!(page.status, 200);
        assert!(page.raw.contains("<textarea"));
        assert!(!page.raw.contains("/approve"));
        assert!(!page.raw.contains("/publish"));
    }
    let review = get(&server, &format!("/review/{}", id_of(&piece))).await;
    assert!(!review.raw.contains("<textarea"));
    assert!(!review.raw.contains("<label class="));
}

#[tokio::test]
async fn browser_content_is_escaped_and_invalid_forms_do_not_mutate() {
    let markup = "</textarea><script>alert('x')</script>&\"";
    let server = start(Scripted::new(scripted::text(markup))).await;
    let token = session(&server).await;
    let created = create(&server, markup).await;
    let piece = piece_of(&created);
    let id = id_of(&piece);
    let page = get(&server, &format!("/review/{id}")).await;
    assert!(page.raw.contains("&lt;/textarea&gt;&lt;script&gt;"));
    assert!(!page.raw.contains("<script"));
    let missing = form(
        &server,
        &format!("/review/{id}/edit"),
        &[("text", "A different draft.")],
    )
    .await;
    assert_eq!(missing.status, 403);
    let repeated = form(
        &server,
        &format!("/review/{id}/edit"),
        &[
            ("csrf", &token),
            ("csrf", &token),
            ("text", "A different draft."),
        ],
    )
    .await;
    assert_eq!(repeated.status, 403);
    let invalid = form(
        &server,
        &format!("/review/{id}/edit"),
        &[
            ("csrf", &token),
            ("expected_revision", &revision_of(&piece)),
            ("text", "  "),
        ],
    )
    .await;
    assert_eq!(invalid.status, 400);
    let current = get(&server, &format!("/v1/pieces/{id}")).await;
    assert_eq!(current.body, piece);
}

#[tokio::test]
async fn browser_visual_approval_uses_displayed_revision_and_exact_hashes() {
    let server = start(Scripted::new(scripted::text("A visual draft."))).await;
    let piece = piece_of(&create(&server, THOUGHT).await);
    let token = session(&server).await;
    let path = format!("/review/{}", id_of(&piece));
    let rendered = form(
        &server,
        &format!("{path}/render"),
        &[
            ("csrf", &token),
            ("expected_revision", &revision_of(&piece)),
            ("template", "paper"),
            ("slides", "First slide.\nSecond slide."),
        ],
    )
    .await;
    assert_eq!(rendered.status, 200, "{}", rendered.raw);
    assert!(rendered.raw.contains("<iframe"));
    assert!(rendered.raw.contains("2 pages"));
    let pdf = client()
        .get(server.url(&format!("/v1/pieces/{}/assets/carousel", id_of(&piece))))
        .send()
        .await
        .unwrap();
    assert_eq!(pdf.headers().get("x-frame-options").unwrap(), "SAMEORIGIN");
    assert!(
        pdf.headers()
            .get("content-security-policy")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("frame-ancestors 'self'")
    );
    let revision = field(&rendered.raw, "expected_revision");
    let image = field(&rendered.raw, "image_sha256");
    let carousel = field(&rendered.raw, "carousel_sha256");
    assert_eq!(image.len(), 64);
    assert_eq!(carousel.len(), 64);
    let bad = form(
        &server,
        &format!("{path}/approve"),
        &[
            ("csrf", &token),
            ("expected_revision", &revision),
            ("image_sha256", "wrong"),
            ("carousel_sha256", &carousel),
        ],
    )
    .await;
    assert_eq!(bad.status, 409);
    assert!(bad.raw.contains("asset_conflict"));
    let approved = form(
        &server,
        &format!("{path}/approve"),
        &[
            ("csrf", &token),
            ("expected_revision", &revision),
            ("image_sha256", &image),
            ("carousel_sha256", &carousel),
        ],
    )
    .await;
    assert_eq!(approved.status, 200);
    assert!(approved.raw.contains("Approved"));
    let edited = form(
        &server,
        &format!("{path}/edit"),
        &[
            ("csrf", &token),
            ("expected_revision", &revision),
            ("text", "Changed after asset approval."),
        ],
    )
    .await;
    assert!(!edited.raw.contains("<iframe"));
    assert!(edited.raw.contains("Render previews"));
    let current: Value = get(&server, &format!("/v1/pieces/{}", id_of(&piece)))
        .await
        .body;
    assert_eq!(current["status"], "pending_review");
    assert!(current.get("assets").is_none());
}

// [itest->req~studio-api.saved-channel-browser-identity~1]
#[tokio::test]
async fn platform_display_names_are_shared_by_review_preview_and_discovery() {
    let server = start(Scripted::new(scripted::text("A saved platform draft."))).await;
    for (channel, display) in [
        ("x", "X"),
        ("bluesky", "Bluesky"),
        ("threads", "Threads"),
        ("mastodon", "Mastodon"),
        ("linkedin_text", "LinkedIn"),
        ("facebook_page", "Facebook"),
        ("instagram", "Instagram"),
        ("manual_export", "Manual export"),
    ] {
        let created = post(
            &server,
            "/v1/thoughts",
            json!({"text":THOUGHT,"channels":[channel]}),
        )
        .await;
        assert_eq!(created.status, 201, "{}", created.raw);
        let p = piece_of(&created);
        let page = get(&server, &format!("/review/{}", id_of(&p))).await;
        assert!(
            page.raw
                .contains(&format!("<p class=\"channel-name\">{display}</p>")),
            "{channel}: {}",
            page.raw
        );
        assert!(page.raw.contains(&format!("{display} preview")));
        assert!(
            get(&server, "/")
                .await
                .raw
                .contains(&format!("<span>{display}</span>"))
        );
        assert_eq!(
            p["channel"], channel,
            "machine identity must stay unchanged"
        );
    }
}

// [itest->req~studio-api.browser-x-cost-consent~1]
#[tokio::test]
async fn browser_cost_consent_rejects_foreign_origin_missing_fields_and_non_x_piece() {
    let server = start(Scripted::new(scripted::text("A saved draft."))).await;
    let p = approved_x(&server).await;
    let path = format!("/review/{}/accept-cost", id_of(&p));
    let fields = [
        ("csrf", server.token.as_str()),
        ("expected_revision", p["revision"].as_str().unwrap()),
        ("accepted_cost_microusd", "15000"),
        ("pricing_as_of", "2026-10-31"),
    ];
    for origin in ["https://foreign.invalid", "null"] {
        let refused = answer(
            client()
                .post(server.url(&path))
                .header("origin", origin)
                .form(&fields)
                .send()
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(refused.status, 403);
        assert!(refused.raw.contains("origin_not_allowed"));
    }
    for index in [2, 3] {
        let partial: Vec<_> = fields
            .iter()
            .enumerate()
            .filter_map(|(i, f)| (i != index).then_some(*f))
            .collect();
        let refused = form(&server, &path, &partial).await;
        assert_eq!(refused.status, 409);
        assert!(refused.raw.contains("cost_confirmation_required"));
    }
    let other = piece_of(
        &post(
            &server,
            "/v1/thoughts",
            json!({"text":THOUGHT,"channels":["bluesky"]}),
        )
        .await,
    );
    let refused = form(
        &server,
        &format!("/review/{}/accept-cost", id_of(&other)),
        &fields,
    )
    .await;
    assert_eq!(refused.status, 400);
    assert!(refused.raw.contains("invalid_request"));
    let page = get(&server, &format!("/review/{}", id_of(&p))).await;
    assert!(
        page.raw
            .contains("class=\"publish\" type=\"submit\" disabled")
    );
}

async fn approved_x(server: &TestServer) -> Value {
    let p = piece_of(
        &post(
            server,
            "/v1/thoughts",
            json!({"text":THOUGHT,"channels":["x"]}),
        )
        .await,
    );
    let approved = send_json(
        server,
        reqwest::Method::PATCH,
        &format!("/v1/pieces/{}", id_of(&p)),
        json!({"expected_revision":p["revision"],"status":"approved"}),
    )
    .await;
    assert_eq!(approved.status, 200, "{}", approved.raw);
    approved.body
}

// [itest->req~studio-api.browser-x-cost-consent~1]
#[tokio::test]
async fn x_publish_is_natively_disabled_before_cost_consent() {
    let server = start(Scripted::new(scripted::text("A saved X draft."))).await;
    let p = approved_x(&server).await;
    let page = get(&server, &format!("/review/{}", id_of(&p))).await;
    assert!(
        page.raw
            .contains("class=\"publish\" type=\"submit\" disabled"),
        "Publish must really be disabled"
    );
    assert!(
        page.raw
            .contains("<time class=\"pricing-date\" datetime=\"2026-10-31\">2026-10-31</time>")
    );
    assert!(!page.raw.contains("<script"));
}

// [itest->req~studio-api.browser-x-cost-consent~1]
#[tokio::test]
async fn x_cost_consent_records_only_matching_quote_and_clears_on_revision() {
    let server = start(Scripted::new(scripted::text("A saved X draft."))).await;
    let p = approved_x(&server).await;
    let path = format!("/review/{}", id_of(&p));
    let revision = p["revision"].as_str().unwrap();
    let fields = [
        ("csrf", server.token.as_str()),
        ("expected_revision", revision),
        ("accepted_cost_microusd", "15000"),
        ("pricing_as_of", "2026-10-31"),
    ];
    for (index, bad, code) in [
        (1, "stale", "revision_conflict"),
        (2, "14999", "cost_confirmation_required"),
        (2, "15001", "cost_confirmation_required"),
        (3, "2026-10-30", "cost_confirmation_required"),
        (0, "wrong-session", "session_required"),
    ] {
        let mut invalid = fields;
        invalid[index].1 = bad;
        let refused = form(&server, &format!("{path}/accept-cost"), &invalid).await;
        assert!(refused.raw.contains(code), "{}", refused.raw);
        assert!(
            get(&server, &path)
                .await
                .raw
                .contains("class=\"publish\" type=\"submit\" disabled")
        );
    }
    let forged = form(&server, &format!("{path}/publish"), &fields[..3]).await;
    assert!(forged.raw.contains("cost_confirmation_required"));
    let accepted = form(&server, &format!("{path}/accept-cost"), &fields).await;
    assert_eq!(accepted.status, 200, "{}", accepted.raw);
    assert!(
        accepted
            .raw
            .contains("Estimated cost accepted for this version.")
    );
    assert!(
        accepted
            .raw
            .contains("class=\"publish\" type=\"submit\">Publish")
    );
    assert_eq!(
        std::fs::read_dir(server.dir.path().join("outbox"))
            .unwrap()
            .count(),
        0,
        "acceptance must not publish"
    );
    let published = form(&server, &format!("{path}/publish"), &fields[..2]).await;
    assert_eq!(published.status, 200, "{}", published.raw);
    assert!(published.raw.contains("contained"));
    assert!(published.raw.contains("Recorded estimate: USD 0.015"));
    assert!(published.raw.contains("Post id: not recorded"));
    assert!(published.raw.contains("Post URL: not recorded"));
    let saved_receipt = get(&server, &path).await;
    assert!(saved_receipt.raw.contains("Recorded estimate: USD 0.015"));
    assert!(saved_receipt.raw.contains("2026-10-31"));
    let edited = send_json(
        &server,
        reqwest::Method::PATCH,
        &format!("/v1/pieces/{}", id_of(&p)),
        json!({"expected_revision":revision,"text":"A changed X draft, same estimate."}),
    )
    .await;
    assert_eq!(edited.status, 200, "{}", edited.raw);
    let updated = edited.body;
    let approved = send_json(
        &server,
        reqwest::Method::PATCH,
        &format!("/v1/pieces/{}", id_of(&p)),
        json!({"expected_revision":updated["revision"],"status":"approved"}),
    )
    .await;
    assert_eq!(approved.status, 200);
    let page = get(&server, &path).await;
    assert!(
        page.raw
            .contains("class=\"publish\" type=\"submit\" disabled")
    );
    assert!(
        !page
            .raw
            .contains("Estimated cost accepted for this version.")
    );
    let refused = form(
        &server,
        &format!("{path}/publish"),
        &[
            ("csrf", &server.token),
            ("expected_revision", updated["revision"].as_str().unwrap()),
            ("accepted_cost_microusd", "15000"),
        ],
    )
    .await;
    assert!(refused.raw.contains("cost_confirmation_required"));
}

// [itest->req~studio-api.browser-text-channel-selection~1]
#[tokio::test]
async fn text_channels_are_repeated_fields_with_strict_scalar_and_unknown_field_checks() {
    let provider = Scripted::new(scripted::text("A synthetic draft."));
    let server = start(provider.clone()).await;
    let token = server.token.as_str();
    let valid = [
        ("csrf", token),
        ("text", THOUGHT),
        ("channels", "bluesky"),
        ("channels", "x"),
    ];
    let created = form(&server, "/actions/thought", &valid).await;
    assert_eq!(created.status, 200, "{}", created.raw);
    let requests = provider.requests();
    assert_eq!(requests.len(), 2);
    let pieces = get(&server, "/v1/pieces").await;
    assert_eq!(pieces.body["pieces"].as_array().unwrap().len(), 2);
    for (extra, expected) in [
        (vec![], 400),
        (vec![("channels", "x"), ("channels", "x")], 400),
        (vec![("channels", "unregistered")], 422),
        (vec![("channels", "x"), ("text", THOUGHT)], 400),
        (vec![("channels", "x"), ("unexpected", "ignored?")], 400),
        (vec![("channels", "x"), ("csrf", token)], 403),
    ] {
        let mut fields = vec![("csrf", token), ("text", THOUGHT)];
        fields.extend(extra);
        let refused = form(&server, "/actions/thought", &fields).await;
        assert_eq!(refused.status, expected, "{}", refused.raw);
    }
    assert_eq!(provider.requests().len(), 2, "invalid forms never draft");
    assert_eq!(get(&server, "/?channel=x").await.status, 400);
    let home = get(&server, "/").await;
    assert!(
        home.raw
            .contains("name=\"channels\" value=\"linkedin_text\" checked")
    );
}

// [itest->req~studio-api.comfortable-text-review~1]
#[tokio::test]
async fn plain_counts_keep_channel_rules_in_closed_details_on_every_writing_surface() {
    let server = start(Scripted::new(scripted::text("e\u{301}🌳"))).await;
    for (channel, count, unit) in [
        ("bluesky", 2, "graphemes"),
        ("x", 4, "weighted units"),
        ("threads", 7, "UTF-8 bytes"),
        ("linkedin_text", 3, "characters"),
    ] {
        let piece = piece_of(
            &post(
                &server,
                "/v1/thoughts",
                json!({"text":THOUGHT,"channels":[channel]}),
            )
            .await,
        );
        assert_eq!(
            piece["fit"]["char_count"], count,
            "display must retain the core's counting rule"
        );
        let path = format!("/review/{}", id_of(&piece));
        for route in [
            "/".into(),
            path.clone(),
            format!("{path}/edit"),
            format!("{path}/render"),
        ] {
            let page = get(&server, &route).await;
            assert!(
                page.raw.contains(&format!("{count} of ")),
                "{route}: {}",
                page.raw
            );
            assert!(page.raw.contains("characters - "));
            assert!(
                page.raw
                    .contains("<details class=\"count-details\"><summary>Details</summary>")
            );
            assert!(page.raw.contains(unit));
            assert!(!page.raw.contains("<details open"));
            assert!(!page.raw.contains("Conservative app"));
        }
    }
    let missing = get(&server, "/review/not-a-saved-piece").await;
    assert!(
        missing
            .raw
            .contains("We couldn't find that piece. It may have been deleted.")
    );
    assert!(
        missing
            .raw
            .contains("<details class=\"error-details\"><summary>Details</summary>")
    );
    assert!(!missing.raw.contains("Nothing is assumed"));
}

// [itest->req~studio-api.comfortable-text-review~1]
#[tokio::test]
async fn text_preview_and_all_edit_surfaces_keep_untrusted_content_inert_and_handles_exact() {
    let markup = "<img src=x onerror=alert('x')> & \"quote\"\n\nSecond paragraph.";
    let server = start(Scripted::new(scripted::text(markup))).await;
    let piece = piece_of(&create(&server, markup).await);
    let id = id_of(&piece);
    let path = format!("/review/{id}");
    let mut stored = piece.clone();
    stored["routing"]["destination"]["handle"] = json!("@synthetic<unsafe>&\"'");
    std::fs::write(
        server.dir.path().join("pieces").join(format!("{id}.json")),
        serde_json::to_vec(&stored).unwrap(),
    )
    .unwrap();
    for route in [
        "/".into(),
        path.clone(),
        format!("{path}/edit"),
        format!("{path}/render"),
    ] {
        let page = get(&server, &route).await;
        assert_eq!(page.status, 200, "{}", page.raw);
        assert!(!page.raw.contains("<img src=x"));
        assert!(page.raw.contains("&lt;img src=x"));
        assert!(!page.raw.contains("class=\"eyebrow\""));
    }
    let review = get(&server, &path).await;
    assert!(review.raw.contains("Approximate local preview"));
    assert!(
        review
            .raw
            .contains("@synthetic&lt;unsafe&gt;&amp;&quot;&#39;")
    );
    assert!(!review.raw.contains("@@synthetic"));
    assert!(!review.raw.contains("No current visual assets"));
    stored.as_object_mut().unwrap().remove("routing");
    std::fs::write(
        server.dir.path().join("pieces").join(format!("{id}.json")),
        serde_json::to_vec(&stored).unwrap(),
    )
    .unwrap();
    assert!(get(&server, &path).await.raw.contains("not recorded"));
}

// [itest->req~studio-api.composed-browser-actions~1]
#[tokio::test]
async fn export_cannot_hide_a_non_retryable_publish_and_get_inspection_never_sends() {
    use fake_linkedin::{AuthMode, PostMode, TokenMode};
    use memory_store::MemoryStore;
    for mode in [
        PostMode::NoId,
        PostMode::Posted("urn:li:share:synthetic".into()),
    ] {
        let fake = fake_linkedin::start(AuthMode::Consent, TokenMode::ok(), mode).await;
        let server = start_linkedin(
            Scripted::new(scripted::text("A synthetic draft.")),
            fast(),
            &fake,
            MemoryStore::with("linkedin", stored_token("2026-12-29T18:00:00Z")),
            false,
            true,
        )
        .await;
        let piece = piece_of(&create(&server, THOUGHT).await);
        assert_eq!(approve(&server, &piece).await.status, 200);
        let id = id_of(&piece);
        let publish = post(
            &server,
            &format!("/v1/pieces/{id}/publish"),
            json!({"expected_revision":revision_of(&piece)}),
        )
        .await;
        assert_eq!(publish.status, 200);
        let export = post(
            &server,
            &format!("/v1/pieces/{id}/export"),
            json!({"expected_revision":revision_of(&piece), "target":"reddit"}),
        )
        .await;
        assert_eq!(export.status, 200, "{}", export.raw);
        assert_eq!(
            get(&server, &format!("/v1/pieces/{id}")).await.body["publish_status"],
            "exported"
        );
        let before = events(&server.dir);
        let review = get(&server, &format!("/review/{id}")).await;
        assert!(
            review
                .raw
                .contains("class=\"publish\" type=\"submit\" disabled>Publish"),
            "{}",
            review.raw
        );
        assert!(review.raw.contains("exported"));
        assert!(
            review
                .raw
                .contains(publish.body["request_id"].as_str().unwrap()),
            "persisted receipt survives export"
        );
        assert_eq!(fake.posts().len(), 1);
        assert_eq!(events(&server.dir), before, "GET inspection is read-only");
    }
}
