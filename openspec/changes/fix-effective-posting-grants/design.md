# Design

References: security, integrations, data, ui; Code 5.4.2.

## Context

See proposal.md. Current parent c0584e8; original launch base 3a059a0. Reproduced paths are server LinkedIn consent and saved-token publishing; adapters share CredentialSession and Token. Inspection read Connection, TokenStore, OAuthLifecycle, platform-specific exchange/publish paths, social_delivery, social_boundaries and platform-credentials/Threads/LinkedIn/Meta scenarios. Spec-map inspection found no companions, so native source/test searches remain authoritative; mapping completeness is unknown.

## Goals / Non-Goals

Keep one credential policy owner and per-action readiness. Do not change identity, approval fingerprints, hosting, scopes, app registration or scheduling. Account-approval owns immutable dispatch credentials and identity.

## Decisions

Move the inherited standalone posting_grant registry into existing Token/credential ownership; reject UI scope parsing because it duplicates authorization policy. Expose action-aware connection results from the same owner used at dispatch.

Keep additive scope_verified evidence for adapters whose legacy scopes were fabricated; reject interpreting old requested strings as grants. X refresh credentials are optional for fresh text posting; a malformed supplied refresh token remains rejected. Missing evidence is unverified, not known denial.

Use the officially documented Threads v1.0 debug_token response after exchange/rotation with existing app credentials; reject both blanket platform rejection and token-only success. Failed token exchange preserves the prior credential, but successful narrowed/unverified rotation durably replaces stale authorization evidence before returning its precise refusal. Direct source excerpts and the prospective acceptance freeze are retained in task recovery evidence; a final project decision record and Main challenge remain required.

## Reuse inventory

Timing: after inherited first edits, before recovery code edits. The inherited miss is not backdated. Findings inventory returned zero lexical candidates for credentials/session/Threads/new registry and server/fake paths. Search: scope_has_all, posting_ready, granted_scope, debug_token across crates, specs and research. Candidates: CredentialSession usable/connection, TokenStore usable_token, Session, OAuthLifecycle, adapter predicates and inherited posting_grant.rs. Decision: existing Token owns the action policy; credential loading and credential writes remain with CredentialSession/CredentialWrites. Remove the duplicate new registry after migrating its useful per-action rules.

## Security boundaries

Only the existing local founder session may request consent/refresh/publish. Effective grants are checked before publishing writes against the loaded dispatch token; identity remains independently mandatory. Secrets remain in the configured environment and TokenStore; no raw response, query URL, token, app secret or debug request is logged or returned. Synthetic tests use only MemoryStore and loopback fakes. Guard mutation must fail the corresponding refusal regression.

## Migration Plan

New evidence fields deserialize absent for legacy bundles. LinkedIn/X/Mastodon old scope values came from provider responses; historical Meta/Threads requested strings do not establish readiness. Reconnect establishes verified evidence. No destructive migration or Keychain access occurs in this task. Rollback is a local code revert before landing, not a permission to restore unsafe authorization.

## Risks / Trade-offs

- Additional critical read-only Threads request can fail: preserve existing token, fail explicitly, no retry.
- Capability/identity concurrency: coordinate one snapshot with account-approval before integration.
- Per-action and hierarchical scope semantics: retain positive controls and report unverified provider facts rather than invent policy.
- No real-platform validation: do not claim official app/review acceptance.

## Testing and validation

LinkedIn TCP reduced grant and full fixture counterfactual; Threads debug reduced/missing/invalid/error plus verified fresh/refresh/post; saved legacy/reduced credentials; X and Mastodon text/media split; Meta target grants. Keep prior success, refresh, expiry, sign-out race and no-secret tests. Run pinned offline Rust tests, fmt, clippy, OpenSpec/shape/research checks and Code Verify against original base and current parent. Browser visual acceptance requires actual captures or remains not evaluated. All results, including interrupted attempts, remain in external task evidence.
