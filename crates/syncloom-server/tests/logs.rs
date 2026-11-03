//! The request log (studio-api "Request log"): one line per request, no content.
//! Its own test binary because the global tracing subscriber is set once per process.
mod support;
use serde_json::json;
use std::sync::{Arc, Mutex};
use support::fake_linkedin::{AuthMode, PostMode, TokenMode};
use support::memory_store::MemoryStore;
use support::scripted::{Scripted, text};
use support::*;

#[derive(Clone, Default)]
struct Captured(Arc<Mutex<Vec<u8>>>);
impl std::io::Write for Captured {
    #[expect(
        clippy::unwrap_used,
        reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
    )]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Captured {
    type Writer = Captured;
    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

// studio-api: Request log, Content stays out of the logs
#[tokio::test]
async fn each_request_logs_one_content_free_line() {
    let captured = Captured::default();
    tracing::subscriber::set_global_default(syncloom_server::log_subscriber(captured.clone()))
        .unwrap();
    let key = "sk-ant-secret-canary";
    let draft = "A draft that repeats zebrafish and nothing else.";
    let fake = fake_linkedin::start(
        AuthMode::Consent,
        TokenMode::ok(),
        PostMode::Posted("urn:li:share:7000000000000000001".to_owned()),
    )
    .await;
    let server = start_linkedin(
        Scripted::new(text(draft)),
        fast(),
        &fake,
        MemoryStore::new(),
        true,
        true,
    )
    .await;
    let started = post_empty(&server, "/v1/linkedin/sign-in").await;
    assert_eq!(started.status, 202, "{}", started.raw);
    client()
        .get(started.body["authorization_url"].as_str().unwrap())
        .send()
        .await
        .unwrap();
    let signed_in = get(&server, "/v1/linkedin/sign-in").await;
    assert_eq!(signed_in.status, 200, "{}", signed_in.raw);
    let connection = get(&server, "/v1/linkedin/connection").await;
    assert_eq!(connection.body["status"], "signed_in");
    let created = create(&server, THOUGHT).await;
    let piece = piece_of(&created);
    let path = format!("/v1/pieces/{}", id_of(&piece));
    get(&server, &path).await;
    let approved = approve(&server, &piece).await;
    assert_eq!(approved.status, 200);
    let published = post(
        &server,
        &format!("{path}/publish"),
        json!({"expected_revision": revision_of(&piece)}),
    )
    .await;
    assert_eq!(published.status, 200, "{}", published.raw);
    assert_eq!(published.body["status"], "posted", "{}", published.raw);
    let exported = post(
        &server,
        &format!("{path}/export"),
        json!({"expected_revision":revision_of(&piece),"target":"reddit"}),
    )
    .await;
    assert_eq!(exported.body["status"], "exported");
    assert_eq!(get(&server, "/v1/mastodon/connection").await.status, 503);
    assert_eq!(
        post(&server, "/v1/capture/suggest", json!({"text":THOUGHT}))
            .await
            .status,
        200
    );
    let capture_preview = post(
        &server,
        "/v1/capture/preview",
        json!({"text":THOUGHT,"channels":["linkedin_text"]}),
    )
    .await;
    assert_eq!(capture_preview.status, 200);
    let capture_confirm = post(&server, "/v1/capture/confirm", json!({"text":THOUGHT,"channels":["linkedin_text"],"expected_revision":capture_preview.body["revision"]})).await;
    assert_eq!(capture_confirm.status, 201);
    let failed = answer(
        client()
            .get(server.url("/missing"))
            .header("X-Request-Id", "log-check-1")
            .header("Authorization", format!("Bearer {key}"))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(failed.status, 404);

    let raw = String::from_utf8(captured.0.lock().unwrap().clone()).unwrap();
    let lines: Vec<serde_json::Value> = raw
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .filter(|line: &serde_json::Value| line["message"] == "request_finished")
        .collect();
    assert_eq!(lines.len(), 15, "one line per request:\n{raw}");
    for line in &lines {
        for field in ["request_id", "route", "status", "total_ms", "outcome"] {
            assert!(line.get(field).is_some(), "missing {field} in {line}");
        }
        assert_eq!(line["outcome"], "completed");
    }
    let last = lines.last().unwrap();
    assert_eq!(last["request_id"], "log-check-1");
    assert_eq!(last["status"], 404);
    assert_eq!(last["error_code"], "not_found");
    assert_eq!(last["route"], "unmatched");
    let routes: Vec<&str> = lines.iter().map(|l| l["route"].as_str().unwrap()).collect();
    assert_eq!(
        routes,
        [
            "/v1/session",
            "/v1/linkedin/sign-in",
            "/v1/linkedin/sign-in",
            "/v1/linkedin/connection",
            "/v1/thoughts",
            "/v1/pieces/{piece_id}",
            "/v1/pieces/{piece_id}",
            "/v1/pieces/{piece_id}",
            "/v1/pieces/{piece_id}/publish",
            "/v1/pieces/{piece_id}/export",
            "/v1/mastodon/connection",
            "/v1/capture/suggest",
            "/v1/capture/preview",
            "/v1/capture/confirm",
            "unmatched"
        ]
    );
    for secret in [
        CANARY,
        &server.token,
        draft,
        key,
        id_of(&piece).as_str(),
        fake_linkedin::DEFAULT_TOKEN,
        CLIENT_SECRET_CANARY,
        "fake-code-",
        "urn:li:person:",
    ] {
        assert!(!raw.contains(secret), "{secret:?} reached the log:\n{raw}");
    }
}
