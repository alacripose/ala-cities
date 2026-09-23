import json
import tempfile
import unittest
from pathlib import Path

from tools.design.validate import (
    validate_artifacts,
    validate_decision_sources,
    validate_decisions,
    validate_packet,
    validate_source_view,
)


REQUIRED_PACKET_FIELDS = {
    "purpose",
    "actors_consumers",
    "vocabulary",
    "invariants",
    "interface",
    "authoritative_model",
    "flows",
    "failure_refusal",
    "verification",
    "artifact_dispositions",
    "inherited_decision_coverage",
    "unresolved_questions",
}


def packet(**overrides):
    value = {field: [] for field in REQUIRED_PACKET_FIELDS}
    value.update(
        {
            "id": "CAP-001",
            "review_state": "draft",
            "unresolved_questions": [],
        }
    )
    value.update(overrides)
    return value


class ValidateTests(unittest.TestCase):
    def test_verify_rejects_duplicate_decision_ids(self):
        errors = validate_decisions([{"id": "C1-Q001"}, {"id": "C1-Q001"}])
        self.assertEqual(errors, ["duplicate decision id C1-Q001"])

    def test_verify_rejects_a_packet_with_unresolved_questions(self):
        errors = validate_packet(packet(unresolved_questions=["Q-001"]))
        self.assertEqual(errors, ["packet CAP-001 has unresolved questions"])

    def test_verify_accepts_a_complete_packet_shape(self):
        errors = validate_packet(packet(review_state="reviewed"))
        self.assertEqual(errors, [])

    def test_verify_rejects_an_unknown_artifact_disposition(self):
        errors = validate_artifacts(
            [{"id": "ART-001", "name": "record", "producer": "test", "disposition": "guess"}]
        )
        self.assertEqual(errors, ["artifact ART-001 has invalid disposition guess"])

    def test_verify_rejects_a_stale_decision_source_digest(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            record = root / "docs" / "GRILLING-C1.md"
            record.parent.mkdir(parents=True)
            record.write_text("record\n", encoding="utf-8")
            errors = validate_decision_sources(
                [{"id": "C1-Q001", "source_file": "GRILLING-C1.md", "source_digest": "0" * 64}],
                root,
            )
        self.assertEqual(errors, ["decision C1-Q001 has a stale source digest"])

    def test_verify_rejects_a_stale_source_view(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "src").mkdir()
            (root / "src" / "main.rs").write_text("fn main() {}\n", encoding="utf-8")
            generated = root / "docs" / "design" / "generated"
            generated.mkdir(parents=True)
            (generated / "source-capabilities.json").write_text(
                json.dumps({"canonical": False, "source_digest": "stale"}),
                encoding="utf-8",
            )
            errors = validate_source_view(root)
        self.assertEqual(errors, ["source-capabilities.json is stale"])

    def test_verify_rejects_a_missing_required_packet_field(self):
        value = packet()
        del value["verification"]
        errors = validate_packet(value)
        self.assertEqual(errors, ["packet CAP-001 is missing verification"])


if __name__ == "__main__":
    unittest.main()
