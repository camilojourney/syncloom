mod support;
use serde_json::json;
use support::{scripted::Scripted, *};

#[tokio::test]
async fn render_view_approve_edit_requires_new_assets() {
    let server = start(Scripted::new(scripted::text("Build with evidence."))).await;
    let original = piece_of(&create(&server, THOUGHT).await);
    let id = id_of(&original);
    let render_path = format!("/v1/pieces/{id}/render");
    let rendered = post(
        &server,
        &render_path,
        json!({
            "expected_revision": revision_of(&original), "template": "midnight",
            "slides": ["Build with evidence.", "Review the exact bytes."]
        }),
    )
    .await;
    assert_eq!(rendered.status, 200, "{}", rendered.raw);
    let piece = rendered.body;
    assert_ne!(revision_of(&piece), revision_of(&original));
    assert_eq!(piece["assets"]["revision"], piece["revision"]);
    for (kind, mime, hash) in [
        ("image", "image/png", &piece["assets"]["image_sha256"]),
        (
            "carousel",
            "application/pdf",
            &piece["assets"]["carousel_sha256"],
        ),
    ] {
        let response = client()
            .get(server.url(&format!("/v1/pieces/{id}/assets/{kind}")))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        assert_eq!(response.headers()["content-type"], mime);
        assert_eq!(
            response.headers()["etag"],
            format!("\"{}\"", hash.as_str().unwrap())
        );
        assert_eq!(response.headers()["cache-control"], "no-store");
        assert!(!response.bytes().await.unwrap().is_empty());
    }
    assert_eq!(approve(&server, &piece).await.status, 200);
    let edited = patch(
        &server,
        &format!("/v1/pieces/{id}"),
        json!({
            "expected_revision": revision_of(&piece), "text": "A new piece of text."
        }),
    )
    .await;
    assert_eq!(edited.status, 200);
    assert!(edited.body["assets"].is_null());
    assert_eq!(
        approve(&server, &edited.body).await.code(),
        "assets_required"
    );
    let viewed = get(&server, &format!("/v1/pieces/{id}/assets/image")).await;
    assert_eq!(viewed.code(), "assets_required");
    let stale = post(
        &server,
        &render_path,
        json!({
            "expected_revision": revision_of(&piece), "template": "midnight", "slides": ["Stale"]
        }),
    )
    .await;
    assert_eq!(stale.code(), "revision_conflict");
    let events = events(&server.dir);
    for name in ["visual_configured", "assets_rendered", "assets_viewed"] {
        assert!(events.iter().any(|e| e["event"] == name));
    }
    let approved = events
        .iter()
        .find(|e| e["event"] == "piece_reviewed")
        .unwrap();
    assert_eq!(approved["image_sha256"], piece["assets"]["image_sha256"]);
    assert!(
        !serde_json::to_string(&events)
            .unwrap()
            .contains("Review the exact")
    );
}

#[tokio::test]
async fn invalid_visual_input_and_host_are_refused() {
    let server = start(Scripted::new(scripted::text("Build with evidence."))).await;
    let piece = piece_of(&create(&server, THOUGHT).await);
    let path = format!("/v1/pieces/{}/render", id_of(&piece));
    for (template, slides) in [
        ("midnight", vec![]),
        ("midnight", vec![""; 1]),
        ("midnight", vec!["slide"; 11]),
        ("unknown", vec!["slide"]),
    ] {
        assert_eq!(
            post(
                &server,
                &path,
                json!({"expected_revision": revision_of(&piece),
            "template": template, "slides": slides})
            )
            .await
            .code(),
            "invalid_request"
        );
    }
    let response = client()
        .get(server.url(&format!("/v1/pieces/{}/assets/image", id_of(&piece))))
        .header("Host", "evil.example")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 403);
    assert_eq!(get(&server, "/v1/pieces/../assets/image").await.status, 404);
}
