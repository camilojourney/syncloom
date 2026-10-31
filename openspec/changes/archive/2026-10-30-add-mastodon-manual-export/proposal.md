## Request

> aslo build synchroloom rebuild it all as we mentioned and also the integrations wtih al lthe social media apis to make deep research onthem too ... use hte openspec last version with the instruciont conf think about what is next and start building all thos features.

## Approval

pending

Duplicate kept: none.

New capability requested: mastodon-integration
New capability requested: manual-export

## Why

Q5 is delivered and imported verbatim. It recommends Mastodon as an API add-on and manual handoff for Reddit, with TikTok later. Q6 is imported research only.

## What Changes

- Per-instance Mastodon registration, OAuth and Keychain sign-in.
- Text and approved rendered-image posting with alt text, bounded media processing and truthful outcomes.
- Approved-revision export bundles with text, optional PNG/PDF and target notes, recorded as exported, never posted.
- Fake-only end-to-end proof, mutation checks and final rebased gates. No real sign-in or post.

## Capabilities

### New Capabilities

- `mastodon-integration`: instance registration, sign-in, media and statuses.
- `manual-export`: local handoff bundles and export receipts.

### Modified Capabilities

- `platform-rules`: Mastodon and generic manual channel identities.
- `platform-credentials`: instance-scoped, non-expiring Mastodon tokens.
- `dispatch-outbox`: Mastodon delivery and concurrent attempt exclusion.
- `studio-cli`: Mastodon and export commands.
- `studio-api`: Mastodon and export operations.

Checked all six existing owners above plus `linkedin-integration`, `draft-review`, `provenance-log`, the visual-assets change and `docs/migration.md`. Read the old Social API client, containment and outbox at `3efcd35`; no Mastodon/export owner exists there.

## Impact

Core adapters, shared OAuth boundary, store, CLI and loopback API; reqwest multipart support. No reading integrations, scheduler or live platform traffic.

## Open questions

None.
