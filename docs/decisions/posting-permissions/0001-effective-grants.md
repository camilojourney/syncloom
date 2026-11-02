# posting-permissions/0001: Effective posting grants

Status: proposed
Owner: Main, through firstmate
Method: decide@1.3.1, source SKILL.md SHA256 826eec40bdc6539dfb29df5d26af92f1ca735d0dfcfdda80407a66cc4f5fa515
Checklist: 1.0.0, SHA256 8c88c2c9c524d97552f67f30f6c4ba25589516b9dc18a51b09f7962688e5b056; [pinned source](../sources/decide-question-checklist-1.0.0.md), Fleet baa7c65c88fa04eaa2d2167e78b24ff56e575e68
Identity: posting-permissions/0001-effective-grants
OpenSpec: [fix-effective-posting-grants](../../../openspec/changes/fix-effective-posting-grants/design.md), [tasks](../../../openspec/changes/fix-effective-posting-grants/tasks.md)
Evaluation: synthetic execution receipts retained in the authorized posting-grants task data; final Code Verify pending.

## 1. Brief

Recorded 2026-11-02 after inherited implementation, bounded official-source research and local tests. This is not a retrospective claim of prospective custody. Original instructions and recovery inventory remain in task data; no immutable pre-research eight-artifact packet was captured. This record is not eligible as a cutoff-valid method comparison.

Choose how existing credentials prove the selected posting action without confusing a requested scope, a token, identity or delivery configuration with permission. Main authorized implementation and requires concrete before/after review plus a specific landing word. Preference and hard constraint: reuse existing credential ownership, no new connection page or duplicate registry.

Class: consequential. A wrong positive can publish to a remote account. A local code revert cannot retract a post; testing therefore uses only MemoryStore and loopback fakes. No real Keychain, platform requests, accounts, remote, paid publication or merge is authorized. Existing worker process budget is the configured two-hour horizon; additional model commissioning belongs to firstmate.

Checklist dispositions: Q1 pass for the delegated repair and retained landing authority; Q2 compare the alternatives below; Q3 hard boundaries above; Q4 measured local failures below, real-provider reliability unknown; Q5 decisive unknown was Threads' effective-grant protocol; Q6 required different-family challenge remains unresolved. Missing prospective preparation is disclosed, not repaired by these labels.

## 2. Frozen acceptance

The original [request](../../../openspec/changes/fix-effective-posting-grants/proposal.md#request) requires reduced-grant refusal and full-grant support across existing integrations, red before repair, saved-token coverage, precise recovery and Main review. Recovery instructions separately prohibit token-only acceptance, deleted negatives and blanket platform denial. These owner constraints predate replacement implementation; this record's cutoff does not.

Pass marks: reduced/unverified saved credentials send zero writes; valid text and supported image grants retain success; text does not require media/offline permission; failed token exchange preserves credentials; successful rotation never borrows old permission evidence; connection and dispatch use the same policy and dispatch credential. Safety limit: synthetic loopback only. No post-hoc relaxation is authorized.

## 3. Unknown-resolution plan

- Threads mechanism: one bounded official public source check, explicitly authorized through firstmate; resolved at documentation level by C1, not by live API testing.
- Runtime boundaries: use existing stateful fakes, counterfactual grants, rotation and legacy shapes. No additional platform calls.
- Account/approval snapshot: separate worker owns identity and immutable dispatch snapshot; shared fields require integration review.
- Main acceptance and fresh different-family challenge: human/authorized-agent route through firstmate. Evidence cannot grant landing authority.

No Phrona study or broad architecture investigation was commissioned for this repair.

## 4. Evidence and options

### Claims

| ID | Claim | Status | Evidence |
| --- | --- | --- | --- |
| C1 | The official Threads debug reference describes is_valid and scopes containing permissions granted in the access token | verified | [Debug Access Token](https://developers.facebook.com/documentation/threads/troubleshooting/debug-access-token), response fields and sample; directly retrieved/read 2026-11-02, documentation only |
| C2 | Official Threads documentation permits app authentication as TH followed by app id and app secret | verified | [App Access Tokens](https://developers.facebook.com/documentation/threads/get-started/app-access-tokens), alternate app-token format; directly retrieved/read 2026-11-02, no real authentication tested |
| C3 | Saved reduced or legacy grants can be distinguished from valid grants at local publishing boundaries | verified | [LinkedIn TCP regression](../../../crates/syncloom-server/tests/linkedin_api.rs), [social TCP regressions](../../../crates/syncloom-server/tests/social_delivery.rs), [Mastodon](../../../crates/syncloom-core/tests/mastodon.rs), [Meta](../../../crates/syncloom-core/tests/meta.rs); executed 2026-11-02, synthetic only; removing permission guards made five regressions fail |
| C4 | Current app-password Bluesky sessions expose no independent requested/granted scope list in this integration | verified | [Bluesky session owner](../../../crates/syncloom-core/src/bluesky.rs), inspected 2026-11-02; bounded to this implementation, not a statement about all Bluesky authorization |
| C5 | Real provider acceptance, app eligibility, revocation timing and debug endpoint reliability are established | unverified | none: no live provider test is authorized; official documentation and loopback success do not establish these facts |

Contradiction resolved: missing scope in the Threads exchange response is not evidence of protocol impossibility. C1 supplies a separate documented read. It also does not justify treating legacy requested strings as granted.

Options against the acceptance criteria:

- Keep token-presence readiness: rejected by observed false connected/posting outcomes and the owner's explicit constraint.
- Deny Threads entirely: rejected because it removes a supported full-grant path despite C1.
- New standalone posting-grant registry or browser parsing: rejected because it duplicates credential policy and can diverge from dispatch.
- Provider-local checks only: feasible for exchange validation, insufficient alone for legacy saved credentials and action-specific UI readiness.
- Existing credential-owned action predicate, provider evidence at exchange/rotation: candidate. Covers stored credentials and one dispatch snapshot; reuses provider-specific network ownership. An extra critical Threads read trades availability for truthful evidence, not universal denial.

Transfer limit: local trials prove the implementation against synthetic contracts, not official app approval or worldwide availability.

## 5. Challenge

Required, not waived. A fresh different-model-family challenger must be commissioned by firstmate through an authorized private-evidence route; this worker may not delegate. Initial pick, defense, disagreement, recommendation reveal and objection/answer are not yet received. No same-family self-review is labeled independent.

Commissioning must provide the question, constraints, criteria and authorized source/test evidence before revealing the candidate, retain the exact input digest, and disclose any implementation/framing leaks. Then reveal the diff and request a stronger competing option, disconfirming counterexample and simpler method. Until that response and owner resolution exist, acceptance remains proposed. Main's separate landing gate also remains open.

## 6. Experiment

Retained prior protocols and failures are in authorized recovery task data; this later record does not invent earlier freeze times. The question was whether reduced/legacy grants and narrowed rotations remain falsely ready while full grants work. Existing TCP/adapter fakes are the boundary, MemoryStore is custody, and zero unintended writes is the pass mark. Commands have bounded timeouts, no hidden retries or external effects.

Observed before: LinkedIn partial consent returned 200 and saved the reduced token in the established audit; Threads fresh reduced and legacy grants falsely connected/posted; X/Threads successful reduced rotations left the old full grant ready; fresh X text without renewal was wrongly rejected. Retained red receipts precede the corresponding repairs. Guard removal later reproduced five saved-grant failures; both guards were restored in finally.

Observed candidate: 156 selected synthetic core tests and 78 server tests passed before integration, no failures or ignored tests. Initial combined core attempt timed out and is not a pass. Clippy initially rejected new helper unwraps; Result propagation fixed them without suppressions. Browser captures at 1280px and 390px showed the missing-permission reason and disabled Publish with no narrow overflow. There is no before browser capture; before/after protocol evidence is the claimed comparison. Post-rebase validation is still pending here.

## 7. Decision

Recommend credential-owned action permission with effective provider provenance, distinguishing missing from unverified. Preserve historical returned scopes for LinkedIn/X/Mastodon; legacy requested Meta/Threads scopes remain unverified. Successful rotations retain the actual reduced/unverified credential before returning its permission error; failed exchanges do not replace credentials. [Platform audit and recovery](../../posting-permissions.md) describe the bounded implementation.

This is a recommendation, not acceptance or landing authorization. Open questions: required challenger, account-snapshot integration and Main's concrete evidence review. No architecture report is a dependency.

### Reconsideration trigger

| Signal | Owner | Action |
| --- | --- | --- |
| Official debug contract differs from C1/C2 or cannot be used with the configured app | Main through firstmate | Reopen protocol choice; preserve explicit unverified refusal rather than invent a grant |
| Any reduced/legacy/rotated grant sends a write, or valid text unexpectedly needs media/offline permission | Credential implementation owner | Reproduce through the existing end-user path and repair before landing |
| Capability and account-approval use different credential snapshots | Integration owner through firstmate | Stop landing and reconcile one dispatch credential |

Verification: Code 5.4.2 is loaded; final Verify against launch 3a059a0 and parent 77e3c42 is not yet run. Structural checks cannot prove truth, independence or approval.

## 8. Outcome review

Proposed owner/date: firstmate, 2026-11-09, subject to owner confirmation. Future prediction: if landing is authorized, the combined identity/capability branch retains reduced-grant zero-write and full-grant success controls. Observation source: integration test receipts at the actual reviewed head, not a live-account experiment. Missing evidence stays unknown.

Outcome-blind process review and observed post-landing outcomes are pending. This proposed record does not count as an accepted method record; Fleet's coordinating owner owns the ten-accepted-record cadence, whose cross-project count is unknown here. Missing prospective custody and challenger remain visible regardless of eventual test outcomes. Implementation progress belongs only to the linked OpenSpec tasks.
