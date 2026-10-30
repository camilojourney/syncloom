# ADR-0001: syncloom.v2 is rebuilt in Rust as a separate repository, with decisions D1-D7

**Date:** 2026-10-30
**Status:** Accepted
**Supersedes:** none
**OpenSpec changes:** [add-studio-slice-1](../../openspec/changes/archive/2026-10-30-add-studio-slice-1/) (slice 1, archived into `openspec/specs/`; D4, D6 and D7 shape it). D1 lands in the review-screen change and D2 in the first platform-integration change when they are proposed
**Agreed:** yes - the captain, 2026-10-30, in [the captain's words](sources/captain-words.md): "yes to all. start building it." after "Let's work on this decision give you you recommendation and I'll accept it"
**Recommended:** none outstanding
**Built:** partly: the repository foundation and slice 1 (one text thought, one model-drafted LinkedIn text, revision-bound approval, contained publish intent, loopback API and CLI over one core) are built and their seven specs archived into `openspec/specs/`; slices 2-5 are not built
**Live-proven:** no - the one live check (a real Claude draft, plan criterion 13) has not run because using a key needs the captain's word; no platform integration exists yet

## Context

Syncloom (formerly Holus) is the captain's thought-to-content studio, a Python repository with a sound core loop (thought, content set, revision-bound review, contained publish intent in a durable outbox) and three weaknesses: fake template drafts, a drifted Social API client, and no spec owning the shipped loop. The evidence is the read-only plan study of 2026-10-29 ([rust-plan](../research/2026-10-29-rust-plan/README.md)). The captain had earlier said the foundation is good but the specs are not, and asked for a Rust rebuild of all of it. Main turned the plan's open points into seven decisions and a recommendation for each; the captain accepted them all, then widened one: the social media integrations are to be rebuilt too, after deep research on each platform's API.

## Decision

The captain's words are in [sources/captain-words.md](sources/captain-words.md). The decisions, as approved:

- **D1 Operator frontend:** the operator studio is a browser page served by the Rust core, phone previews included.
- **D2 Social media integrations:** rebuilt in Rust inside syncloom.v2, one platform at a time, each after deep research on that platform's current API. The old Python Holus Social API (Bluesky, Facebook, Instagram, LinkedIn, Threads, X and YouTube adapters) is a reference, not a dependency. This changes the plan's recommendation ("keep it as an external HTTP boundary for now") by the captain's words "also the integrations wtih al lthe social media apis to make deep research onthem too". The research questions are prepared ([social-api-research-questions](../research/2026-10-30-social-api-research-questions/README.md)); none has run yet.
- **D3 Public portfolio demo:** stays on the old app; not rebuilt now.
- **D4 Model provider and spend:** draft generation uses Claude Sonnet, within USD 5 a day without further approval. Using a key for a real call needs the captain's word.
- **D5 Product identity:** studio first; the autonomous agent fleet is deferred until real performance data exists.
- **D6 Landing authority:** local-only repository, no remote; the same standing landing as invoz.v2 (reviewed, passing, in-scope work lands locally without asking; destructive, security-sensitive or failing work escalates; nothing is pushed without the captain's word).
- **D7 Records:** follow invoz.v2: OpenSpec changes, `docs/research/` with an index and cards, `docs/decisions/` with this template; no root `changes/` folder.
- **Repository:** syncloom.v2 is a separate local repository beside the old one, merged back into the original later, as the captain decided for invoz.v2 and extended here.
- **Process:** the lessons file `data/v2-build-lessons-20261030.md` applies to every build task; OpenSpec 1.15.0 with the Fleet baseline 1.0.0 copied from invoz.v2's configuration ("remember touse the configuaiton we have on invoz for the openspecs").

Behavior belongs to the OpenSpec specs under `openspec/specs/`; this record names no requirement.

## Rationale

| Option | Pros | Cons | Why rejected / chosen |
|---|---|---|---|
| Rebuild in Rust in a separate repository, merge back later (chosen) | Clean scope; the old app keeps running; specs written fresh for shipped behavior | Two repositories until the merge; a ledger to keep | Chosen by the captain ("build it with rust all of it"; separate repository as for invoz.v2) |
| Repair the Python repository in place | No rewrite | Keeps fake drafts, drifted client and spec debt mixed with new work; contradicts "all of it" | Rejected by the captain's words |
| D2 alternative: keep Holus Social API as an external HTTP boundary | Least work now; the plan's recommendation | A second service in Python with a stubbed OAuth flow and a drifted contract; two codebases for one founder | Rejected by the captain on 2026-10-30: the integrations are rebuilt, after research |
| D2 alternative: rebuild integrations before the research | Faster start | The old platform notes are 7-8 months old and unverified; app review and pricing move | Rejected: the captain asked for deep research on them first |
| D1 alternative: native desktop only | Native packaging | No phone preview path without a second server | Rejected (phone previews are wanted) |
| D5 alternative: carry the autonomous strategist now | Nothing deferred | Builds on no performance data; the old architecture itself said not to | Rejected |
| D7 alternative: also a root `changes/` folder | Matches an earlier request for the old repo | Two record systems competing | Rejected: one record system |

## Scope and boundaries

Covers the existence and shape of the syncloom.v2 repository, the seven decisions, the order of work (vision section 6) and the process rules. Preserved: the old Syncloom repository, its specs, its data and the old Holus Social API (read-only references, never edited from here; the ledger in [`docs/migration.md`](../migration.md) tracks each capability). Not covered: any platform integration's design (each waits for its research answer and its own change), the phone access path (Q-ACCESS), rendering (Q7), the merge back (no owner or date), and a real model call or a real post (each needs the captain's word).

## Identities

OpenSpec CLI 1.15.0, stock `spec-driven`, `openspec init --tools codex,claude,pi`. `openspec/config.yaml`: Fleet shared OpenSpec baseline 1.0.0 (fleet-system commit `4501883843df70c50f0726120aca09387ca6c95d`, `system/shared/openspec/baseline.yaml`) copied through invoz.v2's config blob `4027f6d60755ac77927a6f0e5db65a61968fb85b` (invoz.v2 commit `fb6c50691c8f7a6e7b504011bef452bc2b692196`) with only the Syncloom context lines replacing the invoz.v2 ones; see [`docs/development.md`](../development.md). Code skill (Fleet `/code`) version 5.0.0, `SKILL.md` sha256 `bc2574a421d8a41ca9bbe3f2a4edd15818e8d17322055111f4bd958115a8b553`. Old repository pinned at `3efcd35370242a37985fbacb272dacf80b2d42dd`. Rust toolchain 1.98.1 (`rust-toolchain.toml`). The commit that built slice 1 is the tip of branch `fm/syncloom-v2-foundation-slice1-20261030`; the merge records it on `main`.

## Evidence and gaps

- [The captain's words](sources/captain-words.md), unchanged.
- [rust-plan](../research/2026-10-29-rust-plan/README.md): the retain/rebuild/retire map, the seven decisions with their evidence, the slice-1 criteria and the research questions Q-RENDER, Q-UI and Q-ACCESS.
- [social-api-research-questions](../research/2026-10-30-social-api-research-questions/README.md): what the old adapters do today and the eight questions D2 depends on.

Gaps: D2's exact words ("also the integrations wtih al lthe social media apis to make deep research onthem too") were read by Main as "rebuild the integrations in Rust inside syncloom.v2 after research"; the alternative reading, a separate Rust service, was not put to the captain. The daily spend limit of D4 is stated but no spend ledger exists yet (slice 1 records token usage per draft; converting it to dollars needs a price table). The Claude model id pinned in slice 1 is unverified until the live check runs. No inventory of the old repository's runtime data was taken; the record holds no copy of it.

## Consequences

**Positive:**
- One owner per rule from the first commit; shipped behavior has a spec.
- The founder's content never leaves the laptop until a reviewed integration slice says so.
- Integration decisions rest on dated primary sources, not on eight-month-old notes.

**Negative / Trade-offs:**
- Seven platform integrations are now syncloom.v2's own maintenance load, against moving APIs.
- Each integration waits for its research answer; LinkedIn delivery is not in slice 1.
- Decision numbers, research folders and specs may collide with the old repository's at the merge.

## References

- [Vision](../vision.md), [migration ledger](../migration.md), [README](../../README.md)
- Plan report: [docs/research/2026-10-29-rust-plan/report.md](../research/2026-10-29-rust-plan/report.md)
