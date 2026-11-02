#[path = "support/fake_social.rs"]
mod fake;
#[path = "support/social.rs"]
mod fixture;
mod support;
use fake::Fake;
use fixture::*;
use serde_json::json;
use support::memory_store::MemoryStore;
use syncloom_core::{Secret, StudioError, TokenStore};

#[tokio::test]
async fn bluesky_requires_handle_and_password_without_sending_private_identifiers() {
    let fake = Fake::bluesky().await;
    let q = social(&fake, MemoryStore::new(), clock());
    for (identifier, password) in [
        ("", "test"),
        ("test@example.invalid", "test"),
        ("test.invalid", " "),
    ] {
        assert_eq!(
            q.bluesky.sign_in(identifier, Secret::new(password)).await,
            Err(StudioError::InvalidRequest)
        );
    }
    assert!(fake.calls().is_empty());
}

#[tokio::test]
async fn bluesky_rejects_expired_or_unrefreshable_sessions_and_identity_drift() {
    for (exp, refresh) in [(1793412000, "refresh"), (1793455200, "")] {
        let fake = Fake::bluesky().await;
        let store = MemoryStore::new();
        let q = social(&fake, store.clone(), clock());
        fake.reply("/xrpc/com.atproto.server.createSession",200,json!({"accessJwt":fake::jwt(exp),"refreshJwt":refresh,"did":"did:plc:testrepo","handle":"test.invalid"}));
        assert_eq!(
            q.bluesky.sign_in("test.invalid", Secret::new("test")).await,
            Err(StudioError::SignInRejected)
        );
        assert!(
            store
                .load(&q.account(syncloom_core::Channel::Bluesky).unwrap())
                .unwrap()
                .is_none()
        );
    }
    let fake = Fake::bluesky().await;
    let store = MemoryStore::new();
    let q = social(&fake, store.clone(), clock());
    q.bluesky
        .sign_in("test.invalid", Secret::new("test"))
        .await
        .unwrap();
    assert!(
        store
            .load(&q.account(syncloom_core::Channel::Bluesky).unwrap())
            .unwrap()
            .is_some()
    );
    fake.reply("/xrpc/com.atproto.server.refreshSession",200,json!({"accessJwt":fake::jwt(1793541600),"refreshJwt":"new-refresh","did":"did:plc:different","handle":"other.invalid"}));
    assert_eq!(q.bluesky.refresh().await, Err(StudioError::SignInRejected));
    assert_eq!(
        store
            .load(&q.account(syncloom_core::Channel::Bluesky).unwrap())
            .unwrap(),
        None
    );
}

#[tokio::test]
async fn x_rejects_invalid_issued_tokens_without_committing_credentials() {
    for (key, value) in [
        ("expires_in", json!(0)),
        ("expires_in", json!(u64::MAX)),
        ("expires_in", json!(i64::MAX)),
        ("access_token", json!("")),
        ("refresh_token", json!("")),
        ("token_type", json!("basic")),
        ("error", json!({"code":190})),
    ] {
        let fake = Fake::x().await;
        let store = MemoryStore::new();
        let q = social(&fake, store.clone(), clock());
        let mut issued = json!({"access_token":"token","refresh_token":"refresh","expires_in":7200,"scope":"tweet.read tweet.write users.read media.write offline.access","token_type":"bearer"});
        issued[key] = value;
        fake.reply("/2/oauth2/token", 200, issued);
        let start = q.x.sign_in_begin().unwrap();
        let u = reqwest::Url::parse(&start.authorization_url).unwrap();
        let pairs: std::collections::HashMap<_, _> = u.query_pairs().into_owned().collect();
        reqwest::get(format!(
            "{}?code=c&state={}",
            pairs["redirect_uri"], pairs["state"]
        ))
        .await
        .unwrap();
        assert_eq!(
            q.x.sign_in_wait().await,
            Err(StudioError::SignInRejected),
            "{key}"
        );
        assert!(
            store
                .load(&q.account(syncloom_core::Channel::X).unwrap())
                .unwrap()
                .is_none()
        );
    }
}

#[tokio::test]
async fn unresolved_or_mismatched_x_and_threads_identity_never_authorizes_posting()
-> Result<(), Box<dyn std::error::Error>> {
    use syncloom_core::{Channel, ConnectionStatus, Platform};
    for channel in [Channel::X, Channel::Threads] {
        let fake = if channel == Channel::X {
            Fake::x().await
        } else {
            Fake::threads().await
        };
        let store = MemoryStore::new();
        let c = clock();
        let q = social(&fake, store.clone(), c.clone());
        if channel == Channel::X {
            x_login(&q).await;
        } else {
            threads_login(&q).await;
        }
        let old = store
            .load(&q.account(channel)?)?
            .ok_or("missing signed-in token")?;
        assert!(!old.member_id.is_empty());
        let path = if channel == Channel::X {
            "/2/users/me"
        } else {
            "/v1.0/me"
        };
        fake.reply(
            path,
            200,
            if channel == Channel::X {
                json!({"data":{"id":"999"}})
            } else {
                json!({"id":"999"})
            },
        );
        if channel == Channel::Threads {
            c.0.lock()
                .map_err(|_| "clock poisoned")?
                .clone_from(&time::macros::datetime!(2026-11-01 02:00 UTC));
        }
        assert_eq!(q.refresh(channel).await, Err(StudioError::SignInRejected));
        assert_eq!(q.connection(channel)?.status, ConnectionStatus::SignedOut);
        assert!(store.load(&q.account(channel)?)?.is_none());
        let platform: &dyn Platform = if channel == Channel::X {
            q.x.as_ref()
        } else {
            q.threads.as_ref()
        };
        let count = fake.calls().len();
        assert_eq!(
            platform.publish(text(platform)).await,
            Err(syncloom_core::PostFailure::ConsentRequired)
        );
        assert_eq!(fake.calls().len(), count);
        fake.reply(path, 200, json!({}));
        if channel == Channel::X {
            let started = q.x.sign_in_begin()?;
            let url = reqwest::Url::parse(&started.authorization_url)?;
            let pairs: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
            reqwest::get(format!(
                "{}?code=c&state={}",
                pairs["redirect_uri"], pairs["state"]
            ))
            .await?;
            assert_eq!(q.x.sign_in_wait().await, Err(StudioError::SignInRejected));
        } else {
            let started = q.threads.sign_in_begin()?;
            let url = reqwest::Url::parse(&started.authorization_url)?;
            let state = url
                .query_pairs()
                .find(|(k, _)| k == "state")
                .ok_or("missing state")?
                .1
                .into_owned();
            assert_eq!(
                q.threads
                    .complete(&format!(
                        "https://redirect.invalid/callback?code=c&state={state}"
                    ))
                    .await,
                Err(StudioError::SignInRejected)
            );
        }
        assert!(store.load(&q.account(channel)?)?.is_none());
    }
    Ok(())
}
