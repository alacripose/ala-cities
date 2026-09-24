# Dynamic Asset Generator first measurements — 2026-09-24

## Status

**Raw release-build evidence, not a pass/fail performance gate.**

The first #28 measurement pass covers cold and warm asset generation, signed coordinates, cubic occupancy, exact raycast, conservative culling, and structural resource counts. Budgets remain open because #28 requires them to be derived from repeated measurements rather than invented before measurement.

## Reproduction

```text
cargo run --release --bin asset-generator-bench -- 20
```

Raw JSON:

- `docs/design/audits/2026-09-24-asset-generator-benchmark.json`
- schema: `ala-cities/asset-generator-benchmark/v1`
- base commit: `65cb983`
- profile: release
- samples: 20
- raycasts per raycast sample: 1,000
- working tree: explicitly dirty, therefore not release-labelled

Host used for this reading:

- AMD Ryzen 7 5700G
- 16 logical processors exposed to the process
- Microsoft Windows 11 IoT Enterprise LTSC 10.0.26100
- single-process benchmark; the generator is currently synchronous

## Readings

| Scenario | Operations/sample | p50 | p95 | Max |
|---|---:|---:|---:|---:|
| Cold positive starter chunk | 1 | 4.819 ms | 5.655 ms | 6.331 ms |
| Warm cached chunk | 1 | 0.154 ms | 0.243 ms | 0.248 ms |
| Cold negative starter chunk | 1 | 4.601 ms | 5.947 ms | 7.894 ms |
| Cold cubic preview | 1 | 26.862 ms | 32.842 ms | 32.860 ms |
| Cubic visible-triangle filter | 1 | 0.0005 ms | 0.0010 ms | 0.0016 ms |
| Exact raycast batch | 1,000 | 39.233 ms | 40.073 ms | 40.179 ms |

The raycast row is a timed batch, not a per-ray value. Dividing the p50 batch by 1,000 gives approximately `39.2 µs` per exact cubic ray on this host.

Positive and negative starter chunks showed no distinct performance penalty in this ordered sample. This is an observation, not a statistical equivalence claim.

## Resource readings

| Scenario | World entries | Octree occupied | Mesh vertices | Mesh indices | Logical mesh bytes | Octree nodes |
|---|---:|---:|---:|---:|---:|---:|
| Positive starter | 1,024 | 1,024 | 2,588 | 3,882 | 87,992 | 1,365 |
| Negative starter | 1,024 | 1,024 | 2,440 | 3,660 | 82,960 | 1,365 |
| Cubic preview | 32,768 | 32,768 | 24 | 36 | 816 | 37,449 |

The complete octree structural maximum is 37,449 nodes for 32,768 voxel leaves. The JSON records both limits separately as `octree_max_nodes` and `voxel_capacity`.

`logical_mesh_bytes` counts `MeshVertex` and `u32` index storage only. It is not process RSS and excludes octree boxes, manifests, allocator overhead, GPU buffers, and world truth.

The cubic preview retains four camera-facing triangles under the benchmark camera. It exposes all 12 outer-shell triangles before culling.

## Defect found and fixed by measurement

The first run exposed incorrect sparse-octree structural accounting. `insert_at` added one for every recursive ancestor, so a full chunk reported `163,841` nodes even though the complete depth-five tree has exactly `37,449` nodes.

The shared insertion path now counts each allocated child once. Regressions require:

- one voxel at `(0,0,0)` to allocate exactly the root plus five path nodes;
- replacing a voxel to allocate no node;
- all 32,768 voxels to produce exactly `OCTREE_MAX_NODES == 37,449` occupied nodes.

The reported resource table is from the corrected implementation.

## Limits and next evidence

This is one host and one process, not a bronze gate. It does not yet measure:

- concurrent generation or bounded worker queues;
- chunk I/O, scheduling, eviction, or cache pressure;
- replay determinism across process restarts;
- world tick impact or end-to-end frame time;
- GPU draw/upload timing;
- resident-set or GPU-memory high-water marks;
- LOD policy, which is not implemented;
- a multi-hardware percentile envelope;
- human visual/playtest acceptance.

The next pass should measure a deterministic multi-chunk schedule with bounded work per tick and record cache/eviction behavior before deriving any budget. Failed budgets must block completion rather than being relabelled as acceptable.
