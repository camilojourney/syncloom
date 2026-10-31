//! The Claude adapter against a fake Messages endpoint (thought-drafting "Claude provider").
mod support;
use support::fake_claude::{Mode, start};
use syncloom_core::{
    Channel, DraftRequest, ModelProvider, ProviderError, Usage,
    provider::{ClaudeProvider, DEFAULT_MODEL, provider_base},
};

fn request() -> DraftRequest {
    DraftRequest {
        channel: Channel::LinkedinText,
        thought: support::NORMALIZED.to_owned(),
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
fn provider(base: &str, key: Option<&str>, model: &str) -> ClaudeProvider {
    ClaudeProvider::new(
        key.map(str::to_owned),
        provider_base(Some(base)).unwrap(),
        model.to_owned(),
    )
}

// thought-drafting: Claude provider, Request shape
#[tokio::test]
async fn the_request_carries_the_pinned_model_the_instruction_and_the_thought_as_data() {
    let fake = start(Mode::Ok("A draft.".into())).await;
    let draft = provider(&fake.base_url(), Some("sk-fake"), DEFAULT_MODEL)
        .draft(request())
        .await
        .unwrap();
    assert_eq!(draft.text, "A draft.");
    assert_eq!(
        draft.usage,
        Some(Usage {
            input_tokens: 321,
            output_tokens: 123
        })
    );
    let recorded = fake.requests();
    assert_eq!(recorded.len(), 1);
    let sent = &recorded[0];
    assert!(sent.has_api_key);
    assert_eq!(sent.api_key_is.as_deref(), Some("sk-fake"));
    assert_eq!(sent.anthropic_version.as_deref(), Some("2023-06-01"));
    assert_eq!(sent.body["model"], "claude-sonnet-5-5");
    assert_eq!(sent.body["max_tokens"], 1200);
    assert!(sent.body["system"].as_str().is_some_and(|s| !s.is_empty()));
    assert_eq!(sent.body["messages"].as_array().map(Vec::len), Some(1));
    assert_eq!(sent.body["messages"][0]["role"], "user");
    assert_eq!(sent.body["messages"][0]["content"], support::NORMALIZED);
    assert!(
        !sent.body["system"]
            .as_str()
            .unwrap()
            .contains(support::CANARY),
        "the thought must not be inside the system instruction"
    );
}

#[tokio::test]
async fn every_requested_channel_reaches_the_model_without_thought_text_in_the_instruction() {
    let fake = start(Mode::Ok("A draft.".into())).await;
    let claude = provider(&fake.base_url(), Some("sk-fake"), DEFAULT_MODEL);
    for (channel, name, limit) in [
        (Channel::LinkedinText, "linkedin_text", 3000),
        (Channel::Mastodon, "mastodon", 500),
        (Channel::ManualExport, "manual_export", 40000),
    ] {
        claude
            .draft(DraftRequest {
                channel,
                thought: support::NORMALIZED.into(),
            })
            .await
            .unwrap();
        let calls = fake.requests();
        let sent = &calls.last().unwrap().body;
        let system = sent["system"].as_str().unwrap();
        assert!(
            system.contains(name),
            "channel missing from instruction: {system}"
        );
        assert!(system.contains(&limit.to_string()));
        assert!(!system.contains(support::CANARY));
        assert_eq!(sent["messages"][0]["content"], support::NORMALIZED);
    }
}

// thought-drafting: Claude provider, Pinned model with override
#[tokio::test]
async fn the_model_override_is_sent_when_configured() {
    let fake = start(Mode::Ok("A draft.".into())).await;
    provider(&fake.base_url(), Some("sk-fake"), "claude-other-1")
        .draft(request())
        .await
        .unwrap();
    assert_eq!(fake.requests()[0].body["model"], "claude-other-1");
}

// thought-drafting: Provider failures, No key: nothing is sent
#[tokio::test]
async fn no_usable_key_is_unconfigured_without_a_request() {
    let fake = start(Mode::Ok("A draft.".into())).await;
    for key in [None, Some(""), Some("   "), Some("# comment")] {
        let result = provider(&fake.base_url(), key, DEFAULT_MODEL)
            .draft(request())
            .await;
        assert_eq!(result, Err(ProviderError::Unconfigured), "{key:?}");
    }
    assert_eq!(fake.requests().len(), 0);
}

// thought-drafting: Claude provider, Provider statuses
#[tokio::test]
async fn rejected_keys_are_unconfigured_and_other_failures_are_rejected() {
    let fake = start(Mode::Status(401)).await;
    let claude = provider(&fake.base_url(), Some("sk-fake"), DEFAULT_MODEL);
    assert_eq!(
        claude.draft(request()).await,
        Err(ProviderError::Unconfigured)
    );
    fake.set(Mode::Status(403));
    assert_eq!(
        claude.draft(request()).await,
        Err(ProviderError::Unconfigured)
    );
    for status in [429, 500, 400] {
        fake.set(Mode::Status(status));
        assert_eq!(
            claude.draft(request()).await,
            Err(ProviderError::Rejected),
            "{status}"
        );
    }
    fake.set(Mode::NoText);
    assert_eq!(claude.draft(request()).await, Err(ProviderError::Rejected));
    assert_eq!(fake.requests().len(), 6);
}

// thought-drafting: Generation boundary, the adapter returns raw text; the studio trims
#[tokio::test]
async fn an_empty_text_block_is_returned_as_empty_text_not_as_an_error() {
    let fake = start(Mode::Empty).await;
    let draft = provider(&fake.base_url(), Some("sk-fake"), DEFAULT_MODEL)
        .draft(request())
        .await
        .unwrap();
    assert_eq!(draft.text, "");
}

#[tokio::test]
async fn an_unreachable_endpoint_is_rejected() {
    let closed = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap()
    };
    let result = provider(&format!("http://{closed}"), Some("sk-fake"), DEFAULT_MODEL)
        .draft(request())
        .await;
    assert_eq!(result, Err(ProviderError::Rejected));
}
