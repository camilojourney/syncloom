"""Consumer acceptance checks through the released checker's real CLI."""
import pathlib
import subprocess
import sys
import tempfile
import unittest

CHECKER = pathlib.Path(__file__).with_name("check-change-shape.py")
MULTILINE = "Keep the exact words.\nAnd the exact line break.\n\nCount this as the second paragraph.\n"


class ChangeShapeCliTests(unittest.TestCase):
    def run_checker(self, source, quoted=None, approval="pending", capability="requested"):
        with tempfile.TemporaryDirectory() as directory:
            change = pathlib.Path(directory)
            (change / "request.txt").write_text(source)
            text = source if quoted is None else quoted
            quote = "\n".join("> " + line for line in text.splitlines())
            (change / "proposal.md").write_text(
                "## Request\n" + quote + "\n\n## Approval\n" + approval
                + "\nNew capability " + capability + ": None.\nDuplicate kept: None.\n"
                + "\n## Open questions\nNone.\n"
            )
            (change / "design.md").write_text("References: none\n")
            (change / "tasks.md").write_text("- [ ] 1.1 Load the code skill\n")
            return subprocess.run(
                [sys.executable, "-I", "-B", str(CHECKER), str(change),
                 "--request-source", str(change / "request.txt")],
                capture_output=True, text=True, check=False,
            )

    def test_faithful_multiline_request_preserves_two_paragraphs(self):
        result = self.run_checker(MULTILINE)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("Request quotes 2 paragraphs\n", result.stdout)
        self.assertTrue(result.stdout.endswith("problems: 0\n"), result.stdout)

    def test_single_line_and_internal_whitespace_remain_exact(self):
        for source in ("Single line.\n", "Keep  two spaces.\n Next line.\n"):
            with self.subTest(source=source):
                result = self.run_checker(source)
                self.assertEqual(result.returncode, 0, result.stdout)
                self.assertIn("Request quotes 1 paragraphs\n", result.stdout)

    def test_changed_missing_joined_split_and_empty_quotes_are_rejected(self):
        source = "Keep  two spaces.\nNext line.\n"
        for quoted in (
            "Changed words.\nNext line.\n", "Keep  two spaces.\n",
            "Keep  two spaces. Next line.\n", "Keep  two spaces.\n\nNext line.\n",
            "Keep two spaces.\nNext line.\n", "",
        ):
            with self.subTest(quoted=quoted):
                result = self.run_checker(source, quoted)
                self.assertEqual(result.returncode, 1, result.stdout)
                self.assertIn("Request paragraph not verbatim:", result.stdout)

    def test_empty_request_is_rejected(self):
        result = self.run_checker("")
        self.assertEqual(result.returncode, 1, result.stdout)
        self.assertIn("Request quotes nothing", result.stdout)

    def test_attributed_reviewer_tokens_do_not_require_initial_tokens(self):
        for approval, capability in (
            ("approved: synthetic-reviewer", "requested"),
            ("pending", "approved"), ("approved: synthetic-reviewer", "approved"),
        ):
            with self.subTest(approval=approval, capability=capability):
                result = self.run_checker("Single line.\n", approval=approval, capability=capability)
                self.assertEqual(result.returncode, 0, result.stdout)

    def test_unknown_or_unattributed_approval_is_rejected(self):
        for approval, capability in (("approved:", "requested"), ("ok", "requested"), ("pending", "ok")):
            with self.subTest(approval=approval, capability=capability):
                result = self.run_checker("Single line.\n", approval=approval, capability=capability)
                self.assertEqual(result.returncode, 1, result.stdout)


if __name__ == "__main__":
    unittest.main()
