"""Two-way coverage between extracted decisions and the canonical ledger."""

from __future__ import annotations


def coverage_report(records: list[dict], ledger: dict) -> dict:
    source_ids = {record["id"] for record in records}
    ledger_rows = ledger.get("decisions", []) if isinstance(ledger, dict) else []
    ledger_ids = {
        row["id"] for row in ledger_rows if isinstance(row, dict) and isinstance(row.get("id"), str)
    }
    unrepresented = sorted(source_ids - ledger_ids)
    unknown = sorted(ledger_ids - source_ids)
    unreviewed = sorted(
        row["id"]
        for row in ledger_rows
        if isinstance(row, dict)
        and row.get("id") in source_ids
        and row.get("review_state") not in {"reviewed", "approved"}
    )
    return {
        "source_count": len(source_ids),
        "ledger_count": len(ledger_ids),
        "unrepresented_source_ids": unrepresented,
        "unknown_ledger_ids": unknown,
        "unreviewed_ids": unreviewed,
        "complete": not unrepresented and not unknown and not unreviewed and bool(source_ids),
    }
