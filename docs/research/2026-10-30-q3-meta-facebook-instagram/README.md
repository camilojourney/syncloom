# Q3: Meta (Facebook Pages and Instagram) publishing from a local Rust app

Status: pending decision

## Question

What Facebook Pages and Instagram allow a single-user app through Meta's APIs for the founder's own accounts: login path and tokens, access level and app review, scheduling, carousels, insights, rate limits, terms and AI disclosure, 2025-2026 changes, test modes and Rust clients. The question as sent is [`questions/q3-meta-facebook-instagram.md`](../2026-10-30-social-api-research-questions/questions/q3-meta-facebook-instagram.md) (sha256 `0376181b...`, verified unchanged by the operator before the study started).

## Why asked

Slice 5 (more channels and their integrations) and the Instagram carousel in slice 3 (vision 6.2). Recorded now, built later; nothing in slice 2 depends on it.

## What would change the answer

Of the question's conditions: owned-account publishing in development mode without review holds (Standard Access suffices for the founder's own accounts); carousels are in the API (the "no carousel" note of March 2026 was wrong); Page tokens derived from a long-lived user token carry no expiry. What would move the answer: Meta requiring App Review or Business Verification for Standard Access on owned accounts; Instagram adding native scheduling; a change to the public-URL requirement for Instagram image containers, which forces a local-first app to expose each image at a short-lived public URL.

## How collected

Phrona 5.0.12 study `main969-q3-meta`, run by the phrona-lab home on 2026-10-30: Claude Research primary (collected natively, 198 citations, after one `research-not-finished`) and ChatGPT Deep research challenger (collected by hand; sources panel "Sources scanned 0", so model output without citations, lesson 18; the operator's summary lists where it contradicts itself). The study ended at its coordinator call limit (`original-limits`), so Phrona marks it incomplete and not default-eligible; both reports are complete documents. Langfuse trace `4498dca58af62f25674d89a658d0faa2`, 217 events, known cost USD 0.108. `q3-report.md` is the operator's summary; `q3-post.json` the result record.

## Status

Pending decision: answered (Facebook Login for Business with Page tokens, Standard Access without review, local scheduling with Facebook's native window as an option, Graph API v25.0) and the slice 5 decision it serves is not written yet.

## Decision

[Decision 0003](../../decisions/0003-visual-renderer.md) uses the slice brief's conservative ten-slide cap for local assets, without implementing Meta publishing. The slice 5 channels and integrations decision remains pending.

## Files

Verbatim copies; the repository never edits them. Origins are paths on the machine that ran the study.

| File | Origin | sha256 |
|---|---|---|
| `q3-report.md` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q3-report.md` | `61e56fbb559a4ea55a07436e3968066ded6815e16ffbc7a01026df601186cc83` |
| `q3-claude-NATIVE-report.txt` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q3-claude-NATIVE-report.txt` | `a8822295654e92188effbceccf4a0f8f2891a4f64f8e40027d992eb633256336` |
| `q3-chatgpt-HAND-report.txt` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q3-chatgpt-HAND-report.txt` | `292af096c339f97c3b636e152d15615aa02b4aaba30e35e89722e38ad1035abc` |
| `q3-post.json` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q3-post.json` | `149d4f08eba90660f9afd22cb85d395fac0913ed9f5e4b1ea0db4d51363796aa` |
