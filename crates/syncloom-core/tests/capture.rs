//! Capture suggestion, non-approval preview and confirm-to-draft.
#![expect(
    clippy::unwrap_used,
    reason = "Tests fail immediately on unexpected fixture state."
)]
mod support;
use support::scripted::{Reply, Scripted, text};
use support::{fast, studio};
use syncloom_core::{CaptureConfirmRequest, Channel, Status, StudioError, preview, suggest};
use tempfile::TempDir;

const SHORT: &str = "Ship the small fix today.";
const URL_TEXT: &str = "Read this https://example.invalid/note today.";

fn channels_of(text: &str) -> Vec<String> {
    suggest(text)
        .unwrap()
        .into_iter()
        .map(|item| item.channel.name().to_owned())
        .collect()
}

fn assert_route(
    text: &str,
    index: usize,
    channel: Channel,
    confidence: &str,
    format_hint: &str,
    edit_notes: &str,
) {
    let item = &suggest(text).unwrap()[index];
    assert_eq!(item.channel, channel);
    assert_eq!(item.platform, channel.platform());
    assert_eq!(item.confidence, confidence);
    assert_eq!(item.format_hint, format_hint);
    assert_eq!(item.edit_notes, edit_notes);
}

#[test]
fn short_plain_text_uses_the_migrated_table() {
    assert_route(
        SHORT,
        0,
        Channel::X,
        "0.86",
        "single post",
        "Keep as one punchy line; no thread needed.",
    );
    assert_route(
        SHORT,
        1,
        Channel::Threads,
        "0.80",
        "short conversation post",
        "Optional: add a question to invite replies.",
    );
    assert_route(
        SHORT,
        2,
        Channel::LinkedinText,
        "0.78",
        "authority text post",
        "Open with the lesson, then one concrete example.",
    );
    assert_route(
        SHORT,
        3,
        Channel::FacebookPage,
        "0.60",
        "community post",
        "Soften the tone; sound like a note to friends.",
    );
    assert_eq!(suggest(SHORT).unwrap().len(), 4);
}

#[test]
fn plain_text_boundaries_do_not_add_a_carousel() {
    let at = |n: usize| "a".repeat(n);
    let names = |n: usize| channels_of(&at(n));
    let linkedin = |n: usize| {
        suggest(&at(n))
            .unwrap()
            .into_iter()
            .find(|item| item.channel == Channel::LinkedinText)
            .unwrap()
            .confidence
            .to_owned()
    };
    assert_eq!(linkedin(200), "0.78");
    assert_eq!(linkedin(201), "0.90");
    assert!(names(280).contains(&"x".to_owned()));
    assert!(names(280).contains(&"threads".to_owned()));
    for n in [281, 401] {
        let found = names(n);
        assert!(!found.contains(&"x".to_owned()), "{n}");
        assert!(!found.contains(&"threads".to_owned()), "{n}");
        assert!(found.contains(&"facebook_page".to_owned()), "{n}");
        assert_eq!(
            suggest(&at(n))
                .unwrap()
                .iter()
                .find(|item| item.channel == Channel::FacebookPage)
                .unwrap()
                .confidence,
            "0.60"
        );
        assert!(
            found.iter().all(|name| !name.contains("carousel")),
            "{found:?}"
        );
    }
}

#[test]
fn url_hint_does_not_fetch_and_skips_facebook() {
    assert_route(
        URL_TEXT,
        0,
        Channel::LinkedinText,
        "0.88",
        "text post with link",
        "Put the opinion first; drop the URL at the end.",
    );
    assert_route(
        URL_TEXT,
        1,
        Channel::X,
        "0.80",
        "short post or thread seed",
        "Compress to one sharp claim; link at the end if needed.",
    );
    assert_route(
        URL_TEXT,
        2,
        Channel::Threads,
        "0.72",
        "conversation starter",
        "Ask one follow-up question after the link takeaway.",
    );
    assert_eq!(suggest(URL_TEXT).unwrap().len(), 3);
}

#[test]
fn short_text_is_refused_before_suggestion() {
    let error = suggest("a b c d e f g").unwrap_err();
    assert_eq!(error, StudioError::ThoughtTooShort);
}

#[test]
fn preview_revision_is_the_channel_set_not_an_approval() {
    let forward = preview(SHORT, &["x".into(), "threads".into()]).unwrap();
    let reverse = preview(SHORT, &["threads".into(), "x".into()]).unwrap();
    assert_eq!(forward.revision, reverse.revision);
    assert_eq!(forward.revision.len(), 64);
    assert!(
        forward
            .revision
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    );
    assert_ne!(
        preview(&format!("{SHORT}!"), &["x".into()])
            .unwrap()
            .revision,
        preview(SHORT, &["x".into()]).unwrap().revision
    );
    assert_eq!(forward.text, SHORT);
    assert!(forward.approval.is_none());
    assert!(forward.publish_status.is_none());
    assert_eq!(forward.text, SHORT, "founder text is not truncated");
    assert_eq!(
        preview(SHORT, &["instagram".into()]).unwrap_err(),
        StudioError::InvalidRequest
    );
    assert_eq!(
        preview(SHORT, &["tiktok_video".into()]).unwrap_err(),
        StudioError::UnsupportedChannel
    );
    assert_eq!(
        preview(SHORT, &[]).unwrap_err(),
        StudioError::InvalidRequest
    );
}

#[test]
fn preview_revision_frames_text_separately_from_channel_names() {
    let source = "Founder observation";
    let first = preview(source, &["facebook_page".into(), "threads".into()]).unwrap();
    let injected = preview(&format!("{source}\nfacebook_page"), &["threads".into()]).unwrap();
    assert_ne!(first.revision, injected.revision);
}

#[tokio::test]
async fn confirm_creates_pending_drafts_once_and_rejects_a_stale_revision() {
    let dir = TempDir::new().unwrap();
    let provider = Scripted::new(text("  A pending draft from the founder text.  "));
    let studio = studio(&dir, provider.clone(), fast());
    let shown = preview(SHORT, &["linkedin_text".into()]).unwrap();
    let stale = studio
        .confirm_capture(CaptureConfirmRequest {
            source_revision: None,
            text: SHORT.into(),
            channels: vec!["linkedin_text".into()],
            expected_revision: "a".repeat(64),
        })
        .await;
    assert_eq!(stale.unwrap_err(), StudioError::RevisionConflict);
    assert!(provider.requests().is_empty());
    assert!(support::files(&dir, "pieces").is_empty());

    let request = CaptureConfirmRequest {
        source_revision: None,
        text: SHORT.into(),
        channels: vec!["linkedin_text".into()],
        expected_revision: shown.revision,
    };
    let created = studio.confirm_capture(request.clone()).await.unwrap();
    assert!(created.created);
    assert_eq!(created.thought.pieces.len(), 1);
    let piece = &created.thought.pieces[0];
    assert_eq!(piece.status, Status::PendingReview);
    assert!(piece.review_decision_id.is_none());
    assert!(piece.publish_status.is_none());
    assert_eq!(piece.source.raw_input, SHORT);
    assert_eq!(piece.text, "A pending draft from the founder text.");
    assert!(support::files(&dir, "outbox").is_empty());
    assert!(
        support::events(&dir)
            .iter()
            .all(|event| event["event"] == "piece_created")
    );

    let repeated = studio.confirm_capture(request.clone()).await.unwrap();
    assert!(!repeated.created);
    assert_eq!(repeated.thought.group_id, created.thought.group_id);
    assert_eq!(
        repeated.thought.pieces[0].piece_id,
        created.thought.pieces[0].piece_id
    );
    assert_eq!(provider.requests().len(), 1);
    let reopened = support::studio(&dir, provider.clone(), fast());
    let after_restart = reopened.confirm_capture(request).await.unwrap();
    assert!(!after_restart.created);
    assert_eq!(after_restart.thought.group_id, created.thought.group_id);
    assert_eq!(provider.requests().len(), 1);

    provider.set(Reply::Fail);
    let failed = studio
        .confirm_capture(CaptureConfirmRequest {
            source_revision: None,
            text: format!("{SHORT} again"),
            channels: vec!["linkedin_text".into()],
            expected_revision: preview(&format!("{SHORT} again"), &["linkedin_text".into()])
                .unwrap()
                .revision,
        })
        .await;
    assert_eq!(failed.unwrap_err(), StudioError::ProviderError);
    assert_eq!(support::files(&dir, "pieces").len(), 1);
    assert_eq!(
        support::files(&dir, "capture").len(),
        1,
        "failed provider releases its claim"
    );
    provider.set(text("Retry succeeds without approving anything."));
    let retry_source = format!("{SHORT} again");
    let retry = studio
        .confirm_capture(CaptureConfirmRequest {
            source_revision: None,
            text: retry_source.clone(),
            channels: vec!["linkedin_text".into()],
            expected_revision: preview(&retry_source, &["linkedin_text".into()])
                .unwrap()
                .revision,
        })
        .await
        .unwrap();
    assert!(retry.created);
    assert_eq!(support::files(&dir, "pieces").len(), 2);
}

#[tokio::test]
async fn incomplete_claim_refuses_another_provider_call() {
    let dir = TempDir::new().unwrap();
    let provider = Scripted::new(text("Must not be drafted."));
    let studio = studio(&dir, provider.clone(), fast());
    let preview = preview(SHORT, &["linkedin_text".into()]).unwrap();
    std::fs::write(
        dir.path()
            .join("capture")
            .join(format!("{}.json", preview.revision)),
        serde_json::to_vec(&serde_json::json!({
            "revision":preview.revision, "channels":["linkedin_text"], "group_id":"", "piece_ids":[]
        }))
        .unwrap(),
    )
    .unwrap();
    let error = studio
        .confirm_capture(CaptureConfirmRequest {
            source_revision: None,
            text: SHORT.into(),
            channels: vec!["linkedin_text".into()],
            expected_revision: preview.revision,
        })
        .await
        .unwrap_err();
    assert_eq!(error, StudioError::RevisionConflict);
    assert!(provider.requests().is_empty());
    assert!(support::files(&dir, "pieces").is_empty());
}
