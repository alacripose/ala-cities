# ala-cities — the design

The redesign's whole product design, kept short on purpose. `AGENTS.md` remains
the operating authority; this file and `docs/GRILLING-C*.md` hold the product
constitution and its reasoning; GitHub issue #1 holds the route.
`GOD_AGENTS.md` and `UNIFIED_DESIGN.md` remain inherited references until their
questions are re-grilled. **This file and the records are the only product-design
prose the project grows.** Every decision gets one line in the table below the
moment it is made.

## The claim — what the game is

- A deterministic city builder in Rust: **nothing is made from nothing**. Mass is
  drawn, held, carried, made and audited to the gram.
- `(seed, tick, input)` reproduces the same logical world. Equivalence is decided
  by a canonical authoritative domain projection, not byte-identical serialization
  or raw floating-point results; its concrete fields and tolerances remain open.
- Buildings **retire**; nothing is deleted. A claim about the world is read back
  out of the world, never trusted from the act that made it.
- The codebase is **reworked in place** (Q211): a full sweep of the program and
  its tools — not a deletion, not a from-scratch rewrite.

## The build — what lands next (the only place that says so)

| Stage | What it is | State |
|---|---|---|
| **Whole-game design sweep** | a whole-system census plus target designs, interfaces, invariants, dispositions, migration decisions, and gate specifications for every gameplay system and tool | in specification; implementation planning and Stage 1 product work wait for its completion |
| **Stage 1 — the founding day** | the playable slice: a party lands, gathers, makes, survives. A person can play it | blocked by the whole-game design-sweep gate |
| **Later stages** | grilled against the playable slice when stage 1 plays — the route answers to the game, not to old prose | fog |

Every stage has a **gate**: named commands with expected **readings**, the
**evidence** they file, and — wherever the stage has anything to play — a
**playtest**. An **override** is recorded, never silent.

## Vocabulary — fresh words, fixed here (Q194)

| Word | Means |
|---|---|
| **stage** | a unit of the build with a gate; a ticket on the map |
| **gate** | named commands with expected readings, plus the evidence they file |
| **reading** | a number or state a gate expects to observe before it passes |
| **evidence** | what a run produced, filed where a later run can re-check it |
| **playtest** | a person plays the stage and leaves what they saw |
| **override** | a recorded change to an earlier decision; the original stays readable |
| **fog** | what is not yet specified — deliberately open, never assumed |
| **destination** | what every stage orients to: the founding day, playable |

## Decisions — one line each

| Q | Decision | Answer |
|---|---|---|
| Q211 | what the redesign covers | ✔ rework/refactor in place; authority (AGENTS.md, GitHub) stays; **every past question re-grilled**, project and tools both |
| Q212 | the game's claim | ✔ kept: deterministic, nothing made from nothing, mass audited to the gram |
| Q213 | the stack | ⚠ **Rust with free crates** — any crate that earns its place (the keep-it-hand-rolled recommendation overridden) |
| Q214 | the finish line | ✔ playable founding-day slice first; the route is grilled against it afterwards |
| Q215 | where the design lives | ✔ this file, plus per-stage records in `docs/GRILLING-C*.md` |
| Q216 | the GitHub map | ✔ re-chotted: children #2–#11 retired as superseded, the route returns when the design settles |
| Q217 | what the redesign protects | ✔ invariants protected; mechanisms revisable — tests, schemas, migrations, governance formats, structure, UI, and tools may be reworked |
| Q218 | order of redesign and play | ⚠ **whole-game sweep first**, narrowed by Q225: the design sweep completes before implementation planning or Stage 1 product work |
| Q219 | module and crate shape | ✔ domain-first modules with internal layers; promote to workspace crates only when the seam earns one |
| Q220 | dependency rule | ✔ determinism is a tested property; pinned/wrapped dependencies are allowed when they pass replay, save, and conservation gates |
| Q221 | gate and replay envelope | ✔ layered automated gate, then human acceptance; failed automated gates cannot be overridden; replay is semantic determinism, with Q223 choosing the domain-projection method |
| Q222 | authority of artifacts | ✔ one canonical home per fact; `PLAN.md`, if used, is generated rather than independently edited |
| Q223 | semantic replay equivalence | ✔ canonical authoritative domain projection; exact integer/rational facts, explicit tolerance only for declared approximations, presentation/cache/serialization order excluded |
| Q224 | whole-game census | ✔ gameplay through delivery: simulation, content, presentation, persistence, evidence, tools, verifier, assets, release, playtests, and records all receive keep/rework/retire dispositions |
| Q225 | meaning of “whole game first” | ✔ **design sweep first**; census and target design artifacts complete before implementation planning or Stage 1 product work — production code need not be wholly reworked first |
| Q226 | parallel replacement | ✔ parallel executable models may serve as design artifacts; production code is still reworked in place and the old runtime is evidence, not a second target runtime |
| Q227 | re-grill order | ✔ breadth-first by redesigned capability and dependency frontier, with complete traceability over the normalized extracted corpus defined by Q229–Q234 |
| Q228 | inherited artifact compatibility | ✔ artifact-by-artifact keep/migrate/retire decisions; no blanket compatibility or destruction |
| Q229 | inherited decision identity | ✔ dense campaign-scoped canonical IDs; old local numbers and satellites remain immutable aliases |
| Q230 | re-grill ledger | ✔ typed JSON is canonical; Markdown and GitHub are generated projections |
| Q231 | C1–C10 authority | ✔ immutable historical evidence; dispositions and supersessions live in the redesign record |
| Q232 | missing and reused question IDs | ✔ normalize the actual extracted corpus densely per campaign; remove phantom Q197–Q210 rather than infer questions |
| Q233 | re-grill coverage gate | ✔ typed two-way source/ledger coverage; reused, malformed, aliased, and absent IDs are reported separately |
| Q234 | disposition proof | ✔ every row starts unreviewed and needs the person's answer, reason, capability, stage, digest, evidence, and replacement decision when reworked |
| Q235 | typed design package | ✔ canonical data, schemas, generated views, and diagrams live under `docs/design/` |
| Q236 | ledger structure | ✔ normalized versioned JSON with stable ordering, explicit enums/nullability, digests, and ID references |
| Q237 | capability taxonomy | ✔ twelve domain families with explicit sub-capabilities; breadth-first re-grill proceeds by family |
| Q238 | first redesign capability | ✔ world truth and replay: vocabulary, authoritative facts, time/input order, canonical projection, and equivalence |
| Q239 | authority/evidence placement | ✔ foundational constraints with capability-local projections; no duplicated governance mechanisms |
| Q240 | capability design packet | ✔ twelve named fields and zero unresolved questions required for design-sweep completion |
| Q241 | capability graph authorship | ✔ source-derived draft and reference view; agent reconciliation; human-approved typed graph is canonical; no hand-typing required |
| Q242 | artifact census granularity | ✔ logical artifacts, producers, recipes, manifests, provenance, and compatibility decisions; generated derivatives inherit producer disposition |

*Rounds 2–5 (Q217–Q242) are answered in `docs/GRILLING-C11.md`. The next
frontier opens **world truth and replay** and generates the first typed design
package drafts for review.*
