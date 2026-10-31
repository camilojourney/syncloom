# Tasks

## 1. Source review and adoption

- [x] 1.1 Load the code skill: read SKILL.md and the design reference (security); inventory edited configuration with findings.py.
- [x] 1.2 Review immutable kit ledger and source ancestry; realize Rust 1.99.1 and reproduce Cargo's virtual-root lints rejection before choosing the approved mapping.
- [x] 1.3 Verify source hashes, merge baseline and Rust sections, and preserve project context and lockfile; inspect diff and Cargo metadata.
- [x] 1.4 Record identities, selected files, project hashes, omissions and actual outcomes in docs/development.md; compare hashes against the final tree.

## 2. Integrated acceptance

- [x] 2.1 Run cargo fmt --check, all-target workspace Clippy with warnings denied, and full workspace tests with Rust 1.99.1; fix any surfaced violations without weakening checks.
- [x] 2.2 Validate all OpenSpec artifacts strictly and run change-shape and research-record checkers.
- [x] 2.3 Run installed Code Verify with explicit project checks and save verify.txt; review every finding and document results.
- [x] 2.4 Review final diff, commit on the assigned branch, and confirm clean git status including ignored files and fast-forward compatibility with main.

No application regression is introduced: this is configuration adoption. Acceptance is source identity, Cargo realization and full existing suites; no new behavioral test or refactor is needed.

## Workflow follow-up

- Firstmate owns final approval, local merge and post-landing archival.
