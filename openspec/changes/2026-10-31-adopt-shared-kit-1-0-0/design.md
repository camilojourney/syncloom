# Design

References: security.

## Context

See proposal.md for motivation. The repository has a virtual Cargo workspace, three packages and a project-specific OpenSpec context block.

## Decisions

- Merge immutable Fleet sections rather than replace project manifests; this preserves dependencies, metadata and Cargo.lock. Verify source identities against the kit ledger before materialization.
- Keep workspace lint definitions at the virtual root and explicit inheritance in each package. Root package-level lints are N/A: Cargo 1.99.1 rejects them in virtual manifests; firstmate approved this mapping in steering message 001.
- Retain stock generated adapters unchanged rather than regenerate identical OpenSpec 1.15.0 adapters. No public product behavior changes, so skip_specs is explicit.
- Select toolchain, lint/release sections, Clippy and nextest configuration. Nix environment files are not adopted: realization uses the immutable profile in task-local scratch; optional deny/mutants reports are not selected.
- Treat global Code, quality and Decide identities as tested-with inputs, not project enforcement pins. Record actual loaded identities separately from release references.
- Use docs/development.md as the only adoption record rather than introduce another registry. Compare merged project hashes separately from immutable component hashes.

Kept: crates/syncloom-core/examples/fake_claude.rs and crates/syncloom-core/examples/fake_linkedin.rs, because each independently owns a different fake protocol's CLI and lifecycle; shared Clap/bootstrap syntax does not justify coupling their command owners.

## Risks / Trade-offs

- Native Clippy reproduced 36 existing core production invariant unwrap/expect violations. Firstmate message 002 authorizes function-scoped lint expectations for verified invariants and unrecoverable poisoning; these preserve existing panic behavior rather than introduce silent recovery. No security boundary, permission, log or secret storage behavior changes; existing loopback guards and Keychain owners remain unchanged. Reviewed all 176 weakening candidates from Verify: they are these authorized local expectations or existing test-fixture dead-code allowances gaining reasons, not deleted assertions, skipped tests or workspace policy weakening.
- Arbitrary Nix Cargo ignores rust-toolchain.toml; realize the selected 1.99.1 profile before checks and record actual compiler versions.
- Nextest configuration does not prove nextest execution; report only checks actually run.

## Migration Plan

Apply the reviewed source diff and run workspace and OpenSpec checks plus Code Verify. Revert this single adoption commit to roll back materialized configuration; global skills are untouched.
