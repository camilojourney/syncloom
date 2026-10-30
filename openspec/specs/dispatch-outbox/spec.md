# dispatch-outbox Specification

## Purpose
Own what happens when the founder asks to publish an approved piece: the approval and revision checks, the dry run, the one durable intent per piece and revision, its containment while no platform integration exists, and repeat safety.

## Requirements

### Requirement: Publish operation
`POST /v1/pieces/{piece_id}/publish` with `expected_revision` and `dry_run` SHALL answer 200 with the piece, the payload, the outcome status and the request id.

#### Scenario: Response shape
- **WHEN** the operation answers 200
- **THEN** the body is `{"piece":<piece>,"status":"dry_run"|"contained","request_id":<string or null>,"created":<bool>,"payload":{"content":<text>,"platforms":["linkedin"],"style":"raw"}}`
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
A publish with `dry_run` true SHALL return the payload with status `dry_run` and store no intent, no event and no change to the piece.

#### Scenario: Dry run on an unapproved piece
- **WHEN** a `pending_review` piece is published with `dry_run` true and its current revision
- **THEN** status is 200, `status` is `dry_run`, `request_id` is null, `created` is false
- **AND** `payload.content` is the piece text, the outbox holds no intent, the provenance log gained no event and the piece is unchanged

### Requirement: Contained intent
A publish SHALL reserve exactly one intent per piece, revision and operation, marked `contained`, with no outbound HTTP.

#### Scenario: First publish
- **WHEN** an approved piece is published with its revision and `dry_run` false
- **THEN** status is 200 with `status` `contained`, `created` true and `request_id` equal to `dispatch-` plus 32 hex characters
- **AND** the piece's `publish_status` is `contained` and its `status` stays `approved`
- **AND** the outbox holds one intent and the fake provider endpoint received no request during the publish

### Requirement: Request id is deterministic
The intent's `request_id` SHALL be `dispatch-` plus the first 32 characters of the SHA-256 of the compact sorted JSON of `operation`, `piece_id` and `revision`.

#### Scenario: Worked example
- **WHEN** `operation` is `publish`, `piece_id` is `thought-0123456789ab-linkedin_text` and `revision` is `b0ca8c02601b2edcb612b47c88494925c5e5f9d92044e91bc510adf47dd73f92`
- **THEN** `request_id` is `dispatch-904f4c443cf556d7f34c6c307a86dfe3`

### Requirement: Repeat returns the same intent
A repeated publish of the same piece and revision SHALL return the same `request_id` with `created` false and store nothing new.

#### Scenario: Second publish
- **WHEN** the same publish is sent again
- **THEN** status is 200, `request_id` equals the first, `created` is false
- **AND** the outbox still holds one intent and the provenance log gained no event

### Requirement: Durable intents
An intent SHALL survive a restart with its `request_id`, `operation`, `piece_id`, `revision`, `payload`, `status` and `created_at`.

#### Scenario: Restart
- **WHEN** an intent is reserved and the store is reopened on the same directory
- **THEN** a repeated publish returns the same `request_id` with `created` false
