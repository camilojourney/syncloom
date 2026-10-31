//! Browser boundary tests use real TCP and the same core as the CLI, never mocked handlers.
mod support;
use serde_json::{Value, json};
use support::{scripted::Scripted, *};

async fn session(server: &TestServer) -> String {
    let response = get(server, "/v1/session").await;
    assert_eq!(response.status, 200, "{}", response.raw);
    response.body["token"].as_str().unwrap().to_owned()
}

fn field(html: &str, name: &str) -> String {
    html.split_once(&format!("name=\"{name}\" value=\""))
        .unwrap_or_else(|| panic!("missing {name} in {html}"))
        .1
        .split('"')
        .next()
        .unwrap()
        .to_owned()
}

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
        &[("csrf", &token), ("text", THOUGHT)],
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
    let receipt = form(
        &server,
        &format!("{path}/publish"),
        &[("csrf", &token), ("expected_revision", &revision)],
    )
    .await;
    assert!(receipt.raw.contains("contained"));
    assert!(receipt.raw.contains("Nothing was sent"));
    assert!(receipt.raw.contains("Attempts: 0"));
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
    for (mode, expected) in [
        (PostMode::Status(403), "refused"),
        (PostMode::Status(401), "consent_required"),
        (PostMode::Status(429), "rate_limited"),
        (PostMode::NoId, "unknown"),
        (
            PostMode::Posted("urn:li:share:7000000000000000001".into()),
            "posted",
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
            receipt.raw.contains(&format!("<h2>{expected}</h2>")),
            "{}",
            receipt.raw
        );
        assert!(receipt.raw.contains("Attempts: 1"));
        assert_eq!(fake.posts().len(), 1);
        if expected != "posted" {
            assert!(!receipt.raw.contains("<h2>posted</h2>"));
            assert!(receipt.raw.contains("Post id: none"));
        }
        if matches!(expected, "posted" | "unknown") {
            let repeated = form(&server, &path, &fields).await;
            assert!(repeated.raw.contains(&format!("<h2>{expected}</h2>")));
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
