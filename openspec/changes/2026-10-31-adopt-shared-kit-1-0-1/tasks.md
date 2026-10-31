# Tasks

## 1. Shared input adoption

- [x] 1.1 Load the code skill: read SKILL.md and security reference; follow their rules throughout and inventory edited files.
- [x] 1.2 Reproduce reviewer-token rejection using the old checker CLI and request-source binding; retain red receipts outside the repository.
- [x] 1.3 Materialize baseline 1.0.2 and Fleet checker 1.0.0; verify release hashes, unchanged project context and passing token-transition and malformed-approval controls. No local checker refactor.
- [x] 1.4 Update docs/development.md with source custody, component and project hashes, retained Rust profile, exceptions and observed verification receipts.

## 2. Integrated verification

- [x] 2.1 Run cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test under the pinned Rust profile; retain full output.
- [x] 2.2 Run strict OpenSpec validation, shape and research checks, checker unit tests and Code 5.2.1 Verify; save verify.txt and review advisories.
- [x] 2.3 Review the final diff, rebase onto current local main if necessary, commit on the delivery branch and verify git status --porcelain --ignored is empty.

## Workflow follow-up

- Firstmate reviews the ready branch and owns local merge and later archival. No push or remote review is authorized.
