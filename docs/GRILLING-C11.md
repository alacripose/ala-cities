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

*Asked this session, in one frontier: what rework preserves, what the stages are,
how the god files split, what the wheels rule becomes, what a gate is, and where
"what lands next" lives. Answers recorded below when given.*
