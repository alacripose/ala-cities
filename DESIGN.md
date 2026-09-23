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
- `(seed, tick, input)` reproduces the same logical world. The guarantee is
  semantic determinism, not byte-identical serialization or floating-point
  results; the equivalence relation remains to be defined.
- Buildings **retire**; nothing is deleted. A claim about the world is read back
  out of the world, never trusted from the act that made it.
- The codebase is **reworked in place** (Q211): a full sweep of the program and
  its tools — not a deletion, not a from-scratch rewrite.

## The build — what lands next (the only place that says so)

| Stage | What it is | State |
|---|---|---|
| **Whole-game redesign** | every gameplay system and tool reworked behind the target domain-first seams; no Stage 1 product gate is accepted until this redesign is complete | in specification; the route and completion gate remain open |
| **Stage 1 — the founding day** | the playable slice: a party lands, gathers, makes, survives. A person can play it | blocked by the whole-game redesign gate |
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
| Q218 | order of redesign and play | ⚠ **whole game and tools first**; Stage 1 is not permitted as playable product work until the whole-game redesign gate passes |
| Q219 | module and crate shape | ✔ domain-first modules with internal layers; promote to workspace crates only when the seam earns one |
| Q220 | dependency rule | ✔ determinism is a tested property; pinned/wrapped dependencies are allowed when they pass replay, save, and conservation gates |
| Q221 | gate and replay envelope | ✔ layered automated gate, then human acceptance; failed automated gates cannot be overridden; replay guarantee is semantic determinism, with its equivalence relation still open |
| Q222 | authority of artifacts | ✔ one canonical home per fact; `PLAN.md`, if used, is generated rather than independently edited |

*Round 2 (Q217–Q222) is answered in `docs/GRILLING-C11.md`. The next round defines
the semantic-equivalence relation and the completion gate for the whole-game
redesign.*
