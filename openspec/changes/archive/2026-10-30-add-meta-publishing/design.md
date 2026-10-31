# Design

References: design, security, data, integrations, ui (Code 5.0.0).

## Context

See proposal.md. Evidence and choices are recorded in [decision 0005](../../../docs/decisions/0005-meta-integration.md).

## Goals / Non-Goals

Keep one outbox and one credential policy. No real sign-in, post, hosting or scheduling service.

## Decisions

- Use one Meta connection with two Platform delivery targets, rather than two login implementations: the Page token authorizes the Page and linked professional account. The configured numeric Page id, granted permissions and content tasks are checked before replacing a token.
- Use explicit user-token Facebook Login for Business config_id, then long-lived exchange and accounts discovery; reject automatic first-Page selection because it could publish to the wrong account. User tokens remain in memory only; production TokenStore is KeychainStore.
- Pin Graph v26.0, the version in current official examples, rather than the study's v25.0. Handwritten typed requests hide protocol details without adding an SDK.
- Extend rendering for Meta with JPEG screenshots of each slide before approval, rather than converting during dispatch: approval binds exactly what Meta fetches. Preserve PNG/PDF for existing callers; default-empty JPEG hashes allow legacy records to load, but do not authorize Instagram media.
- MediaDelivery owns external byte availability and accepts only bytes, never a caller path/URL; the production default refuses. Reject invented hosting because public exposure and retention need an owner decision.
- Graph calls use fixed origin, bounded JSON, fixed errors, shared HTTP timeout/no-redirect policy and no hidden retry. After a possibly committed publish, any failure remains unknown so outbox prevents duplicates.
- Keep scheduling local by not creating containers before a dispatch. No native scheduling parameters or durable containers.
- Retain the studio's shared 30-second platform budget rather than extending every platform's timeout for Meta: a container still processing at the first query ends unknown before its minute-spaced next poll. Background processing/resumption is not claimed or added; this conservatively avoids publishing or repeating an uncertain intent.
- Shared OAuth session now owns its immutable credential account, callback configuration and connection/sign-out policy, and TokenStore owns usable-token lookup; adapters retain only platform-specific exchange and publishing knowledge. This removes repeated lifecycle policy rather than waiving copy findings.

## Security boundaries

Only the existing protected local HTTP client may start consent or change a piece; callback uses shared exact Host and constant-time state checks. Numeric ids, fixed Graph paths and public HTTPS media URLs are validated; no Graph paging URL is followed. Tokens and app secret never appear in responses/events/logs; Page token is stored under meta:<Page id> in the Keychain, app secret is operator configuration in memory, user tokens are transient. All fakes use generic synthetic content. MediaDelivery is a trusted operator-installed port, not a public URL intake endpoint.

## Risks / Trade-offs

Meta's general login security docs require HTTPS and the inspected manual-flow docs do not establish an HTTP loopback exception: fake callback proof is not evidence of real app eligibility. A real app/redirect decision must precede real use. Meta's rate-limit examples conflict, so do not infer a quota; return truthful throttling. Hosting remains deliberately unavailable.

Final validation also exposed a pre-existing CLI test isolation defect: Keychain service ids used only pid and wall-clock nanoseconds, which repeat on this host. Add an atomic per-process suffix instead of serializing tests or retrying a missing credential; a fixed-clock regression proves distinct services. The observed missing credential is consistent with this race, though that failed run did not retain its two service ids.

## Migration Plan

Existing records deserialize unchanged. New Meta visual records include JPEG hashes. Roll back by removing grants; do not delete credentials or historical intents.

## Open Questions

Which public media host, lifecycle and deletion policy should be approved? What exact real app and redirect configuration is eligible?
