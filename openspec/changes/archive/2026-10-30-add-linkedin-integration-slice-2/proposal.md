## Request

Quoted verbatim from the captain, 2026-10-30, as relayed in the task brief (spelling is the captain's). Omitted: firstmate's context and specification paragraphs, which are supervisor text, not the captain's words.

> aslo build synchroloom rebuild it all as we mentioned and also the integrations wtih al lthe social media apis to make deep research onthem too ... use hte openspec last version with the instruciont conf think about what is next and start building all thos features.

## Approval

pending

New capability requested: platform-credentials

New capability requested: linkedin-integration

Duplicate kept: error status, code and message in `contracts/studio-api/errors/` and in the core error table plus the server status map, because the fixtures are the independent contract a test compares (slice 1 ruling, extended with seven codes).

## Why

Slice 2 (vision 6.2): the first platform integration, LinkedIn, after the Q2 and Q8 answers. One approved, revision-matched piece becomes exactly one Posts API request with a truthful outcome; the token lives in the Keychain. Decision: `docs/decisions/0002-linkedin-integration-decisions.md`.

## What Changes

- Core: `Platform` and `TokenStore` traits; the LinkedIn adapter (sign-in, Keychain token, `POST /rest/posts`, outcomes); the delivery grant `SYNCLOOM_DELIVERY`; dispatch states.
- API: `POST`/`GET /v1/linkedin/sign-in`, `GET`/`DELETE /v1/linkedin/connection`; publish answers carry `post_id` and `post_url`; seven error codes.
- CLI: `syncloom linkedin sign-in|status|sign-out`.
- Proof against a local fake of LinkedIn; never api.linkedin.com.

## Capabilities

### New Capabilities

Checked `openspec list --specs` (the seven slice 1 specs) and `docs/migration.md` (Social API client, containment grant).

- `platform-credentials`: token storage, connection states.
- `linkedin-integration`: sign-in, request shape, outcomes.

### Modified Capabilities

- `dispatch-outbox`, `studio-api`, `studio-cli`, `provenance-log`.

## Impact

New dependencies: `keyring-core`, `apple-native-keyring-store`, `base64`, `getrandom`; `axum` joins the core for the callback listener.

## Open questions

- Does LinkedIn accept an `http://127.0.0.1` redirect URL on the standard flow?
- Are reserved characters in `commentary` escaped by the app or refused by LinkedIn?
- How does the founder resolve an `unknown` outcome when LinkedIn cannot list the member's posts?
- Which slice owns scheduling (Q8)?
