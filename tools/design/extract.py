"""Extract inherited question declarations without repairing their identities."""

from __future__ import annotations

import hashlib
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable


_TABLE_DECLARATION = re.compile(r"^\|\s*Q(\d+[a-z]?)\s*\|")
_HEADING_DECLARATION = re.compile(r"^#{2,3}\s+(?:❓\s*)?Q\d+[a-z]?")
_ALIAS = re.compile(r"\bQ\d+[a-z]?\b")
_CAMPAIGN_NUMBER = re.compile(r"^C(\d+)$")


@dataclass
class ExtractionResult:
    records: list[dict] = field(default_factory=list)
    unparsed: list[str] = field(default_factory=list)
    reused_aliases: list[dict] = field(default_factory=list)
    missing_ranges: list[dict] = field(default_factory=list)


def canonical_id(campaign: str, ordinal: int) -> str:
    """Return the dense campaign-scoped identity for a declaration."""
    if ordinal < 1:
        raise ValueError(f"question ordinal must be positive: {ordinal}")
    return f"{campaign}-Q{ordinal:03d}"


def _source_path(root: Path, campaign: str) -> Path:
    for candidate in (root / "docs" / f"GRILLING-{campaign}.md", root / f"GRILLING-{campaign}.md"):
        if candidate.exists():
            return candidate
    raise FileNotFoundError(f"no record found for {campaign}")


def _digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _is_heading(line: str) -> bool:
    return line.startswith("## ") or line.startswith("### ")


def _declaration(line: str) -> tuple[str, list[str]] | None:
    table = _TABLE_DECLARATION.match(line)
    if table:
        alias = f"Q{table.group(1)}"
        return "table", [alias]
    if not _HEADING_DECLARATION.match(line):
        return None
    aliases = _ALIAS.findall(line)
    return ("heading", aliases) if aliases else None


def _question_text(lines: list[str], start: int) -> str:
    text: list[str] = []
    for line in lines[start + 1 :]:
        if _declaration(line) is not None or line.strip() == "---":
            break
        if line.strip():
            text.append(line.strip())
    return "\n".join(text)


def _numeric_gaps(records: list[dict], campaign: str) -> list[dict]:
    numbers = sorted(
        {
            int(alias[1:])
            for record in records
            for alias in record["aliases"]
            if alias[1:].isdigit()
        }
    )
    if len(numbers) < 2:
        return []
    gaps = []
    for start, end in zip(numbers, numbers[1:]):
        if end > start + 1:
            gaps.append({"campaign": campaign, "start": start + 1, "end": end - 1})
    return gaps


def extract_questions(
    root: Path,
    campaigns: Iterable[str],
    report_unparsed: bool = False,
) -> ExtractionResult:
    """Extract declarations in stable campaign/file/line order.

    A local question number is an alias, never an identity. Reused aliases are
    retained as separate records and reported; this function never guesses how to
    merge them.
    """
    result = ExtractionResult()
    ordered_campaigns = sorted(
        campaigns,
        key=lambda campaign: int(_CAMPAIGN_NUMBER.fullmatch(campaign).group(1)),
    )
    for campaign in ordered_campaigns:
        path = _source_path(root, campaign)
        raw = path.read_bytes()
        lines = raw.decode("utf-8").splitlines()
        source_digest = _digest(raw)
        campaign_records: list[dict] = []
        seen_aliases: dict[str, str] = {}
        for index, line in enumerate(lines):
            declaration = _declaration(line)
            if declaration is None:
                if report_unparsed and _is_heading(line) and "Q" in line:
                    result.unparsed.append(f"{path.name}:{index + 1}: {line.strip()}")
                continue
            kind, aliases = declaration
            canonical = canonical_id(campaign, len(campaign_records) + 1)
            record = {
                "id": canonical,
                "aliases": aliases,
                "source_file": path.name,
                "source_line": index + 1,
                "question_text": _question_text(lines, index),
                "kind": kind,
                "source_digest": source_digest,
            }
            campaign_records.append(record)
            for alias in aliases:
                alias_key = f"{campaign}:{alias}"
                if alias_key in seen_aliases:
                    result.reused_aliases.append(
                        {
                            "campaign": campaign,
                            "alias": alias,
                            "first_id": seen_aliases[alias_key],
                            "second_id": canonical,
                        }
                    )
                else:
                    seen_aliases[alias_key] = canonical
        result.records.extend(campaign_records)
        result.missing_ranges.extend(_numeric_gaps(campaign_records, campaign))
    return result
