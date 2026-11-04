# Text studio design

The browser studio is small Rust/Axum server-rendered HTML, with native forms and no script runtime. [Stack decision](../decisions/frontend/0007-text-studio-frontend-stack.md), [palette/type decision](../decisions/frontend/0008-text-studio-palette-and-typography.md), [behavior change](../../openspec/changes/add-text-studio-frontend/design.md) and [operator guide](../studio.md) have distinct ownership.

## Current visual system

`crates/syncloom-server/src/studio.css` owns all tokens. System sans,16px prose,65ch measure,1.68 light/1.72 dark leading;24px page titles,18px section titles.12px card and8px control radii,1px borders, no decorative shadows. The authored desktop home has a420px composer beside the saved-piece ledger; review places its340px decision column beside the text. Below760px all columns stack. Layout was checked at200% CSS page zoom and desktop reflow equivalents; native browser-menu zoom is not independently verified.

| Role | Light | Dark |
|---|---|---|
| Canvas / surface | #f5f6f3 / #ffffff | #0d1312 / #141d1b |
| Ink / secondary / muted | #11201e / #415450 / #526661 | #ecf1ee / #b0c0bb / #9fb0ac |
| Border / control border | #d5ddd3 / #788c80 | #344842 / #708c80 |
| Accent / hover / ink | #1b5442 / #133e30 / #ffffff | #34d399 / #6ee7b7 / #091e16 |
| Accent tint / focus | #e6f0ec / #b45309 | #132a23 / #fbbf24 |

OS/browser `prefers-color-scheme` selects light/dark. Status words and authored CSS geometry distinguish Review from Publish intent and Last operation; color alone never carries a result. Native disabled buttons remain readable and do not use the accent or subtle fill. Visible3px focus, skip link, native checkboxes/details/forms and minimum44px controls support keyboard use. No optimistic sent state, fake loading bar or client-only validation.

## Screens and honest states

- Home: thought, explicit channel checkboxes (only LinkedIn selected by default), complete saved text, source disclosure, review and last-operation badges, existing after-only20-record pagination. No local-slice filters or invented group navigation.
- Review: exact saved prose/whitespace, channel count units, approximate local preview disclaimer, source language review-target handle exactly once, full revision disclosure, approval and separate publish action, saved current-publish receipt.
- Edit: saved text and fit context, save/cancel; no approval of unsaved content.
- Render: compatible existing template/slide editor and existing verified assets, not a new media workflow.
- Visual evidence: local actual-byte technical checks beside exact hash-bound previews, with semantic/aesthetic quality explicitly not assessed and no quality score. Native disclosures retain named checks, assessor, full revision and limitations. Damaged/stale/unavailable bytes show recovery instead of substituted previews. This extends the existing review, not an approval gate or new visual system.
- Shared errors/empty/results: fixed safe code, request identifier and recovery navigation; no raw provider error or submitted text echo.

Approval never proves account readiness. A persisted Posting marker is unconfirmed, not proven active work. Core `IntentStatus::allows_new_attempt` owns retry eligibility. Unknown/Posting/Posted suppress another send. Export is a separate operation and cannot erase the current revision's publish record. Contained describes only that intent, not all networking. Missing remote id/URL is displayed as not recorded inside closed Details; no invented receipt reason or charge.

`Studio::inspect_publish` returns an owned piece/current-publish snapshot under one store lock. It makes no dispatch, mutation or event. Browser publish redirects to the saved review snapshot, avoiding form re-submission and keeping receipt/action facts together. Templates never read private storage. Main1353's bounded correction keeps developer codes, request/attempt/receipt fields and exact counting rules in closed Details. A shared character label displays the existing count, including conservative X/Threads units, without changing a threshold. Already-approved versions have no second approve form. Their actual date comes from the last matching approved review provenance event under the same core lock, not a later updated_at; missing history is date not recorded and corrupt history fails closed. Reading approval time currently streams local history; add an index only after measured latency justifies it.

## Source and research boundaries

[Impeccable custody](impeccable/README.md) preserves exact guidance/license/source hashes. [Frozen design/challenge archive](text-studio-history.zip) retains the original criteria, conclusion-free input, sealed recommendation, amendments, both final drafts, specimen metrics and full independent Codex response. Its original relative links/line numbers are historical, not current documentation. Final challenge SHA256: `3b148c7badae7a95d66068e1d40d68cdfd0ff1370b5fb3c2baf782b51823efa6`.

Use existing [research custody](../decisions/research/0001-research-custody-and-retrieval.md) and [study index](../research/INDEX.md), not another speculative study. Q1 covers X/Threads/Bluesky API/auth/pricing, Q2 LinkedIn integration, Q8-r2 local-first credentials/OAuth and deferred scheduling. Conservative fit/count/cost UI facts come from current core, not unverified remote parity. Q7 concerns content assets, not application typography. Provider reports are attributed there, with missing native/HAND composition retained as reported gaps. Local layout, reading comfort and remote fold/font parity cannot be established by those reports. No new Phrona request was needed.

The Argon primary and different-family Codex challenger agreed on S1 and provisional P2; agreement is not independent UX validation. P1 warm editorial/serif remains a real alternative. The original specimen had recommendation labels, missing P2/P3 narrow visuals and inactive dark leading. Actual product screenshots/computed styles, not that specimen, own implementation acceptance. Manual per-page Impeccable critique/audit is single-context degraded review: no unauthorized extra workers or downloaded detector, and no claimed independent implementation audit.
