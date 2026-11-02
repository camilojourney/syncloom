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
- **AND** `POST /v1/pieces/{piece_id}/render` takes exactly `expected_revision`, `template` and `slides` and answers 200 with the piece, or 400 `invalid_request`, 404 `piece_not_found`, 409 `revision_conflict`, 503 `renderer_missing`, 503 `renderer_version_mismatch`, 503 `renderer_font_mismatch`, 504 `renderer_timeout`, 502 `renderer_failed`
- **AND** `GET /v1/pieces/{piece_id}/assets/image` and `/assets/carousel` answer 200 with verified PNG or PDF bytes and SHA-256 ETag, no-store and nosniff headers, or 404 `piece_not_found`, 409 `assets_required`, 409 `asset_integrity`
- **AND** visual approval through existing PATCH review supplies expected_assets with image_sha256 and carousel_sha256, and can also answer 409 `assets_required`, `asset_integrity` or `asset_conflict`
- **AND** Mastodon sign-in, connection and export operations follow `Mastodon and export operations` below
- **AND** `GET /v1/pieces` answers 200 or 400 `invalid_request` (`draft-review`), and `GET /v1/session` answers 200 with a session token
- **AND** `GET /`, `GET /review/{piece_id}`, `GET /review/{piece_id}/edit|render`, `POST /actions/thought`, `POST /review/{piece_id}/{action}` and `GET /studio.css` serve the browser studio; actions are `edit`, `render`, `approve`, `publish` and `reconnect`; `POST /review/{piece_id}/reconnect/complete` finishes native reconnection
- **AND** every operation may answer 403 `host_not_allowed`, `origin_not_allowed`, `session_required` or 500 `internal_error`; other paths answer 404 `not_found` and other methods 405 `method_not_allowed`

## ADDED Requirements

### Requirement: Native account reconnect
The browser review SHALL offer functional native reconnect for stored credentials that cannot post, reusing the existing integration's sign-in owner and shared Host, Origin and server-session protections.

#### Scenario: Missing or unverified permission
- **WHEN** review reports posting_permission_missing or posting_permission_unverified
- **THEN** its plain-language reason is visible, Publish is disabled and Reconnect account submits a protected native form
- **AND** POST /review/{piece_id}/reconnect starts the existing provider consent flow and exposes its sign-in link without a client script or client-side secret
- **AND** POST /review/{piece_id}/reconnect/complete finishes that supported flow and returns to the same review without approving or publishing
- **AND** Threads carries its existing full callback URL handoff without new redirect policy
- **AND** missing or invalid CSRF, wrong Host and foreign Origin refuse before any provider operation

#### Scenario: Unconfirmed issued-token identity
- **WHEN** successful X or Threads issuance has not confirmed that new token's expected canonical identity
- **THEN** the connection reports cannot_post, reason identity_unconfirmed, action sign_in and message account identity could not be confirmed. Sign in again.
- **AND** the fixed identity_unconfirmed error uses HTTP403 and its contract fixture without reflecting submitted data
