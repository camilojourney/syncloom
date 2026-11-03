# Second-user prerequisites

This page records what a later separate local installation would require. It is not an invitation, a distribution, or permission to proceed.

The selected boundary is staged separate local installations: each person would run their own copy. There is no hosted broker, shared service, or multi-tenant app. The current stage is narrower. The owner is the only user, on macOS, on a machine and login they control. A second user is not authorized.

[Vision](vision.md) still describes one operator. This page does not change that agreement and does not approve live posting, registration, hosting, spending, publication, credential setup, or app or callback changes.

## Requirements before another user

These are requirements against the selected boundary. None of them is approval to invite someone.

1. Their own macOS machine, or at least their own macOS login. Not the owner's session. Non-macOS secret storage is not supported: `KeychainStore` reports itself unsupported off macOS. Two people on one OS login are not isolated.
2. Their own data directory and secret custody. Do not copy the data directory or Keychain items. Production tokens use the macOS Keychain. The default service and the one-slot-per-configuration rule are in the [README environment table](../README.md#environment). A slot is a provider configuration, not a Syncloom profile.
3. Loopback only. The server binds `127.0.0.1` and rejects other hosts. No tunnel, reverse proxy, shared service, or broker. `GET /v1/session` gives the process mutation token to an allowed local client. That token is one OS user's CSRF capability, not a user account. See [the studio session boundary](studio.md#http-session-handshake).
4. Their own eligible provider account and their own app permissions. No owner password, app secret, or API key.
5. Account-binding and effective-grant guards retained and proved on a commit the owner explicitly accepts, before another user. The current code includes those guards, as described below. Their presence does not authorize a second user.
6. Per-provider redirect, review, and secret eligibility, checked for that person's app and account. Code support is not eligibility, and a local fake is not a live approval.
7. A named owner of any X credits before X could be attempted. Reported pay-per-use charges attach to the app's billing owner. Who would pay, and whether one public client id may be shared, is unresolved. It is not a terms clearance.
8. No country, account-type, production-quota, or universal-provider promise. None is established.

## Current guards and remaining limits

The account-binding and effective-grant repair is included in this code. Its owning change records are [account-bound approval](../openspec/changes/bind-approval-to-account/design.md) and [effective posting grants](../openspec/changes/fix-effective-posting-grants/design.md). These guards close the earlier code gaps; they do not establish second-user readiness or live provider eligibility.

- Approval binds the reviewed revision and assets to an explicitly reviewed provider-account target. A stale approval form cannot silently approve the account connected later. Switching accounts requires fresh approval; legacy approvals are not backfilled. Durable dispatch history retains its original target and uncertainty.
- Canonical provider identities, not slot keys or display handles, bind delivery. These include the Bluesky DID, X account id and Mastodon instance/account id; app-scoped LinkedIn and Threads subjects retain their app authority, and Meta retains its selected Page identity. A configured slot still holds one connection, not multiple Syncloom profiles. Reconnecting a second Mastodon account on the same instance replaces that slot, but cannot redirect an earlier account's approved send.
- Effective posting permission is checked on the same loaded credential used for media and posting. LinkedIn requires exact `w_member_social`; X and Mastodon distinguish text from media permission. Threads uses provider debug evidence, not requested scopes. Historical Meta/Threads requested strings are unverified until reconnect establishes evidence. See the platform audit and recovery rules in [Effective posting permissions](posting-permissions.md).
- Missing permission, unverified permission and unconfirmed identity report `cannot_post` and send no publishing write. After successful X or Threads credential rotation with a failed identity read, the actual new bundle is saved with its expected identity unconfirmed, not the old broader credential. A later publish may retry a read-only identity check on that same token: a match confirms it, a mismatch signs out, and another failure sends no write. Failed exchange before issuance preserves prior credentials. Reconnect uses the existing protected sign-in flow and returns to review without approving or publishing.
- Integration behavior is proved with MemoryStore and stateful loopback fakes, not real accounts. That evidence does not prove live eligibility, provider acceptance, remote revocation or worldwide availability.
- Meta stores the Page token with no local expiry. Public photo and Instagram delivery stay `UnconfiguredMedia`. See [Meta integration](meta.md).
- Sign-out deletes the local credential. It does not call a provider revoke. Bluesky also keeps the app password in that local item until sign-out. Local sign-out is not provider revocation.

LinkedIn go-live stays separately withheld pending an explicit credential and callback instruction. This page does not give one.

## Providers

Already-supported in code means an adapter or local handoff exists. It does not mean a second person is eligible, and it does not mean a live app is approved.

Later candidates, not an approved rollout: Bluesky with that person's own app password, Mastodon on an instance they configure, and manual export. Even then: local sign-out only, no remote revoke, and no country promise. Manual export is a local handoff, never a post. See [Mastodon and export](mastodon-and-export.md) and [Bluesky, X and Threads](social-platforms.md).

LinkedIn, Meta (Facebook Page and Instagram), and Threads stay out until their own registration, secret, redirect, and review questions are separately approved by a person with that authority. Threads requires an exact HTTPS non-localhost redirect and does not host that redirect. Meta public media hosting is unconfigured. X additionally needs the named credit owner and a confirmed callback. Live loopback registration is unverified.

No YouTube, Twitch, Kick, Reddit, TikTok, or Pinterest API adapter is in this repository. Manual export notes for some of those targets are not API access.

## Reported policy, not clearance

Provider-policy statements below are the 2026-10-30 research records. They were not rechecked for this page. They are not legal clearance, newly verified terms, or permission to use an account.

Start at the [research index](research/INDEX.md). The cards that own the claims are [Q1](research/2026-10-30-q1-x-threads-bluesky/README.md), [Q2](research/2026-10-30-q2-linkedin-publishing/README.md), [Q3](research/2026-10-30-q3-meta-facebook-instagram/README.md), [Q5](research/2026-10-30-q5-tiktok-reddit-mastodon-scope/README.md), and [Q8](research/2026-10-30-q8-local-first-auth-and-scheduling/README.md). Decisions [0002](decisions/0002-linkedin-integration-decisions.md), [0004](decisions/0004-mastodon-manual-export.md), [0005](decisions/0005-meta-integration.md), and [0006](decisions/0006-social-platform-integrations.md) record what was chosen for the founder build. They are not second-user acceptance.

- Bluesky: Q1 describes app passwords as the researched path for this product. It does not establish that a distributed second-person install may use them.
- Mastodon: Q5 does not establish a central review. A chosen instance can still refuse loopback or dynamic apps. That is an instance rule, not a universal Mastodon claim.
- X: Q1 did not find official loopback support. Q8 says `http://127.0.0.1` is required and `localhost` is not. [Decision 0006](decisions/0006-social-platform-integrations.md) leaves exact registered loopback acceptance unproven. Pay-per-use pricing is reported; it is not a billing receipt or a spending authorization.
- LinkedIn: Q2 did not find whether the standard flow accepts `http://localhost` or `http://127.0.0.1`. Native PKCE was reported as not self-serve. Neither gap authorizes go-live.
- Threads and Meta: Q1's Threads sample requires HTTPS and not localhost. Q8's "no provider forbids loopback" is weaker than that sample, and Q8 itself treats Meta HTTPS outside development mode as the real obstacle. Left unresolved on purpose. Q3 reports Standard Access, without App Review or Business Verification, when an app serves only the operator's own Instagram professional account or an account they manage. That report does not establish eligibility for a different person's account.
- Q1 and Q8 disagree on how sure loopback is. This page does not pick a winner.

## Not authorized by this page

- Inviting or shipping a build to a second person.
- Landing this code or any security repair. Included guards are not landing authorization.
- Creating or changing a LinkedIn, Meta, Threads, or X app; setting a callback; using an existing credential; or proving a live redirect.
- Registering a domain, opening a tunnel, hosting a callback, or publishing a privacy policy.
- Spending, including letting another person consume the owner's X credits.
- Making a real post or a live provider call.
- Adding non-macOS secret storage.
- Approving a broker if a required provider cannot be completed locally.
- Treating always-on or phone access as in scope. Phone reach remains the unresolved Q-ACCESS question in the [research index](research/INDEX.md).

Until those permissions exist, the second user does not exist as an operator. The safe statement is: one owner, one Mac login, loopback only, no copied Keychain, no shared app secret.
