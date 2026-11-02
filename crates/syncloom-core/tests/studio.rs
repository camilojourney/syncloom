//! The studio use cases over the scripted provider: thought-drafting, draft-review,
//! dispatch-outbox and provenance-log requirements, through the same functions the API and
//! the CLI call.
mod support;
use support::scripted::{Reply, Scripted, text};
use support::{CANARY, FIXED_AT, NORMALIZED, THOUGHT, fast, studio};
use syncloom_core::{
    Channel, Piece, PublishRequest, ReviewRequest, Settings, Status, Store, Studio, StudioError,
    ThoughtRequest, Verdict,
};
use tempfile::TempDir;

fn thought(text: &str) -> ThoughtRequest {
    ThoughtRequest {
        text: text.to_owned(),
        channels: vec!["linkedin_text".to_owned()],
    }
}

fn status(status: Status, revision: &str) -> ReviewRequest {
    ReviewRequest {
        expected_target: None,
        status: Some(status),
        text: None,
        expected_revision: revision.to_owned(),
        expected_assets: None,
    }
}

fn edit(text: &str, revision: &str) -> ReviewRequest {
    ReviewRequest {
        expected_target: None,
        status: None,
        text: Some(text.to_owned()),
        expected_revision: revision.to_owned(),
        expected_assets: None,
    }
}

fn publish(revision: &str, dry_run: bool) -> PublishRequest {
    PublishRequest {
        expected_revision: revision.to_owned(),
        accepted_cost_microusd: None,
        dry_run,
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
async fn drafted(dir: &TempDir, reply: Reply) -> (Studio, Piece) {
    let studio = studio(dir, Scripted::new(reply), Settings::default());
    let response = studio.create_thought(thought(THOUGHT)).await.unwrap();
    assert_eq!(response.pieces.len(), 1);
    (studio, response.pieces.into_iter().next().unwrap())
}

#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
async fn approved(dir: &TempDir) -> (Studio, Piece) {
    let (studio, piece) = drafted(
        dir,
        text("A fine draft about one owner per rule; zebrafish."),
    )
    .await;
    let piece = studio
        .review(&piece.piece_id, status(Status::Approved, &piece.revision))
        .unwrap();
    (studio, piece)
}

fn nothing_stored(dir: &TempDir) {
    assert!(
        support::files(dir, "pieces").is_empty(),
        "a piece was stored"
    );
    assert!(
        support::files(dir, "outbox").is_empty(),
        "an intent was stored"
    );
    assert!(support::events(dir).is_empty(), "an event was appended");
}

// [itest->req~dispatch-outbox.current-publish-inspection~1]
#[tokio::test]
async fn publish_inspection_is_revision_scoped_read_only_and_fails_closed() {
    use syncloom_core::outbox::{IntentStatus, intent_key};
    let dir = TempDir::new().unwrap();
    let (studio, piece) = approved(&dir).await;
    assert_eq!(
        studio.inspect_publish("missing").unwrap_err(),
        StudioError::PieceNotFound
    );
    assert!(
        studio
            .inspect_publish(&piece.piece_id)
            .unwrap()
            .publish_intent
            .is_none()
    );
    studio
        .publish(&piece.piece_id, publish(&piece.revision, false))
        .await
        .unwrap();
    let key = intent_key("publish", &piece.piece_id, &piece.revision);
    let store = Store::open(dir.path()).unwrap();
    let mut intent = store.lock().unwrap().load_intent(&key).unwrap().unwrap();
    intent.status = IntentStatus::Posting;
    store.lock().unwrap().save_intent(&key, &intent).unwrap();
    let before = support::events(&dir);
    let bytes = std::fs::read(dir.path().join("outbox").join(format!("{key}.json"))).unwrap();
    let snapshot = studio.inspect_publish(&piece.piece_id).unwrap();
    assert_eq!(snapshot.piece.revision, piece.revision);
    assert_eq!(snapshot.approved_at.as_deref(), Some(FIXED_AT));
    assert_eq!(
        snapshot.publish_intent.unwrap(),
        intent,
        "GET must not normalize Posting to Unknown"
    );
    assert_eq!(support::events(&dir), before);
    assert_eq!(
        std::fs::read(dir.path().join("outbox").join(format!("{key}.json"))).unwrap(),
        bytes
    );
    let edited = studio
        .review(
            &piece.piece_id,
            edit("A new synthetic revision.", &piece.revision),
        )
        .unwrap();
    assert!(
        studio
            .inspect_publish(&edited.piece_id)
            .unwrap()
            .publish_intent
            .is_none()
    );
    let new_key = intent_key("publish", &edited.piece_id, &edited.revision);
    std::fs::write(
        dir.path().join("outbox").join(format!("{new_key}.json")),
        b"corrupt",
    )
    .unwrap();
    assert_eq!(
        studio.inspect_publish(&edited.piece_id).unwrap_err(),
        StudioError::Internal
    );
}

#[tokio::test]
async fn approval_time_comes_from_matching_history_not_later_activity_or_a_guessed_date() {
    let dir = TempDir::new().unwrap();
    let (studio, piece) = approved(&dir).await;
    let log = dir.path().join("provenance/events.jsonl");
    let original = std::fs::read(&log).unwrap();
    let mut other = support::events(&dir).last().unwrap().clone();
    other["event"] = serde_json::json!("piece_exported");
    other["at"] = serde_json::json!("2026-11-03T19:00:00Z");
    let store = Store::open(dir.path()).unwrap();
    store
        .lock()
        .unwrap()
        .append_event(&serde_json::from_value(other.clone()).unwrap())
        .unwrap();
    other["event"] = serde_json::json!("piece_reviewed");
    other["revision"] = serde_json::json!("another-revision");
    store
        .lock()
        .unwrap()
        .append_event(&serde_json::from_value(other).unwrap())
        .unwrap();
    let before = std::fs::read(&log).unwrap();
    assert_eq!(
        studio
            .inspect_publish(&piece.piece_id)
            .unwrap()
            .approved_at
            .as_deref(),
        Some(FIXED_AT)
    );
    assert_eq!(
        std::fs::read(&log).unwrap(),
        before,
        "date lookup never writes history"
    );
    std::fs::remove_file(&log).unwrap();
    assert!(
        studio
            .inspect_publish(&piece.piece_id)
            .unwrap()
            .approved_at
            .is_none(),
        "legacy records without history do not get invented dates"
    );
    std::fs::write(&log, b"not-json\n").unwrap();
    assert_eq!(
        studio.inspect_publish(&piece.piece_id).unwrap_err(),
        StudioError::Internal
    );
    std::fs::write(&log, original).unwrap();
    let edited = studio
        .review(
            &piece.piece_id,
            edit("A changed saved version.", &piece.revision),
        )
        .unwrap();
    assert!(
        studio
            .inspect_publish(&edited.piece_id)
            .unwrap()
            .approved_at
            .is_none()
    );
}

// thought-drafting: Create pieces from a thought
#[tokio::test]
async fn a_thought_becomes_one_pending_linkedin_piece() {
    let dir = TempDir::new().unwrap();
    let provider = Scripted::new(text("  A draft about the thought.  "));
    let studio = studio(&dir, provider.clone(), Settings::default());
    let response = studio.create_thought(thought(THOUGHT)).await.unwrap();
    assert_eq!(response.group_id.len(), 32);
    assert!(response.group_id.chars().all(|c| c.is_ascii_hexdigit()));
    let piece = &response.pieces[0];
    assert_eq!(
        piece.piece_id,
        format!("thought-{}-linkedin_text", &response.group_id[..12])
    );
    assert_eq!(piece.group_id, response.group_id);
    assert_eq!(piece.channel, Channel::LinkedinText);
    assert_eq!(piece.platform, "linkedin");
    assert_eq!(piece.status, Status::PendingReview);
    assert_eq!(piece.text, "A draft about the thought.");
    assert_eq!(piece.source.kind, "text");
    assert_eq!(piece.source.raw_input, THOUGHT);
    assert_eq!(piece.review_decision_id, None);
    assert_eq!(piece.publish_status, None);
    assert_eq!(piece.revision.len(), 64);
    assert_eq!(piece.created_at, FIXED_AT);
    assert_eq!(piece.updated_at, piece.created_at);
    assert_eq!(piece.fit.char_count, 26);
    assert_eq!(piece.fit.verdict, Verdict::Pass);
}

// thought-drafting: Thought normalization and Generation boundary
#[tokio::test]
async fn the_provider_receives_the_normalized_thought_as_data_once() {
    let dir = TempDir::new().unwrap();
    let provider = Scripted::new(text("A draft."));
    let studio = studio(&dir, provider.clone(), Settings::default());
    studio.create_thought(thought(THOUGHT)).await.unwrap();
    let requests = provider.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].channel, Channel::LinkedinText);
    assert_eq!(requests[0].thought, NORMALIZED);
}

// thought-drafting: Thought too short, both sides of the boundary
#[tokio::test]
async fn seven_non_space_characters_are_too_short_and_eight_are_enough() {
    let dir = TempDir::new().unwrap();
    let provider = Scripted::new(text("A draft."));
    let studio = studio(&dir, provider.clone(), Settings::default());
    let error = studio
        .create_thought(thought("a b c d e f g"))
        .await
        .unwrap_err();
    assert_eq!(error, StudioError::ThoughtTooShort);
    assert_eq!(
        provider.requests().len(),
        0,
        "the provider must not be called"
    );
    nothing_stored(&dir);
    let response = studio.create_thought(thought("a b c d e f g h")).await;
    assert!(response.is_ok(), "{response:?}");
}

// thought-drafting: Unsupported channel, Repeated channel; studio-api: Invalid request body
#[tokio::test]
async fn unknown_repeated_or_missing_channels_are_refused_before_drafting() {
    let dir = TempDir::new().unwrap();
    let provider = Scripted::new(text("A draft."));
    let studio = studio(&dir, provider.clone(), Settings::default());
    let with = |channels: &[&str]| ThoughtRequest {
        text: THOUGHT.to_owned(),
        channels: channels.iter().map(|c| (*c).to_owned()).collect(),
    };
    assert_eq!(
        studio
            .create_thought(with(&["linkedin_text", "tiktok_video"]))
            .await
            .unwrap_err(),
        StudioError::UnsupportedChannel
    );
    assert_eq!(
        studio
            .create_thought(with(&["linkedin_text", "linkedin_text"]))
            .await
            .unwrap_err(),
        StudioError::InvalidRequest
    );
    assert_eq!(
        studio.create_thought(with(&[])).await.unwrap_err(),
        StudioError::InvalidRequest
    );
    assert_eq!(provider.requests().len(), 0);
    nothing_stored(&dir);
}

// thought-drafting: Provider failures persist nothing
#[tokio::test]
async fn provider_failures_map_to_their_codes_and_persist_nothing() {
    let cases = [
        (Reply::Empty, StudioError::ProviderError),
        (Reply::Fail, StudioError::ProviderError),
        (Reply::Hang, StudioError::ProviderTimeout),
        (Reply::Unconfigured, StudioError::ProviderNotConfigured),
    ];
    for (reply, expected) in cases {
        let dir = TempDir::new().unwrap();
        let studio = studio(&dir, Scripted::new(reply.clone()), fast());
        let error = studio.create_thought(thought(THOUGHT)).await.unwrap_err();
        assert_eq!(error, expected, "{reply:?}");
        nothing_stored(&dir);
    }
}

// thought-drafting: Token usage recorded; provenance-log: Event shape for piece_created
#[tokio::test]
async fn the_created_event_carries_ids_hashes_status_and_usage() {
    let dir = TempDir::new().unwrap();
    let (_studio, piece) = drafted(&dir, text("A draft.")).await;
    let events = support::events(&dir);
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert_eq!(event["event"], "piece_created");
    assert_eq!(event["schema_version"], "1.0");
    assert_eq!(event["at"], FIXED_AT);
    assert_eq!(event["group_id"], piece.group_id);
    assert_eq!(event["piece_id"], piece.piece_id);
    assert_eq!(event["channel"], "linkedin_text");
    assert_eq!(event["revision"], piece.revision);
    assert_eq!(event["status"], "pending_review");
    assert_eq!(event["usage"]["input_tokens"], 321);
    assert_eq!(event["usage"]["output_tokens"], 123);
    let id = event["event_id"].as_str().unwrap();
    assert_eq!(id.len(), 36);
    assert_eq!(id.matches('-').count(), 4);
}

// draft-review: Show a piece
#[tokio::test]
async fn show_returns_the_created_piece_and_not_found_for_unknown_or_unsafe_ids() {
    let dir = TempDir::new().unwrap();
    let (studio, piece) = drafted(&dir, text("A draft.")).await;
    assert_eq!(studio.show(&piece.piece_id).unwrap(), piece);
    assert_eq!(
        studio
            .show("thought-000000000000-linkedin_text")
            .unwrap_err(),
        StudioError::PieceNotFound
    );
    // A well-formed piece record outside the pieces directory, plus the one inside it: an id
    // that escaped the directory would load one of these instead of being refused.
    let planted = serde_json::to_string(&piece).unwrap();
    std::fs::write(dir.path().join("evil.json"), &planted).unwrap();
    std::fs::write(dir.path().join("outbox").join("evil.json"), &planted).unwrap();
    for unsafe_id in [
        "../evil",
        "../outbox/evil",
        "a/b",
        "thought-..-x",
        "",
        &"a".repeat(129),
        "piece id",
    ] {
        assert_eq!(
            studio.show(unsafe_id).unwrap_err(),
            StudioError::PieceNotFound,
            "{unsafe_id:?}"
        );
    }
}

// draft-review: Status transitions, Approval bound to a revision
#[tokio::test]
async fn allowed_transitions_move_the_piece_and_approval_records_the_decision_id() {
    let dir = TempDir::new().unwrap();
    let (studio, piece) = drafted(&dir, text("A draft.")).await;
    let approved = studio
        .review(&piece.piece_id, status(Status::Approved, &piece.revision))
        .unwrap();
    assert_eq!(approved.status, Status::Approved);
    assert_eq!(
        approved.review_decision_id.as_deref(),
        Some(format!("review-{}-{}", piece.piece_id, &piece.revision[..16]).as_str())
    );
    assert_eq!(approved.revision, piece.revision);
    assert!(approved.updated_at >= approved.created_at);
    let same = studio
        .review(&piece.piece_id, status(Status::Approved, &piece.revision))
        .unwrap();
    assert_eq!(same, approved, "repeating the status changes nothing");
    let rejected = studio
        .review(&piece.piece_id, status(Status::Rejected, &piece.revision))
        .unwrap();
    assert_eq!(rejected.status, Status::Rejected);
    let pending = studio
        .review(
            &piece.piece_id,
            status(Status::PendingReview, &piece.revision),
        )
        .unwrap();
    assert_eq!(pending.status, Status::PendingReview);
}

// draft-review: Refused transitions
#[tokio::test]
async fn refused_transitions_leave_the_piece_unchanged() {
    let dir = TempDir::new().unwrap();
    let (studio, piece) = approved(&dir).await;
    let error = studio
        .review(
            &piece.piece_id,
            status(Status::PendingReview, &piece.revision),
        )
        .unwrap_err();
    assert_eq!(error, StudioError::InvalidStatusTransition);
    assert_eq!(studio.show(&piece.piece_id).unwrap(), piece);
    let rejected = studio
        .review(&piece.piece_id, status(Status::Rejected, &piece.revision))
        .unwrap();
    let error = studio
        .review(&piece.piece_id, status(Status::Approved, &piece.revision))
        .unwrap_err();
    assert_eq!(error, StudioError::InvalidStatusTransition);
    assert_eq!(studio.show(&piece.piece_id).unwrap(), rejected);
}

// draft-review: Stale revision refused
#[tokio::test]
async fn a_stale_revision_is_refused_and_changes_nothing() {
    let dir = TempDir::new().unwrap();
    let (studio, piece) = drafted(&dir, text("A draft.")).await;
    let stale = "0".repeat(64);
    let error = studio
        .review(&piece.piece_id, status(Status::Approved, &stale))
        .unwrap_err();
    assert_eq!(error, StudioError::RevisionConflict);
    let error = studio
        .review(
            &piece.piece_id,
            edit("New text for the piece, long enough.", &stale),
        )
        .unwrap_err();
    assert_eq!(error, StudioError::RevisionConflict);
    assert_eq!(studio.show(&piece.piece_id).unwrap(), piece);
    assert_eq!(
        support::events(&dir).len(),
        1,
        "refused requests add no event"
    );
}

// draft-review: Edit creates a revision and voids approval; platform-rules: Fit follows the text
#[tokio::test]
async fn an_edit_voids_approval_and_recomputes_revision_and_fit() {
    let dir = TempDir::new().unwrap();
    let (studio, piece) = approved(&dir).await;
    let long = "y".repeat(3001);
    let edited = studio
        .review(
            &piece.piece_id,
            edit(&format!("  {long}  "), &piece.revision),
        )
        .unwrap();
    assert_eq!(edited.text, long);
    assert_ne!(edited.revision, piece.revision);
    assert_eq!(edited.status, Status::PendingReview);
    assert_eq!(edited.review_decision_id, None);
    assert_eq!(edited.fit.verdict, Verdict::Review);
    assert_eq!(edited.fit.char_count, 3001);
    let error = studio
        .publish(&piece.piece_id, publish(&edited.revision, false))
        .await
        .unwrap_err();
    assert_eq!(error, StudioError::ApprovalRequired);
}

// draft-review: Edit body rules
#[tokio::test]
async fn a_review_needs_exactly_one_of_status_or_text_and_text_cannot_be_blank() {
    let dir = TempDir::new().unwrap();
    let (studio, piece) = drafted(&dir, text("A draft.")).await;
    let both = ReviewRequest {
        expected_target: None,
        status: Some(Status::Approved),
        text: Some("x".into()),
        expected_revision: piece.revision.clone(),
        expected_assets: None,
    };
    let neither = ReviewRequest {
        expected_target: None,
        status: None,
        text: None,
        expected_revision: piece.revision.clone(),
        expected_assets: None,
    };
    for request in [both, neither, edit("   \n", &piece.revision)] {
        assert_eq!(
            studio.review(&piece.piece_id, request).unwrap_err(),
            StudioError::InvalidRequest
        );
    }
    assert_eq!(studio.show(&piece.piece_id).unwrap(), piece);
}

// dispatch-outbox: Dry run records nothing (allowed on an unapproved piece)
#[tokio::test]
async fn a_dry_run_returns_the_payload_and_records_nothing() {
    let dir = TempDir::new().unwrap();
    let (studio, piece) = drafted(&dir, text("A draft.")).await;
    let response = studio
        .publish(&piece.piece_id, publish(&piece.revision, true))
        .await
        .unwrap();
    assert_eq!(response.status, "dry_run");
    assert_eq!(response.request_id, None);
    assert!(!response.created);
    assert_eq!(response.payload.content, "A draft.");
    assert_eq!(response.payload.platforms, vec!["linkedin".to_owned()]);
    assert_eq!(response.payload.style, "raw");
    assert_eq!(response.piece, piece);
    assert!(support::files(&dir, "outbox").is_empty());
    assert_eq!(support::events(&dir).len(), 1);
    assert_eq!(studio.show(&piece.piece_id).unwrap(), piece);
}

// dispatch-outbox: Approval required
#[tokio::test]
async fn publishing_an_unapproved_piece_is_refused_without_an_intent() {
    let dir = TempDir::new().unwrap();
    let (studio, piece) = drafted(&dir, text("A draft.")).await;
    let error = studio
        .publish(&piece.piece_id, publish(&piece.revision, false))
        .await
        .unwrap_err();
    assert_eq!(error, StudioError::ApprovalRequired);
    studio
        .review(&piece.piece_id, status(Status::Rejected, &piece.revision))
        .unwrap();
    let error = studio
        .publish(&piece.piece_id, publish(&piece.revision, false))
        .await
        .unwrap_err();
    assert_eq!(error, StudioError::ApprovalRequired);
    assert!(support::files(&dir, "outbox").is_empty());
}

// dispatch-outbox: Stale revision refused on publish, dry run too
#[tokio::test]
async fn a_stale_revision_is_refused_on_publish_and_on_dry_run() {
    let dir = TempDir::new().unwrap();
    let (studio, piece) = approved(&dir).await;
    let stale = "0".repeat(64);
    for dry_run in [false, true] {
        let error = studio
            .publish(&piece.piece_id, publish(&stale, dry_run))
            .await
            .unwrap_err();
        assert_eq!(error, StudioError::RevisionConflict);
    }
    assert!(support::files(&dir, "outbox").is_empty());
    assert_eq!(
        studio
            .publish(
                "thought-000000000000-linkedin_text",
                publish(&piece.revision, true)
            )
            .await
            .unwrap_err(),
        StudioError::PieceNotFound
    );
}

// dispatch-outbox: Contained intent, Repeat returns the same intent; provenance-log: one event
#[tokio::test]
async fn a_publish_reserves_one_contained_intent_and_a_repeat_returns_it() {
    let dir = TempDir::new().unwrap();
    let (studio, piece) = approved(&dir).await;
    let first = studio
        .publish(&piece.piece_id, publish(&piece.revision, false))
        .await
        .unwrap();
    assert_eq!(first.status, "contained");
    assert!(first.created);
    let request_id = first.request_id.clone().expect("a request id");
    assert!(request_id.starts_with("dispatch-"));
    assert_eq!(request_id.len(), "dispatch-".len() + 32);
    assert!(
        request_id["dispatch-".len()..]
            .chars()
            .all(|c| c.is_ascii_hexdigit())
    );
    assert_eq!(first.piece.publish_status.as_deref(), Some("contained"));
    assert_eq!(first.piece.status, Status::Approved);
    assert_eq!(first.payload.content, piece.text);
    assert_eq!(support::files(&dir, "outbox").len(), 1);
    let events = support::events(&dir);
    assert_eq!(events.len(), 3);
    assert_eq!(events[2]["event"], "intent_reserved");
    assert_eq!(events[2]["request_id"], request_id);
    assert_eq!(events[2]["status"], "approved");

    let second = studio
        .publish(&piece.piece_id, publish(&piece.revision, false))
        .await
        .unwrap();
    assert_eq!(second.request_id.as_deref(), Some(request_id.as_str()));
    assert!(!second.created);
    assert_eq!(second.status, "contained");
    assert_eq!(support::files(&dir, "outbox").len(), 1);
    assert_eq!(support::events(&dir).len(), 3, "a repeat adds no event");

    let stored: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            dir.path()
                .join("outbox")
                .join(&support::files(&dir, "outbox")[0]),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(stored["request_id"], request_id);
    assert_eq!(stored["operation"], "publish");
    assert_eq!(stored["piece_id"], piece.piece_id);
    assert_eq!(stored["revision"], piece.revision);
    assert_eq!(stored["status"], "contained");
    assert_eq!(stored["created_at"], FIXED_AT);
    assert_eq!(stored["payload"]["content"], piece.text);
}

// provenance-log: One event per state change, in order; No content in events
#[tokio::test]
async fn every_state_change_appends_one_event_in_order_without_content() {
    let dir = TempDir::new().unwrap();
    let (studio, piece) = drafted(&dir, text("A draft that mentions zebrafish too.")).await;
    let id = piece.piece_id.clone();
    let approved = studio
        .review(&id, status(Status::Approved, &piece.revision))
        .unwrap();
    let edited = studio
        .review(
            &id,
            edit(
                "An edited draft, still about zebrafish.",
                &approved.revision,
            ),
        )
        .unwrap();
    let approved_again = studio
        .review(&id, status(Status::Approved, &edited.revision))
        .unwrap();
    studio
        .publish(&id, publish(&approved_again.revision, true))
        .await
        .unwrap();
    studio
        .publish(&id, publish(&approved_again.revision, false))
        .await
        .unwrap();
    studio
        .publish(&id, publish(&approved_again.revision, false))
        .await
        .unwrap();
    let events = support::events(&dir);
    let kinds: Vec<&str> = events
        .iter()
        .map(|e| e["event"].as_str().unwrap())
        .collect();
    assert_eq!(
        kinds,
        [
            "piece_created",
            "piece_reviewed",
            "piece_edited",
            "piece_reviewed",
            "intent_reserved"
        ]
    );
    assert_eq!(events[1]["revision"], piece.revision);
    assert_eq!(events[1]["status"], "approved");
    assert_eq!(events[2]["revision"], edited.revision);
    assert_eq!(events[2]["status"], "pending_review");
    let raw = std::fs::read_to_string(dir.path().join("provenance").join("events.jsonl")).unwrap();
    assert!(
        !raw.contains(CANARY),
        "thought or draft text leaked into the log"
    );
}

// draft-review and dispatch-outbox: Durable across a restart
#[tokio::test]
async fn pieces_and_intents_survive_reopening_the_studio() {
    let dir = TempDir::new().unwrap();
    let (first, piece) = approved(&dir).await;
    let published = first
        .publish(&piece.piece_id, publish(&piece.revision, false))
        .await
        .unwrap();
    drop(first);
    let second = Studio::new(
        Store::open(dir.path()).unwrap(),
        Scripted::new(Reply::Fail),
        support::fixed_clock(),
        Settings::default(),
    );
    assert_eq!(second.show(&piece.piece_id).unwrap(), published.piece);
    let again = second
        .publish(&piece.piece_id, publish(&piece.revision, false))
        .await
        .unwrap();
    assert_eq!(again.request_id, published.request_id);
    assert!(!again.created);
}

// provenance-log: Event failure is reported, the piece stays
#[cfg(unix)]
#[tokio::test]
async fn a_failed_event_append_is_an_internal_error_and_the_piece_stays() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new().unwrap();
    let studio = studio(&dir, Scripted::new(text("A draft.")), Settings::default());
    let provenance = dir.path().join("provenance");
    std::fs::create_dir_all(&provenance).unwrap();
    std::fs::set_permissions(&provenance, std::fs::Permissions::from_mode(0o555)).unwrap();
    let result = studio.create_thought(thought(THOUGHT)).await;
    std::fs::set_permissions(&provenance, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(result.unwrap_err(), StudioError::Internal);
    let pieces = support::files(&dir, "pieces");
    assert_eq!(
        pieces.len(),
        1,
        "the piece write preceded the failed append"
    );
    let id = pieces[0].trim_end_matches(".json");
    assert_eq!(studio.show(id).unwrap().text, "A draft.");
}
