//! Pure rules: platform-rules, revision, request id, transitions, normalization, config values.
//! Expected values come from the specs' worked examples, never from the code's formulas.
use std::collections::BTreeSet;
use syncloom_core::{
    AuthFlow, BaseUrl, Channel, IntentStatus, StudioError, Verdict, clock, data_dir,
    delivery_grant, fit, keychain_service, normalize_thought, outbox, piece_id, post_url,
    provider::{DEFAULT_MODEL, model_id, provider_base},
    redirect_url, request_id, review_decision_id, revision, usable_key,
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
        "linkedin_not_configured",
        "sign_in_in_progress",
        "sign_in_not_started",
        "sign_in_denied",
        "sign_in_state_mismatch",
        "sign_in_timed_out",
        "sign_in_rejected",
        "renderer_missing",
        "renderer_version_mismatch",
        "renderer_font_mismatch",
        "renderer_timeout",
        "renderer_failed",
        "assets_required",
        "asset_integrity",
        "asset_conflict",
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

// linkedin-integration: LinkedIn configuration, Redirect URL rule
#[test]
fn the_redirect_url_is_loopback_http_with_a_port_and_a_path() {
    let default = redirect_url(None).unwrap();
    assert_eq!(default.as_str(), "http://127.0.0.1:8791/linkedin/callback");
    assert_eq!(default.bind_addr().to_string(), "127.0.0.1:8791");
    assert_eq!(default.path(), "/linkedin/callback");
    let localhost = redirect_url(Some("http://localhost:8791/linkedin/callback")).unwrap();
    assert_eq!(
        localhost.as_str(),
        "http://localhost:8791/linkedin/callback"
    );
    assert_eq!(localhost.bind_addr().to_string(), "127.0.0.1:8791");
    for refused in [
        "https://127.0.0.1:8791/cb",
        "http://127.0.0.1/cb",
        "http://192.168.1.1:8791/cb",
        "http://127.0.0.1:8791/cb?x=1",
        "http://127.0.0.1:8791/cb#f",
        "http://u:p@127.0.0.1:8791/cb",
        "not a url",
    ] {
        let error = redirect_url(Some(refused)).expect_err(refused);
        assert!(
            error.contains("SYNCLOOM_LINKEDIN_REDIRECT_URL"),
            "{refused}: {error}"
        );
    }
}

// linkedin-integration: Flow and base URL rules
#[test]
fn the_auth_flow_is_standard_by_default_or_native_pkce() {
    assert_eq!(AuthFlow::parse(None).unwrap(), AuthFlow::Standard);
    assert_eq!(AuthFlow::parse(Some("  ")).unwrap(), AuthFlow::Standard);
    assert_eq!(
        AuthFlow::parse(Some("standard")).unwrap(),
        AuthFlow::Standard
    );
    assert_eq!(
        AuthFlow::parse(Some("native_pkce")).unwrap(),
        AuthFlow::NativePkce
    );
    for refused in ["pkce", "Standard", "device_code"] {
        let error = AuthFlow::parse(Some(refused)).expect_err(refused);
        assert!(error.contains("SYNCLOOM_LINKEDIN_AUTH_FLOW"), "{error}");
    }
}

#[test]
fn a_base_url_is_https_or_loopback_http_and_names_its_variable_when_refused() {
    assert_eq!(
        BaseUrl::parse(
            "SYNCLOOM_LINKEDIN_API_BASE_URL",
            None,
            "https://api.linkedin.com"
        )
        .unwrap()
        .as_str(),
        "https://api.linkedin.com"
    );
    assert_eq!(
        BaseUrl::parse("X", Some("http://127.0.0.1:4321/"), "https://x")
            .unwrap()
            .as_str(),
        "http://127.0.0.1:4321"
    );
    for refused in [
        "http://localhost:4321",
        "http://example.com",
        "https://a:b@x.y",
        "https://x.y/?q",
    ] {
        let error = BaseUrl::parse(
            "SYNCLOOM_LINKEDIN_AUTH_BASE_URL",
            Some(refused),
            "https://x",
        )
        .expect_err(refused);
        assert!(
            error.contains("SYNCLOOM_LINKEDIN_AUTH_BASE_URL"),
            "{refused}: {error}"
        );
    }
}

// dispatch-outbox: Delivery grant (the pure rule)
#[test]
fn the_delivery_grant_names_known_platforms_or_nothing() {
    assert!(delivery_grant(None).unwrap().is_empty());
    assert!(delivery_grant(Some("  ")).unwrap().is_empty());
    let granted = delivery_grant(Some(" linkedin ")).unwrap();
    assert_eq!(granted.iter().collect::<Vec<_>>(), ["linkedin"]);
    for refused in ["linkedin,tiktok", "instagram", "LinkedIn"] {
        let error = delivery_grant(Some(refused)).expect_err(refused);
        assert!(error.contains("SYNCLOOM_DELIVERY"), "{refused}: {error}");
    }
}

// platform-credentials: the service name
#[test]
fn the_keychain_service_defaults_to_syncloom_with_an_override() {
    assert_eq!(keychain_service(None), "syncloom");
    assert_eq!(keychain_service(Some(" ")), "syncloom");
    assert_eq!(keychain_service(Some("syncloom-test-1")), "syncloom-test-1");
}

// linkedin-integration: Posted with id and URL (the URL rule)
#[test]
fn the_post_url_is_the_feed_update_of_the_urn() {
    assert_eq!(
        post_url("urn:li:share:7000000000000000001"),
        "https://www.linkedin.com/feed/update/urn:li:share:7000000000000000001/"
    );
}

// dispatch-outbox: which statuses allow a new attempt
#[test]
fn only_contained_and_retryable_failures_allow_a_new_attempt() {
    use IntentStatus::*;
    for status in [
        Contained,
        Refused,
        ConsentRequired,
        RateLimited,
        Unreachable,
    ] {
        assert!(status.allows_new_attempt(), "{status:?}");
    }
    for status in [Posting, Posted, Unknown] {
        assert!(!status.allows_new_attempt(), "{status:?}");
    }
    assert_eq!(Posted.as_str(), "posted");
    assert_eq!(ConsentRequired.as_str(), "consent_required");
    assert_eq!(
        serde_json::to_string(&RateLimited).unwrap(),
        "\"rate_limited\""
    );
}
