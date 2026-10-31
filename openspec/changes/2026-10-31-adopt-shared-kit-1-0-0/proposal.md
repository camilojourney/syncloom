## Request

> the c configuraion and the tools we use there like the quality and owther should be versioned. so we can use tthem and improve all the other projects at the same time

## Approval

pending

New capability requested: none.
Duplicate kept: none.

## Why

The current baseline pins stale Code guidance and has no combined kit adoption record. An immutable source identity makes future cross-project improvements reviewable.

## What Changes

- Adopt Shared Kit 1.0.0, OpenSpec baseline 1.0.1 and selected Rust profile 1.0.0 configuration.
- Retain Syncloom context, dependencies, lockfile and stock generated adapters.
- Record source hashes, merged project hashes, global tested-with identities and actual verification in docs/development.md.
- Add firstmate-authorized, narrowly justified lint expectations for existing invariants, CLI output and fail-fast test fixtures; preserve runtime behavior.

## Capabilities

None. Reviewed the OpenSpec capability inventory: existing specifications concern product behavior, not repository tooling. No product contract changes; skip_specs is explicit.

## Impact

Workspace and crate manifests, toolchain, Clippy and nextest configuration, OpenSpec guidance and development documentation. No global skill activation, remote publication or infrastructure administration.

## Open questions

None.
