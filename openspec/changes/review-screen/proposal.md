# Request

> aslo build synchroloom rebuild it all as we mentioned and also the integrations wtih al lthe social media apis to make deep research onthem too ... use hte openspec last version with the instruciont conf think about what is next and start building all thos features.

## Approval

pending

## Why

Slices 1-3 expose the studio through commands and JSON. The next accepted slice makes the same review loop visible in the laptop browser.

## What Changes

- Serve a Rust-rendered studio with thought intake, saved pieces, draft editing, rendering, phone-sized image/PDF previews, exact approval and explicit publish outcomes.
- Add paginated piece discovery without another content store.
- **BREAKING**: HTTP mutations require a server-session token. Reject foreign Origin and cross-site browser requests; retain exact Host checks and no CORS.
- Keep loopback binding and existing delivery containment. No phone access path, platform adapter or live posting.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `studio-api`: browser surface, session protection and security headers.
- `draft-review`: paginated saved-piece discovery.

Checked `studio-api`, `draft-review`, `thought-drafting`, `dispatch-outbox`, the active `visual-assets` change, and the migration ledger. No unmigrated old capability overlaps the new browser surface; the retired Observatory is not revived.

## Impact

Rust core listing, Axum page/form adapters, API clients' session handshake, tests and README. No stored-schema migration or external dependency service.

## Open questions

None.
