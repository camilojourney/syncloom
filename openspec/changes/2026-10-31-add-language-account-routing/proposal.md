## Request

> Build Syncloom v2 Slice 6: language and account routing (English and Spanish) and deterministic contracts, rebuilding and improving the old capability from migration.md and unlanded spec 038. Rebuild with Rust all of it. The routing must accurately route English thoughts to English destinations and Spanish thoughts to Spanish destinations, handling English sentences containing Spanish-looking tokens without false positive Spanish classification, and handling accent-less Spanish without failure.

## Approval

pending

New capability requested: none; extend thought-drafting.

Duplicate kept: enum derive/serde annotations in piece.rs and routing.rs, because Status and Language have independent owners and no shared domain rule.

## Why

Substring triggers misroute English thoughts and miss Spanish without accents. Source-derived review metadata must not depend on model wording.

## What Changes

- Attach deterministic EN/ES founder destinations, thesis, product mentions and visual hook to new pieces.
- Preserve explicit review and the existing connected-account delivery boundary; a target is not an authenticated account.
- Mark Slice 6 migrated in the ledger.

## Capabilities

### Modified Capabilities

- `thought-drafting`: source-derived routing and deterministic extraction. Checked `platform-rules`, `draft-review`, and the old `thought_pipeline.py` routing owner at `3efcd35`. Spec 038's object `16890bd` is unavailable locally; the dispatch brief supplies its selected scenarios.

## Impact

Rust core only; additive optional piece metadata visible through existing HTTP and CLI JSON. No dependencies, new network calls, credential changes or automatic posting.

## Open questions

None.
