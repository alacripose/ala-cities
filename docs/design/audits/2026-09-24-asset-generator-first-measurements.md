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
- schema: `ala-cities/asset-generator-benchmark/v2`
- base commit: `70b50de`
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
| Cold positive starter chunk | 1 | 5.806 ms | 8.068 ms | 10.887 ms |
| Warm cached chunk | 1 | 0.182 ms | 0.304 ms | 0.375 ms |
| Cold negative starter chunk | 1 | 5.916 ms | 7.680 ms | 17.139 ms |
| Cold cubic preview | 1 | 35.734 ms | 40.288 ms | 40.355 ms |
| Cubic visible-triangle filter | 1 | 0.0006 ms | 0.0011 ms | 0.0095 ms |
| Exact raycast batch | 1,000 | 40.288 ms | 41.272 ms | 41.524 ms |
| Nine-chunk stream, one build/tick | 9 | 394.683 ms | 422.785 ms | 469.009 ms |

The raycast row is a timed batch, not a per-ray value. Dividing the p50 batch by 1,000 gives approximately `40.3 µs` per exact cubic ray on this host.

Positive and negative starter chunks showed no distinct performance penalty in this ordered sample. This is an observation, not a statistical equivalence claim.

## Resource readings

| Scenario | World entries | Octree occupied | Mesh vertices | Mesh indices | Logical mesh bytes | Octree nodes |
|---|---:|---:|---:|---:|---:|---:|
| Positive starter | 1,024 | 1,024 | 2,588 | 3,882 | 87,992 | 1,365 |
| Negative starter | 1,024 | 1,024 | 2,440 | 3,660 | 82,960 | 1,365 |
| Cubic preview | 32,768 | 32,768 | 24 | 36 | 816 | 37,449 |

The complete octree structural maximum is 37,449 nodes for 32,768 voxel leaves. The JSON records both limits separately as `octree_max_nodes` and `voxel_capacity`.

`logical_mesh_bytes` counts `MeshVertex` and `u32` index storage only. It is not process RSS and excludes octree boxes, manifests, allocator overhead, GPU buffers, and world truth.

## Bounded streaming evidence

`ChunkAssetStreamer` now provides a deterministic, synchronous staging boundary over already-resident world truth:

- request chunks in explicit signed-coordinate order;
- build no more than the configured number per tick;
- reject duplicate pending requests;
- retain a bounded number of distinct chunks;
- evict the least-recently-built chunk deterministically under cache pressure;
- remove every stale cache version when evicting one chunk;
- preserve the authoritative world digest;
- replay identical request sequences to identical asset digests.

The benchmark submits a `3×3` signed-coordinate neighborhood with a one-build-per-tick budget and three retained chunks. It completes in nine ticks, builds all nine requested chunks, evicts six under pressure, retains exactly three chunks/cache entries, and leaves the world digest unchanged.

This is bounded streaming, not concurrent streaming. It deliberately schedules work already resident in `FoundingWorld`; world generation and residency loading remain separate responsibilities.

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
- chunk I/O or world-residency loading;
- replay determinism across process restarts;
- world tick impact or end-to-end frame time;
- GPU draw/upload timing;
- resident-set or GPU-memory high-water marks;
- runtime-client integration of `ChunkAssetStreamer`;
- LOD policy, which is not implemented;
- a multi-hardware percentile envelope;
- human visual/playtest acceptance.

The next pass should integrate the bounded stream into the runtime target, measure tick/frame impact and cache transitions across camera movement, and compare at least two cold process runs before deriving any budget. Failed budgets must block completion rather than being relabelled as acceptable.
