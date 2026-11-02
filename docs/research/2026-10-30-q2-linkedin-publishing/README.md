# Q2: publishing to LinkedIn from a one-founder local Rust app

Status: open

## Question

How a solo founder's local Rust app publishes to LinkedIn through the official API: personal profile, company page, or both; whether PDF document carousels, multi-image posts and analytics are reachable for an app of one; token lifetimes and refresh; app review; rate limits; terms on automation and AI content; 2025-2026 breaking changes; test mode; Rust crates. The question as sent is [`questions/q2-linkedin.md`](../2026-10-30-social-api-research-questions/questions/q2-linkedin.md) (sha256 `dc08cbb4...`, verified unchanged by the operator before the study started).

## Why asked

Slice 2 of the build order (vision 6.2): the first platform integration, LinkedIn, is specified only after this answer is saved (vision 3.2). It settles which LinkedIn product and scope the integration uses, which surfaces are reachable without vetting, and what the token lifetime costs the founder.

## What would change the answer

The report confirms both conditions named in the question: company-page posting needs the vetted Community Management API that a one-person app without a registered legal entity cannot get, and no refresh tokens exist for the self-serve tier, so a browser re-consent every 60 days is the cost. What would move the answer again: LinkedIn opening programmatic refresh tokens or native PKCE to self-serve apps; LinkedIn restricting `/rest/documents`, `/rest/images` or `/rest/posts` for Share-only apps (the report found no first-hand confirmation either way for documents); a change in the Developer AI Policy on automated posting of AI-assisted content.

## How collected

Phrona 5.0.12 study `main969-q2-linkedin`, run by the phrona-lab home on 2026-10-30: Claude Research as the primary (collected natively, 123 citations) and ChatGPT Deep research as the challenger (collected by hand from the Deep research iframe; its sources panel read "Sources scanned 0", so it is model output without citations and weighs less, lesson 18). The study ended at its coordinator call limit (`original-limits`, 40 of 40), so Phrona marks it incomplete and not default-eligible; both reports are complete documents. Langfuse trace `c3897eab73af8b6cd97d9cd6720f314d`, 213 events, known cost USD 0.158. `q2-report.md` is the operator's summary tying both reports to the decision; `q2-post.json` is the study's result record. Nothing was sent beyond the approved question text.

## Status

Open. Answered for the decision (personal profile through self-serve Share on LinkedIn with `w_member_social` and the versioned REST Posts API, version 202610; company pages out of scope; no refresh tokens). Still open, listed as "Not Found" in the Claude report: whether `http://localhost` or `http://127.0.0.1` redirect URLs are accepted on the standard authorization flow; whether native PKCE can be enabled for a self-serve app without a LinkedIn point of contact; per-endpoint rate limits for `/rest/posts` under Share on LinkedIn; a first-hand report of a Share-only app succeeding on `/rest/documents`; an API field for AI-content disclosure. Not asked and not covered: the Posts API `commentary` text rules (reserved characters and escaping), to be checked at the first live post.

## Decision

[0002 LinkedIn personal-profile integration: self-serve Share on LinkedIn, Keychain tokens, loopback sign-in](../../decisions/0002-linkedin-integration-decisions.md).


## Composition and adoption review (2026-11-02)

Domain: `syncloom.v2` (repository scope, not a delivered catalog identity). Phrona catalog ID: **pending owner/Main1308**.

Original study `main969-q2-linkedin`, Phrona 5.0.12; question SHA256 `dc08cbb4ca38f3720635068d934130b30488673bf7181ec463decc355ca8ef29`. Original result receipt: `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/run/main969-q2-linkedin-result.json`, SHA256 `1723c767f14368445a119703c3ad1a147a757b22e5ca23d2f73adf2a60694060`.

Native phase: **incomplete**; native confidence: **none**; default eligibility false. Original outcome: `original-limits`: Original coordinator call limit reached: 40 of 40 calls used

| Actual member | Provider link | Preserved full report | Outcome |
|---|---|---|---|
| chatgpt | [original conversation](https://chatgpt.com/c/6ae4dcd1-8908-83ea-ad95-841df8b1cbf3) | [HAND bytes](q2-chatgpt-HAND-report.txt), SHA256 `ba56b1f5a9e117e6790fac4a52efa7237faceecfd5204420d10589ca5f490fe5`, 10114 bytes | Native report unresolved; later HAND retention, sources scanned 0; `report-document-changed`, `report-model-only-no-sources`, `report-unfinished-or-ambiguous` |
| claude | [original conversation](https://claude.ai/chat/fbfdf97a-0d29-4dda-a4ff-4568946a45bf) | [NATIVE bytes](q2-claude-NATIVE-report.txt), SHA256 `d7482cd4daaaf2025f6871642133a24c47bd6b8368380281960ee179d864954e`, 26525 bytes | Native full report |

Challenger: `unresolved`. HAND text does not repair the native terminal outcome and does not establish sourced corroboration. Missing challengers are not agreement. Provider claims remain reported, not independently verified by preservation. Original full report files are never annotated.

### Finding disposition

Historical [0002](../../decisions/0002-linkedin-integration-decisions.md), Proposed, no `/decide` marker. Built at `9e83462`: [LinkedIn adapter](../../../crates/syncloom-core/src/linkedin.rs), [fake HTTP sign-in/publish journeys](../../../crates/syncloom-server/tests/linkedin_api.rs). Personal-profile text only; company pages/visual upload/live registration remain unbuilt or unproven as recorded, not newly approved.

See [research/0001](../../decisions/research/0001-research-custody-and-retrieval.md) for this new archive choice, not retrospective acceptance of product recommendations.

## Files

Verbatim copies; the repository never edits them. Origins are paths on the machine that ran the study.

| File | Origin | sha256 |
|---|---|---|
| `q2-report.md` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q2-report.md` | `2db9c8874706ad88f83238ad1134d76855d5709023b79fd9897968ab8b8a80f0` |
| `q2-claude-NATIVE-report.txt` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q2-claude-NATIVE-report.txt` | `d7482cd4daaaf2025f6871642133a24c47bd6b8368380281960ee179d864954e` |
| `q2-chatgpt-HAND-report.txt` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q2-chatgpt-HAND-report.txt` | `ba56b1f5a9e117e6790fac4a52efa7237faceecfd5204420d10589ca5f490fe5` |
| `q2-post.json` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q2-post.json` | `dd0f4b84182906c94ad331dd2a8a7bfec86483001469fce9458b9d57b44292a9` |
