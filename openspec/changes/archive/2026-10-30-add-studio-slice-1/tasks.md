## 1. Preparation

- [x] 1.1 Load the code skill: read its SKILL.md (5.0.0) and the security, data, integrations and llm references, run the findings inventory, and follow its rules in every later task.
- [x] 1.2 Read the old owners at `3efcd35` named in design.md and record each kept or changed behavior there.

## 2. Spec checkpoint

- [x] 2.1 Write the seven delta specs and the proposal; `openspec validate --all --strict --no-interactive` and `scripts/check-change-shape.py` pass.
- [x] 2.2 Write the failing tests for every requirement against the public shape of the three crates (stub bodies), see each fail for the stated reason (an assertion on status, code, body or file state, not a compile error), save the red output outside the repository and commit the red state before any implementation.

## 3. Implementation

- [x] 3.1 Core: platform rules, thought normalization and length rule, revision hash, review transitions, outbox key and request id, error table, provenance events; the pure-rule tests pass.
- [x] 3.2 Core: the file store (lock, atomic write, append, restart, leftover temporary file) and the studio use cases over the fake provider; the store and studio tests pass.
- [x] 3.3 Core: the Claude adapter (request shape, status mapping, base URL rule, key handling) against the fake Messages endpoint; the provider tests pass. Do not call the real endpoint.
- [x] 3.4 Server: bind rule, Host check, no CORS, request id, envelope, strict bodies, routes, panic layer, request log; the API and contract tests pass and the fixtures equal the server.
- [x] 3.5 CLI: the four commands over the same core, same errors; the CLI tests pass, including the shared data directory with the server.
- [x] 3.6 End-to-end run as the operator would: fake Claude example, server, CLI thought, show, approve, publish dry run and contained publish; record the transcript outside the repository.

## 4. Verification

- [x] 4.1 Mutation checks: remove the Host check, let a stale revision pass, let an unapproved piece publish, write the thought text into an event, skip the fsync-rename path, drop the deadline; confirm a named test fails for each, then restore.
- [x] 4.2 `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`; `git status --porcelain --ignored` identical before and after the test run; `openspec validate --all --strict --no-interactive`; change-shape and research-records checks; `gitleaks git`; Code Verify with the weakened-checks analysis.
- [x] 4.3 Criterion 13: report whether an `ANTHROPIC_API_KEY` is available, by name only; do not call the provider. The live draft waits for the captain's word.

## 5. Records and archive

- [x] 5.1 Flip the ledger rows in `docs/migration.md`; update decision 0001 (Built) and README.
- [x] 5.2 Archive the change so `openspec/specs/` holds the seven specs, point decision 0001 at the archived path, and rerun every gate.
