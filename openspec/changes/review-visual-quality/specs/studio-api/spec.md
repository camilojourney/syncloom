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
- **AND** `GET /v1/pieces/{piece_id}/assets/image` and `/assets/carousel` accept optional `sha256` and answer 200 with verified PNG or PDF bytes and SHA-256 ETag, no-store and nosniff headers, or 400 `invalid_request`, 404 `piece_not_found`, 409 `assets_required`, 409 `asset_integrity`, 409 `asset_conflict`
- **AND** visual approval through existing PATCH review supplies expected_assets with image_sha256 and carousel_sha256, and can also answer 409 `assets_required`, `asset_integrity` or `asset_conflict`
- **AND** Mastodon sign-in, connection and export operations follow `Mastodon and export operations` below
- **AND** `GET /v1/pieces` answers 200 or 400 `invalid_request` (`draft-review`), and `GET /v1/session` answers 200 with a session token
- **AND** `GET /`, `GET /review/{piece_id}`, `GET /review/{piece_id}/edit|render`, `POST /actions/thought`, `POST /review/{piece_id}/{action}` and `GET /studio.css` serve the browser studio; actions are `edit`, `render`, `approve` and `publish`
- **AND** `POST /v1/pieces/{piece_id}/visual-quality` follows the Visual assessment operation below
- **AND** every operation may answer 403 `host_not_allowed`, `origin_not_allowed`, `session_required` or 500 `internal_error`; other paths answer 404 `not_found` and other methods 405 `method_not_allowed`

## ADDED Requirements

### Requirement: Visual assessment operation
POST /v1/pieces/{piece_id}/visual-quality SHALL accept expected_revision and expected_assets, returning the local assessment or existing fixed input, identity, integrity and storage errors.

#### Scenario: Protected inspection
- **WHEN** a local client submits the exact rendered revision and hash set with the session token
- **THEN** the response is 200 with the visual-assets assessment
- **AND** malformed or extra fields return invalid_request; unknown pieces return piece_not_found
- **AND** existing Host, Origin, session, no-CORS and content-free logging rules apply

### Requirement: Browser visual evidence
Review pages SHALL co-locate version-bound previews with local technical checks, explicit unassessed quality and recovery guidance without presenting assessment as approval.

#### Scenario: Review states
- **WHEN** a piece is opened without visuals, awaiting render, with valid assets or with unreadable assets
- **THEN** the page respectively explains optional rendering, required rerendering, technical observations or failed inspection
- **AND** semantic and aesthetic quality remain visibly unassessed, with no aggregate passing score
- **AND** full evidence is legible in light and dark themes at narrow widths, all actions support keyboard navigation, and Workspace remains reachable
- **AND** actions already refused by current-asset integrity guards are disabled, without turning technical observations into a new approval condition
