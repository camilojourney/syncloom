# Decisions

Decision records for syncloom.v2, one file per decision: `NNNN-<topic>.md`, numbered in order and never renumbered, written from [`0000-template.md`](0000-template.md). The convention is carried over from invoz.v2 (decision D7 in [0001](0001-syncloom-v2-rebuild-decisions.md)); the template is a byte-identical copy of invoz.v2's.

| Decision | Status |
|---|---|
| [0001 syncloom.v2 is rebuilt in Rust as a separate repository, with decisions D1-D7](0001-syncloom-v2-rebuild-decisions.md) | Accepted |
| [0002 LinkedIn personal-profile integration: self-serve Share on LinkedIn, Keychain tokens, loopback sign-in](0002-linkedin-integration-decisions.md) | Proposed |
| [0003 Pinned headless Chromium for local images and document carousels](0003-visual-renderer.md) | Proposed |
| [0004 Mastodon API add-on and manual handoff target](0004-mastodon-manual-export.md) | Proposed |

| [0005 Owned Facebook Page and linked Instagram delivery](0005-meta-integration.md) | Proposed |
| [0006 Bluesky, X and Threads publishing](0006-social-platform-integrations.md) | Proposed |

## Rules

- A decision records what was decided, why, the real alternatives rejected, its scope and boundaries, the exact identities it used, and its evidence and unresolved gaps. It does not restate requirements: the OpenSpec specs own behavior.
- Four fields stay apart: **Agreed** (someone with the authority said yes, and where their words are), **Recommended** (advice not yet agreed), **Built** (landed in code, with the commit) and **Live-proven** (shown against the real service). Never fold one into another.
- **OpenSpec changes** links the change directories that own the behavior; each change's `design.md` names the decision as `docs/decisions/NNNN-<topic>.md`. The decision links forward, the design links back.
- **Evidence and gaps** links at least one existing file under [`docs/research/`](../research/INDEX.md) or `sources/` below.
- `python3 -I scripts/check-research-records.py` checks all of this. Run it, and its tests, with the other checks.

## Source files

Text copied from outside the repository. `captain-words.md` is a quotation file written here from the task brief; the sha256 is checked.

| File | Origin | sha256 |
|---|---|---|
| `sources/captain-words.md` | `quoted from the firstmate task brief syncloom-v2-foundation-slice1-20261030 (section "Captain's intent") and the vision draft data/syncloom-v2-vision-draft-20261030/vision-draft.md` | `f950d69f92467d6037061558c2b4fc2446d275263896319243bc4421065140f6` |
