# frontend/0007: Small Rust server-rendered text studio

**Date:** 2026-11-02
**Status:** Accepted within delegated implementation scope; not captain taste acceptance
**Supersedes:** none
**OpenSpec changes:** [add-text-studio-frontend](../../../openspec/changes/add-text-studio-frontend/design.md)
**Agreed:** Firstmate promotion selected S1 and provisional P2 after final challenge; no new captain preference inferred
**Recommended:** the bounded selection below
**Built:** unlanded implementation candidate; progress and proof belong to [tasks](../../../openspec/changes/add-text-studio-frontend/tasks.md)
**Live-proven:** no real service/account proof; local synthetic proof only
Method: decide@1.3.1, source baa7c65c88fa04eaa2d2167e78b24ff56e575e68

Checklist: [1.0.0 pinned source](../sources/decide-question-checklist-1.0.0.md), SHA256 `8c88c2c9c524d97552f67f30f6c4ba25589516b9dc18a51b09f7962688e5b056`.
These allocated frontend/0007 and frontend/0008 identities predate project import; preserve them, not a new renumbering.

## 1. Brief and capture

Choose the frontend mechanism under the accepted Rust-first preference, for reading, editing and reviewing text without widening auth/media scope. Consequential: a revert cannot undo an escaping leak or mistaken publish. Rollback is a code revert, no data migration; residual harm is a misrepresented send or disclosed text. Owner: syncloom-v2-lab. Budget: one Argon primary plus one authorized different-family Codex challenger, completed; no further design round.

Before comparisons, Q1-Q5 and Q6 were recorded in the [unchanged history archive](../../design/text-studio-history.zip), entries `decisions/frozen-pre-experiment.md` and `decisions/challenge-input.md`. Capture order1 of2, base c0584e87a808f457bd7b205da3009c485243d4e0, freeze2026-11-02T14:04:00Z, frozen SHA256 `fec3e3b027d05a7c56daac9a1f128033adae66d54dae01137f7e78f97c638481`. Outcome declared2026-11-16 before implementation.

## 2. Frozen acceptance

C1.1 preserve script-src none, exact Host/Origin/site/session checks. C1.2 reproducible Rust build; original freeze incorrectly equated that with zero new crates. C1.3 complete intended browser text workflow, not new credentials/media pages. C1.4 one domain/escaping owner, auditable cohesive modules. The preserved scope amendment explicitly repairs C1.2/C1.3 before challenge; it does not erase the wrong original framing.

## 3. Unknown-resolution plan

Inspect current route/core/build constraints, then challenge S1 versus a compile-time Rust template engine. No answer-changing outside unknown remains. Research N/A: new Phrona calls would not establish repository fit. Phone access beyond loopback remains outside this choice.

## 4. Evidence and options

### Claims

| ID | Claim | Status | Evidence |
|---|---|---|---|
| C1 | The existing HTTP boundary forbids script execution and checks same-origin native forms. | verified | [boundary](../../../crates/syncloom-server/src/boundary.rs), inspected2026-11-02; no boundary edit |
| C2 | Browser form adapters call Studio directly; they are not JSON API consumers. | verified | [actions](../../../crates/syncloom-server/src/page/actions.rs), [home](../../../crates/syncloom-server/src/page/home.rs), inspected2026-11-02 |
| C3 | html::escape is the existing shared HTML escape owner. | verified | [escape](../../../crates/syncloom-core/src/html.rs), inspected2026-11-02 |
| C4 | Current dependency policy permits reviewed locked additions. | verified | [development](../../development.md), inspected2026-11-02; no eternal no-new-crates rule |
| C5 | A maud/askama migration under these lints was not built in this comparison. | unverified | none: no authorized dependency experiment was needed for the current bounded choice |

Structural assessment: S1 keeps the existing transport/domain/escape owners and extracts cohesive page modules. S2 (maud/askama) is viable, stronger default escaping, not excluded by dependency cost; it would replace manual escaping consistently if S1 proves fragile. S3 hydrated Leptos/Dioxus and S4 JS SPA conflict with C1.1 in their client-executed forms; their popularity is not evidence. Existing monolithic SSR is the no-change baseline, but does not meet the requested text-first presentation.

## 5. Challenge

Different-family challenge: primary antigravity/gemini-4-argon; challenger openai-codex/gpt-6.1-sol xhigh, trace583631ab16b4845f28001b7e445f07fe. Conclusion-free input SHA256 `7817abd8e303f4116b90317444b8f4d5a70a115d6267b7c54482e72de2c822df`; initial pick saved before recommendation reveal. Both are in the history archive, not reconstructed here. Same permitted local evidence, with acknowledged specimen framing leaks; shared training/context limits independence.

Initial pick S1; strongest alternative S2. O1 accepted: a voluntary Display wrapper is not enforced escaping. O4 accepted: no page-slice filters, sequential repeated-channel parsing with strict scalars. Other corrections remove invented auth pages and API proxy claims. Final report SHA256 `3b148c7badae7a95d66068e1d40d68cdfd0ff1370b5fb3c2baf782b51823efa6`, unchanged archive entry `challenge/report.md`. No independent implementation challenge is claimed.

## 6. Experiment

N/A for stack compilation comparison: repository constraints and bounded challenge settled the selection. Product tests still must prove it; a current lockfile/cache is not an option-disqualifying experiment.

## 7. Decision and boundaries

Select S1 small Rust/Axum SSR, `page/` modules and one CSS owner. Keep core rules in Studio and escape at output boundaries. Reject no-change presentation; defer viable S2, not on cost grounds. Exclude client-runtime S3/S4 while script-src none is binding. No custom safe-string/template framework.

Premise: manual dynamic output remains auditable and inert. Falsifier: an escaping defect or unresolved maintainability failure at2026-11-16. Human-owned questions: none needed for this delegated slice. Future relaxed CSP, remote access, credential/media workflows require separate authority, not an open task decision.

### Reconsideration trigger

| Signal | Owner | Action |
|---|---|---|
| An escaping defect or blocking maintainability problem in page renderers | syncloom-v2-lab | Reopen this decision; make S2 the strongest competing replacement |

Verification: Code5.4.2 owns actual tests/lint/records/links/residue checks; observed receipts and dispositions stay in implementation proof, not inferred from this record's presence.

## 8. Outcome review

Owner/date: syncloom-v2-lab,2026-11-16. Preserve original predictions: by that date all supported text journeys complete in Chrome with no CSP errors, and tests/fmt/clippy/Code Verify have no blocking escaping/complexity defect. Source: final implementation receipt and product captures. Original uncalibrated probabilities0.90/0.88 are retained in archive; do not score before date. Outcome-blind process review pending; observed outcomes pending. This is capture1 of the two frontend records for Fleet's later ten-record review, not evidence that the method wins.

## Evidence and gaps

[Research custody](../research/0001-research-custody-and-retrieval.md) and [original Rust plan](../../research/2026-10-29-rust-plan/report.md) provide prior context, not proof of frontend performance. [Built design](../../design/text-studio.md) owns current architecture. No remote-device or live-platform parity evidence was obtained.
