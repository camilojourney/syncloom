## Request

> By the way since sync is already working, I want you to create the frontend. use impecable skill to create it, you wouold think about the design with argos, you can use the tool /decision to check colors we need to optimize for what we want and a great experience for me so I can see what I post. let' focus on the text first.

> remember to use impecable and thin on all the pagest we need to create, test all the api points. make it better

## Approval

pending

## Why

The studio hides supported text channels, mixes review and delivery state, and lacks a comfortable dark reading surface.

## What Changes

- Build the challenged S1/P2 direction on existing home, review and editor pages; preserve render and error surfaces.
- Accept strict repeated-channel form fields, with after-only pagination.
- Read the current revision's publish intent through core so export cannot hide a non-retryable publish.
- Preserve pinned Impeccable documentation and evidence limits; verify actual product views.

## Capabilities

### New Capabilities

New capability requested: none; extend existing capabilities only.

### Modified Capabilities

- `studio-api`: text intake, honest previews, accessible themes and safe action display.
- `dispatch-outbox`: bounded read-only current-revision publish inspection.

Checked studio-api, dispatch-outbox, thought-drafting, draft-review, platform-rules and visual-assets; migration.md assigns no overlapping legacy authority.

Duplicate kept: UI explanations and core retry policy have separate owners; immutable third-party guidance remains source evidence.

## Impact

Rust server renderers, core read contract, stylesheet, tests and documentation. No auth policy, new media workflow, external dependencies or remote actions.

## Reuse inventory

Timing: before first edit; inventory and broad searches retained in task proof.
Search: escape, intent, receipt, channel, cursor across crates and specs.
Candidates: Studio, Store, html::escape, Channel, IntentStatus, page.rs.
Decision: reuse these owners; add only core-owned publish inspection, not private-storage reads in templates.

## Open questions

None.
