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
