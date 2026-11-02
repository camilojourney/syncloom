## MODIFIED Requirements

### Requirement: Publish operation
`POST /v1/pieces/{piece_id}/publish` with `expected_revision` and `dry_run` SHALL return the piece, payload, outcome, request identity, post receipt, attempt count and historical account target.

#### Scenario: Response shape
- **WHEN** the operation answers 200
- **THEN** the existing piece, status, request_id, created, payload, post_id, post_url and attempts fields remain
- **AND** approval_target is the intent's non-secret target fingerprint when bound, never the current connection's identity
- **AND** status is dry_run, contained, posted, refused, consent_required, posting_permission_missing, posting_permission_unverified, rate_limited, unreachable or unknown
- **AND** dry_run defaults to false and optional cost_estimate retains the existing cost contract

### Requirement: Contained intent
A publish without a delivery grant SHALL reserve one contained intent per operation, piece, revision and approved target, without outbound HTTP.

#### Scenario: First publish
- **WHEN** an approved piece is published without a delivery grant
- **THEN** status is contained, created is true and request_id is dispatch- plus 32 hex characters
- **AND** the piece remains approved, publish_status is contained, and no outbound request occurs
- **AND** unbound local approval uses its historical targetless identity

### Requirement: Request id is deterministic
The intent's request_id SHALL deterministically identify its operation, piece, revision and, when bound, exact approved target.

#### Scenario: Worked example
- **WHEN** operation is publish, piece_id is thought-0123456789ab-linkedin_text and revision is b0ca8c02601b2edcb612b47c88494925c5e5f9d92044e91bc510adf47dd73f92 without a bound target
- **THEN** request_id remains dispatch-904f4c443cf556d7f34c6c307a86dfe3

#### Scenario: Different explicitly approved target
- **WHEN** the same revision is freshly approved for B after an intent for A
- **THEN** B has a distinct intent and request identity while A's intent remains unchanged

### Requirement: Repeat returns the same intent
A repeated publish for the same approved target SHALL preserve posted or ungranted contained intent identity without a new request or record.

#### Scenario: Second publish
- **WHEN** the same publish is sent again without a grant
- **THEN** request_id is unchanged, created is false, and no intent or event is added

#### Scenario: Second publish after posted
- **WHEN** a posted revision is published again for its approved target
- **THEN** status, request_id, post_id and attempts are unchanged, created is false, and no request or event occurs

### Requirement: Durable intents
An intent SHALL retain its target and existing payload, operation, revision, identity, timing, attempt and outcome fields across restart without fabricating legacy identity.

#### Scenario: Restart
- **WHEN** a posted target-bound intent is reopened
- **THEN** repeating its publish returns the same target, request_id and post_id with created false and no request

#### Scenario: Slice 1 record
- **WHEN** an older intent lacks attempt fields or approval_target
- **THEN** attempts defaults to zero and absent optional fields remain absent

## ADDED Requirements

### Requirement: Exact target dispatch
Publish SHALL refuse a resolved different or unbound account approval with account_approval_required before external effects, using draft-review account approval.

#### Scenario: Wrong account
- **WHEN** account B is connected after approving account A's unchanged revision
- **THEN** publish answers 409 account_approval_required before reserving or sending a new attempt
- **AND** review and fresh approval are required before B can post

#### Scenario: Snapshot race
- **WHEN** credentials change between approval, durable claim and media or post transmission
- **THEN** the attempt uses only credentials for the approved target or refuses without effects on another target
- **AND** concurrent review cannot relabel an in-flight intent or its outcome
- **AND** effective action permission and account target are checked on the same loaded immutable dispatch Token before media or posting writes
- **AND** a reconnect during media transmission cannot change that attempt's credential bundle, grant or receipt target

#### Scenario: Legacy approval
- **WHEN** an old unbound approval is delivered with resolved credentials
- **THEN** it requires fresh account approval rather than inheriting the current account

### Requirement: Unbound historical effect protection
Legacy unbound posted, posting or unknown intents SHALL prevent a new target identity from bypassing their repeat and uncertainty protection.

#### Scenario: Reconnect and fresh approval
- **WHEN** a legacy unbound terminal or uncertain intent exists and another account is freshly approved
- **THEN** publish returns the old outcome without a new request or intent
- **AND** posting becomes unknown through the existing interrupted-attempt rule
- **AND** no historical target is invented
