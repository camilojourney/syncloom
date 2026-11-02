# Q5: Publishing scope: TikTok, Reddit, Mastodon and Pinterest

Status: open

## Question

Which other platforms belong in this founder's publishing scope, through an official API or a manual handoff? Exact question: [q5](../2026-10-30-social-api-research-questions/questions/q5-tiktok-reddit-mastodon-scope.md).

## Why asked

Vision 6.2 slice 5 channels: Mastodon as a cheap API add-on, Reddit manual export now and TikTok later; no Pinterest.

## What would change the answer

An official API becoming available to unaudited solo apps, changed platform terms, or measured evidence of this founder's audience.

## How collected

Phrona 5.0.13 study `main969-q5-scope`, 2026-10-30, Claude Research only, native report with 110 citations. Challenger not requested; no second-provider cross-check. Single-provider result with reduced confidence, not default-eligible. Langfuse trace recorded by the operator: `cc08fcb260c5dfb9d0d0fd4557b94bb7`. The summary's final boilerplate claims two reports; the provenance explicitly establishes one Claude provider and takes precedence. Originals are copied byte for byte, including that inconsistency.

## Status

Open. Single-provider model research, reduced confidence; deciding platform facts need primary-source verification before live use. Reddit closure dates are secondary-source plans, not a guaranteed deadline. Mastodon instance limits and audience measurements remain unverified.

## Decision

[0004 Mastodon/manual-export](../../decisions/0004-mastodon-manual-export.md), proposed; this study informs its OpenSpec change, with approval still pending.


## Composition and adoption review (2026-11-02)

Domain: `syncloom.v2` (repository scope, not a delivered catalog identity). Phrona catalog ID: **pending owner/Main1308**.

Original study `main969-q5-scope`, Phrona 5.0.13; question SHA256 `36aaccf55bb61b8021a98ce3f94fd694c2222546b3621a35aefaead1e1c46c8d`. Original result receipt: `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/run/main969-q5-scope-result.json`, SHA256 `f17bdc8b042a015d76d2af1211500e0356729444c0a6082cd671ef6e947832bb`.

Native phase: **complete**; native confidence: **reduced**; default eligibility false. Native outcome has no failure.

| Actual member | Provider link | Preserved full report | Outcome |
|---|---|---|---|
| chatgpt | none: not sent | none | No Send; no report |
| claude | [original conversation](https://claude.ai/chat/b9e37486-b2f3-4bf4-9780-1241c94f83fd) | [NATIVE bytes](q5-claude-NATIVE-report.txt), SHA256 `13ce580e1d566aadca542df93eff2619bbdbfe6647a419597cef6f70ca50c10b`, 23033 bytes | Native full report; `research-not-finished` |

Challenger: `challenger-not-requested`. Missing challengers are not agreement. Provider claims remain reported, not independently verified by preservation. Original full report files are never annotated.

### Finding disposition

Historical [0004](../../decisions/0004-mastodon-manual-export.md), Proposed, no `/decide` marker. Built at `206b464`: [Mastodon adapter](../../../crates/syncloom-core/src/mastodon.rs), [manual bundles](../../../crates/syncloom-core/src/manual_export.rs), [fake HTTP journeys](../../../crates/syncloom-server/tests/mastodon_export.rs). Export is never posted. No Reddit/TikTok/Pinterest API is built; live-use/terms questions remain separate.

See [research/0001](../../decisions/research/0001-research-custody-and-retrieval.md) for this new archive choice, not retrospective acceptance of product recommendations.

## Files

Verbatim copies; never edit the originals.

| File | Origin | sha256 |
|---|---|---|
| `q5-report.md` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q5-report.md` | `29c3fc27bca77167732004df3c82fcec16153cbaef641c02efb858200cd85704` |
| `q5-claude-NATIVE-report.txt` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q5-claude-NATIVE-report.txt` | `13ce580e1d566aadca542df93eff2619bbdbfe6647a419597cef6f70ca50c10b` |
| `q5-post.json` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q5-post.json` | `2fab891794b2bee32197ed49222ce14f9fc17ef55be14b38ae9c5610aa25eb00` |
