## MODIFIED Requirements

### Requirement: Threads consent operations
`POST /v1/threads/sign-in` and `POST /v1/threads/complete` SHALL start consent and accept `callback_url` respectively, returning an authorization URL, connection or fixed sign-in errors.

#### Scenario: Explicit redirect required
- **WHEN** no HTTPS non-localhost redirect is configured
- **THEN** sign-in fails as not configured without binding a callback listener or inventing a host
- **AND** a configured flow requires the exact callback origin/path and matching one-time state before exchanging its code
- **AND** only successful short-to-long token exchange with verified effective permission stores credentials

#### Scenario: Effective permission evidence
- **WHEN** the long-lived token is inspected through the documented read-only token-debug operation
- **THEN** valid granted `threads_content_publish` evidence permits sign-in
- **AND** reduced scopes return `posting_permission_missing`
- **AND** missing or malformed evidence returns `posting_permission_unverified`
- **AND** requested scopes alone never prove a grant

### Requirement: Threads connection and refresh operations
`GET|DELETE /v1/threads/connection` and `POST /v1/threads/refresh` SHALL return connections or fixed sign-in errors using `platform-credentials`.

#### Scenario: Refresh window
- **WHEN** an unexpired long-lived token is at least 24 hours old
- **THEN** explicit refresh can replace it after verifying the rotated token's effective permissions and retain the new returned lifetime
- **AND** younger or expired tokens send no refresh request
- **AND** a failed exchange preserves the previous credential
- **AND** reduced or unverified permission after successful rotation follows platform-credentials rather than reusing the previous grant
