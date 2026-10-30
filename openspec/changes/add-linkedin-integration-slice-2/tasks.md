## 1. Preparation

- [ ] 1.1 Load the code skill: read its SKILL.md (5.0.0) and the security, data and integrations references, run the findings inventory on the files to edit, and follow its rules in every later task.
- [ ] 1.2 Import the Q2, Q8, Q7 and Q3 research word for word with cards, sha256 and INDEX rows; write decision 0002; `python3 -I scripts/check-research-records.py` passes.

## 2. Spec checkpoint

- [ ] 2.1 Write the delta specs (two new, four modified) and the proposal; `openspec validate --all --strict --no-interactive` and `scripts/check-change-shape.py` pass.
- [ ] 2.2 Write the failing tests for every new or changed requirement against the public shape (platform trait, token store, LinkedIn adapter over the fake, dispatch outcomes, API operations, CLI commands); see each fail for the stated reason (an assertion, not a compile error, once stubs exist), save the red output outside the repository and commit the red state before any implementation.

## 3. Implementation

- [ ] 3.1 Core: `Platform` and `TokenStore` traits, the redacted token type, the `BaseUrl` owner, the delivery grant and redirect URL rules, the intent fields and events; the pure-rule tests pass.
- [ ] 3.2 Core: the dispatch state machine over a scripted platform fake (first attempt, outcomes, repeat after posted, retry after a retryable failure, `posting` found, deadline); the studio tests pass.
- [ ] 3.3 Core: the Keychain store against the real Keychain with a test-only service name (save, load, overwrite, delete, absent) and the in-memory store; the credentials tests pass and leave no item behind.
- [ ] 3.4 Core: the LinkedIn adapter against the fake (sign-in consent, denial, wrong state, token refusal, timeout; posting with each status; request shape and headers; nothing sent on an expired or absent token); the adapter tests pass.
- [ ] 3.5 Server: the four LinkedIn operations, publish response fields, seven codes with fixtures, route labels; the API, contract and log tests pass.
- [ ] 3.6 CLI: `linkedin sign-in|status|sign-out` and the publish outcomes over the same core; the CLI tests pass, including sign-in through the real Keychain under a test service that the test deletes.
- [ ] 3.7 End-to-end run as the operator would: fake LinkedIn example, server, CLI sign-in with a browser-like redirect, publish to posted, repeat without a second request, each failure mode; record the transcript outside the repository.

## 4. Verification

- [ ] 4.1 Mutation checks: let a `posted` intent send again, let a stale revision through, let an unapproved piece dispatch, send with an expired token, skip the state check, mark `posted` without the post id, write the token into an event; confirm a named test fails for each, then restore.
- [ ] 4.2 `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`; `git status --porcelain --ignored` identical before and after the test run; `openspec validate --all --strict --no-interactive`; change-shape and research-records checks; `gitleaks git`; Code Verify with the weakened-checks analysis on the final rebased tree.
- [ ] 4.3 Report, by name only, that no LinkedIn developer app, client id or secret exists on this machine; state the exact one-time steps the captain would do; make no real sign-in and no real post.

## 5. Records

- [ ] 5.1 Flip the ledger rows in `docs/migration.md`; update decision 0001 and 0002 (Built), README (commands, environment, going live) and `docs/development.md` if a command changed.
- [ ] 5.2 Archive the change so `openspec/specs/` holds the two new and four modified specs (as slice 1 did, Approval stays pending for the reviewer), point decisions 0001 and 0002 at the archived path, and rerun every gate on the final tree.
