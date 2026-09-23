# ala-cities

A real-time city builder in Rust on `wgpu`, with an evidence-first record of
everything built, refused, and retired. No game engine: the renderer, the
simulation, and the record are this repository.

> **Status: presentation pass landed; icon library in progress.** Each campaign
> keeps its own runnable build, its own tag, and its own playtest record — see
> `playtest/` and the tags on this repository. Nothing here is described as working
> unless a build has been seen doing it; the open list is at the bottom of this
> file, and it is kept honest rather than short.

---

## Play it

**From a release** — download the latest release, unzip nothing, run
`ala-cities.exe`. It needs a GPU with Vulkan or DX12 support. `verify.exe` sits
beside it and re-checks a season record without needing a window or a GPU.

**From source**

```bash
cargo run --release            # the game
cargo test                     # 85 tests, headless
cargo clippy --all-targets     # expected clean
cargo run --bin verify         # the oracle: re-check a season record, no GPU
```

The toolchain is pinned in `rust-toolchain.toml`.

### Controls

| Input | Action |
|---|---|
| **Left click** | use the current tool: place a road tile, drop a power line, demolish (retire), inspect |
| **Left drag** | draw a road segment, or paint zoning over the tiles the drag covered |
| **Right drag** | orbit — free yaw *and* free tilt |
| **Middle drag** | pan, without changing the angle you are reading the city at |
| **Scroll** | zoom toward the cursor |
| **1 / 2 / 3 / 4 / 5** | road · zoning · power · demolish · inspect |
| **R / C / I** | residential · commercial · industrial (while the zoning tool is in hand) |
| **Space** | pause and resume |
| **Tab** | speed 1× → 2× → 3× → 1× |
| **Home** | return the view to the configured angle — never touches the city |
| **L** | the ledger: tickets, evidence, refusals, retirements |
| **H** | help |
| **Escape** | pause menu (which also pauses the sim) |
| *In the pause menu* | **U** interface scale (100 / 125 / 150 / 200 %) · **S** save · **O** load · **F** leave feedback |

---

## What makes this one different

Most builders let you demolish a neighbourhood and the past disappears with it.
Here, **buildings retire rather than being deleted**. Placing a road files a ticket
before the road exists; validating it means *reading the road back out of the
world*, never trusting the click that placed it. A ticket that cannot be verified
closes honestly as `completed_but_unverified` — it is never promoted to "validated"
to make a number look better.

The population is sampled into **cases**: four hundred hungry citizens in one
district file **one** ticket reading `400`, not four hundred tickets. That keeps the
work queue human-scale, and it is also what makes it drainable — every ticket
reaches a terminal state within a bounded number of **sim-seconds**, at every game
speed, so speeding the game up can never bypass a gate.

The chrome you are looking at is not decoration over an invisible state. Contract,
ticket, governor version and identity status are on screen because that is the
actual authority you are acting under, and a surface is never allowed to look more
authorised than it is.

---

## The simulation

| | |
|---|---|
| Sim tick | **fixed 20 Hz** |
| Calendar | 40 ticks/day, 30 days/month — **60 seconds of sim per month at 1×** |
| Speed | 1× / 2× / 3× = 20 / 40 / 60 ticks per second; pause = 0 |
| Map | 256 × 256 tiles |
| Render | **uncapped**, present mode chosen from what the surface reports (`Mailbox` preferred, then `AutoNoVsync`, then `Fifo`) so a 240 Hz display is not held to vsync |
| Interpolation | sim alpha × each agent's **completed work fraction** — never an assumed frame time |

Determinism is not a feature bolted on; it is why some wheels are hand-written. The
RNG is a hand-rolled **PCG32**, terrain is seeded hash-based value noise, and the
road graph is ours, because a dependency update must never be able to change what a
replay produces. Ticket sampling draws from that same seeded RNG, or nothing
downstream could be re-checked.

---

## The evidence record

Everything the game does to the world leaves a record:

* `saves/season_*.json` — the **season record**: tickets and their terminal states,
  evidence, retirements, corrections. Tracked in git, because it is the city's own
  history.
* `saves/world.ron` — the world snapshot. Local and disposable, and git-ignored for
  that reason: 4.49 MB measured, written in **30.7 ms**, autosaved once a sim-month.
* `playtest/<stage>/` — how to run a stage, the interaction captures, and the
  feedback left, recorded at confidence `tentative` because one playtest is one
  playtest.

`verify.exe` reads those and **re-derives the claims from the world instead of
trusting them**, then exits non-zero if a claim and the world disagree. It is
deliberately conservative: it distinguishes a *contradiction* from a *supersession*
(a recorded later action that legitimately changed the tile), refuses to judge
claims that closed after the snapshot it is reading, and marks what it cannot check
as **uncheckable** rather than passing it. Getting that distinction right was not
cosmetic — an earlier version reported twelve "contradictions" on a real session and
**all twelve were false**.

---

## The interface

The world is **instanced quads in world space with a real depth buffer**, viewed
through a **free orbit camera** — rotation and tilt are free, the projection is
orthographic so two tiles are the same size wherever they are on screen, and
buildings are extruded one height step per level with contact shadows from the same
light that shades their faces. Clicking resolves a tile by **inverse projection
against the ground plane**, which is exact at any angle and is tested across **84
yaw/pitch combinations** plus a 32-case tile round-trip. Deriving a tile from a
screen rectangle would be a tile or two wrong the moment the camera tilts.

Where you are on the grid is answered by three things rather than one: a **tile
highlight** under the cursor in every mode, a **local grid patch** that fades with
distance while a placement tool is in hand, and a **bearing and coordinate readout**
whose north and east arms are projected from world space, so they say where the city
runs rather than where a button was put.

Type and spacing come from `src/design.rs`, whose steps are a **closed enum**:

| Step | 100 % | Used for |
|---|---|---|
| `micro` | 12 px | micro-labels beside the thing they annotate |
| `small` | 14 px | secondary rows |
| `body` | 16 px | message and content text |
| `title` | 20 px | panel headings |
| `display` | 26 px | plaques and critical identifiers |

Sizes are not passed as numbers anywhere — `Step::px(ui)` is the physical size
used to create Glyphon's cosmic-text metrics. Spacing comes from the 4-unit scale,
targets are ≥ 48 px (WCAG 2.5.8's 24 px is the hard floor, not the goal), and the
interface scale re-lays out rather than stretching. Glyphon performs shaping,
font fallback, runtime etagere atlas packing, and screen-space rendering for every
text path.

### The check fails closed

`design::verify` runs at startup and **refuses to start** if it finds anything: type
steps below their floors, spacing off the unit, a target under the floor, a token
with no style record, or a contrast pair below 4.5:1. It reports *every* defect
rather than the first, because a check that reports one problem per run is a check
somebody stops running. It was made to fail on purpose — a 20 × 20 target added
deliberately — and the build refused to start and named it.

What a green run means is that the requirements are present in the source. It says
nothing about whether the interface is legible to a person, nothing about layout at
other window sizes, nothing about composite contrast on a *rendered* frame, and
nothing about frame timing under load. Those are printed as **open**, by the same
program, every run.

```
design: ui scale 100% · 5 type steps · 6 spacing steps · 11 targets
design: smallest target: 48 px (floor 24, comfortable 48)
design: style table: 32 tokens, 32 records, 0 with no record
design: TextBody on Panel: 14.24:1 (AA floor 4.5)
design: TextMuted on Panel: 8.57:1 (AA floor 4.5)
design: open — judgement: whether this is legible to a person, layout at every
        window size, composite contrast on a rendered frame, and frame timing under load
renderer ready adapter=NVIDIA GeForce RTX 2070 SUPER backend=Vulkan
        format=Bgra8UnormSrgb depth=Depth32Float
first frame world_opaque=26167 world_overlay=49 interface=410 yaw_degrees=0.0
        pitch_degrees=52.0 zoom=0.703125
```

---

## Architecture

| Path | What lives there |
|---|---|
| `src/sim/` | the world: `rng` (PCG32), `terrain`, `road` (graph + A*), `citizen` (agents, commutes), and the fixed-step `World` |
| `src/gov/` | the governance layer: tickets, evidence, retirements, the append-only season record, the sampler and the bound |
| `src/render.rs` | the `wgpu` client: instanced quad batcher, depth, camera, inverse-projection picking, and Glyphon render pass |
| `src/hud.rs` | the style table: OKLCH tokens, the text/panel/plaque vocabulary, measured contrast |
| `src/design.rs` | the type and spacing scales, the target floor, and `verify` — the fail-closed gate |
| `src/session.rs` | the session capture: what was clicked, where, and the build it happened in |
| `src/main.rs` | the client: window, input, tools, HUD layout, the sim↔render loop |
| `src/bin/verify.rs` | the oracle: reads a record, checks it against the world, exits non-zero on disagreement |
| `src/bin/pick.rs` | the icon review picker: six authored candidates in a 2×3 grid, and what to change next |

---

## Icons

Icons are **renders, not drawings**. Each one is parametric geometry lit by a fixed
three-light rig and rendered headless in Blender, authored in **OpenPBR** vocabulary
and rendered through the Principled BSDF v2 that Blender actually implements — with
the mapping and every deviation recorded per icon, because Blender 5.3 Alpha has no
OpenPBR node and no MaterialX operators (upstream issue #145127). `cargo build`
never needs Blender: the renders are committed artifacts with a manifest, and
`tools/icons/generate.py` is how they were made. Shared authoring settings live in
`assets/icons/reference.blend`: it contains only the editable three-light rig,
camera/world defaults, and named material roles—not icon meshes. Open it in Blender,
adjust `icon_ref:light:key`, `fill`, or `rim`, and edit the `icon_ref:material:*`
Principled materials. Regenerate the settings reference set with:

```bash
blender --background --factory-startup --python tools/icons/generate.py -- --review stage-1 --limit 1
```

The generator imports those lights and materials on every run, records
`reference-blend` provenance in the manifest, and falls back to the Python defaults
only when the blend is absent. Recreate the settings file itself with:

```bash
blender --background --factory-startup --python tools/icons/reference_blend.py
```

**Status, plainly:** the first pass shipped **13 glyphs at 3 sizes (24/32/48)**. Since
then the ladder is **24/32/48/64/96**, every icon is rendered as **six declared
candidates** — one canonical plus five named alternates — and the material check is a
**measured differential swatch**: each material is rendered as a flat reference under the same
rig and the icon's own render is compared against it, because a model of how a
material *should* arrive is a claim, and a swatch is a measurement.

What is still open, and visible rather than quiet: the inventory is a hand list and
should come from the game's own registry; motion is settled as *store-driven* (a gear
turns only while the sim is actually working that record) and not yet implemented;
and **no icon has a target yet** — the pipeline says so per icon rather than
pretending the set is finished. See `docs/GRILLING-C2.md`, rounds 11–13.

### Choosing between generations

Nothing ships that a person has not checked. The pipeline renders six authored
candidates of each icon and writes `assets/icons/review.json`; the picker reads it:

```
cargo run --release --bin pick
```

For each icon it shows six candidates in a 2×3 grid at the decision size **and** at the
smallest shipped size — a choice that dies at 24 px should be seen to die while it is
being made — each on a surface the icon declares, painted from the same token table
the icon was rendered against. Two controls, and both are the point:

* a **checkbox per candidate** — check one and press Enter (or click the plate) and
  that candidate becomes the icon's **target**, the one the pipeline promotes. It is
a checkbox rather than a radio button on purpose: *none of these* has to be
reachable;
* a **comment box for the next generation** — what is wrong, what to change, what to
try. A comment is recorded whether or not anything is checked, so "none of these, make
the teeth longer" is a usable answer instead of a dead end.

Every decision is appended to `assets/icons/review-decisions.jsonl`: one line per
record, carrying the checkbox state, the comment, the measured facts of the candidate
and the build that made it — plain JSON, one record per line, nothing rewritten in
place. Deciding again **supersedes without erasing** — the last
`target: true` per icon wins, and every comment ever left is read back by
`generate.py` as a **directive** the next generation is authored against. The picker
prints them, the manifest carries them, and the pipeline prints what is still
outstanding, so a promise cannot be lost by being forgotten in a terminal scroll.

---

## Wheel map

The rule applied: **take wheels that are large and not determinism-critical; own the
pieces a replay depends on.**

| Piece | Decision | Why |
|---|---|---|
| `wgpu`, `winit` | taken | huge, and nothing about a replay depends on them |
| `serde`, `ron`, `serde_json` | taken | records and saves |
| `tracing` | taken | logs that can be read back |
| RNG | **ours** — PCG32 | `rand`'s algorithms are not stable across majors; a replay must be |
| terrain noise | **ours** — seeded value noise | same reason; also 40 lines |
| road graph + A\* | **ours** | adjacency lists over a grid rebuilt on every road change |
| `bevy_ecs` | deferred | at this size a struct-of-arrays is less code *and* faster |
| `noise`, `pathfinding`, `petgraph`, `rand`, `rayon` | deferred | either covered above or unjustified before a measurement |
| `glyphon` / `cosmic-text` / `etagere` | taken | shaping, fallback, wrapping, and dynamic runtime atlas packing |

The text stack is deliberately delegated to Glyphon; the application owns only
screen-space layout and solid geometry.

---

## The design trail

This project is designed by **grilling**: numbered rounds of questions with a
recommended answer each, answered in the user's own words, with every override and
every withdrawn claim recorded.

* `docs/GRILLING.md` — rounds 1–6, the campaign plan, plus a **verbatim transcript
  appendix** (where a paraphrase and the original disagree, the original wins).
* `docs/GRILLING-C2.md` — rounds 7–15: the presentation pass, the icon and material
  decisions, the debugger, and the publication decisions.
* `docs/GRILLING-C3.md` — rounds 15–20 (Q70–Q98): the rules that shape an icon — the
  rig, the three pillars and their roles, the seven-family palette matrix, and the
  inventory joined to real game surfaces.
* `docs/GRILLING-C4.md` — rounds 20b–24 (Q99–Q112): what makes six candidates six
  readings — the world's radiance, the corpus ladder, the per-family colour
  mechanisms, and the conformance and separation checks.
* `docs/GRILLING-C5.md` — rounds 25–27 (Q113–Q123): the picker's measured layout
  defect, the design-document audit, and the reconciliation of the six sources with
  the icon's three roles.
* `docs/GRILLING-C6.md` — rounds 28–29 (Q124–Q131): the implementation plan for the
  icon inventory, the picker, the game and the debugger, and the shared coverage
  atlas that is the real cause of the interface being unreadable.
* `UNIFIED_DESIGN.md` — the system doctrine: the six sources of design conventions
  at equal standing, aspect ownership versus role ownership (§1.0), the
  colour-mechanism rule (§3.2.1), and the binding rules for tool surfaces (§5.7).
* `ICON_STANDARD.md` — the specific half: the three roles, the material matrix with
  its ceilings and mechanisms, the corpus ladder and its digests, the judged and
  recorded checks, and the rig.
* `playtest/<stage>/RUN.md` — how to run a stage. `playtest/<stage>/RESULT.md` —
  what it actually produced, including the things that went wrong.

The quality bar is five design pillars — Impeccable, Material Design 1, early
TouchWiz, the iOS 6 era, and the LittleBigPlanet Popit — ranked by what each one
*owns*: Impeccable how things are made, MD1 the metrics, iOS 6 behaviour and clarity,
TouchWiz material language and loaded contents, Popit the contextual tool surface.
Where the document and the user disagree about what makes the game good, **the
user's word wins**, per decision, recorded in the rounds.

---

## Licence

**Code: Apache-2.0** (`LICENSE`). **Icon renders: CC BY 4.0**, author
`alacripose <221770740+alacripose@users.noreply.github.com>`. See
`ATTRIBUTION.md` for the full table, and `assets/icons/manifest.json` for the
provenance of the renders themselves.

No raster textures are shipped: materials are named solid fills and geometry. Fonts
are read from the operating system at runtime and never redistributed. Sound, when
it lands, is synthesised rather than sampled.

---

## Open, and honestly so

* **Whether it is legible to you.** Every mechanical cause of the old softness is
  gone, but that is an argument, not a reading.
* **Frame timing under load** — 26 k world quads and a depth buffer at 240 Hz output
  is unmeasured.
* **Composite contrast on a rendered frame** — token pairs are measured; pixels are
  not.
* **The icon library** — ladder, registry-driven inventory, motion, and the material
  swatch check, above.
* **The session console** — an inspectable pane beside the game (a live agent work
  ledger, playtest tickets from every session, what is being logged, build and git
  identity), reachable from the pause menu or a function key.
* **The info-view registry** — the full view list with one mode per view; a view
  with no data is openable and *says so*, naming what will fill it, and never renders
  a zero that reads as a measurement.
* **Building models** — mostly procedural, generated the same way the icons are.
  The current extrusions are a *permanent named fallback*, not a placeholder: being
  playable with zero assets is a property of the design.
* **Controls for desktop, touch, and controller** — its own round, with input already
  written as an action layer so a second and third binding set is data, not a rewrite.
