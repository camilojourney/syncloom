//! The shared fixtures in contracts/studio-api/ equal what the server really returns.
mod support;
use serde_json::json;
use std::collections::BTreeSet;
use support::*;

/// The documented error table, typed from the studio-api spec (not from the implementation).
const ERROR_TABLE: [(u16, &str); 14] = [
    (400, "invalid_request"),
    (403, "host_not_allowed"),
    (404, "not_found"),
    (404, "piece_not_found"),
    (405, "method_not_allowed"),
    (409, "invalid_status_transition"),
    (409, "revision_conflict"),
    (409, "approval_required"),
    (422, "thought_too_short"),
    (422, "unsupported_channel"),
    (500, "internal_error"),
    (502, "provider_error"),
    (503, "provider_not_configured"),
    (504, "provider_timeout"),
];

async fn with_id(request: reqwest::RequestBuilder) -> Answer {
    answer(
        request
            .header("X-Request-Id", FIXTURE_ID)
            .send()
            .await
            .unwrap(),
    )
    .await
}

async fn produce(code: &str) -> Answer {
    match code {
        "host_not_allowed" => {
            let server = start(Scripted::new(text("x"))).await;
            raw(
                server.addr,
                &head_with_hosts(
                    "GET /v1/health HTTP/1.1",
                    &["Host: evil.example"],
                    &[("X-Request-Id", FIXTURE_ID)],
                ),
            )
            .await
        }
        "not_found" => {
            let server = start(Scripted::new(text("x"))).await;
            with_id(client().get(server.url("/missing"))).await
        }
        "method_not_allowed" => {
            let server = start(Scripted::new(text("x"))).await;
            with_id(client().get(server.url("/v1/thoughts"))).await
        }
        "invalid_request" => {
            let server = start(Scripted::new(text("x"))).await;
            with_id(client().post(server.url("/v1/thoughts")).json(&json!({}))).await
        }
        "piece_not_found" => {
            let server = start(Scripted::new(text("x"))).await;
            with_id(client().get(server.url("/v1/pieces/thought-000000000000-linkedin_text"))).await
        }
        "thought_too_short" => {
            let server = start(Scripted::new(text("x"))).await;
            with_id(
                client()
                    .post(server.url("/v1/thoughts"))
                    .json(&json!({"text": "short", "channels": ["linkedin_text"]})),
            )
            .await
        }
        "unsupported_channel" => {
            let server = start(Scripted::new(text("x"))).await;
            with_id(
                client()
                    .post(server.url("/v1/thoughts"))
                    .json(&json!({"text": THOUGHT, "channels": ["tiktok_video"]})),
            )
            .await
        }
        "invalid_status_transition" | "revision_conflict" | "approval_required" => {
            let server = start(Scripted::new(text("A draft."))).await;
            let piece = piece_of(&create(&server, THOUGHT).await);
            let path = format!("/v1/pieces/{}", id_of(&piece));
            let revision = revision_of(&piece);
            match code {
                "invalid_status_transition" => {
                    approve(&server, &piece).await;
                    with_id(
                        client().patch(server.url(&path)).json(
                            &json!({"status": "pending_review", "expected_revision": revision}),
                        ),
                    )
                    .await
                }
                "revision_conflict" => {
                    with_id(
                        client().patch(server.url(&path)).json(
                            &json!({"status": "approved", "expected_revision": "0".repeat(64)}),
                        ),
                    )
                    .await
                }
                _ => {
                    with_id(
                        client()
                            .post(server.url(&format!("{path}/publish")))
                            .json(&json!({"expected_revision": revision})),
                    )
                    .await
                }
            }
        }
        "internal_error" => {
            let server = start(Scripted::new(Reply::Panic)).await;
            with_id(
                client()
                    .post(server.url("/v1/thoughts"))
                    .json(&json!({"text": THOUGHT, "channels": ["linkedin_text"]})),
            )
            .await
        }
        "provider_error" | "provider_timeout" | "provider_not_configured" => {
            let reply = match code {
                "provider_error" => Reply::Fail,
                "provider_timeout" => Reply::Hang,
                _ => Reply::Unconfigured,
            };
            let server = start_with(Scripted::new(reply), fast()).await;
            with_id(
                client()
                    .post(server.url("/v1/thoughts"))
                    .json(&json!({"text": THOUGHT, "channels": ["linkedin_text"]})),
            )
            .await
        }
        other => panic!("no producer for {other}"),
    }
}

// studio-api: Error envelope, Fixtures equal the server
#[tokio::test]
async fn every_error_fixture_equals_the_servers_response() {
    for (status, code) in ERROR_TABLE {
        let actual = produce(code).await;
        assert_eq!(actual.status, status, "{code}: {}", actual.raw);
        assert_eq!(
            actual.as_fixture(),
            fixture(&format!("errors/{code}.json")),
            "{code}"
        );
    }
}

#[test]
fn the_error_fixture_directory_holds_exactly_the_documented_codes_with_their_statuses() {
    let dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/studio-api/errors");
    let files: BTreeSet<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    let expected: BTreeSet<String> = ERROR_TABLE
        .iter()
        .map(|(_, code)| format!("{code}.json"))
        .collect();
    assert_eq!(files, expected);
    for (status, code) in ERROR_TABLE {
        let file = fixture(&format!("errors/{code}.json"));
        assert_eq!(file["status"], status, "{code}");
        assert_eq!(file["body"]["error"]["code"], code);
        assert_eq!(file["body"]["error"]["request_id"], FIXTURE_ID);
        let message = file["body"]["error"]["message"].as_str().unwrap();
        assert!(
            message.ends_with('.') && !message.is_empty(),
            "{code}: {message:?}"
        );
    }
}
