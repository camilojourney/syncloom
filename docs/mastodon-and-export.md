# Mastodon and manual export

Built behind the existing approval and outbox boundaries. Proven only against a local fake. No real instance registration, sign-in or post has been attempted; those actions require the captain's separate authorization.

## Mastodon

The `mastodon` channel drafts concise text. Its local fit range is 1-500 Unicode scalar values, but the adapter reads the configured instance's limits before each delivery; a smaller or larger remote limit is authoritative. Nothing is truncated or split into threads.

After authorization to use a real instance, set `SYNCLOOM_MASTODON_INSTANCE` to its exact HTTPS origin. This explicit operator configuration is the outbound allowlist. Paths, credentials, queries and fragments are refused. HTTP is allowed only at `127.0.0.1:<port>` for the fake. Redirects are never followed. `SYNCLOOM_MASTODON_REDIRECT_URL` defaults to `http://127.0.0.1:8792/mastodon/callback`; that loopback port must be free.

```sh
syncloom mastodon sign-in
syncloom mastodon status
syncloom mastodon sign-out
```

Sign-in registers an app on that instance with generic client name `Syncloom`, the loopback redirect and `write:statuses write:media` scopes. Open the printed authorization URL, consent in the browser, and wait for the connection result. The app secret stays in memory for that sign-in only. The access token is stored exclusively in the macOS Keychain under `SYNCLOOM_KEYCHAIN_SERVICE` (default `syncloom`), account `mastodon:<sha256-of-instance-origin>`. A token from one instance cannot authorize another. Mastodon tokens have no automatic expiry; no fabricated expiration date is shown. A refused token requires new consent. Keychain code-hash prompts apply as described in the README's LinkedIn section.

`SYNCLOOM_DELIVERY=mastodon` grants delivery; without it, publish stays `contained` and makes no instance/media/status request. A real grant must not be set without authorization for that action.

An approved text piece creates one status. An approved visual piece uploads the verified first-slide PNG with the approved slide text as its alt description; the PDF is not uploaded. A 202 media response is polled with a finite budget, with 206 meaning not ready. No status is created until media is ready. The status carries the outbox request id as `Idempotency-Key`; Mastodon's temporary remote deduplication does not replace the durable local guard.

Only a successful response with a valid status id and same-instance URL becomes `posted`. Consent failure, refusal, rate limit, unreachable and uncertain outcomes remain distinct. A timeout or unreadable status answer is `unknown`; do not blindly retry it. Check the instance first. Media can remain orphaned after partial failure; the app never pretends that an uploaded image alone is a post.

## Manual handoff

Draft for `manual_export`, or export any other approved piece. The generic channel's 1-40000 range is a local drafting bound, not a claim that every target accepts that text.

```sh
syncloom export <piece_id> --expected-revision <revision> --target reddit
syncloom export <piece_id> --expected-revision <revision> --target tiktok
```

The current revision must be approved; visual pieces require the exact reviewed PNG/PDF hashes. The JSON receipt reports `status: exported`, request id, local bundle path and whether the receipt was newly created. It has no post id or URL and sends no network request, regardless of delivery grants.

Bundles live below `SYNCLOOM_DATA_DIR/exports/<deterministic-key>/`:

- `text.txt`: exact approved text, ready to copy.
- `notes.md`: target-specific manual checks.
- `manifest.json`: revision, target, export status and file hashes.
- Visual pieces also include exact `image.png`, `carousel.pdf` and `alt-text.txt`.

Reddit notes require checking subreddit rules, relevance, title and self-promotion policy. TikTok notes describe a later manual video workflow: these are source files, not a ready video or a Direct Post upload. No Reddit/TikTok API client is built.

Repeating the same piece/revision/target returns the same bundle and receipt after restart. Exporting to another target has its own identity. Existing complete files are checked, not silently overwritten. A partial failed bundle has no success receipt and may be completed on retry. Symlink destinations are refused. Bundles contain the founder's approved content: protect or delete them like other local studio data.

## HTTP operations

All routes use the existing exact-Host, no-CORS, fixed-error and content-free logging boundary.

| Operation | Result |
|---|---|
| `POST /v1/mastodon/sign-in` | 202 authorization URL and consent deadline |
| `GET /v1/mastodon/sign-in` | wait; 200 safe connection or fixed sign-in error |
| `GET /v1/mastodon/connection` | 200 safe connection |
| `DELETE /v1/mastodon/connection` | delete the instance's token |
| `POST /v1/pieces/{piece_id}/export` | body `expected_revision`, `target`; 200 export receipt |

No configured instance returns 503 `mastodon_not_configured`. Review, asset, sign-in and storage errors use the shared core codes. Research Q6 is stored only; none of its read APIs, scraping or Research Radar recommendations is implemented here.

## API references

Official docs checked for the adapter's request fields, statuses and processing semantics: [apps](https://docs.joinmastodon.org/methods/apps/), [OAuth](https://docs.joinmastodon.org/methods/oauth/), [instance](https://docs.joinmastodon.org/methods/instance/), [media](https://docs.joinmastodon.org/methods/media/) and [statuses](https://docs.joinmastodon.org/methods/statuses/). These APIs are versioned per endpoint (v1 apps/statuses, v2 instance/media). Instance compatibility and real account permissions remain unproven until an authorized live check.
