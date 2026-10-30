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
- **AND** every operation may answer 403 `host_not_allowed` or 500 `internal_error`; other paths answer 404 `not_found` and other methods 405 `method_not_allowed`

### Requirement: Request id
The server SHALL return in the `X-Request-Id` response header a request id kept from a valid request header or generated as a UUID v4.

#### Scenario: Valid id kept
- **WHEN** the request carries `X-Request-Id` of 1 to 128 characters from `A-Za-z0-9._:-`, such as `Req.1:a_b-2`
- **THEN** the response header and any error envelope carry that value

#### Scenario: Invalid id replaced
- **WHEN** the header is absent, empty, 129 characters, or contains a space, `/`, a tab or a non-ASCII byte
- **THEN** the response header carries a UUID v4

### Requirement: Error envelope
Every error response SHALL be a JSON object `{"error":{"code","message","request_id"}}` with a fixed message per code.

#### Scenario: Message never echoes the request
- **WHEN** a request carrying canary text in its path, headers and body fails
- **THEN** the message equals the message in `contracts/studio-api/errors/<code>.json`
- **AND** no canary text appears anywhere in the response

#### Scenario: Fixtures equal the server
- **WHEN** each error code is produced against the server with `X-Request-Id: fixture-request-id`
- **THEN** status and body equal the fixture file for that code
- **AND** the fixture directory holds exactly the documented codes

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
The server SHALL write one JSON log line per request with the request id, a fixed route label, status, error code and timings, never content.

#### Scenario: Content stays out of the logs
- **WHEN** a thought carrying canary text is drafted, shown, approved and published
- **THEN** each request produced one line with `request_id`, `route`, `status`, `total_ms` and `outcome`
- **AND** the canary text, the draft text and the provider key appear in no log line

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
