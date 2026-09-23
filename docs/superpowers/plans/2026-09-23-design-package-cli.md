# Design Package CLI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the smallest stdlib-only Python design tool that extracts the inherited decision corpus, validates the typed `docs/design/` package, and generates a noncanonical source-capability reference for issues #17 and #18.

**Architecture:** `tools/design/` owns pure parsing, validation, and projection functions. The CLI is a thin `argparse` adapter. Canonical JSON lives under `docs/design/`; generated source views live under `docs/design/generated/` and carry source digests. The tool never edits C1–C10 and never promotes a generated capability into the human-approved graph.

**Tech Stack:** Python 3.11+ standard library (`argparse`, `hashlib`, `json`, `pathlib`, `re`, `subprocess`, `unittest`), existing repository layout, no new dependency.

**Spec:** `DESIGN.md` Q229–Q249; `docs/GRILLING-C11.md` Round 4–6; Wayfinder map #1 and blockers #17/#18.

## Global Constraints

- Use dense campaign-scoped IDs: `C1-Q001`, `C9-Q001`, and `C11-Q229`.
- Preserve historical local numbers and satellite spellings as immutable aliases.
- Treat C1–C10 as immutable evidence; never rewrite them.
- Remove the phantom Q197–Q210 range; do not infer missing questions.
- Keep generated source capability output visibly noncanonical and digest-bound.
- Use no new runtime or Python dependency.
- Every nontrivial parser/validator branch has a unittest before implementation.
- `design-verify` exits non-zero on invalid data or stale generated output.
- Do not touch game source, saves, assets, or the user's uncommitted `AGENTS.md` edit.

## Review Focus

- A reused or malformed question declaration must never silently overwrite another record.
- A missing or phantom question number must be reported, not fabricated.
- A source-derived capability must never satisfy the approved capability graph by itself.
- A changed source file must make the generated source view stale.
- A packet with unresolved questions must not pass the completeness gate.

---

### Task 1: Define the parser contract with failing tests

**Files:**
- Create: `tools/design/__init__.py`
- Create: `tools/design/tests/__init__.py`
- Create: `tools/design/tests/test_extract.py`

**Interfaces:**
- Produces `canonical_id(campaign: str, ordinal: int) -> str`.
- Produces `extract_questions(root: Path, campaigns: Iterable[str]) -> ExtractionResult`.
- Each record contains `id`, `aliases`, `source_file`, `source_line`, `question_text`, `kind`, and `source_digest`.

- [ ] **Step 1: Write failing tests for canonical IDs and aliases**

```python
from pathlib import Path
from tools.design.extract import canonical_id, extract_questions


def test_campaign_scoped_ids_are_dense_and_stable():
    assert canonical_id("C1", 1) == "C1-Q001"
    assert canonical_id("C9", 42) == "C9-Q042"


def test_joint_and_satellite_declarations_keep_their_original_aliases(tmp_path: Path):
    record = tmp_path / "GRILLING-C10.md"
    record.write_text(
        "### Q189 / Q190 — setup\n\n## ❓ Q96b — restated\n",
        encoding="utf-8",
    )
    result = extract_questions(tmp_path, ["C10"])
    assert [row["aliases"] for row in result.records] == [["Q189", "Q190"], ["Q96b"]]
    assert [row["id"] for row in result.records] == ["C10-Q001", "C10-Q002"]
```

- [ ] **Step 2: Run the focused tests and verify the expected failure**

Run: `python -m unittest tools.design.tests.test_extract -v`

Expected: import or attribute failure because `tools.design.extract` does not exist yet.

- [ ] **Step 3: Add failing tests for the source formats**

```python
def test_c1_table_rows_and_c2_emoji_headings_are_declarations(tmp_path: Path):
    (tmp_path / "GRILLING-C1.md").write_text(
        "| Q1 | deliverable | old | answer |\n", encoding="utf-8"
    )
    (tmp_path / "GRILLING-C2.md").write_text(
        "## ❓ Q29 — camera\n", encoding="utf-8"
    )
    result = extract_questions(tmp_path, ["C1", "C2"])
    assert [(row["source_file"], row["aliases"][0]) for row in result.records] == [
        ("GRILLING-C1.md", "Q1"),
        ("GRILLING-C2.md", "Q29"),
    ]


def test_unclassified_question_like_heading_is_reported(tmp_path: Path):
    (tmp_path / "GRILLING-C3.md").write_text(
        "## Q-like prose that is not a declaration\n", encoding="utf-8"
    )
    result = extract_questions(tmp_path, ["C3"], report_unparsed=True)
    assert result.unparsed
    assert "Q-like prose" in result.unparsed[0]
```

- [ ] **Step 4: Keep the tests red until the parser is implemented**

Do not create a permissive fallback that turns every `Q` mention into a declaration.

---

### Task 2: Implement the pure inherited-decision extractor

**Files:**
- Create: `tools/design/extract.py`
- Test: `tools/design/tests/test_extract.py`

**Interfaces:**
- `canonical_id(campaign: str, ordinal: int) -> str`
- `extract_questions(root: Path, campaigns: Iterable[str], report_unparsed: bool = False) -> ExtractionResult`
- `ExtractionResult.records`, `.unparsed`, `.reused_aliases`, and `.missing_ranges` are explicit fields.

- [ ] **Step 1: Implement campaign/file discovery with stable ordering**

Use `Path.glob("GRILLING-C*.md")`, filter requested campaign names, sort by numeric campaign suffix, and read UTF-8. Do not follow symlinks or inspect files outside `docs/`.

- [ ] **Step 2: Implement declaration recognition**

Recognize only:

```text
| Q1 | ...                         # C1 table rows
## ❓ Q29 — ...                    # C2–C8 headings
### Q177 — ...                     # C9–C10 headings
Q41b, Q96b                          # satellite suffixes
Q189 / Q190                         # joint declarations become aliases on one row
```

Use the first following non-empty prose block as `question_text`, bounded by the next declaration heading or `---` separator. Keep the source line and SHA-256 digest of the source bytes.

- [ ] **Step 3: Normalize IDs and aliases**

Assign ordinals in source order per campaign. The canonical ID is `C<number>-Q<ordinal:03d>`. Keep every original token (`Q1`, `Q41b`, `Q189`, `Q190`) in `aliases`; never use a bare alias as the canonical key.

- [ ] **Step 4: Report ambiguity without repairing it**

Put reused aliases, malformed joint headings, and unclassified question-like declarations into result fields. Do not invent records for absent numbers and do not make the extractor fail closed yet; `design-verify` owns the policy gate.

- [ ] **Step 5: Run the focused tests and verify green**

Run: `python -m unittest tools.design.tests.test_extract -v`

Expected: all extractor tests pass, including the malformed/reused cases.

- [ ] **Step 6: Run the extractor against the real C1–C10 records**

Run: `python -m tools.design extract --root . --campaign C1 --campaign C2 --campaign C3 --campaign C4 --campaign C5 --campaign C6 --campaign C7 --campaign C8 --campaign C9 --campaign C10 --output docs/design/generated/inherited-decisions.draft.json`

Expected: the command writes a draft with source digests, normalized IDs, aliases, and explicit ambiguity fields. It must not modify C1–C10.

---

### Task 3: Add typed package validation and the named gate

**Files:**
- Create: `tools/design/validate.py`
- Create: `tools/design/cli.py`
- Create: `tools/design/__main__.py`
- Create: `tools/design/tests/test_validate.py`
- Create: `docs/design/decisions.json`
- Create: `docs/design/capabilities.json`
- Create: `docs/design/artifacts.json`

**Interfaces:**
- `validate_package(root: Path) -> ValidationReport`
- `validate_decisions(decisions: list[dict]) -> list[str]`
- `validate_packet(packet: dict) -> list[str]`
- `main(argv: list[str] | None = None) -> int`
- CLI commands: `extract`, `verify`, `source-scan`, and `project`.

- [ ] **Step 1: Write failing tests for required root fields and ID uniqueness**

```python
def test_verify_rejects_duplicate_decision_ids():
    from tools.design.validate import validate_decisions

    errors = validate_decisions([{"id": "C1-Q001"}, {"id": "C1-Q001"}])
    assert errors == ["duplicate decision id C1-Q001"]


def test_verify_rejects_a_packet_with_unresolved_questions():
    from tools.design.validate import validate_packet

    errors = validate_packet({
        "id": "CAP-001",
        "review_state": "draft",
        "unresolved_questions": ["Q-001"],
    })
    assert errors == ["packet CAP-001 has unresolved questions"]
```

- [ ] **Step 2: Run tests and verify the expected red failure**

Run: `python -m unittest tools.design.tests.test_validate -v`

Expected: import or missing-file failure because the validator does not exist.

- [ ] **Step 3: Implement manual stdlib validation**

Validate JSON parseability, `schema_version`, positive `design_revision`, unique IDs, canonical ID shape, alias shape, packet references, and the twelve capability family IDs. Do not add `jsonschema` or another dependency.

- [ ] **Step 4: Implement the CLI adapter**

`python -m tools.design verify` calls `validate_package(Path.cwd())`, prints every error, and returns 1 when any error exists. `extract` writes deterministic JSON with sorted keys and a trailing newline.

- [ ] **Step 5: Add the initial canonical package roots**

Create minimal package roots with schema version 1, design revision 1, and generator identity. `decisions.json` and `artifacts.json` start with empty typed arrays. `capabilities.json` starts with the twelve approved family records in `draft` state and packet references; the gate reports their unfinished packets rather than pretending the design sweep is complete.

- [ ] **Step 6: Run focused validator tests and verify green**

Run: `python -m unittest tools.design.tests.test_validate -v`

Expected: all validation tests pass.

---

### Task 4: Generate the noncanonical source-capability reference

**Files:**
- Create: `tools/design/source_scan.py`
- Create: `tools/design/tests/test_source_scan.py`
- Create: `docs/design/generated/source-capabilities.json`

**Interfaces:**
- `scan_source(root: Path) -> dict`
- Output includes `canonical: false`, `generator`, `source_digest`, `source_counts`, `modules`, `binaries`, `tools`, `asset_summary`, and `inferred_capabilities`.

- [ ] **Step 1: Write failing tests for counts, digest sensitivity, and noncanonical status**

```python
def test_source_view_is_explicitly_noncanonical(tmp_path: Path):
    (tmp_path / "src").mkdir()
    (tmp_path / "src" / "main.rs").write_text("fn main() {}\n", encoding="utf-8")
    view = scan_source(tmp_path)
    assert view["canonical"] is False
    assert view["source_digest"]


def test_source_digest_changes_when_a_source_file_changes(tmp_path: Path):
    (tmp_path / "src").mkdir()
    source = tmp_path / "src" / "main.rs"
    source.write_text("fn main() {}\n", encoding="utf-8")
    before = scan_source(tmp_path)["source_digest"]
    source.write_text("fn main() { println!(\"x\"); }\n", encoding="utf-8")
    assert scan_source(tmp_path)["source_digest"] != before
```

- [ ] **Step 2: Run tests and verify red**

Run: `python -m unittest tools.design.tests.test_source_scan -v`

Expected: import failure because `source_scan` does not exist.

- [ ] **Step 3: Implement a bounded scanner**

Scan tracked/source-relevant files for Rust, Python, Markdown, JSON manifests, and playtest/run files. Count asset files and bytes without reading every large binary into memory. Use `git ls-files` when available and fall back to a bounded filesystem walk. Hash source bytes with SHA-256 in sorted path order.

- [ ] **Step 4: Infer capability hints without promoting them**

Map current modules/binaries/tools to the twelve approved family names as `inferred_capabilities` with evidence paths. Every inferred entry carries `confidence: "source-derived"` and no disposition. The output is always `canonical: false`.

- [ ] **Step 5: Wire `source-scan` into the CLI and generate the draft**

The CLI accepts `source-scan --root . --output PATH` to write the view and `source-scan --root . --check` to compare the checked-in view's source digest without writing it.

Run: `python -m tools.design source-scan --root . --output docs/design/generated/source-capabilities.json`

Expected: deterministic noncanonical JSON with the current source baseline and no human approval claim.

- [ ] **Step 6: Run source-scan tests and verify green**

Run: `python -m unittest tools.design.tests.test_source_scan -v`

Expected: all source scanner tests pass.

---

### Task 5: Add the two-way coverage report and initial C11 ledger projection

**Files:**
- Modify: `tools/design/validate.py`
- Modify: `tools/design/extract.py`
- Create: `tools/design/tests/test_coverage.py`
- Create: `docs/design/generated/coverage.draft.json`
- Create: `docs/design/generated/decisions.md`

**Interfaces:**
- `coverage_report(records: list[dict], ledger: dict) -> dict`
- Report fields: `source_count`, `ledger_count`, `unrepresented_source_ids`, `unknown_ledger_ids`, `reused_aliases`, `malformed_declarations`, `missing_ranges`, and `complete`.

- [ ] **Step 1: Write failing two-way coverage tests**

```python
def test_coverage_requires_one_ledger_row_per_extracted_record():
    report = coverage_report(
        [{"id": "C1-Q001", "aliases": ["Q1"]}],
        {"decisions": [{"id": "C1-Q001", "review_state": "unreviewed"}]},
    )
    assert report["unrepresented_source_ids"] == []
    assert report["unknown_ledger_ids"] == []
    assert report["complete"] is True


def test_coverage_does_not_call_a_partial_ledger_complete():
    report = coverage_report(
        [{"id": "C1-Q001"}, {"id": "C1-Q002"}],
        {"decisions": [{"id": "C1-Q001"}]},
    )
    assert report["unrepresented_source_ids"] == ["C1-Q002"]
    assert report["complete"] is False
```

- [ ] **Step 2: Run tests and verify red**

Run: `python -m unittest tools.design.tests.test_coverage -v`

Expected: import or missing-function failure.

- [ ] **Step 3: Implement two-way coverage and ambiguity reporting**

Compare canonical IDs, not aliases. Preserve source counts and all ambiguity classes. `complete` is false while any source record is unreviewed, absent from the ledger, or malformed; no default keep disposition is permitted.

- [ ] **Step 4: Generate the initial C11 projection**

Populate `decisions.json` with C11-Q211 through C11-Q249 as `reviewed` records carrying their source digest, answer summary, and evidence links. Keep inherited C1–C10 records in the generated draft until the corpus is human-reviewed.

- [ ] **Step 5: Generate readable Markdown and run the gate**

Run: `python -m tools.design project --root .`

Expected: `docs/design/generated/decisions.md` and `coverage.draft.json` are deterministic and `python -m tools.design verify` exits non-zero while explicitly listing the expected unreviewed inherited coverage and unfinished packets. It must not pass merely because the package is syntactically valid.

- [ ] **Step 6: Run all design tests and verify green**

Run: `python -m unittest discover -s tools/design/tests -p 'test_*.py' -v`

Expected: all tests pass.

---

### Task 6: Review, integrate, and hand off the two blocking drafts

**Files:**
- Review: `tools/design/**`
- Review: `docs/design/**`
- Update: `docs/GRILLING-C11.md` only with verified evidence links
- Update: Wayfinder #17 and #18 with command outputs and artifact links

- [ ] **Step 1: Run the complete verification sequence**

```text
python -m unittest discover -s tools/design/tests -p 'test_*.py' -v
python -m tools.design verify
python -m tools.design source-scan --root . --check
```

Expected: unit tests pass; package validation reports only explicitly expected unreviewed inherited coverage; source-scan check is fresh.

- [ ] **Step 2: Inspect the complete diff for scope and data loss**

Confirm that only `tools/design/**`, `docs/design/**`, and the intended decision-record references changed. Confirm `AGENTS.md` remains untouched by this work and no C1–C10 file changed.

- [ ] **Step 3: Record evidence on #17**

Comment with the exact extraction command, source counts, ambiguity counts, generated artifact paths, and test output. Leave #17 open until the human reviews the normalized corpus.

- [ ] **Step 4: Record evidence on #18**

Comment with the source-scan command, source digest, inferred capability counts, and the fact that the view is noncanonical. Leave #18 open until the human approves or corrects the graph.

- [ ] **Step 5: Do not close #16 yet**

The world-truth packet remains blocked until #17 and #18 are resolved and its twelve fields are grilled.

- [ ] **Step 6: Commit only the design-tool slice**

```text
git add tools/design docs/design
 git commit -m "add the typed design package gate"
 git push origin main
```

Do not stage the user's `AGENTS.md` edit.
