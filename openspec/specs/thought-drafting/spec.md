# thought-drafting Specification

## Purpose
Own thought intake and the drafting of pieces through the model provider: what a thought must be, how it reaches the model, and what a failed draft leaves behind. Channel identities and limits belong to `platform-rules`; the piece's later life belongs to `draft-review`.

## Requirements

### Requirement: Create pieces from a thought
`POST /v1/thoughts` with `text` and `channels` SHALL answer 201 with a group id and one pending piece per channel.

#### Scenario: One LinkedIn text piece
- **WHEN** the body is `{"text":"<a thought of 8 or more non-space characters>","channels":["linkedin_text"]}` and the provider answers `"  A draft about the thought.  "`
- **THEN** status is 201 and the body is `{"group_id":<32 hex>,"pieces":[<piece>]}`
- **AND** the piece has `piece_id` equal to `thought-` plus the first 12 characters of `group_id` plus `-linkedin_text`
- **AND** `channel` is `linkedin_text`, `platform` is `linkedin`, `status` is `pending_review`, `text` is `A draft about the thought.`
- **AND** `source` is `{"type":"text","raw_input":<the text exactly as sent>}`, `review_decision_id` and `publish_status` are null
- **AND** `revision`, `fit`, `created_at` and `updated_at` are present and `updated_at` equals `created_at`

### Requirement: Thought normalization
The studio SHALL send the provider the thought with surrounding whitespace removed and inner whitespace runs collapsed to one space.

#### Scenario: Whitespace collapsed
- **WHEN** the text is `"  one\n\n two   three  "`
- **THEN** the provider receives `one two three` as the thought
- **AND** `source.raw_input` keeps `"  one\n\n two   three  "`

### Requirement: Thought too short
The studio SHALL answer 422 `thought_too_short` when the thought has fewer than 8 non-space characters.

#### Scenario: Seven characters refused
- **WHEN** the text is `"a b c d e f g"` (7 non-space characters)
- **THEN** status is 422 with code `thought_too_short`
- **AND** the provider is not called and nothing is stored

#### Scenario: Eight characters accepted
- **WHEN** the text is `"a b c d e f g h"` (8 non-space characters)
- **THEN** status is 201

### Requirement: Unsupported channel
The studio SHALL answer 422 `unsupported_channel` when any requested channel is not in the `platform-rules` table.

#### Scenario: Unknown channel
- **WHEN** `channels` is `["linkedin_text","tiktok_video"]`
- **THEN** status is 422 with code `unsupported_channel`
- **AND** the provider is not called and nothing is stored

#### Scenario: Repeated channel
- **WHEN** `channels` is `["linkedin_text","linkedin_text"]`
- **THEN** status is 400 with code `invalid_request`

### Requirement: Generation boundary
Every draft SHALL come from the `ModelProvider` trait, which receives the channel and the normalized thought and returns text with the provider's token usage.

#### Scenario: Provider receives the thought as data
- **WHEN** a thought is drafted through a fake provider
- **THEN** the fake received exactly one request with channel `linkedin_text` and the normalized thought
- **AND** the stored piece text is the fake's answer with surrounding whitespace removed

### Requirement: Provider failures persist nothing
A provider failure SHALL answer 502 `provider_error`, 503 `provider_not_configured` or 504 `provider_timeout` and leave no piece and no provenance event.

#### Scenario: Empty answer
- **WHEN** the provider answers text that is empty after trimming
- **THEN** status is 502 with code `provider_error` and the store holds no piece and no event

#### Scenario: Provider error
- **WHEN** the provider fails
- **THEN** status is 502 with code `provider_error` and the store holds no piece and no event

#### Scenario: Provider deadline
- **WHEN** the provider has not answered within the configured deadline
- **THEN** status is 504 with code `provider_timeout` and the store holds no piece and no event

#### Scenario: No key
- **WHEN** the provider has no usable key
- **THEN** status is 503 with code `provider_not_configured` and the store holds no piece and no event

### Requirement: Claude provider
The Claude adapter SHALL call the Messages API with the pinned model id, a fixed system instruction, the thought as the user message, and the key from the environment only.

#### Scenario: Request shape
- **WHEN** a thought is drafted through the Claude adapter against a fake Messages endpoint
- **THEN** the fake received one `POST /v1/messages` with headers `x-api-key` and `anthropic-version`, body `model` equal to the pinned id, a `system` string, `max_tokens` of 1200 and one `user` message whose content is the normalized thought
- **AND** the key appears in no log line and no stored record

#### Scenario: Pinned model with override
- **WHEN** `SYNCLOOM_CLAUDE_MODEL` is unset
- **THEN** `model` is `claude-sonnet-5-5`
- **AND** when it is set, `model` is its value

#### Scenario: Provider statuses
- **WHEN** the endpoint answers 401 or 403
- **THEN** the draft fails as `provider_not_configured`
- **AND** when it answers 429, 500 or a body without text, the draft fails as `provider_error`

#### Scenario: Base URL
- **WHEN** `SYNCLOOM_CLAUDE_BASE_URL` is unset, `https://api.anthropic.com`, or `http://127.0.0.1:<port>`
- **THEN** the server starts and uses it
- **AND** when it is `http://example.com`, `ftp://x`, carries credentials, a query or a fragment, the process exits with an error naming `SYNCLOOM_CLAUDE_BASE_URL`

### Requirement: Token usage recorded
The studio SHALL record the provider's reported input and output tokens for each draft in the `provenance-log` event, never a cost estimate.

#### Scenario: Usage from the provider
- **WHEN** the provider reports `input_tokens` 321 and `output_tokens` 123
- **THEN** the `piece_created` event carries `usage` `{"input_tokens":321,"output_tokens":123}`
