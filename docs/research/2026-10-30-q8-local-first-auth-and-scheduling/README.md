# Q8: local-first credentials, OAuth and scheduling for one person on macOS

Status: open

## Question

How a local-first Rust app for one person should hold social-platform credentials (macOS Keychain, encrypted file, or environment), run OAuth without a hosted server (loopback redirect with PKCE, device code, or manual paste), and schedule publishing when the laptop may be asleep or off (native platform scheduling, a local launchd agent, or post at approval time), including token rotation, launchd behavior after sleep, maintained Rust crates and idempotency. The question as sent is [`questions/q8-local-first-auth-and-scheduling.md`](../2026-10-30-social-api-research-questions/questions/q8-local-first-auth-and-scheduling.md) (sha256 `3bd00183...`, verified unchanged by the operator before the study started).

## Why asked

Slice 2 dispatch and the credentials design before any live post (vision 6.2): where the LinkedIn token lives, how the founder signs in from a laptop with no server, and whether scheduling belongs in the platform or in the app. The scheduling part serves a later slice; it is recorded here and not built in slice 2.

## What would change the answer

Neither condition named in the question holds: only two of the eight surfaces (YouTube, Facebook Pages) schedule natively, and no provider was found that forbids loopback redirects (Meta needs HTTPS outside development mode, the one real obstacle). What would move the answer: a platform adding native scheduling or an idempotency key; Apple changing legacy Keychain access-control behavior for unsigned binaries; the `keyring-core` or `apple-native-keyring-store` crates losing maintenance.

## How collected

Phrona 5.0.12 study `main969-q8-auth-r2`, run by the phrona-lab home on 2026-10-30 as the approved re-run of `main969-q8-auth` (whose Claude composer held a draft leaked from Q7, so nothing was sent in the original). Claude Research as the primary, collected natively with 82 citations after one `research-not-finished`. The ChatGPT Deep research challenger was never sent: Phrona recorded the ChatGPT composer as custody-uncertain (a concurrent paste by a duplicate queue process), and the operator did not repeat the Send. So this study has one report and no cross-check; Phrona marks its confidence reduced and the study not default-eligible. Langfuse trace `0b32559c901db03e646d7ea60cb51509`, 137 events, known cost USD 0.068. `q8-report.md` is the operator's summary; `q8-post.json` the study's result record.

## Status

Open. Answered for the decision (credentials in the macOS Keychain through `keyring-core` 1.x plus `apple-native-keyring-store` 1.0.2; loopback-redirect OAuth from the app; scheduling local, with native scheduling only where it exists). Single provider, no ChatGPT cross-check. Not researched to primary level, per the report: LinkedIn native-PKCE enablement, Power Nap and IOPM wake events, LaunchAgent login-session scope, exact `reqwest`, `atrium-api` and `bsky-sdk` versions, Bluesky app-password refresh, Instagram and Threads loopback rules. The Keychain access-control claim (unsigned or re-signed builds can trigger prompts) is stated as platform behavior without a primary citation.

## Decision

[0002 LinkedIn personal-profile integration: self-serve Share on LinkedIn, Keychain tokens, loopback sign-in](../../decisions/0002-linkedin-integration-decisions.md) (credentials and sign-in; scheduling waits for its own slice).

## Files

Verbatim copies; the repository never edits them. Origins are paths on the machine that ran the study.

| File | Origin | sha256 |
|---|---|---|
| `q8-report.md` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q8-report.md` | `5cfc10d3953f76fd2c91b01a33a91765d7d90ae6979c298c41f977f0d985f6b2` |
| `q8-claude-NATIVE-report.txt` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q8-claude-NATIVE-report.txt` | `dc9556bab04c41bc7dc8f88a31efe29f81fc95151cf9fc27047c3509f31b69f2` |
| `q8-post.json` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q8-post.json` | `7ac6c36fcb1c6eca49af14274b12098cee4355f7458bb85c6641ce195194e5fe` |
