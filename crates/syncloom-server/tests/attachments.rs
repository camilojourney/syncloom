//! Selected local bytes through the actual loopback HTTP server, not handler mocks.
#[path = "attachments/browser.rs"]
mod browser;
#[path = "attachments/resources.rs"]
mod resources;
#[path = "attachments/restart.rs"]
mod restart;
mod support;
use reqwest::multipart::{Form, Part};
use serde_json::{Value, json};
use support::{
    scripted::{Scripted, text},
    *,
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const FILE: &[u8] = b"\xef\xbb\xbf# A small release\r\n\r\nKeep  the <script>words</script> inert.\rFinish locally.\tNo publication.\n";
const EXTRACTED: &str = "# A small release\n\nKeep  the <script>words</script> inert.\nFinish locally.\tNo publication.\n";

fn form(bytes: &[u8], name: &str, mime: Option<&str>) -> Result<Form> {
    let mut part = Part::bytes(bytes.to_vec()).file_name(name.to_owned());
    if let Some(mime) = mime {
        part = part.mime_str(mime)?;
    }
    Ok(Form::new().part("file", part))
}
async fn upload(
    server: &TestServer,
    bytes: &[u8],
    name: &str,
    mime: Option<&str>,
) -> Result<Answer> {
    Ok(answer(
        client()
            .post(server.url("/v1/capture/attachment"))
            .header("x-syncloom-token", &server.token)
            .header("x-request-id", FIXTURE_ID)
            .multipart(form(bytes, name, mime)?)
            .send()
            .await?,
    )
    .await)
}
fn selection(snapshot: &Value) -> Value {
    json!({"text":snapshot["source"]["attachment"]["extracted_text"], "source_revision":snapshot["source_revision"], "channels":["linkedin_text"]})
}

#[tokio::test]
async fn selected_bytes_extract_preview_confirm_and_concurrent_retry_preserve_source() -> Result {
    let provider = Scripted::new(text("A pending local draft."));
    let server = start(provider.clone()).await;
    let intake = upload(&server, FILE, "release.MD", Some("text/markdown")).await?;
    assert_eq!(intake.status, 200, "{}", intake.raw);
    assert_eq!(intake.body["source"]["type"], "attachment");
    assert_eq!(intake.body["source"]["raw_input"], "release.MD");
    let metadata = &intake.body["source"]["attachment"];
    assert_eq!(metadata["filename"], "release.MD");
    assert_eq!(metadata["format"], "markdown");
    assert_eq!(metadata["byte_count"], FILE.len());
    assert_eq!(
        metadata["content_sha256"],
        syncloom_core::visual::digest(FILE)
    );
    assert_eq!(metadata["captured_at"], "2026-10-30T18:00:00Z");
    assert_eq!(metadata["extracted_text"], EXTRACTED);
    assert!(provider.requests().is_empty());
    let selected = selection(&intake.body);
    let preview = post(&server, "/v1/capture/preview", selected.clone()).await;
    assert_eq!(preview.status, 200, "{}", preview.raw);
    assert_eq!(preview.body["text"], EXTRACTED);
    assert!(preview.body["approval"].is_null());
    let mut confirm = selected;
    confirm["expected_revision"] = preview.body["revision"].clone();
    for field in ["text", "source_revision", "expected_revision"] {
        let mut stale = confirm.clone();
        stale[field] = json!("a".repeat(64));
        assert_eq!(
            post(&server, "/v1/capture/confirm", stale).await.code(),
            "revision_conflict"
        );
    }
    let mut stale = confirm.clone();
    stale["channels"] = json!(["facebook_page"]);
    assert_eq!(
        post(&server, "/v1/capture/confirm", stale).await.code(),
        "revision_conflict"
    );
    assert!(provider.requests().is_empty());
    let (one, two) = tokio::join!(
        post(&server, "/v1/capture/confirm", confirm.clone()),
        post(&server, "/v1/capture/confirm", confirm)
    );
    let mut statuses = [one.status, two.status];
    statuses.sort_unstable();
    assert_eq!(statuses, [200, 201]);
    assert_eq!(one.body, two.body);
    assert_eq!(one.body["pieces"][0]["source"], intake.body["source"]);
    assert_eq!(one.body["pieces"][0]["status"], "pending_review");
    assert!(one.body["pieces"][0]["review_decision_id"].is_null());
    assert!(one.body["pieces"][0]["publish_status"].is_null());
    assert_eq!(provider.requests().len(), 1);
    assert_eq!(
        provider.requests()[0].thought,
        "# A small release Keep the <script>words</script> inert. Finish locally. No publication."
    );
    assert!(files(&server.dir, "outbox").is_empty());
    let audit = std::fs::read_to_string(server.dir.path().join("provenance/events.jsonl"))?;
    for secret in [
        "release.MD",
        "small release",
        "pending local draft",
        "script",
    ] {
        assert!(!audit.contains(secret));
    }
    assert_eq!(audit.lines().count(), 1);
    assert!(!server.dir.path().join("uploads").exists());
    Ok(())
}

#[tokio::test]
async fn malformed_unsupported_binary_and_metadata_are_refused_without_side_effects() -> Result {
    let provider = Scripted::new(text("Never called."));
    let server = start(provider.clone()).await;
    let cases: Vec<(&[u8], &str, Option<&str>, &str)> = vec![
        (
            b"This is a useful note.",
            "brief.pdf",
            Some("text/plain"),
            "attachment_unsupported",
        ),
        (
            b"This is a useful note.",
            "note.txt",
            Some("application/pdf"),
            "attachment_unsupported",
        ),
        (
            b"This is a useful note.",
            "note.txt",
            Some("text/markdown"),
            "attachment_unsupported",
        ),
        (b"\xff useful bytes", "note.txt", None, "attachment_invalid"),
        (
            b"  %PDF-1.7 hidden binary",
            "note.txt",
            None,
            "attachment_unsupported",
        ),
        (b"\xef\xbb\xbf", "note.txt", None, "thought_too_short"),
        (b" \r\n\t ", "note.md", None, "thought_too_short"),
        (
            b"Nul\0binary payload",
            "note.md",
            None,
            "attachment_invalid",
        ),
        (
            b"Escape\x1bhidden payload",
            "note.md",
            None,
            "attachment_invalid",
        ),
        (
            "Hide\u{202e}these words".as_bytes(),
            "note.md",
            None,
            "attachment_invalid",
        ),
        (
            b"This is a useful note.",
            "../note.txt",
            None,
            "attachment_invalid",
        ),
        (
            b"This is a useful note.",
            "C:\\note.txt",
            None,
            "attachment_invalid",
        ),
        (
            b"This is a useful note.",
            "bad\u{2066}.txt",
            None,
            "attachment_invalid",
        ),
        (b"", "note.md", None, "thought_too_short"),
        (b"a b c d e f g", "note.txt", None, "thought_too_short"),
    ];
    for (bytes, name, mime, code) in cases {
        let result = upload(&server, bytes, name, mime).await?;
        assert_eq!(result.status, 422, "{name}: {}", result.raw);
        assert_eq!(result.code(), code, "{name}");
        assert_eq!(result.as_fixture(), fixture(&format!("errors/{code}.json")));
        assert!(!result.raw.contains(name));
    }
    assert_eq!(
        upload(&server, b"A useful note.", "", None).await?.code(),
        "attachment_invalid"
    );
    let long_name = format!("{}.txt", "a".repeat(125));
    assert_eq!(
        upload(&server, b"Useful source text.", &long_name, None)
            .await?
            .code(),
        "attachment_invalid"
    );
    assert!(provider.requests().is_empty());
    assert!(files(&server.dir, "capture").is_empty());
    assert!(events(&server.dir).is_empty());
    Ok(())
}

#[tokio::test]
async fn text_and_byte_limits_are_distinct_without_silent_truncation() -> Result {
    let server = start(Scripted::new(text("Never called."))).await;
    for (n, status) in [(7999, 200), (8000, 200), (8001, 422)] {
        let source = "界".repeat(n);
        let response = upload(
            &server,
            source.as_bytes(),
            "unicode.txt",
            Some("application/octet-stream"),
        )
        .await?;
        assert_eq!(response.status, status, "{}", response.raw);
        if status == 200 {
            assert_eq!(
                response.body["source"]["attachment"]["extracted_text"],
                source
            );
        } else {
            assert_eq!(response.code(), "attachment_too_large");
            assert_eq!(
                response.as_fixture(),
                fixture("errors/attachment_too_large.json")
            );
        }
    }
    // Byte bound precedes UTF-8 decoding. At the byte boundary this payload is still invalid UTF-8,
    // not a valid maximum-text document; conjunctive limits are not relaxed to manufacture one.
    for (n, code) in [
        (262143, "attachment_invalid"),
        (262144, "attachment_invalid"),
        (262145, "attachment_too_large"),
    ] {
        assert_eq!(
            upload(&server, &vec![0xff; n], "bytes.txt", None)
                .await?
                .code(),
            code
        );
    }
    let name = format!("{}.txt", "a".repeat(124));
    assert_eq!(
        upload(&server, b"a b c d e f g h", &name, None)
            .await?
            .status,
        200
    );
    Ok(())
}

#[tokio::test]
async fn upload_shape_and_local_request_guards_are_enforced() -> Result {
    let provider = Scripted::new(text("Never called."));
    let server = start(provider.clone()).await;
    let other = start(provider.clone()).await;
    for (header, value, code) in [
        ("host", "evil.invalid", "host_not_allowed"),
        ("origin", "null", "origin_not_allowed"),
        ("sec-fetch-site", "cross-site", "origin_not_allowed"),
        ("x-syncloom-token", other.token.as_str(), "session_required"),
    ] {
        let response = answer(
            client()
                .post(server.url("/v1/capture/attachment"))
                .header("x-syncloom-token", &server.token)
                .header(header, value)
                .multipart(form(FILE, "note.md", None)?)
                .send()
                .await?,
        )
        .await;
        assert_eq!(response.status, 403);
        assert_eq!(response.code(), code);
    }
    let response = answer(
        client()
            .post(server.url("/v1/capture/attachment"))
            .multipart(form(FILE, "note.md", None)?)
            .send()
            .await?,
    )
    .await;
    assert_eq!(response.code(), "session_required");
    let foreign = answer(
        client()
            .post(server.url("/v1/capture/attachment"))
            .header("x-syncloom-token", &other.token)
            .multipart(form(FILE, "note.md", None)?)
            .send()
            .await?,
    )
    .await;
    assert_eq!(foreign.status, 403);
    assert_eq!(foreign.code(), "session_required");
    for csrf in [
        vec![],
        vec![other.token.clone()],
        vec![server.token.clone(), server.token.clone()],
    ] {
        let mut body = form(FILE, "note.md", None)?;
        for token in csrf {
            body = body.text("csrf", token);
        }
        let response = answer(
            client()
                .post(server.url("/capture/attachment"))
                .multipart(body)
                .send()
                .await?,
        )
        .await;
        assert_eq!(response.status, 403, "{}", response.raw);
        assert!(response.raw.contains("session_required"));
    }
    for body in [
        Form::new(),
        form(FILE, "note.md", None)?.text("path", "/etc/passwd"),
        Form::new().part("file", Part::bytes(FILE.to_vec())),
        form(FILE, "note.txt\x1b", None)?, // Invalid multipart header bytes fail before extraction.
        form(FILE, "note.md", None)?.text("csrf", server.token.clone()),
        form(&vec![b'x'; 1_048_577], "large.txt", None)?, // Valid framing must still hit the request cap first.
        form(FILE, "note.md", None)?.part("file", Part::bytes(FILE.to_vec()).file_name("two.md")),
    ] {
        let response = answer(
            client()
                .post(server.url("/v1/capture/attachment"))
                .header("x-syncloom-token", &server.token)
                .multipart(body)
                .send()
                .await?,
        )
        .await;
        assert_eq!(response.code(), "invalid_request");
    }
    let missing_file = answer(
        client()
            .post(server.url("/capture/attachment"))
            .multipart(Form::new().text("csrf", server.token.clone()))
            .send()
            .await?,
    )
    .await;
    assert_eq!(missing_file.status, 400);
    assert!(missing_file.raw.contains("invalid_request"));
    let file_csrf = answer(
        client()
            .post(server.url("/capture/attachment"))
            .multipart(form(FILE, "note.md", None)?.part(
                "csrf",
                Part::bytes(server.token.as_bytes().to_vec()).file_name("token.txt"),
            ))
            .send()
            .await?,
    )
    .await;
    assert_eq!(file_csrf.status, 400);
    assert!(file_csrf.raw.contains("invalid_request"));
    for bytes in [b"bad framing".to_vec(), vec![b'x'; 1048577]] {
        let response = answer(
            client()
                .post(server.url("/v1/capture/attachment"))
                .header("x-syncloom-token", &server.token)
                .header("content-type", "multipart/form-data; boundary=bad")
                .body(bytes)
                .send()
                .await?,
        )
        .await;
        assert_eq!(response.code(), "invalid_request");
    }
    assert!(provider.requests().is_empty());
    assert!(files(&server.dir, "capture").is_empty());
    assert!(files(&other.dir, "capture").is_empty());
    Ok(())
}
