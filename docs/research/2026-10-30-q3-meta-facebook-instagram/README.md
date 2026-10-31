# Q3: Meta (Facebook Pages and Instagram) publishing from a local Rust app

Status: open

## Question

What Facebook Pages and Instagram allow a single-user app through Meta's APIs for the founder's own accounts: login path and tokens, access level and app review, scheduling, carousels, insights, rate limits, terms and AI disclosure, 2025-2026 changes, test modes and Rust clients. The question as sent is [`questions/q3-meta-facebook-instagram.md`](../2026-10-30-social-api-research-questions/questions/q3-meta-facebook-instagram.md) (sha256 `0376181b...`, verified unchanged by the operator before the study started).

## Why asked

Slice 5 (more channels and their integrations) and the Instagram carousel in slice 3 (vision 6.2). Recorded now, built later; nothing in slice 2 depends on it.

## What would change the answer

Of the question's conditions: owned-account publishing in development mode without review holds (Standard Access suffices for the founder's own accounts); carousels are in the API (the "no carousel" note of March 2026 was wrong); Page tokens derived from a long-lived user token carry no expiry. What would move the answer: Meta requiring App Review or Business Verification for Standard Access on owned accounts; Instagram adding native scheduling; a change to the public-URL requirement for Instagram image containers, which forces a local-first app to expose each image at a short-lived public URL.

## How collected

Phrona 5.0.12 study `main969-q3-meta`, run by the phrona-lab home on 2026-10-30: Claude Research primary (collected natively, 198 citations, after one `research-not-finished`) and ChatGPT Deep research challenger (collected by hand; sources panel "Sources scanned 0", so model output without citations, lesson 18; the operator's summary lists where it contradicts itself). The study ended at its coordinator call limit (`original-limits`), so Phrona marks it incomplete and not default-eligible; both reports are complete documents. Langfuse trace `4498dca58af62f25674d89a658d0faa2`, 217 events, known cost USD 0.108. `q3-report.md` is the operator's summary; `q3-post.json` the result record.

## Status

Answered and used by decision 0005; open: real hosting, app eligibility, redirect acceptance and public visibility. The adapter pins v26.0 from re-opened official docs, not the summary's v25.0 recommendation.

## Decision

[Decision 0003](../../decisions/0003-visual-renderer.md) uses the slice brief's conservative ten-slide cap for local assets, without implementing Meta publishing. [Decision 0005](../../decisions/0005-meta-integration.md) records slice 5c Meta integration choices and live-use gaps.

## Official-doc verification for slice 5c (2026-10-31)

Re-opened Meta's current official pages before specifying the adapter:
- [Content publishing](https://developers.facebook.com/docs/instagram-platform/instagram-api-with-facebook-login/content-publishing/): Facebook Login for Business with a Page token, public image URLs, JPEG only, image alt_text, carousel children followed by a parent and media_publish; FINISHED before publishing, ERROR/EXPIRED fail, expiry after 24 hours. No scheduling parameter is listed. The page contradicts itself on 50/100 daily posts; no hard-coded daily quota is inferred.
- [Page photo reference](https://developers.facebook.com/docs/graph-api/reference/page/photos/): caption, alt_text_custom, published and public url parameters. Confirmed separately before exercising photo delivery.
- [IG media reference](https://developers.facebook.com/docs/instagram-platform/instagram-graph-api/reference/ig-media/): permalink is the permanent media URL; numeric ids are not fabricated into Instagram shortcodes.
- [Login security](https://developers.facebook.com/docs/facebook-login/security/): HTTPS required generally; no loopback exemption established in the inspected page, so real redirect eligibility remains open.
- [Page posts](https://developers.facebook.com/docs/pages-api/posts/): feed message, photos url, photo response post_id distinct from photo id; post permalink is facebook.com/<page_post_id>. Native scheduling exists but is not used.
- [Long-lived tokens](https://developers.facebook.com/docs/facebook-login/guides/access-tokens/get-long-lived/): exchange short-lived user token with fb_exchange_token, then derive Page tokens through accounts; derived Page tokens have no expiration date but can be invalidated.
- [Facebook Login for Business](https://developers.facebook.com/docs/facebook-login/facebook-login-for-business/): user-token configuration, manual dialog with config_id instead of scope; advanced access for businesses not owned/managed.
- [Manual login](https://developers.facebook.com/documentation/facebook-login/guides/advanced/manual-flow): code response, state CSRF protection, matching redirect_uri in exchange; app secret must not be embedded in distributed binaries.
- [Instagram overview](https://developers.facebook.com/documentation/instagram-platform/overview): Standard Access suffices for owned/managed professional accounts. This does not prove a particular app/dashboard setup.
- [App modes](https://developers.facebook.com/docs/development/build-and-test/app-modes): development-mode generated data is visible only to role users.
- [Media creation reference](https://developers.facebook.com/docs/instagram-platform/instagram-graph-api/reference/ig-user/media/): JPEG, 8 MB, 4:5 to 1.91:1; parent-only is_ai_generated.
Current examples and token guide name v26.0. The adapter pins v26.0 rather than copying the study's v25.0 recommendation. Retain the local renderer's conservative ten-slide cap. Hosting, real app eligibility/redirect acceptance and public visibility remain unproven; no sign-in or publishing was performed. No claim relies on the uncited ChatGPT report.

## Files

Verbatim copies; the repository never edits them. Origins are paths on the machine that ran the study.

| File | Origin | sha256 |
|---|---|---|
| `q3-report.md` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q3-report.md` | `61e56fbb559a4ea55a07436e3968066ded6815e16ffbc7a01026df601186cc83` |
| `q3-claude-NATIVE-report.txt` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q3-claude-NATIVE-report.txt` | `a8822295654e92188effbceccf4a0f8f2891a4f64f8e40027d992eb633256336` |
| `q3-chatgpt-HAND-report.txt` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q3-chatgpt-HAND-report.txt` | `292af096c339f97c3b636e152d15615aa02b4aaba30e35e89722e38ad1035abc` |
| `question-hashes.txt` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/question-hashes.txt` | `5655b2123c627bc622938ebbeed450e7c46cf6490ffb7bf18c56f3d3f4bb265d` |
| `q3-post.json` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q3-post.json` | `149d4f08eba90660f9afd22cb85d395fac0913ed9f5e4b1ea0db4d51363796aa` |
