#!/usr/bin/env python3
"""Tests for check-research-records.py: each rule fails on a tree that breaks only that rule."""
import hashlib
import importlib.util
import pathlib
import shutil
import subprocess
import sys
import tempfile
import unittest

HERE = pathlib.Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("check_research_records", HERE / "check-research-records.py")
checker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checker)

REPORT = "report body\n"
REPORT_SHA = hashlib.sha256(REPORT.encode()).hexdigest()
FINGERPRINTED = f"claude-{REPORT_SHA}.txt"

CARD = """# Study

Status: complete

## Question

q

## Why asked

w

## What would change the answer

c

## How collected

h

## Status

s

## Decision

d

## Files

| File | Origin | sha256 |
|---|---|---|
| `report.md` | `/origin/report.md` | `{sha}` |
"""

INDEX = """# Research index

| Study | Date | Question | Why asked | How collected | Status | Decision |
|---|---|---|---|---|---|---|
| [topic](2026-01-01-topic/README.md) | 2026-01-01 | q | w | h | complete | [0001](../decisions/0001-pick.md) |
"""

DECISION = """# ADR-0001: Pick

**Date:** 2026-01-01
**Status:** Accepted
**Supersedes:** none
**OpenSpec changes:** [add-x](../../openspec/changes/archive/2026-01-01-add-x/)
**Agreed:** yes - the owner
**Recommended:** none
**Built:** not built
**Live-proven:** no

## Context

c

## Decision

d

## Rationale

r

## Scope and boundaries

s

## Identities

i

## Evidence and gaps

- [the study](../research/2026-01-01-topic/README.md)

## Consequences

c

## References

r
"""

DECISIONS_README = """# Decisions

## Source files

| File | Origin | sha256 |
|---|---|---|
"""


def build(root):
    study = root / "docs" / "research" / "2026-01-01-topic"
    study.mkdir(parents=True)
    (study / "report.md").write_text(REPORT)
    (study / "README.md").write_text(CARD.format(sha=REPORT_SHA))
    (root / "docs" / "research" / "INDEX.md").write_text(INDEX)
    (root / "docs" / "decisions").mkdir()
    (root / "docs" / "decisions" / "README.md").write_text(DECISIONS_README)
    (root / "docs" / "decisions" / "0001-pick.md").write_text(DECISION)
    change = root / "openspec" / "changes" / "archive" / "2026-01-01-add-x"
    change.mkdir(parents=True)
    (change / "design.md").write_text("Decisions: `docs/decisions/0001-pick.md`\n")


class CheckResearchRecords(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp()
        self.root = pathlib.Path(self.tmp)
        build(self.root)

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def edit(self, rel, old, new):
        path = self.root / rel
        text = path.read_text()
        self.assertIn(old, text)
        path.write_text(text.replace(old, new))

    def problems(self):
        return checker.check(self.root)

    def assert_problem(self, fragment):
        found = self.problems()
        self.assertTrue(any(fragment in p for p in found), f"{fragment!r} not in {found}")

    def test_a_consistent_tree_has_no_problem(self):
        self.assertEqual(self.problems(), [])

    def test_decision_without_evidence_link_fails(self):
        self.edit("docs/decisions/0001-pick.md", "- [the study](../research/2026-01-01-topic/README.md)", "reasoning only")
        self.assert_problem("Evidence and gaps cites no file under docs/research/")

    def test_decision_evidence_may_be_a_decision_source_file(self):
        sources = self.root / "docs" / "decisions" / "sources"
        sources.mkdir()
        (sources / "words.md").write_text("words\n")
        row = f"| `words.md` | `/origin/words.md` | `{hashlib.sha256(b'words\n').hexdigest()}` |\n"
        with open(self.root / "docs" / "decisions" / "README.md", "a") as readme:
            readme.write(row.replace("`words.md`", "`sources/words.md`"))
        self.edit("docs/decisions/0001-pick.md", "../research/2026-01-01-topic/README.md", "sources/words.md")
        self.edit("docs/research/INDEX.md", "[0001](../decisions/0001-pick.md)", "none yet")
        self.assertEqual(self.problems(), [])

    def test_decision_evidence_pointing_at_a_missing_file_fails(self):
        self.edit("docs/decisions/0001-pick.md", "topic/README.md", "topic/gone.md")
        self.assert_problem("broken link")

    def test_study_folder_missing_from_index_fails(self):
        (self.root / "docs" / "research" / "2026-01-02-other").mkdir()
        self.assert_problem("2026-01-02-other: study folder has no row in docs/research/INDEX.md")

    def test_index_row_without_folder_fails(self):
        self.edit("docs/research/INDEX.md", "| [topic](2026-01-01-topic/README.md)", "| [topic](2026-01-09-ghost/README.md)")
        self.assert_problem("2026-01-09-ghost: index row has no study folder")

    def test_altered_file_fails(self):
        (self.root / "docs" / "research" / "2026-01-01-topic" / "report.md").write_text("changed\n")
        self.assert_problem("sha256 differs")

    def test_file_missing_from_card_fails(self):
        (self.root / "docs" / "research" / "2026-01-01-topic" / "extra.txt").write_text("x")
        self.assert_problem("extra.txt: file is not listed")

    def test_card_listing_a_missing_file_fails(self):
        (self.root / "docs" / "research" / "2026-01-01-topic" / "report.md").unlink()
        self.assert_problem("report.md: listed file does not exist")

    def test_fingerprint_in_file_name_must_equal_content_hash(self):
        study = self.root / "docs" / "research" / "2026-01-01-topic"
        (study / "report.md").rename(study / FINGERPRINTED)
        self.edit("docs/research/2026-01-01-topic/README.md", "`report.md`", f"`{FINGERPRINTED}`")
        self.assertEqual(self.problems(), [])
        (study / FINGERPRINTED).write_text("different\n")
        card = CARD.format(sha=hashlib.sha256(b"different\n").hexdigest()).replace("`report.md`", f"`{FINGERPRINTED}`")
        (study / "README.md").write_text(card)
        self.assert_problem("file name fingerprint differs")

    def test_card_section_missing_fails(self):
        self.edit("docs/research/2026-01-01-topic/README.md", "## How collected\n\nh\n\n", "")
        self.assert_problem("missing section 'How collected'")

    def test_index_status_must_match_card(self):
        self.edit("docs/research/INDEX.md", "| complete |", "| open |")
        self.assert_problem("status 'open' differs from card status 'complete'")

    def test_index_decision_link_must_resolve(self):
        self.edit("docs/research/INDEX.md", "0001-pick.md", "0009-none.md")
        self.assert_problem("broken link")

    def test_decision_named_by_index_must_cite_the_study(self):
        self.edit("docs/decisions/0001-pick.md", "../research/2026-01-01-topic/README.md", "../research/INDEX.md")
        self.assert_problem("does not cite study 2026-01-01-topic")

    def test_decision_status_must_be_known(self):
        self.edit("docs/decisions/0001-pick.md", "**Status:** Accepted", "**Status:** Maybe")
        self.assert_problem("Status must be")

    def test_superseded_decision_needs_its_successor_link(self):
        self.edit("docs/decisions/0001-pick.md", "**Status:** Accepted", "**Status:** Superseded")
        self.assert_problem("needs a 'Superseded by' link")
        self.edit("docs/decisions/0001-pick.md", "**Status:** Superseded", "**Status:** Superseded\n**Superseded by:** [0002](0002-next.md)")
        self.assertFalse(any("Superseded by" in p for p in self.problems()))

    def test_decision_header_field_missing_fails(self):
        self.edit("docs/decisions/0001-pick.md", "**Live-proven:** no\n", "")
        self.assert_problem("missing field 'Live-proven'")

    def test_decision_section_missing_fails(self):
        self.edit("docs/decisions/0001-pick.md", "## Identities\n\ni\n\n", "")
        self.assert_problem("missing section 'Identities'")

    def test_decision_numbers_have_no_gap(self):
        (self.root / "docs" / "decisions" / "0001-pick.md").rename(self.root / "docs" / "decisions" / "0003-pick.md")
        self.edit("docs/research/INDEX.md", "0001-pick.md", "0003-pick.md")
        self.assert_problem("decision numbers must run 0001, 0002")

    def test_the_template_is_not_a_numbered_decision(self):
        (self.root / "docs" / "decisions" / "0000-template.md").write_text("# ADR-0000: [Title]\n")
        self.assertEqual(self.problems(), [])

    def test_openspec_changes_needs_a_change_link_or_none_with_reason(self):
        self.edit("docs/decisions/0001-pick.md", "[add-x](../../openspec/changes/archive/2026-01-01-add-x/)", "later")
        self.assert_problem("OpenSpec changes needs a link")
        self.edit("docs/decisions/0001-pick.md", "later", "none - no behavior changes")
        self.assertEqual(self.problems(), [])

    def test_change_design_must_link_back_to_the_decision(self):
        design = self.root / "openspec" / "changes" / "archive" / "2026-01-01-add-x" / "design.md"
        design.write_text("no backlink\n")
        self.assert_problem("design.md of ../../openspec/changes/archive/2026-01-01-add-x/ does not name docs/decisions/0001-pick.md")

    def test_broken_relative_link_in_a_decision_fails(self):
        self.edit("docs/decisions/0001-pick.md", "\n## Context\n\nc", "\n## Context\n\n[x](../nowhere.md)")
        self.assert_problem("broken link")

    def test_decision_source_file_needs_its_hash(self):
        (self.root / "docs" / "decisions" / "sources").mkdir()
        (self.root / "docs" / "decisions" / "sources" / "c.md").write_text("contract\n")
        self.assert_problem("c.md: file is not listed")

    def domain_decision(self):
        directory = self.root / "docs" / "decisions" / "research"
        directory.mkdir()
        original = self.root / "docs" / "decisions" / "0001-pick.md"
        target = directory / original.name
        original.rename(target)
        self.edit("docs/decisions/research/0001-pick.md", "../../openspec/", "../../../openspec/")
        self.edit("docs/decisions/research/0001-pick.md", "../research/", "../../research/")
        self.edit("docs/research/INDEX.md", "../decisions/0001-pick.md", "../decisions/research/0001-pick.md")
        design = self.root / "openspec" / "changes" / "archive" / "2026-01-01-add-x" / "design.md"
        design.write_text("Decisions: `docs/decisions/research/0001-pick.md`\n")
        return target

    def test_domain_qualified_record_resolves_links_and_backlink(self):
        self.domain_decision()
        self.assertEqual(self.problems(), [])

    def test_domain_qualified_record_missing_field_fails(self):
        self.domain_decision()
        self.edit("docs/decisions/research/0001-pick.md", "**Live-proven:** no\n", "")
        self.assert_problem("missing field 'Live-proven'")

    def test_domain_qualified_backlink_requires_full_identity(self):
        self.domain_decision()
        design = self.root / "openspec" / "changes" / "archive" / "2026-01-01-add-x" / "design.md"
        design.write_text("Decisions: `docs/decisions/0001-pick.md`\n")
        self.assert_problem("does not name docs/decisions/research/0001-pick.md")

    def test_domain_qualified_numbers_are_checked_per_domain(self):
        target = self.domain_decision()
        # A legacy record and a different domain both start at 0001.
        (self.root / "docs" / "decisions" / "0001-pick.md").write_text(
            DECISION.replace("[add-x](../../openspec/changes/archive/2026-01-01-add-x/)", "none - historical")
        )
        self.assertEqual(self.problems(), [])
        target.rename(target.with_name("0003-pick.md"))
        self.assert_problem("docs/decisions/research/: decision numbers must run 0001, 0002")

    def test_domain_source_files_still_require_hashes(self):
        self.domain_decision()
        (self.root / "docs" / "decisions" / "research" / "source.txt").write_text("source\n")
        self.assert_problem("research/source.txt: file is not listed")

    def test_cli_checks_domain_qualified_decision(self):
        self.domain_decision()
        command = [sys.executable, "-I", "-B", str(HERE / "check-research-records.py"), "--root", str(self.root)]
        result = subprocess.run(command, capture_output=True, text=True, check=False)
        self.assertEqual((result.returncode, result.stdout), (0, "0 problem(s)\n"))
        self.edit("docs/decisions/research/0001-pick.md", "../../research/2026-01-01-topic/README.md", "../../research/gone.md")
        result = subprocess.run(command, capture_output=True, text=True, check=False)
        self.assertEqual(result.returncode, 1)
        self.assertIn("broken link ../../research/gone.md", result.stdout)

    def test_repository_records_are_consistent(self):
        self.assertEqual(checker.check(HERE.parent), [])


if __name__ == "__main__":
    unittest.main()
