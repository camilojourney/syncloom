## MODIFIED Requirements

### Requirement: Publish operation
`POST /v1/pieces/{piece_id}/publish` with `expected_revision` and `dry_run` SHALL answer 200 with piece, payload, outcome status, request id, post id/URL, attempt count and intent's historical account target.

#### Scenario: Response shape
- **WHEN** the operation answers 200
- **THEN** the body is `{"piece":<piece>,"status":<status>,"request_id":<string or null>,"created":<bool>,"payload":{"content":<text>,"platforms":["linkedin"],"style":"raw"},"post_id":<string or null>,"post_url":<string or null>,"attempts":<int>,"approval_target":<intent target or absent>}`; optional cost_estimate retains its existing contract
- **AND** `status` is one of `dry_run`, `contained`, `posted`, `refused`, `consent_required`, `posting_permission_missing`, `posting_permission_unverified`, `rate_limited`, `unreachable`, `unknown`
- **AND** `dry_run` defaults to false when absent
- **AND** approval_target never substitutes the current connection's identity for the intent's historical target
- **AND** account-target and effective-action checks use the same immutable loaded dispatch Token before any media or post write
