## Request

> "aslo build synchroloom rebuild it all as we mentioned and also the integrations wtih al lthe social media apis to make deep research onthem too ... use hte openspec last version with the instruciont conf think about what is next and start building all thos features."

## Approval

pending

Duplicate kept: none.

New capability requested: bluesky-integration
New capability requested: x-integration
New capability requested: threads-integration

## Why

Q1 is imported before specification. More channels can follow contained publishing without inventing access or hosting.

## What Changes

- Bluesky app-password sessions and text/PNG posts using maintained ATrium SDK.
- X PKCE, text/PNG through current v2 endpoints, explicit pre-publish cost confirmation and durable estimates.
- Threads OAuth exchange/refresh and text posts; no image hosting or redirect hosting.
- Stateful loopback fakes for all three; no real sign-in, posting or spend.

## Capabilities

### New Capabilities

- `bluesky-integration`: app-password session lifecycle and publishing.
- `x-integration`: PKCE, renewable credentials, priced publishing.
- `threads-integration`: HTTPS-redirect consent and text publishing.

### Modified Capabilities

- `platform-rules`: three channel identities and platform-specific counting.
- `platform-credentials`: explicit refresh operations for renewable sessions.
- `dispatch-outbox`: acknowledged X estimate retained with outcomes.
- `studio-api`: Q1 routes and fixed error codes.
- `studio-cli`: protected-secret connection commands and cost confirmation.

Checked the owners above, `linkedin-integration`, the Mastodon delta and `docs/migration.md`. No old adapter is ported; these are new integration owners.

## Impact

Core adapters, configuration, protected HTTP connection routes and publish requests, CLI/review cost presentation, pinned ATrium dependency, records and docs.

## Open questions

- Which HTTPS non-localhost redirect host will the founder configure for Threads, and how will its callback return locally?
- Which public-media host, if any, should a later Threads image slice use?
- Will X accept the exact registered loopback redirect? Real validation requires separate authorization.
