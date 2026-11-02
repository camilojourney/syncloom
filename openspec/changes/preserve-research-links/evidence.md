# Research preservation evaluation

## Scope, versions and provenance

Original launch base: `3a059a083887678e9d687378584c2f43e9f98da2`. Reconciled canonical parent: `c0584e87a808f457bd7b205da3009c485243d4e0`. Checkpoint head: `39fea51bd8f7be22f83d074d9765233c05b8820e`. The evaluated pre-final-documentation tree was `b3072e63da654ef556dda35f5236004134cbf83d`. Final committed-head/tree receipts are retained externally in the task data directory, so recording a receipt does not change the tree it identifies.

Delivered scope is documentation only versus the canonical parent. The rejected checker experiment below is not shipped. No runtime, Rust manifest, platform permission, frontend design or shared checker change is delivered.

Loaded Code **5.4.2** and its design, review, security, data, integrations, llm and ui references; Decide **1.3.1** and required checklist/contract/workflow/record/evidence/evaluation/code references; checklist **1.0.0**, SHA256 `8c88c2c9c524d97552f67f30f6c4ba25589516b9dc18a51b09f7962688e5b056`; Phrona skill **1.0.2**, installed bound CLI **5.0.50**, release `11c3b43af82c465ffc1fd00a`; OpenSpec **1.16.0**. Original study versions 5.0.12/5.0.13 are not replaced by the current reader's version. The method record pins its actual source and skill hash.

Durable receipt root, outside Git: `/Users/camiloslaptop/.firstmate-homes/syncloom-v2-lab/data/syncloom-research-adoption/` (called `DATA` below). `inventory.md` was saved before substantial edits, with exact questions, provider/report links, receipt provenance, composition, hashes and actual implementation history. `report-hashes.json` records each report's original source path, repository path, byte length and SHA256. Its explicit batch source is the pointer already present in repository cards, not a sweep through unrelated homes. `reuse-inventory.txt` records the pre-edit lexical inventory; broader owner search found the existing cards/index/checker and Code method checks.

Received worker trace **`5454b3361e69e913c326ef616e107232`**, 50 matching observed rows, using `/api/public/v2/observations?traceId=5454b3361e69e913c326ef616e107232`. `DATA/trace.json` retains the scoped receipt. Current Fleet `system/skills/TRACING.md` and native tracing credential instructions were followed before implementation. Credentials stayed in memory; no legacy trace/list route, personal contact header or service mutation was used.

## Baseline, failed attempts and owner correction

These outcomes remain failures, not retroactive passes:

| Command or experiment | Observed result and disposition |
|---|---|
| `python3 -I scripts/check-research-records.py` on launch tree | Exit 0, zero problems despite omitted Q4/failed Q8 and stale Q1 pending row. Baseline did not establish inventory completeness. |
| `python3 -B -I scripts/test_check_research_records.py`, local recursive candidate before its implementation | Exit 1, five failures among 31 selected tests. `checker-red.txt`. |
| Same candidate after implementation | Exit 0, 31 tests. `checker-green.txt`. Firstmate rejected the local fork; this is NOT delivered coverage. |
| Restored immutable `python3 -B -I scripts/check-research-records.py` | Initially exit 1, nested method record unlisted as a file. `legacy-domain-rejection.txt`. Used its existing per-file custody hash rule instead; not nested structural acceptance. |
| First parent-scoped record/shape checks | Record suites passed 25 + 6; shape failed three requirements: Request-first heading and capability/duplicate tokens. Corrected proposal without altering the source request. `records-initial.txt`. |
| Initial copied retrieval fixture | Exit 1, two links to root README broken because fixture omitted README. Fixture defect, not product defect. Added root README to disposable fixture; `retrieval-initial.txt` retained. |
| Public Code Verify with temporary `C1` status `unsupported` | Exit 1, REJECT: invalid Claims status. Also found a real trailing EOF blank in decision README; fixed that separately, restored valid claim. `method-negative.txt`. |
| `python3 -B /Users/camiloslaptop/.agents/skills/code/scripts/verify.py --decision-records docs/decisions/research/0001-research-custody-and-retrieval.md`, with temporary nonexistent Q4 citation | Exit 1, decision structure/citation FAIL; restored exact valid record. `citation-negative.txt`. |

The original `research-custody-freeze.md` retains its future-clock typo and contemporaneous correction. Tool-call ordering establishes its pre-implementation placement; the typo is not rewritten as timely evidence. The dated C5 mechanism amendment preserves the original criteria and reassesses alternatives after the ownership correction.

Final checker SHA256 `20503765e214235c8e002007e7bad7501f2b2ec8d9081741a6a550a5f34ebae1`; test SHA256 `efcefaf2ceb7d4faf89fb3a26cbb36a5557e89ab25171d0d9d0159f5a6a5f853`, both exact canonical-parent/adopted bytes. Existing flat ADR fields and file hashes are their scope. Current Code Verify owns structure/citations of the new method-marked record. No new validator, wrapper, owner exception or relaxed test is shipped.

## Custody and end-user retrieval

Followed the actual repository README to research INDEX, all 12 cards (10 Phrona-related identities including the request-only hosting record, two non-Phrona scouts), then all 12 full provider reports. Each report equals its named original batch file byte-for-byte and its recorded SHA256. Exact provider URLs and actual composition are in cards; offline navigation is not remote reachability or factual validation.

Real local entry point: `python3 -B -I scripts/check-research-records.py --root <disposable-fixture>`. The fixture copies the actual root README, research and decision records, with code targets retained as needed. No product wrapper or checker fork is involved. `DATA/retrieval.txt` records:

- Healthy complete fixture: exit 0, zero problems.
- One corrupted original report byte: exit 1, recorded SHA256 mismatch.
- Broken card-to-report link: exit 1, broken link.
- Broken INDEX-to-card link: exit 1, broken link plus orphan/missing-row diagnostics (three problems).
- Restored fixture: exit 0, zero problems.

Q4's two previously omitted full reports, original operator summary and post receipt are copied unchanged. Q5/Q6 post receipts and failed original Q8 request retain separate provenance. The hosting card retains the exact request, not a reconstructed report. The read-only installed bound launcher `phrona result --id main969-q1-xtb` and `phrona result --id syncloom-v2-meta-media-hosting-20261031` each exited 1, no study in that bound home. Explicit retained original batch receipts supplied the historical custody; no migration/run/resume/re-Send/provider operation occurred. Main1323's owner search found no hosting match among 95 modern and 181 legacy studies, and this home's inventoried records contain no submission receipt. That scoped locator request is closed, not an indefinitely pending report or evidence of no studies elsewhere.

Native incomplete outcomes remain incomplete. Source-free HAND reports remain distinguished from native challengers; missing challengers are not agreement. Contradictory two-report operator summaries are preserved, with annotations only in cards. No original report bytes were edited or annotated.

## Exact integration command and results

The full invocation below ran sequentially once per base on the evaluated tree. `BASE` was first the original launch SHA above, then canonical parent SHA above; the original-base result was not replaced by the narrower diagnostic.

```sh
ROOT=/Users/camiloslaptop/.treehouse/syncloom.v2-6d3c6e/4/syncloom.v2
DATA=/Users/camiloslaptop/.firstmate-homes/syncloom-v2-lab/data/syncloom-research-adoption
CARGO_HOME=/tmp/fm-syncloom-research-adoption/cargo \
CARGO_TARGET_DIR=/tmp/fm-syncloom-research-adoption/target \
CARGO_NET_OFFLINE=true PYTHONDONTWRITEBYTECODE=1 \
XDG_STATE_HOME="$DATA/quality-state" \
XDG_CACHE_HOME=/tmp/fm-syncloom-research-adoption/cache \
TMPDIR=/tmp/fm-syncloom-research-adoption \
nix develop --offline path:/tmp/fm-syncloom-research-adoption/profile --command bash -c '
  python3 -B /Users/camiloslaptop/.agents/skills/code/scripts/verify.py \
    --repo "$1" --base "$2" --change-kind docs \
    --doc docs/decisions/research/0001-research-custody-and-retrieval.md \
    --check "tests=python3 -B -I scripts/test_check_research_records.py && python3 -B -I scripts/test_check_change_shape.py" \
    --check "records=python3 -B -I scripts/check-research-records.py" \
    --check "shape=python3 -B -I scripts/check-change-shape.py openspec/changes/preserve-research-links --request-source $3/request-source.txt" \
    --timeout 600
' verify "$ROOT" "$BASE" "$DATA"
```

Locked profile files were extracted read-only from Fleet `840493182630729d32e5ee364f7037f01d47e1ed:system/shared/rust/` into the named disposable directory. Cargo registry was APFS-cloned there, not mutated in the shared cache. Actual Rust/Cargo **1.99.1**, Clippy **0.1.99**. No install or paid call. Original-base native lint executes all-target workspace Clippy/default features, not a full cargo test suite.

Pre-final-documentation receipts:

- `DATA/verify-original.txt`, exit 0, **ACCEPT_WITH_ADVISORIES**, 47.9 seconds. Immutable `receipt.json` under `quality-state/fleet/verify/91424a1e1aca3391a0167f5b/b3072e63da654ef556dda35f5236004134cbf83d-3a059a083887678e9d687378584c2f43e9f98da2/run-928xkjne/`.
- `DATA/verify-parent.txt`, exit 0, **ACCEPT**, 1.6 seconds. Receipt under `quality-state/fleet/verify/516bb1f25bc95d80cb5dd28e/b3072e63da654ef556dda35f5236004134cbf83d-c0584e87a808f457bd7b205da3009c485243d4e0/run-56pdxcar/`.
- Final committed-tree recheck results belong to `DATA/final-report.md` and `verify-final-*.txt`; no unexecuted outcome is asserted by this document.

| Check / actual command owner | Original-base observed result | Parent diagnostic |
|---|---|---|
| Explicit Python suites above | PASS, 25 research + 6 shape tests, none skipped | PASS |
| Explicit research CLI above | PASS, zero problems | PASS |
| Explicit source-bound change shape above | PASS, exact two request paragraphs, 236 words, zero problems | PASS |
| `openspec validate --all --strict --no-interactive` | PASS, 22 items | PASS |
| Code `findings.py --base <base> --json` | PASS, zero blocking, two inherited kept copy candidates | N/A, no source changes |
| Code `findings.py --base <base> --json --residue` | **FAIL advisory**, two new lexical identities, scope non-comparable | N/A, no source changes, NOT a measured zero |
| Native Code Rust manifest lint command | PASS, one workspace, all targets/default features, 47.3 seconds | N/A, Rust unchanged |
| Code `spec_maps.py` | PASS advisory structural scan: zero maps/16 specs, zero links/174 scenarios, 233 unknown | 174 unknown; not semantic coverage |
| `lychee --offline --include-fragments` on selected documentation | PASS, zero errors | PASS |
| Code `--decision-records` on changed ADRs/method record | PASS, one opted-in record; five historical unmarked records skipped | PASS, same scope |
| Code `weakened_checks.py --base <base> --json` | **FAIL advisory**, 176 inherited suppression candidates | PASS, zero candidates |
| `gitleaks git` with redaction on original-base round-trip diff | PASS, no leaks | PASS |
| Git diff whitespace check | PASS | PASS |
| Legacy spec_check; Vale | N/A: OpenSpec owns specs; no `.vale.ini` | Same |

Full expanded individual commands, timings, logs and mutation checks are in each immutable receipt, not inferred from the aggregate verdict. `DATA/records-green.txt` also retains standalone research CLI, 25 + 6 suites, source-bound shape and strict OpenSpec results.

## Advisory review, not a favorable relabel

Read complete original-base residue/weakened/findings logs. `DATA/weakened-review.json` records all **176** exact candidate paths/lines/source reasons, whole-file hash equality to canonical parent and the prior project-owned exception authority in `docs/development.md`: 125 fail-fast fixture annotations, 11 fixture dead-code allowances, seven terminal-output annotations and 33 validated-value/fail-closed invariants. All are unchanged from parent; zero introduced by this documentation slice. This is retention of earlier Syncloom maintainer authorization, not new worker approval. Original-base weakened status remains advisory FAIL.

`DATA/residue-review.json` retains raw versions/scopes/exclusions and one disposition per identity/config change. Complexity lizard 1.24.3/complexipy 8.2.0 measured zero base/current/new over-limit functions in scope. jscpd 5.4.2 raw base 2/current 3/new 2/historical 1; both new identities are unchanged inherited source versus parent. The example pair shares intentionally independent provider entry-point scaffolding; Piece/ThoughtContract shares serde/field syntax, not one mutable domain rule. Adjudication: two false semantic-duplication findings, intentionally retained with Syncloom maintainer ownership; no raw identity removed. Historical example scaffold retained. Cargo-machete 0.9.2 raw base/current 1/new 0: server `bytes` dependency remains a LOW deferred runtime-owner lexical-use candidate, not proven symbol reachability. No unrelated dependency deletion is made.

Five original-base analysis config changes (root/member manifests and clippy.toml) are unchanged adopted Fleet Kit configuration; all recorded individually with hashes. All suppression dispositions are cross-referenced. Scope is **non-comparable**, 55 selected source files, tests/build/spec roots excluded by existing tool scope. No comparable no-new, semantic completeness or zero-dead-code claim follows. Knip/Periphery are N/A, not successful zero scans. Raw tool unreviewed counters remain in logs; the separate review disposes the two new identities, historical scaffold/dependency and five scope changes, without changing mechanical FAIL status.

## Decision, build and authority boundaries

New [research/0001](../../../docs/decisions/research/0001-research-custody-and-retrieval.md) adopts only reversible custody/navigation, with current method identity, frozen brief, evidence and firstmate outcome owner/review **2026-11-09**. Historical 0002-0006 remain Proposed/unmarked; dated reviews link actual commits, code and tests without pretending they used current Decide or were rerun here.

Separate conclusion-free Q4/Q6 packets went to firstmate for permitted different-family challenge. Before any challenge-output access, independently sealed `DATA/primary-initial.md` at 2026-11-02T14:26:12Z, SHA256 `2a792029dd93b35418287c898e75b1a2be5315cc54b370e53fa8b1f7a3127000`. Recommendations are metadata-only manual YouTube handoff and explicit non-fetching/non-model local reference attribution, with alternatives, criterion defenses and falsifiers. They are NOT accepted product decisions, dispatched slices or delivered implementations. Firstmate owns controlled reveal/resolution. The earlier scheduling packet does not reopen the existing accepted vision 7.1 deferral. No new scheduler build/decision is authorized. Content-renderer Q7 links are ready for the independent `syncloom-text-ui` owner; no competing stack or palette is chosen.

Phrona domain/catalog mapping remains **pending Main1308 owner delivery** for every entry; repository scope label `syncloom.v2` is not an invented catalog ID. Missing hosting findings cannot support a host/privacy/budget choice. Catalog delivery does not block independent custody.

No runtime change versus parent: existing product smoke is N/A for this documentation slice. The prior kit owner's 232 Rust tests are historical evidence, NOT newly executed here. No live account, paid platform, posting, browser/provider operation, remote reachability test, push, remote pipeline or shared-service mutation occurred. Stock OpenSpec proposal review remains pending; final commit is for firstmate's local review/landing, not self-approved archival.
