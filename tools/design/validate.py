"""Validate the typed design package without a third-party schema dependency."""

from __future__ import annotations

import hashlib
import json
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from .source_scan import scan_source


PACKET_FIELDS = (
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
)
CANONICAL_ID = re.compile(r"^C\d+-Q\d{3}$")
CAPABILITY_ID = re.compile(r"^CAP-\d{3}$")
CAPABILITY_FAMILY_IDS = tuple(f"CAP-{index:03d}" for index in range(1, 13))
ARTIFACT_ID = re.compile(r"^ART-\d{3}$")
ARTIFACT_DISPOSITIONS = {"unreviewed", "keep", "migrate", "retire"}


@dataclass
class ValidationReport:
    errors: list[str] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)

    @property
    def ok(self) -> bool:
        return not self.errors


def validate_decisions(decisions: list[dict]) -> list[str]:
    errors: list[str] = []
    seen: set[str] = set()
    for decision in decisions:
        identity = decision.get("id") if isinstance(decision, dict) else None
        if not isinstance(identity, str) or not CANONICAL_ID.fullmatch(identity):
            errors.append(f"invalid decision id {identity!r}")
            continue
        if identity in seen:
            errors.append(f"duplicate decision id {identity}")
        seen.add(identity)
    return errors


def validate_decision_sources(decisions: list[dict], root: Path) -> list[str]:
    errors: list[str] = []
    for decision in decisions:
        if not isinstance(decision, dict):
            continue
        identity = decision.get("id")
        source_file = decision.get("source_file")
        expected = decision.get("source_digest")
        if not isinstance(identity, str) or not isinstance(source_file, str) or not isinstance(expected, str):
            continue
        candidates = (root / "docs" / source_file, root / source_file)
        source = next((candidate for candidate in candidates if candidate.exists()), None)
        if source is None:
            errors.append(f"decision {identity} names missing source {source_file}")
            continue
        actual = hashlib.sha256(source.read_bytes()).hexdigest()
        if actual != expected:
            errors.append(f"decision {identity} has a stale source digest")
    return errors


def validate_artifacts(artifacts: list[dict]) -> list[str]:
    errors: list[str] = []
    seen: set[str] = set()
    for artifact in artifacts:
        if not isinstance(artifact, dict):
            errors.append("artifact entries must be objects")
            continue
        identity = artifact.get("id")
        if not isinstance(identity, str) or not ARTIFACT_ID.fullmatch(identity):
            errors.append(f"invalid artifact id {identity!r}")
            continue
        if identity in seen:
            errors.append(f"duplicate artifact id {identity}")
        seen.add(identity)
        disposition = artifact.get("disposition")
        if disposition not in ARTIFACT_DISPOSITIONS:
            errors.append(f"artifact {identity} has invalid disposition {disposition}")
    return errors


def validate_packet(packet: dict) -> list[str]:
    if not isinstance(packet, dict):
        return ["packet must be an object"]
    identity = packet.get("id")
    if not isinstance(identity, str) or not CAPABILITY_ID.fullmatch(identity):
        return [f"invalid packet id {identity!r}"]
    errors = [
        f"packet {identity} is missing {field}"
        for field in PACKET_FIELDS
        if field not in packet
    ]
    if packet.get("unresolved_questions"):
        errors.append(f"packet {identity} has unresolved questions")
    return errors


def validate_source_view(root: Path) -> list[str]:
    path = root / "docs" / "design" / "generated" / "source-capabilities.json"
    try:
        existing = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        return ["missing generated/source-capabilities.json"]
    except (OSError, json.JSONDecodeError) as error:
        return [f"cannot read generated/source-capabilities.json: {error}"]
    if existing.get("canonical") is not False:
        return ["source-capabilities.json must be explicitly noncanonical"]
    if existing.get("source_digest") != scan_source(root)["source_digest"]:
        return ["source-capabilities.json is stale"]
    return []


def _read_json(path: Path, errors: list[str]) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        errors.append(f"missing {path.as_posix()}")
    except (OSError, json.JSONDecodeError) as error:
        errors.append(f"cannot read {path.as_posix()}: {error}")
    return None


def validate_package(root: Path) -> ValidationReport:
    report = ValidationReport()
    package = root / "docs" / "design"
    documents: dict[str, Any] = {}
    for name in ("decisions.json", "capabilities.json", "artifacts.json"):
        value = _read_json(package / name, report.errors)
        if value is not None:
            documents[name] = value
        if not isinstance(value, dict):
            report.errors.append(f"{name} must contain an object")
            continue
        if not isinstance(value.get("schema_version"), int) or value["schema_version"] < 1:
            report.errors.append(f"{name} has no positive schema_version")
        if not isinstance(value.get("design_revision"), int) or value["design_revision"] < 1:
            report.errors.append(f"{name} has no positive design_revision")

    decisions = documents.get("decisions.json", {}).get("decisions", [])
    if not isinstance(decisions, list):
        report.errors.append("decisions.json decisions must be an array")
    else:
        report.errors.extend(validate_decisions(decisions))
        report.errors.extend(validate_decision_sources(decisions, root))

    artifacts = documents.get("artifacts.json", {}).get("artifacts", [])
    if not isinstance(artifacts, list):
        report.errors.append("artifacts.json artifacts must be an array")
    else:
        report.errors.extend(validate_artifacts(artifacts))

    capabilities = documents.get("capabilities.json", {}).get("capabilities", [])
    if not isinstance(capabilities, list):
        report.errors.append("capabilities.json capabilities must be an array")
        capabilities = []
    capability_ids = {
        item.get("id") for item in capabilities if isinstance(item, dict)
    }
    for identity in CAPABILITY_FAMILY_IDS:
        if identity not in capability_ids:
            report.errors.append(f"missing capability {identity}")
    for capability in capabilities:
        if not isinstance(capability, dict):
            report.errors.append("capability entries must be objects")
            continue
        identity = capability.get("id")
        packet_path = capability.get("packet")
        if not isinstance(identity, str) or not CAPABILITY_ID.fullmatch(identity):
            report.errors.append(f"invalid capability id {identity!r}")
            continue
        if not isinstance(packet_path, str):
            report.errors.append(f"capability {identity} has no packet path")
            continue
        packet = _read_json(package / packet_path, report.errors)
        if packet is not None:
            report.errors.extend(validate_packet(packet))

    report.errors.extend(validate_source_view(root))
    return report
