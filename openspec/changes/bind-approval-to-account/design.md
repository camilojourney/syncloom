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

## Main1352 named kept findings

Main1352 independently inspected the native fragments and authorized only these two Code kept findings. They are normalized syntax matches, not duplicated policy. No wrapper, macro, source reordering, threshold, exclusion or checker change is warranted.

Kept: crates/syncloom-core/src/bluesky.rs:123-127 and crates/syncloom-core/src/threads.rs:220-224, fragment SHA256 88b5f199d65163ee7b5600c9958880c4ada83b8b2271ae2d025247c0be44bd9d, because independent provider owners retain their target-getter tail, object-safe publish signature, async boxing and first distinct call into the shared request-check owner; no account rule is copied.

Kept: crates/syncloom-core/src/mastodon.rs:302-306 and crates/syncloom-core/src/x.rs:206-210, fragment SHA256 857e94d7649c244654d4b8bcce2c57b2a5122e01a73143060d3d745ee0e495c2, because independent instance/account protocol owners retain their target-getter tail, object-safe publish signature, async boxing and first distinct owner call; merging that syntax would couple independently changing integrations without removing policy.

Account equality stays in PostRequest.require_target; refresh drift handling stays in CredentialSession.confirm_identity. Locations above refer to the final source, unchanged by this documentation disposition. Earlier native diagnostic offsets and exact raw fragments remain in task evidence. Code's normal kept mechanism matches paths, not hashes: final residue identities must be compared on both original/current bases to ensure no other new fragment is covered. Raw measurements and the earlier REJECT receipts remain visible. This narrow finding disposition is not security landing approval.

## Security boundaries

Only the existing trusted local operator may review/post. Approval checks content/assets and the explicitly reviewed target; dispatch checks the durable target against the one token used for effects. Tokens remain in TokenStore or transient memory, never approvals/events/receipts/logs. Only target hashes enter new durable fields.

## Risks / Trade-offs

Provider canonical-ID scope is tested only with loopback fakes, not real provider access. App-scoped subjects are conservatively separated by the app captured at authenticated resolution; two-app same-subject tests failed before that refinement and pass afterward. Mastodon verify_credentials now requests read:accounts after an independently failing consent-scope test; firstmate was notified of this seam. Effective posting grants still belong to the parallel owner. Legacy unbound uncertain outcomes intentionally block replay; no data migration fabricates their targets.

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
- Original browser startup timed out once. Current-main reconciliation later used an owned regular Google Chrome 156.0.8078.80 profile over the same loopback fakes: native A approval, stale A form refusal after reconnect, fresh B approval and B posting; reload retained the receipt and disabled Publish. Independent fake recorded only B in that browser journey. Dark screenshots were inspected at actual 1440px and 500px widths. Default launch remained dark and the requested 390px resize produced 500px, so light and 390px visual acceptance remain unavailable, not passes. Firstmate010 authorized continuing independent checks with those measured tooling limits. Own sessions stopped and temporary fixtures were removed.

## Accepted frontend reconciliation

Presentation follow-up inventory was captured before the first new regression edit; broad searches covered review/status badge rendering, pending/approved classes, snapshot callers and account_approval_required. Reused the existing core approval-problem result and status badge renderer rather than creating another approval rule.

Firstmate011/Main1371 identified the saved switched-B screen's green Approved badge and unqualified A date as misleading despite the correct refusal. The focused native-form and unbound-history regressions failed before the presentation repair. Review now uses the existing account_approval_required observation to show a pending-role Needs fresh approval badge, labels the retained A date Historical approval, and places the plain account explanation before closed technical details. Same-account and freshly approved B retain their actual date and no redundant approval action. No Piece/history/identity/policy changed. Corrected native regular-Chrome dark1440/500 views were captured and inspected. One different owned native Chrome-appearance probe was refused by the installed route's chrome: URL guard. Main1372/firstmate013 then authorized only the owned-page direct DevTools metrics/media calls: all eight switched-B/fresh-B light/dark1440/390 views were captured via chrome-devtools-axi and inspected, each with actual viewport/matchMedia/date/action metrics. The earlier unavailable matrix remains a retained failed route, not a waived or relabeled pass. No repeated emulate, tool changes, DOM/CSS fake or other profile access occurred.

The accepted frontend parent is b97c6b7716bff4ee5f1ebf0bb9a967e8eaffa112. Its ReviewSnapshot and approval-date provenance remain the owners. The snapshot now carries one current-target observation for display and expected_target; the form transports it to existing review policy. Inspecting a receipt calls existing publish_intent selection, retaining target-specific and legacy uncertain history. No account, grant, persistence or approval policy changed during reconciliation.

Both integration defects first failed through native HTTP forms: the missing displayed target made approval fail, and the unbound inspection key hid an existing target-bound receipt. The permanent regression covers both not-yet-posted A and posted A, stale-form refusal, zero new effects before fresh B approval, target-specific B receipt, repeat and export protection. A separately failing assertion corrected misleading disclosure copy to identify the account for the next explicit approval, not the historical A approval. Full selection exposed two newly accepted inspection tests still calling the old intent-key signature; only their arguments were adapted to actual approval_target, without removing assertions. It also exposed account recovery hiding Threads' independent image-unavailable notice. That existing notice now remains separately visible alongside account recovery; the original count/disabled/zero-effect assertions and new coexistence assertions pass. Accepted wording, counts, dates, previews and disclosures otherwise remain unchanged. Exact final checks and limits belong to firstmate task data current-main-integration-report.md; original ready reports and head3ea4c22 remain immutable.
