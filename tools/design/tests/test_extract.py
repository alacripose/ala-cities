import tempfile
import unittest
from pathlib import Path

from tools.design.extract import canonical_id, extract_questions


class ExtractTests(unittest.TestCase):
    def test_campaign_scoped_ids_are_dense_and_stable(self):
        self.assertEqual(canonical_id("C1", 1), "C1-Q001")
        self.assertEqual(canonical_id("C9", 42), "C9-Q042")

    def test_joint_and_satellite_declarations_keep_their_original_aliases(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "GRILLING-C10.md").write_text(
                "### Q189 / Q190 — setup\n\n## ❓ Q96b — restated\n",
                encoding="utf-8",
            )
            result = extract_questions(root, ["C10"])
        self.assertEqual(
            [row["aliases"] for row in result.records],
            [["Q189", "Q190"], ["Q96b"]],
        )
        self.assertEqual(
            [row["id"] for row in result.records],
            ["C10-Q001", "C10-Q002"],
        )

    def test_c1_table_rows_and_c2_emoji_headings_are_declarations(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "GRILLING-C1.md").write_text(
                "| Q1 | deliverable | old | answer |\n", encoding="utf-8"
            )
            (root / "GRILLING-C2.md").write_text(
                "## ❓ Q29 — camera\n", encoding="utf-8"
            )
            result = extract_questions(root, ["C1", "C2"])
        self.assertEqual(
            [(row["source_file"], row["aliases"][0]) for row in result.records],
            [
                ("GRILLING-C1.md", "Q1"),
                ("GRILLING-C2.md", "Q29"),
            ],
        )

    def test_unclassified_question_like_heading_is_reported(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "GRILLING-C3.md").write_text(
                "## Q-like prose that is not a declaration\n", encoding="utf-8"
            )
            result = extract_questions(root, ["C3"], report_unparsed=True)
        self.assertTrue(result.unparsed)
        self.assertIn("Q-like prose", result.unparsed[0])


if __name__ == "__main__":
    unittest.main()
