# Research

Studies done for syncloom.v2, kept with the code they informed. The convention is carried over from invoz.v2 (decision D7 in [decision 0001](../decisions/0001-syncloom-v2-rebuild-decisions.md)): `docs/research/` with an `INDEX.md`, one folder per study, one card per folder.

- [INDEX.md](INDEX.md) - one row per study and the list of pending studies.
- `<date>-<topic>/` - one folder per study. Its `README.md` card holds the question, why it was asked, what would change the answer, how it was collected, its status, the decision it fed and a table of every file with its origin path and sha256.

## Rules

- Reports and sources are copied, never rewritten. Third-party text stays unchanged. Large HTML captures, keys and audio are not imported.
- A new study adds its folder, its card and its index row in one commit. A study that feeds a decision is cited in that decision's "Evidence and gaps"; a decision is written by copying [`docs/decisions/0000-template.md`](../decisions/0000-template.md).
- A research answer that an integration or a slice depends on is saved here, word for word, before that slice is specified (vision 3.2).
- `python3 -I scripts/check-research-records.py` fails when a folder has no row, a row has no folder, a file is missing from its card or its sha256 differs, a decision cites no evidence, or a link is broken.
