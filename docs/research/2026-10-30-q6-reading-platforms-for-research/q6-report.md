# Syncloom Q6: reading social platforms for topic research

Study `main969-q6-reads` on Phrona 5.0.13 (phrona.v2, primary-only, release `010b331631398ef46eff22ac`), Claude Research primary, `challenger: none`. Question file `questions/q6-reading-platforms-for-research.md` passed unchanged, SHA256 `f0600ba8d822367a44ad0bd5bec490c206f4d01866feb20f517034fff792d2ad` (verified against `question-hashes.txt` before start).

## Provenance (read this first)

| Provider | Label | Conversation | Size | SHA256 (word for word) | Sources |
|---|---|---|---|---|---|
| Claude (primary) | NATIVE | https://claude.ai/chat/447eaff0-6934-459f-99fa-3879538df1df | 25885 chars / 26453 B | `a4c5375c18a51ae3f6e0b6c088586c328a99916272c0ae32d4de7e2cb715bf8c` | 136 citations |
| ChatGPT (challenger) | NOT REQUESTED | challenger not requested (captain: ChatGPT replaced while model-only) | | | |

- Report files: `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q6-claude-NATIVE-report.txt`. Native Claude original also at `/Users/camiloslaptop/.local/state/phrona/durable-study-5.0.13-candidate/studies/main969-q6-reads/reports/claude-a4c5375c18a51ae3f6e0b6c088586c328a99916272c0ae32d4de7e2cb715bf8c.txt`.
- Outcome: `complete`. Phase `complete`, `defaultEligible: False`. Coordinator calls 17/40, native calls 0, known cost USD 0.087. Langfuse trace `dad4297694d82aa1c8a5adbe7381adb5`, 88 events received, 0 pending.
- Collection failure codes recorded by 5.0.13: {"chatgpt": {}, "claude": {"research-not-finished": 2}}.

- ChatGPT Sources and activity panel: not applicable. Challenger not requested (captain: ChatGPT replaced while model-only); `challenger: none`, no ChatGPT page was opened or sent. Single-provider result with reduced confidence.


## Single-provider result, reduced confidence

Challenger: **not requested (captain: ChatGPT replaced while model-only)**. This study ran on 5.0.13 with `challenger: none` and a single Claude Research Send, so nothing here is cross-checked by a second provider. It is model research from one report, not independent evidence, and the study is not a native pair and not default-eligible. A challenger-only pass may follow later if a working Gemini or Grok challenger is installed (per Main).

## Answer tied to the DECISION (how a single founder's local studio may read social platforms to research topics)

**Recommended combination: E + D + a narrow A.** Reject B (polite fetching of public pages) and C (licensed provider).

- **(E) news, RSS, Hacker News, arXiv as the base:** no terms friction and free. Hacker News: Firebase API ("There is currently no rate limit") and Algolia search at 10,000 requests per hour per IP.
- **(D) your own posts, comments and metrics:** cheapest and cleanest. X owned reads USD 0.001 per resource; YouTube Analytics; Threads insights (needs `threads_manage_insights`, app approval).
- **(A) narrow official read APIs:** Bluesky public AppView and the open firehose and Jetstream (no authentication, free); YouTube (free; since 1 Jun 2026 `search.list` is its own bucket of 100 calls a day at 1 unit); X pay-per-use USD 0.005 per post read, USD 0.010 per trends request, recent search covers 7 days, with a hard spending limit of about USD 5-10 a month; Threads keyword and tag search after App Review (2,200 queries per user per rolling 24 hours).
- **Closed or effectively closed to a non-academic individual:** Reddit (new public Data API requests stop 31 Oct 2026, RSS ends 13 Nov 2026, remaining public access ends March 2027), LinkedIn (no public post search, member social activity data cached at most 48 hours per secondary sources, scraping barred by contract), TikTok (Research API for academics and non-profits only), Facebook and Instagram research (Meta Content Library needs an academic or not-for-profit affiliation; Instagram hashtag search needs App Review and is limited to 30 unique hashtags per rolling 7 days).
- **(B) rejected:** scraping is contractually barred on X, LinkedIn and Reddit and enforcement is escalating: on 31 Jul 2026 Judge Engelmayer (SDNY) let Reddit's DMCA anti-circumvention claims against SerpApi and Perplexity proceed (CAPTCHA and bot detection can count as access controls), and X sent cease-and-desist letters to Nitter on 24 Aug 2026 (TechCrunch, 25 Aug 2026). X v. Bright Data (May 2024) went the other way on contract preemption, so the legal picture is mixed. Logged-in browser automation is the worst case.
- **(C) rejected on price:** licensed vendors are quote-based (Vendr medians via Octolens, Sep 2026: Brandwatch about USD 50,000 a year, Meltwater about USD 25,800, Talkwalker about USD 13,300); cheap self-serve APIs are scraping-based and move option B's risk to a vendor.
- **Implementation guidance from the report:** keep only derived topic aggregates (term counts, URLs, IDs); expire raw text within 24-48 hours (the strictest limit found, LinkedIn); honour YouTube's 30-day Non-Authorized Data rule and Bluesky deletion events; never use platform content to fine-tune a model (X forbids training any foundation or frontier model on its API data except Grok).

Your "WOULD CHANGE THE ANSWER" conditions: personal research reads banned or priced beyond a few dollars a month on most platforms: **not met across the board** (about half the platforms are open and cheap), but E carries most of the weight. Clear permission for non-commercial research on a platform: the only clear one was Reddit's, and it closes to new applicants on 31 Oct 2026; Meta and TikTok researcher programs exclude non-academic individuals. A pre-launch business studio should assume it is commercial use.

## March 2026 evidence, corrected

- X about USD 0.005 per post read: confirmed (owned reads USD 0.001). X 7-day search on Basic: corrected; Basic is no longer offered to new developers and recent search covers 7 days for all pay-per-use developers, plus full-archive search on pay-per-use.
- Threads 500 queries per 7 days: corrected to 2,200 queries per user per rolling 24 hours; results beyond the user's own posts need `threads_keyword_search` approval. No trending endpoint found.
- LinkedIn: directionally confirmed from secondary sources only (no public post search); "closed to new applicants" wording not found in a primary source. Bluesky public data open: confirmed. YouTube search 100 units of 10,000 a day: outdated since 1 Jun 2026.

## Gaps and conflicts the report names

X monthly read cap conflicts between its own pages (3 million vs 2 million); X rules on LLM prompt input and caching: not found. Reddit terms and exact closure timing were read only through secondary sources. Bluesky terms on third-party AI use, `searchPosts` and trending endpoints, primary Instagram and Facebook insights docs, Meta/TikTok/YouTube scraping clauses and HN's terms: not found. LinkedIn section 8.2, hiQ, Meta v. Bright Data, X v. Bright Data and Reddit v. SerpApi come from secondary legal commentary; have counsel confirm before relying on them.

## Collection provenance

- Claude (primary): collected natively (`complete-loaded-claude-document`) after two transient `research-not-finished`: 26453 B, 136 citations. Phase `complete`, 17 of 40 coordinator calls, known cost USD 0.087 (cost known), Langfuse trace `dad4297694d82aa1c8a5adbe7381adb5` with 88 events received, 0 pending.
- Started at 18:45 local on 5.0.13 as the first of at most two concurrent studies; one Claude Send, no resend; fresh Claude page checked empty before start.

All findings are model research from two reports. Two providers agreeing is not independent evidence. Default stays 4.3.23.
