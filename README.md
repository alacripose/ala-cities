# ala-cities

A real-time city builder in Rust on `wgpu`. Cities-Skylines-style gameplay:
road tools, zoning, demand, citizens with real commutes, services, a budget —
plus an evidence-first record of everything built, refused, and retired.

> **Status: C1 — sim core.** Campaigns are staged and each stage keeps its own
> runnable build and playtest record. See `playtest/C1/RUN.md`.

## Running

```bash
cargo run --release          # the game
cargo test                   # invariant + unit tests (headless)
cargo run --bin verify       # re-check a season record without a GPU or window
```

The repository pins its toolchain in `rust-toolchain.toml`. Everything in
`Cargo.toml` is available from the local crate cache, so a clean `cargo build`
needs no network.

## What makes this one different

Most builders let you demolish a neighbourhood and the past disappears with it.
Here, **buildings retire rather than being deleted**. Placing a road files a
ticket before the road exists; validating it means *reading the road back out
of the world*, never trusting the click that placed it. A ticket that cannot be
verified closes honestly as `completed_but_unverified` — it never gets promoted
to "validated" to make a number look better.

The population is sampled into **cases**: four hundred hungry citizens in one
district file **one** ticket reading `400`, not four hundred tickets. That is
what keeps the work queue human-scale, and it is also what keeps it drainable —
every ticket reaches a terminal state within a bounded number of sim-seconds,
at every game speed.

The screens you are looking at are not decoration over an invisible state.
The governed-session chrome — contract, ticket, governor version, identity
status — is on screen because that is the actual authority you are acting under.

## The interface

The world is **instanced quads in world space with a real depth buffer**, viewed
through a **free orbit camera** — rotation and tilt are free, the projection is
orthographic so two tiles are the same size wherever they are on screen, and
buildings are extruded one height step per level with contact shadows from the
same light that shades their faces. Clicking resolves a tile by **inverse
projection against the ground plane**, which is exact at any angle; deriving a
tile from a screen rectangle would be a tile or two wrong the moment the camera
tilts.

Where you are on the grid is answered by three things rather than one: a
**tile highlight** under the cursor in every mode, a **local grid patch** that
fades with distance while a placement tool is in hand, and a **bearing and
coordinate readout** whose north and east arms are projected from world space,
so they say where the city runs rather than where a button was put.

Type and spacing come from `src/design.rs`, whose steps are a **closed enum**:

| Step | 100 % | Used for |
|---|---|---|
| `micro` | 12 px | micro-labels beside the thing they annotate |
| `small` | 14 px | secondary rows |
| `body` | 16 px | message and content text |
| `title` | 20 px | panel headings |
| `display` | 26 px | plaques and critical identifiers |

Sizes and spacing are not passed as numbers anywhere — `draw_step` takes a
`Step`, so a call site *cannot* carry its own size, and a test reads the sources
back to catch anyone reintroducing the old ad-hoc constants. Spacing is the
design doc's 4-unit scale, targets are ≥ 48 px (WCAG 2.5.8's 24 px is the hard
floor, not the goal), and the interface scale cycles 100 / 125 / 150 / 200 %
from the pause menu, re-laying out rather than stretching.

Glyph positions are **snapped to whole pixels** and the coverage atlas is sampled
with `Nearest`, because the first build's text was soft for exactly one reason: it
was rasterised at one size and drawn at fractional positions through a linear
filter.

### The check fails closed

`design::verify` runs at startup and **refuses to start** if it finds anything:
type steps below their floors, spacing off the unit, a target under the floor, a
token with no style record, or a contrast pair below 4.5:1. It reports *every*
defect rather than the first, because a check that reports one problem per run is
a check somebody stops running.

It is also explicit about what it does **not** prove. A green run means the
requirements are present in the source. It says nothing about whether the
interface is legible to a person, nothing about layout at other window sizes,
nothing about composite contrast on a rendered frame, and nothing about frame
timing under load. Those are judgement items and they are reported as open.

## Wheel map

The rule applied: **take wheels that are large and not determinism-critical;
own the pieces a replay depends on.**

| Piece | Decision | Why |
|---|---|---|
| `wgpu`, `winit` | taken | huge, and nothing about a replay depends on them |
| `ab_glyph` | taken | glyph rasterisation; we build the atlas, not the outline maths |
| `serde`, `ron`, `serde_json` | taken | records and saves |
| `tracing` | taken | logs that can be read back |
| RNG | **ours** — PCG32 | `rand`'s algorithms are not stable across majors; a replay must be |
| terrain noise | **ours** — seeded value noise | same reason; also 40 lines |
| road graph + A\* | **ours** | adjacency lists over a grid we rebuild on every road change |
| `bevy_ecs` | deferred | at this size a struct-of-arrays is less code *and* faster |
| `cosmic-text` | deferred | needed for shaping/i18n, not for the HUD's own text |
| `noise`, `pathfinding`, `petgraph`, `rand`, `rayon` | deferred | each is either covered above or unjustified before a measurement |
| `glyphon` | **refused** | it requires `wgpu` 30; nothing here is verified against wgpu 30 |

These are deviations from the signed-off plan, each reversible, each recorded
here rather than quietly applied.

## Records

- `saves/` — local playthrough saves (git-ignored).
- `playtest/<stage>/` — tracked: how to run the stage, the interaction captures,
  and the feedback you left. Feedback is recorded at confidence `tentative`,
  because one playtest is one playtest.

## Licence

Apache-2.0. No raster textures are shipped: materials are named solid fills and
geometry. Fonts are read from the operating system at runtime and are never
redistributed. Sound, when it lands, is synthesised rather than sampled.
