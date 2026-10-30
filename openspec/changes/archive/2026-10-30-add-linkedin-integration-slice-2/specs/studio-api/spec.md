## Purpose

Own the loopback HTTP boundary of the studio: bind address, Host rule, request ids, the error envelope, request logging, the operations table and the fixtures in `contracts/studio-api/`. Behavior behind each operation belongs to `thought-drafting`, `draft-review`, `dispatch-outbox`, `platform-credentials` and `linkedin-integration`.

## MODIFIED Requirements

### Requirement: Operations
The server SHALL expose exactly the operations in its scenario, each answering with the codes named there.

#### Scenario: Operations and their codes
- **WHEN** a client calls the API
- **THEN** `GET /v1/health` answers 200
- **AND** `POST /v1/thoughts` answers 201 (`thought-drafting`) or 400 `invalid_request`, 422 `thought_too_short`, 422 `unsupported_channel`, 502 `provider_error`, 503 `provider_not_configured`, 504 `provider_timeout`
- **AND** `GET /v1/pieces/{piece_id}` answers 200 (`draft-review`) or 404 `piece_not_found`
- **AND** `PATCH /v1/pieces/{piece_id}` answers 200 (`draft-review`) or 400 `invalid_request`, 404 `piece_not_found`, 409 `invalid_status_transition`, 409 `revision_conflict`
- **AND** `POST /v1/pieces/{piece_id}/publish` answers 200 (`dispatch-outbox`) or 400 `invalid_request`, 404 `piece_not_found`, 409 `revision_conflict`, 409 `approval_required`
- **AND** `POST /v1/linkedin/sign-in` answers 202 (`linkedin-integration`) or 409 `sign_in_in_progress`, 503 `linkedin_not_configured`
- **AND** `GET /v1/linkedin/sign-in` answers 200 (`linkedin-integration`) or 403 `sign_in_denied`, 403 `sign_in_state_mismatch`, 409 `sign_in_not_started`, 502 `sign_in_rejected`, 504 `sign_in_timed_out`
- **AND** `GET /v1/linkedin/connection` and `DELETE /v1/linkedin/connection` answer 200 (`platform-credentials`)
- **AND** every operation may answer 403 `host_not_allowed` or 500 `internal_error`; other paths answer 404 `not_found` and other methods 405 `method_not_allowed`

### Requirement: Request log
The server SHALL write one JSON log line per request with the request id, a fixed route label, status, error code and timings, never content or secrets.

#### Scenario: Content stays out of the logs
- **WHEN** a thought carrying canary text is drafted, shown, approved and published, and a LinkedIn sign-in and connection read happen
- **THEN** each request produced one line with `request_id`, `route`, `status`, `total_ms` and `outcome`
- **AND** the route labels are `/v1/thoughts`, `/v1/pieces/{piece_id}`, `/v1/pieces/{piece_id}/publish`, `/v1/linkedin/sign-in`, `/v1/linkedin/connection` or `unmatched`
- **AND** the canary text, the draft text, the provider key, the LinkedIn token and the client secret appear in no log line
