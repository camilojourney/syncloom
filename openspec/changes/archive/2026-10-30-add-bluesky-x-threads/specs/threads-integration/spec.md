# Threads integration

## Purpose

Own Meta Threads consent, renewable long-lived credentials and text delivery without claiming a local redirect or public image hosting exists.

## ADDED Requirements

### Requirement: Threads consent operations
`POST /v1/threads/sign-in` and `POST /v1/threads/complete` SHALL start consent and accept `callback_url` respectively, returning an authorization URL, connection or fixed sign-in errors.

#### Scenario: Explicit redirect required
- **WHEN** no HTTPS non-localhost redirect is configured
- **THEN** sign-in fails as not configured without binding a callback listener or inventing a host
- **AND** a configured flow requires the exact callback origin/path and matching one-time state before exchanging its code
- **AND** only successful short-to-long token exchange stores credentials

### Requirement: Threads connection and refresh operations
`GET|DELETE /v1/threads/connection` and `POST /v1/threads/refresh` SHALL return connections or fixed sign-in errors using `platform-credentials`.

#### Scenario: Refresh window
- **WHEN** an unexpired long-lived token is at least 24 hours old
- **THEN** explicit refresh can replace it and retain the new returned lifetime
- **AND** younger or expired tokens send no refresh request

### Requirement: Threads text publishing
Threads SHALL publish approved text through container creation and publication, using `dispatch-outbox` for delivery and truthful outcomes.

#### Scenario: Text container
- **WHEN** an approved text-only piece is delivered
- **THEN** a TEXT container is created, its readiness checked and exactly one publication occurs
- **AND** a successful id and verified Threads permalink are required for posted
- **AND** media input is refused without uploading or fetching any URL

#### Scenario: Errors
- **WHEN** Meta returns authorization error code 190, rate-limit code 4 or 32, another 4xx, 5xx, malformed success or timeout
- **THEN** the outcome is consent_required, rate_limited, refused, unknown, unknown or unknown respectively with no hidden retry
