# Development

## Shared Kit adoption

Kit **1.0.0** is adopted from Fleet release commit `e9ce8fecb722848654abb6f887f50939b7e5bdc1`, following `system/shared/kit/README.md` and its release ledger. Pinned in: OpenSpec change `2026-10-31-adopt-shared-kit-1-0-0` and the project commit containing this record. The project commit is the adoption authority, not a global skill activation.

Components: OpenSpec baseline **1.0.1**, Rust profile **1.0.0** (Rust **1.99.1**), quality configuration **code-5.2.0**, Code **5.2.0** and Decide **1.1.0** tested-with references. All selected materialized sources and quality files below come from component commit `840493182630729d32e5ee364f7037f01d47e1ed`, verified as an ancestor of the release. Decide comes from ancestor `367a5c1ff818696c6b3f4e34cf438e086ce80adb`.

### Immutable source identities

Paths are relative to Fleet; hashes are SHA256 unless labeled as Git trees.

| Source | Hash |
| --- | --- |
| `system/shared/openspec/baseline.yaml` | `fb3e135d4c29607dfd9adbf4b96f1554206f90cfa134d93900932939b2a4498a` |
| `system/shared/rust/Cargo.toml` | `79fd95c4616b532ba7e1751b090f5d7ff030507d85512ecdc61f27fe0ac0b495` |
| `system/shared/rust/rust-toolchain.toml` | `f715f440167e20fd8ea9afa0498c3d1ab80174cfcb8cbc895f106a34d907a1e1` |
| `system/shared/rust/clippy.toml` | `741d8b6cbd57d7b01ff8918083bbf313e281cce0dbdcb1ed669dc8cb84f44578` |
| `system/shared/rust/.config/nextest.toml` | `7e08fd48e32d6224af50bc2cd5dd205d87f22bbd8c9499dcd836937c0a7148f0` |
| `system/skills/platform/code` | Git tree `bc87759cac3739e97919fd76d411d5bbb3c8cc76` |
| `system/skills/platform/decide` | Git tree `b92df4099dd2602d40fb049e54840c8cfb0e1fe8` |
| `system/skills/platform/code/scripts/verify.py` | `6b45b23dbf21eb575753a82c4b689cf4e2e9342988a447b6a558d5b34da19d92` |
| `system/skills/platform/code/scripts/findings.py` | `472c9eccb3e0fdaddbaf6d369ab4e3dd3cdca1474fe280286f0005328ba13944` |
| `system/skills/platform/code/scripts/weakened_checks.py` | `faf6c56d1e8b5437d46bc6d432824e555e2c19a62ee880e55d812ef769638e45` |
| `.spec-check.toml` | `3ade0cdec731f2721b95514f51c9be4287f6fa4ef92c193ab44cf9cb404dc595` |
| `.vale.ini` | `f1e8361f4f119ecc9001577442fa50d1200f101d56ff8fd0d342e2a104e7ae18` |
| `ruff.toml` | `acef2f0e0a1cc252d636fda5e9ac6bbb7ddb7bb7bf7d8bb7ade6f78254024212` |

Quality delegates to Fleet's `system/skills/platform/specs` (Git tree `ba3bc7c5398d809219356a5d824105527a08e280`) and `system/skills/meta/skill-creator-fleet/assets/vale` (Git tree `20ec6a0b169b7cbe32e78c3fbc158c42a5da46ef`). These global tested-with references are not copied into the project or enforcement pins.

### Reviewed project configuration

| Project file | SHA256 |
| --- | --- |
| `openspec/config.yaml` | `7b6eb5f4b64d5b39f132db48657fa2c5ed7fd210378b2aaf4bca340e7fa94d37` |
| `Cargo.toml` | `c1fcf57b361df273a6209225523c3f2332aeab3ada44f7bc339619900cb3e6d6` |
| `rust-toolchain.toml` | `f715f440167e20fd8ea9afa0498c3d1ab80174cfcb8cbc895f106a34d907a1e1` |
| `clippy.toml` | `741d8b6cbd57d7b01ff8918083bbf313e281cce0dbdcb1ed669dc8cb84f44578` |
| `.config/nextest.toml` | `7e08fd48e32d6224af50bc2cd5dd205d87f22bbd8c9499dcd836937c0a7148f0` |
| `crates/syncloom-core/Cargo.toml` | `92d10b04838bb6ed983062aff2edf104392d0c404564f88d7fe748433a1d6de1` |
| `crates/syncloom-server/Cargo.toml` | `01dd028586c50aacf525ebeecf55170dce6325077fb5e08e8b3841052bc214ce` |
| `crates/syncloom-cli/Cargo.toml` | `215b8216aee9061bd7d792166d045eb4ba87b785d6e77094b92f5dfdae7bf347` |

Baseline common lines match the source exactly; all pre-existing Syncloom context lines below the common block are preserved. The obsolete `/code 5.0.0` literal is removed from common guidance. Prior baseline 1.0.0 was pinned at `4501883843df70c50f0726120aca09387ca6c95d`; its retained source hash is `47bd2a780e24184cc60f03fc3cd12df3aad1c48a94cab7bdd1538c7e0b605f00`.

Rust lints and release sections match Fleet. Project metadata, dependencies and Cargo.lock remain project-owned and unchanged. Every member inherits workspace lints and MSRV. Package-level root `[lints]` is N/A: Cargo rejects it in this virtual workspace; firstmate approved the correct root `workspace.lints` / member `lints.workspace` mapping in steering message 001. Nix files are not materialized in the project; optional deny/mutants reports are not selected. Nextest timeout configuration is adopted, but nextest execution is not claimed.

Configuration exceptions: none for selected shared sections. Firstmate message 002 authorizes local function-scoped `#[expect]` annotations for verified invariants, fail-closed mutex poisoning, intentional CLI output and fail-fast test fixtures. Each carries its reason in source; these are explicit project lint exceptions owned by the Syncloom maintainer, not global profile weakening. Existing shared-fixture dead-code allowances gain reasons without broadening their scope. Runtime behavior, security guards and test assertions remain unchanged.

### Actual loaded inputs and verification

Initially loaded Code at `/Users/camiloslaptop/.agents/skills/code/SKILL.md` was version 5.2.0, SHA256 `76f2f50100e291057b607f6cb8279f87269aa23dd6d1075937482a96ea16ca2c` (Git blob `33e8f65f1d72d7da103a1cfac8ea8f0053ea8246`, matching the kit component). A separate machine-owner update changed the global symlink target during this task to Fleet commit `f63507c26ebf9510bec1e92f692919ec62f12270`: Code 5.2.1 was re-read before Verify, SKILL.md SHA256 `efaa1df842147b98f515fa09615f9d7cb213d6ce6b295f41ccd6f625258dcc3a`, verify.py SHA256 `8d37ad455935c3e60cef63d3099ea127e03bc7fcb2da3e9e4c82e191829750bf`. The loaded findings.py and weakened_checks.py still match the kit hashes above. Topic reference loaded: `references/security.md`, SHA256 `12ff1d9cc04eee151fe92502402f1df16b31e510249a26d9e9b29ce86cfb8e37`; no authorization or secret-handling boundary changes. Decide was not invoked: 1.1.0 is the kit's tested-with reference, not evidence of a locally executed decision procedure. Other global quality inputs are release references only unless the Verify receipt records their actual use.

Receipts live outside the worktree at `/Users/camiloslaptop/.firstmate-homes/syncloom-v2-lab/data/syncloom-v2-adopt-shared-kit-1-0-0/`.

| Problem (source) | Owner | Change | Regression or acceptance | Result |
| --- | --- | --- | --- | --- |
| Stale baseline guidance and no kit pin | Fleet baseline / Syncloom | Reviewed 1.0.1 merge and this record | Source hashes, retained project context and strict OpenSpec validation | Pass: 18 items |
| Virtual root cannot inherit package lints | Cargo / Syncloom | Correct member inheritance | Cargo 1.99.1 metadata; minimal rejection receipt `virtual-manifest.txt` | Pass with approved N/A root mapping |
| New strict profile exposes existing invariant panics and fixture/output lint findings | Syncloom | Narrow reasoned expectations, no runtime edits | `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` | Pending final workspace run |
| Adoption shape and record integrity | Syncloom | Request-first proposal and adoption record | Change-shape checker, research checker and checker unit tests | Pass: 0 problems; 25 tests |
| Final integrated quality | Code / Syncloom | Installed Code Verify with native project checks | `python3 /Users/camiloslaptop/.agents/skills/code/scripts/verify.py --base main --change-kind refactor` with explicit checks | Pending; receipt `verify.txt` |

Failed attempts are retained: initial Clippy reported 36 core production violations and stopped before tests (`workspace.txt`); subsequent `clippy-round2.txt` through `clippy-round6.txt` capture annotation placement corrections and additional targets exposed as compilation advanced; `clippy-final.txt` passes. Initial source comparison caught an incomplete toolchain copy, corrected to byte-exact profile bytes; the first shape check required explicit none declarations, now present. No tests were skipped, loosened or re-baselined.

## OpenSpec configuration

OpenSpec CLI 1.15.0 uses the stock `spec-driven` schema. Existing generated skills and commands under `.agents/`, `.claude/` and `.pi/` were initialized with `openspec init --tools codex,claude,pi` and remain unmodified. Re-check baseline updates with `git -C <fleet-system> show 840493182630729d32e5ee364f7037f01d47e1ed:system/shared/openspec/baseline.yaml`, compare SHA256, then review only common guidance while preserving project context. Run `openspec validate --all --strict --no-interactive`.

## Toolchain

Rust is pinned to 1.99.1 with rustfmt and Clippy. Verified native versions: rustc 1.99.1 (`58cf8f9c3`), Cargo 1.99.1 (`5f94df478`), Clippy 0.1.99. Arbitrary `nixpkgs#cargo` currently selects Rust 1.98.1 here and does not honor the toolchain file. The preserved OpenSpec context's old Nix invocation must not be mistaken for realization of this pin.

With rustup, use the pinned `cargo` from the repository. On this Nix-only machine, extract Fleet's locked profile outside the worktree and use it explicitly:

```sh
FLEET=/Users/camiloslaptop/github/fleet-system
SOURCE=840493182630729d32e5ee364f7037f01d47e1ed
PROFILE=$(mktemp -d)
for file in flake.nix flake.lock rust-toolchain.toml; do
  git -C "$FLEET" show "$SOURCE:system/shared/rust/$file" > "$PROFILE/$file"
done
CARGO_TARGET_DIR=/tmp/fm-shared-kit/target nix develop "path:$PROFILE" --command sh -c 'rustc --version; cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test'
```

This task realized that profile successfully on aarch64-darwin; other platforms and optional reports remain unverified.

## Visual renderer tests

The [visual asset guide](visual-assets.md) pins the installed browser and font and describes the snapshot tolerance. Tests launch that existing binary; they never install or download one. Use `CARGO_TARGET_DIR=/tmp/fm-<task-id>/target` and `TMPDIR=/tmp/fm-<task-id>/` for build/test scratch. `nix --offline shell` avoids refreshing the toolchain's flake metadata during hermetic validation. Chromium's product version and the font checksum are checked by the renderer before producing assets.

## Record checkers

`scripts/check-change-shape.py`, `scripts/check-research-records.py` and `scripts/test_check_research_records.py` are byte-identical copies of invoz.v2's (sha256 `54f77643ee4e4cdc3cfc9f66b20cda5413efc9dfd95982532d58e62fd14b74f2`, `20503765e214235c8e002007e7bad7501f2b2ec8d9081741a6a550a5f34ebae1` and `efcefaf2ceb7d4faf89fb3a26cbb36a5557e89ab25171d0d9d0159f5a6a5f853`). They stay Python on purpose: the rule "Rust for all first-party code" (vision 1.2) covers the product, and these are repository-record checkers shared with invoz.v2 whose owner is the invoz.v2 copy. Porting them would create a second owner of the same rules; running them from invoz.v2's path would couple the two repositories. When invoz.v2 changes a checker, copy it here and re-run the tests. The checkers are stdlib-only and run with `python3 -B -I`; `-B` prevents the dynamically imported checker test from leaving bytecode in the repository.

## Hermetic tests

`cargo test` uses temporary directories only. Proof: `git status --porcelain --ignored` is captured before and after a run and must be identical. Runtime data lives outside the repository (`SYNCLOOM_DATA_DIR`, default `$HOME/.syncloom`).

The Keychain tests write to the real login Keychain under a service named `syncloom-test-<pid>-...` and delete the item before they finish, also when they fail. Check for leftovers with `security dump-keychain | grep syncloom-test` and remove one with `security delete-generic-password -s <service> -a linkedin`. The Keychain binds an item to the code hash of the binary that created it, so the CLI test (a different binary from `syncloom`) checks presence with `security find-generic-password` and never reads the secret; the core credentials test reads it back through one binary. The same binding means a rebuilt `syncloom` or `syncloom-server` asks once, in a GUI terminal, before reading a token an earlier build stored; from a shell without a GUI session the read fails with `platform_failure` in the log.

The fake Mastodon (`crates/syncloom-core/tests/support/fake_mastodon.rs`), fake LinkedIn (`crates/syncloom-core/tests/support/fake_linkedin.rs`, also the `fake_linkedin` example) and fake Claude never leave the loopback; no test or example reaches api.linkedin.com or api.anthropic.com.

## Mastodon and export checks

`cargo test -p syncloom-server --test mastodon_export` runs the end-user HTTP journey: per-instance registration, consent, rendering, exact-byte approval, async media with alt text, status creation, restart/repeat and manual export. `cargo test -p syncloom-core --test mastodon --test manual_export` covers failure boundaries. The CLI suite covers manual export; the credentials suite round-trips both expiring and non-expiring tokens through a test-only Keychain item. No test signs in to a real platform.

Meta's stateful fake (`crates/syncloom-core/tests/support/fake_meta.rs`) proves the Page and Instagram protocols without contacting Meta. Commands and limitations: [Meta integration](meta.md). An offline nix shell needs the toolchain already cached; if it selects uncached derivations, the normal online nix shell can obtain the binary substitutes before hermetic tests.

OAuth listener/state/JSON bounds are shared in `crates/syncloom-core/src/oauth.rs`; fixed platform errors and transport outcomes remain shared. The OAuth session owns the immutable credential account, callback configuration and lifecycle; TokenStore owns usable-token lookup, so adapters cannot independently drift on expiry or sign-out policy. An aborted callback listener is awaited before the waiting sign-in returns, so its port is actually free. Use the task's red, mutation and final-gate logs as receipts, not the presence of these instructions.

## Q1 platform checks

[Q1 platform guide](social-platforms.md) lists configuration and exact operations. `cargo test -p syncloom-server --test social_delivery --test social_channels` proves local HTTP journeys for Bluesky, X and Threads; `cargo test -p syncloom-core --test social_sessions --test social_boundaries --test social_rules` covers wire, refresh, state, media and price boundaries. ATrium API is pinned at 0.26.0 with a bounded custom transport; no SDK session file or DID-directed fetch is used. Each fake has its own loopback listener and only its platform endpoints.

## Lessons applied

Every build task applies `data/v2-build-lessons-20261030.md` of the firstmate home (27 items: OpenSpec 1.15.0 with this config, the installed Code skill loaded first (initially 5.2.0, then 5.2.1 for this adoption), test first, verbatim requests, research saved word for word, Receipts-first reports, no secrets or personal data in requests or files, `gitleaks git` mode, loopback-only servers with Host check and a fixed error envelope, hermetic tests, scratch under `/tmp/fm-<task>/`, Rust via nix, Rust for first-party code, rebase onto the current local `main` and re-run the gates before done, and the Code skill's Verify gate with the weakened-checks analysis run on the final tree with its output saved as `verify.txt` in the task's data folder, exact Request-first proposal shape, and post-landing OpenSpec archival by the merge owner). A worker that finds one of them wrong says so in its report.
