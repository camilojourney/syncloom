# Q1: X, Threads and Bluesky official publishing APIs

Status: open

## Question

Which current official APIs let a solo founder publish text and images from a local Rust studio to their own X, Threads and Bluesky accounts? Prices, review, authentication, lifetimes, media, policy and Rust SDKs. The exact question is [q1-x-threads-bluesky.md](../2026-10-30-social-api-research-questions/questions/q1-x-threads-bluesky.md).

## Why asked

Vision 3.2 requires deep research before building each integration; vision 6.2 item 5 follows the already-built LinkedIn and Mastodon adapters.

## What would change the answer

Changes to X pay-per-use prices or loopback support, Meta tester access or redirect restrictions, or Bluesky app-password/session support. Each relied-on claim needs a primary-source check before implementation.

## How collected

Phrona 5.0.12 study `main969-q1-xtb`, Claude Research only, collected natively with 124 citations. ChatGPT was never sent because of uncertain composer custody. This is a single-provider study with reduced confidence, not independently corroborated evidence. The operator summary's final claim of two reports contradicts its provenance; it is preserved word for word, not endorsed. The result record is complete but not default-eligible. Trace receipt reported by the study: `61679c020fa532576265b164756c1f82`.

## Status

Open: slice 5b records its choices and primary-source verification in decision 0005. Unresolved: real X loopback registration, Threads HTTPS non-localhost redirect host, public image hosting, and policy clearance for unattended automation. No real sign-in or post is authorized.

## Decision

[0005 X/Threads/Bluesky publishing](../../decisions/0006-social-platform-integrations.md), including separate primary-source verification notes.

## Files

Copies only, never edited. Origins are the study operator's exact files.

| File | Origin | sha256 |
|---|---|---|
| `q1-report.md` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q1-report.md` | `e0657cff41608d5c52646e452a3d85bb9093f6be247438cd8bc5a1616df1bc3a` |
| `q1-claude-NATIVE-report.txt` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q1-claude-NATIVE-report.txt` | `734b594c25035a5034ece477ae387a60e80443a8c031c1523af9159e3cce8c17` |
| `q1-post.json` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q1-post.json` | `0366527f05e2cfbb345f6383287ef20078c89f4462c02b4c0dd1add40ca12e99` |
| `question-hashes.txt` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/question-hashes.txt` | `5655b2123c627bc622938ebbeed450e7c46cf6490ffb7bf18c56f3d3f4bb265d` |
