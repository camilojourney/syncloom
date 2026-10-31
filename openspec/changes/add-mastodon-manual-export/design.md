# Design

References: security, data, integrations, llm.

## Context

See proposal.md and [decision 0004](../../../docs/decisions/0004-mastodon-manual-export.md). Existing Platform receives only text; visual assets already have revision-bound hashes. LinkedIn owns a reusable loopback OAuth mechanism embedded in its adapter.

## Goals / Non-Goals

Preserve existing stores and public LinkedIn names. No live credentials, real publishing, Q6 readers, carousel uploads, implicit threads or video generation.

## Decisions

- Use a thin reqwest adapter against documented Mastodon v1 apps/statuses and v2 instance/media, rather than a broad SDK: the boundary needs only these operations and existing HTTP policy already disables redirects. Read the official apps, oauth, media, statuses and instance docs; Q5 is single-provider guidance, not independent proof.
- Configure one explicit instance origin; reuse BaseUrl validation and reject paths. Reject arbitrary per-request instance URLs: the operator's configuration is the outbound allowlist, with HTTP loopback solely for the fake.
- Split CLI routing from setup/output and command request construction; split Mastodon limits, media preparation and status creation into named steps. Shared credentials own absent/expired-token refusal, and Studio owns atomic claim, attempt marker and export receipt writes; this removes copied policy and keeps each decision inspectable.
- Extract shared OAuth callback/state/listener primitives from LinkedIn rather than duplicate its guards. Bind loopback, check exact Host, bound completion time/body, log only fixed kinds, and store tokens solely through TokenStore; app registration secrets stay in memory and are discarded after completion.
- Store Mastodon tokens under an instance-derived account and represent absent expiry as absent, rather than invent a remote lifetime. Old LinkedIn token JSON remains readable.
- Extend PostRequest with optional owned PNG bytes and alt text, not a URL or file path; Studio verifies approved assets before releasing the lock. LinkedIn visual dispatch remains contained; Mastodon uploads only the first-slide PNG with first-slide text as alt text.
- Fetch instance configuration per publish and reject over-limit content without truncation. Bound asynchronous media polling within the platform deadline, rather than polling forever or retrying status creation; `Idempotency-Key` supplements the durable local guard and does not replace it.
- Recheck current approval and durable intent under the attempt lock, rather than trust the snapshot between reservation and dispatch. This closes a concurrent-call/stale-approval window while keeping network I/O outside the lock.
- Export any approved channel to an explicit Reddit or TikTok handoff, plus offer a generic manual drafting channel. Keep export outside Platform: it must not fabricate Posted or require a network delivery grant.
- Write fixed bundle filenames below the store's exports root with content hashes and a completion manifest, then record an export-specific intent and content-free event. Refuse symlinks and damaged repeated bundles; an interrupted incomplete bundle is retryable, unlike an uncertain network status.
- Retain old containment and idempotent reservation behavior from the migration ledger; reject the old client-wide retries and optimistic `succeeded` interpretation because they can duplicate posts or mislabel outcomes.

- Use the channel table's name and bounds in the provider instruction, rather than reuse a LinkedIn-only prompt for every channel. The fixed voice/safety instruction remains shared and thoughts stay exclusively in the user message; fake-model regression tests are the available baseline, not evidence of live model quality.

## Security boundaries

Only the local operator acts. API Host guards cover every new route; OAuth state is checked before exchange, callback output is fixed and contains no CORS. Tokens never reach piece files, receipts, events, stdout or errors. Export uses the same id/revision/approval/asset owners, writes no caller-chosen path, and logs only ids/hashes/statuses. Fake content and generic client headers contain no personal data.

## Risks / Trade-offs

- Mastodon instance variation: missing limits or malformed responses fail closed. Draft fit is advisory; the instance is authoritative at send time.
- Media can be orphaned after a partial failure: never claim a status exists; unknown outcomes stay guarded and no automatic cleanup is attempted.
- Export is not proof of posting: notes require the operator to check subreddit rules and later TikTok video requirements.
- Live instance compatibility remains unproven: all execution evidence uses a loopback fake; real consent requires separate authorization.

## Migration Plan

Additive channel/status variants and optional token expiry preserve existing records. No bulk rewrite or destructive migration. Rollback uses the earlier binary with pre-change data backups; never read new-channel records with an older binary.
