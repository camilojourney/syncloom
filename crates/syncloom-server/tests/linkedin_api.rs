//! The LinkedIn operations over HTTP (studio-api "Operations"; linkedin-integration and
//! platform-credentials through the API; dispatch-outbox "Publish operation" with a grant).
mod support;
use serde_json::{Value, json};
use support::fake_linkedin::{AuthMode, PostMode, TokenMode};
use support::memory_store::MemoryStore;
use support::scripted::{Scripted, text};
use support::*;
use syncloom_core::TokenStore;

#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
async fn browser(url: &str) -> reqwest::Response {
    client().get(url).send().await.unwrap()
}

// linkedin-integration: Sign-in begins, Sign-in completes (Consent), Connection operations
#[tokio::test]
async fn sign_in_answers_the_url_then_the_connection_and_the_connection_can_be_read_and_deleted() {
    let fake = fake_linkedin::start(AuthMode::Consent, TokenMode::ok(), PostMode::NoId).await;
    let store = MemoryStore::new();
    let server = start_linkedin(
        Scripted::new(text("x")),
        fast(),
        &fake,
        store.clone(),
        true,
        false,
    )
    .await;

    let before = get(&server, "/v1/linkedin/connection").await;
    assert_eq!(before.status, 200, "{}", before.raw);
    assert_eq!(before.body, json!({"status": "signed_out"}));

    let started = post_empty(&server, "/v1/linkedin/sign-in").await;
    assert_eq!(started.status, 202, "{}", started.raw);
    assert_eq!(started.body["status"], "awaiting_consent");
    assert_eq!(started.body["expires_at"], "2026-10-30T18:00:00Z");
    let url = started.body["authorization_url"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(url.starts_with(&fake.base_url()), "{url}");
    let keys: Vec<&str> = started
        .body
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(keys, ["authorization_url", "expires_at", "status"]);

    let during = get(&server, "/v1/linkedin/connection").await;
    assert_eq!(during.body["status"], "awaiting_consent");

    let page = browser(&url).await;
    assert_eq!(page.status(), 200);
    let finished = get(&server, "/v1/linkedin/sign-in").await;
    assert_eq!(finished.status, 200, "{}", finished.raw);
    assert_eq!(
        finished.body,
        json!({"status": "signed_in", "expires_at": "2026-12-29T18:00:00Z", "scope": "openid,profile,w_member_social"})
    );
    assert!(
        !finished.raw.contains("fake-access-token"),
        "{}",
        finished.raw
    );
    assert!(store.get("linkedin").is_some());

    let read = get(&server, "/v1/linkedin/connection").await;
    assert_eq!(read.body, finished.body);
    let deleted = send_json(
        &server,
        reqwest::Method::DELETE,
        "/v1/linkedin/connection",
        Value::Null,
    )
    .await;
    assert_eq!(deleted.status, 200, "{}", deleted.raw);
    assert_eq!(deleted.body, json!({"status": "signed_out"}));
    assert!(store.get("linkedin").is_none());
    assert_eq!(
        get(&server, "/v1/linkedin/connection").await.body,
        json!({"status": "signed_out"})
    );
    assert_eq!(fake.posts().len(), 0);
}

#[tokio::test]
async fn linkedin_account_switch_requires_fresh_approval() {
    let fake = fake_linkedin::start(
        AuthMode::Consent,
        TokenMode::ok(),
        PostMode::Posted(fake_linkedin::DEFAULT_POST_ID.into()),
    )
    .await;
    let server = start_linkedin(
        Scripted::new(text("Build with evidence.")),
        fast(),
        &fake,
        MemoryStore::new(),
        true,
        true,
    )
    .await;
    let start = post_empty(&server, "/v1/linkedin/sign-in").await;
    browser(start.body["authorization_url"].as_str().unwrap()).await;
    assert_eq!(get(&server, "/v1/linkedin/sign-in").await.status, 200);
    let created = create(&server, THOUGHT).await;
    let p = &created.body["pieces"][0];
    assert_eq!(approve(&server, p).await.status, 200);
    send_json(
        &server,
        reqwest::Method::DELETE,
        "/v1/linkedin/connection",
        Value::Null,
    )
    .await;
    fake.set_member("SYNTHETICSECOND");
    let start = post_empty(&server, "/v1/linkedin/sign-in").await;
    browser(start.body["authorization_url"].as_str().unwrap()).await;
    assert_eq!(get(&server, "/v1/linkedin/sign-in").await.status, 200);
    let path = format!("/v1/pieces/{}/publish", p["piece_id"].as_str().unwrap());
    let result = post(&server, &path, json!({"expected_revision":p["revision"]})).await;
    let targets: Vec<_> = fake
        .posts()
        .iter()
        .map(|p| p.body["author"].clone())
        .collect();
    assert_eq!(
        result.code(),
        "account_approval_required",
        "actual authors={targets:?}; {}",
        result.raw
    );
    assert!(targets.is_empty());
    assert_eq!(approve(&server, p).await.status, 200);
    assert_eq!(
        post(&server, &path, json!({"expected_revision":p["revision"]}))
            .await
            .body["status"],
        "posted"
    );
    assert_eq!(
        fake.posts()[0].body["author"],
        "urn:li:person:SYNTHETICSECOND"
    );
}

// linkedin-integration: Not configured, Already waiting, Nothing pending, Denied, Wrong state,
// Token refused, No callback; studio-api: Operations and their codes
#[tokio::test]
async fn each_sign_in_failure_has_its_status_and_code() {
    let fake = fake_linkedin::start(AuthMode::Consent, TokenMode::ok(), PostMode::NoId).await;

    let unconfigured = start_linkedin(
        Scripted::new(text("x")),
        fast(),
        &fake,
        MemoryStore::new(),
        false,
        false,
    )
    .await;
    let answer = post_empty(&unconfigured, "/v1/linkedin/sign-in").await;
    assert_eq!(
        (answer.status, answer.code()),
        (503, "linkedin_not_configured"),
        "{}",
        answer.raw
    );
    assert_eq!(
        answer.body["error"]["message"],
        fixture("errors/linkedin_not_configured.json")["body"]["error"]["message"]
    );
    let answer = get(&unconfigured, "/v1/linkedin/sign-in").await;
    assert_eq!((answer.status, answer.code()), (409, "sign_in_not_started"));

    let server = start_linkedin(
        Scripted::new(text("x")),
        fast(),
        &fake,
        MemoryStore::new(),
        true,
        false,
    )
    .await;
    let answer = get(&server, "/v1/linkedin/sign-in").await;
    assert_eq!((answer.status, answer.code()), (409, "sign_in_not_started"));
    let started = post_empty(&server, "/v1/linkedin/sign-in").await;
    assert_eq!(started.status, 202);
    let answer = post_empty(&server, "/v1/linkedin/sign-in").await;
    assert_eq!((answer.status, answer.code()), (409, "sign_in_in_progress"));
    // No browser: the wait times out under the fast deadline.
    let answer = get(&server, "/v1/linkedin/sign-in").await;
    assert_eq!(
        (answer.status, answer.code()),
        (504, "sign_in_timed_out"),
        "{}",
        answer.raw
    );

    fake.set_auth(AuthMode::Deny);
    let started = post_empty(&server, "/v1/linkedin/sign-in").await;
    assert_eq!(
        started.status, 202,
        "a new sign-in after a timeout: {}",
        started.raw
    );
    browser(started.body["authorization_url"].as_str().unwrap()).await;
    let answer = get(&server, "/v1/linkedin/sign-in").await;
    assert_eq!(
        (answer.status, answer.code()),
        (403, "sign_in_denied"),
        "{}",
        answer.raw
    );
    assert!(!answer.raw.contains("zebrafish"));

    fake.set_auth(AuthMode::WrongState);
    let started = post_empty(&server, "/v1/linkedin/sign-in").await;
    browser(started.body["authorization_url"].as_str().unwrap()).await;
    let answer = get(&server, "/v1/linkedin/sign-in").await;
    assert_eq!(
        (answer.status, answer.code()),
        (403, "sign_in_state_mismatch")
    );
    assert!(fake.token_requests().is_empty(), "nothing exchanged so far");

    fake.set_auth(AuthMode::Consent);
    fake.set_token(TokenMode::Status(401));
    let started = post_empty(&server, "/v1/linkedin/sign-in").await;
    browser(started.body["authorization_url"].as_str().unwrap()).await;
    let answer = get(&server, "/v1/linkedin/sign-in").await;
    assert_eq!((answer.status, answer.code()), (502, "sign_in_rejected"));
    assert_eq!(fake.token_requests().len(), 1);
    assert_eq!(
        get(&server, "/v1/linkedin/connection").await.body["status"],
        "signed_out"
    );
}

// dispatch-outbox: Publish operation (Response shape with a grant), Repeat after posted
#[tokio::test]
async fn a_granted_publish_answers_posted_with_the_platform_id_and_repeats_without_a_request() {
    let fake = fake_linkedin::start(
        AuthMode::Consent,
        TokenMode::ok(),
        PostMode::Posted("urn:li:share:7000000000000000001".to_owned()),
    )
    .await;
    let store = MemoryStore::with("linkedin", stored_token("2026-12-29T18:00:00Z"));
    let server = start_linkedin(
        Scripted::new(text("A draft about zebrafish.")),
        fast(),
        &fake,
        store,
        false,
        true,
    )
    .await;
    let piece = piece_of(&create(&server, THOUGHT).await);
    approve(&server, &piece).await;
    let path = format!("/v1/pieces/{}/publish", id_of(&piece));
    let first = post(
        &server,
        &path,
        json!({"expected_revision": revision_of(&piece)}),
    )
    .await;
    assert_eq!(first.status, 200, "{}", first.raw);
    assert_eq!(first.body["status"], "posted");
    assert_eq!(first.body["created"], true);
    assert_eq!(first.body["post_id"], "urn:li:share:7000000000000000001");
    assert_eq!(
        first.body["post_url"],
        "https://www.linkedin.com/feed/update/urn:li:share:7000000000000000001/"
    );
    assert_eq!(first.body["attempts"], 1);
    assert_eq!(first.body["piece"]["publish_status"], "posted");
    let keys: Vec<&str> = first
        .body
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        keys,
        [
            "approval_target",
            "attempts",
            "created",
            "payload",
            "piece",
            "post_id",
            "post_url",
            "request_id",
            "status"
        ]
    );
    assert_eq!(fake.posts().len(), 1);
    assert_eq!(
        fake.posts()[0].body["commentary"],
        "A draft about zebrafish."
    );

    let second = post(
        &server,
        &path,
        json!({"expected_revision": revision_of(&piece)}),
    )
    .await;
    assert_eq!(second.body["status"], "posted");
    assert_eq!(second.body["created"], false);
    assert_eq!(second.body["post_id"], first.body["post_id"]);
    assert_eq!(fake.posts().len(), 1, "a posted intent is never sent again");

    let shown = get(&server, &format!("/v1/pieces/{}", id_of(&piece))).await;
    assert_eq!(shown.body["publish_status"], "posted");
    let kinds: Vec<String> = events(&server.dir)
        .iter()
        .map(|e| e["event"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(
        kinds,
        [
            "piece_created",
            "piece_reviewed",
            "intent_reserved",
            "dispatch_attempted",
            "dispatch_finished"
        ]
    );
}

// dispatch-outbox: Outcome recorded truthfully (Failure) through the API; platform-credentials: Expired token
#[tokio::test]
async fn a_granted_publish_reports_failures_as_data_not_as_errors() {
    let fake =
        fake_linkedin::start(AuthMode::Consent, TokenMode::ok(), PostMode::Status(429)).await;
    let store = MemoryStore::with("linkedin", stored_token("2026-12-29T18:00:00Z"));
    let server = start_linkedin(
        Scripted::new(text("A draft.")),
        fast(),
        &fake,
        store.clone(),
        false,
        true,
    )
    .await;
    let piece = piece_of(&create(&server, THOUGHT).await);
    approve(&server, &piece).await;
    let path = format!("/v1/pieces/{}/publish", id_of(&piece));
    let answer = post(
        &server,
        &path,
        json!({"expected_revision": revision_of(&piece)}),
    )
    .await;
    assert_eq!(answer.status, 200, "{}", answer.raw);
    assert_eq!(answer.body["status"], "rate_limited");
    assert_eq!(answer.body["post_id"], Value::Null);
    assert_eq!(answer.body["post_url"], Value::Null);
    assert_eq!(answer.body["attempts"], 1);

    fake.set_post(PostMode::Status(401));
    let answer = post(
        &server,
        &path,
        json!({"expected_revision": revision_of(&piece)}),
    )
    .await;
    assert_eq!(answer.body["status"], "consent_required");
    assert_eq!(answer.body["attempts"], 2);
    assert_eq!(fake.posts().len(), 2);

    // An expired token sends nothing.
    store
        .save("linkedin", &stored_token("2026-10-30T18:00:00Z"))
        .unwrap();
    let answer = post(
        &server,
        &path,
        json!({"expected_revision": revision_of(&piece)}),
    )
    .await;
    assert_eq!(answer.body["status"], "consent_required");
    assert_eq!(answer.body["attempts"], 3);
    assert_eq!(fake.posts().len(), 2, "an expired token must not be sent");
    assert_eq!(
        get(&server, "/v1/linkedin/connection").await.body["status"],
        "expired"
    );
}

// studio-api: Host check and Wrong method on the new paths
#[tokio::test]
async fn the_linkedin_paths_are_host_guarded_and_refuse_other_methods() {
    let fake = fake_linkedin::start(AuthMode::Consent, TokenMode::ok(), PostMode::NoId).await;
    let server = start_linkedin(
        Scripted::new(text("x")),
        fast(),
        &fake,
        MemoryStore::new(),
        true,
        false,
    )
    .await;
    for path in [
        "POST /v1/linkedin/sign-in HTTP/1.1",
        "GET /v1/linkedin/connection HTTP/1.1",
    ] {
        let answer = raw(
            server.addr,
            &head_with_hosts(path, &["Host: evil.example"], &[]),
        )
        .await;
        assert_eq!(
            (answer.status, answer.code()),
            (403, "host_not_allowed"),
            "{path}"
        );
    }
    let answer = patch(&server, "/v1/linkedin/connection", json!({})).await;
    assert_eq!((answer.status, answer.code()), (405, "method_not_allowed"));
    let answer = send_json(
        &server,
        reqwest::Method::DELETE,
        "/v1/linkedin/sign-in",
        Value::Null,
    )
    .await;
    assert_eq!((answer.status, answer.code()), (405, "method_not_allowed"));
    assert!(fake.auth_requests().is_empty());
}

/// Smallest counterfactual against `TokenMode::ok()`: the same consent fixture, with only the
/// returned scope reduced to `openid,profile` (no `w_member_social`).
#[tokio::test]
async fn reduced_linkedin_grant_is_not_connected_and_cannot_publish() {
    let fake = fake_linkedin::start(
        AuthMode::Consent,
        TokenMode::Ok {
            access_token: fake_linkedin::DEFAULT_TOKEN.into(),
            expires_in: 3600,
            scope: "openid,profile".into(),
        },
        PostMode::Posted("urn:li:share:7000000000000000001".into()),
    )
    .await;
    let store = MemoryStore::new();
    let server = start_linkedin(
        Scripted::new(text("A draft about zebrafish.")),
        fast(),
        &fake,
        store.clone(),
        true,
        true,
    )
    .await;
    let start = post_empty(&server, "/v1/linkedin/sign-in").await;
    assert_eq!(start.status, 202, "{}", start.raw);
    browser(start.body["authorization_url"].as_str().unwrap()).await;
    let completed = get(&server, "/v1/linkedin/sign-in").await;
    let saved_partial = store
        .get("linkedin")
        .is_some_and(|token| token.scope == "openid,profile");
    assert_eq!(
        (completed.status, completed.code()),
        (403, "posting_permission_missing"),
        "saved_partial={saved_partial} token_requests={} posts={} userinfo={} body={}",
        fake.token_requests().len(),
        fake.posts().len(),
        fake.userinfo_bearers().len(),
        completed.raw
    );
    assert!(
        completed.body["error"]["message"]
            .as_str()
            .unwrap_or("")
            .contains("posting permission missing"),
        "{}",
        completed.raw
    );
    assert!(!saved_partial, "a reduced grant must not be stored");
    assert!(fake.posts().is_empty());
    assert!(
        fake.userinfo_bearers().is_empty(),
        "a refused grant must not be used"
    );
    assert_eq!(
        get(&server, "/v1/linkedin/connection").await.body["status"],
        "signed_out"
    );

    let mut reduced = stored_token("2026-12-29T18:00:00Z");
    reduced.scope = "openid,profile".into();
    store.save("linkedin", &reduced).unwrap();
    let connection = get(&server, "/v1/linkedin/connection").await;
    assert_eq!(connection.status, 200, "{}", connection.raw);
    assert_ne!(connection.body["status"], "signed_in", "{}", connection.raw);
    assert_eq!(connection.body["status"], "cannot_post");
    assert_eq!(connection.body["reason"], "posting_permission_missing");
    assert_eq!(
        connection.body["message"],
        "connected, cannot post: posting permission missing."
    );
    assert_eq!(connection.body["action"], "sign_in");
    assert_eq!(connection.body["scope"], "openid,profile");

    let piece = piece_of(&create(&server, THOUGHT).await);
    approve(&server, &piece).await;
    let published = post(
        &server,
        &format!("/v1/pieces/{}/publish", id_of(&piece)),
        json!({"expected_revision": revision_of(&piece)}),
    )
    .await;
    assert_eq!(published.status, 200, "{}", published.raw);
    assert_eq!(published.body["status"], "posting_permission_missing");
    assert_eq!(published.body["post_id"], Value::Null);
    assert!(
        fake.posts().is_empty(),
        "a reduced grant must not be posted"
    );

    let reconnect = post_empty(&server, "/v1/linkedin/sign-in").await;
    assert_eq!(reconnect.status, 202, "{}", reconnect.raw);
    assert_eq!(reconnect.body["status"], "awaiting_consent");
}

#[tokio::test]
async fn linkedin_posting_permission_requires_the_exact_granted_scope_token() {
    for scope in ["openid profile", "w_member_social_feed", ""] {
        let store = MemoryStore::new();
        let mut token = stored_token("2026-12-29T18:00:00Z");
        token.scope = scope.into();
        store.save("linkedin", &token).unwrap();
        let fake = fake_linkedin::start(AuthMode::Consent, TokenMode::ok(), PostMode::NoId).await;
        let server = start_linkedin(
            Scripted::new(text("unused")),
            fast(),
            &fake,
            store,
            false,
            false,
        )
        .await;
        let connection = get(&server, "/v1/linkedin/connection").await;
        assert_eq!(
            connection.body["status"], "cannot_post",
            "scope {scope:?} must not look connected: {}",
            connection.raw
        );
        assert_eq!(connection.body["reason"], "posting_permission_missing");
    }
}
