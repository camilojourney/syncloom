#[path = "support/fake_meta.rs"]
mod fake_meta;
#[path = "support/meta_fixture.rs"]
mod meta_fixture;
mod support;
use meta_fixture::*;
use std::sync::Arc;
use support::memory_store::{self, MemoryStore};
use syncloom_core::{
    PostFailure, PostRequest, TokenStore,
    meta::{MetaTarget, PublicMediaUrl},
};
fn request(target: MetaTarget, images: usize) -> PostRequest {
    PostRequest {
        approval_target: match target {
            MetaTarget::Facebook => {
                syncloom_core::platform::account_target("facebook", "", "101").ok()
            }
            MetaTarget::Instagram => {
                syncloom_core::platform::account_target("instagram", "101", "202").ok()
            }
        },
        request_id: "intent-1".into(),
        text: "Build with measured evidence.".into(),
        image: None,
        slides: (0..images)
            .map(|n| syncloom_core::platform::PostImage {
                bytes: vec![255, 216, 255, 217],
                alt_text: format!("Slide {n}"),
            })
            .collect(),
    }
}
#[tokio::test]
async fn consent_derives_only_selected_page_token_and_sign_out_deletes_it() {
    let fake = fake_meta::start(Default::default()).await;
    let tokens = MemoryStore::new();
    let meta = adapter(&fake, tokens.clone(), Arc::new(Media::default()));
    consent(&meta).await;
    let token = tokens.get("meta:101").unwrap();
    assert_eq!(token.access_token.expose(), fake_meta::TOKEN);
    assert_eq!(token.expires_at, None);
    assert_eq!(token.member_id, "202");
    assert_eq!(fake.calls("oauth/access_token").len(), 2);
    let before = fake.all().len();
    assert_eq!(
        meta.connection().unwrap().status,
        syncloom_core::ConnectionStatus::SignedIn
    );
    assert_eq!(fake.all().len(), before);
    meta.sign_out().unwrap();
    meta.sign_out().unwrap();
    assert!(tokens.get("meta:101").is_none());
}
#[tokio::test]
async fn denied_state_permission_page_and_task_guards_preserve_existing_token() {
    for c in [
        fake_meta::Config {
            wrong_state: true,
            ..Default::default()
        },
        fake_meta::Config {
            deny: true,
            ..Default::default()
        },
        fake_meta::Config {
            missing_permission: true,
            ..Default::default()
        },
        fake_meta::Config {
            wrong_page: true,
            ..Default::default()
        },
        fake_meta::Config {
            no_tasks: true,
            ..Default::default()
        },
    ] {
        let fake = fake_meta::start(c.clone()).await;
        let mut old = stored();
        old.access_token = syncloom_core::Secret::new("old-synthetic-token");
        let tokens = MemoryStore::with("meta:101", old.clone());
        let meta = adapter(&fake, tokens.clone(), Arc::new(Media::default()));
        let start = meta.sign_in_begin().unwrap();
        reqwest::Client::new()
            .get(start.authorization_url)
            .send()
            .await
            .unwrap();
        assert!(meta.sign_in_wait().await.is_err());
        assert_eq!(tokens.get("meta:101"), Some(old));
        if c.wrong_state || c.deny {
            assert!(fake.all().is_empty());
        }
    }
}
#[tokio::test]
async fn instagram_checks_ready_status_and_never_publishes_failed_or_expired_containers() {
    for status in ["ERROR", "EXPIRED", "IN_PROGRESS", "PUBLISHED", "INVALID"] {
        let fake = fake_meta::start(fake_meta::Config {
            container_status: Some(status.into()),
            ..Default::default()
        })
        .await;
        let meta = adapter(
            &fake,
            MemoryStore::with("meta:101", stored()),
            Arc::new(Media::default()),
        );
        assert!(
            meta.target(MetaTarget::Instagram)
                .publish(request(MetaTarget::Instagram, 1))
                .await
                .is_err()
        );
        assert!(fake.calls("202/media_publish").is_empty());
        assert!(fake.all().len() <= 7);
        if status == "IN_PROGRESS" {
            assert_eq!(fake.calls("600").len(), 6);
        }
    }
}
#[tokio::test]
async fn processing_is_polled_once_per_minute_then_published_when_ready() {
    let fake = fake_meta::start(fake_meta::Config {
        ready_after: 1,
        ..Default::default()
    })
    .await;
    let meta = adapter(
        &fake,
        MemoryStore::with("meta:101", stored()),
        Arc::new(Media::default()),
    );
    let start = std::time::Instant::now();
    assert!(
        meta.target(MetaTarget::Instagram)
            .publish(request(MetaTarget::Instagram, 1))
            .await
            .is_ok()
    );
    assert!(start.elapsed() >= std::time::Duration::from_secs(60));
    assert_eq!(fake.calls("600").len(), 2);
    assert_eq!(fake.calls("202/media_publish").len(), 1);
}
#[tokio::test]
async fn carousel_orders_children_then_parent_and_ai_disclosure_never_on_children() {
    let fake = fake_meta::start(Default::default()).await;
    let media = Arc::new(Media::default());
    let meta = adapter(
        &fake,
        MemoryStore::with("meta:101", stored()),
        media.clone(),
    );
    let posted = meta
        .target(MetaTarget::Instagram)
        .publish(request(MetaTarget::Instagram, 3))
        .await
        .unwrap();
    assert_eq!(posted.post_id, "303");
    assert_eq!(posted.url, "https://www.instagram.com/p/fake303/");
    let containers = fake.calls("202/media");
    assert_eq!(containers.len(), 4);
    for (n, c) in containers[..3].iter().enumerate() {
        assert_eq!(c.body["is_carousel_item"], true);
        assert_eq!(c.body["alt_text"], format!("Slide {n}"));
        assert!(c.body.get("is_ai_generated").is_none());
    }
    assert_eq!(containers[3].body["children"], "600,601,602");
    assert_eq!(containers[3].body["is_ai_generated"], true);
    assert_eq!(fake.calls("202/media_publish").len(), 1);
    assert_eq!(media.images.lock().unwrap().len(), 3);
}
#[tokio::test]
async fn graph_error_codes_and_http_statuses_are_truthful_without_hidden_retries() {
    for (http, code, expected) in [
        (400, 190, PostFailure::ConsentRequired),
        (400, 10, PostFailure::Refused),
        (400, 4, PostFailure::RateLimited),
        (401, 0, PostFailure::ConsentRequired),
        (403, 0, PostFailure::Refused),
        (429, 0, PostFailure::RateLimited),
        (500, 0, PostFailure::Unknown),
    ] {
        let fake = fake_meta::start(fake_meta::Config {
            fault_path: Some("101/feed".into()),
            fault_status: http,
            fault_code: code,
            ..Default::default()
        })
        .await;
        let meta = adapter(
            &fake,
            MemoryStore::with("meta:101", stored()),
            Arc::new(Media::default()),
        );
        assert_eq!(
            meta.target(MetaTarget::Facebook)
                .publish(request(MetaTarget::Facebook, 0))
                .await,
            Err(expected)
        );
        assert_eq!(fake.calls("101/feed").len(), 1);
    }
}
#[tokio::test]
async fn absent_expired_unlinked_or_missing_media_sends_nothing() {
    let fake = fake_meta::start(Default::default()).await;
    let tokens = MemoryStore::new();
    let meta = adapter(&fake, tokens.clone(), Arc::new(Media::default()));
    assert_eq!(
        meta.target(MetaTarget::Facebook)
            .publish(request(MetaTarget::Facebook, 0))
            .await,
        Err(PostFailure::ConsentRequired)
    );
    let mut token = stored();
    token.expires_at = Some("2026-10-31T00:00:00Z".into());
    tokens.save("meta:101", &token).unwrap();
    assert_eq!(
        meta.target(MetaTarget::Facebook)
            .publish(request(MetaTarget::Facebook, 0))
            .await,
        Err(PostFailure::ConsentRequired)
    );
    token = stored();
    token.member_id.clear();
    tokens.save("meta:101", &token).unwrap();
    assert_eq!(
        meta.target(MetaTarget::Instagram)
            .publish(request(MetaTarget::Instagram, 1))
            .await,
        Err(PostFailure::ConsentRequired)
    );
    tokens.save("meta:101", &stored()).unwrap();
    assert_eq!(
        meta.target(MetaTarget::Instagram)
            .publish(request(MetaTarget::Instagram, 0))
            .await,
        Err(PostFailure::Refused)
    );
    assert_eq!(
        meta.target(MetaTarget::Instagram)
            .publish(request(MetaTarget::Instagram, 11))
            .await,
        Err(PostFailure::Refused)
    );
    assert!(fake.all().is_empty());
}
#[tokio::test]
async fn missing_host_and_failed_child_prevent_status_creation() {
    let fake = fake_meta::start(Default::default()).await;
    let media = Arc::new(Media {
        refused: true,
        ..Default::default()
    });
    let meta = adapter(&fake, MemoryStore::with("meta:101", stored()), media);
    assert_eq!(
        meta.target(MetaTarget::Instagram)
            .publish(request(MetaTarget::Instagram, 1))
            .await,
        Err(PostFailure::Refused)
    );
    assert!(fake.all().is_empty());
    fake.configure(fake_meta::Config {
        fault_path: Some("202/media".into()),
        fault_status: 400,
        fault_code: 100,
        fault_after: 1,
        ..Default::default()
    });
    let meta = adapter(
        &fake,
        MemoryStore::with("meta:101", stored()),
        Arc::new(Media::default()),
    );
    assert_eq!(
        meta.target(MetaTarget::Instagram)
            .publish(request(MetaTarget::Instagram, 2))
            .await,
        Err(PostFailure::Refused)
    );
    assert!(fake.calls("202/media_publish").is_empty());
}
#[test]
fn meta_configuration_rejects_path_injection_and_external_callback_hosts() {
    for (variable, value) in [
        ("SYNCLOOM_META_PAGE_ID", "101/evil"),
        ("SYNCLOOM_META_PAGE_ID", "1".repeat(129).as_str()),
        (
            "SYNCLOOM_META_REDIRECT_URL",
            "https://external.invalid/meta/callback",
        ),
        (
            "SYNCLOOM_META_REDIRECT_URL",
            "http://127.0.0.1/meta/callback",
        ),
    ] {
        let result = syncloom_core::meta::MetaConfig::parse(|name| {
            (name == variable).then(|| value.to_owned())
        });
        assert!(result.unwrap_err().contains(variable));
    }
}

#[tokio::test]
async fn jpeg_byte_bound_format_and_alt_text_are_checked_before_any_media_delivery() {
    let fake = fake_meta::start(Default::default()).await;
    let media = Arc::new(Media::default());
    let meta = adapter(
        &fake,
        MemoryStore::with("meta:101", stored()),
        media.clone(),
    );
    let mut invalid_format = request(MetaTarget::Instagram, 1);
    invalid_format.slides[0].bytes = b"not jpeg".to_vec();
    let mut empty_alt = request(MetaTarget::Instagram, 1);
    empty_alt.slides[0].alt_text = "  ".into();
    let mut oversized = request(MetaTarget::Instagram, 1);
    oversized.slides[0].bytes = vec![255; 8 * 1024 * 1024 + 1];
    oversized.slides[0].bytes[..2].copy_from_slice(&[255, 216]);
    for r in [invalid_format, empty_alt, oversized] {
        assert_eq!(
            meta.target(MetaTarget::Instagram).publish(r).await,
            Err(PostFailure::Refused)
        );
    }
    assert!(fake.all().is_empty());
    assert!(media.images.lock().unwrap().is_empty());
    let mut exact = request(MetaTarget::Instagram, 1);
    exact.slides[0].bytes = vec![255; 8 * 1024 * 1024];
    exact.slides[0].bytes[..2].copy_from_slice(&[255, 216]);
    assert!(
        meta.target(MetaTarget::Instagram)
            .publish(exact)
            .await
            .is_ok()
    );
    assert_eq!(fake.calls("202/media_publish").len(), 1);
}

#[test]
fn meta_text_bounds_are_inclusive_and_never_truncate() {
    for (channel, limit) in [
        (syncloom_core::Channel::FacebookPage, 63206),
        (syncloom_core::Channel::Instagram, 2200),
    ] {
        assert_eq!(
            syncloom_core::fit(channel, "").verdict,
            syncloom_core::Verdict::Review
        );
        assert_eq!(
            syncloom_core::fit(channel, "x").verdict,
            syncloom_core::Verdict::Pass
        );
        assert_eq!(
            syncloom_core::fit(channel, &"x".repeat(limit)).verdict,
            syncloom_core::Verdict::Pass
        );
        let exceeded = syncloom_core::fit(channel, &"x".repeat(limit + 1));
        assert_eq!(exceeded.verdict, syncloom_core::Verdict::Review);
        assert_eq!(exceeded.char_count, limit + 1);
    }
}

#[test]
fn public_media_url_refuses_local_and_credential_urls() {
    for url in [
        "http://media.invalid/a.jpg",
        "https://127.0.0.1/a",
        "https://[::1]/a",
        "https://localhost/a",
        "https://host.local/a",
        "https://u:p@media.invalid/a",
        "https://u@media.invalid/a",
        "https://media.invalid/a#fragment",
    ] {
        assert!(PublicMediaUrl::parse(url).is_none(), "{url}");
    }
    assert!(PublicMediaUrl::parse("https://media.invalid/a?signature=synthetic").is_some());
}

#[tokio::test]
async fn empty_http_errors_keep_auth_refusal_and_rate_limit_meaning() {
    for (status, expected) in [
        (401, PostFailure::ConsentRequired),
        (403, PostFailure::Refused),
        (429, PostFailure::RateLimited),
        (500, PostFailure::Unknown),
    ] {
        let fake = fake_meta::start(fake_meta::Config {
            fault_path: Some("101/feed".into()),
            fault_status: status,
            empty_body: true,
            ..Default::default()
        })
        .await;
        let meta = adapter(
            &fake,
            MemoryStore::with("meta:101", stored()),
            Arc::new(Media::default()),
        );
        assert_eq!(
            meta.target(MetaTarget::Facebook)
                .publish(request(MetaTarget::Facebook, 0))
                .await,
            Err(expected)
        );
        assert_eq!(fake.calls("101/feed").len(), 1);
    }
}

#[tokio::test]
async fn photo_id_alone_and_malformed_publish_or_foreign_permalink_never_claim_posted() {
    let fake = fake_meta::start(fake_meta::Config {
        photo_only: true,
        ..Default::default()
    })
    .await;
    let meta = adapter(
        &fake,
        MemoryStore::with("meta:101", stored()),
        Arc::new(Media::default()),
    );
    let mut photo = request(MetaTarget::Facebook, 0);
    photo.image = Some(syncloom_core::platform::PostImage {
        bytes: b"synthetic-png".to_vec(),
        alt_text: "Evidence".into(),
    });
    assert_eq!(
        meta.target(MetaTarget::Facebook).publish(photo).await,
        Err(PostFailure::Unknown)
    );
    fake.configure(fake_meta::Config {
        fault_path: Some("202/media_publish".into()),
        malformed: true,
        ..Default::default()
    });
    assert_eq!(
        meta.target(MetaTarget::Instagram)
            .publish(request(MetaTarget::Instagram, 1))
            .await,
        Err(PostFailure::Unknown)
    );
    fake.configure(fake_meta::Config {
        bad_permalink: true,
        ..Default::default()
    });
    assert_eq!(
        meta.target(MetaTarget::Instagram)
            .publish(request(MetaTarget::Instagram, 1))
            .await,
        Err(PostFailure::Unknown)
    );
}

#[tokio::test]
async fn sign_in_callback_deadline_releases_port_and_unconfigured_host_refuses() {
    use syncloom_core::meta::{Meta, UnconfiguredMedia};
    let fake = fake_meta::start(Default::default()).await;
    let conf = config(&fake);
    let address = conf.redirect.bind_addr();
    let meta = Arc::new(Meta::new(
        conf,
        MemoryStore::with("meta:101", stored()),
        clock(),
        std::time::Duration::from_millis(20),
        Arc::new(UnconfiguredMedia),
    ));
    meta.sign_in_begin().unwrap();
    assert_eq!(
        meta.sign_in_wait().await,
        Err(syncloom_core::StudioError::SignInTimedOut)
    );
    assert!(std::net::TcpListener::bind(address).is_ok());
    assert_eq!(
        meta.target(MetaTarget::Instagram)
            .publish(request(MetaTarget::Instagram, 1))
            .await,
        Err(PostFailure::Refused)
    );
    assert!(fake.all().is_empty());
}

#[tokio::test]
async fn saved_meta_scope_without_posting_permissions_is_not_publish_ready() {
    let fake = fake_meta::start(Default::default()).await;
    let tokens = MemoryStore::new();
    let meta = adapter(&fake, tokens.clone(), Arc::new(Media::default()));
    consent(&meta).await;
    let mut token = tokens.get("meta:101").unwrap();
    token.scope = "pages_show_list pages_read_engagement".into();
    tokens.save("meta:101", &token).unwrap();
    let connection = serde_json::to_value(meta.connection().unwrap()).unwrap();
    let before = fake.all().len();
    let published = meta
        .target(MetaTarget::Facebook)
        .publish(request(MetaTarget::Facebook, 0))
        .await;
    assert_eq!(connection["status"], "cannot_post", "{connection}");
    assert_eq!(connection["reason"], "posting_permission_missing");
    assert_eq!(connection["action"], "sign_in");
    assert_eq!(published, Err(PostFailure::PostingPermissionMissing));
    assert_eq!(fake.all().len(), before);
}

#[tokio::test]
async fn meta_effective_grants_authorize_each_target_independently() {
    use syncloom_core::{Channel, ConnectionStatus};
    for facebook in [true, false] {
        let omitted = if facebook {
            "instagram_content_publish"
        } else {
            "pages_manage_posts"
        };
        let scopes: Vec<String> = fake_meta::SCOPES
            .iter()
            .filter(|scope| **scope != omitted)
            .map(|s| s.to_string())
            .collect();
        let fake = fake_meta::start(fake_meta::Config {
            granted_permissions: Some(scopes.clone()),
            ..Default::default()
        })
        .await;
        let tokens = MemoryStore::new();
        let meta = adapter(&fake, tokens.clone(), Arc::new(Media::default()));
        let start = meta.sign_in_begin().unwrap();
        reqwest::get(start.authorization_url).await.unwrap();
        assert_eq!(
            meta.sign_in_wait().await.unwrap().status,
            ConnectionStatus::SignedIn
        );
        let saved = tokens.get("meta:101").unwrap();
        assert!(saved.scope_verified);
        assert_eq!(saved.scope, scopes.join(" "));
        let (allowed, denied, channel, denied_channel) = if facebook {
            (
                MetaTarget::Facebook,
                MetaTarget::Instagram,
                Channel::FacebookPage,
                Channel::Instagram,
            )
        } else {
            (
                MetaTarget::Instagram,
                MetaTarget::Facebook,
                Channel::Instagram,
                Channel::FacebookPage,
            )
        };
        assert_eq!(
            meta.connection_for(channel, !facebook).unwrap().status,
            ConnectionStatus::SignedIn
        );
        assert_eq!(
            meta.connection_for(denied_channel, facebook)
                .unwrap()
                .status,
            ConnectionStatus::CannotPost
        );
        let before = fake.all().len();
        assert_eq!(
            meta.target(denied)
                .publish(request(denied, usize::from(facebook)))
                .await,
            Err(PostFailure::PostingPermissionMissing)
        );
        assert_eq!(fake.all().len(), before);
        assert!(
            meta.target(allowed)
                .publish(request(allowed, usize::from(!facebook)))
                .await
                .is_ok()
        );
    }
}

#[tokio::test]
async fn meta_granted_permissions_are_exact_scope_tokens() {
    for malformed in ["pages_manage_posts extra", "pages_manage_posts,extra", ""] {
        let fake = fake_meta::start(fake_meta::Config {
            granted_permissions: Some(vec!["pages_show_list".into(), malformed.into()]),
            ..Default::default()
        })
        .await;
        let tokens = MemoryStore::new();
        let meta = adapter(&fake, tokens.clone(), Arc::new(Media::default()));
        let start = meta.sign_in_begin().unwrap();
        reqwest::get(start.authorization_url).await.unwrap();
        assert_eq!(
            meta.sign_in_wait().await,
            Err(syncloom_core::StudioError::PostingPermissionUnverified)
        );
        assert!(tokens.get("meta:101").is_none());
        assert!(
            !fake
                .all()
                .iter()
                .any(|call| call.path.ends_with("/me/accounts"))
        );
    }
}

#[tokio::test]
async fn legacy_meta_requested_scope_without_evidence_is_unverified_for_both_targets() {
    use syncloom_core::{ConnectionStatus, Token};
    let fake = fake_meta::start(Default::default()).await;
    let mut shape = serde_json::to_value(stored()).unwrap();
    shape.as_object_mut().unwrap().remove("scope_verified");
    let legacy: Token = serde_json::from_value(shape).unwrap();
    let meta = adapter(
        &fake,
        MemoryStore::with("meta:101", legacy),
        Arc::new(Media::default()),
    );
    assert_eq!(
        meta.connection().unwrap().status,
        ConnectionStatus::CannotPost
    );
    assert_eq!(
        meta.connection().unwrap().reason.as_deref(),
        Some("posting_permission_unverified")
    );
    for (target, slides) in [(MetaTarget::Facebook, 0), (MetaTarget::Instagram, 1)] {
        assert_eq!(
            meta.target(target).publish(request(target, slides)).await,
            Err(PostFailure::PostingPermissionUnverified)
        );
    }
    assert!(fake.all().is_empty());
}
