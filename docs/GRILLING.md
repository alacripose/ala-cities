# The grilling record

How this project's design was settled, question by question. Six rounds of
questions, each round only asking what the previous round had unblocked. The
point of keeping it is that a decision with its reasoning attached can be
revisited; a decision without one can only be reversed by accident.

**Convention:** `➡️` is the recommendation that was made. `✔` is the answer
received. Where an answer **overrode** a recommendation, it says so.

---

## What prompted it

The starting position was two doctrine documents (`GOD_AGENTS.md`, 51,273 lines,
and `UNIFIED_DESIGN.md`) and a one-line request: *create a city builder game*.
Nothing else existed in the repository — no source, no manifest, no tests.

Facts established before the first question, because finding facts is the
agent's job and not the user's:

- The project is exactly two markdown files and no code.
- `wgpu` 29.0.4, `winit` 0.30.13, `glam`, `bevy_ecs` 0.19.1, `petgraph`, `serde`,
  `ron`, `ab_glyph`, `accesskit_winit` and 743 other crates were present in the
  local cargo cache.
- `glyphon` 0.12 requires `wgpu` 30, which nothing local is verified against.
- `cosmic-text` 0.19 is wgpu-independent and can pair with `wgpu` 29.
- Both installed Rust toolchains link a hello-world; the default gnu toolchain
  needs no external C compiler despite `gcc`/`ld` being absent from PATH. **A
  prediction made in the same command — that the gnu probe would fail — was
  wrong, and is recorded as wrong.**
- crates.io is reachable (`http=200`), so "use an open-source wheel" was a
  choice rather than a capability limit.
- `wasm-bindgen`, `wasm-pack`, `trunk` and `wasm-opt` are all missing, so a
  browser build needs an install before it exists at all.

---

## Round 1 — the three roots

| # | Question | ➡️ | ✔ |
|---|---|---|---|
| Q1 | Deliverable form and runtime | single self-contained `index.html` | **overridden: "Rust game engine, all real time"** |
| Q2 | What binds the game to the doctrine | full conformance, scoped small | (answered in round 2, Q5) |
| Q3 | Core loop and genre | population-layer sim | **overridden: "the gameplay of Cities Skylines 2, while following the unified design doc"** |

**Settled:** Rust. Real-time. CS2-style builder. `UNIFIED_DESIGN.md` applies to
the product surface.

---

## Round 2 — foundation and scope

| # | Question | ➡️ | ✔ |
|---|---|---|---|
| Q4 | Rust foundation | Bevy 0.19.1 | **overridden: "rust + webgpu, no bevy"** |
| Q5 | Which doctrine binds the simulation | both, scoped to build actions | ✔ **both — "but the state machine cannot stall"** |
| Q6 | v1 scope | sim core only | ✔ **"a, b and c"** — the whole CS2 surface, nothing dropped |
| Q7 | How much of `UNIFIED_DESIGN.md` binds | a named subset | ✔ **"as much as possible within the UI"** |

**Settled:** wgpu directly, no engine framework. Two doctrines in force. Full
feature target. Design doctrine applied maximally to the interface.

**Carried forward unresolved:** "the state machine cannot stall" collides with
the doctrine's own measured number — **3 of 230 tickets ever closed**. It became
the load-bearing question of round 4.

---

## Round 3 — runtime, target, timing

| # | Question | ➡️ | ✔ |
|---|---|---|---|
| Q8 | Target runtime | native desktop | ✔ `A` |
| Q9 | What replaces Bevy | all ours, no egui | ✔ `A` — **"but avoid reinventing wheels when open source solutions are available"** |
| Q10 | What "cannot stall" may mean | bounded close + one escalation | ✔ **delegated: "your best decision, just as long as the game state always has something extra for the user to do"** |
| Q11 | How a+b+c get built | three sequential campaigns | ✔ **"a+b"** — staged as campaigns, nothing dropped |
| Q12 | Materials, assets, fonts | no raster textures; OS fonts at runtime | ✔ `A` |
| Q13 | Real-time model | fixed timestep | ✔ **"fixed timestep for agent movement that interpolates based off their work done"** |

**Settled:** native `winit` + `wgpu`; wheels taken where they are large and not
determinism-critical; a **bounded** closure rule plus an always-present supply of
real work; staged delivery of the full feature set; named solid materials and
runtime-referenced fonts; and an interpolation source that is *work completed*,
never a frame clock.

---

## Round 4 — the wheel map and the governed verbs

| # | Question | ➡️ | ✔ |
|---|---|---|---|
| Q14 | The concrete wheel map | take the pure-engineering wheels, gate installs on a version check | ✔ **"use already available high performance tools where available"** |
| Q15 | Where work comes from | the sim files requests | ✔ `a` |
| Q16 | What the save *is* | both: `ron`/binary world + season record + headless verifier | ✔ `C` |
| Q17 | Never-granted list vs CS2's real verbs | fiction labelled as fiction; loans demoted; demolition retires | ✔ `a` |
| Q18 | Build mode vs Inspect lens | two modes, tools absent in the lens | **overridden: `b` — "just like Cities Skylines, the user should be able to build asynchronously and work on their own goals away from the agents"** |
| Q19 | Map scale and agent budget | 256×256, ~5k agents, LOD built to scale | ✔ `a` |

**Settled:** the wheel map (and its consequence — `glyphon` refused, `cosmic-text`
taken); the case-queue as the source of work; a three-part persistence story; the
doctrine's honesty rules mapped onto gameplay verbs; **no read-only mode**, which
narrows the §14 mode-scoping claim; and a small city built to scale.

---

## Round 5 — governance granularity, time, and testing

| # | Question | ➡️ | ✔ |
|---|---|---|---|
| Q20 | Who the ticket system governs | builds plus sim-filed aggregate cases | **overridden: `C` — sample citizen behaviour into tickets** |
| Q21 | Time model and the stall bound | 20 Hz sim, bound in sim-seconds | ✔ **yes — "remember to support higher framerates since my monitor can display up to 240hz"** |
| Q22 | Definition of done and the oracle | mechanical invariants + headless verifier | ✔ `A` — **"but make sure every version can be playtested within the repo, letting the user leave feedback at the end of each testing stage"** |
| Q23 | HUD architecture | CS2 toolbar + Popit per-object page | **overridden: "We are building a city builder game. The design docs are there for best reference."** |
| Q24 | Audio | none in C1 | ✔ **`b` — adopt `rodio` now** |

**Settled:** sampling as the work generator, deduped by case identity; a
240 Hz-conscious render path; playtestability and feedback as first-class
requirements of every stage; procedural audio; and — in a23 — **the doctrine
demoted from law to reference**, which is the single most consequential answer in
the whole record.

---

## Round 6 — the last frontier

| # | Question | ➡️ | ✔ |
|---|---|---|---|
| Q25 | The a5/a23 contradiction: is the doctrine law or guidance? | take a23 as the correction | ✔ **delegated: "your best guess here lol"** |
| Q26 | The sampling policy | event-driven, deduped, carrying a count | ✔ `A` |
| Q27 | Playtest and feedback | in-game form + per-stage runnable builds | ✔ `a` — **plus: pause-menu feedback at any time, and interaction capture during scheduled test runs** |
| Q28 | Audio content | procedural synthesis | ✔ `a` |

**Settled:** doctrine as reference; one ticket per *condition* carrying a real
count; recording that is always available and always visible; and zero audio
assets to license.

---

## Corrections ledger

Recorded rather than quietly overwritten, because the corrections are the most
useful part of this document:

| Was | Now | Because |
|---|---|---|
| a5: "both doctrines **bind**" | a23 supersedes: docs are **best reference** | An explicit correction, then delegated to the agent when the user was asked to arbitrate |
| Recommendation Q1: single HTML file | Rust engine, native | The user named the stack directly |
| Recommendation Q3: population-layer sim | CS2 gameplay | The user named the game |
| Recommendation Q4: Bevy | wgpu direct | The user excluded it |
| Recommendation Q18: mode-scoped tools | Always-available tools | The user wants to build while the city runs |
| Recommendation Q20: builds + aggregates only | Sample behaviour into tickets | The user wants the ticket population to come from the population |
| §14.3 absence claim | Withdrawn | No read-only mode exists to scope against |
| Predicted gnu-toolchain failure | It linked | The check was wrong, not the toolchain |

---

## Decisions taken under delegation

The user said "your best guess here lol" and "go for the moon", so these were
made rather than asked, and are each reversible:

1. **Doctrine as reference, not gate.** Kept because each genuinely improves the
   game: the append-only season record doubling as save, telemetry and replay;
   retire-don't-delete; refusals that name the vocabulary; a governor that fails
   closed. Dropped because each only served compliance: the conformance report
   as a deliverable, the mechanical/judgement separation as paperwork.
2. **Replay is in.** Recorded inputs + seeded RNG + fixed tick means a playtest
   replays headlessly. Nearly free once determinism is committed.
3. **Interaction capture is local-only, in-repo, never networked.** Capped, and
   it refuses rather than silently truncating. Recording is a visible state.
4. **Toolchain pinned to `msvc`** via `rust-toolchain.toml`.
5. **`rayon` deliberately not installed** until something measures a need.

## Deviations from the signed-off plan

Each is recorded in `README.md` and is reversible:

| Planned | Built | Why |
|---|---|---|
| `bevy_ecs` | struct-of-arrays | At this size it is less code *and* faster; adoptable later |
| `cosmic-text` | deferred | Needed for shaping and i18n, not for the HUD's own text |
| `noise`, `pathfinding`, `petgraph`, `rand` | hand-rolled | All determinism-critical; a dependency update must not change a replay |
| `glyphon` | refused | Requires wgpu 30, which nothing here is verified against |

## Explicitly open — deliberate deferrals, not silent assumptions

- C2's service coverage models and milestone curves.
- C3's industry supply chains, external connections, and weather/seasons.
- Districts and policies.
- A future replay/observer mode — the place where §14.3's absence rule would
  finally apply.
- Migrating to `wgpu` 30 in order to adopt `glyphon`.
- A real build identity hash for playtest records (wants a build script).

## Final confirmation

> **1: Confirm. Go for the moon.**
> **2: Real tangible repo, tracking everything as you would in a github project for maximal compatibility.**

---

# Appendix — verbatim transcript

Everything above compresses and interprets. This appendix does not: it is the
raw text of what was actually asked and answered, so that a paraphrase elsewhere
in this document can always be checked against the thing it paraphrases. Where
the two disagree, **this appendix wins.**

Answers are reproduced exactly as written, including typos (`asyncroonously`,
`asychronously`) and the mislabelled `q29` in round 6.

## The initiating request

> read the two files in this project, use the skill grilling with docs and
> wayfinder, and create a city builder game

## Round 1 — the three roots

Questions: Q1 deliverable form and runtime · Q2 what binds the game to the
doctrine · Q3 core loop and genre.

> **a1:** rust game engine, all real time, but follows the gameplay of cities
> skylines 2, while following the unified design doc

## Round 2 — foundation and scope

Questions: Q4 Rust foundation · Q5 which doctrine binds the simulation · Q6 v1
scope · Q7 how much of `UNIFIED_DESIGN.md` binds.

> **a4:** rust+ webgpu, no bevy
> **a5:** a both but the state machine cannot stall
> **a6:** a, b, and c
> **a7:** As much as possible within the ui

## Round 3 — runtime, target, timing

Questions: Q8 target runtime · Q9 what replaces Bevy · Q10 what "cannot stall"
may mean · Q11 how a+b+c get built · Q12 materials, assets, fonts · Q13
real-time model.

> **a8:** A
> **a9:** A but we should avoid reinventing wheels when open source solutions
> are available
> **a10:** Your best decision, just as long as the game state always has
> something extra for the user to do
> **a11:** a+b
> **a12:** A
> **a13:** fixed timestep for agent movement that interpolates based off their
> work done

## Round 4 — the wheel map and the governed verbs

Questions: Q14 the concrete wheel map · Q15 where work comes from · Q16 what the
save *is* · Q17 never-granted list vs CS2's real verbs · Q18 build mode vs
Inspect lens · Q19 map scale and agent budget.

> **a14:** Yes, use already available high performance tools where available
> **a15:** a
> **a16:** C
> **a17:** a
> **a18:** b, just like cities skylines, the user should be able to build
> asyncroonously and work on their own goals away from the agents
> **a19:** a

## Round 5 — governance granularity, time, and testing

Questions: Q20 who the ticket system governs · Q21 time model and the stall
bound · Q22 definition of done and the oracle · Q23 HUD architecture · Q24
audio.

> **a20:** C
> **a21:** yeah, just remember to support higher framerates since my monitor can
> display up to 240hz
> **a22:** A but make sure that every version can be playtested within the repo,
> letting the user leave feedback at the end of each testing stage
> **a23:** We are building a city builder game. The design docs are there for
> best reference.
> **a24:** b

*Note on a20:* the recommendation was (a) — builds and aggregates only. The
answer was `C` — sample citizen behaviour into tickets — with the dedup policy
left to the agent when it was raised again in round 6.

## Round 6 — the last frontier

Questions: Q25 the a5/a23 contradiction · Q26 the sampling policy · Q27 playtest
and feedback · Q28 audio content. The user's fourth answer was labelled `q29`; it
answers Q28.

> **a25:** Your best guess here lol
> **a26:** A
> **a27:** a yes, but allow the user to open the pause menu to leave text
> feedback at any time in the builds - and every time a test is running
> (scheduled to be running first) and the user interacts, extensive data
> analysis and collection happens to see what the user clicked, and where
> **q29:** a

## Sign-off

> **1:** Confirm. Go for the moon.
> **2:** Real tangible repo, tracking everything as you would in a github
> project for maximal compatibility

## Standing instruction carried into the build

Asked for after the first commit, while C1 work was in flight:

> save the grilling questions and the answers I gave you to a document and
> continue where you were working
