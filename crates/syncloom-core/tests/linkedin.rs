//! The LinkedIn integration against the fake LinkedIn (linkedin-integration and
//! platform-credentials requirements), through the same `Linkedin` the server and CLI use.
mod support;
use std::{collections::HashMap, sync::Arc, time::Duration};
use support::fake_linkedin::{AuthMode, FakeLinkedin, PostMode, TokenMode, start};
use support::memory_store::MemoryStore;
use support::{
    CLIENT_SECRET_CANARY, FIXED_AT, TOKEN_EXPIRES_AT, browser, fast, fixed_clock, free_port,
    linkedin_config, stored_token,
};
use syncloom_core::{
    AuthFlow, ConnectionStatus, Linkedin, Platform, PostFailure, PostRequest, StudioError,
};

fn query(url: &str) -> (String, HashMap<String, String>) {
    let parsed = url::Url::parse(url).unwrap();
    (
        parsed.path().to_owned(),
        parsed.query_pairs().into_owned().collect(),
    )
}

async fn fake() -> FakeLinkedin {
    start(
        AuthMode::Consent,
        TokenMode::ok(),
        PostMode::Posted(support::fake_linkedin::DEFAULT_POST_ID.to_owned()),
    )
    .await
}

fn linkedin_with(
    fake: &FakeLinkedin,
    store: Arc<MemoryStore>,
    flow: AuthFlow,
    configured: bool,
) -> Linkedin {
    Linkedin::new(
        linkedin_config(&fake.base_url(), free_port(), flow, configured),
        store,
        fixed_clock(),
        fast().sign_in_deadline,
    )
}

fn request() -> PostRequest {
    PostRequest {
        request_id: "dispatch-904f4c443cf556d7f34c6c307a86dfe3".to_owned(),
        text: "A draft about zebrafish.".to_owned(),
        image: None,
        slides: Vec::new(),
    }
}

// linkedin-integration: Sign-in begins with the authorization URL, Standard flow URL
#[tokio::test]
async fn the_standard_authorization_url_carries_the_exact_parameters_and_no_challenge() {
    let fake = fake().await;
    let store = MemoryStore::new();
    let linkedin = linkedin_with(&fake, store, AuthFlow::Standard, true);
    let start = linkedin.sign_in_begin().unwrap();
    assert_eq!(start.status, ConnectionStatus::AwaitingConsent);
    assert_eq!(
        start.expires_at, FIXED_AT,
        "now plus a sub-second test deadline, whole seconds"
    );
    let (path, params) = query(&start.authorization_url);
    assert!(start.authorization_url.starts_with(&fake.base_url()));
    assert_eq!(path, "/oauth/v2/authorization");
    assert_eq!(params["response_type"], "code");
    assert_eq!(params["client_id"], "fake-client-id");
    assert_eq!(params["redirect_uri"], linkedin_redirect(&linkedin));
    assert_eq!(params["scope"], "openid profile w_member_social");
    let state = &params["state"];
    assert!(state.len() >= 43, "state {state:?} is too short");
    assert!(
        state
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
        "{state}"
    );
    assert!(!params.contains_key("code_challenge"));
    assert!(!params.contains_key("code_challenge_method"));
    assert_eq!(
        linkedin.connection().unwrap().status,
        ConnectionStatus::AwaitingConsent
    );
}

fn linkedin_redirect(linkedin: &Linkedin) -> String {
    linkedin.redirect_url().to_owned()
}

// linkedin-integration: Native PKCE flow URL, and the verifier at the token endpoint
#[tokio::test]
async fn the_native_pkce_flow_sends_an_s256_challenge_and_a_verifier_without_the_secret() {
    use base64::Engine;
    use sha2::Digest;
    let fake = fake().await;
    let store = MemoryStore::new();
    let linkedin = linkedin_with(&fake, store.clone(), AuthFlow::NativePkce, true);
    let start = linkedin.sign_in_begin().unwrap();
    let (path, params) = query(&start.authorization_url);
    assert_eq!(path, "/oauth/native-pkce/authorization");
    assert_eq!(params["code_challenge_method"], "S256");
    let challenge = params["code_challenge"].clone();
    assert!(challenge.len() >= 43);

    let page = browser(&start.authorization_url).await;
    assert_eq!(page.status(), 200);
    let connection = linkedin.sign_in_wait().await.unwrap();
    assert_eq!(connection.status, ConnectionStatus::SignedIn);

    let tokens = fake.token_requests();
    assert_eq!(tokens.len(), 1);
    let form = &tokens[0].form;
    assert!(!form.contains_key("client_secret"), "{form:?}");
    assert_eq!(form["client_id"], "fake-client-id");
    let verifier = form["code_verifier"].as_bytes();
    let digest = sha2::Sha256::digest(verifier);
    let expected = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest);
    assert_eq!(
        challenge, expected,
        "the challenge is the S256 of the verifier"
    );
    assert!(store.get("linkedin").is_some());
}

// linkedin-integration: Sign-in completes, Consent; platform-credentials: Stored after sign-in
#[tokio::test]
async fn consent_exchanges_the_code_once_reads_userinfo_and_stores_the_token() {
    let fake = fake().await;
    let store = MemoryStore::new();
    let linkedin = linkedin_with(&fake, store.clone(), AuthFlow::Standard, true);
    let start = linkedin.sign_in_begin().unwrap();
    let page = browser(&start.authorization_url).await;
    assert_eq!(page.status(), 200);
    let connection = linkedin.sign_in_wait().await.unwrap();
    assert_eq!(connection.status, ConnectionStatus::SignedIn);
    assert_eq!(connection.expires_at.as_deref(), Some(TOKEN_EXPIRES_AT));
    assert_eq!(
        connection.scope.as_deref(),
        Some("openid,profile,w_member_social")
    );

    let auth = fake.auth_requests();
    assert_eq!(auth.len(), 1);
    let tokens = fake.token_requests();
    assert_eq!(tokens.len(), 1, "exactly one token request");
    let form = &tokens[0].form;
    assert_eq!(form["grant_type"], "authorization_code");
    assert_eq!(form["code"], "fake-code-1");
    assert_eq!(form["redirect_uri"], auth[0].query["redirect_uri"]);
    assert_eq!(form["client_id"], "fake-client-id");
    assert_eq!(form["client_secret"], CLIENT_SECRET_CANARY);
    assert!(!form.contains_key("code_verifier"));
    assert_eq!(
        fake.userinfo_bearers(),
        vec![Some("fake-access-token-zebrafish".to_owned())]
    );

    let token = store.get("linkedin").expect("the token is stored");
    assert_eq!(token.access_token.expose(), "fake-access-token-zebrafish");
    assert_eq!(token.expires_at.as_deref(), Some(TOKEN_EXPIRES_AT));
    assert_eq!(token.member_id, support::fake_linkedin::MEMBER_ID);
    assert_eq!(token.obtained_at, FIXED_AT);
    let shown = serde_json::to_string(&connection).unwrap();
    assert!(!shown.contains("fake-access-token"), "{shown}");
    assert_eq!(
        linkedin.connection().unwrap(),
        connection,
        "the connection reads back from the store"
    );
    assert_eq!(fake.posts().len(), 0);
}

// linkedin-integration: Denied
#[tokio::test]
async fn a_denied_consent_exchanges_nothing_and_stores_nothing() {
    let fake = start(AuthMode::Deny, TokenMode::ok(), PostMode::NoId).await;
    let store = MemoryStore::new();
    let linkedin = linkedin_with(&fake, store.clone(), AuthFlow::Standard, true);
    let start = linkedin.sign_in_begin().unwrap();
    let page = browser(&start.authorization_url).await;
    assert_eq!(page.status(), 200);
    let body = page.text().await.unwrap();
    assert!(
        !body.contains("zebrafish"),
        "the callback page echoed the error description: {body}"
    );
    assert_eq!(
        linkedin.sign_in_wait().await.unwrap_err(),
        StudioError::SignInDenied
    );
    assert!(fake.token_requests().is_empty());
    assert!(store.get("linkedin").is_none());
    assert_eq!(
        linkedin.connection().unwrap().status,
        ConnectionStatus::SignedOut
    );
}

// linkedin-integration: Wrong state
#[tokio::test]
async fn a_callback_with_the_wrong_state_is_refused_before_any_exchange() {
    let fake = start(AuthMode::WrongState, TokenMode::ok(), PostMode::NoId).await;
    let store = MemoryStore::new();
    let linkedin = linkedin_with(&fake, store.clone(), AuthFlow::Standard, true);
    let start = linkedin.sign_in_begin().unwrap();
    browser(&start.authorization_url).await;
    assert_eq!(
        linkedin.sign_in_wait().await.unwrap_err(),
        StudioError::SignInStateMismatch
    );
    assert!(
        fake.token_requests().is_empty(),
        "no token request may follow a bad state"
    );
    assert!(store.get("linkedin").is_none());
}

// linkedin-integration: Token refused
#[tokio::test]
async fn a_refused_token_request_is_sign_in_rejected_and_stores_nothing() {
    for status in [400u16, 401] {
        let fake = start(AuthMode::Consent, TokenMode::Status(status), PostMode::NoId).await;
        let store = MemoryStore::new();
        let linkedin = linkedin_with(&fake, store.clone(), AuthFlow::Standard, true);
        let start = linkedin.sign_in_begin().unwrap();
        browser(&start.authorization_url).await;
        assert_eq!(
            linkedin.sign_in_wait().await.unwrap_err(),
            StudioError::SignInRejected,
            "{status}"
        );
        assert_eq!(fake.token_requests().len(), 1);
        assert!(fake.userinfo_bearers().is_empty());
        assert!(store.get("linkedin").is_none());
    }
}

// linkedin-integration: No callback
#[tokio::test]
async fn no_callback_within_the_deadline_times_out_and_frees_the_listener() {
    let fake = fake().await;
    let linkedin = linkedin_with(&fake, MemoryStore::new(), AuthFlow::Standard, true);
    let start = linkedin.sign_in_begin().unwrap();
    assert_eq!(
        linkedin.sign_in_wait().await.unwrap_err(),
        StudioError::SignInTimedOut
    );
    assert!(fake.token_requests().is_empty());
    // The port is free again: a new sign-in binds it.
    let again = linkedin.sign_in_begin().unwrap();
    assert_ne!(
        query(&again.authorization_url).1["state"],
        query(&start.authorization_url).1["state"]
    );
    assert_eq!(
        linkedin.connection().unwrap().status,
        ConnectionStatus::AwaitingConsent
    );
}

// linkedin-integration: Not configured, Already waiting, Nothing pending
#[tokio::test]
async fn sign_in_needs_the_app_credentials_and_runs_one_at_a_time() {
    let fake = fake().await;
    let unconfigured = linkedin_with(&fake, MemoryStore::new(), AuthFlow::Standard, false);
    assert_eq!(
        unconfigured.sign_in_begin().unwrap_err(),
        StudioError::LinkedinNotConfigured
    );
    assert_eq!(
        unconfigured.sign_in_wait().await.unwrap_err(),
        StudioError::SignInNotStarted
    );
    let linkedin = linkedin_with(&fake, MemoryStore::new(), AuthFlow::Standard, true);
    assert_eq!(
        linkedin.sign_in_wait().await.unwrap_err(),
        StudioError::SignInNotStarted
    );
    let start = linkedin.sign_in_begin().unwrap();
    assert_eq!(
        linkedin.sign_in_begin().unwrap_err(),
        StudioError::SignInInProgress
    );
    browser(&start.authorization_url).await;
    linkedin.sign_in_wait().await.unwrap();
    assert_eq!(
        linkedin.sign_in_wait().await.unwrap_err(),
        StudioError::SignInNotStarted,
        "a finished sign-in is not pending"
    );
}

// linkedin-integration: Callback listener answers a fixed page
#[tokio::test]
async fn the_listener_serves_only_the_callback_path_and_ignores_a_second_callback() {
    let fake = fake().await;
    let linkedin = linkedin_with(&fake, MemoryStore::new(), AuthFlow::Standard, true);
    let start = linkedin.sign_in_begin().unwrap();
    let redirect = url::Url::parse(linkedin.redirect_url()).unwrap();
    let other = format!(
        "http://{}:{}/somewhere-else",
        redirect.host_str().unwrap(),
        redirect.port().unwrap()
    );
    assert_eq!(browser(&other).await.status(), 404);
    let first = browser(&start.authorization_url).await;
    assert_eq!(first.status(), 200);
    assert_eq!(
        first.headers()["content-type"]
            .to_str()
            .unwrap()
            .split(';')
            .next(),
        Some("text/html")
    );
    // A second callback (another code from the fake) changes nothing.
    let second = browser(&start.authorization_url).await;
    assert!(second.status() == 200 || second.status() == 404 || second.status() == 410);
    linkedin.sign_in_wait().await.unwrap();
    let tokens = fake.token_requests();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].form["code"], "fake-code-1");
}

// platform-credentials: Connection states, Sign-out deletes the token
#[tokio::test]
async fn connection_states_come_from_the_stored_token_and_the_clock_only() {
    let fake = fake().await;
    let signed_out = linkedin_with(&fake, MemoryStore::new(), AuthFlow::Standard, true);
    let connection = signed_out.connection().unwrap();
    assert_eq!(connection.status, ConnectionStatus::SignedOut);
    assert_eq!(connection.expires_at, None);

    let store = MemoryStore::with("linkedin", stored_token(TOKEN_EXPIRES_AT));
    let signed_in = linkedin_with(&fake, store.clone(), AuthFlow::Standard, false);
    let connection = signed_in.connection().unwrap();
    assert_eq!(connection.status, ConnectionStatus::SignedIn);
    assert_eq!(connection.expires_at.as_deref(), Some(TOKEN_EXPIRES_AT));
    assert_eq!(
        connection.scope.as_deref(),
        Some("openid,profile,w_member_social")
    );

    let expired_store = MemoryStore::with("linkedin", stored_token(FIXED_AT));
    let expired = linkedin_with(&fake, expired_store, AuthFlow::Standard, false);
    let connection = expired.connection().unwrap();
    assert_eq!(connection.status, ConnectionStatus::Expired);
    assert_eq!(connection.expires_at.as_deref(), Some(FIXED_AT));

    let out = signed_in.sign_out().unwrap();
    assert_eq!(out.status, ConnectionStatus::SignedOut);
    assert!(store.get("linkedin").is_none());
    assert_eq!(
        signed_in.sign_out().unwrap().status,
        ConnectionStatus::SignedOut
    );
    assert!(
        fake.auth_requests().is_empty()
            && fake.posts().is_empty()
            && fake.userinfo_bearers().is_empty()
    );
}

// linkedin-integration: One Posts API request per attempt, Request shape
#[tokio::test]
async fn a_publish_sends_one_versioned_posts_request_with_the_text_as_commentary() {
    let fake = fake().await;
    let store = MemoryStore::with("linkedin", stored_token(TOKEN_EXPIRES_AT));
    let linkedin = linkedin_with(&fake, store, AuthFlow::Standard, false);
    let posted = linkedin.publish(request()).await.unwrap();
    assert_eq!(posted.post_id, "urn:li:share:7000000000000000001");
    assert_eq!(
        posted.url,
        "https://www.linkedin.com/feed/update/urn:li:share:7000000000000000001/"
    );
    let posts = fake.posts();
    assert_eq!(posts.len(), 1);
    let sent = &posts[0];
    assert_eq!(
        sent.headers["authorization"],
        "Bearer fake-access-token-zebrafish"
    );
    assert_eq!(sent.headers["linkedin-version"], "202610");
    assert_eq!(sent.headers["x-restli-protocol-version"], "2.0.0");
    assert!(sent.headers["content-type"].starts_with("application/json"));
    assert_eq!(
        sent.body,
        serde_json::json!({
            "author": "urn:li:person:FAKEMEMBER01",
            "commentary": "A draft about zebrafish.",
            "visibility": "PUBLIC",
            "distribution": {
                "feedDistribution": "MAIN_FEED",
                "targetEntities": [],
                "thirdPartyDistributionChannels": []
            },
            "lifecycleState": "PUBLISHED",
            "isReshareDisabledByAuthor": false
        })
    );
    for personal in ["@", "camilo", "Camilo", "+1", "juancamilo"] {
        assert!(!sent.raw_body.contains(personal), "{personal} in body");
        for value in sent.headers.values() {
            assert!(!value.contains(personal), "{personal} in a header");
        }
    }
}

// linkedin-integration: LinkedIn answers map to one outcome, Each failure
#[tokio::test]
async fn each_linkedin_answer_maps_to_its_outcome() {
    let cases: [(PostMode, Result<(), PostFailure>); 7] = [
        (PostMode::Status(401), Err(PostFailure::ConsentRequired)),
        (PostMode::Status(403), Err(PostFailure::Refused)),
        (PostMode::Status(422), Err(PostFailure::Refused)),
        (PostMode::Status(429), Err(PostFailure::RateLimited)),
        (PostMode::Status(500), Err(PostFailure::Unknown)),
        (PostMode::NoId, Err(PostFailure::Unknown)),
        (
            PostMode::Posted(support::fake_linkedin::DEFAULT_POST_ID.to_owned()),
            Ok(()),
        ),
    ];
    let fake = start(AuthMode::Consent, TokenMode::ok(), PostMode::NoId).await;
    let store = MemoryStore::with("linkedin", stored_token(TOKEN_EXPIRES_AT));
    let linkedin = linkedin_with(&fake, store, AuthFlow::Standard, false);
    for (index, (mode, expected)) in cases.into_iter().enumerate() {
        fake.set_post(mode.clone());
        let result = linkedin.publish(request()).await.map(|_| ());
        assert_eq!(result, expected, "{mode:?}");
        assert_eq!(
            fake.posts().len(),
            index + 1,
            "exactly one request per attempt"
        );
    }
}

// linkedin-integration: Each failure, timeout and closed port
#[tokio::test]
async fn a_slow_answer_is_unknown_and_a_closed_port_is_unreachable() {
    let fake = start(AuthMode::Consent, TokenMode::ok(), PostMode::Slow).await;
    let store = MemoryStore::with("linkedin", stored_token(TOKEN_EXPIRES_AT));
    let linkedin = linkedin_with(&fake, store.clone(), AuthFlow::Standard, false);
    let result =
        tokio::time::timeout(Duration::from_millis(300), linkedin.publish(request())).await;
    // The adapter itself does not time out; the studio's deadline does. Dropping the future
    // abandons the call, and the studio records `unknown`.
    assert!(
        result.is_err(),
        "the adapter waits for the platform; the studio applies the deadline"
    );
    assert_eq!(fake.posts().len(), 1);

    let closed = support::free_port();
    let unreachable = Linkedin::new(
        linkedin_config(
            &format!("http://127.0.0.1:{closed}"),
            free_port(),
            AuthFlow::Standard,
            false,
        ),
        store,
        fixed_clock(),
        fast().sign_in_deadline,
    );
    assert_eq!(
        unreachable.publish(request()).await,
        Err(PostFailure::Unreachable)
    );
}

// platform-credentials: Absent or expired token sends nothing
#[tokio::test]
async fn an_absent_or_expired_token_is_consent_required_with_zero_requests() {
    let fake = fake().await;
    let absent = linkedin_with(&fake, MemoryStore::new(), AuthFlow::Standard, false);
    assert_eq!(
        absent.publish(request()).await,
        Err(PostFailure::ConsentRequired)
    );
    let store = MemoryStore::with("linkedin", stored_token(FIXED_AT));
    let expired = linkedin_with(&fake, store.clone(), AuthFlow::Standard, false);
    assert_eq!(
        expired.publish(request()).await,
        Err(PostFailure::ConsentRequired)
    );
    assert!(
        fake.posts().is_empty(),
        "nothing may be sent without a usable token"
    );
    assert_eq!(
        store.get("linkedin"),
        Some(stored_token(FIXED_AT)),
        "the token is unchanged"
    );
}
