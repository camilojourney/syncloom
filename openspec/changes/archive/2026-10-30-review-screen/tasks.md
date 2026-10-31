# Tasks

## 1. Test-first boundaries

- [x] 1.1 Load the code skill: read SKILL.md and security, data and ui references; follow their rules and run findings inventory before code edits.
- [x] 1.2 Capture the missing screen in the real browser; add HTTP red tests for discovery, forms, exact approval, invalidation, escaping and guards; save the selected failing run before implementation.

## 2. Shared core and protected browser

- [x] 2.1 Implement bounded discovery in Store and Studio, expose GET /v1/pieces, and prove empty, pagination, malformed-record and symlink cases with tests; document the cursor.
- [x] 2.2 Implement shared session protection, exact Origin and fetch-site checks, security headers and API token handshake; adapt existing API tests and contract fixtures, document HTTP client migration, and mutation-test each guard.
- [x] 2.3 Implement Rust page/form adapters using existing core request types, responsive image/PDF previews and truthful publish receipts; prove form edits, approval/hash conflicts, stale revisions, containment and escaping, then extract shared rendering helpers without changing tests; document browser use.

## 3. End-to-end and landing gates

- [x] 3.1 Prove thought-to-publish in chrome-devtools-axi against the running server with only fake providers; save wide/narrow screenshots including exact hashes, PDF, contained receipt and an approved edit that invalidates approval.
- [x] 3.2 Rebase onto then-current local main; run fmt, denied-warning clippy, full tests with hermetic status comparison, strict OpenSpec, change/research/link checks, gitleaks git and final Code Verify including weakened-check analysis; save verify.txt and Receipts-first report, commit and leave a clean branch.
