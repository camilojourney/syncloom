# ADR-0002: LinkedIn personal-profile integration: self-serve Share on LinkedIn, Keychain tokens, loopback sign-in

**Date:** 2026-10-30
**Status:** Proposed
**Supersedes:** none
**OpenSpec changes:** [add-linkedin-integration-slice-2](../../openspec/changes/archive/2026-10-30-add-linkedin-integration-slice-2/) (slice 2: LinkedIn sign-in, the platform trait, the LinkedIn adapter, truthful dispatch outcomes; archived into `openspec/specs/`)
**Agreed:** partly - the captain agreed on 2026-10-30 that the integrations are rebuilt in Rust after deep research (decision 0001, D2, [the captain's words](sources/captain-words.md)) and accepted the build order that makes LinkedIn the first (vision 6.2). The specific choices below (product, scope, token store, sign-in flow, API version) are not yet put to the captain
**Recommended:** the choices in the Decision section, by the slice 2 worker from the Q2 and Q8 research; the one-time developer-app steps and the first live post wait for the captain's word
**Built:** built on branch `fm/syncloom-v2-slice2-linkedin-20261030` (the commit is recorded at the merge); proven against a local fake of LinkedIn's endpoints and the real macOS Keychain under a test-only service name
**Live-proven:** no - no LinkedIn developer app exists, no real sign-in and no real post has been made; both need the captain's word (vision 3.4)

The last four fields are different claims and stay apart: a recommendation is not agreed, an agreed decision is not built, a landed change is not proven against the real service.

---

## Context

Slice 1 left publishing contained: an approved piece becomes one durable intent and nothing leaves the laptop. The build order (vision 6.2) makes LinkedIn the first platform integration, "after its research answer (question Q2) and the local-first credentials answer (Q8): truthful dispatch of an approved piece, tested against a fake of the platform's API before any real post". Both answers arrived on 2026-10-30 ([q2-linkedin-publishing](../research/2026-10-30-q2-linkedin-publishing/README.md), [q8-local-first-auth-and-scheduling](../research/2026-10-30-q8-local-first-auth-and-scheduling/README.md)). The old Python adapter (`src/platforms/linkedin.py` of the Holus Social API, read in [social-api-research-questions](../research/2026-10-30-social-api-research-questions/README.md)) posted text, image, PDF, poll and video with version header 202602 and marked pieces `published` when the API had only accepted the job.

---

## Decision

- **Surface: the founder's personal profile only**, through the self-serve "Share on LinkedIn" product (`w_member_social`) plus "Sign In with LinkedIn using OpenID Connect" (`openid profile`, for the member id the Posts API needs as author). Company pages are out of scope: they need the vetted Community Management API, which requires a registered legal organization and a dedicated app.
- **API: the versioned REST Posts API** (`POST /rest/posts`, `LinkedIn-Version: 202610`, `X-Restli-Protocol-Version: 2.0.0`), text posts first. Slice 3 renders local images and PDF documents only. Uploads wait for the first live text post and a later integration change, with each content type tested on the app as Q2 advises.
- **Credentials: the macOS Keychain**, one generic-password item under the service `syncloom` and the account `linkedin`, holding the token bundle (access token, expiry, member id, scope). Written through `keyring-core` 1.0.1 and `apple-native-keyring-store` 1.0.2 (`keychain` feature). Never in files, logs, traces or test output. The app's client id and client secret are read from the environment only while signing in.
- **Sign-in: authorization code over a loopback redirect** (`http://127.0.0.1:<port>/linkedin/callback`, registered exactly in the developer app) with a one-time random `state` checked in constant time. Two flows exist: `standard` (client secret, the self-serve default) and `native_pkce` (S256 code challenge, no secret, LinkedIn's `/oauth/native-pkce/authorization`, which LinkedIn must enable per app). The default is `standard` because the Q2 report found native PKCE is not self-serve.
- **No refresh tokens**: the self-serve tier has none, so the founder re-consents in the browser before the 60-day expiry; the studio reports `expired` and `consent_required` instead of guessing.
- **Delivery is an explicit grant**: `SYNCLOOM_DELIVERY=linkedin` names the platforms that deliver; without it every intent stays contained (vision 3.4, the old `HOLUS_PERSONAL_DELIVERY_GRANT` kept in spirit).
- **Truthful outcomes, one request per attempt**: the intent is written as `posting` before the call and as the outcome after it (`posted` only with LinkedIn's post id, else `refused`, `consent_required`, `rate_limited`, `unreachable` or `unknown`); a `posted` intent is never sent again; a `posting` or `unknown` intent is never retried blindly (Q8: no publish API offers an idempotency key, so verify-then-decide).
- **Scheduling is not built** in slice 2. Q8 recommends local scheduling (launchd) with "publish at approval time or re-approve" as the safety valve; it gets its own change.

Behavior belongs to the OpenSpec specs (`dispatch-outbox`, `platform-credentials`, `linkedin-integration`, `studio-api`, `studio-cli`, `provenance-log`); this record names no requirement.

---

## Rationale

| Option | Pros | Cons | Why rejected / chosen |
|--------|------|------|-----------------------|
| Share on LinkedIn, personal profile, REST Posts API (chosen) | Self-serve, no review; `w_member_social` covers text, image, multi-image, document, poll, video per LinkedIn's own pages | No analytics, no reading own posts (`r_member_social` closed); 60-day tokens | Chosen: the only route a one-person app has today |
| Community Management API for the company page | Page posting, analytics | Vetted: registered legal entity, business email, dedicated app; approval at LinkedIn's discretion | Rejected for now; revisit if the captain registers an entity |
| Partner tier with 365-day refresh tokens | Unattended refresh | "Limited set of partners"; no self-serve route found | Rejected: not available |
| Manual paste, no API | Nothing to maintain | Defeats the studio's publish step | Rejected |
| Keychain (chosen) over an encrypted file or the environment | OS-protected, survives restarts, per-item ACL | The ACL binds the item to the storing binary's code hash: each binary and each rebuild asks once in a GUI session and fails headless; tests need a test-only service | Chosen (Q8); a file would be a second secret store to protect; environment variables cannot persist rotated tokens |
| Keychain item open to every application (`-A` style ACL) | No prompt after a rebuild, one consent for the CLI and the server | Any process of the user reads the token without a prompt; gives up the only per-process guard | Rejected; the optional remedy is signing the binaries with a stable identity (a captain step, not built) |
| Standard authorization-code flow by default, native PKCE as an option (chosen) | Works for a self-serve app; PKCE ready if LinkedIn enables it | Client secret handled by the app on the laptop; two code paths differing in three fields | Chosen; PKCE-only would leave the captain unable to sign in until LinkedIn enables it; sending PKCE on the standard endpoint is reported rejected (`invalid_client`) |
| Automatic retry on 5xx or timeout | Fewer manual retries | No idempotency key: a retry can post twice | Rejected: one request per attempt, `unknown` needs the founder's check |
| Signed-in means delivery enabled | One less setting | Sign-in alone would turn the first publish into a real post | Rejected: vision 3.4 asks for an explicit grant |

---

## Scope and boundaries

Covers the LinkedIn personal-profile text post, the sign-in and token storage, the delivery grant and the dispatch outcomes. Preserved: slice 1 behavior when no delivery is granted (contained intents), the old repository (read-only reference). Not covered: company pages, images, documents, polls, video, scheduling, analytics, reconciliation of an `unknown` outcome (LinkedIn offers no self-serve way to list the member's own posts; the founder checks the feed), and the one-time steps that need the captain: creating the developer app, registering the redirect URL, the first real sign-in and the first real post.

---

## Identities

OpenSpec CLI 1.15.0 with the Fleet baseline 1.0.0 (see decision 0001); Code skill 5.0.0 with the security, data and integrations references; Rust toolchain 1.98.1; crates `keyring-core` 1.0.1, `apple-native-keyring-store` 1.0.2, `base64` 0.22, `getrandom` 0.3 (versions as resolved in `Cargo.lock`); LinkedIn API version header `202610` (Active until 15 Oct 2027 per the Q2 report's migration table); OpenID Connect `userinfo` at `/v2/userinfo`. The commit that built slice 2 is the tip of branch `fm/syncloom-v2-slice2-linkedin-20261030`; the merge records it on `main`.

---

## Evidence and gaps

- [q2-linkedin-publishing](../research/2026-10-30-q2-linkedin-publishing/README.md): product, scope, Posts API, token lifetime, no refresh tokens, PKCE not self-serve, version table, terms (Developer AI Policy satisfied by approve-before-publish), no sandbox.
- [q8-local-first-auth-and-scheduling](../research/2026-10-30-q8-local-first-auth-and-scheduling/README.md): Keychain and the `keyring-core` crates, loopback redirects, no idempotency keys, the intent-record pattern, scheduling hybrid for later.
- [social-api-research-questions](../research/2026-10-30-social-api-research-questions/README.md): what the old adapter did and the questions as sent.

Gaps: whether LinkedIn accepts an `http://127.0.0.1` (or `http://localhost`) redirect URL on the standard flow is "not found" in the research; the first live sign-in settles it. The Keychain prompt for a rebuilt binary was inferred from a probe in a shell without a GUI session (`-128 User canceled`, no dialog); that a GUI terminal shows the standard dialog and that "Always Allow" settles it is Apple's documented behavior, not observed here. The Posts API `commentary` text rules (reserved characters, escaping) were not asked and are unverified; the first live post settles them. The ChatGPT challenger for Q8 was never sent, so Q8 rests on one provider. Nothing here has run against api.linkedin.com.

---

## Consequences

**Positive:**
- A real post can only happen after an explicit grant, a signed-in Keychain token and an approved, revision-matched piece, and it is reported with LinkedIn's own post id and URL or not at all.
- The platform trait and the token store trait let slice 5 add platforms without touching the dispatch rules.

**Negative / Trade-offs:**
- A browser re-consent every 60 days with no unattended refresh.
- An `unknown` outcome (timeout, 5xx, crash mid-call) needs the founder to look at the feed; LinkedIn's self-serve tier cannot list the member's posts.
- The client secret passes through the environment on the laptop at sign-in time.
- `syncloom` and `syncloom-server` are two binaries, so each gets its own Keychain consent, and every rebuild asks again (once, in a GUI terminal); headless runs of a rebuilt binary fail with `internal_error` until that consent is given.

---

## References

- [Vision](../vision.md) sections 3 and 6.2, [migration ledger](../migration.md), [README](../../README.md)
- Specs: `openspec/specs/` (`platform-credentials`, `linkedin-integration`, and the modified `dispatch-outbox`, `studio-api`, `studio-cli`, `provenance-log`); the archived change: [add-linkedin-integration-slice-2](../../openspec/changes/archive/2026-10-30-add-linkedin-integration-slice-2/)
