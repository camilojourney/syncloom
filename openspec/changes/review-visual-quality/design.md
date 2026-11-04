# Design

References: security, data, ui, llm. Code 5.4.4 was loaded before code edits; design.md and review.md govern single ownership, test-first proof and residue dispositions. LLM applies to evaluation limits, not authorization for calls. Integration guidance was also read; no integration changes are selected.

## Context

See proposal.md. Existing png and lopdf dependencies can inspect actual bytes; Studio already owns revision/hash validation, locked reads and content-free provenance. [Decision](../../../docs/decisions/visual-quality/0001-local-assessment.md) records alternatives and evaluation limits.

## Goals / Non-Goals

Keep technical evidence separate from subjective judgement. No quality threshold, automatic rejection, model seam pretending to be a judge, provider call, background job or second artifact store.

## Decisions

- Recompute a bounded read-only report under the existing store lock rather than persist scores: durable assets are the facts, and recomputation cannot retain an obsolete assessment after editing, rerendering or restart.
- Reuse the existing expected-asset validation for inspection and approval; rename its private helper to describe validation rather than introduce a second identity policy.
- Keep the inspection algorithm beside visual assets, with plain byte inputs; Studio alone performs IO, revision checking and provenance.
- Bind image/PDF/JPEG URLs to expected hashes, checked against the bytes returned by the existing view operation; versionless preview URLs can otherwise show replacement bytes beneath old evidence.
- Reuse Operate-mode tokens and native details disclosures. Show a concise technical result and a separate unassessed-quality statement; keep full hashes available with the exact previews.
- Carry the existing asset-guard outcome from preview inspection into action availability, rather than repeat verification or offer an approval the core will reject. Technical statuses do not add an approval gate; known current-byte failures reuse the existing one.

## Old behavior disposition

At 3efcd353 the ledger's visual/quality_score.py does not exist. agents/marketing/quality_score.py starts at 100 and subtracts uncalibrated text penalties; visual/visual_judge.py returns 0.82 and PASS for a readable file plus plan, even declaring human review unnecessary. Neither measures semantic pixels.

thought_pipeline.py:1326-1328 flags source-opening overlap while spec 038 at 16890bd:173-178 explicitly accepts the thesis as the hook. Keep source fidelity and existing rendering; reject this false-REDO heuristic, automatic redo and numeric authority. Historical tests prove encoded thresholds/file contracts, not user quality. No old executable or campaign is run.

## Boundaries

Only the existing local operator may inspect. Host/Origin/session guards remain unchanged. The POST endpoint takes only an existing piece id, revision and hashes, never paths or URLs; strict parsing rejects extra fields. Asset query hashes can only restrict returned current bytes. Logs retain fixed route labels and error codes; provenance retains existing asset-view hashes, never content. No credentials are read by assessment.

## Risks / Trade-offs

Recomputation adds local parser work per review; asset counts remain bounded by existing render limits. JPEG bytes receive existing hash verification only; JPEG decoding, text readability, visual hierarchy, meaning, factual accuracy and audience fit remain unmeasured. Technical success is not overall quality.

## Spec-Impact

| Contract / scenario | Implementation owner | Executed assertion owner |
| --- | --- | --- |
| visual-assets: ordinary thesis, failed/unknown negative controls | visual_quality.rs; Studio::assess_visual | server/tests/visual_quality.rs real_render_assessment; core/tests/visual_quality.rs blank_malformed controls |
| visual-assets: repeat/restart/concurrent identity | Studio lock, existing expected-asset validation | core/tests/visual_quality.rs real_assets_recompute and concurrent_edit |
| visual-assets: version-bound image/PDF/JPEG requests | Studio::view_asset_version; server asset_response | server/tests/visual_quality.rs old_preview_hash; existing Meta suite plus task HTTP receipt |
| studio-api: protected inspection and visible states | server.rs; page/previews.rs | server/tests/visual_quality.rs; actual stable Chrome screenshots and keyboard/layout receipts |
| studio-cli: same evidence/errors | CLI dispatch into Studio | task's actual compiled CLI/API comparison and fixed-error receipts |

Current Code spec_maps reports accepted-spec navigation only; new delta scenarios are mapped here manually. Map presence is not test execution or semantic completeness.

## Findings dispositions

Kept: crates/syncloom-cli/src/main.rs dispatch, because the native command router has one exhaustive arm per independent public operation; the new arm raises cyclomatic complexity from 10 to 11 without adding policy. Splitting it solely for a detector would add a forwarding layer or duplicate command switching. Owner: CLI surface maintainer. No suppression or threshold change.

Four function-scoped unwrap expectations in the new core diagnostic helpers retain the project's fail-fast fixture convention; all are fulfilled under all-target Clippy. Async test annotations were removed when Clippy correctly reported them unfulfilled; no test assertions were weakened. No measured clone/dependency exclusions change.

## Migration Plan

No stored schema change. Revert the commit to remove the slice; existing pieces/assets remain readable. Migration stays partial until an authorized, calibrated semantic judge is genuinely evaluated.
