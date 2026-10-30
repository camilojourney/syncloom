# Design

References: security, data, integrations.

Decision records: `docs/decisions/0002-linkedin-integration-decisions.md` (product, scope, Keychain, sign-in flow, delivery grant, one request per attempt) and `docs/decisions/0001-syncloom-v2-rebuild-decisions.md` (D2: integrations rebuilt in Rust after research; D6 local-only landing). Research: `docs/research/2026-10-30-q2-linkedin-publishing/` and `docs/research/2026-10-30-q8-local-first-auth-and-scheduling/`. Slice 1 design: `openspec/changes/archive/2026-10-30-add-studio-slice-1/design.md`.

## Decisions

- The platform boundary is one trait, `Platform::publish(PostRequest) -> Future<Result<Posted, PostFailure>>`, with `Posted { post_id, url }` and `PostFailure::{ConsentRequired, Refused, RateLimited, Unreachable, Unknown}`; the studio keeps a map from platform name to adapter and applies one deadline (`Settings::platform_deadline`, 30 s). Reject calling the LinkedIn adapter directly from the studio: slice 5 adds six platforms behind the same rules.
- Delivery is granted per platform by `SYNCLOOM_DELIVERY` (comma-separated platform names, validated at startup); the config layer inserts an adapter only for granted platforms, so an ungranted platform cannot be reached by any code path. Reject "signed in means enabled": a sign-in would silently turn the next publish into a real post.
- The intent is the idempotency guard. Before the call the intent is written as `posting` (attempt count and time); after it as the outcome. `posted` is final and never sent again. `posting` or `unknown` found on a later publish is answered as `unknown` without a request (verify-then-decide, Q8). `contained`, `refused`, `consent_required`, `rate_limited` and `unreachable` allow one new attempt on the next publish. Reject automatic retries on 5xx, 429 or timeouts (Code integrations rule 2): LinkedIn offers no idempotency key, so a retry can post twice; the founder retries explicitly.
- The store lock is released during the network call (write `posting`, unlock, call, lock, write the outcome). Invariant: a `posting` record means an attempt whose outcome this process does not know, so it is never retried blindly. Reject holding the file lock across the call: it blocks the CLI and the server on each other for up to the deadline inside async handlers.
- `posted` needs LinkedIn's post id from the `x-restli-id` response header; a 2xx without it is `unknown`. The post URL is `https://www.linkedin.com/feed/update/<urn>/`. Reject deriving "posted" from a 2xx alone (vision 3.4: accepted is not published).
- Outcome mapping, one path per documented failure: 401 is `consent_required` (invalid or expired token), 429 `rate_limited`, any other 4xx `refused`, 5xx `unknown`, a connection failure before the request is sent `unreachable`, a timeout or any other transport failure `unknown`. An absent or locally expired token is `consent_required` with zero requests.
- Tokens live in the macOS Keychain as one generic-password item (service `syncloom`, overridable by `SYNCLOOM_KEYCHAIN_SERVICE` so tests use `syncloom-test-<id>`; account `linkedin`) holding a JSON bundle: access token, expiry, member id, scope, obtained-at. Written through `keyring-core` plus `apple-native-keyring-store` with the `keychain` feature, through a `TokenStore` trait so tests can also use an in-memory store. Reject `keyring` 4.x (its own release notes point apps to `keyring-core`) and an encrypted file (a second secret to protect).
- The token type wraps the secret in a newtype whose `Debug` prints `[redacted]` and that has no `Display`; only the Keychain store serializes it. Responses, events and logs carry statuses and expiry, never the token.
- The Keychain binds an item to the code hash of the binary that created it (probe on this machine: the same bytes at another path read the item; a different build got `-128 User canceled` with no prompt from a non-GUI shell and `-25244` on delete). So `syncloom` and `syncloom-server` are two binaries with two consents, and every rebuild is a new binary: in a GUI terminal macOS asks once per binary and build ("Always Allow" settles it); headless the read fails and the studio answers `internal_error` with log kind `platform_failure`. Accepted for this slice: the founder runs on a laptop with a GUI session and rebuilds rarely. Rejected: an item ACL open to every application (`security add-generic-password -A` style), which gives up the only per-process guard the Keychain adds; signing the binaries with a stable self-signed identity is the optional remedy (a one-time step for the captain, not built here). Tests follow: the CLI test process never reads the item, it checks presence with `/usr/bin/security find-generic-password` and cleans up with `delete-generic-password`; only the core credentials test reads and writes through one binary.
- A redirect port that cannot be bound when a sign-in begins is reported as `linkedin_not_configured`, whose message names the port; it is the same remedy class (fix the environment) and no listener is left behind.
- Sign-in is the authorization-code flow over a loopback redirect. The redirect URL (`SYNCLOOM_LINKEDIN_REDIRECT_URL`, default `http://127.0.0.1:8791/linkedin/callback`) must be `http://127.0.0.1:<port>/<path>` or `http://localhost:<port>/<path>` with no credentials, query or fragment; the listener binds `127.0.0.1` on that port because LinkedIn registers redirect URLs exactly and its own sample uses `localhost:8080`. A 32-byte random `state` is compared in constant time; a mismatch refuses the callback and exchanges nothing. Two flows: `standard` (client secret in the token request, the default) and `native_pkce` (S256 challenge, `code_verifier`, no secret, `/oauth/native-pkce/authorization`). Reject PKCE-only: the Q2 report found native PKCE must be enabled by LinkedIn per app and the standard endpoint is reported to reject a code verifier. Reject sending PKCE and the secret together: reported `invalid_client`.
- The callback listener is an axum router on the bound port that accepts the registered path only, answers a fixed HTML page that echoes nothing, and hands the first `code`/`state`/`error` to the waiting sign-in through a oneshot channel; it stops when the sign-in finishes or its deadline (`Settings::sign_in_deadline`, 300 s) passes. Reject a hand-written HTTP parser: axum is already in the workspace and parses browsers' requests correctly.
- After the exchange the adapter calls `GET /v2/userinfo` and keeps `sub` as the member id for the author URN; scopes requested are `openid profile w_member_social`. Reject `/v2/me` (`r_liteprofile`, superseded by OpenID Connect).
- The sign-in is two steps in the core (`begin` returns the authorization URL, `wait` returns the connection or the failure) so the API can answer the URL first and wait in a second request, and the CLI prints the URL before blocking. One pending sign-in at a time; a second `begin` is `sign_in_in_progress` until the pending one finishes or expires. Reject one blocking operation: the caller needs the URL before the browser can be opened.
- The client id and secret are read from `SYNCLOOM_LINKEDIN_CLIENT_ID` and `SYNCLOOM_LINKEDIN_CLIENT_SECRET` only when a sign-in begins; publishing needs only the Keychain token. Reject storing the secret in the Keychain: one more item, one more command, for a value needed about six times a year.
- Base URLs for LinkedIn (`SYNCLOOM_LINKEDIN_AUTH_BASE_URL`, `SYNCLOOM_LINKEDIN_API_BASE_URL`) follow the slice 1 rule (`https://<host>` or `http://127.0.0.1:<port>`), whose one owner moves from the Claude adapter to `config::BaseUrl`; the Claude provider keeps its entry point. The fake LinkedIn in `tests/support/` and the `fake_linkedin` example serve both bases on one loopback address.
- Error codes join the one core table: `linkedin_not_configured` (503), `sign_in_in_progress` (409), `sign_in_not_started` (409), `sign_in_denied` (403), `sign_in_state_mismatch` (403), `sign_in_timed_out` (504), `sign_in_rejected` (502). A failed delivery is not an error envelope: the publish operation did its work and the outcome is data (`status`), as `contained` already is.
- Provenance gains `dispatch_attempted` and `dispatch_finished` (with `outcome` and, when posted, `post_id`); sign-in and sign-out leave no provenance event because events are per piece; the request log records the operations.
- Kept: `contracts/studio-api/errors/*.json`, because the fixtures are the independent contract that a test compares with the server's responses (slice 1 ruling).
- Kept: `crates/syncloom-core/examples/fake_linkedin.rs` includes `tests/support/fake_linkedin.rs` by path, as `fake_claude` does, because the fake must run as a process for CLI runs without shipping test code in the library.

## Old behavior read at `3efcd35` (kept or changed)

| Old owner | Behavior | Here |
|---|---|---|
| `containment.py:16-32`, `content.py:132-150` | contained unless `HOLUS_PERSONAL_DELIVERY_GRANT` | Kept as `SYNCLOOM_DELIVERY=<platforms>`: an explicit grant per platform. |
| `content.py:717-723` | piece marked `published` when the API only accepted the job | Changed: `posted` only with LinkedIn's post id; otherwise the real outcome. |
| `outbox.py:60-104` | one intent per operation, piece and revision | Kept; the intent now records attempts and the outcome. |
| Holus Social API `src/platforms/linkedin.py` | `ugcPosts`/`rest/posts` with version 202602; token from a stubbed OAuth flow | Changed: `rest/posts` with 202610; token from a real loopback sign-in into the Keychain. |

## Security boundaries

- Who may act: any local process, as in slice 1 (loopback, unauthenticated). The browser consent is the founder's; a real post needs the delivery grant, a Keychain token and an approved revision.
- What is checked: the redirect URL form, the delivery grant names and the base URLs at startup; the `state` in constant time on the callback; approval and revision before any attempt; the intent status before any request; token expiry before any request.
- What is logged: the request log's fixed route labels, statuses and codes; events with ids, hashes, statuses, outcomes and the post id. Never the token, the client secret, the authorization code, the state, the text or the member id.
- Where secrets live: the access token in the Keychain item only; the client id and secret in the process environment during sign-in only; the state and code verifier in process memory for one sign-in. Provider redirects are not followed; the callback listener answers a fixed page.
- Tests: the Keychain tests use a test-only service name and delete their item; the fake LinkedIn never leaves the loopback.

## Data

Stored: the intent gains `attempts`, `attempted_at`, `finished_at`, `post_id`, `post_url` (all optional, so slice 1 records read back). The Keychain bundle holds the founder's own member id and token. Time stays RFC 3339 UTC `Z`. Events are `object_action` with ids and statuses, no content.

## Risks / Trade-offs

- Nothing here has run against LinkedIn; the redirect URL acceptance and the `commentary` text rules are unverified until the first live run.
- A crash between the request and the outcome write leaves `posting`; the founder must check the feed.
- Keychain access is per binary and per build (see Decisions): the first `linkedin status` after a rebuild asks once in a GUI terminal and fails headless. Signing the binaries with a stable identity would remove the prompt; not done here.

## Migration

None in data. Slice 1 intents lack the new optional fields and read back unchanged. The ledger rows for the Social API client and containment flip in `docs/migration.md`.
