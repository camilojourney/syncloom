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
- **AND** `POST /v1/pieces/{piece_id}/render` takes exactly `expected_revision`, `template` and `slides` and answers 200 with the piece, or 400 `invalid_request`, 404 `piece_not_found`, 409 `revision_conflict`, 503 `renderer_missing`, 503 `renderer_version_mismatch`, 503 `renderer_font_mismatch`, 504 `renderer_timeout`, 502 `renderer_failed`
- **AND** `GET /v1/pieces/{piece_id}/assets/image` and `/assets/carousel` answer 200 with verified PNG or PDF bytes and SHA-256 ETag, no-store and nosniff headers, or 404 `piece_not_found`, 409 `assets_required`, 409 `asset_integrity`
- **AND** visual approval through existing PATCH review supplies expected_assets with image_sha256 and carousel_sha256, and can also answer 409 `assets_required`, `asset_integrity` or `asset_conflict`
- **AND** every operation may answer 403 `host_not_allowed` or 500 `internal_error`; other paths answer 404 `not_found` and other methods 405 `method_not_allowed`
