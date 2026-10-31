# Mastodon integration

## Purpose

Own Mastodon's instance registration, OAuth connection and approved-content API publishing, independently of review and outbox rules.

## ADDED Requirements

### Requirement: Per-instance OAuth sign-in
Mastodon sign-in SHALL register an app on the configured instance and exchange a matching loopback OAuth callback for an instance-scoped token.

#### Scenario: Consent
- **WHEN** sign-in starts on the explicitly configured instance
- **THEN** one app registration uses a generic client name, loopback redirect and `write:statuses write:media` scope
- **AND** the returned authorization URL leads to that instance, and completion stores the token through `platform-credentials`
- **AND** wrong state, denied consent, token rejection and deadline return the existing fixed sign-in errors without storing a token
- **AND** the callback requires its exact Host, echoes no input and sends no CORS headers

### Requirement: Instance limits checked before publishing
Mastodon publishing SHALL refuse content exceeding the instance's advertised text or image limits without creating a status.

#### Scenario: Runtime limits
- **WHEN** the instance advertises a text limit below the draft's length or an image byte limit below the rendered PNG's size
- **THEN** the outcome is `refused` with no media upload or status creation
- **AND** missing or malformed limits prevent publishing rather than assuming defaults

### Requirement: Text and rendered image publishing
A Mastodon attempt SHALL create one status with the approved text and, when present, the verified rendered PNG with slide-derived alt text.

#### Scenario: Media completes asynchronously
- **WHEN** media upload returns 202 and then processing completes
- **THEN** bounded polling obtains its media id before one status creation using that id
- **AND** the status request carries the durable intent's `Idempotency-Key`
- **AND** the carousel PDF is not uploaded as an image

#### Scenario: Media fails
- **WHEN** media upload or polling fails or processing never completes
- **THEN** no status is created and the outcome is not `posted`

### Requirement: Truthful Mastodon outcomes
Mastodon SHALL report `posted` only from a successful status response with a nonempty id and URL, using `dispatch-outbox` for all other outcomes and repeat safety.

#### Scenario: Outcomes
- **WHEN** status creation answers 401, 403, 422, 429, 500, malformed success or times out
- **THEN** outcomes are respectively `consent_required`, `refused`, `refused`, `rate_limited`, `unknown`, `unknown` and `unknown`
- **AND** no hidden retries occur; a duplicate or uncertain intent is not sent again

#### Scenario: Containment
- **WHEN** no delivery grant names Mastodon
- **THEN** publishing creates only the contained intent and makes no instance, media or status request
