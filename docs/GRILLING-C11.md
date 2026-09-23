# The grilling record — C11: the re-grill

`C1`–`C10` settled the old design across 210 questions. The redesign vacates all
of them: **every past question is re-asked**, in reference to the new design, with
the satellites the earlier sessions skipped. The complaint that opened this
campaign is recorded verbatim, because the record keeps what was actually said:

> We are not deleting the old codebase wholesale - it a rework/refactor. We are
> keeping the project's authority in place so that the github page stays, ut we
> are regriilling about every past question, in reference to the new design. The
> Grilling session that landed the wayfinder map in github issues was not detailed
> enough and refused to ask me the proper questions/the right amount of questions
> so I was never able to decide what gets reworked and at what stage. This is
> intended to be a full sweep over the project and what it and it's tools do

**Convention:** `➡️` the recommendation that was made. `✔` the answer received.
`⚠` the answer overrode the recommendation.

---

## The sweep's method

> **Superseded by Q229–Q234; the original claim below is preserved as
> evidence.** A direct scan found that the sources reuse local Q numbers (C1 and
> C9 both begin at Q1), contain alphanumeric satellites, and contain no primary
> declarations for Q197–Q210. The canonical corpus is therefore the normalized
> set actually extracted from C1–C10, not a numeric range. Coverage is proved by
> a typed two-way ledger, not by grep.

1. **Every used question number Q1–Q210 is re-asked** (202 numbers are in use
   across C1–C10), each with its **satellites** — the questions the earlier
   session should have asked and did not. Nothing is assumed; nothing is silently
   simplified; the question count is the one needed to decide, and no less.
2. **Numbering continues from Q211.** A re-grilled past question keeps its old
   number in the citation and receives a **disposition** — `keep` / `rework` /
   `retire` — and the **stage** it lands at. One line each in `DESIGN.md`, which
   is exactly the decision the earlier session never managed to extract.
3. **Coverage is the project's own two-way check.** Before the sweep closes,
   every used number Q1–Q210 is named by exactly one round, and every round names
   only real numbers. Verifiable in one `grep`.
4. **A round asks its whole frontier and waits.** The frontier is what the settled
   decisions have unblocked — never a question that guesses at an answer still
   owed to another open question.
5. Answers land in this record the session they are given. The map (#1) records
   overrides as they happen.

---

## Round 1 — what the redesign even is (Q211–Q216)

### Q211 — what "the old shit does not matter" covers

Does it cover only the code, or also the prose (`GOD_AGENTS.md`,
`UNIFIED_DESIGN.md`, `C1`–`C10`), the records (`saves/`, `playtest/`,
`screenshots/`), and the tools?

➡️ Torch the tree except `LICENSE`, `ATTRIBUTION.md`, `AGENTS.md` and a README
skeleton. Git history is the archive.

✔ ⚠ **Overridden — nothing is torched.** Verbatim answer opening this file. So:
the codebase is **reworked in place**, the project's authority stays so the GitHub
page stays, and the sweep is **full** — the project *and its tools*. The prior
state was committed unaltered (`85fa06a`) before this record existed, so history
is complete and the sweep starts from a clean tree.

### Q212 — what the game is

Does the old claim survive — deterministic, nothing made from nothing, mass drawn,
held, carried, made and audited to the gram — or is the concept on the table too?

➡️ Keep the claim — it is the game's identity — and redesign everything around it
from zero.

✔ **(a) confirmed.** The claim is the one thing carried across.

### Q213 — the stack

Rust + `wgpu` + `winit`, no engine, hand-rolled wheels — kept, or up for redesign?

➡️ Keep it. Determinism is the claim, the toolchain is pinned and proven here.

✔ ⚠ **Overridden — Rust with free crates.** Same language; any crate is on the
table if it earns its place. C1's wheels table (`noise`, `pathfinding`,
`petgraph`, `rand` all hand-rolled because "a dependency update must not change a
replay") is **vacated** and re-asked as Q220.

### Q214 — the finish line of "an actual game"

➡️ **(c)** — a playable founding-day slice as stage one, then grill the route
against what is playable rather than against old prose.

✔ **(c) confirmed.** Land, gather, make, survive; then the next stages are
grilled against the game in front of us.

### Q215 — where the new design lives

➡️ One short `DESIGN.md` with a one-line-per-decision table.

✔ ⚠ **Overridden — short *and* records**: `DESIGN.md` plus per-stage decision
records like this file. `DESIGN.md` stays short; the reasoning lives here.

### Q216 — the GitHub map (#1 and children #2–#11)

➡️ Re-chart: record the override on #1, retire the ten children, chart the new
route when the design settles.

✔ **(a) confirmed.** Done this session: #1 carries the record, #2–#11 close as
superseded — not as refused.

---

## Round 2 — the root layer (Q217–Q222)

The earlier session named this frontier but did not preserve the questions. They
are restored here so the round is resumable and every answer has an explicit
question, recommendation, and disposition.

### Q217 — which principles are immutable

Does the redesign protect only the product identity and claim, or must the current
mechanisms survive too? The protected set considered here was deterministic
replay, conservation, retirement/history, and evidence-backed completion. The
current tests, schemas, tables, save migrations, governance formats, module
shape, and tools were all revisable mechanisms rather than inherited answers.

➡️ **A — protect invariants, revisit mechanisms.** Preserve properties and truths,
not the accidental structure that currently implements them.

✔ **A confirmed.** Deterministic replay, conservation, retirement rather than
silent deletion, and evidence-backed completion remain protected. Tests,
schemas, tables, migrations, governance formats, module structure, rendering,
interface, tools, and simulation mechanisms may be reworked when a stronger design
earns it.

### Q218 — what must be redesigned before playable product work

The choices were: A, groundwork first and then playable vertical slices; B,
horizontal subsystem phases; C, continuous vertical slices from the first product
change; or D, the whole game redesigned before any playable product work.

➡️ **A — groundwork first, then every product stage leaves something playable.**

✔ ⚠ **D overridden — the whole game redesign comes first.** Every gameplay system
and tool is to be reworked before Stage 1 is permitted as playable product work.
The existing build may remain runnable during the rework, but no slice is allowed
to satisfy the Stage 1 product gate until the whole-game redesign gate passes.

### Q219 — how domain, technical layers, and crates relate

The original answer combined deep domain modules, technical layers, and a
workspace crate per major system. Those choices can conflict, so the dedicated
Ask UI asked which organization wins when behavior crosses a layer.

➡️ **Domain-first modules, with crates promoted only when a seam earns one.**

✔ **Domain-first, internal layers confirmed.** A domain owns its behavior and
invariants. Serialization, presentation, storage, and adapters sit behind that
module's interface. A module becomes a workspace crate when independent
compilation, testing, reuse, or ownership pressure earns the seam; a crate per
system is not created merely to make the directory tree look architectural.

### Q220 — what replaces the hand-rolled-wheels rule

May the redesign use established crates for replay-affecting algorithms, and what
condition lets one enter the deterministic path?

➡️ **Determinism is a tested property, not an origin.** Pin and wrap the
dependency, preserve the versioned behavior, and prove it through replay,
save, and conservation gates; refuse one that cannot carry the guarantee.

✔ **A confirmed.** The standard library and established crates are free to be used.
Anything affecting replay must be version-bounded, hidden behind a project-owned
module where practical, locked, and proven by the declared gates. Convenience
alone never earns a dependency, and dependency origin is not itself a reason to
refuse one.

### Q221 — what makes a stage pass

The first answer combined the layered evidence gate with human approval. The
dedicated Ask UI then asked whether a person may override a failed machine gate
and what envelope the replay guarantee covers.

➡️ **A layered evidence gate; a person accepts only after it is green.** A failed
machine check is not silently waived.

✔ **A+C confirmed, with two explicit answers.**

- Build, tests, and lint; replay/equivalence; conservation and record
  verification; save/record compatibility where applicable; human playtest for a
  playable stage; and decision/document coverage all form the gate.
- The person gives final acceptance only after the automated gate is green. A
  failed automated gate cannot be overridden by human approval.
- The replay guarantee is **semantic determinism**: the same declared inputs
  produce equivalent simulation outcomes. Identical floating-point results and
  byte-identical serialized state are not promised.

⚠ The equivalence relation itself is not yet defined. The next round must say what
may differ, what a replay observer compares, and which differences are defects
before this gate is executable.

### Q222 — where each kind of truth is stored

The present design accumulated hand-maintained answers across `README.md`,
`DESIGN.md`, the proposed `PLAN.md`, grilling records, GitHub issues, and source
comments. Which artifact owns which kind of truth?

➡️ **One canonical home per fact.** Design, reasoning, route, implementation, and
evidence must not silently compete.

✔ **A confirmed.**

- `DESIGN.md` owns the current product constitution and concise settled decisions.
- C11 and later stage records own reasoning, alternatives, overrides, and history.
- The GitHub Wayfinder map owns open decisions, dependencies, and the route.
- Code and tests own implemented behavior.
- Run artifacts and records own evidence.
- `PLAN.md`, if reintroduced, is a generated projection and never a separately
  edited source of truth.

---

## Round 3 — the whole-system design sweep (Q223–Q228)

### Q223 — what makes replayed worlds semantically equivalent

Semantic determinism does not say which differences matter. Should equivalence
compare the complete domain projection, selected observable outcomes, conservation
properties, or a declared property set?

➡️ **A canonical authoritative domain projection.** Compare every authoritative
fact and all future-affecting state; tolerate only deliberately approximate
quantities under explicit rules.

✔ **A confirmed.** Two runs are equivalent when their canonical domain projections
agree. Integer and rational authoritative facts are exact; deliberately
approximate domain quantities may use explicit tolerances. Camera pose, frame
statistics, wall time, caches, and serialization order are presentation or
implementation observations and do not decide equivalence. The concrete projection
and tolerance declarations remain to be designed.

### Q224 — what census defines the whole-game scope

Is “the whole game” the player runtime, the source modules, the inherited eight
phases, or every system through which the project is built and operated?

➡️ **A whole-system census.** Nothing capable of changing the product, its truth,
its evidence, or its delivery is silently outside scope.

✔ **A confirmed.** The census covers gameplay, simulation, content declarations,
presentation, persistence, evidence/governance, developer tools, verifier, asset
pipeline, release path, playtests, and records. Every discovered capability gets
an explicit `keep` / `rework` / `retire` disposition and a reason.

### Q225 — what “whole game first” requires before Stage 1

Q218 said the whole game and tools are reworked before Stage 1. The follow-up
asked whether “documents complete” narrows that promise to finishing the design
sweep before implementation and product work.

➡️ **A design sweep first.** Finish the census, target designs, interfaces,
invariants, dispositions, migration decisions, and executable gate
specifications; then plan implementation and Stage 1.

✔ **A confirmed.** Q218 is narrowed, not reversed. The **whole-game design
sweep** must be document-complete before implementation planning or Stage 1
product work. Production code is not required to be wholly reworked before
Stage 1; that implementation follows the completed design.

### Q226 — how parallel work coexists with rework in place

Does “parallel replacement” mean a second production runtime, or may a parallel
executable model be used only to make the target design concrete?

➡️ **Parallel design only.** The current runtime is evidence and a behavioral
reference, not a second production implementation; production code is reworked
in place.

✔ **A confirmed.** A parallel executable specification, model, or probe may be
built when it makes a design decision testable. It is a design artifact with an
explicit retirement or promotion decision. The current game remains evidence
and may be run for comparison, but the target production architecture is not a
second whole-game runtime built beside the old one.

### Q227 — in what order the inherited questions return

Should the sweep follow the old numeric sequence, the file tree, user-selected
batches, or the dependency frontier of the redesigned capabilities?

➡️ **Breadth-first by capability with complete old-question traceability.**

✔ **A confirmed.** Capabilities are re-grilled breadth-first along the frontier
their settled dependencies expose. A coverage ledger proves that every old
Q1–Q210 question receives exactly one disposition, while question numbers remain
stable and satellites stay attached to their parent question.

⚠ **The numeric-range promise is superseded by Q229–Q234.** Breadth-first order
stands; the ledger now covers the normalized corpus actually extracted from
C1–C10, including satellites, rather than a fictional continuous Q1–Q210 range.

### Q228 — what compatibility burden inherited artifacts carry

Must the redesign preserve every save and record, reset inherited data, or make
compatibility an explicit decision for each artifact?

➡️ **Artifact by artifact.** No blanket promise and no blanket destruction.

✔ **A confirmed.** Every inherited save, season record, schema, manifest, review
record, evidence artifact, and compatibility fixture receives an explicit
`keep` / `migrate` / `retire` decision, with its reason and expected evidence.

---

## Round 4 — identity and coverage of the inherited record (Q229–Q234)

### Q229 — what identity every inherited decision carries

C1 and C9 both begin at Q1; later records reuse local ranges; satellites add
suffixes; references are not declarations. What is the stable identity?

➡️ **Campaign-scoped canonical IDs with historical aliases.** Never let a bare
number silently choose the wrong record.

✔ **Campaign-scoped IDs confirmed.** C1–C10 use dense per-campaign canonical IDs
in source order, such as `C1-Q001` and `C9-Q001`. Every historical local number
and satellite spelling remains an immutable searchable alias. New C11 decisions
retain the continuing Q211+ sequence, canonically `C11-Q229`, `C11-Q230`, and so
on.

### Q230 — what the canonical re-grill ledger is

Can Markdown, GitHub issues, or a generated CSV each be edited independently
without becoming a second home?

➡️ **A typed JSON ledger.** Structured fields are validated once; human views
are generated from the same rows.

✔ **Typed JSON ledger confirmed.** Each row carries canonical ID, aliases, exact
source location and text digest, question text, review state, capability,
disposition, reason, replacement decision, target stage, evidence, and human
answer. Markdown and GitHub presentations are projections. The ledger's exact
path is the next artifact decision.

### Q231 — what happens to C1–C10

Should inherited records be edited with disposition labels, consolidated, or
preserved as evidence?

➡️ **Immutable evidence.** The redesign ledger links the exact old answer; the
old prose is not rewritten to make the new design look tidy.

✔ **Immutable evidence confirmed.** C1–C10 remain byte-for-byte historical
records. New dispositions, corrections, and supersessions live in the redesign
record and ledger, linked by canonical ID and source digest.

### Q232 — how missing, malformed, or reused identities are handled

The old promise says Q1–Q210 and 202 numbers. The extracted record has reused
IDs, malformed joint headings, satellites, and no primary Q197–Q210. Should the
range survive, should missing questions be inferred, or should the actual corpus
be normalized?

➡️ **Normalize the actual corpus and remove phantom claims.** A missing number
is not a question until a person gives it content.

✔ **Dense per-campaign sequence confirmed.** Primary declarations and satellites
are extracted, normalized to dense campaign-local canonical IDs in source order,
and retain their historical spellings as aliases. Q197–Q210 are removed as a
phantom range; no placeholder or inferred question is created. Completeness is
measured against the extracted canonical corpus plus its satellites.

### Q233 — what machine gate proves re-grill coverage

The promised “one grep” cannot distinguish declarations from references and does
not parse the record's multiple heading/table forms.

➡️ **A typed two-way coverage gate.** Parse declarations, not every mention, and
prove both source-to-ledger and ledger-to-source coverage.

✔ **Two-way typed coverage confirmed.** The extractor recognizes the source
formats, normalizes primary questions and satellites, rejects unclassified
question-like declarations, requires exactly one reviewed ledger row per
canonical ID, and separately reports reused IDs, aliases, malformed records, and
absent numeric ranges.

### Q234 — what evidence a disposition requires

May old decisions default to keep because code still resembles them, or may a
sweep classify them by majority?

➡️ **Evidence-backed disposition.** `unreviewed` is the only default; the
person's answer is required.

✔ **Evidence-backed disposition confirmed.** Every inherited row begins
`unreviewed`. It cannot become `keep`, `rework`, or `retire` without the person's
answer, a reason, a target capability, a target stage, source digest, evidence
links, and a replacement canonical decision when reworked.

---

## Round 5 — the design-sweep substrate (Q235–Q242)

### Q235 — where the canonical typed design data lives

Should the ledger be a lone JSON file, hidden project data, runtime configuration,
or a visible package that can hold schemas and generated views?

➡️ **A visible `docs/design/` package.** Machine truth, schemas, generated
Markdown, and diagrams live together without being mistaken for runtime config.

✔ **Confirmed.** The canonical package is `docs/design/`. It holds the typed
ledger and capability graph, their schemas, generated human-readable views, and
diagrams. Exact filenames are an artifact-layout decision inside the package,
not a second source of truth.

### Q236 — how the typed ledger is structured

Can JSONL, one file per decision, or flexible JSON trade simplicity for weaker
validation and cross-record integrity?

➡️ **A normalized, versioned JSON object.** Stable ordering, explicit enums,
nullable fields, and ID references make the set validatable and diffable.

✔ **Confirmed.** The ledger is a normalized versioned object. Stable ordering,
explicit enums and nullability, source digests, and ID references are required;
human prose fields do not make the structure free-form.

### Q237 — what capability taxonomy covers the whole system

Should the census inherit eight phases, mirror current modules, collapse into a
few macro domains, or name capabilities by the behavior the product owns?

➡️ **Twelve domain families with sub-capabilities.** The family is the
breadth-first re-grill unit; finer concerns remain explicit children.

✔ **Twelve families confirmed:**

1. World and time
2. Matter and transformations
3. Work and agents
4. Settlement and economy
5. Authority and governance
6. Evidence and records
7. Persistence and replay
8. Client and input
9. Presentation and interface
10. Assets and review tools
11. Playtest and feedback
12. Build, verification, and delivery

### Q238 — which capability opens the breadth-first frontier

Which settled dependency should every other design packet be able to rely on?

➡️ **World truth and replay.** Vocabulary, authoritative facts, input/time
order, and semantic projection constrain every later state model.

✔ **World truth and replay confirmed.** The first capability defines vocabulary,
authoritative world facts, time and ordered inputs, the canonical domain
projection, and replay equivalence. Other capabilities cite its interfaces rather
than inventing parallel state meanings.

### Q239 — how authority and evidence constrain future capabilities

Should every capability duplicate its own governance, leave evidence to an
external verifier, defer it, or consume a foundation with local projections?

➡️ **Foundational rules plus capability-local projections.** Authority and
evidence own the general rules; each capability owns only what it actually does.

✔ **Confirmed.** Authority, evidence, retirement, and record ownership are
foundational constraints. Every capability declares its authority needs, evidence
outputs, retirement semantics, and record projection without forking the general
mechanism.

### Q240 — what one capability design packet must contain

Does completion mean a brief, a formal proof package, a code-first capability,
or one complete product-design packet?

➡️ **A twelve-field packet with zero unresolved questions.** A capability is not
designed because its module exists; it is designed when every field is named.

✔ **Confirmed.** Every capability packet states: purpose; actors and consumers;
vocabulary; invariants; interface; authoritative model; flows; failure and
refusal behavior; verification; artifact dispositions; inherited-decision
coverage; and unresolved questions. Design-sweep completion requires zero
unresolved questions in every packet.

### Q241 — who authors the canonical capability graph

Source, modules, and old documents can generate a useful draft, but current
source does not know the target capability system. May automation become truth
without approval?

➡️ **Generate, reconcile, then obtain human approval.** The extractor is fast;
the person owns the capability model.

✔ **Generated then approved confirmed.** A source/document extractor produces a
typed draft and a reference-only source-derived view. The agent reconciles
missing, duplicate, and badly named capabilities. The person reviews and
corrects the typed graph through Ask UI; only the approved graph is canonical.
The human does not hand-type the graph.

### Q242 — how generated asset trees enter the artifact census

The icon workspace contains roughly 318,000 files and 582 MB while only a much
smaller set is tracked. Must every derivative receive a ledger row?

➡️ **Logical artifact provenance.** A recipe plus manifest can represent many
deterministic derivatives without losing decisions or reproducibility.

✔ **Logical artifact provenance confirmed.** The census records logical
artifacts, producers, recipes, manifests, provenance, compatibility decisions,
and representative outputs. Generated derivatives inherit their producer's
disposition and are checked through manifests and digests rather than one manual
row per file.

---

## Round 6 — typed package layout and enforcement (Q243–Q249)

### Q243 — how facts are separated inside `docs/design/`

Does one giant object win simplicity, one file per object win isolation, or do
normalized fact files preserve one canonical home without a huge merge surface?

➡️ **Separate canonical facts and reference them.** Decisions, capabilities,
artifacts, and packets have different shapes and lifecycles.

✔ **Separated fact files confirmed.** The package uses `decisions.json`,
`capabilities.json`, `artifacts.json`, `schemas/`, `capabilities/<id>.json`,
`generated/`, and `diagrams/`. Stable IDs connect the normalized records; no
fact is manually maintained in two files.

### Q244 — what authors and validates the design package

Direct edits and an ad-hoc script make the typed package only decorative. Who
owns extraction, validation, answer application, coverage, and projection?

➡️ **A small Python design CLI.** The existing Python tool ecosystem is the
right owner; the game runtime does not need to carry design-development tools.

✔ **Python design CLI confirmed.** `tools/design/` owns extraction, validation,
approved-answer application, coverage checks, stable formatting, and projection
generation. The agent invokes it after Ask UI decisions; the person does not hand
edit large JSON structures.

### Q245 — which generated artifacts live in Git

Should generated views be ephemeral, canonical, or retained beside their source?

➡️ **Check in useful projections and verify freshness.** Reviews should not
require a hidden generation step, and checked-in output must not drift.

✔ **Check in and verify confirmed.** Canonical JSON, generated human views, and
reviewed diagrams are retained. Every generated artifact records its generator
and source digests, and `design-verify` refuses stale output.

### Q246 — how the package evolves

Do Git commits, semantic versions, or timestamps carry schema and design
identity?

➡️ **Schema version plus monotonic design revision.** Git identity is evidence,
not a substitute for typed compatibility.

✔ **Schema plus revision confirmed.** Every package root carries an integer
schema version, monotonic design revision, generator identity, generated-from
digests, and explicit migration records when structure changes. Git commit
identity remains linked evidence.

### Q247 — where the source-derived capability view lives

The source-derived graph is a factual draft, not target truth. May it disappear
between runs or compete with the approved graph?

➡️ **A checked-in generated reference file.** Review needs a stable diff, but
its authority must remain visibly separate.

✔ **Generated reference file confirmed.**
`docs/design/generated/source-capabilities.json` records the source extractor,
source digests, current modules, binaries, tools, assets, and inferred
capabilities. It is visibly noncanonical and never promoted without human
approval.

### Q248 — how capability packets are stored

Should all packet bodies live in the graph, in Markdown, in issues, or in one
validated file per capability?

➡️ **One JSON packet per capability.** Each packet is independently reviewable
and schema-valid while the graph owns relationships between packet IDs.

✔ **One JSON per capability confirmed.** Each of the twelve capability packets
lives at `docs/design/capabilities/<capability-id>.json`. The capability graph
references packet IDs, and packet completeness is checked independently.

### Q249 — where the design gate runs

Is validation a local habit, a test detail, CI-only concern, or a named gate?

➡️ **A named local gate that CI/release also run.** The package is design truth,
so stale or invalid design data must fail visibly before product work.

✔ **Local plus CI gate confirmed.** `design-verify` validates schemas,
references, canonical IDs, packet completeness, inherited-decision coverage,
source digests, generated-view freshness, and graph approval. It runs locally
now and is reused by CI and release gates when those exist.

**Generated visual projection (draft):**
`docs/design/diagrams/capability-system.excalidraw` and its rendered PNG show
the approved twelve-family taxonomy, the foundational evidence projection, the
measured source baseline, and the world-truth/replay frontier. The diagram is a
review projection, not the canonical capability graph; dependency edges and
sub-capabilities still require human approval through #18 and #16.

### Typed package implementation checkpoint — 2026-09-23

The approved design-package slice is implemented in `tools/design/` with
stdlib-only extraction, validation, source scanning, and projection. The
canonical package roots and schemas are under `docs/design/`; the inherited
ledger and source-capability views are explicitly generated and noncanonical.

Verified readings:

- inherited extraction: 453 declarations, 7 unparsed headings, 70 reused
  aliases, 2 numeric gaps;
- current C11 projection: 39 reviewed rows, Q211–Q249;
- source view: clean-worktree digest fresh, 118 source files, 55 source-derived
  hints;
- Python design suite: 28 tests pass;
- `cargo clippy --all-targets`: pass;
- `cargo test`: six pre-existing material-table failures, with no Rust source
  changed by this slice;
- `python tools/design_verify.py`: intentionally non-zero with exactly the
  twelve unresolved capability packet questions.

Issues [#17](https://github.com/alacripose/ala-cities/issues/17) and
[#18](https://github.com/alacripose/ala-cities/issues/18) remain open for human
review. #16 world truth and replay remains blocked until those reviews and the
capability packet are resolved.
