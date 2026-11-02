# Design

References: security, integrations, data, ui; Code 5.4.2.

## Context

See proposal.md. Original parent c0584e8; integrated parent 77e3c42; original launch base 3a059a0. Reproduced paths are server LinkedIn consent and saved-token publishing; adapters share CredentialSession and Token. Inspection read Connection, TokenStore, OAuthLifecycle, platform-specific exchange/publish paths, social_delivery, social_boundaries and platform-credentials/Threads/LinkedIn/Meta scenarios. Spec-map inspection found no companions, so native source/test searches remain authoritative; mapping completeness is unknown.

## Goals / Non-Goals

Keep one credential policy owner and per-action readiness. Do not change identity, approval fingerprints, hosting, scopes, app registration or scheduling. Account-approval owns immutable dispatch credentials and identity.

## Decisions

Candidate: existing credential-owned action policy, additive scope_verified provenance and documented Threads debug verification; successful rotations retain their actual evidence. The proposed [posting-permissions/0001 decision](../../../docs/decisions/posting-permissions/0001-effective-grants.md) owns rationale, alternatives, custody limits, independent challenge and Main acceptance. Direct source excerpts and failed/passing execution receipts remain in authorized task recovery data.

## Main1360 continuation

Retain the one-owner predicate and zero-write unverified contract. X response decode distinguishes absent, explicit null and present values. Only an absent refresh field inherits prior provider-returned scope; fresh absence and null/empty/noncanonical or wrong-typed evidence are unverified. A valid rotated token and its actual normalized evidence replace the old bundle atomically. Historical nonempty X scopes retain their provider-returned interpretation; a new unresolved rotation records empty scope plus false provenance. No identity reread, generic403 downgrade or obtained_at-as-verification-time claim is introduced.

The 15-case TCP matrix ran against preserved ae89808 first: five already-correct controls passed and ten genuine counterfactuals failed. All cases and the 32-test social TCP target then passed. Null-to-absent guard mutation reproduced false readiness and a synthetic post; the guard was restored. An additional positive control confirms existing unknown-prior refusal before another exchange; the final social TCP target has33 passing tests. Official RFC6749 sections5.1/6 were directly read as normative text, not proof of X conformance. The decision record appends the late Main1357/1360 challenge and disclosed framing/context limits.

## Exact retained findings

Kept: crates/syncloom-core/src/outbox.rs IntentStatus::as_str lines37-51, because Main1360 explicitly retains the flat compiler-exhaustive11-arm stable-string mapping (cyclomatic9->11, limit10); a derive/dependency would obscure wire strings without improving the rule.

Kept: crates/syncloom-core/src/meta/auth.rs lines108-114 and crates/syncloom-core/src/oauth.rs lines218-224, because Main1360 explicitly retains only clone355ce2414f6b555388d1b71b56b592a943c4b04bd067955844fbe1ede36f4347,7lines/30tokens spanning the callback ending and common public connection_for signature over distinct bodies; a shared abstraction would be a forwarding wrapper.

The existing mechanism matches paths, not fragment hashes. Therefore every actual retained entry and native residue identity is reviewed against these two exact authorizations; another identity is not authorized by these lines. Raw pre-retention findings and Main's reasons remain disclosed in task evidence. Original-base canonical inherited retentions are reported separately, not attributed to Main1360. No threshold, scope, checker, macro or source reordering change is made.

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
