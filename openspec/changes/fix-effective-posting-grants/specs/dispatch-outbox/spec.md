## MODIFIED Requirements

### Requirement: Publish operation
`POST /v1/pieces/{piece_id}/publish` with `expected_revision` and `dry_run` SHALL answer 200 with the piece, the payload, the outcome status, the request id, the post id and URL, and the attempt count.

#### Scenario: Response shape
- **WHEN** the operation answers 200
- **THEN** the body is `{"piece":<piece>,"status":<status>,"request_id":<string or null>,"created":<bool>,"payload":{"content":<text>,"platforms":["linkedin"],"style":"raw"},"post_id":<string or null>,"post_url":<string or null>,"attempts":<int>}`
- **AND** `status` is one of `dry_run`, `contained`, `posted`, `refused`, `consent_required`, `posting_permission_missing`, `posting_permission_unverified`, `rate_limited`, `unreachable`, `unknown`
- **AND** `dry_run` defaults to false when absent
