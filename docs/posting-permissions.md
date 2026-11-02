# Effective posting permissions

A stored token and a configured delivery grant are not sufficient permission to publish. The [credential owner](../crates/syncloom-core/src/credentials.rs) evaluates the selected action on the same loaded credential used for dispatch. Identity/approval checks are independent requirements, not substitutes for capability.

## Platform audit

All behavioral proof uses MemoryStore and stateful loopback fakes. Real account eligibility, permission revocation and platform acceptance are unverified.

| Platform | Pattern / effective evidence | Action boundary | Regression proof |
|---|---|---|---|
| LinkedIn | Affected: token response scope was not enforced; legacy returned scope remains evidence | Text needs exact w_member_social, not a substring | server linkedin_api reduced consent, saved grant, exact-token and full-grant controls |
| X | Affected: saved/rotated reduced grants could appear ready; fresh text incorrectly required a refresh token | Text needs tweet.write and users.read; PNG adds media.write. Offline permission is not needed to post | server social_delivery 15-case absent/null/present scope matrix, narrowed refresh, saved denial and fresh text/no-refresh positive; core social_sessions rotation |
| Mastodon | Affected: saved reduced scopes could appear ready | Text needs write:statuses; PNG adds write:media; broad write covers write children | core mastodon reduced saved grant, text-only positive and image refusal |
| Facebook Page | Fresh consent already checked provider /me/permissions; legacy requested strings are unverified | Page posting needs pages_manage_posts; selecting a Page still needs pages_show_list and content-creation tasks | core meta fresh per-target positive/negative and legacy/saved denial |
| Instagram | Same shared Meta evidence issue; grants are target-specific | Posting needs instagram_content_publish; Page discovery and linked professional-account requirements remain independent | core meta independent target grant and legacy controls |
| Threads | Affected: requested scope was fabricated as a grant; token response alone does not prove permission | Official v1.0 debug_token is_valid plus actual granted scopes must establish threads_content_publish | server social_delivery fresh reduced/missing/legacy and narrowed/unverified rotation; core social_sessions verified fresh/refresh/post |
| Bluesky | Not affected by separable-scope pattern: current app-password session has no independent requested/granted posting-scope list | Existing session, identity and explicit delivery/approval checks remain required | Existing social_sessions, social_credentials and server success/refusal controls |
| Manual export | Not a remote publishing integration | Existing local export rules | Existing export tests |

This table describes the implementation and evidence; it is not another runtime capability registry. Absence of real-platform validation is not a claim of worldwide availability.

## Truthful recovery

An unexpired credential without the selected permission reports `cannot_post`, reason `posting_permission_missing`, message `connected, cannot post: posting permission missing.`, and action `sign_in`. Missing effective evidence instead reports `posting_permission_unverified` and says permission could not be verified. Publish refuses with that same stable status and sends no write. Absent/expired credentials retain `consent_required` behavior.

The existing sign-in operations reconnect: `syncloom linkedin sign-in`, `syncloom mastodon sign-in`, `syncloom connect x sign-in`, or `syncloom connect threads sign-in`. Meta uses existing `POST /v1/meta/sign-in` and matching completion. Protected API mutations still require the local session token. No new credential-management page is introduced.

Historical Meta/Threads bundles deserialize without scope_verified and remain unresolved until reconnection establishes effective evidence. Their old requested scope text is not trusted. Historical LinkedIn/X/Mastodon scopes came from provider token responses.

A failed token exchange preserves prior credentials. A successful rotation is different: the actual narrowed or unverified rotated credential is persisted before reporting its permission error, so the old broader grant cannot remain publish-ready. There is no hidden retry or renewal during publishing. X can post text without a refresh token; explicit refresh then refuses without a request.

## X omission versus malformed evidence

X response decoding preserves absent, null and present as distinct cases. An absent refresh scope may retain the prior provider-returned scope. Fresh absence and explicit null, empty, noncanonical or wrong-typed evidence are unverified, not a grant or known denial. A valid rotated credential with unresolved evidence is saved atomically with empty normalized scope and false provenance, never the old broader scope. Historical nonempty X scope strings retain their provider-returned meaning.

[RFC6749 section5.1](https://www.rfc-editor.org/rfc/rfc6749#section-5.1) makes response scope optional only when identical to requested scope; [section6](https://www.rfc-editor.org/rfc/rfc6749#section-6) separately defines omitted refresh-request scope as the originally granted scope and refers successful responses to5.1. Direct source text was read2026-11-02. This informs the bounded omission policy; it does not establish X conformance or permit interpreting null as omission. The legacy comma/whitespace parser is not used to invent grants from noncanonical new X response values.

Permission-unverified means no publishing write was sent. Outbox unknown instead means an effect may already have occurred. Token obtained_at is acquisition time, not a claim of remotely verified current permission. Generic403 does not establish a permission-specific revocation.

## Threads source check

Official public documentation retrieved 2026-11-02, not a live API test:

- [Debug Access Token](https://developers.facebook.com/documentation/threads/troubleshooting/debug-access-token): GET /v1.0/debug_token accepts input_token and a same-app app/tester access_token. `scopes` is the "List of permissions that the user has granted for the app in this access token." `is_valid` identifies token validity.
- [Debug reference](https://developers.facebook.com/documentation/threads/reference/debug): confirms response fields and same-app restriction. There is no documented per-scope declined enum in this response.
- [App Access Tokens](https://developers.facebook.com/documentation/threads/get-started/app-access-tokens): documents alternate app authentication TH|APP_ID|APP_SECRET, using existing configured credentials. Never expose that value in a distributed binary, logs, URLs returned to the browser or errors.
- [Overview](https://developers.facebook.com/docs/threads/overview): both graph.threads.com and graph.threads.net are supported.

Debug verification is a critical bounded read-only request after exchange/rotation. Missing, invalid, malformed or unavailable evidence never becomes a granted scope. The local-only confidential-app setup is not approval to distribute an app secret.
