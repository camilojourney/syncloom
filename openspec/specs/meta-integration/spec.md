# meta-integration Specification

## Purpose
Own Facebook Login for Business and API publishing to an owned Facebook Page and its linked Instagram professional account.

## Requirements

### Requirement: Meta consent operations
Meta sign-in SHALL expose begin, wait, connection and sign-out operations using platform-credentials and the existing protected loopback boundary.

#### Scenario: Consent
- **WHEN** POST /v1/meta/sign-in starts consent and GET /v1/meta/sign-in receives its matching callback
- **THEN** the dialog uses the configured business login configuration and code response
- **AND** short-lived user credentials are exchanged for long-lived user credentials before deriving the selected Page token
- **AND** only that Page token and linked Instagram id persist in the Keychain
- **AND** GET /v1/meta/connection returns the connection and DELETE signs out
- **AND** missing configuration returns meta_not_configured; wrong state, denial and timeout use existing sign-in errors

#### Scenario: Wrong account or permissions
- **WHEN** the configured Page is absent, a required permission is missing, or content creation tasks are absent
- **THEN** sign-in fails without replacing the stored token

### Requirement: Page publishing
Facebook Page delivery SHALL publish approved text or one rendered image using dispatch-outbox and return the Page post id rather than a photo id.

#### Scenario: Photo
- **WHEN** an approved Page piece has rendered assets and facebook delivery is granted
- **THEN** one photos request uses its approved image, caption and alt text
- **AND** a response containing only a photo id is unknown, never posted

### Requirement: Instagram publishing
Instagram delivery SHALL publish one to ten approved JPEG slides as a single image or ordered carousel using dispatch-outbox.

#### Scenario: Carousel
- **WHEN** a three-slide piece is approved and instagram delivery is granted
- **THEN** three child containers precede one carousel parent and one media_publish
- **AND** child alt text follows its slide and AI disclosure is on the parent only
- **AND** only FINISHED containers are published; ERROR, EXPIRED and bounded processing timeout prevent media_publish
- **AND** the published id and verified Instagram permalink determine posted

#### Scenario: No media
- **WHEN** an Instagram attempt has no approved JPEG slides
- **THEN** delivery is refused without Graph calls

### Requirement: Explicit public media delivery
MediaDelivery SHALL accept approved image bytes and MIME type and return a public HTTPS URL or a delivery failure without inventing hosting.

#### Scenario: Unconfigured hosting
- **WHEN** no media delivery implementation is installed
- **THEN** image delivery is refused before any publishing request
- **AND** fake tests supply public-shaped URLs without external network access
- **AND** credentials, local paths, private URLs and arbitrary caller URLs are not accepted as media locations

### Requirement: Truthful Graph outcomes
Meta delivery SHALL use dispatch-outbox outcomes without hidden retries or native scheduling parameters.

#### Scenario: Failure evidence
- **WHEN** Graph returns invalid credentials, permission refusal, throttling, server failure, malformed success or a lost publish response
- **THEN** the outcomes are consent_required, refused, rate_limited, unknown, unknown and unknown respectively
- **AND** uncertain or posted attempts are never blindly repeated
