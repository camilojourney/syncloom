# Bluesky, X and Threads

Slice 5b adds `bluesky`, `x` and `threads` behind the same Platform boundary as LinkedIn/Mastodon. [Decision 0006](decisions/0006-social-platform-integrations.md) records primary-source checks and the [Q1 study](research/2026-10-30-q1-x-threads-bluesky/README.md) is preserved unchanged. Q1 is Claude-only, with reduced confidence. All proof is against independent local fake servers, not live accounts.

No real sign-in, post or paid API request is authorized by building this slice. Real use requires the captain's word for that exact action. No unattended scheduling, replies, engagement or English/Spanish account routing is included.

## Configuration

Run Rust through the nix shell in [development.md](development.md). `SYNCLOOM_DELIVERY` additionally recognizes `bluesky,x,threads`; unset grants nothing. Configuration does not grant delivery, and a delivery grant does not supply credentials.

| Variable | Default | Meaning |
|---|---|---|
| `SYNCLOOM_BLUESKY_BASE_URL` | `https://bsky.social` | Explicit PDS; HTTPS or loopback HTTP fake. Returned DID-document endpoints are never followed. |
| `SYNCLOOM_X_CLIENT_ID` | unset | Public native OAuth client; no client secret is used. |
| `SYNCLOOM_X_REDIRECT_URL` | `http://127.0.0.1:8793/x/callback` | Exact registered loopback HTTP URL, with a free explicit port. Live registration acceptance is unverified. |
| `SYNCLOOM_X_AUTH_BASE_URL` | `https://x.com` | Authorization base, HTTPS or loopback fake. |
| `SYNCLOOM_X_API_BASE_URL` | `https://api.x.com` | OAuth token and v2 API base, HTTPS or loopback fake. |
| `SYNCLOOM_THREADS_CLIENT_ID` | unset | Threads app id, not the general Meta app id. |
| `SYNCLOOM_THREADS_CLIENT_SECRET` | unset | Process environment only, never embedded or printed. Not suitable for a distributed native binary. |
| `SYNCLOOM_THREADS_REDIRECT_URL` | unset | Required exact HTTPS non-localhost hostname/path. No default host, DNS, certificate or listener is created. |
| `SYNCLOOM_THREADS_AUTH_BASE_URL` | `https://threads.com` | Consent URL base, HTTPS or loopback fake. |
| `SYNCLOOM_THREADS_API_BASE_URL` | `https://graph.threads.com` | Exchange/refresh and v1.0 publishing base, HTTPS or loopback fake. |

Keychain account keys are platform-prefixed hashes of the API base and, for OAuth, app id. Tokens, refresh tokens and the Bluesky app password live only in that credential item. No session JSON file is written. Sign-out invalidates in-flight credential commits so a delayed refresh cannot sign the account back in.

## Connections

The CLI reads app passwords/callback URLs from stdin, not arguments, environment variables or files. On a terminal, input is hidden and Enter submits it; piped input is bounded and read until EOF. `sign-in` prints the consent URL for X/Threads. Threads then waits for the full state-bearing callback URL on stdin; the same process must remain open. A chosen HTTPS redirect must preserve that URL for the founder to paste, without logging it. No redirect host has been selected or tested.

```sh
cargo run -p syncloom-cli -- connect bluesky sign-in --identifier <handle>
# Type the app password in the hidden terminal input, then Enter.
cargo run -p syncloom-cli -- connect x sign-in
cargo run -p syncloom-cli -- connect threads sign-in
cargo run -p syncloom-cli -- connect bluesky status
cargo run -p syncloom-cli -- connect x refresh
cargo run -p syncloom-cli -- connect threads sign-out
```

Protected HTTP operations use `X-Syncloom-Token` from `GET /v1/session` for every mutation; the existing exact Host/Origin policy applies. Secret inputs never appear in responses, events or fixed error messages.

| Operation | Body / result |
|---|---|
| `POST /v1/bluesky/sign-in` | `{ "identifier": "<handle>", "app_password": "<secret>" }`; connection or fixed error |
| `POST /v1/x/sign-in` | Start loopback PKCE; 202 with consent URL/deadline |
| `GET /v1/x/sign-in` | Wait for callback; connection or fixed sign-in error |
| `POST /v1/threads/sign-in` | Start external consent; 202 with URL/deadline, or 503 `platform_not_configured` |
| `POST /v1/threads/complete` | `{ "callback_url": "<full HTTPS callback URL>" }`; validates exact redirect and one-time state before exchange |
| `GET /v1/{platform}/connection` | Local connection state, no remote request |
| `DELETE /v1/{platform}/connection` | Delete credentials, signed_out |
| `POST /v1/{platform}/refresh` | Explicit refresh, connection or fixed sign-in error; never a post |

Bluesky uses the maintained ATrium API SDK pinned at 0.26.0, not handwritten XRPC record/session wire types. It creates a session once, persists both JWTs, reads access expiry from the JWT and explicitly refreshes through refreshSession. X asks for tweet.read, tweet.write, users.read, media.write and offline.access; token responses supply expiry and replacement refresh tokens. An expired access token never triggers hidden renewal during publish. Threads exchanges a short-lived token for a long-lived token (60 days nominal, returned expires_in is authoritative); refresh is allowed only after 24 hours and before expiry. Failed refresh retains the prior stored credentials.

## Publishing and cost

Draft with `--channel bluesky`, `--channel x` or `--channel threads`, review and approve as before. Bluesky supports 300 graphemes/3000 bytes and PNGs up to a conservative 1 MB; X uses a conservative 280-unit count and PNGs up to 5 MB. Threads uses a conservative 500-byte text limit. Conservative counts can refuse remotely valid content; they never truncate it.

Rendered PNG delivery rechecks exact approved byte hashes and carries the image alt text. X uses current v2 initialize, per-id append/finalize and nested media metadata. Unfinished PNG processing does not create a post. Threads images are refused, not silently dropped or uploaded somewhere else; PDFs stay local. A Threads text container must report FINISHED before publication; an id and allowlisted Threads permalink are required for posted.

X dry runs include `cost_estimate`, with currency USD, integer `microusd`, link/metadata components, pricing date and `kind: estimate_not_actual_charge`. As checked 2026-10-31:

| Post | Estimate |
|---|---|
| Plain text | USD 0.015 (15000 micro-USD) |
| Plain text and PNG alt metadata | USD 0.020 (20000 micro-USD) |
| URL-like text | USD 0.200 (200000 micro-USD) |
| URL-like text and PNG alt metadata | USD 0.205 (205000 micro-USD) |

URL-like tokens include bare domains and explicit schemes; ambiguous domain-like text may conservatively overestimate. These are not a billing receipt or a spending cap. Confirm current console prices before any authorized real use. A failed or uncertain request may still incur a charge, especially after metadata succeeds.

```sh
cargo run -p syncloom-cli -- publish <piece_id> --expected-revision <revision> --dry-run
# After reviewing the returned estimate, explicitly accept its exact amount:
cargo run -p syncloom-cli -- publish <piece_id> --expected-revision <revision> --accepted-cost-microusd 15000
```

HTTP uses the same optional `accepted_cost_microusd` field. Without matching confirmation, granted X delivery returns 409 `cost_confirmation_required` before reserving an intent. The review page displays the dated estimate and a separate **Accept estimated cost** form before enabling Publish. Browser consent is bound to the exact revision and full estimate for the server session; changed content or pricing and server restart require acceptance again. It never approves or publishes by itself. The intent retains the estimate for contained, posted, failed and unknown outcomes across restarts. Repeating a posted/unknown intent does not send again and retains its original estimate. No actual-charge amount is invented.

## Open questions and real-use prerequisites

- Which HTTPS non-localhost Threads redirect host and callback handoff should the founder use? This slice implements validation/exchange, not hosting.
- Which public media URL host, if any, should later Threads images use? Nothing uploads to public storage here.
- Will X accept the exact loopback URI registered for this app? Only fake consent is proven.
- Has platform policy cleared any future unattended AI-assisted publishing? This slice is explicit, human-approved publishing only.

## Fake proof

`cargo test -p syncloom-server --test social_delivery --test social_channels` exercises protected HTTP drafting, sign-in, render/hash approval, cost confirmation, posting, containment, restart/repeat and session/Origin denial. `cargo test -p syncloom-core --test social_sessions --test social_boundaries --test social_rules` covers refresh, state, wire shapes and platform failures. Each fake binds its own loopback port with only its platform's endpoints. No test signs into or posts to a real platform.
