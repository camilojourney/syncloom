# Syncloom Q8: local-first credentials, OAuth and scheduling (study main969-q8-auth-r2)

Study `main969-q8-auth-r2` on Phrona 5.0.12 (phrona.v2), Claude Research primary, ChatGPT Deep research challenger. Question file `questions/q8-local-first-auth-and-scheduling.md` passed unchanged, SHA256 `3bd00183162a71cf2aeeb9345d5e812c8b2a7b4f098c01a1725d801635e06874` (verified against `question-hashes.txt` before start).

## Provenance (read this first)

| Provider | Label | Conversation | Size | SHA256 (word for word) | Sources |
|---|---|---|---|---|---|
| Claude (primary) | NATIVE | https://claude.ai/chat/b107a9fe-dbd5-48f9-8979-0ab0061421f3 | 22377 chars / 22798 B | `dc9556bab04c41bc7dc8f88a31efe29f81fc95151cf9fc27047c3509f31b69f2` | 82 citations |
| ChatGPT (challenger) | NOT COLLECTED | no report | | | |

- Report files: `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q8-claude-NATIVE-report.txt`. Native Claude original also at `/Users/camiloslaptop/.local/state/phrona/durable-study-5.0.12-candidate/studies/main969-q8-auth-r2/reports/claude-dc9556bab04c41bc7dc8f88a31efe29f81fc95151cf9fc27047c3509f31b69f2.txt`.
- Outcome: `complete`. Phase `complete`, `defaultEligible: False`. Coordinator calls 24/40, native calls 0, known cost USD 0.068. Langfuse trace `0b32559c901db03e646d7ea60cb51509`, 137 events received, 0 pending.
- Collection failure codes recorded by 5.0.12: {"chatgpt": {}, "claude": {"research-not-finished": 1}}.

- ChatGPT Sources and activity panel: not applicable, no ChatGPT Send was made (challenger failed custody-uncertain, nothing sent).


## Which study this report covers

`main969-q8-auth-r2` is the approved re-run (Main, msg 142) of `main969-q8-auth`. The original `main969-q8-auth` stayed terminal and untouched: `incomplete / unsupported-ui`, **nothing sent to either provider**. Cause: its Claude page `claude.ai/new` held a draft equal (whitespace-normalised, 1923 chars) to Q7's approved question, so every paste failed with "unknown or mismatched draft". Q7 and Q8 had been started three seconds apart, so Q7's pasted text leaked into the other Claude tab's composer. I read the page read-only, matched it to Q7's question file, and cleared that leftover (select-all, Backspace) before the re-run.

## This study has ONE report: the ChatGPT challenger was not sent

- Claude (primary): NATIVE, 22798 B, 82 citations, collected after one `research-not-finished`.
- ChatGPT (challenger): `custody-uncertain`, **no Send, no conversation URL**. Phrona's own agent recorded that the ChatGPT composer held an unreadable draft ("Complete composer differs from approved question after paste" / "unknown or mismatched draft"). When I read that page afterwards it held Q8's own text with the Deep research mention. Most likely cause: this study and Pilaster `main973-q3-jobs` were started in the same second by a duplicate queue process of mine (a leftover process survived my `pkill` because its command line was `python3.15 queue.py`), so two paste sequences ran concurrently against the shared ChatGPT account. I cleared that own leftover draft after reading it. Per the rules, no ChatGPT Send was repeated or attempted outside Phrona.
- Result: `phase: complete`, `confidence: reduced`, `defaultEligible: false`. Only one provider answered, so there is no cross-check from ChatGPT. Treat the findings below as one model's research.

## Answer tied to the DECISION (credentials, authorization and scheduling for a one-person local-first Rust app)

Claude's recommendation:
- **Credentials: A, macOS Keychain.** One Keychain item per account holding the whole token bundle (access, refresh, expiry, Bluesky DPoP key). Use `keyring-core` 1.x plus `apple-native-keyring-store` 1.0.2 (6 Aug 2026, `keychain` feature); `keyring` 4.x removed the v3 feature flags and its own release notes tell apps to use `keyring-core`. Environment variables alone (C) cannot persist rotated tokens, so rule them out. Hold a process-wide lock across refresh, write, use. Sign the app and any launchd helper with the same identity, or Keychain ACL prompts can block unattended runs (stated as platform behaviour without a primary citation).
- **Authorization: D (loopback with PKCE) almost everywhere.** Google/YouTube accepts `http://127.0.0.1:port` on any port; TikTok desktop accepts 127.0.0.1 with a wildcard port and requires hex-encoded SHA-256 PKCE; Bluesky's atproto OAuth accepts loopback with port not matched; X requires an exact `http://127.0.0.1` callback (not the word `localhost`), so use a fixed port; LinkedIn docs say HTTPS but its own sample registers `http://localhost:8080`. Meta needs HTTPS outside development mode, so a one-time manual token exchange (F) is the pragmatic route for Facebook Pages, Instagram and Threads. Device code (E) exists only for Google, with a narrow scope set that excludes `youtube.upload`.
- **Scheduling: hybrid.** Native scheduling (G) only for YouTube (`privacyStatus=private` plus `status.publishAt`, never previously published) and Facebook Pages (`scheduled_publish_time`; the current Pages guide says 10 minutes to 30 days, the older Feed reference 10 minutes to 75 days, so plan for 30). A local launchd agent (H) for X, Instagram, Threads, LinkedIn, Bluesky and TikTok, with "post at approval time or re-approve" (I) as the safety valve.

Your "WOULD CHANGE THE ANSWER" conditions: native scheduling on most platforms (no, only two of the eight surfaces); a provider forbidding loopback (none found; Meta's HTTPS rule outside development mode is the only real obstacle, and it rests partly on a secondary source quoting Meta's 2018 notice).

## Evidence to know

- **Rotating refresh tokens (a lost write loses access):** X (staff say creating a new token invalidates the old one; refresh "stays valid for 6 months in theory"), Bluesky (refresh tokens "generally single-use"; public client session limited to 2 weeks, confidential up to 180 days), TikTok (24 h access, 365-day refresh, "returned refresh_token may be different"), and in effect Instagram/Threads (60 days, refresh only once the token is at least 24 h old; whether the old one is invalidated is undocumented). LinkedIn non-partners cannot refresh at all (re-run the browser flow before day 60). Google refresh tokens are reusable, but a "Testing" consent screen kills them after 7 days.
- **launchd:** `StartCalendarInterval` catches up on wake (coalesced into one event) but not after power-off; `StartInterval` misses intervals during sleep; Apple DTS says launchd guarantees nothing about power state when a job starts, so the job must check reachability. `pmset schedule wake` needs root. Power Nap, `IOPMSchedulePowerEvent` and the login-session rule were not verified from primary sources.
- **Idempotency:** no idempotency key found in any publish API. Use an intent record (approval id, revision hash, platform) written before the call and the remote id after it; verify-then-decide on restart. Instagram/Threads: persist the container `creation_id` and retry publish on the same container. Threads containers expire after 24 h; Threads allows 250 API posts per 24 h. X rejects duplicates with a 403 (forum evidence only). TikTok: persist `publish_id` and poll the status endpoint.
- **Crates:** `oauth2` 5.0.0 (21 Jan 2025), `security-framework` 3.7.0 (20 Feb 2026), `jacquard` 0.12.1 (26 Jun 2026; includes loopback OAuth for Bluesky); `atrium-api`/`bsky-sdk` and `reqwest` versions not confirmed.

## Your March 2026 claims, per Claude

- "Only Facebook Pages had native scheduling": corrected (YouTube also schedules natively; Facebook window now documented as 10 min to 30 days).
- "Instagram, Threads, X, LinkedIn had none": no contrary evidence; confirmed from secondary sources only for Threads, unverifiable from primary docs for the rest.
- "TikTok scheduling Business-only, 20 minutes to 10 days": corrected (TikTok's own blog says 15 minutes to 10 days, Business account for desktop scheduling); no scheduling field in the Content Posting API reference.
- YouTube private plus publish time: confirmed. Meta and Threads 60-day tokens: confirmed (Page tokens from a long-lived user token have no expiry). TikTok rotating refresh: confirmed.

## Limits

- Single model, no ChatGPT cross-check. Many official pages showed no last-updated date; several claims rest on forums and competitor guides (X rotation and duplicate rejection, no-scheduling on Instagram/Threads/X/LinkedIn, Meta localhost exemption).
- Not researched to primary level: Power Nap, IOPM events, LaunchAgent session scope, atrium/bsky-sdk/reqwest versions, Bluesky app-password refresh, LinkedIn native-PKCE enablement, Instagram/Threads loopback rules.

## Collection provenance and 5.0.11 / 5.0.12 behaviour seen live

- Primary Claude collected natively (`complete-loaded-claude-document`) after one `research-not-finished`. Coordinator 24 of 40 calls, known cost USD 0.068, trace received (137 events, 0 pending).
- Both Q8 failures were custody problems caused by concurrent new-chat preparation in the shared Claude and ChatGPT accounts: the original (Claude draft leaked from Q7) and the re-run (ChatGPT composer). Main has filed a phrona.v2 fix to serialize new-chat preparation. Note for the known-problems list: clearing our own leftover draft is safe and was needed twice here.

All findings are model research from two reports. Two providers agreeing is not independent evidence. Default stays 4.3.23.
