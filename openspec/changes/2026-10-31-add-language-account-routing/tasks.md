# Tasks

## 1. Contracts and regression evidence

- [x] 1.1 Load the code skill: read SKILL.md and design references security/data; follow them throughout. Installed version is 5.1.0, not the requested 5.0.0.
- [x] 1.2 Reproduce absent routing through POST /v1/thoughts with a fake provider and temporary store; capture the missing-route assertion in red.txt.
- [x] 1.3 Write selected spec 038 behavior tests before implementation, then observe semantic failures against temporary stubs (red-unit.txt and red-spanish.txt).

## 2. Rust rule owner and intake integration

- [x] 2.1 Implement whole-word language evidence and founder destinations; verify English lookalikes, accent-less Spanish, accents, frequency and ties with routing tests.
- [x] 2.2 Implement thesis, abbreviation handling, label stripping, portfolio mentions and shared word-boundary budgets; verify routing tests including empty and Unicode boundaries.
- [x] 2.3 Attach optional source contract to new pieces; verify HTTP create/read round-trip independent of provider language, and older store records without routing.
- [x] 2.4 Document metadata semantics in README and mark Slice 6 migrated; verify linked paths and change shape checker.
- [x] 2.5 Behavior-preserving refactor: share word-prefix logic between thesis and hook; focused regression suite remains green.

## 3. Integration checks

- [x] 3.1 Run cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test through nix, using an external target directory; confirm status has no ignored artifacts.
- [x] 3.2 Run strict OpenSpec validation, change-shape and research-record checkers; save the Code verify gate output to the task evidence directory.
- [x] 3.3 Confirm main remains the base, commit the local branch without push, and verify git status --porcelain --ignored is empty.
