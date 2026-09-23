"""Command-line adapter for the typed design package."""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict
from pathlib import Path

from .extract import extract_questions
from .project import project_package
from .source_scan import scan_source
from .validate import validate_package


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="python -m tools.design")
    commands = parser.add_subparsers(dest="command", required=True)

    extract = commands.add_parser("extract", help="extract inherited question declarations")
    extract.add_argument("--root", type=Path, default=Path("."))
    extract.add_argument("--campaign", action="append", required=True)
    extract.add_argument("--output", type=Path, required=True)
    extract.add_argument("--report-unparsed", action="store_true")

    verify = commands.add_parser("verify", help="validate the typed design package")
    verify.add_argument("--root", type=Path, default=Path("."))

    source_scan = commands.add_parser("source-scan", help="write a noncanonical source reference")
    source_scan.add_argument("--root", type=Path, default=Path("."))
    source_scan.add_argument("--output", type=Path, required=True)
    source_scan.add_argument("--check", action="store_true")

    project = commands.add_parser("project", help="write decision and coverage projections")
    project.add_argument("--root", type=Path, default=Path("."))
    return parser


def _write_extract(args: argparse.Namespace) -> int:
    result = extract_questions(args.root, args.campaign, args.report_unparsed)
    payload = {
        "schema_version": 1,
        "kind": "inherited-decision-draft",
        "canonical": False,
        "generator": "tools/design/cli.py extract",
        **asdict(result),
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(payload, indent=2, sort_keys=True, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    return 0


def _run_source_scan(args: argparse.Namespace) -> int:
    view = scan_source(args.root)
    if args.check:
        try:
            existing = json.loads(args.output.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as error:
            print(f"design: cannot read source view {args.output}: {error}", file=sys.stderr)
            return 1
        if existing.get("source_digest") != view["source_digest"]:
            print("design: source view is stale", file=sys.stderr)
            return 1
        print("design: source view is fresh")
        return 0
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(view, indent=2, sort_keys=True, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    return 0


def _run_verify(args: argparse.Namespace) -> int:
    report = validate_package(args.root)
    for error in report.errors:
        print(f"design: DEFECT: {error}", file=sys.stderr)
    for warning in report.warnings:
        print(f"design: warning: {warning}")
    if report.errors:
        print(f"design: {len(report.errors)} defect(s)", file=sys.stderr)
        return 1
    print("design: typed package is valid")
    return 0


def main(argv: list[str] | None = None) -> int:
    args = _parser().parse_args(argv)
    if args.command == "extract":
        return _write_extract(args)
    if args.command == "verify":
        return _run_verify(args)
    if args.command == "source-scan":
        return _run_source_scan(args)
    if args.command == "project":
        return project_package(args.root)
    raise AssertionError(f"unhandled command {args.command}")
