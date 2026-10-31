## Request

> aslo build synchroloom rebuild it all as we mentioned and also the integrations wtih al lthe social media apis to make deep research onthem too ... use hte openspec last version with the instruciont conf think about what is next and start building all thos features.

## Approval

pending

New capability requested: meta-integration

Duplicate kept: none

## Why

Q3 now answers the Facebook Pages and linked Instagram professional-account API questions. Build the owned-account path without implying live delivery or choosing hosting.

## What Changes

- Facebook Login for Business, derived Page tokens in the Keychain, loopback callback.
- Page text/photo and Instagram single-image/carousel delivery behind Platform.
- Approved rendered JPEG slides and an explicit media-delivery interface, unconfigured in production.
- Stateful fake Graph proof; local scheduling only.

## Capabilities

### New Capabilities

- `meta-integration`: Meta consent and API delivery.

### Modified Capabilities

- `platform-rules`: Page and Instagram channels.
- `dispatch-outbox`: Meta grants.
- `visual-assets`: approved JPEG slides in addition to PNG/PDF (existing owner in render-visual-assets).

Checked platform-credentials, platform-rules, dispatch-outbox, studio-api, draft-review, linkedin-integration and the visual-assets and Mastodon deltas. Read the old Social API client at 3efcd35; retain text bounds, not its retrying proxy.

## Impact

Rust core, protected HTTP API, renderer and review hashes; no new hosting dependency or real Meta action.

## Open questions

Which real media host, retention window and deletion policy should a later approved hosting adapter implement? Will Meta accept the exact loopback redirect for the eventual app configuration?
