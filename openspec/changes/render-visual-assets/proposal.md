## Request

> aslo build synchroloom rebuild it all as we mentioned and also the integrations wtih al lthe social media apis to make deep research onthem too ... use hte openspec last version with the instruciont conf think about what is next and start building all thos features.

## Approval

pending

New capability requested: visual-assets

Duplicate kept: none.

## Why

Slice 3 follows the accepted studio sequence. Q7 now supports Chromium for local images and carousels.

## What Changes

- Render a PNG and a single-document PDF with a pinned local browser and font.
- Bind exact asset hashes to revisions and approval; invalidate on text or template edits.
- Expose rendering and byte viewing through the studio API and CLI, with provenance.
- No browser installation, uploads, new platforms, Typst or review screen.

## Capabilities

### New Capabilities

- `visual-assets`: local rendering, asset integrity and renderer failures. No existing capability owns rendering.

### Modified Capabilities

- `draft-review`: visual revision identity and approval.
- `studio-api`: render and view operations.
- `studio-cli`: render and view commands.
- `provenance-log`: render, view and asset approval evidence.

Checked these owners and `dispatch-outbox`, `platform-rules`, and the migration ledger's old `src/holus/visual/engine.py` at `3efcd35`. Keep HTML layout; replace per-page PDF merging and silent first-page fallback.

## Impact

Rust core, HTTP handlers, CLI, file records, CDP, PNG and PDF dependencies. Existing text-only revisions remain compatible.

## Open questions

None.
