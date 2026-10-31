# Adoption evidence

## Scope and bases

Original launch base: `3a059a083887678e9d687378584c2f43e9f98da2`. Before final validation this branch was rebased onto local main `d8ebd9fa5fe3cf8af0636a7eb2145dc26b9de320`, which already records kit 1.0.1 and retained Rust profile 1.0.0. This adoption changes only the maintained checker, consumer regression tests and records. Original-base Verify includes those intervening main changes; the new adoption delta does not own or relabel them.

Pin authority: docs/development.md. Durable raw receipts: `/Users/camiloslaptop/.firstmate-homes/syncloom-v2-lab/data/syncloom-kit102/`. Disposable profiles, builds and fixtures: `/tmp/fm-syncloom-kit102/`. The final project commit is reported externally to avoid a circular self-hash.

## Immutable release verification

Release: `d861f11cd52934a149077ded482c928cd5203472`. Manifest `system/shared/kit/releases.yaml` blob `8d6eca6e222acf1be59eb8420bf2b4dd730f424b`, SHA256 `58ba554e3fa6910a83d8aa8097460bf20dac210d5b9445fdd3af94727b1a7e67`.

| Component | Source commit | Git blob | SHA256 |
| --- | --- | --- | --- |
| Baseline 1.0.2 | `9df2415d95cf3e753a268734bed098f24ae027b9` | `c44e0ffa2d9facdfff58f91bb04cb8d6cb609622` | `c3cae84909ed326a5bed597a8370ca87ffa0e83e189e21d9ab95ecaa189f4b2b` |
| Checker 1.0.1 | `72ae87b8fc6feeb32996618eb5a228b719924319` | `366f60d4dc377bfd32bc2ec293516b26ce4aa44b` | `c707e00629112d75354791b06734fc0ca1adae1845772a11c08f10a837a917bf` |

`git -C /Users/camiloslaptop/github/fleet-system merge-base --is-ancestor <source> <release>` returned zero for every one of the 20 component identities. `git rev-parse <source>:<path>` and SHA256 of `git show <source>:<path>` match the manifest. Materialized-component release objects equal source objects. All unchanged component identity/version fields equal kit 1.0.1. `identities.json` retains each individual result, project config hashes and actual loaded Code hashes; `release.json` retains launch-baseline comparison.

Checker bytes equal the manifest exactly. Shared baseline bytes equal source after removing only Syncloom-added context. Project config SHA256 remains `0b434cfa8f1553a31a0eabece3a9525ba7519bd5506c9292fa48d6b21e9e50e4`, identical to the kit 1.0.1 parent. All nine project hashes in the development record match actual files.

Limits and unsuccessful probes: the originally documented pre-kit `4501883:system/shared/openspec/baseline.yaml` and external invoz `ddd9880:scripts/check-change-shape.py` paths are absent from Fleet's imported tree (exit 128). The preserved manifest legacy baseline verifies the former bytes; the latter is historical external provenance, not the maintained Fleet component source. No other project was read. An initially overstrict comparison of global tested-with trees to release HEAD failed for Code; Code and Decide HEAD trees differ from their historical tested-with pins. Source identities and ancestry verify correctly; global tested-with scope does not claim HEAD or installation equality.

## Actual loaded inputs and tracing

Code **5.2.2** SKILL SHA256: `d53176e1b89368edbc2e2f4dbd6bfdc20688f37957a243cc47491ea442723466`. Actual Verify SHA256: `8d37ad455935c3e60cef63d3099ea127e03bc7fcb2da3e9e4c82e191829750bf`, identical to the kit's Code 5.2.1 Verify pin. Findings and weakened-check scripts also match their kit hashes. The loaded skill version is not a changed global release pin.

Loaded references: security (secret/file/subprocess boundary), design (single rule owner, no wrapper), testing (independent expected values and actual CLI red/green). Their SHA256s are in identities.json. Read current config, development record, local checker, migration ledger and capability inventory; after rebase, reread current kit 1.0.1 adoption record. Read immutable Fleet kit README, adoption record guidance, OpenSpec README, baseline and release ledger. Stock spec-driven guidance creates a tooling-only change with skip_specs, no invented product spec. Generated adapters remain untouched. No consequential unresolved choice required Decide; no old decision is relabeled.

Received trace: `366104646976b248e7c0cfd8e2960920`, [local Langfuse](http://localhost:3000/project/fleet/traces/366104646976b248e7c0cfd8e2960920). Readback before implementation returned 50 observations scoped to this worker's trace, including its file-read tool call, session `01a1934f-7a3f-76fd-8f18-2b18424dbd6f` and run `a2ec6702-f203-45e4-b8a0-d697e26ed055`. Health was OK, Langfuse 4.56.0. Initial legacy trace GET and listing returned 404; after supervisor-authorized bounded read-only diagnosis, the documented `/api/public/v2/observations?traceId=...` route returned 200 using existing credentials in memory. This confirms receipt, not a tracing failure or new credential requirement. No unrelated trace inspection or service mutation occurred. Diagnose 1.3.2 and Fleet TRACING.md guided that distinction.

## Fresh checker acceptance

These are fresh synthetic adoption checks, not frozen diagnostics or paired attempts. No existing REJECT outcome was changed, interrupted or rerun.

Command: `python3 -B -I -m unittest discover -s scripts -p test_check_change_shape.py -v`.

- Launch checker: six tests selected; five failed subcases, including faithful multiline text rejected and four quoted physical lines reported instead of two paragraphs. Negative mismatch controls passed. Receipt: checker-red.txt.
- Prior kit 1.0.1 checker, extracted from parent into disposable fixtures: same suite exits 1 for the two multiline subcases; reviewer transitions remain valid. Receipt: checker-kit101-red.txt.
- Released checker: all six tests pass, including exact whitespace/internal newlines, empty request rejection, changed/missing/joined/split quote rejection, malformed reviewer rejection and independently valid reviewer-token transitions. Receipt: checker-green.txt. Reviewer identities are synthetic test text, not actual approval.
- Direct local entry point: `python3 -I -B scripts/check-change-shape.py /tmp/fm-syncloom-kit102/cli-accepted --request-source /tmp/fm-syncloom-kit102/cli-accepted/request.txt` exits 0, reports two paragraphs and zero problems. Same command with both paths changed to cli-mismatch exits 1, reports two paragraphs and a nonverbatim paragraph. Receipts: cli-accepted.txt, cli-mismatch.txt.

No local parser refactor: retaining byte identity is deliberate. Inventory before checker/config/docs edits found zero findings; additional README/test inventory also found zero. The stock CLI created only the initial .openspec.yaml scaffold before inventory.

## Validation

Initial integration Verify completed with exit 0, **ACCEPT_WITH_ADVISORIES**, tree `fd2f38329165b5d9e5a166746ebc4bedb37f915f`, original base `3a059a083887678e9d687378584c2f43e9f98da2`, 386.1 seconds. `verify-initial.txt` and its immutable receipt retain the individual results. After recording these outcomes, the identical command is run again on the final committed tree; that final receipt is retained externally as `verify.txt`, not recursively embedded in its own tested tree.

Exact command, from this worktree:

```sh
export TMPDIR=/tmp/fm-syncloom-kit102/
export CARGO_TARGET_DIR=/tmp/fm-syncloom-kit102/target
export CARGO_HOME=/tmp/fm-syncloom-kit102/cargo
export UV_CACHE_DIR=/tmp/fm-syncloom-kit102/uv-cache
export npm_config_cache=/tmp/fm-syncloom-kit102/npm-cache
export XDG_CACHE_HOME=/tmp/fm-syncloom-kit102/cache
export XDG_STATE_HOME=/Users/camiloslaptop/.firstmate-homes/syncloom-v2-lab/data/syncloom-kit102/quality-state
nix develop --offline path:/tmp/fm-syncloom-kit102/profile --command \
  python3 /Users/camiloslaptop/.agents/skills/code/scripts/verify.py \
  --repo /Users/camiloslaptop/.treehouse/syncloom.v2-6d3c6e/3/syncloom.v2 \
  --base 3a059a083887678e9d687378584c2f43e9f98da2 --change-kind bugfix \
  --check "tests=python3 -B -I -m unittest discover -s scripts -p test_*.py -v" \
  --check "rust=cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test" \
  --check "shape=python3 -B -I scripts/check-change-shape.py openspec/changes/adopt-fleet-kit-1-0-2 --request-source openspec/changes/adopt-fleet-kit-1-0-2/request.txt" \
  --check "research=python3 -B -I scripts/check-research-records.py" --timeout 900
```

The disposable Nix profile contains only flake.nix, flake.lock and rust-toolchain.toml extracted with `git show 840493182630729d32e5ee364f7037f01d47e1ed:system/shared/rust/<file>`. Observed Rust/Cargo 1.99.1, Clippy 0.1.99, Python 3.15.1, OpenSpec 1.15.0; exact version output is in tool-versions.txt. This avoids the older arbitrary nixpkgs toolchain described in the retained project caveat.

| Check / command | Initial result |
| --- | --- |
| `python3 -B -I -m unittest discover -s scripts -p test_*.py -v` | PASS, 31 tests including six checker CLI tests and 25 research-record tests |
| `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` | PASS, 232 Rust tests, zero failed/ignored; includes CLI and local-fake HTTP smoke/E2E journeys |
| Request-bound shape command above | PASS, zero problems |
| `python3 -B -I scripts/check-research-records.py` | PASS, zero problems |
| `openspec validate --all --strict --no-interactive` | PASS, 21 items; earlier pre-rebase run passed 18 items |
| Code `findings.py --repo . --base 3a059a083887678e9d687378584c2f43e9f98da2 --json` | PASS, zero blocking findings, two inherited kept copies |
| Verify native Rust-manifest lint | PASS, one workspace, default features; explicit command above covers all targets |
| `lychee --offline --include-fragments <receipt document scope>` | PASS, zero errors |
| Code `weakened_checks.py --repo . --base 3a059a083887678e9d687378584c2f43e9f98da2 --json` | Advisory FAIL, 176 suppression candidates; reviewed below, not called passing |
| `gitleaks git` with original-base prospective-commit range, `--redact` | PASS, no leaks |
| Original-base prospective `git diff --cached --check` | PASS, no whitespace errors |
| spec_check | NOT_APPLICABLE, OpenSpec owns this repository's specs |
| Vale | NOT_APPLICABLE, no project .vale.ini; not run/passed |
| Decision-record structural check | NOT_APPLICABLE, no decision record in changed document scope |

Full expanded internal commands, individual logs, timings, hashes and status are in receipt.json and verify-initial.txt. Before/after `git status --porcelain --ignored` files compare identical (cmp exit 0); Verify also reports mutated=false. No validation was interrupted or rerun to suppress a failure.

### Advisory adjudication

All 176 weakened-check candidates are pre-existing suppressions in 47 Rust files introduced by the intervening, already-landed kit 1.0.0 adoption. Every flagged file is byte-identical to parent d8ebd9f. `weakened-review.json` records each path, line, exact existing reason and parent-byte comparison: 125 fail-fast fixture annotations, 11 shared-fixture dead-code allowances, seven intentional terminal-output annotations, and 33 validated-value or fail-closed production invariants. Keep these previously authorized Syncloom-owner exceptions unchanged, not newly approved or relabeled here. No candidates occur in the kit 1.0.2 adoption delta.

A separate scoped diagnostic, `python3 /Users/camiloslaptop/.agents/skills/code/scripts/weakened_checks.py --repo /Users/camiloslaptop/.treehouse/syncloom.v2-6d3c6e/3/syncloom.v2 --base d8ebd9fa5fe3cf8af0636a7eb2145dc26b9de320 --json`, exits 0 with zero candidates (weakened-adoption-delta.json). This does not replace the required original-base advisory result.

The two findings kept by prior records are independent fake-example CLI scaffolding and enum derive/serde boilerplate for independently owned Status and Language types. Their owner rationales remain in the kit 1.0.0 and language-routing design records; this adoption does not add or expand those waivers.

Other unsuccessful setup probes: `python3 -c 'import yaml; print(yaml.__version__)'` failed because PyYAML is absent; the immutable manifest was parsed with the installed Ruby YAML safe loader instead, without adding a dependency. The rebase had expected docs/checker conflicts; resolved by preserving main's existing kit record and importing the exact released checker, not by changing upstream rules.

## Exceptions and unmeasured coverage

No new configuration or security exception. Preserve the prior Syncloom-owned function-scoped lint expectations, workspace-to-member inheritance mapping, project metadata/dependencies/lockfile and the reviewed Nix/MSRV caveat. The released checker is retained byte-exact, not reformatted locally. Rust profile 1.0.0 remains selected; Nix files are disposable validation inputs, not new project material. Optional deny/mutants reports, nextest execution and non-Darwin platforms are not selected or claimed. No global Code/Decide installation, program-outline candidate, account access, new permission/spend/publication or paid call is authorized or performed. Product runtime is unchanged; its existing local-fake smoke/E2E coverage is exercised by cargo test, without real platforms.
