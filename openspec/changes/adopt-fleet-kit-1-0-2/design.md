# Design

References: security, design, testing.

## Context

See proposal.md. At launch, the checker matched the invoz import and baseline matched legacy 1.0.0. Local main advanced during preflight; after rebasing onto d8ebd9fa5fe3cf8af0636a7eb2145dc26b9de320, kit 1.0.1 is the prior adopted owner record. Baseline 1.0.2 and the Rust profile now remain unchanged.

## Goals / Non-Goals

Materialize the released owners without creating a parser variant, wrapper or policy service. Product behavior, existing historical decisions and frozen diagnostics are not migrated.

## Decisions

- Import the checker byte-for-byte and merge the baseline while retaining project context. Reject a local parser fix because Fleet owns these rules.
- Keep stock spec-driven with skip_specs: true rather than invent a product capability for repository tooling. No existing product requirement changes.
- Use independent synthetic request fixtures through the actual Python CLI, not copied parser logic. Reject private-function assertions because they would not prove entry-point behavior.
- Kept: scripts/check-change-shape.py, because this byte-exact Fleet consumer artifact intentionally retains upstream formatting and rule custody; local cleanup would break release identity.

## Risks / Trade-offs

Shape checks are not authentication or approval. Only authorized reviewers can approve; pending remains pending. Local operators supply readable files; the checker reads them and prints diagnostics without network, credential access or product mutation. Tracing credentials stay in the existing secret store and are read only in memory.

## Migration Plan

Run fresh CLI regressions against the original checker before adoption, then the released checker; retain both receipts separately from frozen/paired experiments. No refactor is needed: byte identity is the contract. Rollback reverts this bounded adoption commit, not historical evidence.

Record source hashes, ancestry, trace readback, loaded Code version, validation results and exceptions in evidence.md. No unresolved consequential choice needs a new decision record; this applies the specifically authorized released baseline.
