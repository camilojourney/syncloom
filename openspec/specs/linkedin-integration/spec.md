# linkedin-integration Specification

## Purpose
Own the LinkedIn integration for the founder's personal profile: its configuration, the browser sign-in over a loopback redirect, the Posts API request for a text piece, and the mapping of LinkedIn's answers to dispatch outcomes. Token storage and connection states belong to `platform-credentials`; the intent and its repeat rules to `dispatch-outbox`.

## Requirements

### Requirement: LinkedIn configuration
The studio SHALL refuse to start when `SYNCLOOM_LINKEDIN_REDIRECT_URL`, `SYNCLOOM_LINKEDIN_AUTH_BASE_URL`, `SYNCLOOM_LINKEDIN_API_BASE_URL` or `SYNCLOOM_LINKEDIN_AUTH_FLOW` holds a value outside its rule.

#### Scenario: Redirect URL rule
- **WHEN** the redirect URL is `http://127.0.0.1:8791/linkedin/callback` (the default) or `http://localhost:8791/linkedin/callback`
- **THEN** it is accepted and the callback listener binds `127.0.0.1` on that port
- **AND** `https://127.0.0.1:8791/cb`, `http://127.0.0.1/cb`, `http://192.168.1.1:8791/cb`, `http://127.0.0.1:8791/cb?x=1`, `http://u:p@127.0.0.1:8791/cb` and `not a url` are refused with an error naming `SYNCLOOM_LINKEDIN_REDIRECT_URL`

#### Scenario: Flow and base URL rules
- **WHEN** `SYNCLOOM_LINKEDIN_AUTH_FLOW` is `standard` (the default) or `native_pkce`
- **THEN** it is accepted, and any other value is refused with an error naming the variable
- **AND** the two base URLs default to `https://www.linkedin.com` and `https://api.linkedin.com` and accept only `https://<host>` or `http://127.0.0.1:<port>`, refusing other values with an error naming the variable

### Requirement: Sign-in begins with the authorization URL
`POST /v1/linkedin/sign-in` SHALL answer 202 with the authorization URL and the deadline, or 503 `linkedin_not_configured`, or 409 `sign_in_in_progress`.

#### Scenario: Standard flow URL
- **WHEN** the flow is `standard` and the client id and secret are set
- **THEN** status is 202 with `{"status":"awaiting_consent","authorization_url":<url>,"expires_at":<instant>}`
- **AND** the URL is the auth base plus `/oauth/v2/authorization` with `response_type=code`, the client id, the exact redirect URL, a `state` of at least 43 URL-safe characters, `scope` `openid profile w_member_social` and no `code_challenge`

#### Scenario: Native PKCE flow URL
- **WHEN** the flow is `native_pkce`
- **THEN** the URL is the auth base plus `/oauth/native-pkce/authorization` with the same parameters plus `code_challenge_method=S256` and a `code_challenge`

#### Scenario: Not configured
- **WHEN** `SYNCLOOM_LINKEDIN_CLIENT_ID` or `SYNCLOOM_LINKEDIN_CLIENT_SECRET` is absent or blank
- **THEN** status is 503 with code `linkedin_not_configured` and no listener is bound

#### Scenario: Already waiting
- **WHEN** a sign-in is waiting for the browser and a second `POST /v1/linkedin/sign-in` arrives
- **THEN** status is 409 with code `sign_in_in_progress`

### Requirement: Sign-in completes or fails truthfully
`GET /v1/linkedin/sign-in` SHALL wait for the pending sign-in and answer 200 with the `signed_in` connection, or the code naming how it failed.

#### Scenario: Consent
- **WHEN** the browser follows the authorization URL and the fake redirects to the callback with a code and the same state
- **THEN** one token request reaches `/oauth/v2/accessToken` with `grant_type=authorization_code`, the code, the exact redirect URL, the client id and, for `standard`, the client secret and no `code_verifier`, or, for `native_pkce`, a `code_verifier` whose S256 digest equals the challenge and no client secret
- **AND** one request reaches `/v2/userinfo` with the new token
- **AND** status is 200 with `{"status":"signed_in","expires_at":<now plus expires_in>,"scope":<scope>}` and the token is stored

#### Scenario: Denied
- **WHEN** the callback carries `error=user_cancelled_authorize` and the state
- **THEN** status is 403 with code `sign_in_denied`, no token request is made and nothing is stored

#### Scenario: Wrong state
- **WHEN** the callback carries a code and a state that differs from the one issued
- **THEN** status is 403 with code `sign_in_state_mismatch`, no token request is made and nothing is stored

#### Scenario: Token refused
- **WHEN** the token endpoint answers 400 or 401
- **THEN** status is 502 with code `sign_in_rejected` and nothing is stored

#### Scenario: No callback
- **WHEN** no callback arrives within the sign-in deadline
- **THEN** status is 504 with code `sign_in_timed_out`, the listener is closed and a new `POST /v1/linkedin/sign-in` is accepted

#### Scenario: Nothing pending
- **WHEN** `GET /v1/linkedin/sign-in` is called with no sign-in waiting
- **THEN** status is 409 with code `sign_in_not_started`

### Requirement: Callback listener answers a fixed page
The callback listener SHALL answer only the registered path with a fixed page that echoes nothing, and ignore every callback after the first.

#### Scenario: Fixed page and other paths
- **WHEN** the callback arrives with an `error_description` carrying canary text
- **THEN** the page returned is 200 without the canary text
- **AND** a request to another path on the listener answers 404
- **AND** a second callback with another code changes nothing and causes no second token request

### Requirement: Connection operations
`GET /v1/linkedin/connection` and `DELETE /v1/linkedin/connection` SHALL answer 200 with the connection, the latter after signing out.

#### Scenario: Read and delete
- **WHEN** a client reads the connection after a sign-in, then deletes it, then reads it again
- **THEN** the answers are `signed_in`, `signed_out` and `signed_out`

### Requirement: One Posts API request per attempt
A dispatch attempt SHALL send exactly one `POST /rest/posts` to the API base with the versioned headers and the piece text as `commentary`.

#### Scenario: Request shape
- **WHEN** an approved piece is published with delivery granted and a valid token
- **THEN** the fake received exactly one request with headers `Authorization: Bearer <the stored token>`, `LinkedIn-Version: 202610`, `X-Restli-Protocol-Version: 2.0.0` and a JSON body
- **AND** the body is `{"author":"urn:li:person:<member id>","commentary":<piece text>,"visibility":"PUBLIC","distribution":{"feedDistribution":"MAIN_FEED","targetEntities":[],"thirdPartyDistributionChannels":[]},"lifecycleState":"PUBLISHED","isReshareDisabledByAuthor":false}`
- **AND** no header or body field carries a name, an email address or a phone number

### Requirement: LinkedIn answers map to one outcome
LinkedIn's answer SHALL map to one dispatch outcome: 201 with `x-restli-id` `posted`; 401 `consent_required`; 429 `rate_limited`; other 4xx `refused`; 5xx, timeout or no id `unknown`; no connection `unreachable`.

#### Scenario: Posted with id and URL
- **WHEN** the fake answers 201 with `x-restli-id: urn:li:share:7000000000000000001`
- **THEN** the publish answers `posted` with `post_id` `urn:li:share:7000000000000000001` and `post_url` `https://www.linkedin.com/feed/update/urn:li:share:7000000000000000001/`

#### Scenario: Each failure
- **WHEN** the fake answers 401, 403, 422, 429, 500, or 201 without `x-restli-id`, or waits longer than the platform deadline, or its port is closed
- **THEN** the publish answers `consent_required`, `refused`, `refused`, `rate_limited`, `unknown`, `unknown`, `unknown` and `unreachable` respectively, each with `post_id` null
