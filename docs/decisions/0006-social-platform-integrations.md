# ADR-0006: Bluesky, X and Threads publishing

**Date:** 2026-10-31
**Status:** Proposed
**Supersedes:** none
**OpenSpec changes:** [add-bluesky-x-threads](../../openspec/changes/archive/2026-10-30-add-bluesky-x-threads/)
**Agreed:** no - implementation brief authorizes building; artifact review remains pending
**Recommended:** ATrium typed Bluesky APIs, X v2 with explicit cost confirmation, Threads text only
**Built:** yes - implemented and independently fake-proven on the rebased tree; final committed-tree receipt is recorded in the task report
**Live-proven:** no - real sign-in, posting and paid calls are not authorized

## Context

Q1 arrived after the first two adapters. Its single-provider provenance limits confidence; endpoint and pricing claims require direct checks.

## Decision

Use the [integration delta specs](../../openspec/changes/archive/2026-10-30-add-bluesky-x-threads/specs/) with existing outbox, credential and channel owners. Threads image delivery and redirect hosting are not implemented.

## Rationale

| Option | Pros | Cons | Why rejected / chosen |
|---|---|---|---|
| ATrium API 0.26.0 typed client | Maintained, typed lexicons, configurable HTTP | Explicit session orchestration | Chosen: avoids automatic retry and arbitrary DID endpoint changes |
| BskyAgent 0.2.0 | Convenient login/refresh | Ignores some store failures, follows DID endpoints, retries expired calls | Rejected for this narrow boundary |
| Handwritten X and Threads reqwest | Small verified surfaces, common transport policy | Wire contracts maintained here | Chosen over extra third-party auth frameworks |
| Hosted Threads image/redirect service | Automatic media and callback | Hosting choice and disclosure not authorized | Rejected for this slice; text and explicit callback completion only |

## Scope and boundaries

Text and one approved PNG for Bluesky/X; Threads text only. No language routing, engagement, scheduling or analytics. Default grants remain empty. One request that may have created a post is never blindly repeated. No app password, JWT, OAuth code or provider error body enters a file outside the Keychain.

## Identities

OpenSpec CLI 1.15.0, config blob `8f976f578610adeefde25096a98101b9f9c2c12c`; Code 5.0.0 during implementation and 5.1.0 at final review (security, data, integrations, ui). Initial base `8a3e01b788d52e18369d637372cbb5b9d2966632`; integration base `1fc1785` includes parallel Meta slice 5c. Build commit: implementation checkpoint on `fm/syncloom-v2-slice5b-bluesky-x-threads-20261031`; final head and gates are recorded in the task receipt.

## Evidence and gaps

[Q1 verbatim study and provenance](../research/2026-10-30-q1-x-threads-bluesky/README.md), imported at `161c4d1` after rebase (original checkpoint `0372b5c`), before specification. Claude-only, reduced confidence; not independently corroborated by a challenger.

### Primary-source checks, 2026-10-31 UTC

These are implementation notes, not edits to the imported research. Public docs were fetched without sign-in, personal headers or payloads.

| Official source | Observed claim and consequence |
|---|---|
| [X pricing](https://docs.x.com/x-api/getting-started/pricing) | Post Create USD 0.015; with URL USD 0.200; Media Metadata USD 0.005. Upload has no separate listed charge. Prices subject to change; estimate is not actual billing. |
| [X OAuth PKCE](https://docs.x.com/fundamentals/authentication/oauth-2-0/authorization-code) | Access token two hours, offline.access yields refresh token, POST https://api.x.com/2/oauth2/token; public clients include client_id. Read expires_in rather than assuming two hours. Exact registered loopback acceptance is not live-proven. |
| [X initialize](https://docs.x.com/x-api/media/initialize-media-upload), [append](https://docs.x.com/x-api/media/append-media-upload), [finalize](https://docs.x.com/x-api/media/finalize-media-upload), [metadata](https://docs.x.com/x-api/media/create-media-metadata) | Current OpenAPI 2.173: initialize JSON; append `/2/media/upload/{id}/append` with media and segment_index; finalize `/2/media/upload/{id}/finalize`; metadata JSON requires `id` and nested `metadata.alt_text`. Do not use the report's older assumed append/finalize paths or metadata shape. |
| [Meta access tokens](https://developers.facebook.com/documentation/threads/get-started/get-access-tokens-and-permissions) | Authorize https://threads.com/oauth/authorize; POST https://graph.threads.com/oauth/access_token with exact redirect, app id/secret and code; returns access_token and user_id. |
| [Meta long-lived tokens](https://developers.facebook.com/documentation/threads/get-started/long-lived-tokens) | Short token one hour; GET /access_token with th_exchange_token; long token 60 days; GET /refresh_access_token with th_refresh_token only when unexpired and at least 24 hours old. Read returned expires_in. App secrets must not be embedded in distributed devices/binaries. |
| [Official Meta sample](https://github.com/fbsamples/threads_api) | README explicitly says redirects only HTTPS and not localhost; suggests local DNS and TLS. We do not configure DNS, certificates or hosting. |
| [Threads posts](https://developers.facebook.com/documentation/threads/posts) | Testers can publish without advanced access; public image/video server required. POST /v1.0/{user}/threads and /threads_publish; 500 characters with emoji UTF-8 counting. Image capability stays unimplemented. |
| [Threads troubleshooting](https://developers.facebook.com/documentation/threads/troubleshooting), [retrieve](https://developers.facebook.com/documentation/threads/retrieve-and-discover-posts/retrieve-posts) | Check container status FINISHED before publication; retrieve id/permalink afterwards. Missing permalink gives unknown, not an invented URL. |
| [AT Protocol createSession lexicon](https://raw.githubusercontent.com/bluesky-social/atproto/main/lexicons/com/atproto/server/createSession.json), [maintained ATrium SDK](https://docs.rs/atrium-api/0.26.0/atrium_api/) and [Bsky SDK](https://docs.rs/bsky-sdk/0.2.0/bsky_sdk/) | App-password login and access/refresh JWTs; typed createSession, refreshSession, uploadBlob, createRecord operations. Installed 0.26.0 source inspected before choosing the SDK. SDK documents grapheme counting; conservative image cap is 1 MB pending a primary-source check of the reported 2 MB increase. |

### Local verification

The rebased implementation passed full Rust tests, formatting, denied-warning clippy, strict OpenSpec, proposal/research checks, offline links and `gitleaks git`. Code Verify on implementation checkpoint `78958c7` returned `ACCEPT_WITH_ADVISORIES`, with zero blocking findings and no static-check waivers. Seven weakened-check candidates mistake Rust's channel `fit()` function for a focused test; two are test-only fixture `dead_code` allowances because each integration test compiles a different subset of shared helpers. No tests are skipped or weakened. Final committed-tree output and the reviewed candidates are retained as task receipts.

Doc retrieval receipts: web fetch ids `mvvq0wz34ucqf0`, `mvvq2v2cjt9hzm`, `mvvq5laplihg58`. Bluesky rendered posting page was not extractable and raw guessed document paths returned 404; no claim of having verified those pages. SDK source and createSession lexicon were available. No real account or payment call occurred.

Open questions: Which Threads HTTPS redirect host and local handoff? Which public-media host for later images? Does X accept the exact registered loopback redirect for this app? When prices change, obtain a fresh primary-source check before real use. All require separate authorization, not a fake test's success.

## Consequences

**Positive:** contained defaults, independent adapters, explicit spend estimates and renewable secrets without plaintext files.

**Negative / Trade-offs:** conservative text and image limits may refuse valid content; Threads needs manual callback transfer until a host is chosen; fake proof does not establish platform registration eligibility.

## References

- [Design](../../openspec/changes/archive/2026-10-30-add-bluesky-x-threads/design.md)
- [Research convention](../research/README.md)

## Dated outcome/linkage review: 2026-11-02

Implementation is present in local history at `bfa582b`: [Bluesky](../../crates/syncloom-core/src/bluesky.rs), [X](../../crates/syncloom-core/src/x.rs), [Threads](../../crates/syncloom-core/src/threads.rs), [fake HTTP journeys](../../crates/syncloom-server/tests/social_delivery.rs). Q1 is Claude-only/reduced, not two-provider agreement. Threads images, redirect host, public hosting and real X registration/prices remain unresolved as originally recorded.

This is source/history inspection, not a fresh platform smoke or newly accepted rationale. Status, agreed boundaries and historical method identity remain unchanged: this record was not made through the current Decide method. [research/0001](research/0001-research-custody-and-retrieval.md) makes only the new reversible custody/retrieval choice and links its executed local evaluation.
