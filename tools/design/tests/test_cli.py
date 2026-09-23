import json
import tempfile
import unittest
from pathlib import Path

from tools.design.cli import main


class CliTests(unittest.TestCase):
    def test_extract_command_writes_deterministic_json(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "docs").mkdir()
            (root / "docs" / "GRILLING-C1.md").write_text(
                "| Q1 | question | recommendation | answer |\n", encoding="utf-8"
            )
            output = root / "draft.json"
            result = main(
                [
                    "extract",
                    "--root",
                    str(root),
                    "--campaign",
                    "C1",
                    "--output",
                    str(output),
                ]
            )
            payload = json.loads(output.read_text(encoding="utf-8"))
        self.assertEqual(result, 0)
        self.assertEqual(payload["records"][0]["id"], "C1-Q001")

    def test_source_scan_command_writes_a_noncanonical_view(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "src").mkdir()
            (root / "src" / "main.rs").write_text("fn main() {}\n", encoding="utf-8")
            output = root / "source.json"
            result = main(
                [
                    "source-scan",
                    "--root",
                    str(root),
                    "--output",
                    str(output),
                ]
            )
            payload = json.loads(output.read_text(encoding="utf-8"))
        self.assertEqual(result, 0)
        self.assertFalse(payload["canonical"])

    def test_source_scan_check_rejects_a_stale_view(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "src").mkdir()
            source = root / "src" / "main.rs"
            source.write_text("fn main() {}\n", encoding="utf-8")
            output = root / "source.json"
            self.assertEqual(
                main(["source-scan", "--root", str(root), "--output", str(output)]),
                0,
            )
            source.write_text("fn main() { println!(\"changed\"); }\n", encoding="utf-8")
            result = main(
                ["source-scan", "--root", str(root), "--output", str(output), "--check"]
            )
        self.assertEqual(result, 1)

    def test_project_command_writes_coverage_and_markdown(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            docs = root / "docs"
            design = docs / "design"
            design.mkdir(parents=True)
            (design / "decisions.json").write_text(
                json.dumps({"schema_version": 1, "design_revision": 1, "decisions": []}),
                encoding="utf-8",
            )
            (docs / "GRILLING-C11.md").write_text(
                "### Q211 — scope\n\nAnswer.\n", encoding="utf-8"
            )
            for index in range(1, 11):
                (docs / f"GRILLING-C{index}.md").write_text(
                    "No declarations in this fixture.\n", encoding="utf-8"
                )
            result = main(["project", "--root", str(root)])
            self.assertTrue((design / "generated" / "coverage.draft.json").exists())
            self.assertTrue((design / "generated" / "decisions.md").exists())
        self.assertEqual(result, 0)

    def test_verify_command_returns_nonzero_for_missing_package(self):
        with tempfile.TemporaryDirectory() as directory:
            result = main(["verify", "--root", directory])
        self.assertEqual(result, 1)


if __name__ == "__main__":
    unittest.main()
