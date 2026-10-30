# dispatch-outbox Specification

## Purpose
Own what happens when the founder asks to publish an approved piece: the approval and revision checks, the dry run, the one durable intent per piece and revision, its containment while no platform integration exists, and repeat safety.

## Requirements

### Requirement: Publish operation
`POST /v1/pieces/{piece_id}/publish` with `expected_revision` and `dry_run` SHALL answer 200 with the piece, the payload, the outcome status, the request id, the post id and URL, and the attempt count.

#### Scenario: Response shape
- **WHEN** the operation answers 200
- **THEN** the body is `{"piece":<piece>,"status":<status>,"request_id":<string or null>,"created":<bool>,"payload":{"content":<text>,"platforms":["linkedin"],"style":"raw"},"post_id":<string or null>,"post_url":<string or null>,"attempts":<int>}`
- **AND** `status` is one of `dry_run`, `contained`, `posted`, `refused`, `consent_required`, `rate_limited`, `unreachable`, `unknown`
- **AND** `dry_run` defaults to false when absent

### Requirement: Approval required
Publishing SHALL answer 409 `approval_required` unless the piece is `approved` with a `review_decision_id`.

#### Scenario: Not approved
- **WHEN** a `pending_review` or `rejected` piece is published with its current revision and `dry_run` false
- **THEN** status is 409 with code `approval_required`
- **AND** no intent is stored

### Requirement: Stale revision refused on publish
Publishing SHALL answer 409 `revision_conflict` when `expected_revision` differs from the piece's revision, for a dry run too.

#### Scenario: Stale publish
- **WHEN** an approved piece is published with an `expected_revision` that is not current
- **THEN** status is 409 with code `revision_conflict` and no intent is stored

### Requirement: Dry run records nothing
A publish with `dry_run` true SHALL return the payload with status `dry_run`, store no intent, no event and no change to the piece, and send no platform request.

#### Scenario: Dry run on an unapproved piece
- **WHEN** a `pending_review` piece is published with `dry_run` true and its current revision
- **THEN** status is 200, `status` is `dry_run`, `request_id` is null, `created` is false
- **AND** `payload.content` is the piece text, the outbox holds no intent, the provenance log gained no event and the piece is unchanged

#### Scenario: Dry run with delivery granted
- **WHEN** an approved piece is published with `dry_run` true and `SYNCLOOM_DELIVERY` names its platform
- **THEN** `status` is `dry_run` and the fake platform received no request

### Requirement: Contained intent
A publish for a platform without a delivery grant SHALL reserve exactly one intent per piece, revision and operation, marked `contained`, with no outbound HTTP.

#### Scenario: First publish
- **WHEN** an approved piece is published with its revision and `dry_run` false and no grant names its platform
- **THEN** status is 200 with `status` `contained`, `created` true and `request_id` equal to `dispatch-` plus 32 hex characters
- **AND** the piece's `publish_status` is `contained` and its `status` stays `approved`
- **AND** the outbox holds one intent and the fake provider endpoint received no request during the publish

### Requirement: Request id is deterministic
The intent's `request_id` SHALL be `dispatch-` plus the first 32 characters of the SHA-256 of the compact sorted JSON of `operation`, `piece_id` and `revision`.

#### Scenario: Worked example
- **WHEN** `operation` is `publish`, `piece_id` is `thought-0123456789ab-linkedin_text` and `revision` is `b0ca8c02601b2edcb612b47c88494925c5e5f9d92044e91bc510adf47dd73f92`
- **THEN** `request_id` is `dispatch-904f4c443cf556d7f34c6c307a86dfe3`

### Requirement: Repeat returns the same intent
A repeated publish of a `posted` intent, or of a `contained` intent without a grant, SHALL return the same `request_id` with `created` false, send no request and store nothing new.

#### Scenario: Second publish
- **WHEN** the same publish is sent again without a grant
- **THEN** status is 200, `request_id` equals the first, `created` is false
- **AND** the outbox still holds one intent and the provenance log gained no event

#### Scenario: Second publish after posted
- **WHEN** a `posted` piece is published again with the same revision
- **THEN** the response is `posted` with the same `request_id`, `post_id` and `attempts`, `created` false
- **AND** the fake platform received no new request and the provenance log gained no event

### Requirement: Durable intents
An intent SHALL survive a restart with its `request_id`, `operation`, `piece_id`, `revision`, `payload`, `status`, `created_at` and, when set, `attempts`, `attempted_at`, `finished_at`, `post_id` and `post_url`.

#### Scenario: Restart
- **WHEN** an intent is `posted` and the store is reopened on the same directory
- **THEN** a repeated publish returns the same `request_id` and `post_id` with `created` false and no request

#### Scenario: Slice 1 record
- **WHEN** an intent file written before this change, without the attempt fields, is read
- **THEN** it loads with `attempts` 0 and the other new fields absent

### Requirement: Delivery grant
`SYNCLOOM_DELIVERY` SHALL name the platforms whose intents are dispatched; other platforms' intents stay contained, and an unknown name refuses startup.

#### Scenario: No grant
- **WHEN** `SYNCLOOM_DELIVERY` is unset or blank and an approved piece is published
- **THEN** the intent is `contained` and the fake platform received no request

#### Scenario: LinkedIn granted
- **WHEN** `SYNCLOOM_DELIVERY` is `linkedin` and an approved `linkedin_text` piece is published
- **THEN** one dispatch attempt is made

#### Scenario: Unknown platform
- **WHEN** `SYNCLOOM_DELIVERY` is `linkedin,tiktok` or `instagram`
- **THEN** the process exits with an error naming `SYNCLOOM_DELIVERY` and binds no port

### Requirement: Attempt recorded before the request
A dispatch SHALL write the intent as `posting` with the attempt count and time before any platform request is sent.

#### Scenario: Intent is posting during the call
- **WHEN** the fake platform reads the outbox while it handles the request
- **THEN** the intent file holds `status` `posting`, `attempts` 1 and `attempted_at` set
- **AND** the provenance log holds a `dispatch_attempted` event for the request id

### Requirement: Outcome recorded truthfully
After the platform answers, the intent and the piece's `publish_status` SHALL carry the outcome, with `post_id` and `post_url` only when it is `posted`.

#### Scenario: Posted
- **WHEN** the platform answers posted with a post id
- **THEN** the response is 200 with `status` `posted`, `created` true on a first attempt, `post_id` and `post_url` set and `attempts` 1
- **AND** the intent holds the same values plus `finished_at`, and the piece's `publish_status` is `posted` while its `status` stays `approved`
- **AND** the provenance log gained `dispatch_finished` with `outcome` `posted` and the `post_id`

#### Scenario: Failure
- **WHEN** the platform answers `refused`, `consent_required`, `rate_limited`, `unreachable` or `unknown`
- **THEN** the response is 200 with that `status`, `post_id` and `post_url` null, and the intent and the piece carry the same status

### Requirement: Retry after a retryable outcome
A publish of an intent in `contained`, `refused`, `consent_required`, `rate_limited` or `unreachable` SHALL make exactly one new attempt and increment `attempts`.

#### Scenario: Rate limited then posted
- **WHEN** the first attempt answers `rate_limited` and the piece is published again with the platform answering posted
- **THEN** the fake platform received two requests in total, the second response is `posted` with `attempts` 2 and `created` false

### Requirement: Unknown is never retried blindly
A publish that finds the intent `posting` or `unknown` SHALL answer `unknown` with no platform request.

#### Scenario: Interrupted attempt
- **WHEN** the intent file is `posting` (an attempt interrupted before its outcome was written) and the piece is published again
- **THEN** the response is `unknown`, the fake platform received no request, and the intent is now `unknown`
- **AND** publishing once more still sends nothing

### Requirement: Platform deadline
A platform call that exceeds `platform_deadline` SHALL be recorded as `unknown`.

#### Scenario: Hanging platform
- **WHEN** the platform does not answer within the deadline
- **THEN** the response is `unknown`, the fake received exactly one request and the intent is `unknown`
