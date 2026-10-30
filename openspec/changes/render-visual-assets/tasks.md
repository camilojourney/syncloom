# Tasks

## 1. Test-first implementation
- [x] 1.1 Load the code skill: read SKILL.md and security, data and integrations references; follow their rules throughout.
- [x] 1.2 Add API/CLI visual workflow and renderer failure tests, run them red and save the selected failures before implementation.
- [x] 1.3 Implement pinned Chromium rendering, raster gradients, font loading, normalized PDF and content-addressed storage.
- [x] 1.4 Implement revision binding, approval/integrity guards, API/CLI operations and provenance; refactor shared behavior without weakening tests.

## 2. Evidence and landing
- [x] 2.1 Prove real rendering, snapshot tolerance, byte repeats, page size/count, failure outcomes, concurrent edits and mutation checks.
- [x] 2.2 Update development/API docs, migration ledger and Q7 decision/card links; run specs, records, links and secret checks.
- [ ] 2.3 Commit, rebase onto current local main, rerun all gates and Code Verify on the final tree; save receipts and leave a clean branch.

## Test-first evidence

The selected HTTP render scenario first returned 404, and the CLI rejected render with exit 2. Renderer tests then exposed wrong PDF paper rounding, visible screenshot scrollbars and unpinned glyph fallback; each was corrected without loosening an expectation. The exact-byte approval scenario first rejected its new hash input rather than detecting a stale pair.

The real pinned-browser API/CLI scenarios, repeat hashes, two reviewed palette snapshots, two/ten-page PDF geometry, selectable text, failure outcomes and same-revision byte replacement now pass. Refactoring centralized revision identity, invalidation and persistence/event ordering and split CDP lifecycle steps, preserving those tests. Final mutation evidence rejects removed asset-revision, approval and concurrent-revision guards and an incorrect image/PDF hash-pair comparison.
