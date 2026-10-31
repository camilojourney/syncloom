# Facebook Pages and Instagram (fake-proven only)

The [Q3 research card](research/2026-10-30-q3-meta-facebook-instagram/README.md) records immutable reports and current official-doc checks. [Decision 0005](decisions/0005-meta-integration.md) explains the chosen path and live-use gaps.

## Scope

- `facebook_page`: text or the approved first-slide PNG, with caption and alt text.
- `instagram`: one to ten ordered 1080x1350 JPEG slides from the local renderer, with slide-derived alt text. Single image uses one container; carousel uses children then a parent. AI disclosure is true for this generated-content path, on the single container or carousel parent only.
- The existing outbox, exact-revision approval, delivery grants and uncertain-outcome guard apply unchanged. No hidden retry and no native scheduling parameter. Containers are created only during a local dispatch; no scheduler is added here.
- Graph API v26.0. No video, analytics, first comment, moderation or account discovery UI.

## Consent and credentials

HTTP clients use the [existing local session boundary](studio.md): POST/PATCH/DELETE require X-Syncloom-Token; GETs including asset previews inherit Host/Origin checks.

| Operation | Answer |
|---|---|
| POST /v1/meta/sign-in | 202, authorization URL and deadline |
| GET /v1/meta/sign-in | wait for callback, 200 connection |
| GET /v1/meta/connection | 200 connection from the Keychain, no network |
| DELETE /v1/meta/connection | delete this Page's credential, 200 signed_out |

Missing configuration is 503 `meta_not_configured`. Existing fixed sign-in errors cover state mismatch, denial, timeout and exchange refusal.

One Facebook Login for Business **user-token configuration** (not a system-user configuration) uses config_id, code response and the loopback callback. Required configured permissions: pages_show_list, pages_read_engagement, pages_manage_posts, instagram_basic, instagram_content_publish. The adapter checks the granted permission list, selects only the configured numeric Page id from accounts and checks CREATE_CONTENT, MANAGE and MODERATE tasks. It never silently chooses the first Page.

Short-lived user token is exchanged for a long-lived user token before deriving the Page token. Only the Page token, linked professional-account id and credential metadata persist, under account `meta:<Page id>` in the configured macOS Keychain service. User tokens remain transient; no file, response, event or log contains them. App secret is operator-supplied configuration in memory, never embedded in a binary. Sign-out cancels consent and deletes only that Page's token.

| Variable | Default |
|---|---|
| SYNCLOOM_META_CLIENT_ID | unset |
| SYNCLOOM_META_CLIENT_SECRET | unset |
| SYNCLOOM_META_CONFIG_ID | unset |
| SYNCLOOM_META_PAGE_ID | unset, numeric only |
| SYNCLOOM_META_REDIRECT_URL | http://127.0.0.1:8913/meta/callback |
| SYNCLOOM_META_GRAPH_BASE | https://graph.facebook.com |
| SYNCLOOM_META_AUTH_BASE | https://www.facebook.com |

Base overrides are trusted operator configuration, not HTTP input; loopback HTTP is for the fake. Delivery names are `facebook` and `instagram`, independently granted through SYNCLOOM_DELIVERY. The CLI can draft, render, approve and publish these channels; Meta consent operations are currently HTTP-only.

## Exact JPEG approval

Meta rendering also saves `assets.slides_sha256` in slide order. Protected `GET /v1/pieces/{id}/assets/slides/{index}` returns a zero-based JPEG with its ETag; the review screen shows all JPEGs and hashes.

Supply the ordered list as `expected_assets.slides_sha256` alongside image_sha256 and carousel_sha256. Missing, reordered or changed hashes fail approval. The CLI uses repeated `--slide-sha256 <hash>` arguments. Every byte hash is rechecked before dispatch. Old PNG/PDF-only records still load, but Instagram needs a new render.

## Public media is deliberately unconfigured

Meta fetches each image from a public URL. `MediaDelivery` accepts only approved bytes and MIME type and must expose those exact bytes unchanged until Graph finishes fetching. It returns a validated public-shaped HTTPS URL or a delivery failure. It is a trusted integration port, not a user URL endpoint.

Production installs `UnconfiguredMedia`: photo/Instagram attempts are refused before publishing, even with a delivery grant and valid token. There is no invented upload host. Tests use a fake URL and record delivered bytes without external traffic.

**Open question:** Which actual media host, public lifetime, retention and deletion policy should a later approved implementation use?

## Outcomes and proof

Graph credential errors (including code 190 on HTTP 400) become consent_required; permission refusal becomes refused; throttling becomes rate_limited. Server failures, malformed success and lost publish responses become unknown. Only FINISHED containers may be published. An immediate status query is followed by at most five queries one minute apart while processing, matching Meta's documented cadence; no create or publish request is retried. Facebook photo id alone never proves a post; Instagram permalink is read from Graph, never fabricated from its numeric id.

`cargo test -p syncloom-core --test meta` checks failures and guards; `cargo test -p syncloom-server --test meta_api` proves the HTTP consent, render, exact approval, Page text/photo, Instagram image/carousel, containment and restart/repeat journey against the stateful loopback Graph fake.

No real Meta app, sign-in or post was used. Standard Access is documented for owned/managed accounts, but real app eligibility, dashboard permissions, exact loopback redirect acceptance and public visibility remain unproven. General login security docs require HTTPS and do not establish a loopback exception. Development-mode generated data may be visible only to app roles. Real use requires separate captain permission for each exact external action and a resolved hosting choice.
