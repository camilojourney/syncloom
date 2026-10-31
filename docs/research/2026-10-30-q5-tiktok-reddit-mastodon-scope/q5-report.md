# Syncloom Q5: TikTok, Reddit, Mastodon and Pinterest publishing scope

Study `main969-q5-scope` on Phrona 5.0.13 (phrona.v2, primary-only, release `010b331631398ef46eff22ac`), Claude Research primary, `challenger: none`. Question file `questions/q5-tiktok-reddit-mastodon-scope.md` passed unchanged, SHA256 `36aaccf55bb61b8021a98ce3f94fd694c2222546b3621a35aefaead1e1c46c8d` (verified against `question-hashes.txt` before start).

## Provenance (read this first)

| Provider | Label | Conversation | Size | SHA256 (word for word) | Sources |
|---|---|---|---|---|---|
| Claude (primary) | NATIVE | https://claude.ai/chat/b9e37486-b2f3-4bf4-9780-1241c94f83fd | 22489 chars / 23033 B | `13ce580e1d566aadca542df93eff2619bbdbfe6647a419597cef6f70ca50c10b` | 110 citations |
| ChatGPT (challenger) | NOT REQUESTED | challenger not requested (captain: ChatGPT replaced while model-only) | | | |

- Report files: `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q5-claude-NATIVE-report.txt`. Native Claude original also at `/Users/camiloslaptop/.local/state/phrona/durable-study-5.0.13-candidate/studies/main969-q5-scope/reports/claude-13ce580e1d566aadca542df93eff2619bbdbfe6647a419597cef6f70ca50c10b.txt`.
- Outcome: `complete`. Phase `complete`, `defaultEligible: False`. Coordinator calls 17/40, native calls 0, known cost USD 0.079. Langfuse trace `cc08fcb260c5dfb9d0d0fd4557b94bb7`, 84 events received, 0 pending.
- Collection failure codes recorded by 5.0.13: {"chatgpt": {}, "claude": {"research-not-finished": 2}}.

- ChatGPT Sources and activity panel: not applicable. Challenger not requested (captain: ChatGPT replaced while model-only); `challenger: none`, no ChatGPT page was opened or sent. Single-provider result with reduced confidence.


## Single-provider result, reduced confidence

Challenger: **not requested (captain: ChatGPT replaced while model-only)**. 5.0.13, `challenger: none`, one Claude Research Send. Nothing here is cross-checked by a second provider; it is model research from one report (110 citations), not independent evidence. Not a native pair, not default-eligible. A challenger-only pass is allowed only if Claude's report leaves a deciding fact uncited (per Main); the deciding facts below are cited to primary pages or flagged as secondary by the report itself.

## Answer tied to the DECISION (which of TikTok, Reddit, Mastodon, Pinterest belong in the publishing scope)

**Only Mastodon belongs, as an API channel.** The others: manual export or skip.

| Platform | Can an unaudited solo app publish publicly? | Verdict | Effort |
|---|---|---|---|
| Mastodon | Yes, no app review (per-instance OAuth app registration) | Build API publish | about 2-4 dev-days, very low upkeep |
| Reddit | No: new public API requests stop 31 Oct 2026, RSS ends 13 Nov 2026, public access ends March 2027 | Manual paste and read in the browser; no API | about 0.5 day (export helper) |
| TikTok | No: unaudited Direct Post is SELF_ONLY and only to private accounts | Skip until short video exists; then manual handoff, optionally inbox/draft upload (video.upload) | manual about 0.5 day; inbox 4-6 days plus 1-3 weeks review |
| Pinterest | No on Trial (Pins visible only to the creator); yes after Standard review | Skip (no evidence the audience is there) | 4-6 dev-days plus review |

Your "WOULD CHANGE THE ANSWER" conditions: (1) a platform whose API cannot publish for an unaudited solo app, so manual only: **met for Reddit, TikTok and Pinterest Trial**, not Mastodon. (2) Audience evidence that a platform matters to the founder's professional audience: **none found** for Pinterest; Mastodon's audience is small (about 734,000 monthly active accounts as of 14 Oct 2026 per Mastodon's statistics API as reported by Postory, down from a 2.6M peak in Nov 2022) and concentrated in open-source, security and privacy circles, so treat it as a cheap add-on, not a priority channel.

Recommended build list from the report: ship Mastodon (megalodon 1.1.1 or a thin `reqwest` client; read per-instance limits at runtime from `GET /api/v2/instance`; split more than 4 images into a thread; behind the approval queue; about 3 days) and add one generic manual-export target (about 1 day) covering Reddit now and TikTok later; use Reddit only by hand and do not apply for API access; re-evaluate TikTok when video ships; leave Pinterest out.

## Per-platform facts (Claude, dated)

- **Mastodon:** text, up to 4 images or one video/audio attachment, polls, content warnings, replies and threads; asynchronous media upload (202); own statuses with counts, no impressions metric found. Tokens "will not expire automatically" (no refresh rotation). Limits: 300 requests per 5 minutes per account, 30 media uploads per 30 minutes; defaults 500 characters and 4 attachments, which vary per instance (read at runtime; a maintainer confirmed on 19 May 2025 that stock Mastodon has no support for a different character limit). No sandbox needed. Rust: `megalodon` 1.1.1 maintained; `mastodon-async` 1.3.2 stale.
- **Reddit:** Responsible Builder Policy since 11 Nov 2025 ("you must request access and get explicit approval"); developers report few requests accepted since; Devvit is not a general posting API. The free tier (100 queries per minute for approved non-commercial use) is moot for a new app. Broadcasting finished posts into subreddits is the textbook spam pattern (Reddiquette's informal 1-in-10 norm; Reddit's spam policy sets no ratio). Rust `roux` 2.2.15, "not in active development".
- **TikTok:** Content Sharing Guidelines say clients "must not be limited to test applications" and name "a utility tool to help upload contents to the account(s) you or your team manages" as unacceptable, so an own-accounts studio is expected to fail the Direct Post audit. Access token 24 hours, refresh token 365 days (store the returned refresh token). Photo posts up to 35 images. Unaudited clients post SELF_ONLY, at most 5 users per 24 hours, and the account must be private (`unaudited_client_can_only_post_to_private_accounts`). App review "several days to two weeks" officially, 13 days to six weeks in field reports. Sandbox: up to 10 target users, no public posts. `is_aigc` exists on Direct Post. No native schedule parameter; no creator comment API. Rust crates effectively unmaintained.
- **Pinterest:** image, video and 2-5 image carousel Pins; no text-only posts; access tokens 30 days, continuous refresh token 60 days; Trial access Pins visible only to the creator (1,000 requests per day, 300 writes per day); Standard needs a video of the OAuth flow even for a sole user; review time unpublished; "the end user must choose each Pin to be published" (compatible with your approval flow); since 14 Sep 2026 new apps are denied reading other users' boards and Pins.

## March 2026 evidence, corrected

TikTok audit and easier inbox upload: confirmed, with a sharper finding (the audit rejects own-account tools). 24-hour access and 365-day refresh tokens: confirmed. 35-image photo posts: confirmed. Unaudited posts forced private: confirmed and stricter (public accounts are blocked entirely). Sandbox private only: confirmed. "Mandatory AI label since August 2025": imprecise (labels on realistic AIGC required earlier; Community Guidelines AIGC update dated 13 Sep 2025 per a secondary source; date `is_aigc` was added not found). "Bluesky covers federated social": half true (different protocols; Bluesky mobile MAU 10.4M in June 2026, down 27.2% year on year per Similarweb via TechCrunch).

## Gaps the report names

Not found: TikTok creator comment API, the date `is_aigc` was added, whether inbox upload works from an unaudited app; Pinterest comment API and audience data for software and AI professionals; Mastodon per-instance limits and activity for fosstodon, hachyderm and infosec.exchange and any measured migration to Bluesky; Reddit sandbox and re-verified token lifetimes. Several figures are third-party (TikTok audit-duration reports, Postory's Mastodon statistics, Pinterest Trial approval time); Reddit's dates are stated plans that "can change".

## Collection provenance

- Claude (primary): collected natively (`complete-loaded-claude-document`) after two transient `research-not-finished`: 23033 B, 110 citations. Phase `complete`, 17 of 40 coordinator calls, known cost USD 0.079 (cost known), Langfuse trace `cc08fcb260c5dfb9d0d0fd4557b94bb7` with 84 events received, 0 pending.
- Started 19:04 local as the third study on 5.0.13 after a fresh Claude page check (empty composer) and a stagger after the previous Send; one Claude Send, no resend.

All findings are model research from two reports. Two providers agreeing is not independent evidence. Default stays 4.3.23.
