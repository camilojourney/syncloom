# ADR-0005: Owned Page and linked Instagram delivery

**Date:** 2026-10-31
**Status:** Proposed
**Supersedes:** none
**OpenSpec changes:** [add-meta-publishing](../../openspec/changes/archive/2026-10-30-add-meta-publishing/)
**Agreed:** no - implementation brief authorizes fake-only work, not live actions or hosting
**Recommended:** Facebook Login for Business, Page token, explicit public-media port
**Built:** fake-proven implementation on the task branch; not yet landed on main
**Live-proven:** no

## Context

Q3 answers vision 3.2's research prerequisite for slice 5 Meta channels.

## Decision

Use [meta-integration](../../openspec/changes/archive/2026-10-30-add-meta-publishing/specs/meta-integration/spec.md), with platform-credentials and dispatch-outbox retaining ownership of secrets and repeat safety.

## Rationale

| Option | Pros | Cons | Why rejected / chosen |
|---|---|---|---|
| Facebook Login for Business, user-token configuration and derived Page token | One consent covers both targets, non-expiring Page token | Page link and owned-account roles required | Chosen for the stated accounts |
| Instagram Login | No Page link needed | Does not cover Facebook Page | Rejected for this slice |
| Unified Login/system user | Automated business use | Additional setup and eligibility not proven | Deferred |
| Explicit media delivery port | Honest local-only default and fake proof | Real media delivery unavailable without hosting | Chosen instead of inventing a host |

## Scope and boundaries

Page text/photo and Instagram one-to-ten rendered JPEG slides. No real Meta app, consent or post; no video, insights, native scheduling or hosting. Containers are created only during local dispatch. App secret is operator-supplied, never embedded in binaries.

## Identities

OpenSpec 1.15.0; config blob 8f976f578610adeefde25096a98101b9f9c2c12c; Code 5.0.0 (design, security, data, integrations, ui). Graph v26.0, matching re-opened official examples rather than the report's v25.0 recommendation. Building commit is recorded by git history.

## Evidence and gaps

[Q3 card](../research/2026-10-30-q3-meta-facebook-instagram/README.md) records immutable reports, hashes and current official-doc checks. ChatGPT was hand-collected with zero sources scanned, not independent corroboration. [Q8](../research/2026-10-30-q8-local-first-auth-and-scheduling/README.md) informs local credentials/outbox policy.

Standard Access is sufficient for owned accounts in the official overview, not evidence of real app configuration. Development-mode posts may only be visible to app roles. Current manual-flow/security docs do not prove HTTP loopback acceptance for a real app; this is a live-use prerequisite, not claimed proven by a fake.

Open question: which public media host, retention duration and deletion policy is acceptable? The production media interface refuses until a separate approved implementation exists.

## Consequences

**Positive:** shared consent, approval and outbox; fake-only proof is distinguishable from live proof.

**Negative / Trade-offs:** media cannot be delivered in production yet; real app eligibility, redirect acceptance and public visibility require exact-action permission and separate proof.

## References

- [Design](../../openspec/changes/archive/2026-10-30-add-meta-publishing/design.md)

## Dated outcome/linkage review: 2026-11-02

The original branch/not-landed field is historical. Implementation is present in local history at `809c77b`: [Meta publish](../../crates/syncloom-core/src/meta/publish.rs), [fake HTTP carousel/repeat/unknown tests](../../crates/syncloom-server/tests/meta_api.rs). Q3 native incomplete/original-limits remains incomplete after HAND retention. The [media-hosting request](../research/2026-10-31-meta-media-hosting/README.md) has unavailable original result custody; it supplies no finding or permission to choose public hosting, retention, budget or real app setup.

This is source/history inspection, not a fresh platform smoke or newly accepted rationale. Status, agreed boundaries and historical method identity remain unchanged: this record was not made through the current Decide method. [research/0001](research/0001-research-custody-and-retrieval.md) makes only the new reversible custody/retrieval choice and links its executed local evaluation.
