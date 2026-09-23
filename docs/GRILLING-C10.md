# C10 — the re-grilling, and the one plan

*Opened 2026-09-23, immediately after the round-28 slice's tables landed. It exists because the
previous nine documents are a **design record**, not a plan: the only statement of "what lands
next" is the eight-phase table at the end of `GRILLING-C9.md` plus a handful of "in the order it
has to happen" lists in its build notes — and the user's report, from outside the code, is that
adding to the codebase **fights it**, which is the failure mode this document exists to remove.*

**Convention** (inherited from `docs/GRILLING.md`): `➡️` is the recommendation made, `✔` the
answer received, and an answer that **overrode** a recommendation says so. A correction is recorded,
never quietly overwritten. Where this document claims a fact about the code, the fact was measured
(tests run, tables read) and the measurement is named.

**Question numbering continues the record.** `GRILLING-C9.md` closed at Q176, so this document
opens at **Q177**.

## The situation, measured rather than assumed

Facts established before the first question, because finding facts is the agent's job:

* **Ten documents, four jobs.** `GOD_AGENTS.md` (51 273 lines) and `UNIFIED_DESIGN.md` are the
  doctrine and the design reference; `docs/GRILLING.md` plus `GRILLING-C2…C9.md` are the design
  *record*; `README.md` is the product's front page; `AGENTS.md` is the standing instruction
  (*grill, and save the grilling*). **No document is the plan.**
* **The order exists in prose, in three places.** `GRILLING-C9.md`'s eight phases (substrate →
  ledger → processes → agents → institutions → progression → food → growth), round 28's
  "the founding party, the needs, the off-road walk and the player's proposal" list, and the
  build notes' "what this slice deliberately leaves open". They agree today and nothing makes
  them agree.
* **There is no gate on a stage.** The gates that *do* exist are real and mechanical —
  `cargo test --lib` (202 tests), `tools/materials/emit.py` (the emitting gate, which refuses to
  write while it reports a defect), `src/materials/*::table_defects()`, `verify.exe` (the season
  record's read-back), `src/design.rs`'s `OPEN` list — but no document says which of them a stage
  must pass before the next one starts.
* **The code is red right now, uncommitted.** `tools/materials/declare.py` and
  `src/materials/generated.rs` carry six parts and prices for a `Shelter` and a `Kiln`
  (`shelter.posts/thatch/screen`, `kiln.body/mouth/stack`) that a previous session invented and
  no `BuildingKind` reads. `cargo test --lib` reports **197 passed, 5 failed**: the typed
  `world::PARTS` table never got the rows (23 vs 29 surfaces), and four prices are charged for
  parts no structure declares.
* **Two facts that bound what a first day can be.** Deposits are `stone`, `sand`, `clay`
  (all `ceramic`), `coal` (**`soil`**), `iron_ore` (`metal`); patches grow `timber` and
  `plant_fibre` (both `organic`). `plan_material` searches **deposits only**, so a structure whose
  body is `organic` has no source today, and a structure whose body is `soil` would draw coal.
  The user's standing correction: **there is no clay in the environment** — the world generator
  pass (Q168) has not landed — so the founding day may not be built on clay, brick or fired
  anything.
* **The refactoring complaint, in the user's words** (2026-09-23): *"A lot of the older code
  doesn't like to be modified, but actually it's supposed to be able to be adaptable. So when we
  add new features in they actually sit into the system instead of having it fight against
  itself."* Named instances found by reading, not by being told: `effects::table_defects` carries
  the kind vocabulary as the literal `["home", "shop", "factory", "power"]`; `recount`,
  `post_month`, `assign_jobs`, `post_demand_tasks`, `BuildingKind::{name, part_prefix, zone,
  build_cost, capacity}` and `main.rs::building_height` each match on `BuildingKind` exhaustively;
  five test bodies enumerate the four kinds by hand; `STRUCTURE_MASS_G` is a second kind-keyed
  table; and `world::PARTS` vs `declare.py`'s `WORLD_SURFACES` is the one two-view pair with a
  two-way check (a pattern the other vocabularies do not have).
* **The season machinery already exists and is not wired to a plan.** `saves/season_2026_s1/`
  holds `season.json`, `events.jsonl`, 221 tickets, evidence, retirements and corrections;
  `verify.exe` reads them back and refuses a claim with nothing behind it (a
  `completed_and_validated` ticket with no grade-A/B evidence is a finding).

## Round 1 — the frontier as it stands

*Everything below is independent of everything else below: none of these answers is needed to ask
another. The questions the plan's **content** raises (which stages exist, what each one's checks
are, what each stage's decisions are) depend on these and so belong to round 2 and later — as does
the user's own clarifying information, which is invited at the end.*

| # | Question | ➡️ recommendation | ✔ answer |
|---|---|---|---|
| Q177 | What the new document **is** — one record that re-grills, or a rewrite of the meta doc, and where the plan itself lives | a new record, `docs/GRILLING-C10.md`, whose product is one root-level `PLAN.md` | — |
| Q178 | What a **stage** and a **gate** are, and whether a gate can block | a stage is a contract-sized slice; a gate is exact commands with expected readings **plus** a playtest with feedback **plus** evidence in the season store; the next stage refuses to start, and a waiver is a recorded override | — |
| Q179 | How far the **re-grilling** goes, and in what order | triage first, then re-ask driven by the plan's stage order — never the whole record at once, never twice | — |
| Q180 | What the **refactor** covers, and when it lands | the *seams* now (every hand-maintained enumeration becomes a declared table with a two-way check, gated), the *structure* later (5 008-line `sim/mod.rs`, 2 728-line `main.rs`, 1 823-line `gov/mod.rs`) | — |
| Q181 | What happens to the **red six parts** and the two kinds | revert the six, keep the kinds *undecided*, and let the plan's materials stage declare them — the kiln was held back by the user, and the parts were never a decision | — |
| Q182 | Whether the **plan gates the plan's own work** (docs, refactor) or only the game | it gates everything: the document and the refactor are stages with gates like any other | — |
| Q183 | The **playtest loop** as a gate | per a22/a27, every stage ends playable, with the run's record and the player's feedback filed as the stage's acceptance evidence | — |

### Q177 — what the new document is

The user asked to be grilled "on creating a new grilling doc, re-grilling me on the content of the
previous docs, to create one singular holistic plan document that checks and gates progress for
each stage". That is two artifacts with different jobs, and the first question is whether they are
two documents or one.

Options: **(a)** two — `docs/GRILLING-C10.md` is the *re-grilling record* (decisions re-confirmed,
overridden or retired, with the reasoning), and `PLAN.md` at the root is the *single holistic plan*
(stages, each with its gates, its evidence and its reading); **(b)** one — the plan carries its own
decisions inline and there is no separate record; **(c)** the re-grilling *rewrites* `docs/GRILLING.md`
and the plan replaces the eight-phase table inside `GRILLING-C9.md`.

➡️ **(a)**, and the ownership rule that makes it singular: **`PLAN.md` is the only place that says
what lands next**; the grilling documents keep *why*; `GOD_AGENTS.md` and `UNIFIED_DESIGN.md` keep
being doctrine and reference; `README.md` keeps describing what exists. Every "in the order it has
to happen" list in `GRILLING-C9.md`'s build notes becomes a pointer to the plan rather than a
second home for the same claim.

### Q178 — what a stage and a gate are

The user's word is "checks and gates progress for each stage". A gate that is a feeling is not a
gate, so the question is what makes one mechanical here, and whether it is allowed to stop work.

Options: **(a)** a gate is *documentation*: the stage lists what must be true and a person checks
it; **(b)** a gate is *commands*: a named command per gate with an expected reading
(`cargo test --lib` → 202 passed; `python tools/materials/emit.py` → no defect; `verify.exe` →
`loose_g` zero), and the plan names which of them a stage must show; **(c)** a gate is *refusal*:
(b) plus the season machinery — the stage's pass files its evidence and its retirement, the next
stage's work is not posted until the previous stage's gate reads green, and `verify.exe` (or a
successor reader) prints the stage's state; **(d)** (c) plus a **playtest**: the stage is played
and the player's feedback is filed as acceptance evidence.

➡️ **(d)**, and the two properties that keep it honest: a gate **names its command and its
expected reading** (so a green gate can be re-run by a reader, not believed), and a stage that
cannot pass is **retired as unverified with the reason recorded** rather than left open — the
existing retirement taxonomy already carries that shape. A **waiver is an override**: recorded in
the record with the reason, like every other override, never a silent pass.

### Q179 — how far the re-grilling goes

The record is nine documents and several hundred decisions; re-asking all of them is a month of
questions and would re-open things that are built, tested and standing.

Options: **(a)** re-ask everything, oldest first; **(b)** triage every decision into *standing and
built*, *decided but unbuilt*, *contradicted by the code or another document*, *contested now* —
put the first class in a veto list the user can strike from, and re-ask the rest; **(c)** let the
plan drive it: the stages are written in order, and each stage opens with a short round that
re-confirms only the decisions that stage depends on, so the whole document is re-grilled exactly
once and never out of order.

➡️ **(c), inside (b)'s triage.** The triage is the *index* — every decision in C2–C9 mapped to
the stage it is needed by, and to one of the four classes above — and the plan's stage rows cite
it. A decision that no stage needs is not re-asked at all: it is listed as *not needed yet*, which
is a statement the record can hold.

### Q180 — what the refactor covers, and when

The user's complaint is that adding to the codebase fights it. Two different failures wear that
name, and they cost different amounts to fix:

* **The seams**: a fact that must be written in several places to exist once. The measured
  instances are listed in *The situation* above. Fixing them means one declared table per
  vocabulary, a two-way check (the `world::PARTS` ↔ `WORLD_SURFACES` pattern, generalised), and a
  test that fails when a new row is added without the check — so the *next* feature sits in rather
  than fights.
* **The structure**: `sim/mod.rs` (5 008 lines), `main.rs` (2 728) and `gov/mod.rs` (1 823) hold
  several concerns each. Splitting them is not needed for the next feature to sit in; it is needed
  for a *reader* to hold the file in mind, and it becomes cheaper the more the seams above are
  declared.

Options: **(a)** seams now, as a gated stage before new kinds land, and structure later as a named
stage whose row states its own justification and cost; **(b)** both now, structure first;
**(c)** neither as a stage — refactor opportunistically as features land; **(d)** structure only.

➡️ **(a)**, with the structure stage **named and sized in the plan** rather than left implicit,
and a rule that makes the seams provable: *a new row in any vocabulary costs exactly one declared
table edit and zero match-arm edits outside it, and the gate names the row if it does not.*

### Q181 — the red six parts, and the two kinds

The tree currently carries an uncommitted change that is red: six parts and prices no kind reads,
invented by a previous session, for a `Shelter` and a `Kiln` the user has now **held back** (the
kiln, because there is no clay in the environment to make one out of yet; the shelter, because its
organic body has no source and its real construction is the hauled build the founding party is
for).

Options: **(a)** revert the six parts and the prices, leaving the kinds undecided, and let the
plan's materials stage declare what a shelter and a kiln are — with the round-28 record unchanged,
since the invention was never a decision; **(b)** keep them as a declared-but-unused vocabulary so
the shapes are visible, and let the stage decide later; **(c)** finish the kinds as they are,
against the user's holding-back.

➡️ **(a)** — a red gate is not a foundation, and the record's own rule is that a claim nothing can
check is not a claim. The parts are cheap to re-declare from the stage's answer.

### Q182 — whether the plan gates its own work

If `PLAN.md` says what lands next, does it govern the document work and the refactor too, or only
the game?

Options: **(a)** everything — the plan's first stages are "the plan exists and is gated" and "the
seams are declared", each with its own gate; **(b)** only the game — the docs and the refactor are
process, and process has its own maturity.

➡️ **(a)**, because the failure the user reported is a process failure: if the plan cannot gate the
work that produces the plan, it is a description of intent rather than a gate. The first stage's
gate is deliberately trivial and hard to fake: `PLAN.md` exists, every stage row names its
commands and expected readings, and each row's decisions are cited from this record.

### Q183 — the playtest as a gate

a22 and a27 settled that **every version is playtestable inside the repo** and that the player can
leave feedback at the end of each stage, with interaction capture during scheduled runs. The
question is whether that is a gate or a courtesy.

Options: **(a)** a courtesy — the playtest is offered, the feedback is kept, but a stage can pass
without it; **(b)** a gate — a stage that a person cannot play does not pass, and the stage's own
record carries what the player saw and wrote; **(c)** a gate only for stages that change the
product surface.

➡️ **(b)**, because the record's most expensive pattern so far is *"nothing happens in a fresh
world"* — a fact that was only visible from playing it, after the tables behind it were green.

### The clarifying information, invited

The user said there is information to give about all of this that the questions above cannot ask
for. It is taken first, and it re-cuts this round if it changes a premise: anything about what the
code is supposed to be, what "adaptable" means for it, what the plan is for, or what the previous
documents got wrong.

## Round 1 — answered

*Answered 2026-09-23, all seven at once. Two answers overrode their recommendation, and one of
them re-cut the round: the process itself changed while it was being asked.*

| # | ➡️ recommendation | ✔ answer |
|---|---|---|
| Q177 | two artifacts, `GRILLING-C10.md` + root `PLAN.md` | ✔ **(a) confirmed** — and the ownership rule stands: `PLAN.md` is the only place that says what lands next |
| Q178 | commands + season evidence + playtest, blocking, waiver recorded | ✔ **(d) confirmed** |
| Q179 | plan-driven re-grill inside a triage index | ✔ **(c) inside (b) — and an override of scope**: *"don't forget also the doc named just grilling.md"* — the re-grill covers **`docs/GRILLING.md` (C1)** as well, so the record to be triaged is `GRILLING.md` + `C2…C9`, and the triage index must carry C1's decisions too |
| Q180 | seams now, structure later | ⚠ **overridden: "everything miss nothing"** — the refactor excludes nothing: the seams *and* the structure, every vocabulary and every god-file. What that means for sequencing is round 2's question, because "everything" is not a stage, it is a property |
| Q181 | revert the six parts, kinds undecided | ⚠ **overridden in the direction of the whole**: *"everything is going to be changed. recheck agents.md for the new skill suite"* — the codebase is going to change wholesale, so the six parts are not the question; the process is. Re-read below |
| Q182 | the plan gates everything, including itself | ✔ **(a) confirmed** |
| Q183 | the playtest is a gate | ✔ **(b) confirmed** |

### The fact that re-cut the round: `AGENTS.md` changed mid-session

The standing instruction was rewritten at 06:44 on the day this round was asked, and carries a
**skill suite** the previous sessions did not have. Recorded verbatim, because it is now the
process this document's plan has to live inside:

```text
ALWAYS USE THE WAYFINDER SKILL FROM MATTPOCOCK

USE THE EXCALIDRAW SKILL TO MAKE DIAGRAMS OF EVERYTHING YOU CAN
https://github.com/coleam00/excalidraw-diagram-skill

ALWAYS GRILL THE USER AND SAVE THE GRILLING DECISIONS INSTEAD OF ASSUMING ANYTHING (ESPECIALLY SIMPLIFICATIONS) BEFORE CONTINUING WHEN FACED WITH ANY AMOUNT OF UNCERTAINTY

ALWAYS USE https://github.com/mattpocock/skills THESE SKILLS

USE THESE SKILLS IF YOU CAN: https://github.com/AlpacaLabsLLC/skills-for-architects

AND https://github.com/adrianpuiu/claude-skills-marketplace
```

What the suite *is*, measured from the installed skills rather than guessed:

* **`wayfinder`** — plan work larger than one session as a **map** (a single issue labelled
  `wayfinder:map`, its **decision tickets** children of it), worked one ticket per session until
  the way to the **destination** is clear. Tickets are typed `research` (AFK, a subagent),
  `prototype`, `grilling` (always with `domain-modeling`), or `task`; blocking is the tracker's
  native dependency relationship; the **frontier** is the open, unblocked, unclaimed children.
  Crucially: **wayfinder plans, it does not do** — a ticket resolves a *decision*, and the map is
  done when nothing is left to decide.
* **`setup-matt-pocock-skills`** — a per-repo setup the suite assumes: an **issue tracker**
  (`docs/agents/issue-tracker.md`), triage labels, and domain-doc layout. The local-markdown
  tracker's wayfinding operations are `.scratch/<effort>/map.md` + `.scratch/<effort>/issues/NN-slug.md`
  with `Type:`/`Status:`/`Blocked by:` lines. **This repo has not been set up**: no `docs/agents/`,
  no `.scratch/`, no `CONTEXT.md`.
* **The other four** — `grilling` (already in use), `domain-modeling` (CONTEXT.md + ADRs),
  `research`, `prototype` (all installed), plus `wayfinder-task-retirement` (the retirement state
  machine already implemented in this repo's season store). **Not installed**: the excalidraw
  diagram skill and the two optional suites (`skills-for-architects`, `claude-skills-marketplace`).

### What round 1 settles about `PLAN.md`

`PLAN.md` is the one place that says what lands next, and by Q178/Q182/Q183 a stage in it is
*only* a stage when it names: its **scope**, the **decisions it depends on** (cited from the
triaged record), its **gates** (commands + expected readings, the evidence they file, the playtest
they end in), and its state — `not started / open / gated-green / retired-unverified`.

---

## Round 2 — the map, the plan, and what "everything" means

*The frontier round 1 unblocked. Facts first, as before: the repo is not set up for the suite; the
repo's own tracker is the governor store (`saves/season_2026_s1/` — 221 tickets, evidence,
retirements, `verify.exe` read-back), which is an *execution* record; wayfinder wants a *decision*
record and defaults to local markdown when no tracker is provided.*

| # | Question | ➡️ recommendation | ✔ answer |
|---|---|---|---|
| Q184 | Which tracker holds the map: local markdown, the repo's own governor store, GitHub, or the map in one and the tickets in another | local markdown under `.scratch/plan/` for the **map and its decision tickets**, and the governor store for **execution gates** — the two are different kinds of record and the plan is the bridge | — |
| Q185 | What `PLAN.md`'s **destination** is — the thing every ticket orients to | the destination is **the founding day plays**: a party lands, survives by hand, sleeps, eats, warms, and everything it does is drawn, held and audited. Every earlier phase is a prerequisite of it, every later phase is beyond it | — |
| Q186 | How a plan **stage** relates to a map **ticket**, and who owns "what lands next" | a stage is a ticket of type `task` (its work must exist before a decision can be judged), a stage's open questions are `grilling` tickets blocking it, its unknowns are `research`, and the stage's **gate** is filed in the governor store; the map owns the route, `PLAN.md` is the map's readable projection | — |
| Q187 | What "**everything is going to be changed**" means for the code | a **rework in place, total in coverage and staged in time**: nothing is exempt from the seams-and-structure rule, but the rewrite happens stage by stage with the tests, the two-view tables, determinism, the save format's migrations and the season record all preserved — a from-scratch rewrite is a different question and would be asked as one | — |
| Q188 | The **red six parts** while all this happens | revert them now, so the tree is green before the map is charted — a red gate under a plan whose first rule is *a gate names its reading* is the worst possible starting state | — |
| Q189 | Whether to run the suite's **setup** (writes `docs/agents/issue-tracker.md`, `domain.md`, triage labels, and an `## Agent skills` block into the existing `AGENTS.md`) | yes, run it now, with the tracked choices above written into it | — |
| Q190 | The **uninstalled skills** (`excalidraw`, `skills-for-architects`, `claude-skills-marketplace`) | install `coleam00/excalidraw-diagram-skill` now (AGENTS.md asks for diagrams "of everything you can", and the plan wants a picture); inspect the other two and report what they offer before installing anything | — |

### Q184 — which tracker holds the map

Wayfinder insists the tracker be *provided* and defaults to local markdown. This repo already has a
tracker — `saves/season_2026_s1/` with 221 tickets, evidence grades, retirements and a taxonomy
`verify.exe` reads back — but it is an **execution** record: its tickets are work, its terminal
states are retirement reasons, and it has no notion of a *decision*, a label, a blocker or a
frontier.

Options: **(a)** the map and its decision tickets live as **local markdown** (`.scratch/plan/…`),
and the governor store keeps doing what it does — the plan bridges them, filing each stage's gate
as evidence and retirement in the season; **(b)** the map lives in the **governor store** as a new
record kind (a decision ticket with blockers and a type), which means extending that store's
vocabulary; **(c)** **GitHub Issues** for the map (`gh` CLI), and the season stays local; **(d)** the
map in markdown, the tickets mirrored into the season as they resolve.

➡️ **(a)**. The two records answer different questions — *what is decided and what is next* versus
*what was done and what it cost* — and Q178 already put the gates in the second one. Merging them
would make the plan's own history hostage to the execution record's retention rules.

### Q185 — the destination

A map's destination fixes its scope: everything past it is **out of scope**, and the fog only
gathers toward it. "The whole game" is not a destination; it is the horizon.

Options: **(a)** the destination is **the founding day plays** — a party lands with nothing, and
their first day (forage, timber and fibre, a shelter, a fire, sleep, hunger, warmth, death if it
comes) is survivable *and* every step of it is drawn, held and audited; **(b)** the destination is
**the plan exists** (the map is charted, `PLAN.md` written, every stage gated) — a planning
destination, with the game as fog beyond it; **(c)** the destination is a later milestone (the
stone rung's economy, or the first institution); **(d)** the whole build, in phases, as one map.

➡️ **(a)**, with (b) as the *first* deliverable inside it rather than a separate effort: the
founding day is the smallest destination that forces every layer to exist at once — substrate,
mass, processes, agents, gates, playtest — and it is the one the round-28 record already says is
next. **(d)** is the fog; it is not yet specifiable as tickets, which is exactly what fog means.

### Q186 — stage, ticket, and who owns "what lands next"

Q177 gave `PLAN.md` sole ownership of "what lands next"; wayfinder wants that route in a map of
resolution tickets.

Options: **(a)** `PLAN.md` **is** the readable projection of the map (the map is canonical, the
file is what a person reads), and each stage is a map ticket of type `task` whose gate is filed in
the season; **(b)** `PLAN.md` is canonical and the map is a working artifact that gets deleted when
the plan is written; **(c)** every stage is a *grilling* ticket and the plan is only the summary of
resolved decisions.

➡️ **(a)**, and the split that makes it pay: a stage's **open questions** are `grilling` tickets
that block it, its **unknowns** are `research` tickets (AFK, fired in parallel), its **"nothing to
decide but a discussion is blocked until it exists"** work is a `task` ticket, and its **gate** is
the season's record — commands, readings, evidence, playtest. A stage is *specified* when its
grilling tickets are resolved; it is *green* when the season says so.

### Q187 — what "everything is going to be changed" means

Q180's override said *"everything miss nothing"* and Q181's said *"everything is going to be
changed"*. Two readings, and they cost very different amounts:

Options: **(a)** **rework in place, total in coverage, staged in time** — every vocabulary gets one
declared table and a two-way check, the god-files are split, the dead ends are removed, and it
happens stage by stage inside the plan, with the 202 tests, the two-view tables, the
`(seed, tick, input)` determinism rule, the save-format migrations and the season record
**preserved** — each stage's gate says what still holds; **(b)** a **from-scratch rewrite** of the
Rust source against the declared tables, keeping only the tables, the record and the tests as the
acceptance target; **(c)** rework only what the destination's stages touch, with the rest left as
fog.

➡️ **(a)**. "Everything is going to be changed" is satisfied by coverage — nothing is exempt — while
(a) keeps a green tree, a playable build and a readable history the whole way, which (b) cannot
evidence until it is finished. If (b) is what is wanted, it is a *bigger* decision and deserves its
own question with its own gate (what must the rewrite keep passing to prove it is the same game).

### Q188 — the red six parts

They are six parts and prices for a `Shelter` and a `Kiln`, invented by a previous session, read by
no kind, and currently failing five tests. Q181's override said the codebase is changing wholesale.

Options: **(a)** revert the two files now, leaving the tree green and the kinds undecided;
**(b)** leave them — the refactor will rewrite that area anyway; **(c)** keep them and finish the
kinds as a `task` ticket under the map.

➡️ **(a)**. A gate is the thing this plan is made of, and a red tree at the moment of charting makes
every reading suspect — including the playtest the destination's own gate depends on.

### Q189 / Q190 — the setup, and the uninstalled skills

The suite asks for a per-repo setup (`docs/agents/issue-tracker.md`, `docs/agents/domain.md`,
triage labels, an `## Agent skills` block appended to the existing `AGENTS.md`), and AGENTS.md names
three skills that are not installed.

➡️ **Run the setup** with the answers of this round written into it (tracker: local markdown; domain
docs: single-context `CONTEXT.md` + `docs/adr/`), and **install only the excalidraw diagram skill**
for now, because AGENTS.md asks for diagrams and the map wants one. The other two suites are
inspected and reported before anything is installed from them, per the standing rule that community
skills are not vetted and are confirmed first.

### Q189 / Q190 — as built

The setup ran, and the skills installed, because the user's answers went further than the
recommendations: **all of them, if I could**. Recorded as done, with what each one turned out to be:

* `docs/agents/issue-tracker.md`, `docs/agents/domain.md`, `docs/agents/triage-labels.md` written,
  and an `## Agent skills` block appended to `AGENTS.md`. **The tracker file records GitHub, not
  local markdown** (Q184's answer), so the wayfinding operations it documents are `gh` CLI calls:
  a `wayfinder:map` issue, child issues as sub-issues, blocking via GitHub's native issue
  dependencies, the frontier as *open children with no open blocker and no assignee*, and the claim
  as `--add-assignee @me`.
* **Installed** (project-scoped, `.agents/skills/`, 156 skills):
  `coleam00/excalidraw-diagram-skill` → `excalidraw-diagram`;
  `adrianpuiu/claude-skills-marketplace` → `project-planner`;
  `AlpacaLabsLLC/skills-for-architects` → the Arch Studio suite (a *building*-architecture suite —
  zoning, workplace programming, work plans, quantity extraction — useful as domain reference for a
  city builder, not a software-planning method);
  `StChiotis/Library-First-Engineering` → the `lfe-*` pipeline (`lfe-grill-with-docs` → `lfe-prd`,
  inspector, mutation-verify, complexity-check, whats-next);
  `obra/superpowers` → `writing-plans`, `verification-before-completion`, `brainstorming`,
  `executing-plans`, and the skill-authoring set;
  `avibebuilder/claude-prime` → `review-code`, `diagnose`, `test`, `skill-creator`, `self-evolve`;
  `mattpocock/skills` → the rest of the suite (`ask-matt`, `retro`, `writing-*`, `setup-*`).
* **`reporails/cli` has no skills** — it is a CLI, not a skill source, and `npx skills add` refused it
  with *"No valid skills found"*. Reported rather than silently skipped.
* **`smith.attck.com`** is not a skill repo either; it is a hosted service, and nothing about it was
  installed.
* **The two planning methodologies AGENTS.md points at** (`docs.aiblueprint.dev/advanced/apex` and
  `/oneshot`) were read: Apex is an **Analyze → Plan → Execute → eXamine** loop with checkpoints in
  `.agents/apex/runs/<run-id>/`, risk-based review, *validate and examine*, and *prove and hand off*
  ("it distinguishes local checks, provider state, deployed artifacts, and authenticated live
  behavior. Unavailable checks and pre-existing failures are reported explicitly"). That is the same
  shape as this document's gate: **acceptance criteria with current evidence, and a re-plan when
  evidence invalidates an assumption**. It is adopted as the plan's stage loop; the CLI itself is not
  installed.

## Round 2 — answered

*Answered 2026-09-23. Three answers overrode their recommendation, and the round's biggest effect was
not in the answers at all: **`AGENTS.md` was edited again, mid-round.***

| # | ➡️ recommendation | ✔ answer |
|---|---|---|
| Q184 | map + tickets in local markdown, gates in the governor store | ⚠ **overridden: GitHub Issues** — *"recheck agents.md, do not use the game's built in files for this that would be incredibly dumb when we're about to refactor the whole thing. Github issues would be nice"*. The season store is the game's own record and stays that; the plan's map and its decision tickets live on GitHub |
| Q185 | the founding day plays | ⚠ **overridden: (d) the whole build in phases** — the map's destination is the whole build, phase by phase |
| Q186 | map canonical, `PLAN.md` its projection | ✔ **(a) confirmed** |
| Q187 | rework in place, total coverage, staged | ✔ **(a) confirmed** |
| Q188 | revert the six parts now | ⚠ **overridden: (b) leave them for the refactor** — the tree stays red until the refactor stage rewrites that area; the plan's first stage therefore *starts* by naming the red state rather than pretending it is green |
| Q189 | run the suite setup | ✔ **yes, run it** — done, above |
| Q190 | excalidraw only, inspect the rest | ⚠ **overridden: all of them** — *"all of them if you gosh darn can reread agents.md"*. Done, above |

### The third `AGENTS.md`, and what it changes

Re-read after being told to twice, and it had grown again. The additions, verbatim:

```text
IF YOU CAN UTILIZE MULTIPLE PLANNING SKILLS TOGETHER TO MAKE MARKDOWN AND DIAGRAMS THAT WOULD BE SUPER HELPFUL
https://docs.aiblueprint.dev/advanced/apex
https://docs.aiblueprint.dev/advanced/oneshot
UNIFIED_DESIGN.MD IS A NON CONCLUSIVE /EXCLUSIVE DESIGN DOCUMENT THAT OUTLINES SOME OF THE BASIC PRINCIPALS
https://github.com/StChiotis/Library-First-Engineering
https://github.com/reporails/cli
https://github.com/obra/superpowers
https://smith.attck.com/
https://github.com/avibebuilder/claude-prime
Commit often to actual github when a commit lands in the local project
```

Three consequences the plan must carry:

1. **`UNIFIED_DESIGN.md` is demoted again** — from "design doctrine" to *non-conclusive, non-exclusive,
   basic principles*. C1's a23 had already made both doctrine documents *best reference*; this narrows
   `UNIFIED_DESIGN.md` explicitly. The plan cites it as reference and never as gate.
2. **Commits are pushed to GitHub as they land.** The remote is
   `https://github.com/alacripose/ala-cities.git`, `gh` is authenticated as `alacripose`, and the
   local branch was **69 commits ahead** of `origin/main` when the round closed.
3. **Multiple planning skills together, with markdown and diagrams.** Wayfinder (the map),
   `project-planner` (requirements → design → task breakdown with traceability), the `lfe-*` pipeline
   (grill-with-docs → PRD, inspector, whats-next), `writing-plans`, `brainstorming`, Apex's
   checkpointed loop, and `excalidraw-diagram` for the pictures. The plan is where they meet: the map
   is the route, `PLAN.md` is its readable projection, the ADRs and `CONTEXT.md` are the domain
   vocabulary (created lazily by `domain-modeling`), and each stage's gate is the evidence.

### What round 2 settles

* **The plan's home is GitHub**, as one `wayfinder:map` issue with child tickets, and `PLAN.md` in the
  repo is its projection — regenerated from the map, never a second home for a decision.
* **The destination is the whole build, phase by phase**, so the map's fog is *all eight phases of
  C9's order*, and its first tickets are the ones that can be specified now: the map itself, the
  refactor's seam inventory, the triage index of `GRILLING.md` + C2…C9, and the first stage.
* **The tree is red on purpose until the refactor stage**, which means the first stage's gate must
  *name* the five failing tests and carry them as a known reading rather than a surprise.

## Round 3 — the map's charting

*Opened when the suite was in place. Wayfinder's charting is its own session: name the destination,
fan out breadth-first, create the map, create the tickets that can be specified now, wire blocking in
a second pass, and fire the research tickets. The destination is settled by Q185. What follows is the
breadth-first fan-out — the frontier as it can be stated today.*

| # | Question | ➡️ recommendation | ✔ answer |
|---|---|---|---|
| Q191 | The map's **notes**: which skills every session on this map must consult | wayfinder + grilling + domain-modeling; Apex's loop for stages; `writing-plans` before code | — |
| Q192 | The **phases as tickets**: nine or eight, and does the map carry the refactor and the plan itself as tickets | nine children: *the plan and its gates*, *the seams*, then C9's eight phases | — |
| Q193 | The **triage index** as a ticket type, and its size | a `research` ticket (AFK): every decision in `GRILLING.md` + C2…C9 mapped to the phase that needs it and classified standing / unbuilt / contradicted / contested | — |
| Q194 | Whether **`CONTEXT.md` and ADRs** are created now or lazily | lazily, by `domain-modeling`, when the first term or decision needs a home — the suite's own rule | — |
| Q195 | The **first frontier ticket** to work after charting | the triage index (AFK), then the seams inventory; the founding-day grilling follows | — |
| Q196 | Whether the **red five tests** are carried as a known reading in the map's Notes | yes — named with their count and the reason, so the first stage's gate can retire them explicitly | — |

### Q191 — the map's Notes

Wayfinder says the map's Notes section holds the domain and the skills every session should consult.
Options: **(a)** wayfinder + grilling + domain-modeling as the standing three, with Apex's
Analyze→Plan→Execute→eXamine loop as every stage's working method, `writing-plans` before any code, and
`excalidraw-diagram` when a picture explains better than prose; **(b)** wayfinder + grilling only;
**(c)** the whole installed suite, listed.

➡️ **(a)**. A Notes section that lists 156 skills is a Notes section nobody reads; the four named are
the ones every ticket on this map actually needs.

### Q192 — the phases as tickets

C9's order has **eight phases**. Q182 added two things the plan must gate: *the plan and its gates*
and *the seams*. Options: **(a)** nine children — `the plan and its gates`, `the seams`, then the eight
phases in order, each blocked by its predecessor and by the seams; **(b)** eight children, with the plan
and the seams as tickets *inside* phase 1; **(c)** eight, with the refactor folded into each phase.

➡️ **(a)**. The plan and the seams are prerequisites of *every* phase rather than parts of the first
one, and making that visible in the tracker is the point of putting the map there.

### Q193 — the triage index

The record to be triaged is `GRILLING.md` (C1) plus `C2…C9` — several hundred decisions. Options:
**(a)** one `research` ticket that produces the index as a file (a table: decision, where it lives, the
phase that needs it, its class, and what would falsify its class), worked by a subagent and published
as a gist/artifact linked from the ticket; **(b)** one ticket per document; **(c)** no index — re-grill
per stage and let the classes emerge.

➡️ **(a)**. Q179 asked for the triage as the index the plan's stages cite; one artifact, one ticket,
AFK.

### Q194 — `CONTEXT.md` and ADRs

Options: **(a)** lazily, by `domain-modeling`, when the first term or decision needs a home;
**(b)** now — write a `CONTEXT.md` from the glossary the record already uses (mass, ledger, claim,
account, ticket, retirement, stage, gate) and ADRs for the load-bearing decisions already made.

➡️ **(a)**, which is the suite's own rule, with one exception: the plan's own vocabulary (**stage**,
**gate**, **evidence**, **reading**, **override**, **fog**, **destination**) is new and has no home yet,
so the plan's drafting session is a `grilling` ticket and `domain-modeling` is consulted *inside it*.

### Q195 — the first frontier ticket

The frontier after charting is whatever has no open blocker. Options: **(a)** the triage index first
(AFK, fires immediately), then the seams inventory, then the founding-day grilling; **(b)** the
founding-day grilling first, because it is what the destination wants; **(c)** the seams first.

➡️ **(a)**. The triage index and the seams inventory are research, they need no human loop, and every
later decision is cheaper once they exist — which is exactly what a frontier research ticket is for.

### Q196 — the red five tests

Q188 left the tree red. Options: **(a)** map Notes carries it explicitly ("five tests fail: the typed
surface table is short six rows and four prices are charged for parts no kind declares — left for the
refactor stage by decision Q188"), and the seams ticket's gate names retiring them; **(b)** leave it
unmentioned.

➡️ **(a)**. A known red carried as a reading is the difference between a decision and an oversight.
