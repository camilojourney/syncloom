//! Task-created, exposed diagnostic cases. Real TCP + real renderer, not model calibration.
mod support;
use serde_json::{Value, json};
use support::{scripted::Scripted, *};

async fn rendered() -> (TestServer, Value) {
    let server = start_with(Scripted::new(scripted::text(THOUGHT)), fast()).await;
    let piece = piece_of(&create(&server, THOUGHT).await);
    let result = post(
        &server,
        &format!("/v1/pieces/{}/render", id_of(&piece)),
        json!({"expected_revision":revision_of(&piece),"template":"paper",
            "slides":[THOUGHT,"Review the exact bytes."]}),
    )
    .await;
    assert_eq!(result.status, 200, "{}", result.raw);
    (server, result.body)
}

fn identity(piece: &Value) -> Value {
    json!({"expected_revision":piece["revision"],"expected_assets":{
        "image_sha256":piece["assets"]["image_sha256"],
        "carousel_sha256":piece["assets"]["carousel_sha256"]}})
}

#[tokio::test]
async fn real_render_assessment_binds_bytes_without_claiming_visual_judgement() {
    let (server, piece) = rendered().await;
    let path = format!("/v1/pieces/{}/visual-quality", id_of(&piece));
    let report = post(&server, &path, identity(&piece)).await;
    assert_eq!(report.status, 200, "{}", report.raw);
    assert_eq!(report.body["revision"], piece["revision"]);
    assert_eq!(report.body["assets"], identity(&piece)["expected_assets"]);
    assert_eq!(report.body["technical_status"], "satisfied");
    assert_eq!(report.body["semantic_quality"], "unassessed");
    assert!(report.body["quality_score"].is_null());
    assert_eq!(report.body["assessor"], "local-technical-v1");
    assert_eq!(report.body["checks"].as_array().map(Vec::len), Some(5));
    let repeat = post(&server, &path, identity(&piece)).await;
    assert_eq!(repeat.body, report.body);
    let unchanged = get(&server, &format!("/v1/pieces/{}", id_of(&piece))).await;
    for field in [
        "revision",
        "status",
        "review_decision_id",
        "publish_status",
        "assets",
    ] {
        assert_eq!(unchanged.body[field], piece[field]);
    }
}

#[tokio::test]
async fn old_preview_hash_never_returns_replacement_bytes() {
    let (server, piece) = rendered().await;
    let replacement = post(&server, &format!("/v1/pieces/{}/render", id_of(&piece)),
        json!({"expected_revision":piece["revision"],"template":"midnight","slides":["A distinct replacement render."]})).await;
    assert_eq!(replacement.status, 200);
    for kind in ["image", "carousel"] {
        let key = format!("{kind}_sha256");
        let old_hash = piece["assets"][&key].as_str().unwrap();
        let new_hash = replacement.body["assets"][&key].as_str().unwrap();
        assert_ne!(old_hash, new_hash);
        for hash in [old_hash.to_owned(), "0".repeat(64)] {
            let path = format!("/v1/pieces/{}/assets/{kind}?sha256={hash}", id_of(&piece));
            let response = get(&server, &path).await;
            assert_eq!(response.status, 409, "{kind} must refuse stale evidence");
            assert_eq!(response.code(), "asset_conflict");
        }
        let response = client()
            .get(server.url(&format!(
                "/v1/pieces/{}/assets/{kind}?sha256={new_hash}",
                id_of(&piece)
            )))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        assert_eq!(
            syncloom_core::visual::digest(&response.bytes().await.unwrap()),
            new_hash
        );
    }
    assert_eq!(
        post(
            &server,
            &format!("/v1/pieces/{}/visual-quality", id_of(&piece)),
            identity(&piece)
        )
        .await
        .code(),
        "revision_conflict"
    );
    let page = get(&server, &format!("/review/{}", id_of(&piece))).await;
    assert!(page.raw.contains("?sha256="));
}

#[tokio::test]
async fn review_exposes_unknown_quality_and_recovers_from_damaged_assets() {
    let (server, piece) = rendered().await;
    let path = format!("/review/{}", id_of(&piece));
    let page = get(&server, &path).await;
    assert_eq!(page.status, 200);
    assert!(page.raw.contains("Visual quality: not assessed"));
    assert!(page.raw.contains("Local technical checks"));
    assert!(page.raw.contains("Not an approval"));
    std::fs::write(
        server
            .dir
            .path()
            .join("assets")
            .join(piece["assets"]["image_sha256"].as_str().unwrap()),
        b"damaged",
    )
    .unwrap();
    let report = post(
        &server,
        &format!("/v1/pieces/{}/visual-quality", id_of(&piece)),
        identity(&piece),
    )
    .await;
    assert_eq!(report.code(), "asset_integrity");
    let page = get(&server, &path).await;
    assert_eq!(page.status, 200);
    assert!(page.raw.contains("Could not inspect these previews"));
    assert!(page.raw.contains("Render new previews"));
    assert!(!page.raw.contains("Technical checks satisfied"));
    assert!(
        page.raw
            .contains(r#"<button type="submit" disabled>Approve this version</button>"#),
        "A known integrity failure must not offer approval that the core will refuse"
    );
}

#[tokio::test]
async fn assessment_rejects_stale_input_and_uses_existing_security_boundary() {
    let (server, piece) = rendered().await;
    let path = format!("/v1/pieces/{}/visual-quality", id_of(&piece));
    let mut stale = identity(&piece);
    stale["expected_revision"] = json!("stale");
    assert_eq!(
        post(&server, &path, stale).await.code(),
        "revision_conflict"
    );
    let mut stale = identity(&piece);
    stale["expected_assets"]["image_sha256"] = json!("0".repeat(64));
    assert_eq!(post(&server, &path, stale).await.code(), "asset_conflict");
    let mut extra = identity(&piece);
    extra["approve"] = json!(true);
    assert_eq!(post(&server, &path, extra).await.code(), "invalid_request");
    let no_session = answer(
        client()
            .post(server.url(&path))
            .json(&identity(&piece))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(no_session.code(), "session_required");
    for (header, value, code) in [
        ("host", "evil.invalid", "host_not_allowed"),
        ("origin", "https://evil.invalid", "origin_not_allowed"),
        ("sec-fetch-site", "cross-site", "origin_not_allowed"),
    ] {
        let response = answer(
            client()
                .post(server.url(&path))
                .header(header, value)
                .header("x-syncloom-token", &server.token)
                .json(&identity(&piece))
                .send()
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(response.code(), code);
        assert!(
            !response
                .headers
                .keys()
                .any(|h| h.as_str().starts_with("access-control-"))
        );
    }
    let edited = patch(
        &server,
        &format!("/v1/pieces/{}", id_of(&piece)),
        json!({"expected_revision":piece["revision"],"text":"A different saved draft."}),
    )
    .await;
    assert_eq!(edited.status, 200);
    assert_eq!(
        post(&server, &path, identity(&piece)).await.code(),
        "revision_conflict"
    );
    assert_eq!(post(&server, &path, json!({"expected_revision":edited.body["revision"],"expected_assets":identity(&piece)["expected_assets"]})).await.code(), "assets_required");
}
