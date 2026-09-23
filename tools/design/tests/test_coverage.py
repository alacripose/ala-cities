import unittest

from tools.design.coverage import coverage_report


class CoverageTests(unittest.TestCase):
    def test_coverage_requires_one_ledger_row_per_extracted_record(self):
        report = coverage_report(
            [{"id": "C1-Q001", "aliases": ["Q1"]}],
            {"decisions": [{"id": "C1-Q001", "review_state": "reviewed"}]},
        )
        self.assertEqual(report["unrepresented_source_ids"], [])
        self.assertEqual(report["unknown_ledger_ids"], [])
        self.assertTrue(report["complete"])

    def test_coverage_does_not_call_a_partial_ledger_complete(self):
        report = coverage_report(
            [{"id": "C1-Q001"}, {"id": "C1-Q002"}],
            {"decisions": [{"id": "C1-Q001"}]},
        )
        self.assertEqual(report["unrepresented_source_ids"], ["C1-Q002"])
        self.assertFalse(report["complete"])

    def test_coverage_rejects_an_unreviewed_ledger_row(self):
        report = coverage_report(
            [{"id": "C1-Q001"}],
            {"decisions": [{"id": "C1-Q001", "review_state": "unreviewed"}]},
        )
        self.assertEqual(report["unreviewed_ids"], ["C1-Q001"])
        self.assertFalse(report["complete"])

    def test_coverage_rejects_a_missing_review_state(self):
        report = coverage_report(
            [{"id": "C1-Q001"}],
            {"decisions": [{"id": "C1-Q001"}]},
        )
        self.assertEqual(report["unreviewed_ids"], ["C1-Q001"])
        self.assertFalse(report["complete"])

    def test_coverage_reports_unknown_ledger_ids(self):
        report = coverage_report([], {"decisions": [{"id": "C1-Q999"}]})
        self.assertEqual(report["unknown_ledger_ids"], ["C1-Q999"])
        self.assertFalse(report["complete"])


if __name__ == "__main__":
    unittest.main()
