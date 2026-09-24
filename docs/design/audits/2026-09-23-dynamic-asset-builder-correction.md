# Dynamic Asset Builder correction — 2026-09-23

## Status

**Partial implementation evidence, not release or completion evidence.**

This checkpoint corrects the earlier X/Y/Z voxel-scaling prototype. The Dynamic Asset Generator remains required runtime infrastructure for target-world presentation, while semantic asset authoring lives in a separate external tool.

## Corrected boundary

- `src/asset_generator.rs` builds two explicit products:
  - runtime chunk assets from `FoundingWorld` truth, sparse occupancy, greedy visible quads, an OpenPBR-ready material manifest, and a chunk-builder cache identity;
  - one coherent settings-gear candidate mesh from named semantic recipe values plus its own material manifest.
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
- Added a versioned `ChunkInputDigest` to chunk cache identity. It hashes the resident chunk plus the residency and inward-facing boundary voxels of its six cardinal neighbours, using the existing textual `VoxelKind` identity rather than a duplicate numeric enum encoding.
- Added cache regressions proving a partial typed mass delta invalidates presentation identity even when geometry is unchanged, and neighbour load/unload invalidates then restores the correct boundary cache entry.
- Replaced per-voxel visible-face output with deterministic axis-aligned greedy quads. Same-material coplanar faces merge, while different-material faces and chunk boundaries remain distinct. Integration coverage compares every emitted unit face and material against the authoritative world for positive and negative chunks, and checks that triangle winding agrees with every declared outward normal on all six signed directions.
- Added versioned OpenPBR-ready manifests to chunk and shape assets. Every current Soil/Forage/Wood, gear-body, and gear-accent material is explicitly `opaque`, with base weight and alpha exactly `1.0`, transmission exactly `0.0`, and opaque blending. Manifest construction fails closed on contradictory optical data; renderer colours now come from the validated manifest rather than duplicate UI constants.
- Replaced 3D world-axis orb projection with three 48 px screen-space slider rows, pointer mapping, visible orb handles, and control-specific keyboard increments.
- Kept sparse signed-coordinate chunks and exact Amanatides–Woo DDA ray traversal intact.
- Kept the corrected rotated screen-space pan and target-specific zoom range.

## Verification

Environment: Windows, Rust/Cargo serial build (`CARGO_BUILD_JOBS=1`) against the normal ignored `target` directory to avoid unrelated PDB contention.

Passed:

```text
cargo test --test material_manifest --test chunk_cache --test greedy_mesh --test shape_builder --test asset_builder_controls --test voxel_raycast
  material_manifest:      3 passed
  asset_builder_controls: 4 passed
  shape_builder:          5 passed
  voxel_raycast:          3 passed
  chunk_cache:            2 passed
  greedy_mesh:            2 passed

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
- Final rebuilt `ala-cities-target` and `asset-builder` binaries were each shown for 15 seconds. Their panels read material count, opaque classification, alpha, and transmission from the generated manifest; the runtime also reports greedy quads rather than unit faces.
- `docs/design/diagrams/rendered/dynamic-asset-generator.png` — accepted 4120×2944 official Excalidraw render of the corrected runtime/authoring, manifest, and winding boundary.
- `docs/design/diagrams/rendered/world-generation-grilling-structure.png` — official Excalidraw render with the corrected Q856–Q858 amendment.

## Still open

- Full OpenPBR texture/material authoring and review. The runtime manifest and optical invariants are implemented, but promoted texture assets and human material review are not.
- Hash-aware local reference browser.
- Broader culling/LOD and measured performance budgets.
- Full generator/source/table digests and a canonical world/projection identity beyond the current seven-chunk presentation input.
- Mechanical plus human promotion and a promoted asset registry.
- Integration into the eventual replacement client rather than the bounded target binaries.
- Bronze release gates and human playtest evidence.
