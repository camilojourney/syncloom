# 0001: Local technical evidence, not a quality grade

Status: proposed
Owner: Syncloom maintainer through firstmate
Method: decide@1.3.2, source SHA256 0a49138d17d054bd3c75a4b9963f1c110fb69863f7d791d39293848c68bd05fb
Checklist: 1.0.0, SHA256 8c88c2c9c524d97552f67f30f6c4ba25589516b9dc18a51b09f7962688e5b056
Identity: visual-quality/0001-local-assessment
OpenSpec: [implementation tasks](../../../openspec/changes/review-visual-quality/tasks.md)
Evaluation: task-created rendered material and negative controls; no model judgement evaluated.

## 1. Brief

2026-11-04, before implementation. Help the founder decide whether to revise an asset or continue manual review. Q1/Q2: compare current manual-only review, transparent local technical inspection, and a vision-model judge. Q3: local-only, no provider calls or publishing. Q4: existing byte integrity is checked, but no semantic accuracy baseline exists. Q5: a calibrated authorized judge could change the choice. Q6: strongest objection is that technical checks do not measure visual quality; the report must say so.

Class: ordinary reversible advisory implementation, no approval gate or external disclosure. Rollback is a code revert, with no stored assessment migration. Independent challenge and prospective consequential capture are N/A for this bounded choice; no independent reviewer is claimed. Budget: one local implementation and safe validation; zero model calls. Human preference: truthfulness outranks having a number. Unexplored: audience taste and real model calibration.

## 2. Frozen acceptance

Before implementation: observations bind exact bytes, ordinary thesis repetition is not penalized, blank images do not establish meaning, failed/unknown evidence cannot produce a passing quality score, approval is unchanged. No numeric quality pass mark is invented. Baseline is manual previews plus existing renderer/integrity checks. Amendments: none.

## 3. Unknown-resolution plan

Local real-render tests settle geometry, decoding, hash binding and restart/repeat behavior. Negative controls settle whether technical success is mislabeled semantic quality. External research is N/A: current repository contracts and inspected old code settle this bounded design; no external factual premise is needed. A future vision judge needs exact provider, disclosure, case, budget and execution authority before evaluation.

## 4. Evidence and options

### Claims

| ID | Claim | Status | Evidence |
| --- | --- | --- | --- |
| C1 | Existing visual assets already bind revisions and hashes. | verified | [Visual assets](../../visual-assets.md#revision-and-byte-identity), read and compared to Studio on 2026-11-04; local implementation scope. |
| C2 | The old review emitted ordinary-visual REDO noise. | reported | [Pinned plan report](../../research/2026-10-29-rust-plan/report.md), section 2 visual-judge row; old source inspected separately in change design. |
| C3 | This implementation improves subjective visual decisions. | unverified | none: requires founder observations or an authorized calibrated judging comparison; technical fixtures cannot establish it. |

Manual-only preserves honesty but adds no legible technical evidence. Porting historical heuristics repeats unvalidated penalties and false-REDO risk. Local technical inspection adds checkable evidence without claiming taste or meaning. A vision judge could inspect semantics but lacks approved calls and calibration; a fake would prove only plumbing. These options are not equally capable: the chosen slice is explicitly partial.

## 5. Challenge

N/A independent challenger for this ordinary reversible read-only slice. Single-context objection retained: even correct geometry can hide blank, unreadable or irrelevant content. Answer: no overall passing verdict or quality score; require human judgement visibly. This is a limitation, not empirical rebuttal.

## 6. Bounded experiment

Protocol: two actual renderer palettes with source-faithful copy, literal-markup edge case and blank/malformed/wrong-geometry negative controls. Expected outcomes follow the frozen acceptance, not tuning. Safety: only owned synthetic content, zero calls, no thresholds optimized; stop on incorrect identity or false quality pass. Existing tests and manual browser inspection are the execution path, not a new evaluation framework. Failures and unrun checks stay in task receipts.

## 7. Decision

Choose local, recomputed technical evidence. Reject a single score because no authorized evidence gives its weights or passing threshold. Keep manual review as semantic authority; do not present a stub as a migrated judge.

### Reconsideration trigger

| Signal | Owner | Action |
| --- | --- | --- |
| Provider/model, disclosure, cases, calls and budget are explicitly authorized | Syncloom maintainer | Propose and calibrate semantic judgement separately before promoting migration status |
| Local inspection associates evidence with different displayed bytes | Implementation owner | Block landing and fix identity regression |

Verification: current Code 5.4.4 structural checks and exact command/results are retained in the task delivery report; no causal improvement claim follows.

## 8. Outcome review

Owner: firstmate at landing review, then maintainer after first founder use. Prediction: local observations match served bytes while subjective quality stays unknown. Review source: actual server/browser receipts and subsequent founder feedback, not test counts alone. Batch/longitudinal review N/A to this single project change. Observed founder outcomes remain unknown until use. [Change](../../../openspec/changes/review-visual-quality/) owns implementation progress.
