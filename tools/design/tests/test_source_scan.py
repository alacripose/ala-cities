import tempfile
import unittest
from pathlib import Path

from tools.design.source_scan import scan_source


class SourceScanTests(unittest.TestCase):
    def test_source_view_is_explicitly_noncanonical(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "src").mkdir()
            (root / "src" / "main.rs").write_text("fn main() {}\n", encoding="utf-8")
            view = scan_source(root)
        self.assertFalse(view["canonical"])
        self.assertTrue(view["source_digest"])
        self.assertEqual(view["source_counts"]["rust_files"], 1)

    def test_source_digest_changes_when_a_source_file_changes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "src").mkdir()
            source = root / "src" / "main.rs"
            source.write_text("fn main() {}\n", encoding="utf-8")
            before = scan_source(root)["source_digest"]
            source.write_text('fn main() { println!("x"); }\n', encoding="utf-8")
            after = scan_source(root)["source_digest"]
        self.assertNotEqual(before, after)

    def test_source_hints_separate_agents_from_world_substrate(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            sim = root / "src" / "sim"
            sim.mkdir(parents=True)
            (sim / "citizen.rs").write_text("pub struct Citizen;\n", encoding="utf-8")
            (sim / "task.rs").write_text("pub struct Task;\n", encoding="utf-8")
            (sim / "terrain.rs").write_text("pub fn terrain() {}\n", encoding="utf-8")
            gov = root / "src" / "gov"
            gov.mkdir()
            (gov / "mod.rs").write_text("pub struct Government;\n", encoding="utf-8")
            (gov / "season.rs").write_text("pub struct Season;\n", encoding="utf-8")
            view = scan_source(root)
        hints = {item["path"]: item["family_id"] for item in view["inferred_capabilities"]}
        self.assertEqual(hints["src/sim/citizen.rs"], "CAP-003")
        self.assertEqual(hints["src/sim/task.rs"], "CAP-003")
        self.assertEqual(hints["src/sim/terrain.rs"], "CAP-001")
        self.assertEqual(hints["src/gov/mod.rs"], "CAP-005")
        self.assertEqual(hints["src/gov/season.rs"], "CAP-006")

    def test_generated_design_output_is_not_part_of_the_source_digest(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "src").mkdir()
            (root / "src" / "main.rs").write_text("fn main() {}\n", encoding="utf-8")
            generated = root / "docs" / "design" / "generated"
            generated.mkdir(parents=True)
            view = generated / "source-capabilities.json"
            view.write_text("{}\n", encoding="utf-8")
            before = scan_source(root)["source_digest"]
            view.write_text('{"changed": true}\n', encoding="utf-8")
            after = scan_source(root)["source_digest"]
        self.assertEqual(before, after)

    def test_asset_summary_counts_files_without_opening_them(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            assets = root / "assets" / "generated"
            assets.mkdir(parents=True)
            (assets / "one.bin").write_bytes(b"12345")
            (assets / "two.bin").write_bytes(b"678")
            view = scan_source(root)
        self.assertEqual(view["asset_summary"]["file_count"], 2)
        self.assertEqual(view["asset_summary"]["byte_count"], 8)


if __name__ == "__main__":
    unittest.main()
