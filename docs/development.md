# Development

## OpenSpec configuration

`openspec/config.yaml` adopts Fleet kit **1.0.2**, baseline **1.0.2**, from immutable release `d861f11cd52934a149077ded482c928cd5203472`. Baseline source: `9df2415d95cf3e753a268734bed098f24ae027b9:system/shared/openspec/baseline.yaml`, blob `c44e0ffa2d9facdfff58f91bb04cb8d6cb609622`, SHA256 `c3cae84909ed326a5bed597a8370ca87ffa0e83e189e21d9ab95ecaa189f4b2b`. Every shared line is byte-identical; only Syncloom context lines appended to `context:` differ. [Adoption evidence](../openspec/changes/adopt-fleet-kit-1-0-2/evidence.md) records the prior baseline, ancestry, exceptions and actual loaded Code version.

Re-check: save the baseline with `git -C <fleet-system> show 9df2415d95cf3e753a268734bed098f24ae027b9:system/shared/openspec/baseline.yaml > /tmp/base.yaml`, then `diff /tmp/base.yaml openspec/config.yaml`. Only added context lines may appear. On a baseline update, carry the common diff by hand and run `openspec validate --all --strict --no-interactive`.

OpenSpec CLI 1.15.0, initialized with `openspec init --tools codex,claude,pi` (stock `spec-driven` schema; the generated skills and commands under `.agents/`, `.claude/` and `.pi/` are not hand-edited).

## Toolchain

Rust, pinned in `rust-toolchain.toml` (1.98.1). `cargo` is not on PATH on this machine; run every check through nix:

```sh
nix shell nixpkgs#cargo nixpkgs#rustc nixpkgs#clippy nixpkgs#rustfmt -c sh -c 'cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test'
```

## Visual renderer tests

The [visual asset guide](visual-assets.md) pins the installed browser and font and describes the snapshot tolerance. Tests launch that existing binary; they never install or download one. Use `CARGO_TARGET_DIR=/tmp/fm-<task-id>/target` and `TMPDIR=/tmp/fm-<task-id>/` for build/test scratch. `nix --offline shell` avoids refreshing the toolchain's flake metadata during hermetic validation. Chromium's product version and the font checksum are checked by the renderer before producing assets.

## Record checkers

`scripts/check-change-shape.py` is the byte-exact Fleet checker **1.0.1** from kit **1.0.2**, source `72ae87b8fc6feeb32996618eb5a228b719924319:system/shared/scripts/check-change-shape.py`, blob `366f60d4dc377bfd32bc2ec293516b26ce4aa44b`, SHA256 `c707e00629112d75354791b06734fc0ca1adae1845772a11c08f10a837a917bf`. Fleet owns this checker from kit 1.0.1 onward; adopt future releases in recorded project changes, never patch the consumer independently. It binds multiline quotes to exact request paragraphs and accepts explicit attributed reviewer tokens without authenticating or granting approval. Run `python3 -B -I scripts/check-change-shape.py openspec/changes/<change> --request-source <request.txt>`; omitting the source cannot prove verbatim intent. Consumer CLI regressions: `python3 -B -I -m unittest discover -s scripts -p 'test_check_change_shape.py' -v`.

`scripts/check-research-records.py` and `scripts/test_check_research_records.py` retain their invoz.v2 ownership and byte-exact SHA256 identities `20503765e214235c8e002007e7bad7501f2b2ec8d9081741a6a550a5f34ebae1` and `efcefaf2ceb7d4faf89fb3a26cbb36a5557e89ab25171d0d9d0159f5a6a5f853`. They are not kit components and are not repinned here. These repository tools stay standard-library Python: the Rust rule covers product code, and porting the checkers would create another rule owner. They run with `python3 -B -I`; `-B` prevents bytecode in the repository.

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

Every build task applies `data/v2-build-lessons-20261030.md` of the firstmate home (27 items: OpenSpec 1.15.0 with this config, installed Code loaded first (actual version recorded per change), test first, verbatim requests, research saved word for word, Receipts-first reports, no secrets or personal data in requests or files, `gitleaks git` mode, loopback-only servers with Host check and a fixed error envelope, hermetic tests, scratch under `/tmp/fm-<task>/`, Rust via nix, Rust for first-party code, rebase onto the current local `main` and re-run the gates before done, and the Code skill's Verify gate with the weakened-checks analysis run on the final tree with its output saved as `verify.txt` in the task's data folder, exact Request-first proposal shape, and post-landing OpenSpec archival by the merge owner). A worker that finds one of them wrong says so in its report.
