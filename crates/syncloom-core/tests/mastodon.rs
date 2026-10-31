#[path = "support/fake_mastodon.rs"]
mod fake_mastodon;
mod support;
use fake_mastodon::{Config, FakeMastodon};
use std::{sync::Arc, time::Duration};
use support::{fixed_clock, free_port, memory_store::MemoryStore};
use syncloom_core::{
    mastodon::{Mastodon, MastodonConfig},
    *,
};

fn adapter(fake: &FakeMastodon, store: Arc<MemoryStore>) -> Arc<Mastodon> {
    Arc::new(Mastodon::new(
        MastodonConfig::parse(
            Some(fake.base()),
            Some(&format!(
                "http://127.0.0.1:{}/mastodon/callback",
                free_port()
            )),
        )
        .unwrap(),
        store,
        fixed_clock(),
        Duration::from_secs(2),
    ))
}
async fn consent(fake: &FakeMastodon, store: Arc<MemoryStore>) -> Arc<Mastodon> {
    let m = adapter(fake, store);
    let start = m.sign_in_begin().await.unwrap();
    assert!(start.authorization_url.starts_with(fake.base()));
    let page = support::browser(&start.authorization_url).await;
    assert_eq!(page.status(), 200);
    let connection = m.sign_in_wait().await.unwrap();
    assert_eq!(connection.status, ConnectionStatus::SignedIn);
    assert_eq!(connection.expires_at, None);
    m
}
fn request() -> PostRequest {
    PostRequest {
        request_id: "dispatch-fixture".into(),
        text: "Build with evidence.".into(),
        image: None,
        slides: Vec::new(),
    }
}

#[tokio::test]
async fn per_instance_registration_consent_token_and_signout_are_private_and_isolated() {
    let fake = fake_mastodon::start(Config::default()).await;
    let store = MemoryStore::new();
    let m = consent(&fake, store.clone()).await;
    let registrations = fake.calls("apps");
    assert_eq!(registrations.len(), 1);
    let form: std::collections::HashMap<String, String> =
        url::form_urlencoded::parse(&registrations[0].body)
            .into_owned()
            .collect();
    assert_eq!(form["client_name"], "Syncloom");
    assert_eq!(form["scopes"], "write:statuses write:media");
    assert!(form["redirect_uris"].starts_with("http://127.0.0.1:"));
    assert_eq!(fake.calls("token").len(), 1);
    let token = store.get(&m.account()).unwrap();
    assert_eq!(token.access_token.expose(), fake_mastodon::TOKEN);
    assert_eq!(token.expires_at, None);
    assert!(
        !serde_json::to_string(&m.connection().unwrap())
            .unwrap()
            .contains(fake_mastodon::TOKEN)
    );
    let other = fake_mastodon::start(Config::default()).await;
    let second = adapter(&other, store.clone());
    assert_eq!(
        second.connection().unwrap().status,
        ConnectionStatus::SignedOut
    );
    assert_eq!(
        second.publish(request()).await,
        Err(PostFailure::ConsentRequired)
    );
    assert!(other.calls("statuses").is_empty());
    m.sign_out().unwrap();
    assert!(store.get(&m.account()).is_none());
}

#[tokio::test]
async fn wrong_state_denial_and_callback_host_send_no_token_exchange() {
    for config in [
        Config {
            wrong_state: true,
            ..Config::default()
        },
        Config {
            deny: true,
            ..Config::default()
        },
    ] {
        let expected = if config.deny {
            StudioError::SignInDenied
        } else {
            StudioError::SignInStateMismatch
        };
        let fake = fake_mastodon::start(config).await;
        let store = MemoryStore::new();
        let m = adapter(&fake, store.clone());
        let start = m.sign_in_begin().await.unwrap();
        let auth = reqwest::Url::parse(&start.authorization_url).unwrap();
        let params: std::collections::HashMap<_, _> = auth.query_pairs().into_owned().collect();
        let bad = reqwest::Client::new()
            .get(&params["redirect_uri"])
            .header("Host", "evil.example")
            .send()
            .await
            .unwrap();
        assert_eq!(bad.status(), 403);
        assert!(
            !bad.headers()
                .keys()
                .any(|h| h.as_str().starts_with("access-control-"))
        );
        support::browser(&start.authorization_url).await;
        assert_eq!(m.sign_in_wait().await, Err(expected));
        assert!(fake.calls("token").is_empty());
        assert!(store.get(&m.account()).is_none());
    }
}

#[tokio::test]
async fn image_alt_text_media_polling_and_status_idempotency_use_the_official_shapes() {
    let fake = fake_mastodon::start(Config {
        async_media: true,
        ..Config::default()
    })
    .await;
    let m = consent(&fake, MemoryStore::new()).await;
    let mut request = request();
    request.image = Some(platform::PostImage {
        bytes: b"fixture-png".to_vec(),
        alt_text: "Build with evidence.".into(),
    });
    let posted = m.publish(request).await.unwrap();
    assert_eq!(posted.post_id, "status-1");
    assert_eq!(fake.calls("media").len(), 1);
    assert_eq!(fake.calls("poll").len(), 2);
    let media = fake.calls("media");
    let body = String::from_utf8_lossy(&media[0].body);
    assert!(body.contains("name=\"description\""));
    assert!(body.contains("Build with evidence."));
    assert!(body.contains("fixture-png"));
    let statuses = fake.calls("statuses");
    assert_eq!(statuses.len(), 1);
    assert_eq!(statuses[0].headers["idempotency-key"], "dispatch-fixture");
    assert_eq!(
        statuses[0].headers["authorization"],
        format!("Bearer {}", fake_mastodon::TOKEN)
    );
    let body: serde_json::Value = serde_json::from_slice(&statuses[0].body).unwrap();
    assert_eq!(
        body,
        serde_json::json!({"status":"Build with evidence.","media_ids":["media-1"],"visibility":"public"})
    );
}

#[tokio::test]
async fn limits_and_media_failures_prevent_status_creation() {
    for config in [
        Config {
            max_chars: 19,
            ..Config::default()
        },
        Config {
            max_image_bytes: 2,
            ..Config::default()
        },
        Config {
            media_status: 422,
            ..Config::default()
        },
    ] {
        let fake = fake_mastodon::start(config).await;
        let m = consent(&fake, MemoryStore::new()).await;
        let mut r = request();
        r.image = Some(platform::PostImage {
            bytes: vec![1, 2, 3],
            alt_text: "Alt".into(),
        });
        assert_eq!(m.publish(r).await, Err(PostFailure::Refused));
        assert!(fake.calls("statuses").is_empty());
    }
}

#[tokio::test]
async fn status_outcomes_are_truthful_without_hidden_retries() {
    for (status, expected) in [
        (401, PostFailure::ConsentRequired),
        (403, PostFailure::Refused),
        (422, PostFailure::Refused),
        (429, PostFailure::RateLimited),
        (500, PostFailure::Unknown),
    ] {
        let fake = fake_mastodon::start(Config {
            status,
            ..Config::default()
        })
        .await;
        let m = consent(&fake, MemoryStore::new()).await;
        assert_eq!(m.publish(request()).await, Err(expected));
        assert_eq!(fake.calls("statuses").len(), 1);
    }
    let fake = fake_mastodon::start(Config {
        no_id: true,
        ..Config::default()
    })
    .await;
    let m = consent(&fake, MemoryStore::new()).await;
    assert_eq!(m.publish(request()).await, Err(PostFailure::Unknown));
}

#[tokio::test]
async fn exact_instance_text_and_image_limits_are_accepted() {
    let fake = fake_mastodon::start(Config {
        max_chars: 20,
        max_image_bytes: 3,
        ..Default::default()
    })
    .await;
    let m = consent(&fake, MemoryStore::new()).await;
    let mut r = request();
    r.image = Some(platform::PostImage {
        bytes: vec![1, 2, 3],
        alt_text: "Alt".into(),
    });
    assert_eq!(m.publish(r).await.unwrap().post_id, "status-1");
    assert_eq!(fake.calls("statuses").len(), 1);
}

#[tokio::test]
async fn malformed_limits_media_id_and_status_url_fail_closed() {
    for config in [
        Config {
            oversized: true,
            ..Default::default()
        },
        Config {
            bad_limits: true,
            ..Default::default()
        },
        Config {
            media_id: "../escape".into(),
            ..Default::default()
        },
        Config {
            foreign_url: true,
            ..Default::default()
        },
    ] {
        let fake = fake_mastodon::start(config).await;
        let m = consent(&fake, MemoryStore::new()).await;
        let mut r = request();
        r.image = Some(platform::PostImage {
            bytes: vec![1, 2, 3],
            alt_text: "Alt".into(),
        });
        assert_eq!(m.publish(r).await, Err(PostFailure::Unknown));
        assert!(fake.calls("poll").is_empty());
    }
}

#[tokio::test]
async fn rejected_tokens_and_absent_callbacks_store_nothing_and_release_the_listener() {
    let fake = fake_mastodon::start(Config {
        token_status: 401,
        ..Default::default()
    })
    .await;
    let tokens = MemoryStore::new();
    let m = adapter(&fake, tokens.clone());
    let start = m.sign_in_begin().await.unwrap();
    support::browser(&start.authorization_url).await;
    assert_eq!(m.sign_in_wait().await, Err(StudioError::SignInRejected));
    assert!(tokens.get(&m.account()).is_none());
    let m = Mastodon::new(
        MastodonConfig::parse(
            Some(fake.base()),
            Some(&format!(
                "http://127.0.0.1:{}/mastodon/callback",
                free_port()
            )),
        )
        .unwrap(),
        tokens.clone(),
        fixed_clock(),
        Duration::from_millis(50),
    );
    m.sign_in_begin().await.unwrap();
    assert_eq!(m.sign_in_wait().await, Err(StudioError::SignInTimedOut));
    m.sign_in_begin().await.unwrap();
    assert!(tokens.get(&m.account()).is_none());
}

#[tokio::test]
async fn uncertain_status_is_never_retried_by_the_outbox() {
    let fake = fake_mastodon::start(Config {
        hang_status: true,
        ..Default::default()
    })
    .await;
    let m = consent(&fake, MemoryStore::new()).await;
    let dir = tempfile::tempdir().unwrap();
    let s = support::studio(
        &dir,
        support::scripted::Scripted::new(support::scripted::text("Build with evidence.")),
        support::fast(),
    )
    .with_delivery("mastodon", m);
    let p = s
        .create_thought(ThoughtRequest {
            text: "Build with evidence.".into(),
            channels: vec!["mastodon".into()],
        })
        .await
        .unwrap()
        .pieces
        .remove(0);
    let p = s
        .review(
            &p.piece_id,
            ReviewRequest {
                expected_revision: p.revision,
                status: Some(Status::Approved),
                text: None,
                expected_assets: None,
            },
        )
        .unwrap();
    let r = PublishRequest {
        expected_revision: p.revision,
        accepted_cost_microusd: None,
        dry_run: false,
    };
    let first = s.publish(&p.piece_id, r.clone()).await.unwrap();
    assert_eq!(first.status, "unknown");
    let again = s.publish(&p.piece_id, r).await.unwrap();
    assert_eq!(again.status, "unknown");
    assert_eq!(fake.calls("statuses").len(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_mastodon_publishes_make_only_one_uncertain_status_request() {
    let fake = fake_mastodon::start(Config {
        hang_status: true,
        ..Default::default()
    })
    .await;
    let m = consent(&fake, MemoryStore::new()).await;
    let dir = tempfile::tempdir().unwrap();
    let s = Arc::new(
        support::studio(
            &dir,
            support::scripted::Scripted::new(support::scripted::text("Build with evidence.")),
            support::fast(),
        )
        .with_delivery("mastodon", m),
    );
    let p = s
        .create_thought(ThoughtRequest {
            text: "Build with evidence.".into(),
            channels: vec!["mastodon".into()],
        })
        .await
        .unwrap()
        .pieces
        .remove(0);
    let p = s
        .review(
            &p.piece_id,
            ReviewRequest {
                expected_revision: p.revision,
                status: Some(Status::Approved),
                text: None,
                expected_assets: None,
            },
        )
        .unwrap();
    let barrier = Arc::new(tokio::sync::Barrier::new(16));
    let mut tasks = Vec::new();
    for _ in 0..16 {
        let s = s.clone();
        let b = barrier.clone();
        let p = p.clone();
        tasks.push(tokio::spawn(async move {
            b.wait().await;
            s.publish(
                &p.piece_id,
                PublishRequest {
                    expected_revision: p.revision,
                    accepted_cost_microusd: None,
                    dry_run: false,
                },
            )
            .await
            .unwrap()
        }));
    }
    for task in tasks {
        let response = task.await.unwrap();
        assert_eq!(response.status, "unknown");
        assert_eq!(response.attempts, 1);
    }
    assert_eq!(fake.calls("statuses").len(), 1);
}

#[tokio::test]
async fn pending_media_has_a_finite_poll_budget_and_never_creates_a_status() {
    let fake = fake_mastodon::start(Config {
        async_media: true,
        pending_forever: true,
        ..Default::default()
    })
    .await;
    let m = consent(&fake, MemoryStore::new()).await;
    let mut r = request();
    r.image = Some(platform::PostImage {
        bytes: vec![1, 2, 3],
        alt_text: "Alt".into(),
    });
    assert_eq!(m.publish(r).await, Err(PostFailure::Unknown));
    assert_eq!(fake.calls("poll").len(), 20);
    assert!(fake.calls("statuses").is_empty());
}

#[test]
fn instance_configuration_rejects_paths_credentials_and_non_loopback_http() {
    for value in [
        "http://example.test",
        "https://user:pass@example.test",
        "https://example.test/path",
        "https://example.test?secret=1",
    ] {
        assert!(MastodonConfig::parse(Some(value), None).is_err());
    }
    assert!(MastodonConfig::parse(Some("https://example.test"), None).is_ok());
}
