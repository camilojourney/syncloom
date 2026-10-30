# Syncloom Q3: Meta (Facebook Pages and Instagram) publishing from a local Rust app

Study `main969-q3-meta` on Phrona 5.0.12 (phrona.v2), Claude Research primary, ChatGPT Deep research challenger. Question file `questions/q3-meta-facebook-instagram.md` passed unchanged, SHA256 `0376181b391f348548d182fc20f50a55e289f8b8dde58a1865dc82948b98c1d0` (verified against `question-hashes.txt` before start).

## Provenance (read this first)

| Provider | Label | Conversation | Size | SHA256 (word for word) | Sources |
|---|---|---|---|---|---|
| Claude (primary) | NATIVE | https://claude.ai/chat/65c9c98f-6fca-4074-8259-5a3272c13c7c | 30933 chars / 31771 B | `a8822295654e92188effbceccf4a0f8f2891a4f64f8e40027d992eb633256336` | 198 citations |
| ChatGPT (challenger) | HAND | https://chatgpt.com/c/6ae4dccd-4330-83ea-a952-ffde6ce84f30 | 8954 chars / 9039 B | `292af096c339f97c3b636e152d15615aa02b4aaba30e35e89722e38ad1035abc` | sourcesScanned 0; panel: Citations 0 ("No sources detected"); 2 links inside the report body |

- Report files: `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q3-chatgpt-HAND-report.txt`; `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q3-claude-NATIVE-report.txt`. Native Claude original also at `/Users/camiloslaptop/.local/state/phrona/durable-study-5.0.12-candidate/studies/main969-q3-meta/reports/claude-a8822295654e92188effbceccf4a0f8f2891a4f64f8e40027d992eb633256336.txt`.
- Outcome: `original-limits` ("Original coordinator call limit reached: 40 of 40 calls used"). Phase `incomplete`, `defaultEligible: False`. Coordinator calls 40/40, native calls 0, known cost USD 0.108. Langfuse trace `4498dca58af62f25674d89a658d0faa2`, 217 events received, 0 pending.
- Collection failure codes recorded by 5.0.12: {"chatgpt": {"report-document-changed": 1, "report-unfinished-or-ambiguous": 10, "report-model-only-no-sources": 1}, "claude": {"research-not-finished": 1}}.
- HAND reads (ChatGPT) were zero-click and zero-input (inputEvents {cg.get('inputEvents','n/a')}, clicks {cg.get('clicks','n/a')}) after browser-identity, exact-URL, question-association and frame-lineage checks, taken from the Deep research iframe. HAND reports never count toward native acceptance.
- ChatGPT Sources and activity panel (5.0.12 opened it natively; I made no click): Sources scanned 0, Citations 0 ("No sources detected"), connector sources scanned 0. Panel text: `Sources | Activity · 6m |  | Citations |  | No sources detected |  | Sources scanned · 0 |  | No sources scanned |  | Connector sources scanned |  | No connector sources scanned`.


## Answer tied to the DECISION (Meta APIs for one founder's own Facebook Pages and Instagram professional account)

Both reports agree on the headline; the Claude report (NATIVE, 198 citations) is the one with dated Meta primary sources and it corrects several of your March 2026 items.

- **(A) Login path: Facebook Login for Business with Page tokens (Instagram API with Facebook Login).** One login covers Facebook Pages and the linked Instagram account; Page tokens derived from a long-lived user token have no expiry date; resumable local video upload (`upload_type=resumable`) works only on this path. Instagram Login is the fallback only if the Instagram account is not linked to a Page. Claude flags the newer "Instagram API with Unified Login" (Meta landing page updated 16 Sep 2026, system-user tokens) as untested; ChatGPT does not mention it.
- **(B) Access: stay on Standard Access; no App Review and no Business Verification for your own accounts.** Meta's Instagram Platform Overview: "If your app only serves your Instagram professional account or an account you manage, Standard Access is all your app needs." ChatGPT agrees. Claude adds the catch to test first: Meta's App Modes doc says data created in Development mode, such as test posts, is visible only to role users, so check the first posts from a logged-out browser; if they are not public, Live mode needs a privacy URL and a Data Use Checkup but no App Review.
- **(C) Scheduling: local.** Instagram has no scheduling parameter in the Content Publishing API (not found) and containers expire after 24 h. Facebook Pages natively schedule feed and photo posts with `published=false` plus `scheduled_publish_time` (10 minutes to 30 days in the current guide). ChatGPT says Instagram "also supports scheduling" via the publishing API; no primary source for that was found by Claude.

Your "WOULD CHANGE THE ANSWER" conditions: owned-account publishing in development mode without review: **yes**, holds. Carousel missing from the API: **no**, carousels are supported. Page tokens needing periodic human re-login: **no**, unless invalidated by a password change, lost Page role, removed app or new Page security requirements.

## Your March 2026 evidence, corrected (Claude, dated)

- Graph v20.0: obsolete, removed 24 Sep 2026. Current: v26.0 (29 Jul 2026), v25.0 (expires 29 Jul 2028), v24.0, v23.0, v22.0, v21.0 (removed 21 Jan 2027). Target v25.0 now.
- Page text/photo via `/feed` and `/photos`, Reels via `/video_reels`, Instagram single image, Reels and Stories via container, poll, publish: confirmed.
- "No carousel": wrong. Carousels (images and videos mixed) have been in the API since the 15 Mar 2022 changelog entry. Meta's own docs disagree on the maximum: 10 in the guide with v25.0 examples, 20 in the one with v26.0 examples. Design for 10 and test 11+ with a real call.
- Insights HTTP 400: most likely deprecated metrics (impressions, plays, `video_views` removed in 2025; Page and post reach metrics removed June 2026 with the v26.0 launch and replaced by "viewers" and "views" metrics), unsupported metric/breakdown combinations, accounts under 100 followers, or Stories queried after 24 h.
- "App Review takes 2-8 weeks": not found in Meta's docs; Meta says it typically takes less than a week, often 2-3 days. The 2-8 weeks and a 20-day dashboard estimate come only from vendors. Irrelevant for your plan because you do not need review.
- "350 scheduled posts per Page": not found in any Meta source; the only documented limit is the time window.

## Build-first plan from the Claude report

1. Day 1: pin Graph v25.0, run `debug_token` on stored tokens, re-derive Page tokens and check "Expires: Never", complete Page Publishing Authorization (PPA) and 2FA on the Pages.
2. Publish one Page post and one Instagram image as the app and check public visibility from a logged-out browser.
3. Core path: local approval queue, then the Instagram carousel of 4:5 sRGB JPEGs with `alt_text` per slide, `location_id` and `is_ai_generated` on the parent.
4. Facebook Page photo and multi-photo posts; then first comment and moderation; then insights with current metrics only; then Reels and Stories via resumable upload.
5. Optional: Facebook native scheduling for offline windows; Unified Login or system-user tokens.

## Constraints that bite your 1080x1350 carousel

- Instagram feed images must be JPEG, at most 8 MB, aspect ratio between 4:5 and 1.91:1. 1080x1350 is exactly 4:5 and passes only just; a 3:4 image would be rejected. All carousel images are cropped to the first image's ratio. PNG is not accepted, so convert to sRGB JPEG locally.
- Instagram fetches images by URL ("must be hosted on a publicly accessible server at the time of the attempt"), so a local-first app needs a short-lived public URL per image; videos can use resumable upload from disk.
- Rate limits: Instagram says 100 API-published posts per 24 h in one place and 50 in another on the same page (read `content_publishing_limit` and trust it); 400 containers per 24 h; Facebook Reels 30 per 24 h; business-use-case limits 4800 x impressions. None matter at single-founder volume.
- Terms: Developer Policies require consent before publishing on someone's behalf (your per-post approval satisfies it) and no specific ban on automated posting to your own accounts was found. AI disclosure: `is_ai_generated=true` on Instagram containers (carousel parent only, since 22 Jun 2026) and on Facebook Reels; no parameter found for Facebook feed or photo posts. Make the AI toggle a required approval field.
- Not found: Instagram native scheduling, a scheduled-post cap, a test-user/test-app mode for Pages (only role types confirmed), exact Page insights permission names, whether Business Discovery needs review at Standard Access.
- Rust: no maintained crate tracks v25/v26 (`facebook_api_rs` 0.1.2 targets v23.0; others are years old). Hand-write typed `reqwest`/`serde` calls, the `oauth2` crate for loopback, the keychain for tokens.

## Where the ChatGPT report is weaker or wrong (HAND, uncited; sources panel: 0 scanned, 0 citations)

- Contradicts itself on carousels: says carousels up to 10 are supported, then that scheduling is "single-image/video, not carousels" and publishing "not carousels yet".
- Says the Content Publishing API supports scheduling and gives the limit as 25 posts per 24 h; Claude found no scheduling parameter and a limit of 100 (or 50) per 24 h per Meta's docs.
- Says alt text goes in the `caption` field for images; Claude cites a dedicated `alt_text` field (added 24 Mar 2025). Says Page tokens "auto-refresh" when the user token is refreshed; Meta says they simply carry no expiry date.
- Repeats "2-8 weeks" for App Review without a Meta source, and "reach/impressions replaced by views" only for Instagram, missing the June 2026 Facebook Page reach removal.
- Omits PPA/2FA, the Development-mode visibility catch, the JPEG/aspect-ratio rule, the public-URL requirement and the AI-label parameter. Agrees with Claude on v20.0 removal, Basic Display shutdown, and Standard Access sufficiency.

## Collection provenance and 5.0.11 / 5.0.12 behaviour seen live

- Claude (primary) collected natively (`complete-loaded-claude-document`) after one `research-not-finished`: 31771 B, 198 citations.
- ChatGPT: 5.0.12 opened the Sources and activity panel by itself, read "Sources scanned 0", and refused with `report-model-only-no-sources` once, then `report-unfinished-or-ambiguous` repeatedly, until the study stopped at the 40/40 coordinator limit. The page, read afterwards, held one complete report. HAND applied and is labelled; it is not a native pair.

All findings are model research from two reports. Two providers agreeing is not independent evidence. Default stays 4.3.23.
