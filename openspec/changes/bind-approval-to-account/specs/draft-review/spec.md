## MODIFIED Requirements

### Requirement: Show a piece
`GET /v1/pieces/{piece_id}` SHALL return the saved piece plus resolved current_target for review, or 404 piece_not_found.

#### Scenario: Known and unknown ids
- **WHEN** a client requests a created piece without a resolved delivery target
- **THEN** status is 200 and the body equals the saved piece
- **AND** an unknown piece id returns 404 piece_not_found

#### Scenario: Account context
- **WHEN** a configured delivery adapter resolves the current account
- **THEN** current_target identifies that account without disclosing credentials
- **AND** approval_target, when present, remains the historical approved target

### Requirement: Status transitions
`PATCH /v1/pieces/{piece_id}` with `status` and `expected_revision` SHALL move the piece only along the transitions in its scenario.

#### Scenario: Allowed transitions
- **WHEN** the piece is `pending_review`
- **THEN** `approved` and `rejected` are accepted with 200
- **AND** from `approved` only `rejected` is accepted, and from `rejected` only `pending_review`
- **AND** repeating the current status changes nothing unless fresh account approval is required

#### Scenario: Refused transitions
- **WHEN** the piece is `approved` and the body asks for `pending_review`, or the piece is `rejected` and the body asks for `approved`
- **THEN** status is 409 with code `invalid_status_transition` and the piece is unchanged

## ADDED Requirements

### Requirement: Exact account approval
Approval SHALL bind the reviewed content and assets to the resolved authenticated posting target, without treating credentials, handles or connection slots as account identity.

#### Scenario: Reconnected account
- **WHEN** a revision approved for account A is viewed after connecting account B
- **THEN** its old approval remains historical and does not approve B
- **AND** review/API/CLI expose the need to approve the current account

#### Scenario: Stale review form
- **WHEN** status approved supplies expected_revision and expected_target different from the resolved current_target, including an omitted target
- **THEN** review returns 409 account_approval_required and changes nothing
- **AND** the browser submits the same target it displays

#### Scenario: Fresh approval
- **WHEN** an already approved revision is approved again after changing accounts
- **THEN** an explicit matching expected_target binds the current resolved account and appends new decision evidence without rewriting prior events
- **AND** the content revision and asset hashes remain unchanged

#### Scenario: Offline and legacy review
- **WHEN** a saved approval has no resolved authenticated target
- **THEN** it remains usable only for contained or manual operations until fresh approval binds a resolved account
- **AND** no target is fabricated for historical approvals

#### Scenario: Same account token rotation
- **WHEN** the provider renews credentials for the same canonical account and exact target
- **THEN** the account approval remains valid without another review
