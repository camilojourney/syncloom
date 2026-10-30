# syncloom.v2: research questions for the social integrations (and Q-RENDER)

Date: 2026-10-30. Knowledge task, read only. No web research was done and no `phrona` command was run. The eight questions are in `questions/` (each is the exact text to pass as the operator question file).

## Receipts

- Read: Social API `README.md`, `src/platforms/*.py` (all 7 adapters plus `base.py`), `src/api/v1/platform_matrix.py`, `src/api/v1/routes/connections.py`, `docs/platforms/*.md`, `docs/decisions/0001-platform-api-strategy.md`, `docs/roadmap.md` (old Social API, `/Users/camiloslaptop/.firstmate-homes/syncloom/projects/holus-social-api`, cited below as `API:`); old studio `docs/research/domain/INDEX.md`, `.agents/skills/post/SKILL.md`, `ARCHITECTURE.md`; plan sections 1-5 and the vision draft; `data/v2-build-lessons-20261030.md`; the invoz.v2 `phrona-questions.md` for the shape.
- Not read: `tests/` line by line (I only checked file names: `test_platforms.py`, `test_linkedin_document.py`, `test_linkedin_poll.py`); `shared_publish.py` and the queue workers (so "how scheduling is executed" below rests on ADR 0001 and the adapters, not on the worker code).
- Character counts (`wc -m`, limit 2000): q1 1975, q2 1973, q3 1978, q4 1987, q5 1994, q6 1981, q7 1924, q8 1963. A grep of the questions for personal names, the product names, the two brand-account labels, repository paths and em dashes finds nothing.
- Lessons file applied: items 10 (no personal contact in requests), 13 (nothing published or sent; these questions are research scope that still needs the owner's approval of the exact rendered text before `phrona` runs), 18 (every figure below is re-opened at a primary source; ChatGPT answers weigh less), 6 (answers get saved word for word under `docs/research/` with card, sha256, origin and INDEX row). I found none of the 23 items wrong or out of date for this task.

## Summary

1. The old Python service really does five things well enough to port: text and single-image posting to X, Threads, Facebook Pages, LinkedIn (personal) and Bluesky; LinkedIn PDF document and poll; Instagram image, Reel and Story; Facebook Reels; YouTube upload. It does no multi-image posts anywhere, no carousel on Instagram, Threads or Facebook, no company-page posting, no scheduling on any platform, no reply or comment reading, no research reads, and its OAuth connect flow is a stub.
2. The old platform notes (`API:docs/platforms/*.md`) are dated 2026-03-04 to 2026-03-11, seven to eight months old, with 30-day review cadences that lapsed in April 2026. Several are exactly the kind of fact that moves (X pricing, Meta Graph versions, LinkedIn scopes, YouTube quota). They are inputs to confirm or correct, not answers.
3. Eight questions: four platform studies, one scope question for TikTok/Reddit/Mastodon, one for research reads, Q-RENDER refreshed, and one I added: local-first credentials, OAuth and scheduling (the plan has no home for it and it decides whether Facebook-only native scheduling forces a local daemon).
4. Run order: Q7 now (unblocks slice 3, independent), then Q3 and Q2 (highest value and the longest approval clocks), Q8, Q1, Q4, Q6, Q5.

## 1. Adapter inventory (what the old adapters do today)

Base contract: `publish_text`, `publish_image`, `publish_video`, `validate_credentials`, optional `get_analytics`; one `PublishResult` (`API:src/platforms/base.py:61-152,17-27`). Tokens come from settings or a per-call dict, with two hard-coded brand accounts per platform.

| Platform | Operations built | Auth as coded | Limits encoded | Gaps and known failures | Citations |
|---|---|---|---|---|---|
| X | Text; single image (v1.1 `media_upload` via Tweepy, then v2 `create_tweet`); thread as a reply chain; video refused ("NOT published") | OAuth 1.0a user tokens plus bearer (consumer key/secret, access token/secret) | Truncates to 277 chars plus "..." silently; matrix says 280 and 17 posts/day | Docs and ADR 0001 say OAuth 2.0 PKCE with `media.write`; code uses 1.0a, so the doc and the code disagree. v1.1 media upload may be retired (verify in Q1). Analytics ask for `non_public_metrics` and fall back on 403 | `twitter.py:41,64-130,136-182,184-250,253-278,280-340,359-430`; `platform_matrix.py:121-137`; `API:docs/platforms/twitter.md:11,22-34` |
| Threads | Text, image, video through container, poll status, publish; insights (views, likes, replies, reposts, quotes, shares) | Long-lived user access token in settings; no refresh code | 500 chars (matrix); 250/day official, 50 safety | No carousel (docs list 2-20 items), no reply-reading, no text attachment, no search. Container polling 5 s, 60 s max. Token refresh was a stub for a worker never written | `threads.py:37-60,60-100,101-150,182-235,291-360,398-456`; `config/constants.py:154`; `platform_matrix.py:83-99`; `API:docs/platforms/connections.md:43-48` |
| Bluesky | Text; single image with alt text; video refused; analytics returns zeros | App password per account, atproto SDK login on every call (session not cached) | Cuts to 300 characters, but the limit is 300 graphemes | Zeros are indistinguishable from real zero engagement. No link or mention facets, no multi-image (up to 4 in the product) | `bluesky.py:18,31-49,51-67,69-102,104-114,124-137`; `API:docs/platforms/bluesky.md:23,45` |
| LinkedIn (personal) | Text; image; PDF document (initialize, PUT, post); poll (2-4 options, 30 chars, 140-char question, 1/3/7/14 days); video | Bearer access token plus hard-coded person URN; headers `LinkedIn-Version: 202602`, Restli 2.0.0 | 3000 chars (matrix); 20/day | No company page (no organization URN anywhere), no multi-image, no schedule. Analytics blocked: `r_member_social` closed to new applicants. Roadmap flagged the API version as "legacy" | `linkedin.py:29-66,125-262,264-421,423-550,551-729,747-774`; `platform_matrix.py:102-118`; `API:docs/roadmap.md:100-104` |
| Facebook Pages | Text `/feed`, photo `/photos`, Reels `/video_reels` (start, upload, finish); post insights | Page token per page; Graph version from settings default `v20.0` | 63206 chars; 25/day safety | No multi-photo, no link post, no native scheduling used even though ADR says only Facebook has it. Version default is old (docs mention v22 to v24) | `facebook.py:29-60,60-143,143-298,339-420`; `API:config/settings.py:128`; `API:docs/decisions/0001-platform-api-strategy.md` |
| Instagram | Single image, Reel, Story (image or video) via create container, poll, publish; insights (impressions, reach, likes, comments, shares, saved, views) | Same Graph token and version as Facebook | Caption 2200, 25/day safety, 100/24 h official; poll 5 s up to 60 times | Text refused. **No carousel**. Default location ID baked in. Insights 400 handled by returning None. Stories carry no stickers | `instagram.py:52-93,93-160,163-318,340-560,591-640`; `platform_matrix.py:35-52` |
| YouTube | Resumable upload of one video (public; "#Shorts" appended when `is_short`); view/like/comment counts | Google OAuth refresh token exchanged each call | Title cut to 100; matrix 5000 chars | Reads the whole file into memory, no chunked resume, no scheduled `publishAt`, no thumbnails or captions, no `containsSyntheticMedia`. Text and image refused | `youtube.py:33-62,87-176,188-241`; `API:docs/platforms/youtube.md:16-25,56` |
| TikTok, Pinterest, Reddit | **No adapter.** Capability-matrix rows and routing rules only ("Stage 2") | Declared as OAuth 2.0 in the matrix | Matrix limits only | Never built. TikTok has a long research note (24 h access token, rotating 365-day refresh, audit) | `platform_matrix.py:15-19,192-248`; `API:docs/platforms/tiktok.md:14-31,132-160` |

Cross-cutting facts about the service:

- **The OAuth connect flow is a stub.** The start URL is `https://oauth.local/{platform}/authorize` and the callback stores fake tokens built from the code string (`API:src/api/v1/routes/connections.py:71,146-147`). Real credentials live in env vars; the docs list manual token exchanges with expiry around 2026-05-05 (`API:docs/platforms/connections.md:21-24,63`), which has passed. So nothing in the old code proves a working per-user authorization path.
- **Scheduling is a local cron and queue.** Only Facebook has native scheduling per ADR 0001; the queue enforces per-platform daily counts and minimum intervals (matrix `rate_limits`, e.g. 25/day and 3 s, 17/day on X, 20/day on LinkedIn).
- **Analytics are uneven:** real on X (public plus non-public metrics), Threads, Facebook, Instagram, YouTube (counts only); zeros on Bluesky; None on LinkedIn.
- **The old studio never reads platforms for research.** `.agents/skills/post/SKILL.md` and `ARCHITECTURE.md:43-49,156-158` only call publish/schedule and analytics through the Social API client. Research Radar sources are not social platforms (plan section 2.2).
- **Default target for the studio is LinkedIn text, carousel and image** (`.agents/skills/post/SKILL.md:33-39`); the studio pipeline lists linkedin text/image/carousel, instagram image/carousel, threads, X thread and facebook (plan section 1.2). So Instagram carousel and Facebook/Threads images are wanted by the studio but absent in the adapters: that gap is the main thing Q3 and Q1 must price.

## 2. Which question serves which syncloom.v2 slice

Slices are from plan section 3 and vision 6.2.

| Question | File | Serves | What it settles |
|---|---|---|---|
| Q1 X + Threads + Bluesky | `q1-x-threads-bluesky.md` | Slice 5 (more channels); X thread and Threads text come early; slice 2 platform rules | which X access path and cost; Threads dev-mode vs review; Bluesky auth and SDK; platform-rules owner values |
| Q2 LinkedIn | `q2-linkedin.md` | Slice 2 (first live channel), slice 3 (PDF document carousel), later analytics | personal vs company page, document upload, refresh tokens, approval clock |
| Q3 Facebook Pages + Instagram | `q3-meta-facebook-instagram.md` | Slice 5, slice 3 (Instagram carousel) | Meta review need for owned accounts, carousel support, token handling |
| Q4 YouTube incl. Shorts | `q4-youtube.md` | Video slice (after slice 5; video is deferred, vision 7.2) | verification, quota, upload privacy limits, Shorts rules |
| Q5 TikTok, Reddit, Mastodon scope | `q5-tiktok-reddit-mastodon-scope.md` | Scope decision for slice 5 | which extra platforms exist at all in v2 |
| Q6 Reading platforms for research | `q6-reading-platforms-for-research.md` | Research Radar and intake slices ("Then" in plan section 3); also the performance feedback loop | what research reading is allowed; which stay out |
| Q7 Q-RENDER | `q7-render-images-carousels-rust.md` | Slice 3 (visual assets) | R1-R4 renderer choice |
| Q8 Local-first auth and scheduling | `q8-local-first-auth-and-scheduling.md` | Slice 2 (dispatch) and the credentials design before any live post | keychain, loopback OAuth, rotating tokens, scheduling model, idempotency |

Dependency note for the captain's D2 (Social API stays external vs folded in): the captain now wants the integrations rebuilt. If D2 becomes "fold in", Q8 and Q1-Q4 become the design inputs for a `syncloom-social` crate and slice 2 changes from "client for the old API" to "adapters in Rust". Q1-Q4 and Q8 are useful under either answer.

## 3. Order to run

1. **Q7 (Q-RENDER) first.** Independent, needed before slice 3, no platform dependency, and the only question about a pure engineering tradeoff.
2. **Q2 (LinkedIn) and Q3 (Meta) in parallel.** LinkedIn is the studio's primary channel, and the old notes carry the longest approval estimates (LinkedIn 2-16 weeks, Meta 2-8 weeks, `API:docs/decisions/0001-platform-api-strategy.md`, `API:docs/platforms/instagram.md:104`). If a clock exists, starting the app registration early is worth more than anything else. Q3 also decides whether Instagram carousels are possible.
3. **Q8 (credentials, OAuth, scheduling)** once Q2 and Q3 are in, since it reuses their auth findings, but it can start earlier if the providers allow parallel runs.
4. **Q1 (X, Threads, Bluesky).** Dense; if the answer is shallow on X pricing, split X into its own run.
5. **Q4 (YouTube)**, then **Q6 (research reads)**, then **Q5 (extra platforms)**. Q5 is the lowest value: my read of the captain's "all the social media" for this founder is below.

Provider split (Phrona skill): Claude Research primary for every question; ChatGPT Deep research as challenger only on Q1, Q2, Q3 and Q6, the four where pricing and policy facts drift, and with lesson 18 in mind (its reports so far read "Sources scanned 0"). Before submitting, each exact rendered request needs the owner's approval (or a stated public-context delegation) under the Phrona skill, since it goes to an outside provider. Suggested request keys: `syncloom-q1-x-threads-bluesky-20261030` through `syncloom-q8-local-first-20261030`.

## 4. Which platforms belong in "all the social media" for this founder

- **In, with questions Q1-Q4:** X, Threads, Bluesky, LinkedIn, Facebook Pages, Instagram, YouTube. These are the seven the old service shipped and the studio already plans (plan section 1.2).
- **TikTok: in question, not in plan.** The old service had a 160-line research note and a matrix row but no adapter. The founder's outputs are text, images and PDFs; TikTok is video-first with photo posts as a side format, and video is deferred (vision 7.2). One scope question (Q5) decides it. My recommendation: do not build it before Q5 returns.
- **Reddit: reading yes, posting probably no.** Every Reddit post targets a community with its own rules, which the old roadmap itself says is unlike a broadcast channel (`API:docs/roadmap.md`, Reddit note). Reddit is a good research source (Q6); Q5 asks whether posting is sensible at all.
- **Mastodon: no.** The old roadmap skipped it because Bluesky covers federated social (`API:docs/roadmap.md:113`). It stays in Q5 only so the captain has an evidence-based "no". If the captain does not want that spent, delete it from Q5 and the question shrinks.
- **Pinterest, Telegram, Google Business Profile, Snapchat, Discord, WhatsApp:** not asked. Pinterest and Telegram appeared in the old Stage 2 plan; no evidence in the studio's journey asks for them. Pinterest is named in Q5's decision line only so the answer can say no.

## 5. What the old research already answers (do not ask again), with dates

None of these is closed, because every date is older than eight months. They are the "current evidence to confirm or correct" inside each question, which is the cheaper way to use them: the provider confirms or corrects instead of discovering.

| Topic | Old claim | Date and source | Treated as |
|---|---|---|---|
| LinkedIn formats and limits | 3000-char posts; document posts PDF/PPT up to 100 MB and 300 pages; MultiImage 2-20; poll limits; images up to ~36M pixels | `API:docs/platforms/linkedin.md` 2026-03-04; adapter code `linkedin.py:423-550` | seed for Q2; limits are cheap to re-check |
| LinkedIn tokens and tiers | 60-day access token; 365-day refresh needs partner approval; tiers Development/Standard; approval 2-16 weeks | same file; ADR 0001 2026-02-13 | seed for Q2 (the high-risk item) |
| LinkedIn analytics | `r_member_social` closed to new applicants | `API:docs/platforms/linkedin-analytics.md` 2026-03-11; `linkedin.py:754-762` | seed for Q2 |
| LinkedIn content and cadence | engagement by format, posting frequency, visual lens | old studio `docs/research/domain/linkedin-content-format-engagement.md` 2026-03-27, `linkedin-posting-frequency.md` 2026-03-19, `linkedin-visual-impact-lens.md` 2026-06-18 | **not API questions; not re-asked.** The plan already declines to re-ask carousel limits (plan section 5). Their 30-day review dates lapsed April 2026; refresh when the carousel slice runs |
| X tiers | Free 500 writes; Basic $200; pay-per-use from Feb 2026 about $0.01 per post and $0.005 per read | `API:docs/platforms/twitter.md:22-34`, `twitter-analytics.md:36-51` 2026-03 | seed for Q1; **pricing is exactly the kind of figure that moves** |
| Threads | 500 chars; text attachment 10,000; 5 links; carousels 2-20; 250 posts/day, 1000 replies/day, 500 searches/7 days; 60-day tokens, refresh endpoint | `API:docs/platforms/threads.md`, `docs/platforms/analytics-api-reference.md` 2026-03-04/11; `connections.md` 2026-03-06 | seed for Q1 |
| Meta | Page tokens non-expiring; user tokens 60 days; Instagram 100 posts/24 h, content-publishing-limit endpoint; App Review 2-8 weeks; Graph v21-v24 metric removals | `API:docs/platforms/instagram.md`, `facebook.md`, `analytics-api-reference.md` 2026-03 | seed for Q3 |
| Bluesky | no app review; 300 graphemes; app password; session limits 30/5 min and 300/day (WAF 100/day); 25 uploads/day | `API:docs/platforms/bluesky.md` 2026-03-04 | seed for Q1 |
| YouTube | 10,000 units/day; unverified apps 100 users; audit for quota increase; scheduling via private plus `publishAt` | `API:docs/platforms/youtube.md` 2026-03-04 | seed for Q4 |
| TikTok | Direct Post needs audit, Inbox/draft easier; token 24 h, refresh 365 d rotating; photo posts up to 35; AIGC labelling Aug 2025; Creator Health Rating Jan 2026 | `API:docs/platforms/tiktok.md`, `docs/knowledge/tiktok.md`, `API:docs/IMPLEMENTATION-STATUS.md:209` 2026-03 | seed for Q5 |
| Rendering | Playwright/Chromium works, 49 tests; no Rust evidence | old studio `docs/research/domain/carousel.md` 2026-03-16, `agent-rendering-capabilities.md` 2026-06-18; plan Q-RENDER | Q7 |
| Platform-rule mismatch to fix in slice 2 | three disagreeing platform-rule tables (plan section 2.1; arbiter review) | plan 2026-10-29 | design, not research |

Provenance warning: the old notes do not say whether they came from a cited report or from model output, and several URLs inside them are third-party blogs (for example `API:docs/platforms/analytics-api-reference.md:581-585`). Per lesson 18 treat every number as unverified until Q1-Q6 return with primary-source citations.

## 6. Refinements for the question set (what I changed from the proposal)

- Added Q8 (local-first credentials, OAuth and scheduling) because the old service's OAuth is a stub and the new app has no server; without it each platform study would answer auth in isolation.
- Kept the proposed grouping (X + Threads + Bluesky; LinkedIn; Facebook Pages + Instagram; YouTube) but gave LinkedIn and YouTube their own studies. A grouped X/Threads/Bluesky question is dense (1975 chars); if the provider's answer is shallow, run X alone.
- Q6 includes Hacker News and arXiv-like sources only as the fallback option (E), not as research targets.
- No question asks the provider about the founder, accounts, or code. Evidence from the old adapters is stated generically ("a Python client...").

## 7. Risks and what I did not do

- Pricing, scopes and review times are the likeliest to be wrong in the seeds; the questions say "unverified, confirm or correct".
- The 2000-character cap forces compression; Q3 and Q6 are the two most likely to need a follow-up.
- I did not check Phrona CLI argument shapes or run a dry rendering of the requests.
- Approvals: running these questions sends text to outside providers. No captain-owned decisions are opened by this report; the only decision the captain may want is the Q5 trim above.
