# platform-credentials Specification

## Purpose
Own where a platform's token lives, the connection states the studio reports from it, and the rule that no secret ever leaves the store. Which platform operations exist belongs to that platform's capability (`linkedin-integration`).

## Requirements

### Requirement: Token stored only in the Keychain
A platform token SHALL be stored only as one macOS Keychain generic-password item under the configured service name and the platform's account name.

#### Scenario: Stored after sign-in
- **WHEN** a LinkedIn sign-in completes
- **THEN** the Keychain holds one item with service `syncloom` (or the value of `SYNCLOOM_KEYCHAIN_SERVICE`) and account `linkedin` whose secret is the token bundle
- **AND** no file under the data directory and no log line contains the token

#### Scenario: Test-only service name
- **WHEN** `SYNCLOOM_KEYCHAIN_SERVICE` is `syncloom-test-<id>`
- **THEN** the item is written under that service, read back from it, and deleted by sign-out, leaving no item behind

### Requirement: Connection states
A platform connection SHALL be reported as `signed_out`, `awaiting_consent`, `signed_in` or `expired` from the stored token and the clock, never from a platform call.

#### Scenario: States from the token
- **WHEN** no token is stored
- **THEN** the connection is `signed_out` with no `expires_at`
- **AND** with a token whose `expires_at` is after now it is `signed_in` with `expires_at` and `scope`
- **AND** with a token whose `expires_at` is now or earlier it is `expired` with `expires_at`
- **AND** while a sign-in waits for the browser it is `awaiting_consent`
- **AND** the fake platform received no request while the connection was read

### Requirement: Sign-out deletes the token
Signing out SHALL delete the platform's Keychain item and answer the `signed_out` connection, also when no item exists.

#### Scenario: Sign out twice
- **WHEN** the founder signs out after a sign-in, then signs out again
- **THEN** both answers are `signed_out` and the Keychain holds no item for the service and account

### Requirement: Absent or expired token sends nothing
A dispatch attempt SHALL be recorded as `consent_required` with no platform request when the platform's token is absent or expired.

#### Scenario: Expired token
- **WHEN** an approved piece is published with delivery granted and the stored token's `expires_at` is now or earlier
- **THEN** the publish answers `consent_required`, the fake platform received no request and the token is unchanged

#### Scenario: No token
- **WHEN** an approved piece is published with delivery granted and no token is stored
- **THEN** the publish answers `consent_required` and the fake platform received no request

### Requirement: Secrets never leave the store
No response body, event, log line, error message or command output SHALL contain a platform token, client secret, authorization code, state value or code verifier.

#### Scenario: Canary secrets
- **WHEN** the fake platform issues a token containing a canary word and the sign-in uses a client secret containing another canary word
- **THEN** neither word appears in any API response, in `provenance/events.jsonl`, in the server log lines or in the CLI's stdout and stderr
