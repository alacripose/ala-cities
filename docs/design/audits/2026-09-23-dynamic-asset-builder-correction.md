# Dynamic Asset Builder correction — 2026-09-23

## Status

**Partial implementation evidence, not release or completion evidence.**

This checkpoint corrects the earlier X/Y/Z voxel-scaling prototype. The Dynamic Asset Generator remains required runtime infrastructure for target-world presentation, while semantic asset authoring lives in a separate external tool.

## Corrected boundary

- `src/asset_generator.rs` builds two explicit products:
  - runtime chunk assets from `FoundingWorld` truth, sparse occupancy, visible unit-voxel faces, and a chunk-builder cache identity;
  - one coherent settings-gear candidate mesh from named semantic recipe values.
- The semantic recipe contains:
  - `Teeth` — continuous profile morph;
  - `Opening` — center-opening ratio to gear root radius;
  - `Accent` — continuous authored construction axis.
- The builder derives `BuilderHash` from the quantized recipe. Raw hash bits are not editable.
- Shape recipe changes do not alter, divide, or independently scale world chunk voxels.
- `src/bin/asset-builder.rs` is the external authoring/review process with three separate screen-space sliders and floating orb handles.
- `src/bin/ala-cities-target.rs` is the target game observation surface. It renders the real generator-produced chunk and exact octree-backed ray read-back, but exposes no authoring controls.
- The candidate remains visibly marked as requiring mechanical/human promotion before runtime use.

## Implementation

- Replaced anonymous X/Y/Z recipe fields and independent voxel-face scaling.
- Added deterministic one-mesh gear generation with a real center opening, continuous tooth-profile morph, and a continuous accent construction form.
- Added separate shape and chunk cache paths; semantic shape edits do not clear or change chunk geometry.
- Replaced 3D world-axis orb projection with three 48 px screen-space slider rows, pointer mapping, visible orb handles, and control-specific keyboard increments.
- Kept sparse signed-coordinate chunks and exact Amanatides–Woo DDA ray traversal intact.
- Kept the corrected rotated screen-space pan and target-specific zoom range.

## Verification

Environment: Windows, Rust/Cargo serial build (`CARGO_BUILD_JOBS=1`) against `target-review` to avoid unrelated PDB contention.

Passed:

```text
cargo test --test shape_builder --test asset_builder_controls --test voxel_raycast
  asset_builder_controls: 4 passed
  shape_builder:          5 passed
  voxel_raycast:          3 passed

cargo test --lib founding_day
  5 passed

cargo test --lib 'render::tests::'
  12 passed

cargo clippy --all-targets -- -D warnings
  pass
```

Full `cargo test --all-targets` result:

```text
203 passed
6 failed
1 ignored
```

The six failures are the pre-existing legacy material-table drift under B-010:

- unaccounted `shelter.*` and `kiln.*` part prices;
- stale/incomplete `src/materials/generated.rs` versus the declared Rust material tables.

No failure is in the Dynamic Asset Generator, shape builder, sparse octree, exact raycast, target client, or camera correction slice.

The typed design verifier remains intentionally red on the broader package: unresolved inherited dispositions, twelve draft capability packets, and incomplete inherited coverage. These are not silently waived by this checkpoint.

## Visual and diagram evidence

- `docs/design/audits/asset-builder/asset-builder.png` — real external builder window; three semantic sliders, one combined gear mesh, candidate hash, and promotion state are visible.
- `docs/design/diagrams/rendered/dynamic-asset-generator.png` — official Excalidraw render of the corrected runtime/authoring boundary.
- `docs/design/diagrams/rendered/world-generation-grilling-structure.png` — official Excalidraw render with the corrected Q856–Q858 amendment.

## Still open

- OpenPBR authoring and validation.
- Hash-aware local reference browser.
- Greedy meshing, broader culling/LOD, and measured performance budgets.
- Full generator/source/table and mutable-world digest cache invalidation.
- Mechanical plus human promotion and runtime asset manifests.
- Integration into the eventual replacement client rather than the bounded target binaries.
- Bronze release gates and human playtest evidence.
