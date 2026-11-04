## Request

> keep moving
>
> Syncloom: do not wait. Take the next unfinished capability in docs/migration.md in your charter's order (the LinkedIn go-live stays held for the captain's credentials; skip past it), build it Rust-first with OpenSpec and /code, prove it with the real end-to-end run the captain would see, and send Main the screenshots (light and dark, narrow and wide) before landing. Nothing posts to a real social account and nothing is pushed. Name the capability you picked in your first status line.
>
> The selected next capability is URL thought intake with provenance, the first partly migrated row of docs/migration.md. Text thoughts and the capture suggestion/preview/confirm-to-draft flow already exist on canonical local main03b6269738bfbe1e431a49faf2ef9b0520438276. That row and the adopted Rust plan explicitly assign URL retrieval to its own slice and security review. LinkedIn go-live is not part of this work.

## Approval

pending

New capability requested: none.

Duplicate kept: none.

## Why

Text capture cannot retrieve a source. The migration ledger assigns URL intake a separate security-reviewed slice.

## What Changes

- Explicit, bounded URL retrieval followed by existing selection, preview and confirmation.
- Exact-host operator allowlist, empty by default; public-address pinning at every redirect.
- Persist requested URL, final URL, retrieval time, response hash and extracted text with pending pieces, never content in events.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `thought-drafting`: URL retrieval and source provenance.
- `studio-api`: guarded URL/capture operations.

Checked `thought-drafting`, `provenance-log`, `studio-api`, and the adopted capture delta.

## Impact

Core retrieval and capture; existing store, browser and API. Read old thought pipeline, public URL guard/redirect transport and lineage at `3efcd35370242a37985fbacb272dacf80b2d42dd` read-only. No publishing, Research Radar or attachment work.

## Reuse inventory

Timing: before first edit; findings inventory ran before creating this proposal.
Search: capture, Source, create_thought, source_url, redirect, allowlist, resolve_to across crates/specs/research.
Candidates: Studio::confirm_capture/create_thought, capture::preview, Source, Store capture records, config::http_client.
Decision: reuse these owners; retrieval needs distinct DNS pinning and content limits, not the platform client's fixed destinations.

## Open questions

None for implementation. Exact-head security landing and image review remain required. Firstmate selected the narrower allowlist engineering default, not a host or landing approval.
