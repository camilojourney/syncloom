# X integration

## Purpose

Own the founder's X PKCE connection and text or PNG publishing through the current v2 API, without engagement automation.

## ADDED Requirements

### Requirement: X sign-in
`POST /v1/x/sign-in` and `GET /v1/x/sign-in` SHALL start PKCE authorization and complete its loopback callback respectively, returning an authorization URL, connection or fixed sign-in errors.

#### Scenario: Public native client
- **WHEN** the browser returns the matching state and code
- **THEN** token exchange includes the exact loopback redirect, client id and S256 verifier, never a client secret
- **AND** the requested scopes include tweet.read, tweet.write, users.read, media.write and offline.access
- **AND** wrong state, denial, timeout or rejected credentials store nothing

### Requirement: X connection and refresh operations
`GET|DELETE /v1/x/connection` and `POST /v1/x/refresh` SHALL return connections or fixed sign-in errors, using `platform-credentials` for storage and deletion.

#### Scenario: Rotating refresh token
- **WHEN** refresh succeeds with a replacement refresh token
- **THEN** subsequent refresh uses that replacement and no post is sent
- **AND** absent or expired access tokens do not silently refresh during dispatch

### Requirement: X publishing
An X attempt SHALL send approved text and optional PNG through v2 media and post operations, using `dispatch-outbox` for delivery and truthful outcomes.

#### Scenario: PNG and metadata
- **WHEN** the piece includes a verified PNG
- **THEN** initialize, append and finalize complete before alt metadata and exactly one POST /2/tweets
- **AND** media failures prevent post creation, and incomplete processing is never treated as ready

#### Scenario: Truthful result
- **WHEN** X returns a successful post response with a numeric id
- **THEN** the outcome is posted with its x.com status URL
- **AND** HTTP 401, 429, other 4xx, 5xx, timeout, no connection and malformed success use the existing truthful failure mapping without hidden retries
