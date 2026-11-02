## MODIFIED Requirements

### Requirement: Sign-in completes or fails truthfully
`GET /v1/linkedin/sign-in` SHALL wait for the pending sign-in and answer 200 with the `signed_in` connection only when effective posting permission is granted, or the code naming how it failed.

#### Scenario: Consent
- **WHEN** the browser follows the authorization URL and the fake redirects to the callback with a code and the same state, and the token response grants exact `w_member_social`
- **THEN** one token request reaches `/oauth/v2/accessToken` with `grant_type=authorization_code`, the code, the exact redirect URL, the client id and, for `standard`, the client secret and no `code_verifier`, or, for `native_pkce`, a `code_verifier` whose S256 digest equals the challenge and no client secret
- **AND** one request reaches `/v2/userinfo` with the new token
- **AND** status is 200 with `{"status":"signed_in","expires_at":<now plus expires_in>,"scope":<scope>}` and the token is stored

#### Scenario: Reduced grant
- **WHEN** the token response lacks exact `w_member_social`
- **THEN** status is 403 with code `posting_permission_missing`, no userinfo or post request occurs, and the reduced grant does not replace stored credentials
- **AND** the existing sign-in operation remains available for reconnecting

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
