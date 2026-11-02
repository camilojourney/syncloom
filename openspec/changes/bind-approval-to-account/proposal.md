## Request

> The two auth defects you found are real bugs; fix both now as separate tested slices (synthetic accounts only), without waiting for the architecture report: (2) Bluesky: a content approval must be bound to the exact account it was approved for; switching accounts invalidates it, and posting to a different account requires a fresh approval. Check every other platform for the same two patterns and fix any you find the same way. Reproduce each first with a failing test, then fix. Because both touch who may post where, send Main the before/after evidence before landing.

## Approval

pending

New capability requested: none.
Duplicate kept: provider identity parsers, because each provider independently owns its wire protocol.

## Why

All seven implemented remote channels accept content approval after switching authenticated targets. Synthetic TCP regressions reproduce the incorrect destination.

## What Changes

- Bind content approval to a canonical provider target, independently of tokens and handles.
- Refuse stale or unbound approval before external effects; allow fresh review through existing interfaces.
- Retain approval history and durable repeat protection.
- Preserve same-account rotation and local/manual containment.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `draft-review`: exact-account approval and fresh review.
- `dispatch-outbox`: target-bound dispatch and historical intents.
- `platform-credentials`: observed canonical target identity.
- `studio-cli`: explicit account approval input.

Checked these owners and all platform integration owners; no competing capability is needed.

## Impact

Core approval, credential adapters, existing outbox; review screen and API/CLI errors. No dependency, hosted auth or new connection page. Effective posting-grant validation has a parallel owner.

## Open questions

None. Main1333 supplied the required independent initial challenge; no landing approval has been granted.
