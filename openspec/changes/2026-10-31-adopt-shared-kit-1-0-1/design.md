# Design

References: security.

## Context

See proposal.md. Kit 1.0.0 is already adopted at c8718007ab87d6062c70cfa7bf301bb9accc9b83; compare its actual project bytes and recorded source identities before upgrading.

## Decisions

- Use immutable release f63507c26ebf9510bec1e92f692919ec62f12270 and its ledger rather than the dispatch's conflicting checker hash. Firstmate steering message 001 authorizes the release's blob 0420aa817549db1462386794a1a5c39bc402dce5 and SHA256 53b3ba7c7c78f6e9ef60f51b24a9d59cb9381fdea01fc13ee8cb7905ffc8366b.
- Merge only shared baseline guidance and replace the checker with exact Fleet bytes rather than maintaining local policy variants. Kept: openspec/config.yaml and scripts/check-change-shape.py, because independently usable repositories materialize shared inputs while Fleet remains their source owner.
- Retain Rust profile 1.0.0 and its existing selected sections, omissions and project-owned lint expectations rather than re-adopting unchanged runtime configuration. No product behavior or capability changes justify delta specs.
- Use the installed Code 5.2.1 Verify owner rather than inventing a project scripts/verify.py wrapper. Store all receipts outside the worktree in this task's authorized evidence directory.

## Risks / Trade-offs

Shape validation accepts attributed approval text but does not authenticate its author. Only an authorized reviewer may approve; this worker leaves Approval pending. The checker reads local record files and prints structural diagnostics; no secret storage, product authorization or network boundary changes.

## Verification and migration

Reproduce the old checker's rejection of attributed approval and capability replacement through its public CLI using scratch proposals, then run the same cases against the exact Fleet artifact. Include pending, independent token transitions and malformed approval controls; preserve request-source binding. No local checker refactor is appropriate for a materialized artifact.

Verify hashes, common baseline equality, retained context and unchanged Rust configuration. Run full formatting, all-target Clippy, tests, strict OpenSpec validation, record checks and Code Verify. Revert this adoption commit to roll back project inputs; no global activation occurs.
