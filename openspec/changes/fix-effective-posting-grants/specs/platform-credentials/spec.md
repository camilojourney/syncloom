## MODIFIED Requirements

### Requirement: Connection states
A platform connection SHALL report stored credential readiness for the selected action without a platform call.

#### Scenario: States from the token
- **WHEN** no token is stored
- **THEN** the connection is `signed_out` with no `expires_at`
- **AND** an unexpired token with effective posting permission is `signed_in` with `expires_at` and `scope`
- **AND** with a token whose `expires_at` is now or earlier it is `expired` with `expires_at`
- **AND** while a sign-in waits for the browser it is `awaiting_consent`
- **AND** the fake platform received no request while the connection was read

#### Scenario: Reduced or unverified grant
- **WHEN** an unexpired stored credential lacks the selected action's effective posting permission
- **THEN** the connection is `cannot_post` with reason `posting_permission_missing`, message `connected, cannot post: posting permission missing.`, and action `sign_in`
- **AND** absent evidence is instead reported as `posting_permission_unverified`, never a known denial
- **AND** browser review disables Publish and names the reason; the connection action identifies the existing sign-in operation

### Requirement: Explicit renewable-session refresh
Renewable integrations SHALL refresh only on explicit request, durably recording successful rotations and preserving existing credentials when token exchange fails.

#### Scenario: No implicit renewal
- **WHEN** a dispatch finds an expired access token
- **THEN** the existing absent-or-expired policy applies and no network request is made
- **AND** the founder can explicitly refresh X or Bluesky credentials, or unexpired eligible Threads credentials

#### Scenario: Successful rotation narrows permission
- **WHEN** token exchange succeeds but the rotated credential has reduced or unverifiable effective posting permission
- **THEN** the actual rotated credential is retained as not publish-ready and the precise permission error is returned
- **AND** the previous credential's broader grant never authorizes the rotated token

## ADDED Requirements

### Requirement: Effective posting permission
Publishing SHALL require effective permission for the selected action from the same credential used for dispatch.

#### Scenario: Reduced saved grant
- **WHEN** a currently saved credential lacks the required posting permission
- **THEN** publishing records `posting_permission_missing` without a platform write
- **AND** substring scope matches and requested scopes do not authorize posting
- **AND** text delivery does not require offline or media-only permission

#### Scenario: Legacy permission evidence
- **WHEN** historical credentials contain requested scopes without effective-grant evidence
- **THEN** publishing records `posting_permission_unverified` without a platform request
- **AND** reconnection can establish actual granted permissions

#### Scenario: Failed exchange or renewal
- **WHEN** fresh sign-in returns reduced or unverifiable permission evidence
- **THEN** it returns the precise fixed permission error without replacing saved credentials
- **AND** verified full-grant sign-in and renewal remain supported
