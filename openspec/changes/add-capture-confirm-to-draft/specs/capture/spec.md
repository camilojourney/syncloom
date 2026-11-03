## Purpose

Own founder-controlled capture suggestions, a preview that is not approval, and explicit confirmation of user-owned text into pending drafts. Draft creation stays with `thought-drafting`. Channel bounds stay with `platform-rules`.

## ADDED Requirements

### Requirement: Suggest from founder text
`POST /v1/capture/suggest` with exactly `text` SHALL return text-only route suggestions without calling a provider or storing a piece.

#### Scenario: Short plain text
- **WHEN** `text` is `Ship the small fix today.`
- **THEN** status is 200 and `suggestions` are `x` at `0.86`, `threads` at `0.80`, `linkedin_text` at `0.78`, then `facebook_page` at `0.60`
- **AND** `x` has format hint `single post` and notes `Keep as one punchy line; no thread needed.`
- **AND** `threads` has format hint `short conversation post` and notes `Optional: add a question to invite replies.`
- **AND** `linkedin_text` has format hint `authority text post` and notes `Open with the lesson, then one concrete example.`
- **AND** `facebook_page` has format hint `community post` and notes `Soften the tone; sound like a note to friends.`
- **AND** the body has no piece, approval, or publish status, and the provider is not called

#### Scenario: Plain-text boundaries
- **WHEN** trimmed text has 200 Unicode scalar values and no URL
- **THEN** `linkedin_text` confidence is `0.78` and `x` and `threads` are present
- **AND** when trimmed text has 201 scalar values and no URL, `linkedin_text` confidence is `0.90`
- **AND** when it has 280 scalar values, `x` and `threads` are present
- **AND** when it has 281 or 401 scalar values and no URL, `x` and `threads` are absent and `facebook_page` remains at `0.60`
- **AND** no carousel channel is suggested

#### Scenario: URL hint without retrieval
- **WHEN** `text` is `Read this https://example.invalid/note today.`
- **THEN** suggestions are `linkedin_text` at `0.88`, `x` at `0.80`, then `threads` at `0.72`
- **AND** their format hints are `text post with link`, `short post or thread seed`, and `conversation starter`
- **AND** their notes are `Put the opinion first; drop the URL at the end.`, `Compress to one sharp claim; link at the end if needed.`, and `Ask one follow-up question after the link takeaway.`
- **AND** no request is sent to that URL and `facebook_page` is absent

#### Scenario: Too short or unknown field
- **WHEN** `text` has 7 non-space characters
- **THEN** status is 422 `thought_too_short` and the provider is not called
- **AND** when the body also has `attachment_url`, status is 400 `invalid_request`

### Requirement: Preview without drafting
`POST /v1/capture/preview` with exactly `text` and `channels` SHALL return a non-approval preview and create no piece.

#### Scenario: Selected routes
- **WHEN** `channels` is a non-empty subset of that text's suggestions, without repeats
- **THEN** status is 200, `text` equals the submitted text, and `approval` and `publish_status` are null
- **AND** the body includes `revision`, `normalized_thought`, and each selected channel's `platform-rules` fit
- **AND** the displayed founder text is complete, not trimmed to a channel bound
- **AND** the provider is not called and no piece is stored

#### Scenario: Selection refused
- **WHEN** `channels` is empty, repeated, or includes `instagram` for plain text
- **THEN** status is 400 `invalid_request` and nothing is stored
- **AND** when `channels` is `["tiktok_video"]`, status is 422 `unsupported_channel`

### Requirement: Preview revision
A preview revision SHALL identify the exact submitted text and the selected channel set.

#### Scenario: Stable set
- **WHEN** the same text and channel set are previewed with `channels` reversed
- **THEN** both `revision` values are the same 64-character lowercase hex digest
- **AND** changing one character of `text` changes the revision

### Requirement: Confirm into drafts
`POST /v1/capture/confirm` with `text`, `channels` and `expected_revision` SHALL create pending drafts only through `thought-drafting` when the revision matches.

#### Scenario: Matching revision
- **WHEN** `expected_revision` equals that preview revision and the provider answers a draft
- **THEN** status is 201 and the body is the `thought-drafting` group for that text and those channels
- **AND** each piece is `pending_review`, with null `review_decision_id` and `publish_status`
- **AND** `source.raw_input` is the submitted text
- **AND** no approval event and no publish intent are recorded

### Requirement: Stale confirmation
Confirm SHALL reject an expected revision that is not the preview revision for that text and channels.

#### Scenario: Stale digest
- **WHEN** `expected_revision` is `a` repeated 64 times and is not that preview revision
- **THEN** status is 409 `revision_conflict`
- **AND** the provider is not called and no piece is stored

### Requirement: Duplicate confirmation
A repeated confirm of one matching revision SHALL return the original drafts without drafting again.

#### Scenario: Interrupted claim
- **WHEN** an earlier confirm stopped after reserving the revision but before recording a complete result
- **THEN** another confirm returns 409 `revision_conflict` without a provider call
- **AND** the founder can inspect Saved pieces; no automatic recovery runs

#### Scenario: Retry
- **WHEN** the first confirm returns 201 and the same body is submitted again
- **THEN** status is 200 and `group_id` and piece ids match the first response
- **AND** the provider was called once per selected channel

### Requirement: Capture pages
Capture pages SHALL present suggestion, preview and confirm as separate steps with no approve or publish control.

#### Scenario: Explicit confirm
- **WHEN** the founder submits text, selects one suggestion, previews, and confirms
- **THEN** the preview says it is not an approval and not yet a draft
- **AND** confirm redirects to a result linking each new review page
- **AND** the same home thought form offers both immediate drafting and Suggest destinations first

#### Scenario: Narrow return navigation
- **WHEN** a capture page is shown at 390px in either theme, including long source text
- **THEN** the shared header's Workspace link remains visible, keyboard reachable and returns to `/`
- **AND** the brand, navigation and local-status text are not clipped and the page has no horizontal overflow

#### Scenario: Empty selection
- **WHEN** preview is submitted with no selected channel
- **THEN** the page says at least one suggestion must be selected and no draft exists

### Requirement: Capture result
`GET /capture/result/{revision}` SHALL list confirmed drafts for that revision or answer not found.

#### Scenario: Known and unknown
- **WHEN** the revision was confirmed
- **THEN** the page links each piece review and does not offer publish
- **AND** an unknown or malformed revision answers 404 `not_found` and stores nothing
- **AND** an incomplete or inconsistent confirmation answers 409 `revision_conflict` without drafting

### Requirement: Capture log labels
Capture requests SHALL log fixed route labels and never the submitted text.

#### Scenario: Canary stays out
- **WHEN** suggest, preview, confirm and the capture pages carry canary text
- **THEN** logged routes are `/v1/capture/suggest`, `/v1/capture/preview`, `/v1/capture/confirm`, `/capture`, `/capture/suggest`, `/capture/preview`, `/capture/confirm` or `/capture/result/{revision}`
- **AND** the canary is absent from every log line
