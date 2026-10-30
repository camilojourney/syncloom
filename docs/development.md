# Development

## OpenSpec configuration

`openspec/config.yaml` is a reviewed copy of the Fleet shared OpenSpec baseline **1.0.0**, commit `4501883843df70c50f0726120aca09387ca6c95d` (`system/shared/openspec/baseline.yaml` in fleet-system), taken through invoz.v2's `openspec/config.yaml` (blob `4027f6d60755ac77927a6f0e5db65a61968fb85b`, which carries the same baseline). Every baseline line is byte-identical; only the Syncloom context lines appended to `context:` differ, and they replace invoz.v2's project lines.

Re-check: save the baseline with `git -C <fleet-system> show 4501883843df70c50f0726120aca09387ca6c95d:system/shared/openspec/baseline.yaml > /tmp/base.yaml`, then `diff /tmp/base.yaml openspec/config.yaml`. Only added context lines may appear. On a baseline update, carry the common diff by hand and run `openspec validate --all --strict --no-interactive`.

OpenSpec CLI 1.15.0, initialized with `openspec init --tools codex,claude,pi` (stock `spec-driven` schema; the generated skills and commands under `.agents/`, `.claude/` and `.pi/` are not hand-edited).

## Toolchain

Rust, pinned in `rust-toolchain.toml` (1.98.1). `cargo` is not on PATH on this machine; run every check through nix:

```sh
nix shell nixpkgs#cargo nixpkgs#rustc nixpkgs#clippy nixpkgs#rustfmt -c sh -c 'cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test'
```

## Record checkers

`scripts/check-change-shape.py`, `scripts/check-research-records.py` and `scripts/test_check_research_records.py` are byte-identical copies of invoz.v2's (sha256 `54f77643ee4e4cdc3cfc9f66b20cda5413efc9dfd95982532d58e62fd14b74f2`, `20503765e214235c8e002007e7bad7501f2b2ec8d9081741a6a550a5f34ebae1` and `efcefaf2ceb7d4faf89fb3a26cbb36a5557e89ab25171d0d9d0159f5a6a5f853`). They stay Python on purpose: the rule "Rust for all first-party code" (vision 1.2) covers the product, and these are repository-record checkers shared with invoz.v2 whose owner is the invoz.v2 copy. Porting them would create a second owner of the same rules; running them from invoz.v2's path would couple the two repositories. When invoz.v2 changes a checker, copy it here and re-run the tests. The checkers are stdlib-only and run with `python3 -I`.

## Hermetic tests

`cargo test` uses temporary directories only. Proof: `git status --porcelain --ignored` is captured before and after a run and must be identical. Runtime data lives outside the repository (`SYNCLOOM_DATA_DIR`, default `$HOME/.syncloom`).

The Keychain tests write to the real login Keychain under a service named `syncloom-test-<pid>-...` and delete the item before they finish, also when they fail. Check for leftovers with `security dump-keychain | grep syncloom-test` and remove one with `security delete-generic-password -s <service> -a linkedin`. The Keychain binds an item to the code hash of the binary that created it, so the CLI test (a different binary from `syncloom`) checks presence with `security find-generic-password` and never reads the secret; the core credentials test reads it back through one binary. The same binding means a rebuilt `syncloom` or `syncloom-server` asks once, in a GUI terminal, before reading a token an earlier build stored; from a shell without a GUI session the read fails with `platform_failure` in the log.

The fake LinkedIn (`crates/syncloom-core/tests/support/fake_linkedin.rs`, also the `fake_linkedin` example) and the fake Claude never leave the loopback; no test or example reaches api.linkedin.com or api.anthropic.com.

## Lessons applied

Every build task applies `data/v2-build-lessons-20261030.md` of the firstmate home (25 items: OpenSpec 1.15.0 with this config, Code 5.0.0 loaded first, test first, verbatim requests, research saved word for word, Receipts-first reports, no secrets or personal data in requests or files, `gitleaks git` mode, loopback-only servers with Host check and a fixed error envelope, hermetic tests, scratch under `/tmp/fm-<task>/`, Rust via nix, Rust for first-party code, rebase onto the current local `main` and re-run the gates before done, and the Code skill's Verify gate with the weakened-checks analysis run on the final tree with its output saved as `verify.txt` in the task's data folder). A worker that finds one of them wrong says so in its report.
