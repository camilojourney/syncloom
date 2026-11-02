mod support;
use support::{
    scripted::{Scripted, text},
    *,
};
use syncloom_core::TokenStore;

#[tokio::test]
async fn native_reconnect_uses_existing_consent_and_returns_without_approving_or_posting()
-> Result<(), Box<dyn std::error::Error>> {
    let fake = fake_linkedin::start(
        fake_linkedin::AuthMode::Consent,
        fake_linkedin::TokenMode::ok(),
        fake_linkedin::PostMode::NoId,
    )
    .await;
    let mut reduced = stored_token("2026-12-29T18:00:00Z");
    reduced.scope = "openid profile".into();
    let store = memory_store::MemoryStore::with("linkedin", reduced);
    let server = start_linkedin(
        Scripted::new(text("Build with evidence.")),
        fast(),
        &fake,
        store.clone(),
        true,
        true,
    )
    .await;
    let piece = piece_of(&create(&server, THOUGHT).await);
    approve(&server, &piece).await;
    let review = format!("/review/{}", id_of(&piece));
    let page = get(&server, &review).await;
    assert!(
        page.raw.contains(&format!("action=\"{review}/reconnect\"")),
        "missing supported native reconnect control"
    );
    let csrf = get(&server, "/v1/session").await.body["token"]
        .as_str()
        .ok_or("csrf")?
        .to_owned();
    let begin = format!("{review}/reconnect");
    let complete = format!("{begin}/complete");
    for fields in [vec![], vec![("csrf", "wrong")]] {
        let denied = answer(
            client()
                .post(server.url(&begin))
                .form(&fields)
                .send()
                .await?,
        )
        .await;
        assert_eq!(
            (denied.status, denied.raw.contains("session_required")),
            (403, true)
        );
    }
    for path in [&begin, &complete] {
        let denied = answer(
            client()
                .post(server.url(path))
                .header("origin", "https://untrusted.invalid")
                .form(&[("csrf", csrf.as_str())])
                .send()
                .await?,
        )
        .await;
        assert_eq!(
            (denied.status, denied.raw.contains("origin_not_allowed")),
            (403, true)
        );
        let denied = raw(
            server.addr,
            &head_with_hosts(
                &format!("POST {path} HTTP/1.1"),
                &["Host: untrusted.invalid"],
                &[],
            ),
        )
        .await;
        assert_eq!(
            (denied.status, denied.raw.contains("host_not_allowed")),
            (403, true)
        );
    }
    assert!(fake.auth_requests().is_empty());
    let start = answer(
        client()
            .post(server.url(&begin))
            .header("origin", server.url(""))
            .form(&[("csrf", csrf.as_str())])
            .send()
            .await?,
    )
    .await;
    assert_eq!(start.status, 200);
    assert!(
        start
            .raw
            .contains("This does not approve or publish your piece.")
    );
    let auth = start
        .raw
        .split_once("id=\"provider-sign-in\" href=\"")
        .ok_or("provider link")?
        .1
        .split('"')
        .next()
        .ok_or("provider href")?
        .replace("&amp;", "&");
    reqwest::get(&auth).await?;
    let finished = answer(
        reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .build()?
            .post(server.url(&complete))
            .header("origin", server.url(""))
            .form(&[("csrf", csrf.as_str())])
            .send()
            .await?,
    )
    .await;
    assert_eq!(finished.status, 303);
    assert_eq!(finished.headers["location"], review);
    assert_eq!(
        get(&server, "/v1/linkedin/connection").await.body["status"],
        "signed_in"
    );
    assert!(
        store
            .load("linkedin")?
            .ok_or("token")?
            .scope
            .contains("w_member_social")
    );
    let refreshed = get(&server, &review).await;
    assert!(!refreshed.raw.contains("posting permission missing"));
    assert!(fake.posts().is_empty());
    assert_eq!(fake.token_requests().len(), 1);
    assert_eq!(
        get(&server, &format!("/v1/pieces/{}", id_of(&piece)))
            .await
            .body["approval_target"],
        get(&server, &format!("/v1/pieces/{}", id_of(&piece)))
            .await
            .body["current_target"]
    );
    Ok(())
}
