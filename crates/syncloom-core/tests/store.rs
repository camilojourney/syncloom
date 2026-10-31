//! The file store: durable, atomic, append-only, locked (draft-review, dispatch-outbox,
//! provenance-log "Durable" and "Append-only" requirements).
mod support;
use std::time::{Duration, Instant};
use support::FIXED_AT;
use syncloom_core::{
    Channel, DispatchIntent, Event, EventKind, Fit, IntentStatus, Payload, Piece, Source, Status,
    Store, Verdict, outbox,
};
use tempfile::TempDir;

fn piece(text: &str) -> Piece {
    Piece {
        piece_id: "thought-0123456789ab-linkedin_text".into(),
        group_id: "0123456789abcdef0123456789abcdef".into(),
        channel: Channel::LinkedinText,
        platform: "linkedin".into(),
        text: text.into(),
        revision: "b0ca8c02601b2edcb612b47c88494925c5e5f9d92044e91bc510adf47dd73f92".into(),
        status: Status::PendingReview,
        review_decision_id: None,
        publish_status: None,
        visual: None,
        assets: None,
        routing: None,
        fit: Fit {
            verdict: Verdict::Review,
            char_count: 11,
            min_chars: 20,
            max_chars: 3000,
        },
        source: Source {
            kind: "text".into(),
            raw_input: "hello world".into(),
        },
        created_at: FIXED_AT.into(),
        updated_at: FIXED_AT.into(),
    }
}

fn event(kind: EventKind) -> Event {
    Event {
        schema_version: "1.0".into(),
        event_id: "11111111-1111-4111-8111-111111111111".into(),
        event: kind,
        at: FIXED_AT.into(),
        group_id: "0123456789abcdef0123456789abcdef".into(),
        piece_id: "thought-0123456789ab-linkedin_text".into(),
        channel: Channel::LinkedinText,
        revision: "b0ca8c02601b2edcb612b47c88494925c5e5f9d92044e91bc510adf47dd73f92".into(),
        status: Status::PendingReview,
        usage: None,
        request_id: None,
        outcome: None,
        post_id: None,
        image_sha256: None,
        carousel_sha256: None,
    }
}

// draft-review: Durable pieces, restart
#[test]
fn a_piece_survives_reopening_the_store() {
    let dir = TempDir::new().unwrap();
    {
        let store = Store::open(dir.path()).unwrap();
        store
            .lock()
            .unwrap()
            .save_piece(&piece("hello world"))
            .unwrap();
    }
    let store = Store::open(dir.path()).unwrap();
    let loaded = store
        .lock()
        .unwrap()
        .load_piece("thought-0123456789ab-linkedin_text")
        .unwrap();
    assert_eq!(loaded, Some(piece("hello world")));
}

// draft-review: Durable pieces, a second write replaces the first whole
#[test]
fn saving_again_replaces_the_record_and_leaves_no_temporary_file() {
    use std::os::unix::fs::MetadataExt;
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path()).unwrap();
    let record = dir
        .path()
        .join("pieces")
        .join("thought-0123456789ab-linkedin_text.json");
    store.lock().unwrap().save_piece(&piece("first")).unwrap();
    let first_inode = std::fs::metadata(&record).unwrap().ino();
    store.lock().unwrap().save_piece(&piece("second")).unwrap();
    // A replaced record is a new file moved over the old one, never the old file rewritten
    // in place: that is what keeps a reader from seeing a torn record.
    assert_ne!(
        std::fs::metadata(&record).unwrap().ino(),
        first_inode,
        "the record was rewritten in place instead of replaced by rename"
    );
    let loaded = store
        .lock()
        .unwrap()
        .load_piece("thought-0123456789ab-linkedin_text")
        .unwrap()
        .unwrap();
    assert_eq!(loaded.text, "second");
    assert_eq!(
        support::files(&dir, "pieces"),
        vec!["thought-0123456789ab-linkedin_text.json".to_owned()]
    );
}

// draft-review: Leftover temporary file
#[test]
fn a_leftover_temporary_file_is_never_read_as_the_record() {
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path()).unwrap();
    store
        .lock()
        .unwrap()
        .save_piece(&piece("whole record"))
        .unwrap();
    std::fs::write(
        dir.path()
            .join("pieces")
            .join(".thought-0123456789ab-linkedin_text.json.abc123.tmp"),
        b"{\"piece_id\": \"thought-0123456789ab-linkedin_text\", \"text\": \"torn",
    )
    .unwrap();
    let loaded = store
        .lock()
        .unwrap()
        .load_piece("thought-0123456789ab-linkedin_text")
        .unwrap()
        .unwrap();
    assert_eq!(loaded.text, "whole record");
}

#[test]
fn an_unknown_piece_loads_as_none() {
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path()).unwrap();
    assert_eq!(
        store
            .lock()
            .unwrap()
            .load_piece("thought-000000000000-linkedin_text")
            .unwrap(),
        None
    );
}

// dispatch-outbox: Durable intents
#[test]
fn an_intent_survives_reopening_the_store() {
    let dir = TempDir::new().unwrap();
    let key = outbox::intent_key(
        "publish",
        "thought-0123456789ab-linkedin_text",
        "b0ca8c02601b2edcb612b47c88494925c5e5f9d92044e91bc510adf47dd73f92",
    );
    let intent = DispatchIntent {
        request_id: "dispatch-904f4c443cf556d7f34c6c307a86dfe3".into(),
        operation: "publish".into(),
        piece_id: "thought-0123456789ab-linkedin_text".into(),
        revision: "b0ca8c02601b2edcb612b47c88494925c5e5f9d92044e91bc510adf47dd73f92".into(),
        payload: Payload {
            content: "hello world".into(),
            platforms: vec!["linkedin".into()],
            style: "raw".into(),
        },
        status: IntentStatus::Contained,
        created_at: FIXED_AT.into(),
        attempts: 0,
        attempted_at: None,
        finished_at: None,
        post_id: None,
        post_url: None,
        cost_estimate: None,
    };
    {
        let store = Store::open(dir.path()).unwrap();
        let lock = store.lock().unwrap();
        assert_eq!(lock.load_intent(&key).unwrap(), None);
        lock.save_intent(&key, &intent).unwrap();
    }
    let store = Store::open(dir.path()).unwrap();
    assert_eq!(
        store.lock().unwrap().load_intent(&key).unwrap(),
        Some(intent)
    );
    assert_eq!(support::files(&dir, "outbox"), vec![format!("{key}.json")]);
}

// provenance-log: Append-only across restarts
#[test]
fn reopening_and_appending_leaves_earlier_lines_byte_identical() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("provenance").join("events.jsonl");
    {
        let store = Store::open(dir.path()).unwrap();
        let lock = store.lock().unwrap();
        lock.append_event(&event(EventKind::PieceCreated)).unwrap();
        lock.append_event(&event(EventKind::PieceReviewed)).unwrap();
    }
    let before = std::fs::read_to_string(&path).unwrap();
    assert_eq!(before.lines().count(), 2);
    {
        let store = Store::open(dir.path()).unwrap();
        store
            .lock()
            .unwrap()
            .append_event(&event(EventKind::IntentReserved))
            .unwrap();
    }
    let after = std::fs::read_to_string(&path).unwrap();
    assert!(after.starts_with(&before), "earlier lines were rewritten");
    assert_eq!(after.lines().count(), 3);
    let last: serde_json::Value = serde_json::from_str(after.lines().last().unwrap()).unwrap();
    assert_eq!(last["event"], "intent_reserved");
    assert_eq!(last["schema_version"], "1.0");
    assert!(
        last.get("usage").is_none(),
        "absent usage is omitted, not null"
    );
}

// store: one exclusive lock across store handles (cross-process safety for CLI plus server)
#[test]
fn the_lock_is_exclusive_across_store_handles() {
    let dir = TempDir::new().unwrap();
    let first = Store::open(dir.path()).unwrap();
    let second = Store::open(dir.path()).unwrap();
    let held = first.lock().unwrap();
    let started = Instant::now();
    let waiter = std::thread::spawn(move || {
        let _lock = second.lock().unwrap();
        Instant::now()
    });
    std::thread::sleep(Duration::from_millis(300));
    drop(held);
    let acquired = waiter.join().unwrap();
    assert!(
        acquired.duration_since(started) >= Duration::from_millis(250),
        "the second lock was taken while the first was held"
    );
}
