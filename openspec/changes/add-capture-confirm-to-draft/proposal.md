## Request

> Bounded product ship: a local founder-controlled capture suggestion -> preview -> explicit confirm-to-draft slice, on current local main b97c6b7716bff4ee5f1ebf0bb9a967e8eaffa112, independent of frozen security86e5cc4c. Do not modify, merge, relatch or rewrite that candidate or any sibling branch. If a true security dependency appears, report it and stop before adding it; do not fold a security repair into this task.

> Read the existing migration capture row, adopted vision/spec owners, current thought/review routes, and the old capture owner at pinned old commit3efcd35370242a37985fbacb272dacf80b2d42dd. Keep the existing behavior worth migrating, not the old unsafe confirm-and-publish coupling. Confirming a capture must only accept user-selected, user-owned text into a draft via the existing core owner; it never approves, grants, schedules or publishes. Preview is not an actual approval. No source scraping, external URL retrieval, clipboard surveillance, third-party-text model import, background agents, new platform adapters, model-provider switch, real model call or new external service. Use existing fake drafting/provider paths for all tests. Avoid duplicating the existing thought form with a new name; if source inspection shows there is no distinct useful capture behavior within these bounds, report the concrete options instead of fabricating a feature.

## Approval

pending

New capability requested: capture

Duplicate kept: capture plain-length route hints and platform-rules bounds, because a suggestion threshold is not a second text limit.

## Why

The review screen exists, but old capture still approves and publishes in one confirm. Founder-owned text has no suggestion and preview before a draft.

## What Changes

- Add text-only route suggestions and a revision-bound preview that stores nothing and approves nothing.
- Confirm selected suggestions only by calling thought-drafting; repeat the same revision, reject a stale one.
- Show that path without a second thought composer. Exclude attachments, URL fetch, connection probes and publish-on-confirm.

## Capabilities

### New Capabilities

- `capture`: suggestion, non-approval preview, and explicit confirm-to-draft.

### Modified Capabilities

- `studio-api`: document the capture operations and their existing error codes.

Checked thought-drafting, draft-review, studio-api, platform-rules, dispatch-outbox, provenance-log and docs/migration.md. Capture does not fit those owners: thought-drafting already creates drafts, and confirm must not become a second drafter.

## Impact

Core suggestion and confirmation records, thin loopback routes and pages, fake-provider tests, migration row and studio guide. No new adapter, provider, credential or publish path.

## Reuse inventory

Timing: before first edit; findings.py reported 0 existing findings on the planned paths.
Search: create_thought, suggest and expected_revision across crates and openspec.
Candidates: Studio::create_thought, Channel, fit, Store and page forms. Rejected a second draft writer.
Decision: capture owns suggestion and revision only; confirm calls create_thought.

## Open questions

None.
