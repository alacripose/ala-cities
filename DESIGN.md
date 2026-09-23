# ala-cities — the design

The redesign's whole design document, kept short on purpose. The reasoning lives
in `docs/GRILLING-C*.md`, the authority in `AGENTS.md` / `GOD_AGENTS.md` /
`UNIFIED_DESIGN.md`, and the route on GitHub issue #1. **This file and the records
are the only prose the project grows.** Every decision gets one line in the table
below the moment it is made.

## The claim — what the game is

- A deterministic city builder in Rust: **nothing is made from nothing**. Mass is
  drawn, held, carried, made and audited to the gram.
- `(seed, tick, input)` reproduces the same world. That property — not a
  dependency's origin — is what a dependency has to satisfy.
- Buildings **retire**; nothing is deleted. A claim about the world is read back
  out of the world, never trusted from the act that made it.
- The codebase is **reworked in place** (Q211): a full sweep of the program and
  its tools — not a deletion, not a from-scratch rewrite.

## The build — what lands next (the only place that says so)

| Stage | What it is | State |
|---|---|---|
| **Groundwork** | the seams, the declared tables, the crate policy, the module skeleton — nothing playable | being specified (Q217–Q222) |
| **Stage 1 — the founding day** | the playable slice: a party lands, gathers, makes, survives. A person can play it | open |
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

*Round 2 (Q217–Q222) is asked; its answers land here and in
`docs/GRILLING-C11.md`.*
