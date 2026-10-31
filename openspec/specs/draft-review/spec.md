# draft-review Specification

## Purpose
Own a piece after it is drafted: how it is shown, its statuses and their transitions, its revision, edits, the approval bound to a revision, and the durability of the piece record.

## Requirements

### Requirement: Show a piece
`GET /v1/pieces/{piece_id}` SHALL answer 200 with the piece, or 404 `piece_not_found`.

#### Scenario: Known and unknown ids
- **WHEN** a client requests a piece created earlier
- **THEN** status is 200 and the body equals the piece returned at creation
- **AND** `GET /v1/pieces/thought-000000000000-linkedin_text` for an id never created answers 404 with code `piece_not_found`

### Requirement: Revision
A piece's revision SHALL identify its channel, id, text and, when configured, visual content.

#### Scenario: Worked example
- **WHEN** `piece_id` is `thought-0123456789ab-linkedin_text`, `channel` is `linkedin_text` and `text` is `hello world`
- **THEN** `revision` is `b0ca8c02601b2edcb612b47c88494925c5e5f9d92044e91bc510adf47dd73f92`
- **AND** a different text gives a different revision

#### Scenario: Text-only compatibility
- **WHEN** a piece has no visual configuration
- **THEN** its revision remains the lowercase hex SHA-256 of compact sorted-key JSON containing channel, piece_id and text
- **AND** the existing hello-world worked example remains unchanged

#### Scenario: Visual revision
- **WHEN** text, slide content or template changes
- **THEN** the revision changes, assets are invalidated and approval is cleared
- **AND** a render finishing after a concurrent edit is refused with revision_conflict

### Requirement: Statuses
A piece SHALL be in exactly one of the statuses `pending_review`, `approved` or `rejected`.

#### Scenario: Statuses are the only values
- **WHEN** `PATCH /v1/pieces/{piece_id}` names a status outside the three, such as `draft`, `published` or `scheduled`
- **THEN** status is 400 with code `invalid_request`

### Requirement: Status transitions
`PATCH /v1/pieces/{piece_id}` with `status` and `expected_revision` SHALL move the piece only along the transitions in its scenario.

#### Scenario: Allowed transitions
- **WHEN** the piece is `pending_review`
- **THEN** `approved` and `rejected` are accepted with 200
- **AND** from `approved` only `rejected` is accepted, and from `rejected` only `pending_review`
- **AND** the piece's current status repeated is accepted with 200 and changes nothing

#### Scenario: Refused transitions
- **WHEN** the piece is `approved` and the body asks for `pending_review`, or the piece is `rejected` and the body asks for `approved`
- **THEN** status is 409 with code `invalid_status_transition` and the piece is unchanged

### Requirement: Approval bound to a revision
Approving a piece SHALL record `review_decision_id` as `review-` plus the piece id, `-`, and the first 16 characters of its revision.

#### Scenario: Approve
- **WHEN** a `pending_review` piece with revision `b0ca8c02601b2edc...` and id `thought-0123456789ab-linkedin_text` is approved at that revision
- **THEN** `status` is `approved` and `review_decision_id` is `review-thought-0123456789ab-linkedin_text-b0ca8c02601b2edc`
- **AND** `updated_at` is later than or equal to `created_at`

### Requirement: Stale revision refused
`PATCH /v1/pieces/{piece_id}` SHALL answer 409 `revision_conflict` and change nothing when `expected_revision` differs from the piece's revision.

#### Scenario: Stale approval
- **WHEN** the body carries an `expected_revision` that is not the current one
- **THEN** status is 409 with code `revision_conflict`
- **AND** the piece's status, text and `review_decision_id` are unchanged

### Requirement: Edit creates a revision and voids approval
`PATCH /v1/pieces/{piece_id}` with `text` and `expected_revision` SHALL replace the text, compute a new revision, set `pending_review` and clear `review_decision_id`.

#### Scenario: Edit an approved piece
- **WHEN** an `approved` piece is edited with its current revision and a different text
- **THEN** status is 200, `text` is the new text with surrounding whitespace removed, `revision` differs from the old one
- **AND** `status` is `pending_review`, `review_decision_id` is null and `fit` matches the new text

#### Scenario: Publish after an edit needs a new approval
- **WHEN** the edited piece is published with its new revision
- **THEN** status is 409 with code `approval_required`

#### Scenario: Edit body rules
- **WHEN** the body has both `status` and `text`, neither, or a `text` that is empty after trimming
- **THEN** status is 400 with code `invalid_request`

### Requirement: Durable pieces
A piece SHALL survive a restart, and an interrupted write leaves the previous record or the new one, never a torn file.

#### Scenario: Restart
- **WHEN** a piece is created and approved, and the store is reopened on the same directory
- **THEN** the piece reads back equal to the last response

#### Scenario: Leftover temporary file
- **WHEN** a file with the temporary-write name pattern and partial content lies beside a piece record
- **THEN** the piece reads back from its record and the temporary file is not read as a piece

### Requirement: Visual approval guard
Visual approval SHALL require intact current-revision assets matching the caller's expected image and carousel SHA-256 values.

#### Scenario: Render before approval
- **WHEN** visual content changes or rendering fails
- **THEN** approval is refused with assets_required until a successful render
- **AND** approval supplies both rendered PNG and PDF SHA-256 values as expected_assets
- **AND** missing hashes return assets_required and mismatched hashes return asset_conflict, including after a same-revision rerender
- **AND** rerendering clears approval even when the input revision stays unchanged

#### Scenario: No visual upload
- **WHEN** an approved visual piece is published with LinkedIn delivery granted
- **THEN** it stays contained without a platform request

### Requirement: Discover saved pieces
`GET /v1/pieces` with optional `after` SHALL return a deterministic page of saved pieces and `next_after`, or 400 `invalid_request` for an invalid cursor.

#### Scenario: Multiple pages
- **WHEN** more than twenty pieces exist
- **THEN** the response is 200 with `{"pieces":<up to twenty pieces>,"next_after":<last id or null>}` ordered by piece id
- **AND** passing `next_after` as `after` returns the next page without duplicates
- **AND** no pieces returns an empty array and null cursor
- **AND** interrupted-write files are ignored, malformed selected records return `internal_error`, and symlinks are refused
