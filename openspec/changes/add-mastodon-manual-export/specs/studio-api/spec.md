# API delta

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
- **AND** Mastodon sign-in, connection and export operations follow `Mastodon and export operations` below
- **AND** `GET /v1/pieces` answers 200 or 400 `invalid_request` (`draft-review`), and `GET /v1/session` answers 200 with a session token
- **AND** `GET /`, `GET /review/{piece_id}`, `GET /review/{piece_id}/edit|render`, `POST /actions/thought`, `POST /review/{piece_id}/{action}` and `GET /studio.css` serve the browser studio; actions are `edit`, `render`, `approve` and `publish`
- **AND** every operation may answer 403 `host_not_allowed`, `origin_not_allowed`, `session_required` or 500 `internal_error`; other paths answer 404 `not_found` and other methods 405 `method_not_allowed`

### Requirement: Request log
The server SHALL write one JSON log line per request with the request id, a fixed route label, status, error code and timings, never content or secrets.

#### Scenario: Content stays out of the logs
- **WHEN** a thought carrying canary text is drafted, shown, approved and published, and a LinkedIn sign-in and connection read happen
- **THEN** each request produced one line with `request_id`, `route`, `status`, `total_ms` and `outcome`
- **AND** the route labels are `/v1/thoughts`, `/v1/pieces/{piece_id}`, `/v1/pieces/{piece_id}/publish`, `/v1/linkedin/sign-in`, `/v1/linkedin/connection`, `/v1/pieces/{piece_id}/export`, `/v1/mastodon/sign-in`, `/v1/mastodon/connection` or `unmatched`
- **AND** the session, browser and asset route labels added by their capabilities remain unchanged
- **AND** the canary text, the draft text, the provider key, both platform tokens, the session token and the client secret appear in no log line

## ADDED Requirements

### Requirement: Mastodon and export operations
The loopback API SHALL expose Mastodon connection operations and approved-piece export under the existing Host, Origin, server-session, no-CORS and fixed-error boundary.

#### Scenario: Operations
- **WHEN** a client calls the API with the correct Host, permitted Origin and session token for mutations
- **THEN** `POST /v1/mastodon/sign-in` returns 202 with authorization URL and expiry; `GET` waits and returns 200 with connection
- **AND** `GET /v1/mastodon/connection` and `DELETE /v1/mastodon/connection` return 200 connection results
- **AND** `POST /v1/pieces/{piece_id}/export` accepts `expected_revision` and `target` and returns the 200 export receipt
- **AND** failures use the existing review, sign-in and storage codes; an unconfigured instance returns 503 `mastodon_not_configured`
- **AND** missing session, wrong Host, foreign Origin, cross-site metadata and malformed bodies send no platform request

### Requirement: Saved-channel browser identity
The browser SHALL identify saved pieces using their actual channel without claiming that every visual publish remains contained.

#### Scenario: Non-LinkedIn pieces
- **WHEN** a Mastodon or manual-export piece saved by the CLI or API is opened in the browser
- **THEN** the discovery card, review heading and text preview show its actual channel, not LinkedIn
- **AND** publish guidance distinguishes contained LinkedIn visuals from granted Mastodon PNG delivery and local-only carousels/manual export
