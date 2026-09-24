# Dynamic Asset Builder correction — 2026-09-23

## Status

**Partial implementation evidence, not release or completion evidence.**

This checkpoint corrects the earlier X/Y/Z voxel-scaling prototype. The Dynamic Asset Generator remains required runtime infrastructure for target-world presentation, while semantic asset authoring lives in a separate external tool.

## Corrected boundary

- `src/asset_generator.rs` builds two explicit products:
  - runtime chunk assets from `FoundingWorld` truth, sparse occupancy, greedy-masked indexed triangle meshes, an OpenPBR-ready material manifest, and a chunk-builder cache identity;
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
- Replaced per-voxel visible-face output with deterministic axis-aligned greedy masks emitted as indexed triangles. Same-material coplanar faces merge, while different-material faces and chunk boundaries remain distinct. Integration coverage compares every emitted unit face and material against the authoritative world for positive and negative chunks, and checks that triangle winding agrees with every declared outward normal on all six signed directions.
- Added versioned OpenPBR-ready manifests to chunk and shape assets. Every current Soil/Forage/Wood, gear-body, and gear-accent material is explicitly `opaque`, with base weight and alpha exactly `1.0`, transmission exactly `0.0`, and opaque blending. Manifest construction fails closed on contradictory optical data; renderer colours now come from the validated manifest rather than duplicate UI constants.
- Added conservative per-frame world-triangle culling. The camera matrix is captured once per frame; backfaces, triangles wholly outside the orthographic clip volume, and off-camera chunks are skipped, while a triangle crossing a viewport edge is retained. The target panel reports submitted/total triangles without mutating the asset or world.
- Fixed a native-camera ray false positive at the sparse-octree boundary. `SparseVoxelOctree::get` now rejects coordinates outside its local `0..32` domain instead of allowing fixed-bit traversal paths to alias far-away coordinates into valid branches. A target-camera regression now requires `ChunkAsset::raycast` to equal authoritative `FoundingWorld` raycasting exactly.
- Added a named `FoundingWorld::cubic_preview` fixture containing one full 32³ section. It proves cubic occupancy, indexed meshing, and culling without claiming that the canonical field-coupled generator is a solid cube. The next target preview uses that fixture and the licensed architectural reference record.
- Added deterministic `ChunkAssetStreamer` staging over already-resident signed world chunks. It enforces a per-tick build budget, rejects duplicate pending requests, preserves world truth, and evicts least-recently-built chunks under an explicit distinct-chunk capacity. Cache eviction removes every stale version for the selected chunk.
- Added a release benchmark covering cold/warm generation, negative coordinates, cubic resources, culling, exact raycast, and a nine-chunk/one-build-per-tick stream. Raw v2 JSON and interpretation live in the 2026-09-24 measurement audit.
- Replaced 3D world-axis orb projection with three 48 px screen-space slider rows, pointer mapping, visible orb handles, and control-specific keyboard increments.
- Kept sparse signed-coordinate chunks and exact Amanatides–Woo DDA ray traversal intact.
- Kept the corrected rotated screen-space pan and target-specific zoom range.

## Verification

Environment: Windows, Rust/Cargo serial build (`CARGO_BUILD_JOBS=1`) against the normal ignored `target` directory to avoid unrelated PDB contention.

Passed:

```text
cargo test --test cubic_chunk_preview --test view_culling --test voxel_raycast --test greedy_mesh --test material_manifest --test chunk_cache --test shape_builder --test asset_builder_controls
  cubic_chunk_preview:    1 passed
  view_culling:           2 passed
  voxel_raycast:          4 passed
  material_manifest:      3 passed
  asset_builder_controls: 4 passed
  shape_builder:          5 passed
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
- `docs/design/audits/target-cubic-chunk-preview.png` — accepted native cubic preview: complete visible 32³ hull, `6/12` submitted triangles, and valid camera ray `(25,12,31)`.
- `docs/design/audits/asset-builder-indexed-triangles.png` — accepted final builder frame: one mesh, `1932` triangles, `5796` vertices, three semantic controls, and the real indexed-triangle path.
- `docs/design/audits/target-indexed-triangle-default.png` — accepted indexed runtime capture with a valid native ray read-back and no visible interior pinholes.
- `docs/design/audits/2026-09-24-asset-generator-benchmark.json` — raw release benchmark schema v2, including bounded nine-chunk streaming and cache-eviction readings.
- `docs/design/audits/2026-09-24-asset-generator-first-measurements.md` — measured interpretation, corrected octree structural evidence, and explicit remaining benchmark limits.
- Final rebuilt `ala-cities-target` and `asset-builder` binaries were each shown for at least 15 seconds. Their panels read material classification and optical values from the generated manifest; the runtime reports camera-culled triangles rather than unit faces.
- `docs/design/diagrams/rendered/dynamic-asset-generator.png` — accepted 4120×2944 official Excalidraw render of the corrected runtime/authoring, triangle, manifest, and culling boundary.
- `docs/design/diagrams/rendered/world-generation-grilling-structure.png` — official Excalidraw render with the corrected Q856–Q858 amendment.

## Still open

- Full OpenPBR texture/material authoring and review. The runtime manifest and optical invariants are implemented, but promoted texture assets and human material review are not.
- Hash-aware local reference browser.
- Runtime integration and concurrent I/O/generation workers for the bounded synchronous streamer; LOD policy; and derived build/frame budgets.
- Full generator/source/table digests and a canonical world/projection identity beyond the current seven-chunk presentation input.
- Mechanical plus human promotion and a promoted asset registry.
- Integration into the eventual replacement client rather than the bounded target binaries.
- Bronze release gates and human playtest evidence.
