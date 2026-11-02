use super::*;

#[tokio::test]
async fn joint_account_and_grant_checks_keep_the_dispatch_snapshot_across_media_reconnect()
-> Result<(), Box<dyn std::error::Error>> {
    use syncloom_core::{Channel, TokenStore};
    let fake = fake::Fake::x().await;
    let (server, q, _, store) = setup(&fake, "x", true).await;
    login(&server, "x").await;
    let piece = draft(&server, "x", true).await;
    approve_piece(&server, &piece).await;
    let before = get(&server, &format!("/v1/pieces/{}", id_of(&piece)))
        .await
        .body["approval_target"]
        .clone();
    let hold = fake.hold("/2/media/upload/initialize");
    let count = fake.calls().len();
    let url = server.url(&publish_path(&piece));
    let session = server.token.clone();
    let revision = piece["revision"].clone();
    let publishing = tokio::spawn(async move {
        client()
            .post(url)
            .header("x-syncloom-token", session)
            .json(&json!({"expected_revision":revision,"accepted_cost_microusd":20000}))
            .send()
            .await
    });
    fake.wait_calls(count + 1).await;
    send_json(
        &server,
        reqwest::Method::DELETE,
        "/v1/x/connection",
        Value::Null,
    )
    .await;
    fake.reply("/2/users/me", 200, json!({"data":{"id":"222"}}));
    fake.reply("/2/oauth2/token",200,json!({"access_token":"joint-b","refresh_token":"joint-b-refresh","expires_in":7200,"token_type":"bearer","scope":"tweet.write users.read media.write"}));
    login(&server, "x").await;
    let stale=send_json(&server,reqwest::Method::PATCH,&format!("/v1/pieces/{}",id_of(&piece)),json!({"status":"approved","expected_revision":piece["revision"],"expected_target":before,"expected_assets":{"image_sha256":piece["assets"]["image_sha256"],"carousel_sha256":piece["assets"]["carousel_sha256"],"slides_sha256":[]}})).await;
    assert_eq!(
        (stale.status, stale.code()),
        (409, "account_approval_required")
    );
    approve_piece(&server, &piece).await;
    let mut b = store.load(&q.account(Channel::X)?)?.ok_or("B token")?;
    b.scope = "tweet.write users.read".into();
    store.save(&q.account(Channel::X)?, &b)?;
    hold.notify_one();
    let result: Value = publishing.await??.json().await?;
    assert_eq!(result["status"], "posted");
    assert_eq!(result["approval_target"], before);
    let a_effects = fake
        .calls()
        .into_iter()
        .filter(|c| c.path.contains("/media/") || c.path == "/2/tweets")
        .collect::<Vec<_>>();
    assert_eq!(a_effects.len(), 5);
    assert!(
        a_effects
            .iter()
            .all(|c| c.authorization == "Bearer x-access-1")
    );
    let denied = post(
        &server,
        &publish_path(&piece),
        json!({"expected_revision":piece["revision"],"accepted_cost_microusd":20000}),
    )
    .await;
    assert_eq!(denied.body["status"], "posting_permission_missing");
    assert_ne!(denied.body["approval_target"], before);
    assert_eq!(
        fake.calls()
            .iter()
            .filter(|c| c.path.contains("/media/") || c.path == "/2/tweets")
            .count(),
        5
    );
    b.scope = "tweet.write users.read media.write".into();
    store.save(&q.account(Channel::X)?, &b)?;
    hold.notify_one();
    let posted = post(
        &server,
        &publish_path(&piece),
        json!({"expected_revision":piece["revision"],"accepted_cost_microusd":20000}),
    )
    .await;
    assert_eq!(posted.body["status"], "posted");
    assert_eq!(
        posted.body["approval_target"],
        denied.body["approval_target"]
    );
    let all = fake.calls();
    let posts = all
        .iter()
        .filter(|c| c.path == "/2/tweets")
        .collect::<Vec<_>>();
    assert_eq!(posts.len(), 2);
    assert_eq!(posts[1].authorization, "Bearer joint-b");
    Ok(())
}

async fn assert_issued_identity_failure(
    scope: Value,
    permission: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use syncloom_core::{Channel, TokenStore};
    let fake = fake::Fake::x().await;
    let (server, q, _, store) = setup(&fake, "x", true).await;
    login(&server, "x").await;
    let piece = draft(&server, "x", true).await;
    approve_piece(&server, &piece).await;
    let old = store.load(&q.account(Channel::X)?)?.ok_or("old token")?;
    fake.reply(
        "/2/oauth2/token",
        200,
        json!({
            "access_token":"seam-new", "refresh_token":"seam-refresh",
            "expires_in":7200, "token_type":"bearer", "scope":scope
        }),
    );
    fake.reply(
        "/2/users/me",
        403,
        json!({"error":"synthetic identity unavailable"}),
    );
    let refreshed = post_empty(&server, "/v1/x/refresh").await;
    let connection = get(&server, "/v1/x/connection").await;
    let before = fake
        .calls()
        .iter()
        .filter(|c| c.path.contains("/media/") || c.path == "/2/tweets")
        .count();
    let result = post(
        &server,
        &publish_path(&piece),
        json!({
            "expected_revision":piece["revision"],"accepted_cost_microusd":20000
        }),
    )
    .await;
    let after = fake
        .calls()
        .iter()
        .filter(|c| c.path.contains("/media/") || c.path == "/2/tweets")
        .count();
    assert_eq!(
        (
            refreshed.status,
            refreshed.code(),
            connection.body["status"].as_str(),
            result.body["status"].as_str(),
            after - before
        ),
        (403, permission, Some("cannot_post"), Some(permission), 0)
    );
    let token = store
        .load(&q.account(Channel::X)?)?
        .ok_or("issued token lost")?;
    let bundle: Value = serde_json::from_str(token.access_token.expose())?;
    assert_eq!(bundle["access_token"], "seam-new");
    assert_eq!(bundle["refresh_token"], "seam-refresh");
    assert_eq!(
        token.member_id, old.member_id,
        "expected identity must not be inferred or lost"
    );
    assert_eq!(serde_json::to_value(&token)?["identity_confirmed"], false);
    assert_eq!(token.scope, scope.as_str().unwrap_or(""));
    assert_eq!(token.scope_verified, scope.is_string());
    Ok(())
}

#[tokio::test]
async fn sign_out_during_new_token_identity_confirmation_cannot_resurrect_or_publish()
-> Result<(), Box<dyn std::error::Error>> {
    use syncloom_core::{Channel, TokenStore};
    let fake = fake::Fake::x().await;
    let (server, q, _, store) = setup(&fake, "x", true).await;
    login(&server, "x").await;
    let piece = draft(&server, "x", false).await;
    approve_piece(&server, &piece).await;
    fake.reply("/2/oauth2/token", 200, json!({"access_token":"confirmation-new", "refresh_token":"confirmation-refresh", "expires_in":7200, "token_type":"bearer", "scope":"tweet.write users.read"}));
    fake.reply(
        "/2/users/me",
        503,
        json!({"error":"temporary identity failure"}),
    );
    assert_eq!(
        post_empty(&server, "/v1/x/refresh").await.code(),
        "identity_unconfirmed"
    );
    fake.reply("/2/users/me", 200, json!({"data":{"id":"111"}}));
    let hold = fake.hold("/2/users/me");
    let count = fake.calls().len();
    let url = server.url(&publish_path(&piece));
    let session = server.token.clone();
    let revision = piece["revision"].clone();
    let publishing = tokio::spawn(async move {
        client()
            .post(url)
            .header("x-syncloom-token", session)
            .json(&json!({"expected_revision":revision, "accepted_cost_microusd":15000}))
            .send()
            .await
    });
    fake.wait_calls(count + 1).await;
    let signed_out = send_json(
        &server,
        reqwest::Method::DELETE,
        "/v1/x/connection",
        Value::Null,
    )
    .await;
    assert_eq!(signed_out.body["status"], "signed_out");
    hold.notify_one();
    let result: Value = publishing.await??.json().await?;
    assert_eq!(result["status"], "consent_required");
    assert_eq!(
        get(&server, "/v1/x/connection").await.body["status"],
        "signed_out"
    );
    assert!(store.load(&q.account(Channel::X)?)?.is_none());
    assert!(
        !fake
            .calls()
            .iter()
            .any(|c| c.path.contains("/media/") || c.path == "/2/tweets")
    );
    Ok(())
}

// Main1375: successful issuance is not a pre-issuance exchange failure.
#[tokio::test]
async fn rotation_reduced_x_with_failed_identity_keeps_new_bundle_without_media()
-> Result<(), Box<dyn std::error::Error>> {
    assert_issued_identity_failure(json!("tweet.write"), "posting_permission_missing").await
}
#[tokio::test]
async fn rotation_null_x_with_failed_identity_keeps_new_bundle_without_media()
-> Result<(), Box<dyn std::error::Error>> {
    assert_issued_identity_failure(Value::Null, "posting_permission_unverified").await
}

async fn assert_transient_identity_recovery(
    platform: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use syncloom_core::{Channel, TokenStore};
    {
        let fake = if platform == "x" {
            fake::Fake::x().await
        } else {
            fake::Fake::threads().await
        };
        let (server, q, clock, store) = setup(&fake, platform, true).await;
        login(&server, platform).await;
        let piece = draft(&server, platform, false).await;
        approve_piece(&server, &piece).await;
        let channel = if platform == "x" {
            Channel::X
        } else {
            Channel::Threads
        };
        let old = store.load(&q.account(channel)?)?.ok_or("old token")?;
        let identity = if platform == "x" {
            "/2/users/me"
        } else {
            "/v1.0/me"
        };
        if platform == "threads" {
            *clock.0.lock().map_err(|_| "clock")? += time::Duration::days(1);
        }
        if platform == "x" {
            fake.reply("/2/oauth2/token", 200, json!({"access_token":"x-access-2","refresh_token":"x-refresh-2","expires_in":7200,"token_type":"bearer","scope":"tweet.write users.read"}));
        }
        fake.reply(
            identity,
            503,
            json!({"error":"synthetic identity unavailable"}),
        );
        let refresh = post_empty(&server, &format!("/v1/{platform}/refresh")).await;
        let connection = get(&server, &format!("/v1/{platform}/connection")).await;
        let saved = store
            .load(&q.account(channel)?)?
            .ok_or("issued token lost")?;
        assert_ne!(
            saved.access_token, old.access_token,
            "rotation must retain the new credential even after identity read failure"
        );
        assert_eq!(saved.member_id, old.member_id);
        assert_eq!(serde_json::to_value(&saved)?["identity_confirmed"], false);
        assert_eq!(
            (
                refresh.status,
                refresh.code(),
                connection.body["reason"].as_str()
            ),
            (403, "identity_unconfirmed", Some("identity_unconfirmed"))
        );
        let before = fake.calls().len();
        let refused = post(
            &server,
            &publish_path(&piece),
            json!({"expected_revision":piece["revision"],"accepted_cost_microusd":15000}),
        )
        .await;
        assert_eq!(refused.body["status"], "consent_required");
        assert_eq!(
            fake.calls().len(),
            before + 1,
            "only read-only identity retry"
        );
        fake.reply(
            identity,
            200,
            if platform == "x" {
                json!({"data":{"id":"111"}})
            } else {
                json!({"id":"303"})
            },
        );
        let posted = post(
            &server,
            &publish_path(&piece),
            json!({"expected_revision":piece["revision"],"accepted_cost_microusd":15000}),
        )
        .await;
        assert_eq!(posted.body["status"], "posted");
        let confirmed = store.load(&q.account(channel)?)?.ok_or("confirmed token")?;
        assert_eq!(confirmed.access_token, saved.access_token);
        assert_eq!(
            serde_json::to_value(&confirmed)?["identity_confirmed"],
            true
        );
        let writes = fake
            .calls()
            .into_iter()
            .filter(|c| c.path == "/2/tweets" || c.path.ends_with("/threads_publish"))
            .collect::<Vec<_>>();
        assert_eq!(writes.len(), 1);
        assert_eq!(
            writes[0].authorization,
            if platform == "x" {
                "Bearer x-access-2"
            } else {
                "Bearer threads-long-2"
            }
        );
    }
    Ok(())
}

#[tokio::test]
async fn rotation_transient_x_identity_failure_recovers_only_this_new_token()
-> Result<(), Box<dyn std::error::Error>> {
    assert_transient_identity_recovery("x").await
}
#[tokio::test]
async fn rotation_transient_threads_identity_failure_recovers_only_this_new_token()
-> Result<(), Box<dyn std::error::Error>> {
    assert_transient_identity_recovery("threads").await
}
