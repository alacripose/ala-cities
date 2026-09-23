"""Produce a digest-bound, noncanonical view of the current source tree."""

from __future__ import annotations

import hashlib
import os
from pathlib import Path


_SOURCE_EXTENSIONS = {
    ".rs": "rust_files",
    ".py": "python_files",
    ".md": "markdown_files",
    ".json": "json_files",
    ".toml": "toml_files",
    ".ron": "ron_files",
    ".wgsl": "wgsl_files",
}
_SKIP_DIRECTORIES = {".git", "target", "__pycache__", ".venv", ".superpowers"}
_ASSET_ROOT = "assets"
_PLAYTEST_ROOT = "playtest"


def _walk_files(root: Path):
    for directory, names, files in os.walk(root):
        names[:] = sorted(name for name in names if name not in _SKIP_DIRECTORIES)
        for name in sorted(files):
            yield Path(directory) / name


def _relative(root: Path, path: Path) -> str:
    return path.relative_to(root).as_posix()


def _is_source_file(relative: str) -> bool:
    path = Path(relative)
    if path.parts[:3] == ("docs", "design", "generated"):
        return False
    if path.name in {"Cargo.toml", "rust-toolchain.toml", "README.md", "DESIGN.md"}:
        return True
    if path.parts and path.parts[0] in {"src", "tools", "docs", "config", "playtest"}:
        return path.suffix.lower() in _SOURCE_EXTENSIONS
    if path.parts and path.parts[:2] == ("assets", "icons"):
        return path.suffix.lower() in {".json", ".md"}
    return False


def _source_digest(root: Path, source_files: list[Path]) -> str:
    digest = hashlib.sha256()
    for path in source_files:
        relative = _relative(root, path).encode("utf-8")
        data = path.read_bytes()
        digest.update(len(relative).to_bytes(4, "big"))
        digest.update(relative)
        digest.update(len(data).to_bytes(8, "big"))
        digest.update(data)
    return digest.hexdigest()


def _asset_summary(root: Path) -> dict:
    file_count = 0
    byte_count = 0
    asset_root = root / _ASSET_ROOT
    if not asset_root.exists():
        return {"file_count": 0, "byte_count": 0}
    for path in _walk_files(asset_root):
        file_count += 1
        try:
            byte_count += path.stat().st_size
        except OSError:
            continue
    return {"file_count": file_count, "byte_count": byte_count}


def _family_for(relative: str) -> str | None:
    if relative in {"src/sim/citizen.rs", "src/sim/task.rs"}:
        return "CAP-003"
    if relative.startswith("src/sim/") or relative == "src/sim/mod.rs":
        return "CAP-001"
    if relative.startswith("src/materials/"):
        return "CAP-002"
    if relative == "src/gov/season.rs":
        return "CAP-006"
    if relative.startswith("src/gov/"):
        return "CAP-005"
    if relative in {"src/session.rs", "src/agentledger.rs"} or relative.startswith("playtest/"):
        return "CAP-011"
    if relative in {"src/render.rs", "src/hud.rs", "src/design.rs", "src/ui.rs", "src/text.rs", "src/audio.rs"}:
        return "CAP-009"
    if relative == "src/main.rs":
        return "CAP-008"
    if relative.startswith("src/icons") or relative.startswith("src/iconreview") or relative.startswith("tools/icons/") or relative.startswith("assets/icons/"):
        return "CAP-010"
    if relative.startswith("src/bin/") or relative in {"src/buildinfo.rs", "tools/materials/"}:
        return "CAP-012"
    return None


def scan_source(root: Path) -> dict:
    """Scan source metadata without treating the result as target architecture."""
    root = root.resolve()
    all_files = list(_walk_files(root))
    source_files = [
        path for path in all_files if _is_source_file(_relative(root, path))
    ]
    source_files.sort(key=lambda path: _relative(root, path))
    counts = {name: 0 for name in _SOURCE_EXTENSIONS.values()}
    counts["total_source_files"] = len(source_files)
    for path in source_files:
        key = _SOURCE_EXTENSIONS.get(path.suffix.lower())
        if key:
            counts[key] += 1

    modules = [
        _relative(root, path)
        for path in source_files
        if path.suffix.lower() == ".rs"
    ]
    binaries = [
        _relative(root, path)
        for path in source_files
        if path.parts[:2] == ("src", "bin") or _relative(root, path) == "src/main.rs"
    ]
    tools = [
        _relative(root, path)
        for path in source_files
        if path.suffix.lower() == ".py"
    ]
    inferred = []
    for path in source_files:
        relative = _relative(root, path)
        family = _family_for(relative)
        if family:
            inferred.append({"family_id": family, "path": relative, "confidence": "source-derived"})

    return {
        "schema_version": 1,
        "kind": "source-capability-reference",
        "canonical": False,
        "generator": "tools/design/source_scan.py",
        "source_digest": _source_digest(root, source_files),
        "source_counts": counts,
        "modules": modules,
        "binaries": binaries,
        "tools": tools,
        "asset_summary": _asset_summary(root),
        "playtest_file_count": sum(
            1 for path in all_files if _relative(root, path).startswith(_PLAYTEST_ROOT + "/")
        ),
        "inferred_capabilities": inferred,
    }
