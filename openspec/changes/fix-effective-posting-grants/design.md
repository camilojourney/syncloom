# Design

References: security, integrations, data, ui; Code 5.4.2.

## Context

See proposal.md. Original parent c0584e8; integrated parent 77e3c42; original launch base 3a059a0. Reproduced paths are server LinkedIn consent and saved-token publishing; adapters share CredentialSession and Token. Inspection read Connection, TokenStore, OAuthLifecycle, platform-specific exchange/publish paths, social_delivery, social_boundaries and platform-credentials/Threads/LinkedIn/Meta scenarios. Spec-map inspection found no companions, so native source/test searches remain authoritative; mapping completeness is unknown.

## Goals / Non-Goals

Keep one credential policy owner and per-action readiness. Do not change identity, approval fingerprints, hosting, scopes, app registration or scheduling. Account-approval owns immutable dispatch credentials and identity.

## Decisions

Candidate: existing credential-owned action policy, additive scope_verified provenance and documented Threads debug verification; successful rotations retain their actual evidence. The proposed [posting-permissions/0001 decision](../../../docs/decisions/posting-permissions/0001-effective-grants.md) owns rationale, alternatives, custody limits, independent challenge and Main acceptance. Direct source excerpts and failed/passing execution receipts remain in authorized task recovery data.

## Reuse inventory

Timing: after inherited first edits, before recovery code edits. The inherited miss is not backdated. Findings inventory returned zero lexical candidates for credentials/session/Threads/new registry and server/fake paths. Search: scope_has_all, posting_ready, granted_scope, debug_token across crates, specs and research. Candidates: CredentialSession usable/connection, TokenStore usable_token, Session, OAuthLifecycle, adapter predicates and inherited posting_grant.rs. Decision: existing Token owns the action policy; credential loading and credential writes remain with CredentialSession/CredentialWrites. Remove the duplicate new registry after migrating its useful per-action rules.

## Security boundaries

Only the existing local founder session may request consent/refresh/publish. Effective grants are checked before publishing writes against the loaded dispatch token; identity remains independently mandatory. Secrets remain in the configured environment and TokenStore; no raw response, query URL, token, app secret or debug request is logged or returned. Synthetic tests use only MemoryStore and loopback fakes. Guard mutation must fail the corresponding refusal regression.

## Migration Plan

New evidence fields deserialize absent for legacy bundles. LinkedIn/X/Mastodon old scope values came from provider responses; historical Meta/Threads requested strings do not establish readiness. Reconnect establishes verified evidence. No destructive migration or Keychain access occurs in this task. Rollback is a local code revert before landing, not a permission to restore unsafe authorization.

## Risks / Trade-offs

- Additional critical read-only Threads request can fail: preserve credentials on fresh exchange refusal, but retain the actual unverified credential after successful rotation; fail explicitly, no retry.
- Capability/identity concurrency: coordinate one snapshot with account-approval before integration.
- Per-action and hierarchical scope semantics: retain positive controls and report unverified provider facts rather than invent policy.
- No real-platform validation: do not claim official app/review acceptance.

## Testing and validation

LinkedIn TCP reduced grant and full fixture counterfactual; Threads debug reduced/missing/invalid/error plus verified fresh/refresh/post; saved legacy/reduced credentials; X and Mastodon text/media split; Meta target grants. Keep prior success, refresh, expiry, sign-out race and no-secret tests. Run pinned offline Rust tests, fmt, clippy, OpenSpec/shape/research checks and Code Verify against original base and current parent. Browser visual acceptance requires actual captures or remains not evaluated. All results, including interrupted attempts, remain in external task evidence.
