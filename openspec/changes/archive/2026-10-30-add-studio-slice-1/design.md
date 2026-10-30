# Design

References: security, data, integrations, llm.

Decision record: `docs/decisions/0001-syncloom-v2-rebuild-decisions.md` (D4 Claude Sonnet, D6 local-only, D7 records). Plan: `docs/research/2026-10-29-rust-plan/report.md`, section 3 (the thirteen criteria) and section 4 (the common core).

## Decisions

- One Cargo workspace with three crates: `syncloom-core` (rules, ports, use cases, file store, Claude adapter), `syncloom-server` (Axum), `syncloom-cli` (clap). Reject separate adapter crates for the store and the provider: two more crates that only forward calls for a single-operator app; the plan's "no I/O in core" is kept as a module boundary (rules in pure functions, file and network access in `store` and `provider::claude`), not as crate boundaries.
- Records are per-record JSON files under one data directory (`pieces/`, `outbox/`, `provenance/events.jsonl`) written through a temporary file, `fsync`, rename, directory `fsync`, all under one exclusive lock file held with std `File::lock` (cross-process, so the CLI and the server may share a directory). Reject SQLite: transactions and constraints are not needed for a handful of records per day, the files stay inspectable as the old queue was, and no C dependency is added. Reject the old fcntl-plus-glob scan: each record is addressed by name, nothing is scanned.
- A write failure after a partial change (piece written, event append failed) is answered as `internal_error` and the piece stays; the log is appended after the artifact, as the old pipeline did, but the failure is no longer swallowed. Reject appending the event first: an event for a piece that does not exist is worse than a piece without its event.
- Codes and fixed messages for domain errors live in one core table (`StudioError`), so the CLI and the API print the same text; the HTTP status per code lives in one server map; transport-only codes (`host_not_allowed`, `not_found`, `method_not_allowed`, `internal_error`) are the server's. Reject one table in the server for everything: the CLI would then import HTTP.
- The Host check accepts exactly one Host header equal to `127.0.0.1:<bound port>`; no CORS layer exists. Copied from the invoz.v2 contract. Reject allow-listing `localhost`: the browser page of slice 4 is served by this same process on the same host and port.
- Request bodies are strict: unknown fields, missing fields and wrong types are `invalid_request`. Reject lenient parsing: a misspelled `expected_revision` would silently skip the revision check.
- The provider port is `ModelProvider::draft(DraftRequest) -> Future<Result<Draft, ProviderError>>` with `ProviderError::{Unconfigured, Rejected}`; the studio applies one deadline (`Settings::provider_deadline`, 60 s by default, injected by tests) and maps empty text to `provider_error`. Reject deciding in the adapter which answers are empty: the rule is the studio's, as in invoz.v2.
- All drafts of a request are generated before anything is written, so a provider failure persists nothing even with several channels (invariant: all or nothing per request). Reject writing pieces as drafts arrive.
- The Claude adapter is raw `reqwest` over the Messages API (`POST {base}/v1/messages`, headers `x-api-key` and `anthropic-version: 2023-06-01`, body `model`, `max_tokens` 1200, `system`, one `user` message), with `connect_timeout` 5 s, no redirects, `rustls`. Reject an SDK crate: none is verified locally and the call is one request.
- The model id is pinned to `claude-sonnet-5-5` with the `SYNCLOOM_CLAUDE_MODEL` override, following Code llm rule 6. The id is UNVERIFIED until the live check (criterion 13) runs; the captain's word is needed to use a key.
- The system instruction is fixed text and the thought is the user message, marked as data; model output is trimmed and otherwise stored as given, never truncated (the `platform-rules` fit flags an overrun as `REVIEW`). Reject truncating to the limit: it hides the overrun from the reviewer.
- Base URL rule: `https://<host>` or `http://127.0.0.1:<port>`, no credentials, query or fragment, validated once at startup (invoz.v2's rule). The fake Claude endpoint in `tests/support/` and the `fake_claude` example use the loopback form.
- Token usage from the provider's `usage` object is stored on the `piece_created` event. Reject a dollar estimate: Code llm rule 8; the daily limit needs a price table (open question).
- Timestamps are RFC 3339 UTC with `Z` from a `Clock` port (`SystemClock` in binaries, a fixed clock in tests). Ids are UUID v4; the group id is the 32-hex form and the piece id keeps the old `thought-<12 hex>-<channel>` shape so operators recognize it.
- Characters are counted as Unicode scalar values. Reject bytes (wrong for accents) and grapheme clusters (needs a dependency; LinkedIn's own count is unverified either way, a slice-5 research item).
- The store is synchronous file I/O called inline from async handlers; records are small and the server serves one operator. Reject `spawn_blocking` for now: no measured need.
- The request log is one JSON line per request written by a drop guard in the outermost middleware (request id, fixed route label, status, code, `total_ms`, outcome); route labels are fixed strings so paths with ids never reach the log. Copied from invoz.v2.
- Kept: `contracts/studio-api/errors/*.json`, because the fixtures are the independent contract that a test compares with the server's responses; the core table and the server status map are the implementation side.
- Kept: `crates/syncloom-core/examples/fake_claude.rs` includes `tests/support/fake_claude.rs` by path, because the fake must be runnable as a process for the CLI and manual runs without shipping test code in the library.

## Old behavior read at `3efcd35` (kept or changed)

| Old owner | Behavior | Here |
|---|---|---|
| `thought_pipeline.py:959-960` `_clean_thought` | strip and collapse whitespace | Kept (normalization). `raw_input` keeps the original text, as `ThoughtSource.raw_input` did. |
| `thought_pipeline.py:464-466` | thought shorter than 8 characters refused | Kept as 8 non-space characters (the plan's wording); 422 `thought_too_short`. |
| `thought_pipeline.py:639` | `piece_id = thought-{group_id[:12]}-{channel}` | Kept. |
| `thought_pipeline.py:972-1160` | deterministic template drafts with hard-coded copy; agent labels on records | Changed: model-backed drafts behind the provider trait; no agent labels, no claimed work. |
| `thought_pipeline.py:1659-1667` | fit bounds 20 to 3000 for every platform | Kept for `linkedin_text` only, in `platform-rules`; other channels get their own rows later. |
| `thought_pipeline.py:50-59` `CHANNEL_TARGET` | `linkedin_text` maps to platform `linkedin` | Kept. Other channels are out of scope. |
| `capture.py:154-159` | unknown channel refused | Kept as 422 `unsupported_channel` (the old code answered 400). |
| `content.py:507-517` | transitions: draft to pending_review/approved/rejected; pending_review to approved/rejected; approved to rejected; rejected to pending_review; same status is a no-op | Kept, minus `draft`: nothing creates a draft in v2. |
| `content.py:89-100` `_content_revision` | SHA-256 of piece id, text, platform and rendered asset paths | Changed: SHA-256 of `channel`, `piece_id`, `text` (no assets in slice 1; the channel stands for the platform). |
| `content.py:523-526` | `review_decision_id = review-{piece_id}-{revision[:16]}` on approval | Kept. |
| `content.py` PATCH | no revision check on a status change | Changed: `expected_revision` is required and a stale one is 409 `revision_conflict`, so a status change never lands on text the reviewer did not see. |
| old PATCH | no text edit; `scheduled_at` accepted | Changed: text edits create a revision and void approval; `scheduled_at` is out of scope. |
| `content.py:107-118` `_require_approved_dispatch` | 409 `APPROVAL_REQUIRED` unless approved with a decision id; 409 `REVISION_CONFLICT` on a stale or missing revision | Kept, lower-case codes. |
| `content.py:540-553` dry run | returned the payload before any check | Changed: a stale `expected_revision` is refused on a dry run too; the approval check is still skipped. |
| `content.py:815-826` `_publish_payload` | `{content, platforms:[platform], style:"raw"}` plus media | Kept without media. |
| `outbox.py:60-104` | one intent per (operation, piece, revision, scheduled_at) reserved atomically under a lock; `request_id = dispatch-{hash[:32]}` | Kept; `scheduled_at` dropped (no schedule operation). |
| `containment.py:16-32`, `content.py:132-150` | contained unless `HOLUS_PERSONAL_DELIVERY_GRANT`; intent marked `contained`, piece `publish_status` `contained` | Kept as always contained; no grant exists until a platform integration slice adds one. |
| `content.py:717-723` | piece marked `published` when the API only accepted the job | Not carried (the client is not ported; truthful status is a slice-2 requirement). |
| `docs/lineage.md`, `thought_pipeline.py:484-489` | append-only JSONL with ids and hashes, never text; lineage failure never breaks the artifact | Kept: ids and hashes only. Changed: an append failure is reported as `internal_error` instead of being logged and ignored. Not carried: export API, validation CLI, event chaining, Holusight boundary. |
| `app.py:76-80` | permissive dev CORS, no Host check | Changed: loopback bind, exact Host check, no CORS headers. |
| old tests | wrote into `data/` | Changed: temporary directories only; `git status --porcelain --ignored` unchanged. |

## Security boundaries

- Who may act: any local process; the API is intentionally unauthenticated (one operator, loopback only). The Host check stops browser pages (DNS rebinding); it does not stop local programs.
- What is checked: bind address, Host header, request body shape, channel names, the thought length, revisions on every state change, approval before an intent, the provider base URL form.
- What is logged: request id, fixed route label, status, error code, timings, outcome. Never thought text, draft text, headers or the key. Events hold ids, hashes, statuses and token counts.
- Where secrets live: `ANTHROPIC_API_KEY` in the process environment only, read once at startup, sent only as the `x-api-key` header to the validated base URL, never returned or logged. Provider redirects are not followed.
- Files: the data directory comes from configuration, never from a request; record names are built from ids the studio generated, never from request text. Temporary files are created in the record's directory and renamed over it.

## Data

Stored: pieces (the founder's draft text and the thought as `raw_input`, which is the founder's own content on the founder's machine), intents, events. Time: RFC 3339 UTC `Z`. Events are `object_action` names with ids, no content (Code data rules 10-12). No personal data beyond the founder's own text; no deletion path yet (one operator, local files; a `delete` command is a later change).

## LLM

One call per draft: fixed system instruction, the thought as the user message, pinned model id, `max_tokens` 1200, one deadline. Output is treated as untrusted text: trimmed, stored, flagged by `platform-rules`, never executed or interpreted. Token usage is recorded per draft; cost is not computed. The live check (criterion 13) is not run in this change.

## Risks / Trade-offs

- The pinned model id may not exist; the live check settles it.
- Spend is recorded as tokens; nothing stops a 6th dollar in a day yet.
- Two processes sharing a data directory rely on the lock file; a reader outside the lock may see a record between two writes (each record is still whole).
- The provenance log is append-only but unchained; tampering is not detected (not a slice-1 threat: one operator, local disk).

## Migration

None. The old application is a separate repository and is untouched; the ledger rows for the capabilities above flip in `docs/migration.md` in this branch.
