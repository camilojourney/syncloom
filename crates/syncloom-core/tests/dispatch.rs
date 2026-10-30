//! Dispatch through the studio over a scripted platform (dispatch-outbox "Delivery grant",
//! "Attempt recorded before the request", "Outcome recorded truthfully", "Retry after a
//! retryable outcome", "Unknown is never retried blindly", "Platform deadline", "Repeat",
//! "Durable intents"; provenance-log "Dispatch events").
mod support;
use std::sync::Arc;
use support::scripted::{Scripted, text};
use support::scripted_platform::{Outcome, ScriptedPlatform, posted};
use support::{FIXED_AT, fast, fixed_clock, studio};
use syncloom_core::{
    IntentStatus, Piece, PostFailure, PublishRequest, ReviewRequest, Settings, Status, Store,
    Studio, StudioError,
};
use tempfile::TempDir;

const POST_ID: &str = "urn:li:share:7000000000000000001";
const POST_URL: &str = "https://www.linkedin.com/feed/update/urn:li:share:7000000000000000001/";

fn publish(revision: &str) -> PublishRequest {
    PublishRequest {
        expected_revision: revision.to_owned(),
        dry_run: false,
    }
}

async fn approved_with(dir: &TempDir, platform: Option<Arc<ScriptedPlatform>>) -> (Studio, Piece) {
    let mut studio = studio(dir, Scripted::new(text("A draft about zebrafish.")), fast());
    if let Some(platform) = platform {
        studio = studio.with_delivery("linkedin", platform);
    }
    let response = studio
        .create_thought(syncloom_core::ThoughtRequest {
            text: support::THOUGHT.to_owned(),
            channels: vec!["linkedin_text".to_owned()],
        })
        .await
        .unwrap();
    let piece = response.pieces.into_iter().next().unwrap();
    let piece = studio
        .review(
            &piece.piece_id,
            ReviewRequest {
                status: Some(Status::Approved),
                text: None,
                expected_revision: piece.revision.clone(),
            },
        )
        .unwrap();
    (studio, piece)
}

fn kinds(dir: &TempDir) -> Vec<String> {
    support::events(dir)
        .iter()
        .map(|e| e["event"].as_str().unwrap().to_owned())
        .collect()
}

// dispatch-outbox: Delivery grant, No grant
#[tokio::test]
async fn without_a_grant_the_intent_stays_contained_and_no_platform_is_called() {
    let dir = TempDir::new().unwrap();
    let (studio, piece) = approved_with(&dir, None).await;
    let response = studio
        .publish(&piece.piece_id, publish(&piece.revision))
        .await
        .unwrap();
    assert_eq!(response.status, "contained");
    assert_eq!(response.post_id, None);
    assert_eq!(response.post_url, None);
    assert_eq!(response.attempts, 0);
    let again = studio
        .publish(&piece.piece_id, publish(&piece.revision))
        .await
        .unwrap();
    assert_eq!(again.request_id, response.request_id);
    assert!(!again.created);
    assert_eq!(
        kinds(&dir),
        ["piece_created", "piece_reviewed", "intent_reserved"]
    );
}

// dispatch-outbox: LinkedIn granted, Outcome recorded truthfully (Posted); provenance-log: Dispatch events
#[tokio::test]
async fn with_a_grant_one_attempt_posts_and_records_the_platform_id() {
    let dir = TempDir::new().unwrap();
    let platform = ScriptedPlatform::new(posted());
    let (studio, piece) = approved_with(&dir, Some(platform.clone())).await;
    let response = studio
        .publish(&piece.piece_id, publish(&piece.revision))
        .await
        .unwrap();
    assert_eq!(response.status, "posted");
    assert!(response.created);
    assert_eq!(response.post_id.as_deref(), Some(POST_ID));
    assert_eq!(response.post_url.as_deref(), Some(POST_URL));
    assert_eq!(response.attempts, 1);
    assert_eq!(response.piece.publish_status.as_deref(), Some("posted"));
    assert_eq!(response.piece.status, Status::Approved);
    let requests = platform.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].text, "A draft about zebrafish.");
    assert_eq!(requests[0].request_id, response.request_id.clone().unwrap());

    let intent = support::intent(&dir);
    assert_eq!(intent["status"], "posted");
    assert_eq!(intent["attempts"], 1);
    assert_eq!(intent["attempted_at"], FIXED_AT);
    assert_eq!(intent["finished_at"], FIXED_AT);
    assert_eq!(intent["post_id"], POST_ID);
    assert_eq!(intent["post_url"], POST_URL);
    assert_eq!(studio.show(&piece.piece_id).unwrap(), response.piece);

    assert_eq!(
        kinds(&dir),
        [
            "piece_created",
            "piece_reviewed",
            "intent_reserved",
            "dispatch_attempted",
            "dispatch_finished"
        ]
    );
    let events = support::events(&dir);
    assert_eq!(
        events[3]["request_id"],
        response.request_id.clone().unwrap()
    );
    assert!(events[3].get("outcome").is_none());
    assert_eq!(events[4]["request_id"], response.request_id.unwrap());
    assert_eq!(events[4]["outcome"], "posted");
    assert_eq!(events[4]["post_id"], POST_ID);
}

// dispatch-outbox: Repeat returns the same intent, Second publish after posted
#[tokio::test]
async fn a_posted_intent_is_never_sent_again() {
    let dir = TempDir::new().unwrap();
    let platform = ScriptedPlatform::new(posted());
    let (studio, piece) = approved_with(&dir, Some(platform.clone())).await;
    let first = studio
        .publish(&piece.piece_id, publish(&piece.revision))
        .await
        .unwrap();
    let events_before = support::events(&dir).len();
    let second = studio
        .publish(&piece.piece_id, publish(&piece.revision))
        .await
        .unwrap();
    assert_eq!(second.status, "posted");
    assert!(!second.created);
    assert_eq!(second.request_id, first.request_id);
    assert_eq!(second.post_id, first.post_id);
    assert_eq!(second.attempts, 1);
    assert_eq!(
        platform.requests().len(),
        1,
        "a posted intent must not be sent again"
    );
    assert_eq!(
        support::events(&dir).len(),
        events_before,
        "a repeat adds no event"
    );
}

// dispatch-outbox: Outcome recorded truthfully (Failure)
#[tokio::test]
async fn each_platform_failure_is_recorded_as_its_own_status_without_a_post_id() {
    let cases = [
        (PostFailure::Refused, "refused"),
        (PostFailure::ConsentRequired, "consent_required"),
        (PostFailure::RateLimited, "rate_limited"),
        (PostFailure::Unreachable, "unreachable"),
        (PostFailure::Unknown, "unknown"),
    ];
    for (failure, status) in cases {
        let dir = TempDir::new().unwrap();
        let platform = ScriptedPlatform::new(Outcome::Fail(failure));
        let (studio, piece) = approved_with(&dir, Some(platform.clone())).await;
        let response = studio
            .publish(&piece.piece_id, publish(&piece.revision))
            .await
            .unwrap();
        assert_eq!(response.status, status, "{failure:?}");
        assert_eq!(response.post_id, None, "{failure:?}");
        assert_eq!(response.post_url, None, "{failure:?}");
        assert_eq!(response.attempts, 1);
        assert_eq!(response.piece.publish_status.as_deref(), Some(status));
        let intent = support::intent(&dir);
        assert_eq!(intent["status"], status);
        assert!(intent.get("post_id").is_none(), "{failure:?}: {intent}");
        let events = support::events(&dir);
        assert_eq!(events.last().unwrap()["outcome"], status);
        assert!(events.last().unwrap().get("post_id").is_none());
    }
}

// dispatch-outbox: Retry after a retryable outcome
#[tokio::test]
async fn a_retryable_failure_allows_exactly_one_new_attempt_on_the_next_publish() {
    let dir = TempDir::new().unwrap();
    let platform = ScriptedPlatform::new(Outcome::Fail(PostFailure::RateLimited));
    let (studio, piece) = approved_with(&dir, Some(platform.clone())).await;
    let first = studio
        .publish(&piece.piece_id, publish(&piece.revision))
        .await
        .unwrap();
    assert_eq!(first.status, "rate_limited");
    platform.set(posted());
    let second = studio
        .publish(&piece.piece_id, publish(&piece.revision))
        .await
        .unwrap();
    assert_eq!(second.status, "posted");
    assert_eq!(second.attempts, 2);
    assert!(!second.created);
    assert_eq!(second.request_id, first.request_id);
    assert_eq!(platform.requests().len(), 2);
    assert_eq!(support::intent(&dir)["attempts"], 2);
}

// dispatch-outbox: Attempt recorded before the request
#[tokio::test]
async fn the_intent_is_posting_while_the_platform_call_is_in_flight() {
    let dir = TempDir::new().unwrap();
    let platform = ScriptedPlatform::new(Outcome::Hold);
    let (studio, piece) = approved_with(&dir, Some(platform.clone())).await;
    let studio = Arc::new(studio);
    let publishing = {
        let studio = studio.clone();
        let (id, revision) = (piece.piece_id.clone(), piece.revision.clone());
        tokio::spawn(async move { studio.publish(&id, publish(&revision)).await })
    };
    // Wait until the platform has been asked, then look at the records as another process would.
    for _ in 0..200 {
        if !platform.requests().is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert_eq!(platform.requests().len(), 1);
    let intent = support::intent(&dir);
    assert_eq!(intent["status"], "posting");
    assert_eq!(intent["attempts"], 1);
    assert_eq!(intent["attempted_at"], FIXED_AT);
    assert_eq!(kinds(&dir).last().unwrap(), "dispatch_attempted");
    // The store lock is free during the call: another reader is not blocked.
    assert_eq!(
        studio
            .show(&piece.piece_id)
            .unwrap()
            .publish_status
            .as_deref(),
        Some("posting")
    );
    platform.release();
    let response = publishing.await.unwrap().unwrap();
    assert_eq!(response.status, "posted");
}

// dispatch-outbox: Unknown is never retried blindly
#[tokio::test]
async fn a_posting_or_unknown_intent_found_later_answers_unknown_and_sends_nothing() {
    let dir = TempDir::new().unwrap();
    let platform = ScriptedPlatform::new(posted());
    let (studio, piece) = approved_with(&dir, Some(platform.clone())).await;
    // Plant the record an interrupted attempt leaves behind.
    let key = syncloom_core::outbox::intent_key("publish", &piece.piece_id, &piece.revision);
    let intent = syncloom_core::DispatchIntent {
        request_id: syncloom_core::request_id("publish", &piece.piece_id, &piece.revision),
        operation: "publish".to_owned(),
        piece_id: piece.piece_id.clone(),
        revision: piece.revision.clone(),
        payload: syncloom_core::Payload {
            content: piece.text.clone(),
            platforms: vec!["linkedin".to_owned()],
            style: "raw".to_owned(),
        },
        status: IntentStatus::Posting,
        created_at: FIXED_AT.to_owned(),
        attempts: 1,
        attempted_at: Some(FIXED_AT.to_owned()),
        finished_at: None,
        post_id: None,
        post_url: None,
    };
    Store::open(dir.path())
        .unwrap()
        .lock()
        .unwrap()
        .save_intent(&key, &intent)
        .unwrap();

    let response = studio
        .publish(&piece.piece_id, publish(&piece.revision))
        .await
        .unwrap();
    assert_eq!(response.status, "unknown");
    assert!(!response.created);
    assert_eq!(response.attempts, 1);
    assert!(
        platform.requests().is_empty(),
        "an unknown outcome must not be retried blindly"
    );
    assert_eq!(support::intent(&dir)["status"], "unknown");
    let again = studio
        .publish(&piece.piece_id, publish(&piece.revision))
        .await
        .unwrap();
    assert_eq!(again.status, "unknown");
    assert!(platform.requests().is_empty());
    assert_eq!(response.piece.publish_status.as_deref(), Some("unknown"));
}

// dispatch-outbox: Platform deadline
#[tokio::test]
async fn a_hanging_platform_is_recorded_as_unknown_after_the_deadline() {
    let dir = TempDir::new().unwrap();
    let platform = ScriptedPlatform::new(Outcome::Hang);
    let (studio, piece) = approved_with(&dir, Some(platform.clone())).await;
    let response = studio
        .publish(&piece.piece_id, publish(&piece.revision))
        .await
        .unwrap();
    assert_eq!(response.status, "unknown");
    assert_eq!(platform.requests().len(), 1);
    assert_eq!(support::intent(&dir)["status"], "unknown");
    assert_eq!(kinds(&dir).last().unwrap(), "dispatch_finished");
}

// dispatch-outbox: Dry run with delivery granted
#[tokio::test]
async fn a_dry_run_with_a_grant_sends_nothing() {
    let dir = TempDir::new().unwrap();
    let platform = ScriptedPlatform::new(posted());
    let (studio, piece) = approved_with(&dir, Some(platform.clone())).await;
    let response = studio
        .publish(
            &piece.piece_id,
            PublishRequest {
                expected_revision: piece.revision.clone(),
                dry_run: true,
            },
        )
        .await
        .unwrap();
    assert_eq!(response.status, "dry_run");
    assert!(platform.requests().is_empty());
    assert!(support::files(&dir, "outbox").is_empty());
}

// dispatch-outbox: Approval required and Stale revision still guard a granted dispatch
#[tokio::test]
async fn a_grant_does_not_bypass_approval_or_revision_checks() {
    let dir = TempDir::new().unwrap();
    let platform = ScriptedPlatform::new(posted());
    let (studio, piece) = approved_with(&dir, Some(platform.clone())).await;
    assert_eq!(
        studio
            .publish(&piece.piece_id, publish(&"0".repeat(64)))
            .await
            .unwrap_err(),
        StudioError::RevisionConflict
    );
    let edited = studio
        .review(
            &piece.piece_id,
            ReviewRequest {
                status: None,
                text: Some("Edited after approval, still about zebrafish.".to_owned()),
                expected_revision: piece.revision.clone(),
            },
        )
        .unwrap();
    assert_eq!(
        studio
            .publish(&piece.piece_id, publish(&edited.revision))
            .await
            .unwrap_err(),
        StudioError::ApprovalRequired
    );
    assert!(platform.requests().is_empty());
    assert!(support::files(&dir, "outbox").is_empty());
}

// dispatch-outbox: Durable intents, Restart
#[tokio::test]
async fn a_posted_intent_survives_a_restart_and_is_not_sent_again() {
    let dir = TempDir::new().unwrap();
    let platform = ScriptedPlatform::new(posted());
    let (first, piece) = approved_with(&dir, Some(platform.clone())).await;
    let published = first
        .publish(&piece.piece_id, publish(&piece.revision))
        .await
        .unwrap();
    drop(first);
    let second = Studio::new(
        Store::open(dir.path()).unwrap(),
        Scripted::new(text("x")),
        fixed_clock(),
        Settings::default(),
    )
    .with_delivery("linkedin", platform.clone());
    let again = second
        .publish(&piece.piece_id, publish(&piece.revision))
        .await
        .unwrap();
    assert_eq!(again.request_id, published.request_id);
    assert_eq!(again.post_id.as_deref(), Some(POST_ID));
    assert!(!again.created);
    assert_eq!(platform.requests().len(), 1);
}

// dispatch-outbox: Durable intents, Slice 1 record
#[test]
fn a_slice_1_intent_without_attempt_fields_loads_with_zero_attempts() {
    let dir = TempDir::new().unwrap();
    let key = "0".repeat(64);
    std::fs::write(
        dir.path().join("outbox").join(format!("{key}.json")),
        r#"{"request_id":"dispatch-904f4c443cf556d7f34c6c307a86dfe3","operation":"publish","piece_id":"thought-0123456789ab-linkedin_text","revision":"b0ca8c02601b2edcb612b47c88494925c5e5f9d92044e91bc510adf47dd73f92","payload":{"content":"hello world","platforms":["linkedin"],"style":"raw"},"status":"contained","created_at":"2026-10-30T18:00:00Z"}"#,
    )
    .unwrap_or_else(|_| {
        std::fs::create_dir_all(dir.path().join("outbox")).unwrap();
        std::fs::write(
            dir.path().join("outbox").join(format!("{key}.json")),
            r#"{"request_id":"dispatch-904f4c443cf556d7f34c6c307a86dfe3","operation":"publish","piece_id":"thought-0123456789ab-linkedin_text","revision":"b0ca8c02601b2edcb612b47c88494925c5e5f9d92044e91bc510adf47dd73f92","payload":{"content":"hello world","platforms":["linkedin"],"style":"raw"},"status":"contained","created_at":"2026-10-30T18:00:00Z"}"#,
        )
        .unwrap()
    });
    let store = Store::open(dir.path()).unwrap();
    let intent = store.lock().unwrap().load_intent(&key).unwrap().unwrap();
    assert_eq!(intent.status, IntentStatus::Contained);
    assert_eq!(intent.attempts, 0);
    assert_eq!(intent.attempted_at, None);
    assert_eq!(intent.post_id, None);
}
