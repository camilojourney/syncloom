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
- **AND** `GET /`, `GET /review/{piece_id}`, `GET /review/{piece_id}/edit|render`, `POST /actions/thought`, `POST /review/{piece_id}/{action}` and `GET /studio.css` serve the browser studio; actions are `edit`, `render`, `approve`, `accept-cost` and `publish`
- **AND** every operation may answer 403 `host_not_allowed`, `origin_not_allowed`, `session_required` or 500 `internal_error`; other paths answer 404 `not_found` and other methods 405 `method_not_allowed`


### Requirement: Saved-channel browser identity
The browser SHALL identify saved pieces using proper platform display names without changing machine channel identities.

#### Scenario: Non-LinkedIn pieces
- **WHEN** saved pieces appear on discovery cards, review headings and text previews
- **THEN** the names are LinkedIn, Mastodon, Facebook, Instagram, Bluesky, X and Threads for their respective channels, and Manual export for local handoff
- **AND** delivery guidance retains the existing platform-specific containment rules

## ADDED Requirements

### Requirement: Browser X cost consent
`POST /review/{piece_id}/accept-cost` SHALL record session consent for the current revision, micro-USD estimate and pricing date without publishing.

#### Scenario: Accept displayed estimate
- **WHEN** the operator submits csrf, expected_revision, accepted_cost_microusd and pricing_as_of matching the displayed current X quote
- **THEN** the response redirects to the review page and records consent for that exact revision and estimate
- **AND** Publish is natively disabled until consent exists and ordinary approval requirements hold, using the same style as unavailable Threads image delivery
- **AND** after consent the page displays acceptance and permits a separate Publish action
- **AND** the pricing date stays on one line at 390 CSS pixels

#### Scenario: Invalid or stale consent
- **WHEN** acceptance submits a stale revision, mismatched estimate/date, malformed fields or a non-X piece
- **THEN** no consent is recorded; errors are respectively revision_conflict, cost_confirmation_required or invalid_request under the existing browser error boundary
- **AND** missing or foreign session and Origin checks follow the existing server-session and origin requirements

#### Scenario: Consent cannot follow changed content or prices
- **WHEN** a piece's revision or current estimate differs from its accepted record
- **THEN** the record no longer enables Publish or supplies confirmation
- **AND** server restart also requires fresh acceptance
- **AND** a browser Publish without a matching record cannot bypass the existing core cost refusal, even if a forged accepted amount is submitted
