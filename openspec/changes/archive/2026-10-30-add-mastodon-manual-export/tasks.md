# Tasks

## 1. Evidence and test-first boundaries

- [x] 1.1 Load the code skill: read SKILL.md and security, data, integrations and llm references, then follow them throughout.
- [x] 1.2 Save failing CLI/API scenarios for missing Mastodon and export operations, then failing boundary tests for consent, limits, media, repeat safety and export guards before implementation.

## 2. Implementation

- [x] 2.1 Extract shared OAuth primitives without changing LinkedIn behavior; add per-instance registration, bounded consent completion and Keychain-only non-expiring tokens.
- [x] 2.2 Add Mastodon channel and adapter with instance limits, PNG alt text, bounded async media, truthful status outcomes and durable concurrent attempt guard.
- [x] 2.3 Add confined, repeat-safe manual bundles and export receipts, shared review/asset guards and Reddit/TikTok notes.
- [x] 2.4 Expose CLI and Host-guarded API operations and prove fake-only end-to-end thought, render, approve, publish/export and repeat/restart.

## 3. Final gates

- [x] 3.1 Break each new security/idempotency guard, observe a failing test, restore it and save the mutation outcomes.
- [x] 3.2 Update README, development, migration and decision/research records; run fmt, clippy, tests, specs, records, links and gitleaks git, recording every result including failed attempts.
- [x] 3.3 Rebase onto then-current local main, rerun every gate and Code Verify against the pre-change base, saving verify.txt; commit the clean local branch without pushing.
