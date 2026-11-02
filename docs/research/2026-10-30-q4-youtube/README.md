# Q4: YouTube publishing from a local Rust app

Status: open

## Question

The exact approved [Q4 question](../2026-10-30-social-api-research-questions/questions/q4-youtube.md) asks about owner-only upload, verification/audit, quota, private/public defaults, scheduling, Shorts, analytics and Rust clients. Question SHA256 `0361cdae3af233cfa1339582c306333cc90323c28071e0eddd2e25dd5b7c296e`.

## Why asked

Vision 3.2 and slice 5 require platform research before specifying the YouTube adapter. The original study ran on 2026-10-30 but its reports were omitted from this repository.

## What would change the answer

Resolve the report's conflicting official forced-private wording, current quota/schema claims and owner-only OAuth eligibility before live use. A real account/upload experiment requires separate authorization and is not part of preservation.

## How collected

Phrona 5.0.12 `main969-q4-youtube`. Native Claude full report, 89 citations; ChatGPT was sent but its native report remained unresolved after source-free/ambiguous collection. Later HAND retention saved its complete text with sources scanned 0. Native study remains incomplete: original coordinator model request failed; no replacement model or allowance. No new research was run to import it.

## Status

Open, incomplete native outcome, no substantive decision or adapter built. Claims about quota, private locks, legal exemptions and audit timing are reported, not verified here. The provider's recommendations to create a project, submit an audit or upload a test video are not instructions to this worker and were not followed.

## Decision

[research/0001 custody and retrieval](../../decisions/research/0001-research-custody-and-retrieval.md) preserves the evidence only. YouTube implementation decision is none; prospective packet and challenge request are handed to firstmate, not accepted on account of report availability.

## Composition and adoption review (2026-11-02)

Domain: `syncloom.v2` (repository scope, not a delivered catalog identity). Phrona catalog ID: **pending owner/Main1308**.

Original study `main969-q4-youtube`, Phrona 5.0.12; question SHA256 `0361cdae3af233cfa1339582c306333cc90323c28071e0eddd2e25dd5b7c296e`. Original result receipt: `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/run/main969-q4-youtube-result.json`, SHA256 `78ad536eb979c33e62c81af9f13dba5603b4e626115e48afd0674d342f7cd797`.

Native phase: **incomplete**; native confidence: **none**; default eligibility false. Original outcome: `other`: The original coordinator model request failed; no replacement model or fresh allowance was selected

| Actual member | Provider link | Preserved full report | Outcome |
|---|---|---|---|
| chatgpt | [original conversation](https://chatgpt.com/c/6ae4e8a8-3c98-83e9-8f7b-dae9c0339d8f) | [HAND bytes](q4-chatgpt-HAND-report.txt), SHA256 `a36b47247720652cc935c0adbb058bbe8e222cfd1a8ab36be9a658659f358539`, 9536 bytes | Native report unresolved; later HAND retention, sources scanned 0; `report-model-only-no-sources`, `report-unfinished-or-ambiguous` |
| claude | [original conversation](https://claude.ai/chat/03589a1c-dac5-4375-b747-2cb237a57092) | [NATIVE bytes](q4-claude-NATIVE-report.txt), SHA256 `61e9241478cbd6390794797ef7076c2a6e087cc7de3e89bd2a01ba03845c5221`, 23933 bytes | Native full report; `research-not-finished` |

Challenger: `unresolved`. HAND text does not repair the native terminal outcome and does not establish sourced corroboration. Missing challengers are not agreement. Provider claims remain reported, not independently verified by preservation. Original full report files are never annotated.

### Finding disposition

No substantive YouTube decision or adapter implementation. A prospective decision packet covers a contained fake-only Rust adapter versus a local handoff or deferral; account configuration, audits, privacy defaults and real uploads remain owner calls.

See [research/0001](../../decisions/research/0001-research-custody-and-retrieval.md) for this new archive choice, not retrospective acceptance of product recommendations.

## Files

Verbatim source bytes, including the summary's own claims. No summary replaces either full report.

| File | Origin | sha256 |
|---|---|---|
| `q4-claude-NATIVE-report.txt` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q4-claude-NATIVE-report.txt` | `61e9241478cbd6390794797ef7076c2a6e087cc7de3e89bd2a01ba03845c5221` |
| `q4-chatgpt-HAND-report.txt` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q4-chatgpt-HAND-report.txt` | `a36b47247720652cc935c0adbb058bbe8e222cfd1a8ab36be9a658659f358539` |
| `q4-report.md` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q4-report.md` | `1202e8d51f165a5a45d14306eb346ff469811f89ddf1a45d0db8a669328369d9` |
| `q4-post.json` | `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q4-post.json` | `8b8fd00c16bee90a258df4e8d4462245029ca3608f4f60b99083165f1f700ebd` |
