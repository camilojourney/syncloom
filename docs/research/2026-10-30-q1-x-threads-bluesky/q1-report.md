# Syncloom Q1: official API paths for X, Threads and Bluesky from a local Rust app

Study `main969-q1-xtb` on Phrona 5.0.12 (phrona.v2), Claude Research primary, ChatGPT Deep research challenger. Question file `questions/q1-x-threads-bluesky.md` passed unchanged, SHA256 `285735365f261f18051e8a92c3b0aacca6577933a7871500151bbdb919ed80aa` (verified against `question-hashes.txt` before start).

## Provenance (read this first)

| Provider | Label | Conversation | Size | SHA256 (word for word) | Sources |
|---|---|---|---|---|---|
| Claude (primary) | NATIVE | https://claude.ai/chat/0ba608e3-1e74-4f9f-a261-d9fa5eec596a | 23998 chars / 24514 B | `734b594c25035a5034ece477ae387a60e80443a8c031c1523af9159e3cce8c17` | 124 citations |
| ChatGPT (challenger) | NOT COLLECTED | no report | | | |

- Report files: `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q1-claude-NATIVE-report.txt`. Native Claude original also at `/Users/camiloslaptop/.local/state/phrona/durable-study-5.0.12-candidate/studies/main969-q1-xtb/reports/claude-734b594c25035a5034ece477ae387a60e80443a8c031c1523af9159e3cce8c17.txt`.
- Outcome: `complete`. Phase `complete`, `defaultEligible: False`. Coordinator calls 22/40, native calls 0, known cost USD 0.106. Langfuse trace `61679c020fa532576265b164756c1f82`, 129 events received, 0 pending.
- Collection failure codes recorded by 5.0.12: {"chatgpt": {}, "claude": {}}.

- ChatGPT Sources and activity panel: not applicable, no ChatGPT Send was made (challenger failed custody-uncertain, nothing sent).


## Answer tied to the DECISION (official API path per platform for one founder's own X, Threads and Bluesky accounts)

Single-model result: this study has the Claude report only (NATIVE, 124 citations). ChatGPT was never sent (see provenance), so nothing here is cross-checked by a second provider, and it is model research, not independent evidence.

- **X: pay-per-use API.** The Free tier closed to new developers on 6 Feb 2026 and Basic/Pro are legacy (Basic subscribers migrated after 1 Jun 2026). Post create is USD 0.015, or USD 0.20 if it contains a URL (since 20 Apr 2026). About 30 plain posts a month is about USD 0.45, 30 link posts about USD 6. Media upload uses the v2 chunked endpoints, which pay-per-use includes, plus `POST /2/media/metadata` (USD 0.005) for alt text. Auth is OAuth 2.0 PKCE with `offline.access`; access tokens last 2 hours, so persist the newest refresh token (rotation rule not found). Do not build reply bots, likes, follows, quote-posts or search/trends dashboards; automated replies are limited to "summoned" accounts since 23 Feb 2026, and AI-generated replies need X's prior approval.
- **Threads: development-mode Meta app with your own account as Threads Tester.** No App Review is needed for tester accounts (Meta: without advanced access "you can only post on behalf of Threads testers on your app"). Long-lived tokens last 60 days and refresh after 24 hours. Real friction: media is fetched only from a public URL, and Meta's sample README says redirect URIs must be HTTPS and not localhost.
- **Bluesky: app password, `createSession` once then refresh, through a Rust SDK (`bsky-sdk`/atrium 0.2.x, or jacquard 0.12.x), not raw XRPC.** Free, no review. Treat atproto OAuth as a later upgrade (public localhost clients get sessions of up to 2 weeks).

Your "WOULD CHANGE THE ANSWER" conditions: X posting for about 30 posts a month costing more than a few dollars: **no** (about USD 0.45 to 6). Media upload needing a paid tier: **no**. Threads publishing blocked by a review a solo app cannot pass: **no**.

## Your March 2026 evidence, corrected

- X Free 500 writes a month, no reads: corrected; closed to new sign-ups 6 Feb 2026. Basic USD 200: legacy, migrated to pay-per-use. Pay-per-use since Feb 2026: confirmed. About USD 0.01 per post: corrected to USD 0.015 (USD 0.20 with URL). USD 0.005 per read: confirmed for posts; owned reads USD 0.001, user reads USD 0.010.
- Scopes `tweet.write`, `media.write`, `offline.access`: confirmed, also need `tweet.read`, `users.read`. The v1.1 media endpoint is gone (sunset 9 Jun 2025), replaced by v2.
- Threads 500 characters, 250 posts a day, 60-day refreshable tokens, carousels up to 20 (minimum 2): confirmed. The 10,000-character text attachment: **not found** (attachments exist, figure unconfirmed).
- Bluesky 300 graphemes: confirmed via secondary sources citing the lexicon (primary lexicon not fetched); no official analytics API: not contradicted.

## Points to test in your first spike

- Whether self-replies to build your own X thread count as "summoned" replies: not found, test before relying on it. Putting a link in a later reply to avoid the USD 0.20 URL fee depends on it.
- X loopback redirect support: not found in official docs; register a fixed-port loopback URL and test. Publish on approval rather than on a timer to stay outside X's "automated account" rules; add delayed scheduling only after a policy support request.
- X post-read cap docs conflict (2 million vs 3 million per billing cycle); irrelevant at your volume.
- Rust crates: `twapi-v2` 0.27.0 (maintained), `xdk-rs` 0.3.0 (new), `threads-rs` 0.4.0 (maintained), atrium family released 27 Oct 2026 with breaking changes (pin versions). For X, hand-written `reqwest` is reasonable for the small surface.
- Not found: Threads and Bluesky terms on AI-generated content or automation, Bluesky sandbox, Bluesky trends, typical Meta review duration (irrelevant for tester mode).

## Collection provenance and behaviour seen live

- Claude (primary): collected natively (`complete-loaded-claude-document`), zero refusals, phase complete, confidence reduced, coordinator calls 22, known cost about USD 0.106.
- **ChatGPT was never sent.** Every paste failed with "Composer contains an unknown or mismatched draft"; reconcile reported custody uncertain with no conversation URL. I read the fresh ChatGPT pages afterwards, read-only: each composer held exactly the approved text of Syncloom Q8 (whitespace-normalised match to `q8-local-first-auth-and-scheduling.md`), i.e. an account-side restored draft from my own earlier Q8 studies. I cleared it (select-all, Backspace) on those pages only after the exact match, and a fresh owned ChatGPT page now opens empty. Main decided (msg 145): no rerun, same precedent as G4; the study stays complete with reduced confidence and the ChatGPT failure named. This is a phrona.v2 defect (own-question draft recognition is going into 5.0.13).

All findings are model research from two reports. Two providers agreeing is not independent evidence. Default stays 4.3.23.
