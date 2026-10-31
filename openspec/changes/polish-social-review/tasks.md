# Tasks

## 1. Context

- [x] 1.1 Load the code skill: read SKILL.md and security/ui references; follow their rules throughout.
- [x] 1.2 Reproduce review defects through the browser and retain before PNGs at 1440/390.

## 2. Review corrections

- [x] 2.1 Add failing real-HTTP tests for display names and disabled X Publish; observe expected failures, implement shared names and native disabled state, rerun green.
- [x] 2.2 Test consent record, amount/date/revision refusal, new-revision invalidation and forged Publish; implement session-scoped consent and document the script-free workflow.
- [x] 2.3 Keep the date unbroken and capture after PNGs at both widths; inspect keyboard and accessibility disabled state.

## 3. Integration

- [x] 3.1 Rebase on current main, run fmt/clippy/tests and strict OpenSpec validation, commit and save final Code Verify on the committed tree outside the worktree.

## Evidence

- Real HTTP red run: three new tests failed on internal X label, enabled Publish and missing consent operation. Green runs cover all platform names and consent boundaries.
- A contained-publish test exposed that core cost refusal intentionally applies only to granted delivery. Added the approved browser consent gate without changing core policy; the same HTTP test now passes.
- Full workspace tests initially caught obsolete lowercase Mastodon/manual-export UI expectations. Updated the expected display names, retaining every assertion.
- Mutations removing full-quote matching, acceptance revision validation and the browser Publish gate each fail their selected regression tests; all guards restored.
- Initial Code Verify rejected page-renderer complexity growth. Cohesive publish-controls extraction removes the finding without changing markup; focused regression tests remain green.
- Weakened-check advisories concern two old channel-id assertions replaced by equally strict proper-name assertions. The new eight-channel HTTP test independently checks every label and unchanged machine ids; no assertion was dropped.
- Browser PNGs at 1440/390 show the date on one line and matching disabled X/Threads styles; keyboard Enter records consent without publishing, and accessibility snapshots expose the disabled button state.

## Workflow follow-up

- Merge authority archives the change after review.
