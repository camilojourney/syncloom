## ADDED Requirements

### Requirement: Browser studio
`GET /` and `GET /review/{piece_id}` SHALL show saved thoughts and pieces, draft editing, visual rendering, exact approval and explicit publishing through existing core operations.

#### Scenario: Founder reviews a piece
- **WHEN** the founder opens the page on the laptop
- **THEN** saved thoughts and pieces are discoverable, including pieces created by the CLI
- **AND** a selected piece shows its source, editable draft, status and full revision
- **AND** ordinary forms create drafts, save edits, render visuals, approve and publish without client-side scripts
- **AND** empty and error states explain recovery, and the browser indicates loading during navigation

### Requirement: Phone-sized asset previews
The review page SHALL display current PNG and PDF assets at phone-sized widths with their full SHA-256 hashes and links to their verified bytes.

#### Scenario: Render and edit
- **WHEN** a rendered piece is opened
- **THEN** the PNG and PDF are previewed at no more than 360 CSS pixels wide, with the PDF's page count
- **AND** approval submits the displayed revision and both displayed hashes, as required by `visual-assets`
- **AND** editing clears approval and old previews according to `draft-review` and `visual-assets`
- **AND** narrow layouts have no horizontal page overflow

### Requirement: Truthful browser outcomes
The review page SHALL display the core's publish outcome without equating contained, refused or unknown outcomes with posted.

#### Scenario: Contained publish
- **WHEN** an approved piece is published without a delivery grant
- **THEN** the page says `contained`, explains that nothing was sent, and shows the request id and attempt count
- **AND** repeating returns the same intent, and an unapproved piece cannot be published

### Requirement: Origin and browser-site checks
Every request SHALL reject foreign, null or repeated Origin headers and cross-site browser fetches with 403 `origin_not_allowed` before any core operation.

#### Scenario: Local origin only
- **WHEN** Origin is present
- **THEN** only the single exact `http://127.0.0.1:<bound port>` is accepted
- **AND** foreign ports, localhost, opaque null and multiple origins are rejected, even with a valid session token
- **AND** absent Origin is accepted for non-browser clients, but `Sec-Fetch-Site: cross-site` or `same-site` is rejected
- **AND** rejected requests store nothing, invoke no provider and never echo supplied values

### Requirement: Server-session token
Every state-changing HTTP request SHALL require the current server-session token, otherwise answering 403 `session_required` without invoking core operations.

#### Scenario: Session handshake
- **WHEN** a client calls `GET /v1/session` through the guarded loopback origin
- **THEN** it receives 200 with `{"token":<unpredictable token>}`
- **AND** API mutations send it once in `X-Syncloom-Token`; browser forms send it once in `csrf`
- **AND** missing, incorrect, repeated or another server's tokens fail
- **AND** the token changes on restart, is not placed in URLs, and is never logged

### Requirement: Browser response protections
Every response SHALL prevent caching, MIME sniffing, framing by other pages, referrer disclosure, scripts and off-origin resource loading.

#### Scenario: Stored markup is inert
- **WHEN** saved thoughts, draft text or slide text contain HTML metacharacters
- **THEN** the page displays them as text rather than executable markup
- **AND** responses include no-store, nosniff, no-referrer, DENY framing and a restrictive Content-Security-Policy
- **AND** no CORS response headers are added

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
- **AND** `GET /v1/pieces` answers 200 or 400 `invalid_request` (`draft-review`), and `GET /v1/session` answers 200 with a session token
- **AND** `GET /`, `GET /review/{piece_id}`, `POST /actions/thought`, `POST /review/{piece_id}/{action}` and `GET /studio.css` serve the browser studio; actions are `edit`, `render`, `approve` and `publish`
- **AND** every operation may answer 403 `host_not_allowed`, `origin_not_allowed`, `session_required` or 500 `internal_error`; other paths answer 404 `not_found` and other methods 405 `method_not_allowed`


### Requirement: Error envelope
Every API error response SHALL be a JSON object `{"error":{"code","message","request_id"}}` with a fixed message per code.

#### Scenario: Message never echoes the request
- **WHEN** a request carrying canary text in its path, headers and body fails
- **THEN** the message equals the message in `contracts/studio-api/errors/<code>.json`
- **AND** no canary text appears anywhere in the response

#### Scenario: Fixtures equal the server
- **WHEN** each error code is produced against the server with `X-Request-Id: fixture-request-id`
- **THEN** status and body equal the fixture file for that code
- **AND** the fixture directory holds exactly the documented codes

#### Scenario: Browser errors
- **WHEN** a browser page or form operation fails
- **THEN** the HTTP status, fixed code and fixed message equal the API failure
- **AND** an HTML error page shows its request id and a link back to the studio without reflecting the submitted content

