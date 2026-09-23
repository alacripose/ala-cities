import json
import tempfile
import unittest
from pathlib import Path

from tools.design.project import extract_current_decisions, project_package


class ProjectTests(unittest.TestCase):
    def test_current_c11_headings_become_reviewed_rows(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            record = root / "docs" / "GRILLING-C11.md"
            record.parent.mkdir(parents=True)
            record.write_text(
                "# C11\n\n### Q211 — scope\n\nAnswer text.\n\n"
                "### Q212 — claim\n\nConfirmed.\n",
                encoding="utf-8",
            )
            rows = extract_current_decisions(root)
        self.assertEqual([row["id"] for row in rows], ["C11-Q211", "C11-Q212"])
        self.assertTrue(all(row["review_state"] == "reviewed" for row in rows))
        self.assertEqual(rows[0]["source_file"], "GRILLING-C11.md")

    def test_project_writes_coverage_and_decision_projection(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "docs" / "design" / "generated").mkdir(parents=True)
            (root / "docs" / "design" / "decisions.json").write_text(
                json.dumps({"decisions": []}), encoding="utf-8"
            )
            (root / "docs" / "GRILLING-C11.md").write_text(
                "### Q211 — scope\n\nAnswer.\n", encoding="utf-8"
            )
            for index in range(1, 11):
                (root / "docs" / f"GRILLING-C{index}.md").write_text(
                    "No declarations in this fixture.\n", encoding="utf-8"
                )
            result = project_package(root)
            coverage = json.loads(
                (root / "docs" / "design" / "generated" / "coverage.draft.json").read_text(
                    encoding="utf-8"
                )
            )
            projection = (
                root / "docs" / "design" / "generated" / "decisions.md"
            ).read_text(encoding="utf-8")
        self.assertEqual(result, 0)
        self.assertIn("unrepresented_source_ids", coverage)
        self.assertIn("C11-Q211", projection)


if __name__ == "__main__":
    unittest.main()
