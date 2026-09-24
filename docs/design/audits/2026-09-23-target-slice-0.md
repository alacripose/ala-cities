# Target founding slice 0/1 checkpoint

**Date:** 2026-09-23
**Status:** implementation started; not release evidence
**Target:** new `ala_cities::founding_day` namespace, separate from legacy C1 runtime

## Implemented

- `src/founding_day.rs`
  - signed `Coord` and `ChunkCoord` types;
  - deterministic seed/revision chunk derivation;
  - sparse 32³ chunk storage with correct global-to-local lookup in negative and non-zero chunks;
  - first-band coordinate contract;
  - authoritative typed voxel-mass deltas held outside resident chunk data;
  - chunk unload/reload that discards derived data, rederives the baseline, and reapplies deltas;
  - canonical digest independent of whether unchanged derived chunks are resident;
  - authoritative tick/clock and automatic day/night/dawn transition;
  - material ledger with source/world/carried/spent conservation;
  - land, forage, wood, fire, and shelter actions;
  - refusal/evidence records;
  - deterministic, signed-coordinate, unload/rederive, and conservation tests.
- `src/bin/founding-day-probe.rs`
  - headless target-system probe; no legacy client dependency;
  - visibly demonstrates unload, delta reapplication, stable digest, and automatic dawn.

## Evidence

```text
cargo test founding_day --lib
5 passed; 0 failed

cargo clippy --all-targets -- -D warnings
pass

cargo run --quiet --bin founding-day-probe
accepted: forage(2,2,0)
unloaded: chunk=0,0,0 digest=f97f6367d331e29b
rederived: forage_mass=1500g digest=f97f6367d331e29b
final: phase=dawn tick=43200 day=28800 night=14400 food=1500g wood=2500g balanced=true digest=c19fa5b15e29a807

cargo test
201 passed; 6 failed; 1 ignored
```

The six full-suite failures are the existing legacy material-table drift recorded
by B-010; all target-module tests pass. The shared probe capture is retained at
`docs/design/audits/target-slice-0/probe.txt`.

The probe performs actions first, then advances the world clock with no player
action. Dawn is evaluated by the target system; there is no survive button.

## Still blocked / not claimed

- graphical target client;
- subvoxel occupancy, density, porosity, and fluid-account boundaries;
- full regional generator/fields/fluids;
- governed task/authority/evidence integration;
- full death/retirement/replacement lifecycle;
- durable save/replay lineage;
- asset builder/mesher/cache;
- performance/accessibility/human release gate.

This checkpoint proves the target seam and headless slice only. It does not
validate the legacy C1 client or claim a human playtest.
