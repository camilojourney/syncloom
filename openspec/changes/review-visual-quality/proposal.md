## Request

> Implement one coherent local visual-quality review slice through the existing Rust visual-assets/draft-review/core and thin CLI/loopback/browser owners.

## Approval

pending

New capability requested: none
Duplicate kept: none

## Why

The founder needs to decide whether to revise a visual or continue human review. File geometry is measurable locally; meaning and aesthetics are not established by file validity.

## What Changes

- Inspect actual current rendered bytes, showing named technical checks and explicit unassessed visual quality, never an invented quality score.
- Bind assessment and browser previews to the displayed revision and asset hashes.
- Expose the same bounded local inspection through core, CLI and loopback API.
- Leave semantic model judging and its evaluation unimplemented and the migration row partial.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `visual-assets`: technical inspection and exact-version asset viewing.
- `studio-api`: inspection endpoint and browser evidence.
- `studio-cli`: inspection command.

Checked full visual-assets, draft-review, provenance-log, studio-api and studio-cli specs, migration and vision. Old owners read at 3efcd353: agents/marketing/quality_score.py, visual/visual_judge.py and thought_pipeline.py; the ledger's old quality-score path is incorrect.

## Impact

Existing Rust owners; no new dependency, provider call, approval gate, artifact store or delivery permission.

## Reuse inventory

Timing: before first code edit; findings inventory retained in task scratch.
Search: visual, assets, render, integrity, quality and REDO across core/server/CLI, specs and pinned old owners.
Candidates: Studio::checked_assets, view_asset, Store::load_asset, page::previews, visual constants, existing png/lopdf parsers.
Decision: reuse them; inspection policy belongs beside visual assets.

## Open questions

Which provider/model, sanitized assets, call count and spend may a later semantic-judge evaluation use? No such execution is authorized here.
