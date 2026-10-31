# syncloom.v2

From-scratch Rust rebuild of Syncloom (formerly Holus), a solo founder's thought-to-content studio: one written thought becomes platform drafts the founder reviews and approves before an explicit publish. The old Python app lives in the separate `syncloom` repository and is not part of this one. What Syncloom is and is not: [docs/vision.md](docs/vision.md). What moved from the old app and what did not: [docs/migration.md](docs/migration.md).

## Layout

- `crates/syncloom-core/` - the rules: thought intake, drafting through a model provider, platform rules, review transitions, the dispatch outbox (contained, or delivered through a granted platform with the truthful outcome), the provenance log, the file store and the Keychain token store. The Claude adapter lives here behind the provider trait; the LinkedIn and Mastodon adapters behind the platform trait; manual export is a separate local handoff, never a post.
- `crates/syncloom-server/` - the loopback HTTP API and Rust-rendered browser studio (Axum), binary `syncloom-server`.
- `crates/syncloom-cli/` - the command line, binary `syncloom`; it calls the same core functions and returns the same errors.
- `contracts/` - fixtures the server tests against (`contracts/studio-api/`, owned by the capability `studio-api`).
- `openspec/` - specs and changes; `openspec/specs/` is the only specification authority.
- `docs/research/` and `docs/decisions/` - the studies behind the product and the decisions they served. Start at [`docs/research/INDEX.md`](docs/research/INDEX.md) and [`docs/decisions/README.md`](docs/decisions/README.md).
- `scripts/` - record checkers carried over from invoz.v2 ([docs/development.md](docs/development.md) says why they stay Python).

## Run and check

OpenSpec baseline, toolchain and how to re-check them: [docs/development.md](docs/development.md). `cargo` is reached through nix on this machine: `nix shell nixpkgs#cargo nixpkgs#rustc nixpkgs#clippy nixpkgs#rustfmt -c sh -c '<command>'`.

Server (needs `ANTHROPIC_API_KEY` in the environment to draft through Claude):

```sh
cargo run -p syncloom-server            # binds 127.0.0.1:8790, override with SYNCLOOM_ADDR
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

Open **http://127.0.0.1:8790/** in the laptop browser to see thoughts and pieces, edit a draft, render phone-sized PNG/PDF previews, approve the exact revision and hashes, and explicitly publish through the existing outbox. Edits open on a separate screen so unsaved text cannot be mistaken for an approved draft. [Browser studio and HTTP session protection](docs/studio.md).

Command line, same core, same errors:

```sh
cargo run -p syncloom-cli -- thought --text "<the thought>" --channel linkedin_text
cargo run -p syncloom-cli -- show <piece_id>
cargo run -p syncloom-cli -- approve <piece_id> --expected-revision <revision>
cargo run -p syncloom-cli -- publish <piece_id> --expected-revision <revision> --dry-run
cargo run -p syncloom-cli -- publish <piece_id> --expected-revision <revision>   # contained, or posted when SYNCLOOM_DELIVERY grants the platform
cargo run -p syncloom-cli -- linkedin sign-in     # prints the authorization URL, waits for the browser, stores the token in the Keychain
cargo run -p syncloom-cli -- linkedin status      # signed_out | awaiting_consent | signed_in | expired, from the Keychain and the clock
cargo run -p syncloom-cli -- linkedin sign-out    # deletes the token
```

HTTP clients first call `GET /v1/session` and pass its `token` once in `X-Syncloom-Token` for POST, PATCH and DELETE. `GET /v1/pieces?after=<last_piece_id>` discovers twenty saved pieces per page. All routes reject foreign Host/Origin and cross-site browser requests, with no CORS opening. The direct-core CLI needs no HTTP token.

The API has the same four LinkedIn operations: `POST /v1/linkedin/sign-in` (202, the URL), `GET /v1/linkedin/sign-in` (waits, 200 the connection), `GET /v1/linkedin/connection`, `DELETE /v1/linkedin/connection`.

Development fake Claude provider, to drive success and every provider error without a key:

```sh
cargo run -p syncloom-core --example fake_claude -- --mode ok     # ok|empty|slow|401|429|500, prints its address
SYNCLOOM_CLAUDE_BASE_URL=http://127.0.0.1:<port> ANTHROPIC_API_KEY=fake cargo run -p syncloom-server
```

Development fake LinkedIn, serving the authorization, token, userinfo and Posts endpoints on one loopback address; point both LinkedIn base URLs at it and use a test-only Keychain service:

```sh
cargo run -p syncloom-core --example fake_linkedin -- --post-mode posted   # --auth consent|deny|wrong-state, --token ok|400|401, --post-mode posted|noid|slow|401|403|422|429|500
SYNCLOOM_LINKEDIN_AUTH_BASE_URL=http://127.0.0.1:<port> SYNCLOOM_LINKEDIN_API_BASE_URL=http://127.0.0.1:<port> \
  SYNCLOOM_LINKEDIN_CLIENT_ID=fake SYNCLOOM_LINKEDIN_CLIENT_SECRET=fake SYNCLOOM_KEYCHAIN_SERVICE=syncloom-test-dev \
  SYNCLOOM_DELIVERY=linkedin cargo run -p syncloom-cli -- linkedin sign-in   # open the printed URL with curl -L to play the browser
```

Specs and records:

```sh
openspec validate --all --strict --no-interactive
python3 scripts/check-change-shape.py openspec/changes/<change> [--request-source FILE]   # config.yaml rules that validate does not enforce
python3 -I scripts/check-research-records.py
python3 -I scripts/test_check_research_records.py   # the check's own tests
```

## Environment

| Variable | Default | Meaning |
|---|---|---|
| `SYNCLOOM_ADDR` | `127.0.0.1:8790` | server bind address, must be on `127.0.0.1` |
| `SYNCLOOM_DATA_DIR` | `$HOME/.syncloom` | where pieces, outbox intents and the provenance log are stored |
| `ANTHROPIC_API_KEY` | none | Claude key, read only from the environment, never logged |
| `SYNCLOOM_CLAUDE_BASE_URL` | `https://api.anthropic.com` | provider URL; `https` or `http://127.0.0.1:<port>` only |
| `SYNCLOOM_CLAUDE_MODEL` | `claude-sonnet-5-5` | the pinned Claude Sonnet model id (unverified until the live check runs) |
| `SYNCLOOM_DELIVERY` | none | comma-separated platforms allowed to deliver (`linkedin,mastodon`); unset keeps every intent contained; an unknown name refuses startup |
| `SYNCLOOM_KEYCHAIN_SERVICE` | `syncloom` | the Keychain service name of the token items (account `linkedin` or instance-derived `mastodon:<hash>`); tests use `syncloom-test-<id>` |
| `SYNCLOOM_MASTODON_INSTANCE` | none | explicitly allowed instance origin; HTTPS, or loopback HTTP for the fake only |
| `SYNCLOOM_MASTODON_REDIRECT_URL` | `http://127.0.0.1:8792/mastodon/callback` | free loopback callback port registered per instance |
| `SYNCLOOM_LINKEDIN_CLIENT_ID` | none | the developer app's client id, read only when a sign-in begins |
| `SYNCLOOM_LINKEDIN_CLIENT_SECRET` | none | the developer app's client secret, read only when a sign-in begins, never logged |
| `SYNCLOOM_LINKEDIN_REDIRECT_URL` | `http://127.0.0.1:8791/linkedin/callback` | the redirect URL registered in the app; `http://127.0.0.1:<port>/<path>` or `http://localhost:<port>/<path>` only |
| `SYNCLOOM_LINKEDIN_AUTH_FLOW` | `standard` | `standard` (client secret) or `native_pkce` (S256, once LinkedIn enables it for the app) |
| `SYNCLOOM_LINKEDIN_AUTH_BASE_URL` | `https://www.linkedin.com` | authorization and token endpoints; `https` or `http://127.0.0.1:<port>` only |
| `SYNCLOOM_LINKEDIN_API_BASE_URL` | `https://api.linkedin.com` | `userinfo` and Posts endpoints; `https` or `http://127.0.0.1:<port>` only |

## Local images and carousels

[Visual assets](docs/visual-assets.md) documents the pinned local browser/font, operations and failure codes. No browser is downloaded or bundled.

```sh
cargo run -p syncloom-cli -- render <piece_id> --expected-revision <revision> --template midnight \
  --slide "Build with evidence." --slide "Review the exact bytes."
cargo run -p syncloom-cli -- view <piece_id> --kind image > /tmp/piece.png
cargo run -p syncloom-cli -- view <piece_id> --kind carousel > /tmp/piece.pdf
cargo run -p syncloom-cli -- approve <piece_id> --expected-revision <rendered_revision> --image-sha256 <image_sha256> --carousel-sha256 <carousel_sha256>
```

The API exposes `POST /v1/pieces/{piece_id}/render` and `GET /v1/pieces/{piece_id}/assets/image|carousel`. Approval verifies both SHA-256 hashes at the rendered revision. Text/template changes and every rerender clear approval. LinkedIn visual publish stays contained, even with a delivery grant; its uploads wait for the first live text post. Mastodon can deliver the approved first-slide PNG with alt text when explicitly granted; PDF carousels remain local export files.

## Mastodon and manual handoff

[Mastodon and export](docs/mastodon-and-export.md) documents instance-scoped OAuth, Keychain tokens, PNG publishing with alt text, and local Reddit/TikTok bundles. The channel table now includes `mastodon` and `manual_export`. Mastodon registration and publishing have been proven against a local fake only; a real sign-in or post requires separate captain authorization. Q6 is imported research only, not a reading integration.

```sh
cargo run -p syncloom-cli -- thought --text "<thought>" --channel manual_export
cargo run -p syncloom-cli -- export <piece_id> --expected-revision <revision> --target reddit
```

Export requires current approval, produces text and optional PNG/PDF files plus platform notes under the data directory, and records `exported`, never `posted`.

## Going live with LinkedIn

Nothing in this repository has talked to LinkedIn. No developer app, client id or secret exists on the build machine. The one-time steps are the captain's (vision 3.4):

1. Create a LinkedIn developer app for the personal profile and add the self-serve products **Share on LinkedIn** and **Sign In with LinkedIn using OpenID Connect**.
2. Register the redirect URL exactly as syncloom uses it (`http://127.0.0.1:8791/linkedin/callback`, or the value of `SYNCLOOM_LINKEDIN_REDIRECT_URL`). Whether LinkedIn accepts a loopback `http` URL on the standard flow is unverified (decision 0002, gaps); if it refuses, `http://localhost:<port>/...` is the next thing to try.
3. Export `SYNCLOOM_LINKEDIN_CLIENT_ID` and `SYNCLOOM_LINKEDIN_CLIENT_SECRET` in the shell that runs `syncloom linkedin sign-in`, open the printed URL, consent. The token lands in the Keychain; the secret is needed again only at the next sign-in (tokens last 60 days, there is no refresh).
4. Set `SYNCLOOM_DELIVERY=linkedin` for the process that should deliver. Approve a piece and publish it; the answer is `posted` with LinkedIn's post id and URL, or the real failure. Check the feed for the first post.

The Keychain binds the token item to the binary that stored it. The first `linkedin status` from `syncloom-server` after the CLI signed in, or from any rebuilt binary, asks once in a GUI terminal ("Always Allow"); a headless process gets `internal_error` instead. Signing the binaries with a stable identity would remove that prompt; it is not done here.
