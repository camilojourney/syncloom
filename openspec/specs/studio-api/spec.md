# studio-api Specification

## Purpose
Own the loopback HTTP boundary of the studio: bind address, Host rule, request ids, the error envelope, request logging, the operations table and the fixtures in `contracts/studio-api/`. Behavior behind each operation belongs to `thought-drafting`, `draft-review` and `dispatch-outbox`.

## Requirements

### Requirement: Loopback bind
The server SHALL listen only on 127.0.0.1 and refuse to start when SYNCLOOM_ADDR names any other address.

#### Scenario: Default address
- **WHEN** the server starts without `SYNCLOOM_ADDR`
- **THEN** it listens on `127.0.0.1:8790`

#### Scenario: Bound address announced
- **WHEN** the server is ready with `SYNCLOOM_ADDR=127.0.0.1:0`
- **THEN** it writes one JSON log line with `message` `listening` and `address` holding the bound socket address

#### Scenario: Other addresses refused
- **WHEN** `SYNCLOOM_ADDR` is `0.0.0.0:8790`, `127.0.0.2:8790`, `192.168.1.1:8790`, `[::1]:8790`, `localhost:8790` or not a socket address
- **THEN** the process exits with an error naming `SYNCLOOM_ADDR` and binds no port

### Requirement: Host check
The server SHALL answer 403 `host_not_allowed` to every request whose Host header is not exactly `127.0.0.1:` followed by the bound port.

#### Scenario: Rebinding host refused
- **WHEN** a request carries `Host: evil.example:<port>`, `localhost:<port>`, `127.0.0.1` without a port, `[::1]:<port>`, a different port, or no Host header
- **THEN** status is 403 with code `host_not_allowed`
- **AND** the model provider is not called and nothing is stored

#### Scenario: Two Host headers
- **WHEN** a request carries two Host headers, even when one is `127.0.0.1:<port>`
- **THEN** status is 403 with code `host_not_allowed`

#### Scenario: Every path is guarded
- **WHEN** a request with a wrong Host targets `/v1/health`, `/v1/thoughts`, `/v1/pieces/<id>` or an unknown path
- **THEN** status is 403 with code `host_not_allowed`

### Requirement: No cross-origin access
The server SHALL never send CORS headers.

#### Scenario: Browser-style requests
- **WHEN** a request carries an `Origin` header, or is an `OPTIONS` preflight
- **THEN** no response header name starts with `access-control-`

### Requirement: Health
`GET /v1/health` SHALL answer 200 with the status and the server crate version.

#### Scenario: Ready without a key
- **WHEN** a client requests `GET /v1/health` and no provider key is configured
- **THEN** status is 200 and the body is `{"status":"ok","version":"<crate version>"}`

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
- **AND** `GET /`, `GET /review/{piece_id}`, `GET /review/{piece_id}/edit|render`, `POST /actions/thought`, `POST /review/{piece_id}/{action}` and `GET /studio.css` serve the browser studio; actions are `edit`, `render`, `approve` and `publish`
- **AND** every operation may answer 403 `host_not_allowed`, `origin_not_allowed`, `session_required` or 500 `internal_error`; other paths answer 404 `not_found` and other methods 405 `method_not_allowed`

### Requirement: Request id
The server SHALL return in the `X-Request-Id` response header a request id kept from a valid request header or generated as a UUID v4.

#### Scenario: Valid id kept
- **WHEN** the request carries `X-Request-Id` of 1 to 128 characters from `A-Za-z0-9._:-`, such as `Req.1:a_b-2`
- **THEN** the response header and any error envelope carry that value

#### Scenario: Invalid id replaced
- **WHEN** the header is absent, empty, 129 characters, or contains a space, `/`, a tab or a non-ASCII byte
- **THEN** the response header carries a UUID v4

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

### Requirement: Invalid request body
A documented operation SHALL answer 400 `invalid_request` when its body is not a JSON object with exactly the documented fields and types.

#### Scenario: Malformed bodies
- **WHEN** `POST /v1/thoughts` receives a body that is not JSON, lacks `text`, has `text` as a number, has an unknown field, or has `channels` empty or missing
- **THEN** status is 400 with code `invalid_request`
- **AND** the model provider is not called

### Requirement: Unknown path
The server SHALL answer 404 `not_found` for any path other than the documented operations.

#### Scenario: Unknown path
- **WHEN** a client requests `GET /missing`
- **THEN** status is 404 with code `not_found`

### Requirement: Wrong method
The server SHALL answer 405 `method_not_allowed` when a documented path receives an unsupported method.

#### Scenario: Wrong method
- **WHEN** a client sends `GET /v1/thoughts` or `POST /v1/health`
- **THEN** status is 405 with code `method_not_allowed`

### Requirement: Internal failure
The server SHALL answer 500 `internal_error` without details when handling a request fails unexpectedly.

#### Scenario: Unexpected panic
- **WHEN** the model provider panics while drafting
- **THEN** status is 500 with code `internal_error` and the standard envelope
- **AND** the server keeps serving later requests

### Requirement: Request log
The server SHALL write one JSON log line per request with the request id, a fixed route label, status, error code and timings, never content or secrets.

#### Scenario: Content stays out of the logs
- **WHEN** a thought carrying canary text is drafted, shown, approved and published, and a LinkedIn sign-in and connection read happen
- **THEN** each request produced one line with `request_id`, `route`, `status`, `total_ms` and `outcome`
- **AND** the route labels are `/v1/thoughts`, `/v1/pieces/{piece_id}`, `/v1/pieces/{piece_id}/publish`, `/v1/linkedin/sign-in`, `/v1/linkedin/connection`, `/v1/pieces/{piece_id}/export`, `/v1/mastodon/sign-in`, `/v1/mastodon/connection` or `unmatched`
- **AND** the session, browser and asset route labels added by their capabilities remain unchanged
- **AND** the canary text, the draft text, the provider key, both platform tokens, the session token and the client secret appear in no log line

### Requirement: Hermetic tests
`cargo test` SHALL use temporary directories only and leave `git status --porcelain --ignored` unchanged.

#### Scenario: Tree unchanged by a test run
- **WHEN** `git status --porcelain --ignored` is captured, `cargo test` runs, and it is captured again
- **THEN** the two captures are identical

### Requirement: Repository checks
A change SHALL land only when fmt, clippy with denied warnings, the tests and strict OpenSpec validation all pass.

#### Scenario: Gate commands
- **WHEN** `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` and `openspec validate --all --strict --no-interactive` run on the change
- **THEN** every command exits 0

### Requirement: Browser studio
`GET /` and `GET /review/{piece_id}` SHALL show saved thoughts and pieces, draft editing, visual rendering, exact approval and explicit publishing through existing core operations.

#### Scenario: Founder reviews a piece
- **WHEN** the founder opens the page on the laptop
- **THEN** saved thoughts and pieces are discoverable, including pieces created by the CLI
- **AND** a selected piece shows its source, saved draft, status and full revision; a separate editor has no approval controls
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
Every response SHALL prevent caching, MIME sniffing, framing by other pages, off-origin referrer disclosure, scripts and off-origin resource loading.

#### Scenario: Stored markup is inert
- **WHEN** saved thoughts, draft text or slide text contain HTML metacharacters
- **THEN** the page displays them as text rather than executable markup
- **AND** responses include no-store, nosniff, same-origin referrer policy and a restrictive Content-Security-Policy; pages deny framing, while verified PDFs permit only same-origin framing
- **AND** no CORS response headers are added

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

### Requirement: Q1 connection surface
The API SHALL expose the three Q1 integration operations through the existing boundary and fixed error envelope.

#### Scenario: Operations and errors
- **WHEN** a client calls the Bluesky, X or Threads operations defined by their integration capabilities
- **THEN** existing Host, Origin and mutation-session checks apply without CORS
- **AND** missing app settings return 503 platform_not_configured; unconfirmed X delivery returns 409 cost_confirmation_required
- **AND** error responses match their fixed contract fixtures and never echo secret input
