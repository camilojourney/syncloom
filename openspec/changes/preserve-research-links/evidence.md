# Research preservation evaluation

## Scope and identities

Original launch base: `3a059a083887678e9d687378584c2f43e9f98da2`. Canonical parent reconciliation requested to `c0584e87a808f457bd7b205da3009c485243d4e0` or later; final parent/head/tree recorded after reconciliation. This change edits documentary records and their existing checker, not product runtime.

Loaded Code 5.4.2 (security/data, plus inspected integrations/llm/ui), Decide 1.3.1 with checklist 1.0.0 SHA256 `8c88c2c9c524d97552f67f30f6c4ba25589516b9dc18a51b09f7962688e5b056`, Phrona skill 1.0.2 bound CLI 5.0.50/release `11c3b43af82c465ffc1fd00a`, OpenSpec 1.16.0. Code's pre-edit reuse inventory found zero lexical candidates; broad owner search found existing cards/index/checker and Code Verify method checks. No parallel validator/catalog was created.

Received worker trace: `5454b3361e69e913c326ef616e107232`, read through `/api/public/v2/observations?traceId=5454b3361e69e913c326ef616e107232`, 50 observed rows. No legacy trace/list endpoint, service mutation, copied credential or personal header. Source setup was the current Fleet `system/skills/TRACING.md` and dotfiles' native tracing credential instructions; secrets stayed in memory. Private receipt is in the task data directory.

## Test-first checker evidence

- Before edits, `python3 -I scripts/check-research-records.py`: exit 0, zero problems on the launch tree despite omitted Q4/failed Q8 and stale Q1 pending row. This was not proof of complete research coverage.
- `python3 -B -I scripts/test_check_research_records.py` after six domain tests but before checker fix: exit 1, 31 tests selected, five intended failures. Nested records were treated as unhashed source files; required nested-field/backlink checks did not run. `checker-red.txt` retained privately.
- Same command after extending the existing owner: exit 0, all 31 tests passed. It includes actual CLI subprocess traversal, broken nested evidence links, complete identity backlinks, per-domain numbering and source-hash coverage. `checker-green.txt` retained privately. No test was skipped/deleted/loosened.

## Custody and retrieval

Final hash equality, local README/index/card/report traversal, exact request-bound shape check, strict OpenSpec and final Code Verify: **not yet run at this checkpoint**. Later results append here without treating pending work as a pass.

Q4's two original full reports and operator receipt/summary were copied unchanged from the explicit existing batch source. Q5/Q6 original post receipts were retained. Failed original Q8 and the home hosting request retain separate identities. All provider links/compositions/native incomplete outcomes and report hashes are in the cards. The current bound CLI could not locate Q1/hosting in its own home; retained original batch receipts were read instead, with no migration/run/re-Send.

## Decision and action boundary

New `research/0001` adopts reversible custody/navigation only. Historical 0002-0006 remain Proposed and unmarked; dated source/history reviews identify actual build commits without claiming fresh runtime proof. Prospective video/reads/scheduling packets, their frozen criteria and different-family challenge requests were sent to firstmate. Scheduler deferral and privacy/budget/account choices are not overridden. No product slice is claimed dispatched or built by the archive worker without a firstmate receipt. Phrona catalog IDs and hosting original locator remain pending owner delivery.
