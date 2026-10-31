mod support;

use serde_json::json;
use support::{get, post, scripted::Scripted, start};

#[tokio::test]
async fn founder_can_draft_each_social_channel_and_preview_cost_without_delivery() {
    let server = start(Scripted::new(support::scripted::text(
        "A small approved test post.",
    )))
    .await;
    for channel in ["bluesky", "x", "threads"] {
        let draft = post(
            &server,
            "/v1/thoughts",
            json!({"text":"Build only after checking the official API.","channels":[channel]}),
        )
        .await;
        assert_eq!(draft.status, 201, "{channel}: {}", draft.raw);
        let piece = &draft.body["pieces"][0];
        let preview = post(
            &server,
            &format!("/v1/pieces/{}/publish", piece["piece_id"].as_str().unwrap()),
            json!({"expected_revision":piece["revision"],"dry_run":true}),
        )
        .await;
        assert_eq!(preview.status, 200, "{}", preview.raw);
        assert_eq!(preview.body["status"], "dry_run");
        if channel == "x" {
            assert_eq!(preview.body["cost_estimate"]["microusd"], 15000);
            assert_eq!(preview.body["cost_estimate"]["currency"], "USD");
        }
        let connection = get(&server, &format!("/v1/{channel}/connection")).await;
        assert_ne!(connection.status, 404, "connection route must exist");
    }
}
