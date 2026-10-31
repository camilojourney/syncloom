#!/usr/bin/env python3
"""Check an OpenSpec change against the openspec/config.yaml rules that `openspec validate` does not enforce.

usage: check-change-shape.py <change dir> [--request-source FILE]

Checks: one SHALL sentence of at most 30 words and at least one scenario per requirement; unique
requirement names; Request section with quoted paragraphs; Approval pending or attributed approved; capability and
"Duplicate kept" lines; Open questions; proposal body under 250 words; design References line;
task 1.1 "Load the code skill".

--request-source FILE holds the requester's paragraphs, separated by blank lines. With it, every
paragraph must appear verbatim as a "> " quote in the proposal. Without it, only the presence and
count of quoted paragraphs are reported (verbatim cannot be proven).
Exit 0 when no problem is found, 1 otherwise. Standard library only.

Fleet checker 1.0.1, owned here from kit 1.0.1. Imported from invoz.v2
ddd9880ef316d7cac7b4d87be84e480fd148e237:scripts/check-change-shape.py
(SHA256 54f77643ee4e4cdc3cfc9f66b20cda5413efc9dfd95982532d58e62fd14b74f2).
Shape validation does not authenticate the reviewer or grant approval.
"""
import pathlib
import re
import sys

MAX_SENTENCE_WORDS = 30
MAX_BODY_WORDS = 249


def check_requirement(name, rest):
    sentence = rest.strip().split("\n")[0]
    problems = []
    if len(re.findall(r"\bSHALL\b", sentence)) != 1:
        problems.append(f"[{name}] needs exactly one SHALL")
    if len(sentence.split()) > MAX_SENTENCE_WORDS:
        problems.append(f"[{name}] sentence is {len(sentence.split())} words (max {MAX_SENTENCE_WORDS})")
    if "#### Scenario:" not in rest:
        problems.append(f"[{name}] has no scenario")
    return problems


def check_specs(change):
    problems, notes = [], []
    for spec in sorted((change / "specs").rglob("spec.md")):
        blocks = re.split(r"\n### Requirement: ", spec.read_text())[1:]
        names = [block.partition("\n")[0] for block in blocks]
        for block in blocks:
            name, _, rest = block.partition("\n")
            problems += check_requirement(name, rest)
        if len(names) != len(set(names)):
            problems.append(f"{spec}: duplicate requirement names")
        notes.append(f"{spec.parent.name}: {len(blocks)} requirements")
    return problems, notes


def _quoted_paragraphs(request):
    quotes, lines = [], []
    # Remove only quote prefixes; internal newlines and content must stay exact.
    for line in request.splitlines() + [""]:
        if line.startswith("> ") and line[2:]:
            lines.append(line[2:])
        elif lines:
            quotes.append("\n".join(lines))
            lines = []
    return quotes


def check_request(request, source):
    quotes = _quoted_paragraphs(request)
    problems = [] if quotes else ["Request quotes nothing"]
    if source is not None:
        paragraphs = [p.strip() for p in source.read_text().split("\n\n") if p.strip()]
        problems += ["Request paragraph not verbatim: " + p[:60] for p in paragraphs if p not in quotes]
    return problems, [f"Request quotes {len(quotes)} paragraphs"]


# [impl->req~shared-approval-shape.reviewer-path~1]
def check_body(body):
    problems = []
    if not re.search(r"^(?:pending|approved: \S[^\n]*)$", body, re.M):
        problems.append("Approval needs pending or approved: <reviewer>")
    if not any(token in body for token in ("New capability requested:", "New capability approved:")):
        problems.append("proposal lacks 'New capability requested:' or 'New capability approved:'")
    for needle in ("Duplicate kept:", "## Open questions"):
        if needle not in body:
            problems.append(f"proposal lacks '{needle}'")
    words = len(body.split())
    if words > MAX_BODY_WORDS:
        problems.append(f"proposal body is {words} words (max {MAX_BODY_WORDS})")
    return problems, [f"proposal body outside the Request quote: {words} words"]


def check_proposal(change, source):
    proposal = (change / "proposal.md").read_text()
    problems = [] if proposal.startswith("## Request") else ["proposal must start with a Request section"]
    request, _, body = proposal.partition("## Approval")
    found, notes = check_request(request, source)
    more, more_notes = check_body(body)
    return problems + found + more, notes + more_notes


def check_files(change):
    problems = []
    if not re.search(r"^References: .+", (change / "design.md").read_text(), re.M):
        problems.append("design lacks a References line")
    if "1.1 Load the code skill" not in (change / "tasks.md").read_text():
        problems.append("task 1.1 is not 'Load the code skill'")
    return problems


def parse_args(argv):
    source = None
    if "--request-source" in argv:
        index = argv.index("--request-source")
        source = pathlib.Path(argv[index + 1])
        argv = argv[:index] + argv[index + 2:]
    return (pathlib.Path(argv[0]) if len(argv) == 1 else None), source


def main(argv):
    change, source = parse_args(argv)
    if change is None:
        print(__doc__)
        return 2
    problems, notes = check_specs(change)
    found, more_notes = check_proposal(change, source)
    problems += found + check_files(change)
    for note in notes + more_notes:
        print("note:", note)
    for problem in problems:
        print("PROBLEM:", problem)
    print(f"problems: {len(problems)}")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
