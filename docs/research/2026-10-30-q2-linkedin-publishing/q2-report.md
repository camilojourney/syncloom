# Syncloom Q2: publishing to LinkedIn from a one-founder local Rust app

Study `main969-q2-linkedin` on Phrona 5.0.12 (phrona.v2), Claude Research primary, ChatGPT Deep research challenger. Question file `questions/q2-linkedin.md` passed unchanged, SHA256 `dc08cbb4ca38f3720635068d934130b30488673bf7181ec463decc355ca8ef29` (verified against `question-hashes.txt` before start).

## Provenance (read this first)

| Provider | Label | Conversation | Size | SHA256 (word for word) | Sources |
|---|---|---|---|---|---|
| Claude (primary) | NATIVE | https://claude.ai/chat/fbfdf97a-0d29-4dda-a4ff-4568946a45bf | 25901 chars / 26525 B | `d7482cd4daaaf2025f6871642133a24c47bd6b8368380281960ee179d864954e` | 123 citations |
| ChatGPT (challenger) | HAND | https://chatgpt.com/c/6ae4dcd1-8908-83ea-ad95-841df8b1cbf3 | 9996 chars / 10114 B | `ba56b1f5a9e117e6790fac4a52efa7237faceecfd5204420d10589ca5f490fe5` | sourcesScanned 0; panel: Citations 0 ("No sources detected"); 2 links inside the report body |

- Report files: `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q2-chatgpt-HAND-report.txt`; `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q2-claude-NATIVE-report.txt`. Native Claude original also at `/Users/camiloslaptop/.local/state/phrona/durable-study-5.0.12-candidate/studies/main969-q2-linkedin/reports/claude-d7482cd4daaaf2025f6871642133a24c47bd6b8368380281960ee179d864954e.txt`.
- Outcome: `original-limits` ("Original coordinator call limit reached: 40 of 40 calls used"). Phase `incomplete`, `defaultEligible: False`. Coordinator calls 40/40, native calls 0, known cost USD 0.158. Langfuse trace `c3897eab73af8b6cd97d9cd6720f314d`, 213 events received, 0 pending.
- Collection failure codes recorded by 5.0.12: {"chatgpt": {"report-document-changed": 2, "report-model-only-no-sources": 1, "report-unfinished-or-ambiguous": 7}, "claude": {}}.
- HAND reads (ChatGPT) were zero-click and zero-input (inputEvents {cg.get('inputEvents','n/a')}, clicks {cg.get('clicks','n/a')}) after browser-identity, exact-URL, question-association and frame-lineage checks, taken from the Deep research iframe. HAND reports never count toward native acceptance.
- ChatGPT Sources and activity panel (5.0.12 opened it natively; I made no click): Sources scanned 0, Citations 0 ("No sources detected"), connector sources scanned 0. Panel text: `Sources | Activity · 8m |  | Citations |  | No sources detected |  | Sources scanned · 0 |  | No sources scanned |  | Connector sources scanned |  | No connector sources scanned`.


## Answer tied to the DECISION (how a one-founder local Rust app publishes to LinkedIn)

Both reports reach the same top-level answer; the Claude report is the one with dated primary sources.

- **Personal profile: Option A.** Self-serve "Share on LinkedIn" with `w_member_social` and the versioned REST Posts API. Claude (NATIVE, 123 citations) cites Microsoft Learn pages (Posts, Documents, MultiImage, Poll, Images, Videos) that list `w_member_social` for text, image, multi-image (2-20), PDF document (100 MB, 300 pages), poll, video and article posts. ChatGPT (HAND, uncited) agrees.
- **Company page: Option D today, Option B only with a registered legal entity.** Claude: the Community Management API is vetted ("registered legal organizations for commercial use cases only", business email, page verified by a super admin, must be the only product on its app, approval "at its discretion"). Development tier allows 500 calls per app and 100 per member per 24 h. ChatGPT agrees B is gated but adds "single-person use is usually rejected", for which the Claude report found no source (it says no doc excludes or approves single-user apps; a sole founder with a registered company and domain email can plausibly pass).
- **Option C (refresh tokens): not realistically available.** Both: 365-day programmatic refresh tokens only for "a limited set of partners" / approved MDP partners; no self-serve route found.
- **Cost of no refresh tokens:** a browser re-consent at least every 60 days (about 6-7 runs a year). If the member is still logged in to linkedin.com and the token has not expired, LinkedIn skips the consent screen (primary source). Changing requested scopes invalidates old tokens.

Your "WOULD CHANGE THE ANSWER" conditions: company-page posting does need an approval a one-person app may not get (confirmed, primary); PDF posting for the personal profile does not need that approval on paper, but Claude found no first-hand (non-vendor) 403 or success report for a Share-only app on `/rest/documents`, so test each content type before building on it. No refresh tokens is confirmed.

## Your March 2026 evidence, confirmed or corrected (Claude, dated)

- Text/image/PDF/poll/video on a personal profile with `w_member_social`: consistent; only a vendor (checked 2026-10-25) and your own test support PDF in practice.
- Poll 2-4 options and durations 1/3/7/14 days: confirmed (Poll API, view 202602). The 30-character option limit: not found in primary docs, only third-party SDK docs.
- Version 202602: confirmed Active until 15 Feb 2027; move to 202610 (Active until 15 Oct 2027). 202511 sunsets 16 Nov 2026.
- "Analytics blocked because r_member_social is closed": corrected. `r_member_social` is closed to new requests, but post metrics come from `r_member_postAnalytics` (`memberCreatorPostAnalytics`, launched 202506), which belongs to the vetted Community Management API. Analytics are gated, not closed.
- Access tokens 60 days: confirmed. 365-day refresh needs partner approval: confirmed. Approval "2-16 weeks": not found in primary sources, unverified ("no fixed timeline" for restricted APIs).
- Company-page posting never built: it needs the gated Community Management API.

## Other findings that matter

- No native scheduling field in the Posts API (local scheduling needed). Edit via PARTIAL_UPDATE; delete via DELETE (idempotent). First comment, replies and reading comments need `w_member_social_feed` / Community Management on paper; a forum report shows a Share-only app getting 403 on `/rest` reactions. Reading others' public posts: not possible. Organic native carousel: not supported (PDF document is the swipe format).
- Rate limits: Share on LinkedIn lists member 150/day, application 100,000/day on the legacy page; per-endpoint limits for `/rest/posts` and `/rest/documents` not published.
- Terms: User Agreement forbids bots and unauthorised automation; official API through an approved product is authorised. Developer AI Policy (updated 2026-01-14): restrict automated posting of AI-generated content without end-user involvement; users must be able to edit first. Your approve-before-publish design satisfies the core requirement. LinkedIn shows a visible C2PA badge on uploads that carry content credentials; no API field for AI disclosure was found.
- Auth for a local app: LinkedIn's native PKCE flow exists but is not self-serve ("reach out to your point of contact"); the standard flow with the client secret in the OS keychain is the practical pattern. Whether `http://localhost` redirects are accepted on the standard flow: not found.
- Test mode: no sandbox found; Token Generator or a separate unpublicised page. Rust: no official SDK; `gen-linkedin` 0.1.36 is a minimal text-post client; no crate covers Documents/MultiImage/Poll; use `oauth2` 5.0.0 plus `reqwest` and hand-written wrappers.

## Where the ChatGPT report is weaker (HAND, uncited)

- No dates and no per-claim links (0 citations in the sources panel, only 2 links inside the report body, listed in q2-post.json).
- States that a self-serve app can use the Comments API with `w_member_social_feed`; the Claude report traces that permission to Community Management only.
- Says "r_member_social_feed" closed and personal analytics essentially unavailable, but misses `r_member_postAnalytics` as the gated route. Misses the "only product on the app" exclusivity rule for B, the Developer AI Policy details, and the version sunset table; says Marketing API v202511 "was sunset Nov 2026" although that date is still two weeks away.
- Video limits stated as 500 MB and 5 GB together without noting the doc inconsistency Claude flags.
- It adds Rust advice (`oauth2` plus `reqwest`), which agrees with Claude.

## Collection provenance and 5.0.11 / 5.0.12 behaviour seen live

- Claude (primary) collected natively on the first try with zero refusals: `complete-loaded-claude-document`, 123 citations.
- ChatGPT: 5.0.12 opened the Sources and activity panel itself ("Sources scanned 0", Citations "No sources detected") and recorded `report-model-only-no-sources` once, `report-document-changed` twice and `report-unfinished-or-ambiguous` seven times. The page, read afterwards, held one complete report with 0 busy nodes. So the report is complete, uncited and refused by design; HAND applied and is labelled.
- The study ended `original-limits` at 40/40 coordinator calls, so it is not default-eligible.

All findings are model research from two reports. Two providers agreeing is not independent evidence. Default stays 4.3.23.
