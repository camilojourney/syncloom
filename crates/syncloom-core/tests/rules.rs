//! Pure rules: platform-rules, revision, request id, transitions, normalization, config values.
//! Expected values come from the specs' worked examples, never from the code's formulas.
use std::collections::BTreeSet;
use syncloom_core::{
    Channel, StudioError, Verdict, clock, data_dir, fit, normalize_thought, outbox, piece_id,
    provider::{DEFAULT_MODEL, model_id, provider_base},
    request_id, review_decision_id, revision, usable_key,
};
use syncloom_core::{Status, Status::*};
use time::macros::datetime;

const PIECE: &str = "thought-0123456789ab-linkedin_text";
const REVISION: &str = "b0ca8c02601b2edcb612b47c88494925c5e5f9d92044e91bc510adf47dd73f92";

// platform-rules: Channel table
#[test]
fn the_channel_table_knows_linkedin_text_and_nothing_else() {
    let channel = Channel::parse("linkedin_text").expect("linkedin_text is in the table");
    assert_eq!(channel.platform(), "linkedin");
    assert_eq!(
        (channel.bounds().min_chars, channel.bounds().max_chars),
        (20, 3000)
    );
    for unknown in [
        "linkedin",
        "LinkedIn_Text",
        "linkedin_text ",
        "",
        "tiktok_video",
    ] {
        assert_eq!(Channel::parse(unknown), None, "{unknown:?} must be unknown");
    }
}

// platform-rules: Platform fit, upper and lower bound, both sides
#[test]
fn fit_passes_inside_the_bounds_and_reviews_outside_them() {
    let at = |n: usize| fit(Channel::LinkedinText, &"x".repeat(n));
    assert_eq!(at(3000).verdict, Verdict::Pass);
    assert_eq!(at(3000).char_count, 3000);
    assert_eq!((at(3000).min_chars, at(3000).max_chars), (20, 3000));
    assert_eq!(at(3001).verdict, Verdict::Review);
    assert_eq!(at(3001).char_count, 3001);
    assert_eq!(at(20).verdict, Verdict::Pass);
    assert_eq!(at(19).verdict, Verdict::Review);
}

// platform-rules: Characters are Unicode scalar values
#[test]
fn fit_counts_unicode_scalar_values_not_bytes() {
    let text = format!("{}{}", "é".repeat(1500), "日".repeat(1500));
    let result = fit(Channel::LinkedinText, &text);
    assert_eq!(result.char_count, 3000);
    assert_eq!(result.verdict, Verdict::Pass);
}

// draft-review: Revision worked example
#[test]
fn the_revision_is_the_sha256_of_the_sorted_compact_json() {
    assert_eq!(
        revision(Channel::LinkedinText, PIECE, "hello world"),
        REVISION
    );
    assert_ne!(
        revision(Channel::LinkedinText, PIECE, "hello world!"),
        REVISION
    );
}

// draft-review: Approval bound to a revision
#[test]
fn the_review_decision_id_names_the_piece_and_the_revision_prefix() {
    assert_eq!(
        review_decision_id(PIECE, REVISION),
        "review-thought-0123456789ab-linkedin_text-b0ca8c02601b2edc"
    );
}

// thought-drafting: piece id shape
#[test]
fn the_piece_id_keeps_the_old_shape() {
    assert_eq!(
        piece_id("0123456789abcdef0123456789abcdef", Channel::LinkedinText),
        PIECE
    );
}

// dispatch-outbox: Request id is deterministic (worked example)
#[test]
fn the_request_id_is_dispatch_plus_32_hex_of_the_intent_key() {
    assert_eq!(
        request_id(outbox::OPERATION_PUBLISH, PIECE, REVISION),
        "dispatch-904f4c443cf556d7f34c6c307a86dfe3"
    );
    assert_eq!(
        outbox::intent_key(outbox::OPERATION_PUBLISH, PIECE, REVISION).len(),
        64
    );
}

// draft-review: Status transitions (the retained table, both sides)
#[test]
fn status_transitions_follow_the_retained_table() {
    let allowed = [
        (PendingReview, Approved),
        (PendingReview, Rejected),
        (Approved, Rejected),
        (Rejected, PendingReview),
    ];
    for (from, to) in allowed {
        assert!(from.can_move_to(to), "{from:?} -> {to:?} must be allowed");
    }
    for status in [PendingReview, Approved, Rejected] {
        assert!(status.can_move_to(status), "same status is a no-op");
    }
    let refused = [(Approved, PendingReview), (Rejected, Approved)];
    for (from, to) in refused {
        assert!(!from.can_move_to(to), "{from:?} -> {to:?} must be refused");
    }
    let _: Status = Approved;
}

// thought-drafting: Thought normalization
#[test]
fn normalization_trims_and_collapses_whitespace() {
    assert_eq!(
        normalize_thought("  one\n\n two   three  "),
        "one two three"
    );
    assert_eq!(normalize_thought("\t\n"), "");
}

// studio-api: one error table, codes unique, messages fixed and content-free
#[test]
fn the_error_table_has_the_documented_codes_with_non_empty_messages() {
    let expected: BTreeSet<&str> = [
        "invalid_request",
        "thought_too_short",
        "unsupported_channel",
        "piece_not_found",
        "invalid_status_transition",
        "revision_conflict",
        "approval_required",
        "provider_not_configured",
        "provider_error",
        "provider_timeout",
        "internal_error",
    ]
    .into_iter()
    .collect();
    let actual: BTreeSet<&str> = StudioError::ALL.iter().map(|e| e.code()).collect();
    assert_eq!(actual, expected);
    for error in StudioError::ALL {
        assert!(
            error.message().ends_with('.'),
            "{error:?} message is a sentence"
        );
    }
    assert_eq!(
        StudioError::ThoughtTooShort.message(),
        "The thought needs at least 8 characters that are not spaces."
    );
}

// provenance-log: `at` is RFC 3339 UTC ending in Z
#[test]
fn instants_format_as_rfc3339_utc_with_z() {
    assert_eq!(
        clock::rfc3339(datetime!(2026-10-30 18:01:24 UTC)),
        "2026-10-30T18:01:24Z"
    );
}

// thought-drafting: Claude provider, key handling and model pin
#[test]
fn a_key_is_usable_only_when_visible_ascii_and_not_a_comment() {
    assert_eq!(usable_key("  sk-ant-abc  ").as_deref(), Some("sk-ant-abc"));
    assert_eq!(usable_key(""), None);
    assert_eq!(usable_key("   "), None);
    assert_eq!(usable_key("# no key here"), None);
    assert_eq!(usable_key("sk-\nant"), None);
    assert_eq!(usable_key("sk-ént"), None);
}

#[test]
fn the_model_id_is_pinned_with_an_environment_override() {
    assert_eq!(model_id(None), DEFAULT_MODEL);
    assert_eq!(model_id(None), "claude-sonnet-5-5");
    assert_eq!(model_id(Some("  ")), DEFAULT_MODEL);
    assert_eq!(model_id(Some("claude-other-1")), "claude-other-1");
}

// thought-drafting: Base URL
#[test]
fn the_provider_base_is_https_or_loopback_http_with_a_port() {
    assert_eq!(
        provider_base(None).unwrap().as_str(),
        "https://api.anthropic.com"
    );
    assert_eq!(
        provider_base(Some("https://api.anthropic.com/"))
            .unwrap()
            .as_str(),
        "https://api.anthropic.com"
    );
    assert_eq!(
        provider_base(Some("http://127.0.0.1:4321"))
            .unwrap()
            .as_str(),
        "http://127.0.0.1:4321"
    );
    for refused in [
        "http://example.com",
        "http://127.0.0.1",
        "http://localhost:4321",
        "ftp://x",
        "https://user:pw@api.anthropic.com",
        "https://api.anthropic.com/?x=1",
        "https://api.anthropic.com/#frag",
        "not a url",
    ] {
        let error = provider_base(Some(refused)).expect_err(refused);
        assert!(
            error.contains("SYNCLOOM_CLAUDE_BASE_URL"),
            "{refused}: {error}"
        );
    }
}

// studio-cli: Data directory
#[test]
fn the_data_dir_is_the_variable_or_home_dot_syncloom() {
    assert_eq!(
        data_dir(Some("/tmp/x"), Some("/Users/me")).unwrap(),
        std::path::PathBuf::from("/tmp/x")
    );
    assert_eq!(
        data_dir(None, Some("/Users/me")).unwrap(),
        std::path::PathBuf::from("/Users/me/.syncloom")
    );
    assert!(data_dir(None, None).is_err());
}
