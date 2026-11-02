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

## Approved frontend reconciliation (009/010)

Rebased the isolated branch onto approved frontend mainb97c6b7 without skipping commits. The old page.rs was replaced by the approved page modules; the existing CannotPost connection adapter and disabled-Publish guard now live in page/review.rs. Paired outcome copy in page/status.rs consumes stable core outcomes, never scopes. Both frontend decision records, prose and custody hashes remain unchanged; the posting decision keeps hash90d2b1de.

Two additional real-HTTP review tests first prove the verified full grant leaves Publish enabled, then prove missing/unverified grants disable it, expose sign-in recovery and precise saved refusal, and issue zero platform requests. Removing the UI guard makes the missing-permission test fail; restoration is retained. Server tests and all-target Clippy pass. Current screenshots/visual acceptance remain unverified: a temporary fixture initially used an invalid channel identifier and the isolated CDP target failed to start; no old-frontend screenshot is relabeled as current. The temporary fixture/browser were removed/stopped. Exact final execution receipts belong to the private handoff, not to backdated decision claims.

## Combined security integration (Main1374-1376)

Ordinary merges preserve posting source b8ee8e6 and account/wording source fa3f3c1 as ancestors on b97c6b7. Neither history or source report is rewritten. The account adapter's target check and the credential-owned effective-action predicate consume the same loaded immutable dispatch Token before media or posting. Reconnect cannot relabel an in-flight attempt, its bundle or its one-target receipt.

Main1375 resolves the successful-issuance/failed-identity seam: persist the actual new X/Threads bundle and normalized grant evidence atomically, retain the canonical identity only as expected, and clear confirmation for that token. The minimal identity_confirmed Token field defaults false for legacy records. A read-only identity retry before a later publish may confirm that exact token; mismatch uses existing sign-out, persistent failure remains cannot_post with identity_unconfirmed/sign-in-again recovery. The auth lock serializes confirmation with issuance; the existing write generation excludes sign-out. Pre-issuance exchange failure preserves old credentials. Missing and Unverified permission remain distinct and take priority over transient identity failure; neither is outbox Unknown. This continuation supersedes only the earlier no-identity-reread/old-bundle prose where inconsistent.

The native review reconnect adapter uses the shared Host/Origin/session boundary and existing consent owners, finishes their supported flow, and returns to the original review without approval or publication. Threads retains its existing callback URL handoff. It adds no credential console, scopes, client script or OAuth policy. Shared ReconnectForm::resolve owns native path/form normalization and piece lookup. No second authorization owner or dispatch token read is added.

Combined counterfactuals use the existing stateful loopback fakes and MemoryStore. Runnable mechanical composition first failed reduced/null X issuance plus failed users/me and Threads failed-identity rotation. The existing mismatch sign-out control already passed. Corrected transient-X token inequality was rechecked against the mechanical baseline after initial implementation, explicitly not backdated. The joint media/reconnect test retains exact stale-account409, immutable A bearer/receipt, B narrowed-media-grant zero-write refusal and independent good-B assertions. Early request-shape mistakes were fixture failures, not product RED. Native LinkedIn reconnect first failed for its missing control, then passed protected HTTP consent/completion with no publication or approval mutation. All attempts and final exact-head evidence remain external.

Main1380's bounded browser control isolated a launch-environment discrepancy, not a product failure: temporary HOME stalled own static and review pages; inherited HOME with a fresh owned profile rendered both. The authorized session-scoped direct-CDP fallback captured all four combined states in light/dark at measured1440x900 and390x844. Existing native forms completed fresh B approval and X/Threads reconnect against synthetic provider boundaries, with zero posts. Temporary capture fixtures were removed. A visually observed adjacent recovery/Publish spacing defect was corrected in the existing approval form rule; all16 images were recaptured. Raw before/after images, dimensions, contrast, hashes, exact binary/source identity and degraded manual Impeccable review are external task evidence, not real-provider validation. Final committed-tree Verify receipts remain external so no evidence-only project commit changes the checked head.

## Exact retained findings

Kept: crates/syncloom-core/src/outbox.rs IntentStatus::as_str lines37-51, because Main1360 explicitly retains the flat compiler-exhaustive11-arm stable-string mapping (cyclomatic9->11, limit10); a derive/dependency would obscure wire strings without improving the rule.

Kept: crates/syncloom-core/src/meta/auth.rs lines108-114 and crates/syncloom-core/src/oauth.rs lines218-224, because Main1360 explicitly retains only clone355ce2414f6b555388d1b71b56b592a943c4b04bd067955844fbe1ede36f4347,7lines/30tokens spanning the callback ending and common public connection_for signature over distinct bodies; a shared abstraction would be a forwarding wrapper.

Kept: crates/syncloom-server/src/page/reconnect.rs begin cyclomatic12 over10 and complete cyclomatic12 over10, because Main1376 explicitly retains these two flat provider dispatches after shared ReconnectForm::resolve; each calls independently owned consent flows, and a facade or trait would only forward without owning a rule.

Kept: crates/syncloom-server/src/page/reconnect.rs begin lines25-29 and complete lines58-62 at measurement, because Main1376 explicitly retains only framework-required typed Axum handler-signature clone1e59a8e092b063993131111aa375eae17df49ca3c457d8a6f853f82ddfaf50d8,5lines/43tokens, with distinct bodies. Final native measurement must retain this exact fragment and the two exact12 scores; any different/new identity requires separate disposition.

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
