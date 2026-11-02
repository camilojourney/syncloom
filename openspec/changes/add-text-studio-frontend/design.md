# Design

References: ui, security, data; strategic design and review references also loaded from Code 5.4.2.

## Decisions

Stack: [docs/decisions/frontend/0007-text-studio-frontend-stack.md](../../../docs/decisions/frontend/0007-text-studio-frontend-stack.md). Palette/type: [docs/decisions/frontend/0008-text-studio-palette-and-typography.md](../../../docs/decisions/frontend/0008-text-studio-palette-and-typography.md). [Current built design and exact archived evidence](../../../docs/design/text-studio.md).

Use small cohesive Rust SSR modules, not a new template framework, under the unchanged script-free boundary. Manual escaping remains a review/test obligation; repeated omissions reopen S2 rather than justify a custom safe-string framework.

Use provisional P2 tokens with system sans, restrained surfaces and authored state geometry. P1 remains competitive; the static specimen proved neither remote parity nor full accessibility.

Expose a core-owned current-revision publish snapshot and actual approval time under one store lock. Render one persisted receipt and redirect browser publish to that snapshot, rather than let the browser infer a publish intent from the last-operation summary or read storage itself.

Keep existing after-only discovery. Parse repeated channels sequentially, reject unknown/duplicate scalar fields, delegate channel validation to Studio and omit speculative group discovery.

## Boundaries

Only the existing exact loopback Host/Origin/site/session holder may act. No new routes or authentication classification, secret stores, live account actions, media workflows, proxy or outbound fetches. Request logs retain fixed labels and omit content/tokens. Tests use isolated synthetic state; no schema migration.

## Ownership

Expose the existing Bluesky extra byte cap as a core constant used by both fit validation and the preview; no duplicated numeric policy or changed channel limit. The counts renderer presents the existing Fit, never a second fit evaluator. All writing surfaces use its plain character label and closed Details explaining the core-owned counting rule and bounds.

Kept: server page copy versus core intent policy, because presentation words change independently of retry eligibility. IntentStatus::allows_new_attempt remains the policy owner.
Kept: docs/design/impeccable/source.zip, because the immutable 18-file upstream subset must retain its original links/whitespace/hashes. An archive preserves source bytes without presenting an incomplete subset as an installed skill. No link or whitespace checker suppression is added.

## Risks and proof

A last export can hide a previous posted/unknown publish in Piece.publish_status; a read-only core snapshot makes the actual publish intent visible without dispatching it. Persisted posting is unconfirmed, never an active-progress promise. Approval is not delivery/account readiness.

Use adversarial output and strict-form red/green tests, repeat/export histories and real browser journeys. Inspect all five page states in light/dark at 1440x900 and 390x844, plus keyboard/zoom/contrast/error cases. Retain failed commands and findings; no static specimen reruns.

Main1353 correction: keep technical status/request/attempt/receipt data and counting rules in closed Details. Show natural outcome/count copy and suppress repeat approval for an already-approved exact revision. Approval dates come from the last matching piece_reviewed/approved provenance event, not updated_at or guessed time. The store streams that local history under the same inspection lock; missing history displays date not recorded, corrupt history fails closed. This adds no persisted field, JSON route or delivery policy. A large history may justify a core-owned index later if measured read latency exceeds the review budget.

Main1361 final polish: distinct counting/version/publishing/latest-action disclosure names. Hide the duplicate latest-action copy only for the exact same technical result already shown in the banner; keep separate export history and any summary without an equivalent banner. No lifecycle or account behavior changed.

Spec-Impact: extend existing studio-api and dispatch-outbox only. Preserve JSON schemas, approval/cost binding and intent retry behavior.
