# Development

## Shared Kit adoption

Kit **1.0.1** is adopted from Fleet release commit `f63507c26ebf9510bec1e92f692919ec62f12270`, following `system/shared/kit/README.md` and its release ledger. Pinned in: OpenSpec change `2026-10-31-adopt-shared-kit-1-0-1` and the project commit containing this record. Previous adoption: Kit 1.0.0 at project commit `c8718007ab87d6062c70cfa7bf301bb9accc9b83`, Fleet release `e9ce8fecb722848654abb6f887f50939b7e5bdc1`.

Components: OpenSpec baseline **1.0.2**, change-shape checker **1.0.0**, Rust profile **1.0.0** retained (Rust **1.99.1**), Code **5.2.1** and quality configuration **code-5.2.1** tested-with. Decide **1.1.0** is the release reference, not invoked here. Global inputs are not project enforcement pins or installation authorization.

### Immutable source identities

Baseline and checker source commit: `9df2415d95cf3e753a268734bed098f24ae027b9`, verified as an ancestor of the release. Baseline blob: `c44e0ffa2d9facdfff58f91bb04cb8d6cb609622`; checker blob: `0420aa817549db1462386794a1a5c39bc402dce5`. Firstmate steering 001 corrected the dispatch's conflicting checker identity to the immutable release ledger.

Paths below are relative to Fleet; hashes are SHA256 unless labeled Git tree.

| Source | Hash |
| --- | --- |
| `system/shared/openspec/baseline.yaml` | `c3cae84909ed326a5bed597a8370ca87ffa0e83e189e21d9ab95ecaa189f4b2b` |
| `system/shared/scripts/check-change-shape.py` | `53b3ba7c7c78f6e9ef60f51b24a9d59cb9381fdea01fc13ee8cb7905ffc8366b` |
| `system/shared/rust/Cargo.toml` | `79fd95c4616b532ba7e1751b090f5d7ff030507d85512ecdc61f27fe0ac0b495` |
| `system/shared/rust/rust-toolchain.toml` | `f715f440167e20fd8ea9afa0498c3d1ab80174cfcb8cbc895f106a34d907a1e1` |
| `system/shared/rust/clippy.toml` | `741d8b6cbd57d7b01ff8918083bbf313e281cce0dbdcb1ed669dc8cb84f44578` |
| `system/shared/rust/.config/nextest.toml` | `7e08fd48e32d6224af50bc2cd5dd205d87f22bbd8c9499dcd836937c0a7148f0` |
| `system/skills/platform/code` | Git tree `62cd39633ecea96fe97bc202458c2aaf160fda89` |
| `system/skills/platform/code/scripts/verify.py` | `8d37ad455935c3e60cef63d3099ea127e03bc7fcb2da3e9e4c82e191829750bf` |
| `system/skills/platform/code/scripts/findings.py` | `472c9eccb3e0fdaddbaf6d369ab4e3dd3cdca1474fe280286f0005328ba13944` |
| `system/skills/platform/code/scripts/weakened_checks.py` | `faf6c56d1e8b5437d46bc6d432824e555e2c19a62ee880e55d812ef769638e45` |

Rust and unchanged findings/weakened-checks sources remain at `840493182630729d32e5ee364f7037f01d47e1ed`; Code/Verify source is `db815962b0364ba22fcb5b2c2713953f6f31ff6d`. Both are ancestors of the selected release. Other quality configuration and delegated owner identities remain the exact unchanged entries in that release ledger; they are references, not copied project configuration.

### Reviewed project configuration

| Project file | SHA256 |
| --- | --- |
| `openspec/config.yaml` | `0b434cfa8f1553a31a0eabece3a9525ba7519bd5506c9292fa48d6b21e9e50e4` |
| `Cargo.toml` | `c1fcf57b361df273a6209225523c3f2332aeab3ada44f7bc339619900cb3e6d6` |
| `rust-toolchain.toml` | `f715f440167e20fd8ea9afa0498c3d1ab80174cfcb8cbc895f106a34d907a1e1` |
| `clippy.toml` | `741d8b6cbd57d7b01ff8918083bbf313e281cce0dbdcb1ed669dc8cb84f44578` |
| `.config/nextest.toml` | `7e08fd48e32d6224af50bc2cd5dd205d87f22bbd8c9499dcd836937c0a7148f0` |
| `crates/syncloom-core/Cargo.toml` | `92d10b04838bb6ed983062aff2edf104392d0c404564f88d7fe748433a1d6de1` |
| `crates/syncloom-server/Cargo.toml` | `01dd028586c50aacf525ebeecf55170dce6325077fb5e08e8b3841052bc214ce` |
| `crates/syncloom-cli/Cargo.toml` | `215b8216aee9061bd7d792166d045eb4ba87b785d6e77094b92f5dfdae7bf347` |
| `scripts/check-change-shape.py` | `53b3ba7c7c78f6e9ef60f51b24a9d59cb9381fdea01fc13ee8cb7905ffc8366b` |

Common baseline lines match the source exactly; every pre-existing Syncloom context line is preserved. Previous baseline 1.0.1 source SHA256 was `fb3e135d4c29607dfd9adbf4b96f1554206f90cfa134d93900932939b2a4498a`. Previous checker SHA256 was `54f77643ee4e4cdc3cfc9f66b20cda5413efc9dfd95982532d58e62fd14b74f2`. All retained Rust configuration hashes match the previous committed record.

Rust lints and release sections match Fleet. Project metadata, dependencies and Cargo.lock remain project-owned and unchanged. Every member inherits workspace lints and MSRV. Package-level root `[lints]` is N/A: Cargo rejects it in this virtual workspace; the prior Kit 1.0.0 adoption's steering message 001 approved the correct root `workspace.lints` / member `lints.workspace` mapping. Nix files are not materialized in the project; optional deny/mutants reports are not selected. Nextest timeout configuration is adopted, but nextest execution is not claimed.

Configuration exceptions: none for selected shared sections. The prior Kit 1.0.0 adoption's firstmate message 002 authorizes local function-scoped `#[expect]` annotations for verified invariants, fail-closed mutex poisoning, intentional CLI output and fail-fast test fixtures. Each carries its reason in source; these are explicit project lint exceptions owned by the Syncloom maintainer, not global profile weakening. Existing shared-fixture dead-code allowances gain reasons without broadening their scope. Runtime behavior, security guards and test assertions remain unchanged.

No new exceptions; configuration exceptions: none for this adoption. The historical project-owned lint expectations above remain unchanged.

### Actual loaded inputs and verification

Loaded `/Users/camiloslaptop/.agents/skills/code/SKILL.md` version 5.2.1, SHA256 `efaa1df842147b98f515fa09615f9d7cb213d6ce6b295f41ccd6f625258dcc3a`; loaded Verify matches the source hash above. Loaded topic reference: security, SHA256 `12ff1d9cc04eee151fe92502402f1df16b31e510249a26d9e9b29ce86cfb8e37`. No product security boundary changes. Decide was not invoked.

Receipts: `/Users/camiloslaptop/.firstmate-homes/syncloom-v2-lab/data/syncloom-v2-adopt-shared-kit-1-0-1/`.

| Problem (source) | Owner | Change | Regression or acceptance | Result |
| --- | --- | --- | --- | --- |
| Reviewer-token replacement rejected by old checker | Fleet | Exact baseline/checker adoption together | Public CLI with request-source binding: pending, independent reviewer/capability transitions, both transitions, malformed approval and altered request | Old checker failed three transition cases (`reviewer-red.txt`); adopted checker passes all seven (`reviewer-green.txt`) |
| Unrecorded shared ownership transition | Fleet / Syncloom | This record and Fleet custody | Source SHA256/blob identities, baseline/common-context comparison and retained project hashes | Pass; no project Rust changes |
| Integrated product checks | Syncloom | Unchanged Rust profile | `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` under locked Fleet Nix profile | Pass (`workspace.txt`), no ignored tests |
| Specification and record integrity | Syncloom | Tooling-only OpenSpec change | `openspec validate --all --strict --no-interactive`; shape checker; research checker and its unit suite | Pass: 20 items, zero shape/research problems, 25 tests |
| Final quality | Code / Syncloom | Installed Code 5.2.1 Verify | `python3 /Users/camiloslaptop/.agents/skills/code/scripts/verify.py --base main --change-kind refactor` with explicit project checks | ACCEPT; explicit full Rust and record checks pass, strict specs pass, zero blocking findings, zero weakened-check candidates, links and secrets pass. Receipt `verify.txt`; native auto-lint is N/A because Rust inputs are unchanged, but the explicit check covers all targets. Vale is N/A (no project .vale.ini). |

No tests were skipped, loosened or re-baselined. No local checker refactor: Fleet is its source owner. Nextest, optional Rust reports and non-Darwin platforms remain unmeasured; materialized configuration does not imply those checks ran.

## OpenSpec configuration

OpenSpec CLI 1.15.0 uses the stock `spec-driven` schema. Existing generated skills and commands under `.agents/`, `.claude/` and `.pi/` were initialized with `openspec init --tools codex,claude,pi` and remain unmodified. Re-check baseline updates with `git -C <fleet-system> show 9df2415d95cf3e753a268734bed098f24ae027b9:system/shared/openspec/baseline.yaml`, compare SHA256, then review only common guidance while preserving project context. Run `openspec validate --all --strict --no-interactive`.

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

`scripts/check-change-shape.py` is Fleet-owned from Shared Kit 1.0.1, checker 1.0.0. Its immutable provenance is invoz.v2 `ddd9880ef316d7cac7b4d87be84e480fd148e237:scripts/check-change-shape.py` (original SHA256 `54f77643ee4e4cdc3cfc9f66b20cda5413efc9dfd95982532d58e62fd14b74f2`). Update from Fleet's release ledger, not from another project's materialized copy. Shape validation does not authenticate a reviewer or grant approval.

`scripts/check-research-records.py` and `scripts/test_check_research_records.py` remain invoz.v2-owned byte-identical copies (SHA256 `20503765e214235c8e002007e7bad7501f2b2ec8d9081741a6a550a5f34ebae1` and `efcefaf2ceb7d4faf89fb3a26cbb36a5557e89ab25171d0d9d0159f5a6a5f853`). Copy their updates from that owner and rerun tests. Repository-record tooling stays Python; porting it would create a second policy owner. All checkers are stdlib-only; run with `python3 -B -I` to avoid repository bytecode.

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

Every build task applies `data/v2-build-lessons-20261030.md` of the firstmate home (27 items: OpenSpec 1.15.0 with this config, the installed Code skill loaded first (5.2.1 for Kit 1.0.1 adoption), test first, verbatim requests, research saved word for word, Receipts-first reports, no secrets or personal data in requests or files, `gitleaks git` mode, loopback-only servers with Host check and a fixed error envelope, hermetic tests, scratch under `/tmp/fm-<task>/`, Rust via nix, Rust for first-party code, rebase onto the current local `main` and re-run the gates before done, and the Code skill's Verify gate with the weakened-checks analysis run on the final tree with its output saved as `verify.txt` in the task's data folder, exact Request-first proposal shape, and post-landing OpenSpec archival by the merge owner). A worker that finds one of them wrong says so in its report.
