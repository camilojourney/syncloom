# Design

References: security, data, ui.

## Context

See proposal.md. There is no existing v2 browser UI; the CLI and HTTP handlers already call Studio. Single-founder local access has no account model.

## Goals / Non-Goals

Thin Rust surfaces, with no browser build pipeline or new content store. No network access from a phone, social adapter, credential UI or live post.

## Decisions

- Use server-rendered Rust HTML and ordinary forms instead of a JavaScript SPA: all first-party code stays Rust and Studio remains the sole rule owner. Form adapters construct the existing typed core requests; JSON API handlers are unchanged except their access guard and listing operation.
- List persisted pieces in id order with twenty-item cursor pages instead of an unbounded response or a second index. Source thoughts already live in each piece, so there is no schema migration or duplicate thought store.
- Use one unpredictable in-memory server-session token, embedded in forms and available through a guarded no-store JSON handshake, instead of cookies or a persistent password. Constant-time matching belongs to Secret, shared with the existing OAuth state check rather than copied.
- The local operator may read and mutate all pieces; every path checks exact Host, exact Origin when present and browser fetch-site metadata. Mutations additionally check the session token before any core operation; fixed route labels, status and request ids are logged, never content or tokens.
- Share one core HTML escaping function between the renderer and page instead of keeping identical escaping code in each surface. Separate file-record identity checking, typed submitted-action parsing, per-action core execution and approval copy from their callers so each function has one job.
- Escape all interpolated content and send restrictive CSP, no-store, same-origin referrer policy, frame denial and nosniff on every response instead of trusting stored drafts. Keep core errors and statuses identical; browser errors add an inert recovery link while API errors retain JSON envelopes.
- Use same-origin referrer policy instead of no-referrer: the real Chromium form reproduction sent Origin null under no-referrer, while same-origin preserves local form Origin without disclosing referrers off-origin.
- Open draft and visual edits on separate pages with no approve/publish controls instead of mixing unsaved text and saved-revision approval. Verified PDFs allow same-origin framing only; studio pages remain unframeable.
- Embed verified current PNG and PDF endpoints at phone widths instead of re-rendering previews or approving thumbnails. Full hashes and revision are submitted from the viewed form, never silently refreshed at approval time.
- Preserve containment and truthful outcomes from Studio instead of adding browser-specific dispatch logic. Render/publish navigation is explicit and separate from approval; an edit always returns the current core state.

Kept: `contracts/studio-api/errors/` fixtures and `failure.rs`, because the fixtures independently pin the public error contract rather than deriving expectations from implementation.

Kept: `crates/syncloom-server/tests/logs.rs` changed count assertion, because the new session handshake adds one logged request: eight becomes nine, the full route table gains the session row, and token absence is additionally asserted. This is the weakened-check analyzer's removed-assertion candidate, not a removed check.

## Risks / Trade-offs

- PDF embedding varies by browser -> provide a verified PDF link, page count and full hash alongside the inline preview; prove the installed browser's rendering.
- Ordinary form navigation can take a renderer/provider deadline -> browser loading feedback remains native and copy warns before slow actions; no optimistic approval or publish state.
- Local malware can read the local token -> this protects the web boundary, not a compromised laptop. No remote access claim is made.

## Migration Plan

No data migration. HTTP clients first fetch `/v1/session` and attach `X-Syncloom-Token` for writes. Rollback restores the old binary without changing records.
