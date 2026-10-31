# ADR-0004: Mastodon API add-on and a manual handoff target

**Date:** 2026-10-31
**Status:** Proposed
**Supersedes:** none
**OpenSpec changes:** [add-mastodon-manual-export](../../openspec/changes/add-mastodon-manual-export/)
**Agreed:** no - proposal approval remains pending; the firstmate task brief authorizes this fake-only implementation, not live sign-in or publication
**Recommended:** Mastodon API publishing plus generic Reddit/TikTok manual handoff; no Pinterest or Reddit/TikTok publishing APIs
**Built:** partly: implemented on the local task branch; landing is not yet claimed
**Live-proven:** no - local fake Mastodon only, no real instance traffic

---

## Context

Vision 6.2 adds channels when their research arrives. Q5 is now imported verbatim; Q6 is imported as research only. Review, visual integrity and dispatch already have owners, so a new channel must use them rather than create a parallel publishing path.

---

## Decision

The proposed capabilities [mastodon-integration](../../openspec/changes/add-mastodon-manual-export/specs/mastodon-integration/spec.md) and [manual-export](../../openspec/changes/add-mastodon-manual-export/specs/manual-export/spec.md) own the new behavior. Existing credentials, rules, outbox, CLI and API owners are extended in the same change; Q6 reading integrations remain out of scope.

---

## Rationale

| Option | Pros | Cons | Why rejected / chosen |
|---|---|---|---|
| Thin Mastodon adapter and local bundles | Fits existing Platform and approval boundaries; no review workflow for Mastodon; manual targets send nothing | Instance variation and a small audience; manual handoff is not publishing | Chosen within the task's stated scope; independent live compatibility is not assumed |
| Broad multi-network SDK | More features available | More surface and lifecycle policy than the required endpoints | Rejected; reuse the existing HTTP policy and typed outcomes |
| Reddit/TikTok API publishing | Automated delivery | Q5 reports unavailable or private-only unaudited public publishing; no authorization for live actions | Rejected; manual export is explicit and truthful |
| Treat export as Posted | Reuses an outcome | Claims a remote action that never occurred | Rejected; separate local export operation and receipt |
| Duplicate OAuth and review guards | Adapter-local code | Drifting state, Host, expiry and approval rules | Rejected; shared OAuth primitives and existing review/asset owners |

---

## Scope and boundaries

Single-user loopback API and CLI, instance-scoped Keychain tokens, text and the first rendered PNG with approved slide-derived alt text. No real sign-in/post, automatic status retry, thread splitting, carousel upload, reader, scheduler or video creation. LinkedIn visual dispatch stays contained.

---

## Identities

OpenSpec CLI 1.15.0, config blob `8f976f578610adeefde25096a98101b9f9c2c12c`, Code 5.0.0 with security/data/integrations/llm references. Implementation is on `fm/syncloom-v2-slice5a-mastodon-export-20261031`; the final task report pins the commit after rebase and gates. The endpoint versions are Mastodon v1 apps/statuses and v2 instance/media, not an assumed server release.

---

## Evidence and gaps

- [Q5 card and originals](../research/2026-10-30-q5-tiktok-reddit-mastodon-scope/README.md): Claude-only single-provider model research, reduced confidence. This supports the scope recommendation, not an independent permission or live-success claim.
- [Q8 credentials research](../research/2026-10-30-q8-local-first-auth-and-scheduling/README.md): Keychain and loopback OAuth context, also single-provider reduced confidence.
- [Q6 card and originals](../research/2026-10-30-q6-reading-platforms-for-research/README.md): imported for later intake decisions only; no implementation decision adopted here.
- Official [apps](https://docs.joinmastodon.org/methods/apps/), [OAuth](https://docs.joinmastodon.org/methods/oauth/), [instance](https://docs.joinmastodon.org/methods/instance/), [media](https://docs.joinmastodon.org/methods/media/) and [statuses](https://docs.joinmastodon.org/methods/statuses/) docs were re-opened for request fields and success/processing statuses. No numerical audience or effort estimate is used to decide an implementation rule.

Open gaps: real-instance compatibility, account permissions and effective per-instance limits. Q5's Reddit closure timing is secondary-source reporting of plans that can change; no network implementation relies on those dates. The fake verifies protocol behavior and failure handling, not a real provider's acceptance. No live model-quality evaluation was authorized; the channel-instruction regression uses a fake Messages endpoint.

---

## Consequences

**Positive:** one durable status-creation guard, one approved-asset owner and a truthful local handoff that can be used without API access.

**Negative / Trade-offs:** a failed media attempt can leave an orphaned attachment; unknown status outcomes require human reconciliation. Export files contain approved local content and must be protected as local data. Per-instance app registrations are recreated for a new sign-in instead of persisting app secrets.

---

## References

- [Mastodon and export operator guide](../mastodon-and-export.md)
- [Design](../../openspec/changes/add-mastodon-manual-export/design.md)
