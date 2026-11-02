# Design

References: security, data, integrations, ui.

## Context

See proposal.md. The existing Studio approval and outbox own state; adapters own canonical provider identity and credential snapshots.

## Goals / Non-Goals

Keep account approval independent of credential bytes and avoid a second state machine. Hosted identity, scheduling, real services and grant-policy redesign are excluded.

## Decisions

- Candidate A follows the task's Decide record and Main1333 Claude/Opus 5.5 independent initial challenge. The frozen packet, separate sealed recommendation, full initial review and differences response are retained under firstmate task data syncloom-account-approval; review was requirement-aware but recommendation-unexposed.
- Canonical IDs remain with existing credential owners; Meta additionally retains its provider-confirmed Page ID, and LinkedIn/Threads retain app authority captured with the authenticated subject. Reject slot keys and handles because reconnect reuses them.
- The current target is echoed as expected_target during explicit approval. Reject stamping whichever account is connected at submit because a stale page can silently approve another account.
- Approval and existing durable intents retain non-secret target fingerprints; target joins the intent key. This accepts Main1333's challenge against the initial revision-only key: unknown A remains reconciliation-only while explicitly approved B has a separate intent.
- Each adapter loads one credential snapshot, validates its target before any effect, then keeps that token throughout media and post calls. Reject reloading after a check because reconnect can redirect a send.
- Preserve ordinary failed-refresh credentials, but clear a connection after observed refresh identity mismatch. Reject preserving mismatched credentials because identity authority is no longer established.
- Legacy approvals stay unbound and are never backfilled. Legacy posted/unknown/posting intents remain on their original keys and cannot be bypassed through a new target key.

## Reuse inventory

Timing: before first edit; task inventory.txt was captured before the first regression edit.
Search: all crates plus approval/hash/identity/outbox terms and git history, beyond lexical inventory.
Candidates: Studio review_status/claim_attempt, Piece, DispatchIntent, CredentialSession, Platform, and every implemented adapter.
Decision: deepen those owners, not a new account lifecycle or grant state machine.
Kept: adapter canonical-ID extraction, because each provider's independent wire protocol and authority differ; fingerprint encoding and equality have one Platform owner.

## Security boundaries

Only the existing trusted local operator may review/post. Approval checks content/assets and the explicitly reviewed target; dispatch checks the durable target against the one token used for effects. Tokens remain in TokenStore or transient memory, never approvals/events/receipts/logs. Only target hashes enter new durable fields.

## Risks / Trade-offs

Provider canonical-ID scope is tested only with loopback fakes, not real provider access. App-scoped subjects are conservatively separated by the app captured at authenticated resolution; two-app same-subject tests failed before that refinement and pass afterward. Mastodon verify_credentials needs an identity-read scope coordinated with the posting-grants owner. Legacy unbound uncertain outcomes intentionally block replay; no data migration fabricates their targets.

## Migration Plan

Nullable fields deserialize old records without rewriting them. Reauthenticate unresolved credentials, then review the current account explicitly. Keep target-bound and legacy intent history. Rollback can stop future posting but cannot undo disclosure.


## Findings

Accepted owners consulted: draft-review (statuses/revision/assets), dispatch-outbox (claim/repeat/unknown/durability), platform-credentials (expiry/secrets/refresh), each implemented integration, studio-api and studio-cli. Inspected callers include HTTP show/review/publish, review forms, CLI approve, adapter media/post pipelines, Store and provenance.

The broad source/spec/history investigation and RED tests preceded product edits. The companion-map CLI ran after edits: a procedural miss, not a preflight pass. It found zero adopted maps and 177 unknowns, not 177 unrelated cases. Source/caller searches and native Rust executions supply bounded evidence; map completeness remains unknown. Candidate code began before the full stock design/tasks/apply instruction pass, under firstmate003 reversible-work authorization; no backdated compliance claim.

New expected_target was discovered by a separate RED test: the original account comparison alone still allowed a stale form to approve the account connected at submit. Review now checks the explicit observed fingerprint and renders display/form from one observation.

Validation exposed an inherited renderer-test race: real 200ms expiry could reap a child before its PID marker was scheduled. A controlled 300ms startup delay reproduced the missing-marker failure. Test time is now frozen until PID readiness, then advanced by the same 200ms; all timeout/reaping assertions remain. Only the test-util feature of existing Tokio is added, not a runtime dependency or deadline relaxation. This is a separate tested quality repair, not an account invariant workaround.

## Testing and validation

- Seven account-only switch regressions: independent fake actual B target before; 409 and zero effects after; fresh B and no-switch paths still post.
- Missing/stale reviewer target: 409 and unchanged piece.
- Same-account rotation: pre-refresh bound request remains accepted on all three renewable adapters; mismatch signs out and missing identity rejects.
- Durable history: posted/unknown A plus fresh B has distinct keys; old unbound terminal/uncertain intent cannot be bypassed; provenance prefix retained.
- Held-media reconnect and concurrent B review: A credentials, A repo, immutable A receipt/event.
- Guard sensitivity: removing the adapter guard fails direct Mastodon reconnect; removing both preflight and adapter guards makes all seven original regressions fail with independently observed B effects. Guards restored.
- All exact commands and failed attempts are retained in firstmate task data syncloom-account-approval. No real Keychain execution tests or real service requests are authorized.
- Browser startup timed out once; own bridge stopped, temporary capture test removed. API/markup evidence exists; visual acceptance is not evaluated, not a pass.
