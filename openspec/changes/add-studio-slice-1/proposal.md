## Request

Quoted verbatim from the captain, 2026-10-30, as relayed in the task brief (spelling is the captain's). Omitted: firstmate's context paragraph, which is supervisor context, not the captain's words.

> yes to all. start building it.  make suer we are applying all thethings we learned. aslo build synchroloom rebuild it all as we mentioned and also the integrations wtih al lthe social media apis to make deep research onthem too. we need to build this really well so use hte openspec last version with the instruciont conf think about what is next and start building all thos features.

Earlier the same day:

> Let's work on this decision give you you recommendation and I'll accept it

> remember touse the configuaiton we have on invoz for the openspecs

## Approval

pending

New capability requested: studio-api

New capability requested: thought-drafting

New capability requested: platform-rules

New capability requested: draft-review

New capability requested: dispatch-outbox

New capability requested: provenance-log

New capability requested: studio-cli

Duplicate kept: error status, code and message in `contracts/studio-api/errors/` and in the core error table plus the server status map, because the fixtures are the independent contract a test compares.

## Why

Slice 1 of the Rust rebuild (plan section 3): one text thought becomes one model-drafted LinkedIn text piece, approved at an exact revision, then a contained publish intent. It needs no frontend or platform decision.

## What Changes

- Cargo workspace: `syncloom-core` (rules, ports, file store, Claude adapter), `syncloom-server` (loopback Axum API), `syncloom-cli`.
- Plan criteria 1-12 as requirements with scenarios; criterion 13 (a live Claude draft) is built behind the trait but not run.
- Old owners read at `3efcd35`: `thought_pipeline.py`, `content.py`, `capture.py`, `outbox.py`, `containment.py`, `docs/lineage.md`; ledger rows in `docs/migration.md`.

## Capabilities

### New Capabilities

`openspec list --specs` returned nothing. Unmigrated owners: `docs/migration.md`.

- `studio-api`, `thought-drafting`, `platform-rules`, `draft-review`, `dispatch-outbox`, `provenance-log`, `studio-cli`.

### Modified Capabilities

None.

## Impact

New code only. Dependencies: axum, tokio, reqwest (rustls), serde, serde_json, sha2, uuid, time, clap, tracing, tower-http.

## Open questions

- Is `claude-sonnet-5-5` the Sonnet the captain means? Unverified until the live check.
- How is the USD 5 a day limit enforced? Tokens are recorded; dollars need a price table.
- English only here; EN/ES routing waits for slice 5.
- Approve and edit in one call? Refused here.
