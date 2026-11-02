//! The shared fixtures in contracts/studio-api/ equal what the server really returns.
#[path = "../../syncloom-core/tests/support/fake_social.rs"]
mod fake;
#[path = "../../syncloom-core/tests/support/social.rs"]
mod social_fixture;
mod support;
use serde_json::json;
use std::collections::BTreeSet;
use support::fake_linkedin::{AuthMode, PostMode, TokenMode};
use support::memory_store::MemoryStore;
use support::scripted::{Reply, Scripted, text};
use support::*;

/// The documented error table, typed from the studio-api spec (not from the implementation).
const ERROR_TABLE: [(u16, &str); 37] = [
    (403, "origin_not_allowed"),
    (403, "session_required"),
    (400, "invalid_request"),
    (403, "host_not_allowed"),
    (404, "not_found"),
    (404, "piece_not_found"),
    (405, "method_not_allowed"),
    (409, "invalid_status_transition"),
    (409, "revision_conflict"),
    (409, "approval_required"),
    (409, "sign_in_in_progress"),
    (409, "sign_in_not_started"),
    (403, "sign_in_denied"),
    (403, "sign_in_state_mismatch"),
    (422, "thought_too_short"),
    (422, "unsupported_channel"),
    (500, "internal_error"),
    (502, "provider_error"),
    (502, "sign_in_rejected"),
    (503, "provider_not_configured"),
    (503, "linkedin_not_configured"),
    (503, "mastodon_not_configured"),
    (503, "meta_not_configured"),
    (503, "platform_not_configured"),
    (409, "cost_confirmation_required"),
    (403, "posting_permission_missing"),
    (403, "posting_permission_unverified"),
    (504, "provider_timeout"),
    (504, "sign_in_timed_out"),
    (503, "renderer_missing"),
    (503, "renderer_version_mismatch"),
    (503, "renderer_font_mismatch"),
    (504, "renderer_timeout"),
    (502, "renderer_failed"),
    (409, "assets_required"),
    (409, "asset_integrity"),
    (409, "asset_conflict"),
];

#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
async fn with_id(request: reqwest::RequestBuilder) -> Answer {
    let url = request
        .try_clone()
        .unwrap()
        .build()
        .unwrap()
        .url()
        .join("/v1/session")
        .unwrap();
    let session: serde_json::Value = client()
        .get(url)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let request = request.header("x-syncloom-token", session["token"].as_str().unwrap());
    answer(
        request
            .header("X-Request-Id", FIXTURE_ID)
            .send()
            .await
            .unwrap(),
    )
    .await
}

#[expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
async fn produce(code: &str) -> Answer {
    match code {
        "session_required" => {
            let server = start(Scripted::new(text("x"))).await;
            answer(
                client()
                    .post(server.url("/v1/thoughts"))
                    .header("X-Request-Id", FIXTURE_ID)
                    .send()
                    .await
                    .unwrap(),
            )
            .await
        }
        "origin_not_allowed" => {
            let server = start(Scripted::new(text("x"))).await;
            with_id(
                client()
                    .get(server.url("/v1/health"))
                    .header("origin", "https://evil.invalid"),
            )
            .await
        }
        "meta_not_configured" => {
            let server = start(Scripted::new(text("x"))).await;
            with_id(client().get(server.url("/v1/meta/connection"))).await
        }
        "platform_not_configured" => {
            let server = start(Scripted::new(text("x"))).await;
            with_id(client().get(server.url("/v1/x/connection"))).await
        }
        "cost_confirmation_required" => {
            use syncloom_core::social::{SocialConfig, SocialPlatforms};
            let q = SocialPlatforms::new(
                SocialConfig::parse(|_| None).unwrap(),
                MemoryStore::new(),
                fixed_clock(),
                fast().sign_in_deadline,
            );
            let server = start_studio(Scripted::new(text("A short test post.")), fast(), |s| {
                s.with_delivery("x", q.x)
            })
            .await;
            let created = post(
                &server,
                "/v1/thoughts",
                json!({"text":THOUGHT,"channels":["x"]}),
            )
            .await;
            let piece = &created.body["pieces"][0];
            approve(&server, piece).await;
            with_id(
                client()
                    .post(server.url(&format!("/v1/pieces/{}/publish", id_of(piece))))
                    .json(&json!({"expected_revision":revision_of(piece)})),
            )
            .await
        }
        "mastodon_not_configured" => {
            let server = start(Scripted::new(text("x"))).await;
            with_id(client().get(server.url("/v1/mastodon/connection"))).await
        }
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
        "linkedin_not_configured" => {
            let fake =
                fake_linkedin::start(AuthMode::Consent, TokenMode::ok(), PostMode::NoId).await;
            let server = start_linkedin(
                Scripted::new(text("x")),
                fast(),
                &fake,
                MemoryStore::new(),
                false,
                false,
            )
            .await;
            with_id(client().post(server.url("/v1/linkedin/sign-in"))).await
        }
        "posting_permission_missing" => {
            let fake = fake_linkedin::start(
                AuthMode::Consent,
                TokenMode::Ok {
                    access_token: fake_linkedin::DEFAULT_TOKEN.into(),
                    expires_in: 3600,
                    scope: "openid,profile".into(),
                },
                PostMode::NoId,
            )
            .await;
            let server = start_linkedin(
                Scripted::new(text("x")),
                fast(),
                &fake,
                MemoryStore::new(),
                true,
                false,
            )
            .await;
            let started = post_empty(&server, "/v1/linkedin/sign-in").await;
            assert_eq!(started.status, 202, "{}", started.raw);
            client()
                .get(started.body["authorization_url"].as_str().unwrap())
                .send()
                .await
                .unwrap();
            with_id(client().get(server.url("/v1/linkedin/sign-in"))).await
        }
        "posting_permission_unverified" => {
            let fake = fake::Fake::threads().await;
            fake.reply("/v1.0/debug_token", 200, json!({"data":{"is_valid":true}}));
            let q = std::sync::Arc::new(social_fixture::social(
                &fake,
                MemoryStore::new(),
                social_fixture::clock(),
            ));
            let server = start_studio(
                Scripted::new(text("x")),
                syncloom_core::Settings::default(),
                |s| s.with_social(q),
            )
            .await;
            let started = post_empty(&server, "/v1/threads/sign-in").await;
            assert_eq!(started.status, 202);
            let url =
                reqwest::Url::parse(started.body["authorization_url"].as_str().unwrap()).unwrap();
            let state = url
                .query_pairs()
                .find(|(k, _)| k == "state")
                .unwrap()
                .1
                .into_owned();
            with_id(client().post(server.url("/v1/threads/complete")).json(&json!({"callback_url":format!("https://redirect.invalid/callback?code=test&state={state}")}))).await
        }
        "sign_in_not_started" => {
            let fake =
                fake_linkedin::start(AuthMode::Consent, TokenMode::ok(), PostMode::NoId).await;
            let server = start_linkedin(
                Scripted::new(text("x")),
                fast(),
                &fake,
                MemoryStore::new(),
                true,
                false,
            )
            .await;
            with_id(client().get(server.url("/v1/linkedin/sign-in"))).await
        }
        "sign_in_in_progress"
        | "sign_in_timed_out"
        | "sign_in_denied"
        | "sign_in_state_mismatch"
        | "sign_in_rejected" => {
            let auth = match code {
                "sign_in_denied" => AuthMode::Deny,
                "sign_in_state_mismatch" => AuthMode::WrongState,
                _ => AuthMode::Consent,
            };
            let token = if code == "sign_in_rejected" {
                TokenMode::Status(400)
            } else {
                TokenMode::ok()
            };
            let fake = fake_linkedin::start(auth, token, PostMode::NoId).await;
            let server = start_linkedin(
                Scripted::new(text("x")),
                fast(),
                &fake,
                MemoryStore::new(),
                true,
                false,
            )
            .await;
            let started = post_empty(&server, "/v1/linkedin/sign-in").await;
            assert_eq!(started.status, 202, "{}", started.raw);
            if code == "sign_in_in_progress" {
                return with_id(client().post(server.url("/v1/linkedin/sign-in"))).await;
            }
            if code != "sign_in_timed_out" {
                client()
                    .get(started.body["authorization_url"].as_str().unwrap())
                    .send()
                    .await
                    .unwrap();
            }
            with_id(client().get(server.url("/v1/linkedin/sign-in"))).await
        }
        "renderer_missing"
        | "renderer_version_mismatch"
        | "renderer_font_mismatch"
        | "renderer_timeout"
        | "renderer_failed"
        | "assets_required"
        | "asset_integrity"
        | "asset_conflict" => {
            #[path = "../../syncloom-core/tests/support/rendering.rs"]
            mod rendering;
            let process = tempfile::tempdir().unwrap();
            let mut config = rendering::config();
            match code {
                "renderer_missing" => config.binary = process.path().join("missing"),
                "renderer_version_mismatch" => config.version = "0.0.0.0".into(),
                "renderer_font_mismatch" => config.font_sha256 = "0".repeat(64),
                "renderer_failed" => {
                    config.binary = rendering::executable(process.path(), "exit 12")
                }
                "renderer_timeout" => {
                    config.binary = rendering::executable(process.path(), "exec /bin/sleep 30");
                    config.deadline = std::time::Duration::from_millis(200);
                }
                _ => {}
            }
            let server = start_studio(
                Scripted::new(text("Build with evidence.")),
                syncloom_core::Settings::default(),
                |s| s.with_renderer(config),
            )
            .await;
            let piece = piece_of(&create(&server, THOUGHT).await);
            let id = id_of(&piece);
            if code == "assets_required" {
                return with_id(client().get(server.url(&format!("/v1/pieces/{id}/assets/image"))))
                    .await;
            }
            let rendered = with_id(client().post(server.url(&format!("/v1/pieces/{id}/render"))).json(&json!({
                "expected_revision":revision_of(&piece), "template":"midnight", "slides":["One slide"]
            }))).await;
            if code == "asset_conflict" {
                assert_eq!(rendered.status, 200);
                return with_id(client().patch(server.url(&format!("/v1/pieces/{id}"))).json(&json!({
                    "status":"approved", "expected_revision":revision_of(&rendered.body),
                    "expected_assets":{"image_sha256":"0".repeat(64), "carousel_sha256":rendered.body["assets"]["carousel_sha256"]}
                }))).await;
            }
            if code == "asset_integrity" {
                assert_eq!(rendered.status, 200);
                let hash = rendered.body["assets"]["image_sha256"].as_str().unwrap();
                std::fs::write(server.dir.path().join("assets").join(hash), b"damaged").unwrap();
                return with_id(client().get(server.url(&format!("/v1/pieces/{id}/assets/image"))))
                    .await;
            }
            rendered
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
