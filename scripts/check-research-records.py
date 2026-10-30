#!/usr/bin/env python3
"""Check the research index and the decision records under docs/.

usage: check-research-records.py [--root DIR]

Research (docs/research/): every study folder has a row in INDEX.md and every row has a folder; the study
card (README.md) has its sections and its Status matches the index row; every file in a study is listed in
the card with its sha256 (a 64-hex fingerprint in a file name must equal it).
Decisions (docs/decisions/NNNN-*.md, from the 0000-template.md form): a Status of Draft, Proposed, Accepted,
Superseded or Deprecated; the header fields and sections of the template; "Evidence and gaps" links an existing
file under docs/research/ or docs/decisions/sources/; "OpenSpec changes" links existing change directories
whose design.md names the decision (or says "none - <reason>"); a Superseded decision links its successor;
a decision named by an index row cites that study; numbers run without gaps; source files are listed with
their sha256. Relative links in these records resolve.
Exit 0 when no problem is found, 1 otherwise. Standard library only.
"""
import hashlib
import pathlib
import re
import sys

CARD_SECTIONS = ["Question", "Why asked", "What would change the answer", "How collected", "Status", "Decision", "Files"]
DECISION_FIELDS = ["Date", "Status", "Supersedes", "OpenSpec changes", "Agreed", "Recommended", "Built", "Live-proven"]
DECISION_SECTIONS = ["Context", "Decision", "Rationale", "Scope and boundaries", "Identities", "Evidence and gaps", "Consequences", "References"]
DECISION_STATUSES = ["Draft", "Proposed", "Accepted", "Superseded", "Deprecated"]
DECISION_FILE = re.compile(r"^(\d{4})-[a-z0-9-]+\.md$")
FILE_ROW = re.compile(r"^\|\s*`([^`]+)`\s*\|\s*`([^`]+)`\s*\|\s*`([0-9a-f]{64})`\s*\|\s*$")
LINK = re.compile(r"\[[^\]]*\]\(([^)\s]+)\)")
FINGERPRINT = re.compile(r"[0-9a-f]{64}")
NONE_WITH_REASON = re.compile(r"^none - \S", flags=re.I)


def sections(text):
    """Map each '## ' heading to its body."""
    parts = re.split(r"^## (.+)$", text, flags=re.M)
    return {parts[i].strip(): parts[i + 1].strip() for i in range(1, len(parts) - 1, 2)}


def header_fields(text):
    """Map each '**Name:** value' line above the first '## ' heading to its value."""
    head = re.split(r"^## ", text, maxsplit=1, flags=re.M)[0]
    return {m.group(1): m.group(2).strip() for m in re.finditer(r"^\*\*([^*:]+):\*\*[ \t]*(.*)$", head, flags=re.M)}


def card_status(text):
    match = re.search(r"^Status:\s*(.+)$", text, flags=re.M)
    return match.group(1).strip() if match else None


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def link_targets(text, base):
    """(resolved path, raw link) for each relative link in text, resolved against the directory base."""
    found = []
    for raw in LINK.findall(text):
        if re.match(r"^[a-z][a-z0-9+.-]*:", raw) or raw.startswith("#"):
            continue
        found.append(((base / raw.split("#")[0]).resolve(), raw))
    return found


def check_links(root, path, text):
    shown = path.relative_to(root).as_posix()
    return [f"{shown}: broken link {raw}" for target, raw in link_targets(text, path.parent) if not target.exists()]


def listed_files(text):
    return {m.group(1): m.group(3) for m in map(FILE_ROW.match, text.splitlines()) if m}


def check_listed(label, directory, listed):
    problems = []
    for rel, expected in listed.items():
        path = directory / rel
        if not path.is_file():
            problems.append(f"{label}: {rel}: listed file does not exist")
            continue
        actual = sha256(path)
        embedded = FINGERPRINT.search(path.name)
        if actual != expected:
            problems.append(f"{label}: {rel}: sha256 differs from the recorded {expected[:12]}")
        if embedded and embedded.group(0) != actual:
            problems.append(f"{label}: {rel}: file name fingerprint differs from the content hash")
    return problems


def check_files_table(label, directory, text, ignore):
    """Every file under directory (except ignore) is listed with its sha256, and every listed file matches."""
    listed = listed_files(text)
    problems = check_listed(label, directory, listed)
    for path in sorted(directory.rglob("*")):
        rel = path.relative_to(directory).as_posix()
        if path.is_file() and rel not in listed and not ignore(rel):
            problems.append(f"{label}: {rel}: file is not listed")
    return problems


def check_card(root, study, row_status):
    card = study / "README.md"
    if not card.is_file():
        return [f"{study.name}: study folder has no README.md card"]
    text = card.read_text()
    body = sections(text)
    problems = [f"{study.name}: missing section '{name}'" for name in CARD_SECTIONS if not body.get(name)]
    if card_status(text) != row_status:
        problems.append(f"{study.name}: index status '{row_status}' differs from card status '{card_status(text)}'")
    problems += check_files_table(study.name, study, text, lambda rel: rel == "README.md")
    return problems + check_links(root, card, text)


def index_rows(text):
    """(study folder, status, decision link target or None) for each study row above the pending studies."""
    rows = []
    for line in text.split("## Pending studies")[0].splitlines():
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        match = re.match(r"^\[[^\]]+\]\(([^/)]+)/README\.md\)$", cells[0]) if len(cells) == 7 else None
        if match:
            decision = LINK.search(cells[6])
            rows.append((match.group(1), cells[5], decision.group(1) if decision else None))
    return rows


def evidence_names(path, folder):
    """True when the decision's evidence section links into the study folder."""
    evidence = sections(path.read_text()).get("Evidence and gaps", "")
    return any(folder + "/" in raw for _, raw in link_targets(evidence, path.parent))


def check_index_row(root, folders, row):
    name, status, decision = row
    if name not in folders:
        return [f"{name}: index row has no study folder"]
    problems = check_card(root, folders[name], status)
    target = (root / "docs" / "research" / decision).resolve() if decision else None
    if target and target.is_file() and not evidence_names(target, name):
        problems.append(f"{target.name}: Evidence and gaps does not cite study {name}, but the index row names this decision")
    return problems


def check_index(root):
    research = root / "docs" / "research"
    index = research / "INDEX.md"
    if not index.is_file():
        return ["docs/research/INDEX.md: missing"]
    text = index.read_text()
    rows = index_rows(text)
    folders = {p.name: p for p in research.iterdir() if p.is_dir() and not p.name.startswith(".")}
    problems = check_links(root, index, text)
    for name in sorted(set(folders) - {row[0] for row in rows}):
        problems.append(f"{name}: study folder has no row in docs/research/INDEX.md")
    for row in rows:
        problems += check_index_row(root, folders, row)
    return problems


def evidence_roots(root):
    return [(root / "docs" / "research").resolve(), (root / "docs" / "decisions" / "sources").resolve()]


def check_evidence(root, name, body):
    cited = [t for t, _ in link_targets(body, root / "docs" / "decisions") if t.exists()]
    if any(base in t.parents for t in cited for base in evidence_roots(root)):
        return []
    return [f"{name}: Evidence and gaps cites no file under docs/research/ or docs/decisions/sources/"]


def check_changes(root, name, value):
    """The OpenSpec changes field links existing change directories whose design.md names this decision."""
    if NONE_WITH_REASON.match(value):
        return []
    targets = link_targets(value, root / "docs" / "decisions")
    if not targets:
        return [f"{name}: OpenSpec changes needs a link to a change or 'none - <reason>'"]
    problems = []
    for target, raw in targets:
        design = target / "design.md"
        if not design.is_file():
            problems.append(f"{name}: OpenSpec changes link {raw} has no design.md")
        elif f"docs/decisions/{name}" not in design.read_text():
            problems.append(f"{name}: design.md of {raw} does not name docs/decisions/{name}")
    return problems


def check_succession(name, fields):
    status = fields.get("Status")
    if status == "Superseded" and not LINK.search(fields.get("Superseded by", "")):
        return [f"{name}: a Superseded decision needs a 'Superseded by' link"]
    return []


def check_decision(root, path):
    text = path.read_text()
    body = sections(text)
    fields = header_fields(text)
    name = path.name
    problems = [f"{name}: missing field '{f}'" for f in DECISION_FIELDS if not fields.get(f)]
    problems += [f"{name}: missing section '{s}'" for s in DECISION_SECTIONS if not body.get(s)]
    if fields.get("Status") not in DECISION_STATUSES:
        problems.append(f"{name}: Status must be one of {', '.join(DECISION_STATUSES)}")
    problems += check_succession(name, fields)
    problems += check_evidence(root, name, body.get("Evidence and gaps", ""))
    problems += check_changes(root, name, fields.get("OpenSpec changes", "none - missing"))
    return problems + check_links(root, path, text)


def check_decisions(root):
    directory = root / "docs" / "decisions"
    if not directory.is_dir():
        return ["docs/decisions/: missing"]
    numbered = sorted(p for p in directory.iterdir() if DECISION_FILE.match(p.name) and not p.name.startswith("0000-"))
    numbers = [int(DECISION_FILE.match(p.name).group(1)) for p in numbered]
    problems = []
    if numbers != list(range(1, len(numbers) + 1)):
        problems.append(f"docs/decisions/: decision numbers must run 0001, 0002, ... without gaps; found {numbers}")
    for path in numbered:
        problems += check_decision(root, path)
    readme = directory / "README.md"
    if not readme.is_file():
        return problems + ["docs/decisions/README.md: missing"]
    text = readme.read_text()
    is_record = lambda rel: rel == "README.md" or bool(DECISION_FILE.match(rel))
    return problems + check_links(root, readme, text) + check_files_table("docs/decisions", directory, text, is_record)


def check(root):
    root = pathlib.Path(root)
    return check_index(root) + check_decisions(root)


def main(argv):
    root = pathlib.Path(__file__).resolve().parent.parent
    if "--root" in argv:
        root = pathlib.Path(argv[argv.index("--root") + 1])
    problems = check(root)
    for problem in problems:
        print(problem)
    print(f"{len(problems)} problem(s)")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
