# Decisions

Decision records for syncloom.v2. Historical flat `NNNN-<topic>.md` identities remain unchanged. New records use `<domain>/NNNN-<topic>.md`, numbered without gaps within each domain, with the project fields from [`0000-template.md`](0000-template.md) and the actual loaded Decide method interface. The historical template is a byte-identical copy of invoz.v2's (D7 in [0001](0001-syncloom-v2-rebuild-decisions.md)). A genuine current use records `Method: decide@<version>`; never add that marker retroactively to an old ADR.

| Decision | Status |
|---|---|
| [0001 syncloom.v2 is rebuilt in Rust as a separate repository, with decisions D1-D7](0001-syncloom-v2-rebuild-decisions.md) | Accepted |
| [0002 LinkedIn personal-profile integration: self-serve Share on LinkedIn, Keychain tokens, loopback sign-in](0002-linkedin-integration-decisions.md) | Proposed |
| [0003 Pinned headless Chromium for local images and document carousels](0003-visual-renderer.md) | Proposed |
| [0004 Mastodon API add-on and manual handoff target](0004-mastodon-manual-export.md) | Proposed |
| [0005 Owned Facebook Page and linked Instagram delivery](0005-meta-integration.md) | Proposed |
| [0006 Bluesky, X and Threads publishing](0006-social-platform-integrations.md) | Proposed |
| [research/0001 Original study custody and direct retrieval](research/0001-research-custody-and-retrieval.md) | Accepted within delegated reversible engineering scope, decide@1.3.1; not product acceptance |
| [posting-permissions/0001 Effective posting grants](posting-permissions/0001-effective-grants.md) | Accepted bounded repair, decide@1.3.1; late existing-context Main1360 challenge, security landing pending |
| [frontend/0007 Small Rust server-rendered text studio](frontend/0007-text-studio-frontend-stack.md) | Delegated implementation selection, decide@1.3.1 |
| [frontend/0008 Provisional mineral-botanical palette/type](frontend/0008-text-studio-palette-and-typography.md) | Provisional direction, not captain taste acceptance, decide@1.3.1 |
| [attachments/0001 Bounded local text-file intake](attachments/0001-local-text-scope.md) | Accepted ordinary sequencing by firstmate, decide@1.3.2; exact-head security acceptance remains separate |
| [visual-quality/0001 Local technical evidence, not a quality grade](visual-quality/0001-local-assessment.md) | Proposed reversible advisory implementation, decide@1.3.2; semantic judge not evaluated |

The frontend identities were allocated in the frozen design before this import and are preserved, not renumbered. Their unchanged original packet and challenge remain in the [design history archive](../design/text-studio-history.zip).

## Rules

- A decision records what was decided, why, the real alternatives rejected, its scope and boundaries, the exact identities it used, and its evidence and unresolved gaps. It does not restate requirements: the OpenSpec specs own behavior.
- Four fields stay apart: **Agreed** (someone with the authority said yes, and where their words are), **Recommended** (advice not yet agreed), **Built** (landed in code, with the commit) and **Live-proven** (shown against the real service). Never fold one into another.
- **OpenSpec changes** links the change directories that own the behavior; each change's `design.md` names the complete repository-relative decision identity, including its domain when present. The decision links forward, the design links back. Current Code Verify owns method-record claims/triggers/citations, including nested records. The adopted legacy local checker structurally checks flat records only; nested files receive per-file custody hashes below, not a claimed legacy structural pass.
- **Evidence and gaps** links at least one existing file under [`docs/research/`](../research/INDEX.md) or `sources/` below.
- Run `python3 -B -I scripts/check-research-records.py` and its tests for legacy flat-record fields, local links and file custody hashes. Run current Code Verify for method-record structure/citations; neither check authenticates authority or establishes semantic completeness.

## Source files

Text copied from outside the repository. `captain-words.md` is a quotation file written here from the task brief; the sha256 is checked. The checklist and frozen brief are unchanged pinned sources.

| File | Origin | sha256 |
|---|---|---|
| `sources/captain-words.md` | `quoted from the firstmate task brief syncloom-v2-foundation-slice1-20261030 (section "Captain's intent") and the vision draft data/syncloom-v2-vision-draft-20261030/vision-draft.md` | `f950d69f92467d6037061558c2b4fc2446d275263896319243bc4421065140f6` |
| `sources/decide-question-checklist-1.0.0.md` | `Fleet baa7c65c88fa04eaa2d2167e78b24ff56e575e68 system/skills/platform/decide/references/question-checklist.md, version 1.0.0; copied unchanged` | `8c88c2c9c524d97552f67f30f6c4ba25589516b9dc18a51b09f7962688e5b056` |
| `sources/research-custody-freeze.md` | `syncloom-v2-lab/data/syncloom-research-adoption/decision-freeze.md; before assessment/implementation, including retained timestamp correction` | `4bc08077fd70e57cc319ca426b55cfad5a04c9a7d4d30cbfe285faaeed47deb3` |

## Domain record custody (legacy checker scope)

The immutable legacy checker treats nested records as files needing a listed hash. This row satisfies that existing custody rule only. Current Code Verify, not the legacy flat-record parser, checks the method record's structure/citations. It is a decision record, not an external source or proof of semantic approval.

| File | Origin | sha256 |
|---|---|---|
| `research/0001-research-custody-and-retrieval.md` | `project-authored domain decision; structure owned by current Code Verify` | `4b77fce5d2127ae7a842744697afe401572c4294b456d45128a6ccb2ad8963ff` |
| `frontend/0007-text-studio-frontend-stack.md` | `project-authored import of completed decide packet; original bytes retained in design archive` | `9212471c0651d080b4590d1da92fdafed7e950d48a72cd7517fb1021630f2654` |
| `frontend/0008-text-studio-palette-and-typography.md` | `project-authored import of completed decide packet; original bytes retained in design archive` | `419c967692c259ee27b364d86bfa9992f9b592b5824127abfd8333ea8ad7622a` |
| `posting-permissions/0001-effective-grants.md` | `project-authored domain decision; structure owned by current Code Verify` | `90d2b1de78a64fd90f11cc9a4e13ed7691303afee39da46db592a2262ec712a2` |
| `attachments/0001-local-text-scope.md` | `project-authored domain decision; structure owned by current Code Verify` | `0e7fe5d202adcc82756c4f7a33e328b4b37e3b85919baee20d4232b702fb2695` |
| `visual-quality/0001-local-assessment.md` | `project-authored domain decision; structure owned by current Code Verify` | `a7f3f15cd739efd787ce71ab81c206003d27ad147e1d49b4c5b13f094235abb0` |
