# Syncloom Q4: publishing video and Shorts to your own YouTube channel from a local Rust app

Study `main969-q4-youtube` on Phrona 5.0.12 (phrona.v2), Claude Research primary, ChatGPT Deep research challenger. Question file `questions/q4-youtube.md` passed unchanged, SHA256 `0361cdae3af233cfa1339582c306333cc90323c28071e0eddd2e25dd5b7c296e` (verified against `question-hashes.txt` before start).

## Provenance (read this first)

| Provider | Label | Conversation | Size | SHA256 (word for word) | Sources |
|---|---|---|---|---|---|
| Claude (primary) | NATIVE | https://claude.ai/chat/03589a1c-dac5-4375-b747-2cb237a57092 | 23425 chars / 23933 B | `61e9241478cbd6390794797ef7076c2a6e087cc7de3e89bd2a01ba03845c5221` | 89 citations |
| ChatGPT (challenger) | HAND | https://chatgpt.com/c/6ae4e8a8-3c98-83e9-8f7b-dae9c0339d8f | 9460 chars / 9536 B | `a36b47247720652cc935c0adbb058bbe8e222cfd1a8ab36be9a658659f358539` | sourcesScanned 0; panel: Citations 0 ("No sources detected"); 3 links inside the report body |

- Report files: `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q4-chatgpt-HAND-report.txt`; `/Users/camiloslaptop/.firstmate-homes/phrona-lab/data/phrona-syncloom-studies-20261030/q4-claude-NATIVE-report.txt`. Native Claude original also at `/Users/camiloslaptop/.local/state/phrona/durable-study-5.0.12-candidate/studies/main969-q4-youtube/reports/claude-61e9241478cbd6390794797ef7076c2a6e087cc7de3e89bd2a01ba03845c5221.txt`.
- Outcome: `other` ("The original coordinator model request failed; no replacement model or fresh allowance was selected"). Phase `incomplete`, `defaultEligible: False`. Coordinator calls 37/40, native calls 0, known cost USD 0.113. Langfuse trace `23cabdc692501319f2d9d3d39708723e`, 205 events received, 0 pending.
- Collection failure codes recorded by 5.0.12: {"chatgpt": {"report-unfinished-or-ambiguous": 10, "report-model-only-no-sources": 1}, "claude": {"research-not-finished": 2}}.
- HAND reads (ChatGPT) were zero-click and zero-input (inputEvents {cg.get('inputEvents','n/a')}, clicks {cg.get('clicks','n/a')}) after browser-identity, exact-URL, question-association and frame-lineage checks, taken from the Deep research iframe. HAND reports never count toward native acceptance.
- ChatGPT Sources and activity panel (5.0.12 opened it natively; I made no click): Sources scanned 0, Citations 0 ("No sources detected"), connector sources scanned 0. Panel text: `Sources | Activity · 6m |  | Citations |  | No sources detected |  | Sources scanned · 0 |  | No sources scanned |  | Connector sources scanned |  | No connector sources scanned`.


## Outcome (named)

**Coordinator model request failed: Claude five-hour limit.** The study ended `incomplete` (reason `other`, "The original coordinator model request failed; no replacement model or fresh allowance was selected") after 37 coordinator calls, because Phrona's own Claude coordinator hit the Claude five-hour usage limit. This is not a provider-page limit on the research chats. Both Sends had already happened (Claude and ChatGPT) and nothing was re-sent. Claude's report was collected natively before the limit (so no Claude HAND read was needed); ChatGPT's was HAND-read afterwards, zero-click, from its own page. Not a native pair, not default-eligible.

## Answer tied to the DECISION (publishing video and Shorts to your own YouTube channel through the official API)

The two reports agree on the mechanics but disagree on the two answer-changing risks, and Claude's treatment is the more careful one.

- **Claude (NATIVE, 89 citations): build the API path as an A/C hybrid, not B.** Run the OAuth client as External, "In production" but not submitted for verification (Google's personal-use exemption, fewer than 100 users), so refresh tokens do not die after 7 days. Upload resumably with `privacyStatus=private`, read the status back, publish by `publishAt` or by hand, file the YouTube API Services audit in parallel, and keep Studio upload (C) as the fallback if the first upload is locked private. OAuth verification (B) is not needed for an owner-only app and does not lift any upload lock.
- **ChatGPT (HAND, uncited): Testing mode works but needs weekly re-login; verification (B) is the long-term answer.** It says Publish "requires verification by Google's team ... days to several weeks", which conflates OAuth verification with the YouTube audit and misses the "In production, unverified" personal-use route.

Your "WOULD CHANGE THE ANSWER" conditions:
1. **Refresh tokens expiring weekly in Testing mode: confirmed** by both (Google's OAuth doc: External user type with publishing status "Testing" gets a refresh token expiring in 7 days, YouTube scopes not exempt). Claude's mitigation: "In production" unverified.
2. **Uploads locked private until an audit a one-person app cannot pass: unresolved, and the two reports differ.** Claude: Google's own docs contradict each other. The July 2020 revision entry and the YouTube Help "locked as private" page say uploads from unverified API projects created after 28 July 2020 are restricted to private and cannot be appealed, while the current `videos.insert` reference (updated 15 Oct 2026) says "Videos uploaded from unverified API projects are not restricted to private viewing mode", with no revision entry. Every first-hand report Claude found (porjo/youtubeuploader issue #86, a 2026 PyPI package warning) still shows locking. ChatGPT quotes only the current sentence and treats it as settled. The honest reading is Claude's: test with a throwaway private upload before relying on either. A solo audit pass is reported possible (two personal-use developers in issue #86; about 20 days at the fast end, 3-6 months elsewhere), but no 2026 first-hand approval was found.

## Quota and cost (both agree, Claude more precise)

- Since 1 Jun 2026 `videos.insert` costs 1 call from its own bucket of 100 per day; `search.list` the same; 10,000 units per day for everything else; reset at midnight Pacific (Quota Calculator updated 15 Oct 2026). Revision history 4 Dec 2025: upload cost "approximately 1600 units to approximately 100 units". A typical day with 2 uploads, thumbnails, captions, playlist adds, stats and replies is about 1,265 units. Quota is not a constraint.
- Per-call: `thumbnails.set` 50, `captions.insert` 400 (update 450), `playlistItems.insert` 50, `comments.insert` 50, `comments.setModerationStatus` 50, list calls 1.

## Capabilities (Claude, with sources and dates)

- Shorts have no API flag; classification is by square or vertical shape and at most 3 minutes (uploads on or after 15 Oct 2024). "#Shorts" is not required. ChatGPT says "#Shorts" must be included "per YouTube guidelines"; Claude found no such current requirement.
- Altered or synthetic disclosure: `status.containsSyntheticMedia` (revision history 30 Oct 2024); both agree. `publishAt` works only on a never-published private video and needs `privacyStatus=private` on update. Captions need timing info (the `sync` parameter was removed Apr 2024). Thumbnails need a phone-verified channel; max 50 MB since 14 Sep 2026.
- Community and image posts: not found (no API; bulletins removed 2020); both agree. Own statistics: `videos.list` (and `videos.batchGetStats` from 3 Jun 2026) and the Analytics API with a 48-72 hour lag. Comments: read, reply, moderate are supported; `markAsSpam` was deprecated in 2023.
- Terms: authorised data other than statistics may be stored at most 30 days; derived-metrics storage policy changed 4 May and 1 Jun 2026; do not shard projects for quota; no clause found banning human-approved uploads; "inauthentic content" YPP rename on 15 Jul 2025. Plan a 30-day refresh job and a disconnect button that revokes and purges. Projects can lose access after 90 days idle per issue #86 (anecdotal).
- Rust: `google-youtube3` 7.0.0+20251222 (resumable upload supported; newer fields such as `brandPartner` need raw JSON), `yup-oauth2` 12.1.2 for the loopback flow, or hand-written `reqwest` resumable upload (about 200 lines, 256 KiB chunk multiples). Both reports name `google-youtube3`.

## Your March 2026 evidence, corrected (Claude)

Refresh-token exchange upload works but dies in 7 days in Testing; `categoryId` 22 valid; `privacy public` is risky (possible lock, bypasses your approval gate), so use private plus `publishAt`; "#Shorts" is unnecessary; whole-file-in-memory upload should be a resumable chunked stream; the 100-user cap is right but irrelevant; Shorts up to 3 minutes is right; "10,000 units a day" is partly outdated (separate 100-call buckets since June 2026).

## Where the ChatGPT report is weaker (HAND, uncited; sources panel 0 scanned, 0 citations; 3 links in the body)

- Treats the forced-private question as settled from one sentence; misses the 2020 revision entry, the Help page and the field reports.
- Conflates OAuth verification with the YouTube API Services audit and says "publishing the app requires verification"; does not mention the unverified personal-use route.
- Says verification is "ideal" for stable long-term access, which Claude shows is unnecessary for one user; no audit timelines, no 30-day data rule, no thumbnail phone-verification or 50 MB change.
- Agrees on 1-unit uploads, the quota calculator figures, `containsSyntheticMedia`, no community-post API, and `google-youtube3` 7.0.0.

## Collection provenance

- Claude collected natively (`complete-loaded-claude-document`) after two transient `research-not-finished`: 23933 B, 89 citations.
- ChatGPT: 5.0.12 recorded `report-model-only-no-sources` once and `report-unfinished-or-ambiguous` ten times; the page, read afterwards, held one complete report (9536 B). The HAND read passed identity, URL, question-association (1 turn, matches the approved question) and frame-lineage checks, with zero input events and zero clicks. The Sources and activity panel had been opened by 5.0.12 itself and read "Sources scanned 0", "No sources detected".

All findings are model research from two reports. Two providers agreeing is not independent evidence. Default stays 4.3.23.
