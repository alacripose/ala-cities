# World-generation grilling map

**Status:** subordinate, reviewable synthesis; not product authority
**Scope:** C11 world-generator decisions WG-Q1–WG-Q91 / C11-Q662–Q753, the later target corrections C11-Q775–Q776, Q816–Q820, and Q840–Q858, the current draft capability packets, Wayfinder tickets observed during this synthesis, and the inspected target Rust modules.
**Authority rule:** `DESIGN.md` remains the concise constitution, `docs/GRILLING-C11.md` remains the reasoning/answer record, the Wayfinder map remains the live route, and code/tests remain implementation evidence. This file only makes their consequences easier to implement and review. It does not resolve a disputed record, close a ticket, or turn a prototype reading into a gate.

## 1. Executive implementation thesis

The world-generation answer is a dependency chain, not a list of effects:

1. **CAP-001 fixes identity and authority first.** A named, versioned, digest-pinned generator derives world truth from seed and signed coordinates; it does not apply gameplay changes (C11-Q662, Q665, Q716, Q723, Q737). Baseline truth is derived; only typed deltas mutate it (Q667, Q695, Q727, Q752).
2. **Coordinates and coupled fields define one substrate.** Global/regional constraints precede chunk-local derivation; boundaries are shared; geology, terrain, hydrology, climate/biology, soil/materials, caves, fluids, atmosphere, and eventually ruins are one coherent 3D system (Q663–Q666, Q688–Q714).
3. **Dynamic processes are authoritative, not decorative.** Tectonics, erosion, caves, weather, fluids, biology, and gameplay projections have declared deterministic clocks, dependency order, conservation, and cross-scale behavior from tick zero (Q682, Q684, Q691, Q697, Q706, Q724, Q739–Q740, Q746).
4. **Persistence and presentation are downstream.** CAP-007 transports and compares a CAP-001 canonical projection; CAP-010/CAP-008 consume immutable truth but cannot write it back (Q699, Q723, Q742, Q747, Q751, Q775–Q776, Q856–Q858).
5. **The first playable slice is now concretely scoped.** The first target uses 1 m voxels, 32³ chunks, signed i32 coordinates, a first vertical band of `z=-16..+47`, a CAP-001-owned clock, and one autonomous citizen observed by the player (Q840, Q846–Q853). This is a first-slice contract, not completion of the long-term Earth-inspired generator.
6. **The gate remains open.** The required proof spans deterministic projection/replay, signed and cross-boundary coordinates, conservation, revision refusal, migration, measured performance, asset provenance, accessibility, and the first-night human gate (Q683, Q699–Q700, Q708, Q730, Q734, Q775–Q776, Q806, Q824–Q825). No inspected record or source closes that gate.

## 2. How to read status

| Label | Meaning in this synthesis |
|---|---|
| **Settled** | The answer or boundary is canonically decided. Implementation may still be absent. |
| **Scoped / deferred** | The direction and scope are decided, but an algorithm, schema, rate, field resolution, boundary, or fixture remains explicitly open. |
| **Open / blocker** | Work may not cross the affected local gate until the named evidence exists. |
| **Partial source evidence** | Inspected code demonstrates a bounded slice, not the whole decision. |
| **Not evidenced** | The inspected target modules do not prove the decision. This is not proof that no other code exists. |

## 3. Live issue register used for ownership

All tickets below were observed **open** during this synthesis. Their names, not bare numbers, are the route references used in the rest of this document.

| Owning ticket | Scope used here |
|---|---|
| [world generator pass — substrate, terrain, and seed determinism](https://github.com/alacripose/ala-cities/issues/21) | Parent decision/implementation-through-gate ticket; C11-Q736 keeps it open through implementation and required playtest. |
| [Dynamic asset builder and chunk mesher boundary](https://github.com/alacripose/ala-cities/issues/22) | CAP-010 presentation boundary, chunk assets, OpenPBR-ready output, meshing/culling, and cache identity. |
| [3D chunk and subvoxel substrate contract](https://github.com/alacripose/ala-cities/issues/23) | Signed chunks, sparse authoritative storage, baseline/deltas, unload/rederive, subvoxel and ownership boundaries. |
| [Founding material chain and seeded ruin baseline](https://github.com/alacripose/ala-cities/issues/24) | Forage/fire/shelter/direct-water material chain and later seeded ruin baseline/provenance. |
| [Earth quantitative source table admission](https://github.com/alacripose/ala-cities/issues/25) | Cited source rows, units, ranges, scaling, provisional admission, and human review before release. |
| [New save lineage and semantic replay migration](https://github.com/alacripose/ala-cities/issues/26) | Lineage-first save/run envelope, explicit v3 refusal, replay journal, and proven migration only. |
| [CAP-001 canonical world truth and replay projection](https://github.com/alacripose/ala-cities/issues/27) | Production canonical projection fields, exact/tolerant quantities, tick/epoch fixtures, and replay separation. |
| [Generator performance and first-gate evidence](https://github.com/alacripose/ala-cities/issues/28) | Commands, raw readings, budgets, conservation/replay evidence, and human first-night evidence. |
| [Coupled climate hydrology and conserved fluid prototype](https://github.com/alacripose/ala-cities/issues/29) | Smallest climate/hydrology/atmosphere/typed-fluid process proving closed accounts and tick-zero determinism. |
| [capability packet completion and evidence matrix](https://github.com/alacripose/ala-cities/issues/30) | Honest packet completion across CAP-001–CAP-012; all inspected packets remain draft/partial. |

## 4. Later corrections and retired mechanisms

### 4.1 Corrections that change implementation consequences

| Later decision | What it changes from the WG pass | Concrete implementation consequence |
|---|---|---|
| **Q775 — replay boundary** | Makes Q699 executable: equivalence is exact canonical integer/rational state at declared simulation epochs/ticks; presentation is excluded. | Replay must compare a named production projection at declared boundaries. A `u64` digest, session capture, or mesh cache is not by itself that projection. Owner: [CAP-001 canonical world truth and replay projection](https://github.com/alacripose/ala-cities/issues/27). |
| **Q776 — v3 lineage** | Narrows Q707/Q722: current Rust v3 remains historical evidence but is unsupported for the new authoritative lineage. | The new loader validates lineage before interpreting fields and refuses v3 unless a separately proven migration exists. No silent regeneration. Owner: [New save lineage and semantic replay migration](https://github.com/alacripose/ala-cities/issues/26). |
| **Q816 — seamless transition state** | Resolves the persistent-identity half of Q814: the same dedicated citizen, possessions, needs, history, and task context survive generic-spawn transition. | Mode transition cannot construct a replacement citizen or discard task/save context. Owner: packet completion plus the first-target client slice. |
| **Q817–Q818 — contextual effects** | Physical verbs require an immediate, subject-appropriate, inspectable consequence; planned builds may preview before incremental materialization. | Picking/UI feedback cannot be the only effect, and a tree/terrain/material change must appear in canonical state. The exact effect taxonomy remains open. |
| **Q819 — preview reservation** | A transparent build preview is authoritative reservation state, not merely decoration. | Preview creation must reserve declared material/labour with ownership, release, cancellation, and compensation semantics still to be specified. |
| **Q820 — unauthorized open effect** | An unauthorized physical effect is not silently rolled back. | Preserve the real conserved effect/reservation, then append attributable adjudication/corrective consequences. This sharpens Q718/Q728 and CAP-002 conservation. |
| **Q840–Q841** | CAP-001 owns the clock; the first founding path must use real material/process/conservation. | Day/night cannot be a client animation or a fake inventory counter. Current hard-coded action quantities are not the required data recipe. |
| **Q842 and Q844** | Click inspects; use invokes the selected governed verb; the first camera uses exact signed 3D picking. | The ray hit is a typed read-back, not a mutation command. Missing/blocked selections must stay explicit. |
| **Q843** | The first client captures evidence; durable save follows. | An in-memory snapshot/evidence vector is allowed only as prototype evidence, not a durable authoritative save. |
| **Q845** | Every action, refusal, phase, material delta, and task outcome emits typed evidence/read-back. | Stringly result logs are insufficient for the full first-night contract; each event needs stable typed identity and canonical read-back. |
| **Q846** | Freezes the first target geometry: 1 logical metre per voxel, 32³ chunks, signed i32 coordinates, first playable band `z=-16..+47`, sea level `z=0`, `x` east/`y` north/`z` up. | The first implementation may use these constants, but it must still implement band refusal/seams and may not treat the first band as the long-term core boundary. This supersedes the first-target portion of Q838 and the “dimensions open” packet wording. |
| **Q847–Q848 and Q852–Q853** | Amend Q790: one autonomous citizen acts from needs/utility and may open unclaimed work; the human observes the first night and need not act. | The first-night gate must prove autonomous governed agency, not player button completion. The current `Action` API is not sufficient evidence. |
| **Q849** | Fixes first-slice day/night durations at 24/12 real minutes at 1×. | Tick mapping and speed semantics remain measured implementation details; the durations are parameters of the world clock, not UI animation. |
| **Q850–Q855** | Data recipes, complete lifecycle, death ending the attempt, and explicit append-only estate transfer are first-slice scope. | Replace hard-coded recipe constants with one named data source; implement citizen death/retirement/replacement and transfer evidence before the first-night gate. |
| **Q856–Q858** | Dynamic Asset Generator is required runtime infrastructure; named semantic recipe values derive the builder hash; three orb-handle sliders live only in the external asset-builder tool. | The real path is world snapshot → runtime chunk asset → game, with a separate authoring candidate path → mechanical/human review → promoted presentation asset. Raw hash editing, in-game authoring controls, detached mock implementations, legacy shortcuts, and presentation write-back are forbidden. |

### 4.2 Mechanisms retired or demoted

| Retired/demoted mechanism | Replacement direction | Canonical basis |
|---|---|---|
| Flat `World`/`Tile` as canonical world truth | Sparse 3D target substrate with foundational 2D logic only where sufficient | Q663, Q669, Q729, Q744 |
| Hard-coded cosmetic edge road as bootstrap | Typed cut/fill/borrow/spoil and path deltas with material lineage | Q696, Q728 |
| 8 m treated as the voxel/octree dimension | 8 m remains a logical/material tile; first target voxels are 1 m under one conversion contract | Q676, Q726, Q846 |
| Caches or generated chunks as save/authority | Disposable revision-keyed derivations; persisted typed deltas are authoritative | Q667, Q695, Q727, Q742, Q752 |
| Current v3 save as authoritative target lineage | Historical/readable evidence plus explicit refusal; new lineage or proven migration | Q707, Q722, Q775–Q776 |
| Direct player/editor mutation or command-body bypass | Governed proposal/task/material/evidence path; embodied physical effects remain attributable | Q690, Q692, Q721, Q817–Q820, Q848 |
| Player-completed loop as the first-night prerequisite | Autonomous citizen loop observed by the human player | Q790 amended by Q839, Q847–Q848, Q852–Q853 |
| Optional/detached asset builder, legacy renderer shortcut, independent zero-asset fallback | Required real Dynamic Asset Generator consuming world truth | Q744, Q747, Q751, Q832, Q856–Q858 |
| Biomes as arbitrary hard labels or independent noise | Authoritative quantitative fields with derived labels and declared transitions | Q666, Q672, Q687, Q735, Q741 |
| Water-only special cases | Conserved typed fluid roster with shared pressure/flow/phase semantics | Q671, Q679, Q685, Q698, Q703, Q713, Q740 |
| Activating target dynamics only when their implementation lands | Correct tick-zero state for every accepted world; later stages preserve/reconstruct it | Q682, Q684, Q724, Q746 |

## 5. Dependency-cluster decisions

### 5.1 Authority and determinism

**Exact decisions.** The generator is a pure staged derivation and never applies gameplay mutation (Q662). Chunk truth is a function of seed, revision, and signed coordinates (Q665). Baseline fields are derived while explicit deltas are stored (Q667). Identity is a named ruleset/version plus source/table digests (Q716). CAP-001 owns world facts, generator semantics, clock, and canonical projection; CAP-002 owns matter; CAP-007 owns persistence; CAP-008 consumes spatial truth (Q723). Pure typed Rust functions/modules are required, with no hidden mutable generator state (Q737). Prototype work may tune mechanics but not semantic truth, conservation, or authority (Q753).

**What this forbids or requires.** Forbid a gameplay callback inside generation, a mutable singleton/noise accumulator, an unversioned table, a cache used as truth, or a prototype assertion that chooses semantics. Require a typed identity containing ruleset/revision/source/table digests, pure stage interfaces, explicit refusal on mismatch, and a canonical projection at Q775 tick/epoch boundaries.

**Code and architecture meaning.** Introduce a generator identity distinct from `WorldSeed`; make each stage consume immutable inputs and return typed facts/deltas; expose projection independently of presentation; pass identity/digests into cache keys and save validation.

**Current implementation state.** `src/founding_day.rs:83-215` has `WorldSeed`, a `u64` `GeneratorRevision`, a pure coordinate hash, and `Chunk::derive`; `src/founding_day.rs:749-823` has a candidate state digest. This is useful target-seam evidence, but no named stages, source/table digests, unsupported-revision refusal, production projection, or full field model is evidenced. The Q775 Python probe is explicitly fixture-only (`prototypes/cap001-projection/README.md:26-40`).

**Remaining questions, owner, and exact next evidence.** Production projection fields/tolerances are owned by [CAP-001 canonical world truth and replay projection](https://github.com/alacripose/ala-cities/issues/27); generator identity and conservation/schema closure also involve [3D chunk and subvoxel substrate contract](https://github.com/alacripose/ala-cities/issues/23) and [capability packet completion and evidence matrix](https://github.com/alacripose/ala-cities/issues/30). Next evidence: a typed identity round-trip, revision/rule/source-digest mismatch refusal, golden cross-run projection equality, and a changed-authoritative-state mismatch fixture at the same tick.

### 5.2 Signed coordinates, chunks, and octrees

**Exact decisions.** The target is 3D and chunked with a bounded initial area; 2D logic is allowed only where sufficient (Q663, Q669). Chunks are signed and seamless across boundaries (Q688), and sparse voxel/octree techniques are required. An 8 m logical tile contains finer voxels under one conversion contract (Q676, Q726). Geometry is measured power-of-two (Q694), later frozen for the first target as 1 m voxels, 32³ chunks, signed i32, and first band `z=-16..+47` (Q846). Core is a terminal supported depth; longer-term generation must refuse beyond it (Q731).

**What this forbids or requires.** Forbid unsigned casts, chunk-local noise, a second 8 m-to-voxel map, arbitrary old dimensions, or unbounded generation past a declared support boundary. Require shared global-coordinate sampling, floor-division/local-coordinate conversion, first-band checks at the proper boundary, and explicit long-term vertical support/refusal behavior.

**Code and architecture meaning.** Keep authoritative sparse chunk state separate from presentation acceleration. A global coordinate/hash is sampled before local storage; conversion owns geometry, mass, simulation, and rendering coordinates. The long-term vertical-domain policy must wrap the first-slice band rather than be replaced by it.

**Current implementation state.** `src/founding_day.rs:11-87` implements signed `i32` coordinates, Euclidean chunk division, 32³ chunks, and the first vertical band. `src/founding_day.rs:862-877` tests a negative/non-zero chunk. `src/octree.rs:1-177` is a sparse 32³ presentation octree. The first-band predicate is used only by landing (`:530-545`), while `load_chunk` accepts arbitrary i32 chunk coordinates (`:476-486`); no core-domain refusal, explicit horizontal bound, 8 m conversion, or cross-boundary equality fixture is evidenced.

**Remaining questions, owner, and exact next evidence.** Owner: [3D chunk and subvoxel substrate contract](https://github.com/alacripose/ala-cities/issues/23). Next evidence: adjacent-chunk samples on both sides of `x/y/z` seams; negative and `i32` boundary fixtures; first-band refusal; one conversion fixture proving geometry/mass/simulation/render coordinates agree; and a declared terminal-core refusal test.

### 5.3 Regional fields, geology, and material baseline

**Exact decisions.** The full target includes bedrock/geology, soil/surface, elevation, water, climate/biology, material deposits, and ruins (Q664). Fields are coupled, not independent noise (Q666). The complete declared material library is required, including geology-permitted clay (Q673). Geology is Earth-inspired rather than a literal Earth copy (Q710), and initial state is the declared result of deep-time prehistory (Q725). Untouched stock is a derived baseline reconciled with every delta (Q695, Q733). Earth values require source/range/unit/scaling/tolerance records (Q720, Q738, Q745); agents may admit cited values provisionally, but release admission requires review (Q750, clarified by Q804).

**What this forbids or requires.** Forbid free-floating constants, hard-coded starter stock as world authority, material labels without mass/source lineage, a partial generated table treated as complete, and provisional Earth values presented as release-approved. Require global/regional fields first, then chunk sampling; one typed, sourced, versioned table home; and baseline/source/delta read-back.

**Code and architecture meaning.** Model field values and units explicitly, keep regional fields authoritative, and derive biome/material labels where appropriate. Derive starter resources from the selected golden seed/site or a declared fixture, never from unconditional coordinates in production code. Keep material accounting in CAP-002 even when the spatial location comes from CAP-001.

**Current implementation state.** `src/founding_day.rs:89-215` contains only `Soil`, `Forage`, and `Wood`; starter coordinates at `(2,2,0)`, `(6,2,0)`, and `(2,6,0)` are unconditional, while soil below zero and random material at `z=0` do not constitute coupled geology or terrain. `docs/research/earth-source-table.md:1-23,82-96` is a cited provisional research artifact, explicitly not generator data or release approval. No regional fields, deposit model, ruin baseline, or prehistory derivation is evidenced.

**Remaining questions, owner, and exact next evidence.** Owners: [Founding material chain and seeded ruin baseline](https://github.com/alacripose/ala-cities/issues/24), [Earth quantitative source table admission](https://github.com/alacripose/ala-cities/issues/25), and [Coupled climate hydrology and conserved fluid prototype](https://github.com/alacripose/ala-cities/issues/29). Next evidence: a versioned table schema with digest; a golden seed/site whose forage, timber, water, and geology arise from coupled rules; baseline mass reconciliation before/after unload; and a separate ruin baseline with poor yield/high processing cost.

### 5.4 Terrain, elevation, seams, and supported extent

**Exact decisions.** Global/regional constraints precede chunk-local detail (Q689). Sparse local truth and regional/global fields have separate roles (Q702). The initial play/gate region is landing-centered, while outside chunks may stream (Q709), but the exact initial horizontal boundary remains deferred (Q693). The long-term target is Earth-scaled and technically open-ended through declared layers, with core expeditions as real progression, while the core is the terminal supported depth (Q701, Q731). The first implementation's vertical band is the Q846 subset, not the long-term answer.

**What this forbids or requires.** Forbid chunk-local noise contradicting a regional field, an undeclared “infinite” world, a first-band constant silently becoming the core boundary, or a fixed arbitrary initial rectangle. Require a measurable landing-centered region policy, declared elevation/terrain coupling, shared seam sampling, and typed refusal beyond supported extent.

**Code and architecture meaning.** Separate `FirstPlayableRegion` from `SupportedWorldDomain`; sample regional elevation/geology first; derive local terrain/materials second; attach initial gate scoring/play scope to a declared landing-centered region while allowing independent streaming.

**Current implementation state.** The target module has no elevation, terrain height, slope, seam field, regional sampling, or long-term extent type. Chunk derivation can return soil for every `z<0` and air for every `z>0`, independent of chunk or seed; only landing checks the first band. Therefore Q689, Q693, Q701, Q709, Q731, and most of Q846 are not evidenced beyond coordinate types/constants.

**Remaining questions, owner, and exact next evidence.** Owners: [3D chunk and subvoxel substrate contract](https://github.com/alacripose/ala-cities/issues/23) and [world generator pass — substrate, terrain, and seed determinism](https://github.com/alacripose/ala-cities/issues/21). Next evidence: a declared region predicate; a terrain profile sampled identically across chunk boundaries; first-band and out-of-band fixtures; and an explicit unsupported-core refusal carrying coordinate/reason evidence.

### 5.5 Hydrology, fluids, climate, weather, ecology, and atmosphere

**Exact decisions.** Fluids are generalized, conserved, typed world actors with pressure, flow, groundwater, rain, phase, and declared carriers/processes (Q671, Q679, Q685). Initial water/ice/steam/oil/lava/gas state is rule-derived, not random (Q698). The water cycle closes across ocean, groundwater, atmosphere, ice, steam, and other accounts (Q703). Climate is quantitative, seasonal, Earth-like; biomes are derived views over temperature/moisture/elevation/water/soil with transition bands (Q672, Q680, Q687). Atmospheric composition/weather is authoritative when it affects simulation; decorative sky/stars are downstream (Q678, Q686). Initial season/weather is seed/revision-derived (Q712). Hydrology is a coupled conserved network (Q713), and regional numeric climate/biology facts—not biome labels—are authoritative (Q735, Q741).

**What this forbids or requires.** Forbid water-only code, untracked reservoirs, random fluid placement, hard biome labels, decorative weather driving simulation, and a climate model that creates/deletes water or matter. Require typed fluid accounts, source/sink/phase conservation, regional fields with units/tolerances, and presentation derived from authoritative atmosphere where relevant.

**Code and architecture meaning.** Define fluid substance/phase/pressure/quantity accounts in CAP-002 and spatial/process state in CAP-001. Couple hydrology to elevation/geology/climate. Treat biome names and visual vegetation as queries/projections. Keep sky/stars presentation-only unless they affect simulation.

**Current implementation state.** No typed fluid, water account, hydrology, climate field, biome, weather, ecology, or atmosphere model is present in the inspected target modules. `Phase::{Day,Night,Dawn}` and the 24/12 clock are a first-slice day/night mechanism (Q849), not the Q703–Q704 climate/water system.

**Remaining questions, owner, and exact next evidence.** Owner: [Coupled climate hydrology and conserved fluid prototype](https://github.com/alacripose/ala-cities/issues/29), with source admission from [Earth quantitative source table admission](https://github.com/alacripose/ala-cities/issues/25). Next evidence: a tiny closed rainfall/groundwater/river/evaporation or condensation fixture; typed phase transition; regional climate fields with seed-derived initial state; cross-chunk conservation; and changed-authoritative-state replay at the same tick.

### 5.6 Caves, erosion, tectonics, and dynamic clocks

**Exact decisions.** Tectonics, erosion, cave change, weather, and fluids have explicit deterministic cadences and simulation classes (Q684). Active chunks receive exact updates; inactive regions use declared lazy/aggregate updates with conservation/replay checks (Q691). Caves have a deterministic baseline and dynamic formation/collapse/erosion from tick zero (Q697), with connected expedition-scale systems toward the supported core (Q714). Dynamic order is declared per cadence (Q706). All target dynamics are conceptually active from tick zero; a later implementation stage must reproduce the correct tick-zero state for accepted worlds (Q724). Tectonics operates at plate/region scale, erosion at chunk/voxel scale, and cave change locally with declared cross-scale effects (Q739). Q746 places fluids, weather, erosion/caves, and climate in the first complete generator gate while keeping ruins later.

**What this forbids or requires.** Forbid baked-only tectonics/erosion/weather, subsystem activation at implementation landing, nondeterministic worker timing, lazy updates that lose mass/state, or a first gate that omits Q746 core dynamics. Require per-process cadence, dependency order, cross-scale state, conservation, and replay.

**Code and architecture meaning.** Model each process as a deterministic tick/cadence scheduler with exact and aggregate tiers. Persist enough state to reconstruct tick zero under a later stage or explicitly refuse/reseed incompatible worlds; never silently “turn on” history.

**Current implementation state.** The inspected code advances only founding day/night (`src/founding_day.rs:686-747`). It has no tectonics, erosion, cave, weather, geological-time, LOD/tiering, or process scheduler. The implementation plan defers generator breadth/dynamics to Slice 6 while Q746 includes core dynamics in the first complete gate; this can be an early prototype order only if those worlds are not accepted as satisfying #21 and Q724 state is not silently changed.

**Remaining questions, owner, and exact next evidence.** Owners: [Coupled climate hydrology and conserved fluid prototype](https://github.com/alacripose/ala-cities/issues/29) and [Generator performance and first-gate evidence](https://github.com/alacripose/ala-cities/issues/28). Next evidence: one small process per scale with declared cadence/order; a lazy/exact cross-tier conservation fixture; a save/rederive fixture showing tick-zero state before and after adding the subsystem; and a Q746 gate inventory showing which dynamics are present versus still red.

### 5.7 Deltas, mutation authority, and conservation

**Exact decisions.** Generated baselines are immutable derivations; extraction, roads, terrain edits, structures, and fluid changes are typed deltas (Q727). Untouched baseline stock reconciles with extraction, harvest, processing, loss, and player changes (Q695, Q733). Earthworks carry cut/fill/borrow/spoil lineage and can affect later dynamics (Q696, Q728). The initial founder-only landing exception is recorded and creates no matter (Q690, Q692). Physical interaction is mediated by the normal task/authority/conservation stack (Q721). Q817–Q820 refine effects: visible contextual consequence, reservation on preview, and preserved real effect with later adjudication rather than silent rollback.

**What this forbids or requires.** Forbid mutating derived baseline in place, losing deltas on unload, cosmetic roads without lineage, free founder placement that creates stock, or rollback that destroys a conserved physical effect. Require typed delta identity, source/target accounts, applicability to the supported coordinate domain, persistence across unload, and evidence of authority/reservation/adjudication.

**Code and architecture meaning.** Keep a delta log/store separate from resident chunks and caches. Model earthworks and effects as domain transitions, not mesh changes. Reservations must be authoritative and distinguishable from completed structure; unauthorized effects remain real with appended consequences.

**Current implementation state.** `src/founding_day.rs:118-175,660-684` persists one delta kind, `RemoveVoxelMass`; unload/rederive reapplies it and the five target-module tests include the digest fixture (`:880-899`). The ledger closes source/world/carried/spent totals (`:217-267`). There are no cut/fill/borrow/spoil, fluid, terrain, road, structure, reservation, task, consent, ownership, or adjudication deltas. `FoundingWorld::act` applies actions directly (`:500-638`), so it does not prove Q817–Q820’s governed path.

**Remaining questions, owner, and exact next evidence.** Owner: [Founding material chain and seeded ruin baseline](https://github.com/alacripose/ala-cities/issues/24), with CAP-003/005/006 packet closure under [capability packet completion and evidence matrix](https://github.com/alacripose/ala-cities/issues/30). Next evidence: a typed multi-delta fixture covering extraction, cut/fill, fluid transfer, and reservation; conservation after unload/reload; an unauthorized-effect fixture proving effect preservation plus appended consequence; and a refusal/rollback boundary table.

### 5.8 Streaming, workers, culling, and performance

**Exact decisions.** Worker jobs may run concurrently only with fixed dependency, RNG, revision, cross-chunk sampling, and merge order (Q718). Dynamic LOD is deterministic and conservation-safe (Q691). Performance targets are measured per stage—generation, streaming, memory, tick, and draw—and re-gated when target changes (Q700). Presentation culling is downstream (Q747). Q753 permits measured worker/cache/rate choices but not semantic changes.

**What this forbids or requires.** Forbid speed-dependent results, worker-count-dependent output, random merge order, or invented budgets. Require a fixed job graph, deterministic merge, resident/unloaded state accounting, raw timing/memory/draw readings, and gate budgets derived from target measurements.

**Code and architecture meaning.** Worker result identity must use the same generator identity and signed coordinates as single-thread derivation. Culling changes visibility only. Performance evidence must cover the Q746 first gate, not only a tiny probe.

**Current implementation state.** `FoundingWorld` has manual `load_chunk`/`unload_chunk` and bounded B-tree residency, but no worker scheduler, streaming policy, dynamic LOD, culling policy, or performance capture. No inspected source establishes a performance budget or #28 command set.

**Remaining questions, owner, and exact next evidence.** Owner: [Generator performance and first-gate evidence](https://github.com/alacripose/ala-cities/issues/28), with builder measurements shared with [Dynamic asset builder and chunk mesher boundary](https://github.com/alacripose/ala-cities/issues/22). Next evidence: 1-worker and N-worker fixture equality, fixed merge trace, unloaded-chunk conservation, generation/streaming/memory/tick/draw raw readings on named hardware/build identity, and a target-derived budget decision.

### 5.9 Presentation boundary, assets, meshing, and cache

**Exact decisions.** World truth and mesher/render adapters are separate (Q747). The Dynamic Asset Generator owns visible chunk meshes, OpenPBR materials/assets, greedy meshing, and sparse voxel/octree optimization; it consumes generator truth and cannot mutate it (Q744). Generated caches are disposable and identity-keyed (Q742); mesh identity includes generator ruleset/digest, builder hash, and chunk coordinates (Q751). Q856 makes the generator required target infrastructure while keeping its authoring/review UI in a separate tool. Q857 makes the candidate hash a deterministic derivative of named semantic recipe values: the gear-first proof uses Teeth/profile morph, Opening/root-radius ratio, and Accent/construction, never X/Y/Z voxel scaling or raw hash-bit editing. Q858 places three separate orb-handle sliders in the external asset-builder tool; the target game exposes no authoring controls.

**What this forbids or requires.** Forbid meshes/materials/labels as alternate world/material truth, a detached mock/legacy renderer path, an independent zero-asset fallback, raw hash editing, presentation write-back, or in-game asset-authoring controls. Require an immutable world snapshot, deterministic recipe/hash, provenance/manifests, cache rebuild, and Q805 mechanical plus human promotion before runtime promotion.

**Code and architecture meaning.** Key runtime chunk presentation by full generator identity, chunk-mesh recipe/hash, chunk coordinate, and an authoritative world/projection digest sufficient to invalidate changed deltas. Keep semantic shape authoring, OpenPBR mapping, sparse acceleration, greedy meshing, manifests, review, and culling behind CAP-010. The external tool may rebuild shape candidates freely, but it cannot write CAP-001 facts or expose controls in CAP-008.

**Current implementation state.** `src/asset_generator.rs` now has two explicit outputs: `build_chunk` reads `&FoundingWorld`, builds a sparse octree plus unit-voxel visible faces, and caches by seed/revision/chunk/chunk-builder recipe plus a versioned `ChunkInputDigest`; that digest covers the resident chunk and the residency/inward-facing boundary voxels of its six cardinal neighbours, including typed voxel mass and the existing textual voxel-kind identity. `build_shape` derives a candidate `BuilderHash` from quantized Teeth/Opening/Accent values and builds one coherent gear mesh. `src/asset_controls.rs` implements three separate screen-space slider rows with orb handles, pointer mapping, 48 px targets, and keyboard nudges. `src/bin/ala-cities-target.rs` renders the real runtime chunk and exact octree-backed ray read-back without authoring UI. `src/bin/asset-builder.rs` is the external semantic authoring/review process. `src/octree.rs` and `src/raycast.rs` retain sparse occupancy and exact signed traversal. This is meaningful partial evidence for Q744/Q747/Q751/Q856–Q858. It still does not evidence full generator/source/table digests, a canonical whole-world projection identity, OpenPBR assets/manifests, greedy meshing, culling, measured performance, completed human promotion, or replacement of the legacy client.

**Remaining questions, owner, and exact next evidence.** Owners: [Dynamic asset builder and chunk mesher boundary](https://github.com/alacripose/ala-cities/issues/22) and [Generator performance and first-gate evidence](https://github.com/alacripose/ala-cities/issues/28). Next evidence: generator/source-digest change, whole-world projection invalidation beyond the current seven-chunk input, recipe/hash reproducibility, OpenPBR/manifest references, sparse culling correctness, and mechanical+human promotion records.

### 5.10 Save, replay, migration, and unload persistence

**Exact decisions.** Saves pin seed/revision and preserve explicit deltas; baseline is regenerated (Q667, Q752). Unsupported revision handling cannot silently discard or reinterpret deltas; it requires explicit migration input and reports every unresolved conversion (Q675, Q707, Q722; current v3 is specifically historical/unsupported under Q776). Replay compares a canonical integer/rational projection and excludes presentation/cache (Q699), now at declared tick/epoch boundaries (Q775). Current v3 is preserved as evidence but refused for authoritative new play (Q776).

**What this forbids or requires.** Forbid generic deserializer fallback, v3 reinterpretation, cache/session capture as replay, baseline loss, or a digest that omits future-affecting authoritative facts. Require lineage validation before field interpretation, a new envelope, journal/input ordering, delta reapplication, projection comparison, and explicit refusal/migration evidence.

**Code and architecture meaning.** CAP-001 supplies projection/tick semantics; CAP-007 owns envelope/journal/migration. Unload drops derived chunk/cache data but keeps typed deltas. Session capture remains evidence, not the semantic journal.

**Current implementation state.** Unload/rederive is partial source evidence for Q752 (`src/founding_day.rs:452-486,660-684,880-899`). `FoundingSnapshot` (`:810-840`) is an in-memory view, not a save envelope. The Q775 Python fixture demonstrates equality/presentation exclusion only. No production projection schema, journal, new save, v3 refusal, or migration path is evidenced.

**Remaining questions, owner, and exact next evidence.** Owners: [New save lineage and semantic replay migration](https://github.com/alacripose/ala-cities/issues/26) and [CAP-001 canonical world truth and replay projection](https://github.com/alacripose/ala-cities/issues/27). Next evidence: unsupported-v3 refusal artifact; new-lineage round trip; revision/source-digest mismatch refusal; same-tick changed-authority replay failure; presentation-only difference replay success; and migration fixture that either proves every mapping or fails closed.

### 5.11 Landing, founding-day UX, player, citizen, and picking

**Exact decisions.** A seed need only contain required resources somewhere; universal contiguous founding viability is not promised (Q674, Q677). An impossible founder landing may still be playable and must report that no resource/survival chain is guaranteed (Q719), while a valid golden seed/site and impossible edge are separate tests (Q734). The initial founder landing override is founder-only and recorded (Q690, Q692). The first-gate loop is hand-work land → gather → make → shelter/food/warmth → survive through the governed stack (Q749). Later Q840–Q853 narrow the first actor: one autonomous citizen, player inspection/governed requests, world-owned clock, real material path, data recipes, and observation-only first-night human gate. Q816 preserves citizen identity/task context across generic spawn. Q842/Q844 require inspect-click/use-key and exact free 3D picking.

**What this forbids or requires.** Forbid a fixed starter-camp implementation masquerading as unrestricted landing, a fake inventory, a player-completion prerequisite amended by Q852, direct command-body control, client-set night, or picking that mutates on click. Require a landing-domain/vitality read-back, golden and impossible-site fixtures, autonomous needs/utility task selection, governed execution, exact typed selection, evidence, and lifecycle.

**Code and architecture meaning.** Separate landing placement from world generation; record impossible-site state rather than create stock. Implement citizen/task adapters over the same material/authority/evidence interfaces. The first-night human sees autonomous work; optional player requests do not unlock completion.

**Current implementation state.** `FoundingWorld::starter_camp()` is fixed at `(4,4,0)` and landing rejects anything outside the first band that is not soil (`src/founding_day.rs:440-450,530-545`), so it does not implement unrestricted/impossible founder placement. The module directly executes synchronous `Action`s; it has no citizen, needs, utility, task, proposal, consent, claim, reservation, inspection UI, or lifecycle. `src/raycast.rs` proves exact signed traversal, not the complete Q844 camera or Q842 interaction. The clock constants implement Q849 durations, but the inspected module has no independent scheduler calling `advance_ticks`.

**Remaining questions, owner, and exact next evidence.** Generator-side owner: [Founding material chain and seeded ruin baseline](https://github.com/alacripose/ala-cities/issues/24); evidence owner: [Generator performance and first-gate evidence](https://github.com/alacripose/ala-cities/issues/28); actor/client contracts remain draft under [capability packet completion and evidence matrix](https://github.com/alacripose/ala-cities/issues/30). Next evidence: declared landing-domain predicate; valid and impossible-site seeds; autonomous first-night trace with typed task/material/evidence events; click-inspect/use-verb capture; exact ray selection; death/estate transfer fixture; and human observation report.

### 5.12 Gates, evidence, and release

**Exact decisions.** The first complete generator gate is comprehensive, not a rendering smoke test (Q683, Q708, Q730). It includes golden seeds/projections, baseline mass, signed/cross-boundary coordinates, revision refusal, resource/founding evidence, performance, visual capture, and feedback (Q683, Q708, Q730, Q734). Earth values are provisional until reviewed (Q750/Q804). Asset promotion requires mechanical and human approval (Q805). Release requires automated replay/conservation/accessibility/performance/provenance evidence plus the untimed human first-night playtest (Q806); Q852 scopes the human participation to observing the autonomous citizen. The parent ticket remains open through implementation and gate (Q736), and prototype evidence cannot promote semantics (Q753).

**What this forbids or requires.** Forbid closing the world-generator parent because prose exists, a headless probe standing in for a human gate, missing readings converted to zero/pass, provisional Earth values treated as approved, or a mock/legacy presentation path. Require named commands, raw readings, artifact links, failure retention, and append-only evidence.

**Code and architecture meaning.** Gate execution belongs downstream of capability semantics. A pass requires a reproducible build/source identity plus the exact Q708/Q730 evidence set. Current target-module tests are bounded fixtures, not the complete gate.

**Current implementation state.** The checked-in target audit reports five `founding_day` tests and a passing headless probe, while explicitly withholding graphical client, full fields/fluids, governed integration, lifecycle, durable save/replay, assets, performance/accessibility, and human release evidence (`docs/design/audits/2026-09-23-target-slice-0.md:26-64`). The inspected asset modules add partial presentation evidence, but no inspected record supplies the complete #28 evidence set or closes any owning ticket.

**Remaining questions, owner, and exact next evidence.** Owner: [Generator performance and first-gate evidence](https://github.com/alacripose/ala-cities/issues/28), with the parent [world generator pass — substrate, terrain, and seed determinism](https://github.com/alacripose/ala-cities/issues/21). Next evidence: a gate manifest that maps every required artifact to a command, raw reading, build/generator/builder/source identity, failure owner, and final human observation; every missing item remains red until produced.

## 6. Cross-cluster implementation order

This is a dependency order, not a claim that the live route can skip blockers:

1. Freeze the typed identity, coordinate, first-band, projection, and refusal envelopes (Q665, Q688, Q716, Q775–Q776, Q846).
2. Define regional/local field seams, subvoxel/mass schema, and the golden/impossible seed contracts (Q666, Q689, Q694–Q695, Q705, Q726, Q734).
3. Implement the smallest conserved hydrology/fluid/climate process and explicit process scheduler (Q684–Q706, Q735–Q740, Q746).
4. Extend extraction deltas to the real material/task/authority/evidence path (Q727–Q728, Q749, Q817–Q820, Q841, Q845, Q850–Q855).
5. Prove unload/rederive, replay, new-lineage save, explicit v3 refusal, and migration refusal (Q752, Q775–Q776).
6. Connect the required Dynamic Asset Generator to the immutable target snapshot and prove cache invalidation/presentation non-authority (Q744, Q747, Q751, Q856–Q858).
7. Integrate autonomous citizen/client observation, then run measured performance and the first-night human gate (Q840–Q853, Q806).
8. Add later ruin/civilization generation only after the Q746 core gate, retaining world-only mass and poor-yield/high-cost salvage (Q668, Q715, Q732, Q734).

The current implementation plan puts durable save/replay in Slice 5 and broad dynamics in Slice 6. That can support early prototypes only if their worlds remain explicitly non-authoritative/non-release and Q724 prevents later subsystem landing from changing accepted tick-zero state. It does not satisfy Q746 by schedule order alone.

## 7. Traceability matrix — every WG-Q1 through WG-Q91 exactly once

Cluster codes: **A** authority/determinism; **B** coordinates/chunks/octree; **C** regional fields/geology/materials; **D** terrain/elevation/seams; **E** hydrology/fluids/climate/ecology/atmosphere; **F** caves/erosion/tectonics/clocks; **G** deltas/mutation/conservation; **H** streaming/workers/performance; **I** presentation/assets/cache; **J** save/replay/migration; **K** landing/founding UX; **L** gates/evidence/release.

| WG / C11 | Cluster | Status | Decision and implementation/test consequence | Current source / next evidence |
|---|---|---|---|---|
| WG-Q1 / Q662 | A | Settled | One pure staged seed-to-world pipeline; gameplay changes are separate typed deltas/contracts. | `Chunk::derive` is pure, but named stages and the full delta contract are absent. Prove no-mutation and stage boundaries. |
| WG-Q2 / Q663 | B | Settled | 3D chunked world, bounded initial area, streamable expansion; 2D logic only where sufficient. | 3D chunks/first band exist; horizontal region policy and streaming are not evidenced. |
| WG-Q3 / Q664 | C | Settled scope; schema open | Full layered bedrock/geology/soil/elevation/water/climate/material/ruin substrate. | Only soil/forage/wood exist; no regional fields or complete layer schema. |
| WG-Q4 / Q665 | A | Settled | Chunk derivation is a pure function of seed, revision, and chunk coordinates. | Pure hash/derive exists; identity lacks named ruleset/source/table digests. |
| WG-Q5 / Q666 | C | Settled | Coupled physical fields, not independent noise, constrain terrain/water/material/biology. | No coupled fields are implemented. |
| WG-Q6 / Q667 | A | Settled | Baseline is re-derived; saves retain seed/revision plus explicit world/player deltas. | One removal delta survives unload; other delta classes and save are absent. |
| WG-Q7 / Q668 | G | Settled scope; later stage | Seeded in-world ruins have poor yield/high processing cost and already-counted mass; no outside import. | No ruin baseline. Prove later-stage provenance and conservation. |
| WG-Q8 / Q669 | B | Settled direction | 3D voxel generation; external voxel references inform scale/streaming, not copied architecture or dependencies. | Authoritative `BTreeMap` plus presentation octree align the separation; no full generator. |
| WG-Q9 / Q670 | C | Settled target; stage details partly open | Expanded target includes geology, erosion, climate/biology, hydrology, terrain, tectonics, soil/materials, ruins, caves, atmosphere, and star system, decomposed into passes. | Nearly all target layers are absent. |
| WG-Q10 / Q671 | E | Settled direction; roster open | Generalized conserved fluid/resource actor with pressure, flow, groundwater, rain, and carriers. | No fluid model. Build the smallest closed typed-fluid fixture. |
| WG-Q11 / Q672 | E | Scoped / deferred | Realistic Earth biomes are direction; exact realism envelope and field semantics remain open. | No biome fields or labels. |
| WG-Q12 / Q673 | C | Settled | Complete declared baseline material/family coverage, including geology-permitted clay, with evidence-gated distribution. | Three kinds are not a complete library. |
| WG-Q13 / Q674 | K | Settled but consequential | Resource existence somewhere is guaranteed; no universal contiguous viable founding area. | Fixed starter site exists; valid/impossible seed tests do not. |
| WG-Q14 / Q675 | J | Settled | Pin generator revision; mismatch requires named regeneration/migration or a new run. | `u64` revision exists; no mismatch refusal. |
| WG-Q15 / Q676 | B | Settled | 8 m logical/material tile contains finer sparse 3D voxels; it is not the voxel scale. | No tile/subvoxel conversion; Q846 separately freezes 1 m voxels. |
| WG-Q16 / Q677 | K | Settled but consequential | Generator guarantees resources somewhere, not a reachable first-work chain. | Headless path hard-codes a valid starter; edge behavior absent. |
| WG-Q17 / Q678 | E | Settled | Simulation-affecting atmosphere/weather is world truth; decorative sky/stars are presentation. | Neither is implemented. |
| WG-Q18 / Q679 | E | Settled roster direction; fidelity open | Typed water/oils/lava/gases share pressure/flow/phase semantics. | No typed-fluid roster. |
| WG-Q19 / Q680 | C | Settled current scope | Earth-like biomes only for this pass; alien biome physics needs a new decision. | No biome implementation. |
| WG-Q20 / Q681 | L | Settled | Substrate, hydrology, climate/biology, materials/surfaces, ruins, and atmosphere/sky are separately gated stages. | Current `derive` is not that staged architecture. |
| WG-Q21 / Q682 | F | Scoped commitment | Tectonics, erosion, caves, and weather are dynamic from tick zero; cadence/authority/conservation still require specification. | None exists. Later Q684/Q724/Q746 narrow the contract. |
| WG-Q22 / Q683 | L | Settled | First complete generator gate proves projection/replay, seams/negative coordinates, mass, revision refusal, resources, performance, and playability. | No complete gate evidence. |
| WG-Q23 / Q684 | F | Settled | Each dynamic process has explicit deterministic cadence and simulation class. | Only founding day/night exists; no process scheduler. |
| WG-Q24 / Q685 | E | Settled | Fluids carry quantity/volume, pressure, phase, flow, sources/sinks, and ledger accounts. | No fluid state. |
| WG-Q25 / Q686 | E | Settled | Pressure, temperature, humidity, precipitation, wind/weather are authoritative; sky/stars derive where applicable. | No atmosphere model. |
| WG-Q26 / Q687 | C | Settled | Biomes derive from quantitative fields and transition bands, not arbitrary hard labels. | No fields/labels. |
| WG-Q27 / Q688 | B | Settled | Signed seamless chunks share cross-boundary sampling. | Global coordinate hash exists; explicit adjacent-seam equality fixture is absent. |
| WG-Q28 / Q689 | D | Settled | Global/regional fields derive first; chunks sample consistently and cannot override them. | No regional stage. |
| WG-Q29 / Q690 + Q692 | K | Settled | Founder-only initial placement may be unrestricted/impossible; record it and create no matter. Q692 confirms that later movement, building, material use, and all other actors remain governed. | Current landing is restricted to first-band soil, so the edge contract is not implemented. |
| WG-Q30 / Q691 | F | Settled | Active chunks update exactly; inactive regions use deterministic aggregate tiers with conservation/replay. | No LOD/tier scheduler. |
| WG-Q31 / Q693 | D | Deferred | Exact bounded-start policy must choose radius, rectangle, landing-centered region, or another measurable rule. | Q709 gives center; exact horizontal shape remains open. |
| WG-Q32 / Q694 | B | Settled then first-slice frozen | Sparse octree sections use measured power-of-two geometry; Q846 freezes first target at 32³. | Chunk size and presentation octree are 32³; long-term measurement rationale is not evidenced. |
| WG-Q33 / Q695 | G | Settled | Derive untouched stock and reconcile it with extraction, processing, loss, and player deltas. | Coarse ledger works for accounted chunks and removal only. |
| WG-Q34 / Q696 | G | Settled | Retire cosmetic edge road; use governed cut/fill/borrow/spoil/path deltas with lineage. | Target earthworks absent; legacy edge road remains historical only. |
| WG-Q35 / Q697 | F | Settled | Derive cave/strata baseline; formation, collapse, and erosion are dynamic from tick zero. | No caves/dynamics. |
| WG-Q36 / Q698 | E | Settled | Initialize fluid sources/quantities/phases from coupled geology/climate/terrain rules, not random placement. | No fluid initialization. |
| WG-Q37 / Q699 | J | Settled; Q775 sharpens | Replay compares canonical integer/rational world truth at boundaries; cache/presentation are excluded. | Candidate digest and Python fixture only; no production projection/journal. |
| WG-Q38 / Q700 | H | Settled policy; values open | Record generation/streaming/memory/tick/draw readings per stage; derive budgets from measurements. | No performance evidence or budgets. |
| WG-Q39 / Q701 | D | Settled target direction; support policy open | Earth-scaled, technically open-ended declared vertical layers with real core progression. | First band exists; terminal support/refusal does not. |
| WG-Q40 / Q702 | D | Settled | Sparse local truth plus regional/global authoritative fields; caches are not authoritative. | Sparse chunk truth only; no regional fields. |
| WG-Q41 / Q703 | E | Settled | Ocean/groundwater/atmosphere/ice/steam and other water accounts close; every source/sink is explicit. | No water cycle. |
| WG-Q42 / Q704 | E | Settled | Quantitative Earth seasonal climate feeds authoritative atmosphere/biology. | Day/night is not climate; no fields. |
| WG-Q43 / Q705 | C | Settled | Deposits may be hidden below surfaces but require exposure and authority-aware extraction. | No deposit model. |
| WG-Q44 / Q706 | F | Settled | Tectonics, erosion/caves, hydrology, atmosphere, biology, and gameplay project in deterministic cadence order. | No process order. |
| WG-Q45 / Q707 | J | Settled; Q776 narrows | New lineage or explicit migration; never silently reinterpret current worlds. | No target save path; v3 disposition is historical/unsupported. |
| WG-Q46 / Q708 | L | Settled | Publish golden seeds, projections/hashes, mass, seam/negative cases, revision refusals, performance, and playable feedback. | Only bounded unit/probe evidence exists. |
| WG-Q47 / Q709 | D | Settled; exact shape open | Initial gate region is landing-centered; outside chunks may stream but are not in the first gate. | No initial-region predicate. |
| WG-Q48 / Q710 | C | Settled | Earth-inspired geological abstraction at game scale, not literal Earth thickness. | No geological model. |
| WG-Q49 / Q711 | F | Settled | Record/measure accelerated geological and weather time ratios rather than silently using literal rates. | No process clocks or source/time scaling. |
| WG-Q50 / Q712 | E | Settled | Initial season/weather derives from seed/revision and is replayable/recorded. | No seed-derived season/weather state. |
| WG-Q51 / Q713 | E | Settled | Rivers, lakes, seas, groundwater, rain, and flow derive as one conserved coupled network. | No hydrology. |
| WG-Q52 / Q714 | F | Settled | Connected 3D caves include strata, materials, fluids/gas, hazards, and core depth. | No caves. |
| WG-Q53 / Q715 | C | Settled | Ruin density/age/poverty/terrain/water/geology/history are seed/rule constrained; no outside import. | No ruins. |
| WG-Q54 / Q716 | J | Settled | Save identity is named ruleset/version plus source/table digests. | `GeneratorRevision(u64)` is only partial identity. |
| WG-Q55 / Q717 | L | Settled staging; later narrowed | Develop/prove core 3D substrate/conservation/replay before expanded systems; Q746 later places core dynamics in the first gate and keeps ruins later. | Target seam exists; #21 remains open. Treat Slice 6 breadth as prototype work unless Q746/Q724 state compatibility is proved. |
| WG-Q56 / Q718 | H | Settled | Workers may parallelize independent jobs, but dependencies, RNG, revision, sampling, and merge order are fixed. | No worker implementation. |
| WG-Q57 / Q719 | K | Settled | Impossible founder site may pass as playable; the game reports no guaranteed resources/survival rather than inventing matter. | Current strict landing rejects the edge instead of representing it. |
| WG-Q58 / Q720 | C | Settled | Earth tables carry source/range, units, rationale, and declared game-scaling tolerances. | Provisional research table exists; no admitted generator table/digest. |
| WG-Q59 / Q721 | G | Settled; `DESIGN.md` summary stale | Embodied physical interaction still passes task/authority/conservation; it is not a world editor or godmode. | Direct `Action` path has no actor/task/authority proof. |
| WG-Q60 / Q722 | J | Settled; `DESIGN.md` summary stale | Retained deltas may feed explicit migration; unresolved conversions are reported; silent baseline loss/reinterpretation is forbidden. | No migration implementation. |
| WG-Q61 / Q723 | A | Settled | CAP-001 owns world/chunks/generator/projection; CAP-002 matter; CAP-007 persistence; CAP-008 spatial consumption. | Module boundary is directionally consistent; packet/project remains draft. |
| WG-Q62 / Q724 | F | Settled | Every accepted world has all target dynamics active from tick zero; later implementation must reproduce correct state. | Current checkpoint has no target geology/fluid/weather dynamics. |
| WG-Q63 / Q725 | C | Settled | Derive deep-time prehistory as baseline, then expose simulation tick zero. | Hard-coded starter material is not derived prehistory. |
| WG-Q64 / Q726 | B | Settled | One typed 8 m logical-tile ↔ voxel/subvoxel conversion serves geometry, mass, simulation, and rendering. | No conversion contract. |
| WG-Q65 / Q727 | G | Settled | Baseline immutable; extraction, roads, terrain, structures, fluids, and player/world changes are typed deltas. | Only voxel-mass removal is represented. |
| WG-Q66 / Q728 | G | Settled | Cut/fill/borrow/spoil have lineage and affect traversal, erosion, hydrology, and later dynamics. | No earthworks. |
| WG-Q67 / Q729 | A | Settled | Flat `World`/`Tile` is not canonical target truth; only explicit artifact migration is allowed. | New module is separate; legacy runtime remains evidence. |
| WG-Q68 / Q730 | L | Settled | Full generator/schema/material/ledger/projection/boundary/performance/playtest command and evidence set. | No complete command set. |
| WG-Q69 / Q731 | D | Settled | Core is the terminal supported depth; requests beyond it refuse. | Arbitrary i32 chunks can load; no refusal. |
| WG-Q70 / Q732 | L | Settled | Ruins follow the first core-substrate gate; reserve their provenance contract now. | Ruins absent and ticket still open. |
| WG-Q71 / Q733 | C | Settled | Initial stock is derived, account-visible, and reconciled through every source/sink/process/delta. | Coarse three-kind accounts are partial. |
| WG-Q72 / Q734 | L | Settled | Use a named valid golden seed/site and test unrestricted/impossible landing separately. | Probe uses seed 7 and fixed camp; no declared golden contract or impossible edge. |
| WG-Q73 / Q735 | E | Settled | Regional numeric fields are authoritative; biome names/readable categories derive. | No fields or labels. |
| WG-Q74 / Q736 | L | Settled | The world-generator parent remains open through implementation and required gates/playtest. | Parent observed open; no closure claimed. |
| WG-Q75 / Q737 | A | Settled | Pure typed Rust modules/functions for global fields, chunk derivation, and deltas; no hidden mutable generator state/ECS requirement. | `Chunk::derive` is pure; global fields and full stage API are absent. |
| WG-Q76 / Q738 | C | Settled | Earth values live in one typed, sourced, versioned table with units/scaling/tolerances/digest consumed by generated Rust. | Markdown research artifact is provisional; no canonical generated table. |
| WG-Q77 / Q739 | F | Settled | Tectonics is plate/region scale, erosion chunk/voxel, cave change local, with declared cross-scale effects. | No geological process tiers. |
| WG-Q78 / Q740 | E | Settled | Fluid flow uses deterministic typed pressure/cellular semantics and conservation across pipes, groundwater, weather, and phases. | No fluid solver. |
| WG-Q79 / Q741 | E | Settled | Numeric regional fields are authoritative; biome labels and vegetation visuals derive. | No climate/biology fields. |
| WG-Q80 / Q742 | I | Settled | Generated cache identity includes ruleset/digests, seed, and chunk coordinates; cache is disposable/non-authoritative. | Builder cache has seed/u64 revision/chunk/builder hash; ruleset/table and world-state invalidation are not complete. |
| WG-Q81 / Q743 | B | Settled | Signed horizontal/vertical coordinates and declared negative/far-coordinate hash conversion from the beginning. | Signed types/hash/negative chunk test exist; full seam/extreme fixtures remain. |
| WG-Q82 / Q744 | I | Settled | Retire flat models as target truth; Dynamic Asset Generator owns visible 3D presentation, not world truth. | Partial module separation is visible. |
| WG-Q83 / Q745 | C | Settled policy; values provisional | Earth values require citation, range, unit, scaling, and review before entry. | Provisional table records these fields; human release admission remains. |
| WG-Q84 / Q746 | F | Settled | First gate includes fluids, weather, erosion/caves, climate/core dynamics; ruins/civilizations follow later. | These dynamics are not implemented; #21 remains open. |
| WG-Q85 / Q747 | I | Settled | Generator emits spatial truth; separate mesher, culling, and render adapter consume it. | `&FoundingWorld` presentation build demonstrates the one-way boundary. |
| WG-Q86 / Q748 | L | Settled | Parent world-generator decision retains named child tasks for schema, core, fluids, climate, ruins, gates, and migration. | The named child tickets in the issue register exist and were observed open. |
| WG-Q87 / Q749 | K | Settled; later first-slice corrections | First generator gate proves a governed hand-work founding survival loop, not only rendering. | Synchronous actions/material counters do not prove citizen/task/authority/lifecycle integration. |
| WG-Q88 / Q750 | C | Settled; Q804 clarifies release | Agent may admit researched Earth values provisionally with provenance; human review remains required before release. | Research artifact labels all rows provisional/human-review-required. |
| WG-Q89 / Q751 | I | Settled | Mesh key includes generator ruleset/digest, builder hash, and chunk coordinates; output is disposable. | Current key is partial and lacks a mutable world-state digest. |
| WG-Q90 / Q752 | J | Settled | Unload drops derived/cache data; typed deltas persist and reapply on reload. | One delta class and digest test provide bounded evidence. |
| WG-Q91 / Q753 | L | Settled | Prototypes may tune dimensions/rates/cache/workers/tolerances only inside contracts; semantics, conservation, and authority require explicit decisions. | Current constants/fixtures are evidence, not authority; packet and release gates remain open. |

## 8. Contradiction and ambiguity ledger

The following conflicts are recorded rather than silently repaired. Later-question chronology resolves some items; others remain explicit blockers.

| ID | Conflict or ambiguity | Controlling/disputed reading | Required disposition |
|---|---|---|---|
| A-01 | `DESIGN.md` Q721 says mediated-vs-direct mutation is unresolved, while detailed C11-Q721 resolves it and Q848 later confirms no direct mutation. | Detailed answer plus later Q848 control the consequence: embodied physical acts still pass the governed stack. | Treat the one-line `DESIGN.md` row as stale projection; packet #30 must synchronize it without rewriting history. |
| A-02 | `DESIGN.md` Q722 says discard-baseline/conflict unresolved, while detailed C11-Q722 requires explicit delta migration and Q776 refuses v3. | Detailed Q722 + Q707 + Q775–Q776 control: v3 is historical/unsupported; only a separately proven migration may proceed. | Correct the projection; do not infer a generic migration. |
| A-03 | CAP-001 inherited coverage binds Q667 to field coupling and Q668 to baseline/deltas, but canonical C11 binds coupling to Q666 and baseline/deltas to Q667. | The C11 headings and answers are the direct record: Q666 coupling, Q667 derived/stored, Q668 ruins. | Fix packet coverage under #30; keep this matrix’s Q-ID mapping explicit. |
| A-04 | CAP-001 says exact first-target voxel dimensions/vertical domain remain measured/open, but Q846 freezes 1 m, 32³, signed i32, and `z=-16..+47`. | Q846 controls the **first target implementation**. The longer-term core extent, regional-field resolution, and subvoxel schema may remain open. | Split “first-slice constants” from “long-term domain/schema” in packet wording. |
| A-05 | CAP-008 still lists Q790 “player must act,” but Q852 explicitly amends the first-night human gate to observation-only. | Q852 is the scoped amendment; Q847–Q848 and Q853 define autonomous citizen work. | Remove player-action prerequisite from the first-night packet claim; preserve Q790 history. |
| A-06 | Q717 places expanded systems after a core first pass, while Q746 includes climate/weather/erosion/caves/fluids in the first complete generator gate. | Q746 is later and narrows the **acceptance gate**: development may stage substrate first, but first-gate acceptance includes core dynamics; ruins remain later. | The plan’s later dynamics slice cannot close #21 without Q724 state compatibility and Q746 evidence. |
| A-07 | Q682/Q724 require dynamics from tick zero, while the current plan/checkpoint exposes a playable founding slice before generator breadth/dynamics. | Early prototype use is allowed, but accepted canonical worlds must either already have correct tick-zero dynamics or be deterministically reconstructed/refused when the later model lands. | Add an explicit prototype/non-acceptance marker and state-compatibility evidence; do not silently activate dynamics. |
| A-08 | Q690/Q719 allow unrestricted/impossible initial founder placement, while Q846 defines a first playable band and Q693 leaves horizontal bounds open. | “Unrestricted” cannot override the declared first-target domain silently. The exact horizontal landing predicate remains a blocker; vertical out-of-band must refuse under Q846. | Decide the landing-domain/vitality policy and test valid plus impossible in-domain and out-of-band cases separately. |
| A-09 | Q701 says technically open-ended vertical target, Q731 makes core terminal, and Q846 defines a small first band. | These can coexist only if the first band is a subset, future declared layers extend it, and a finite supported core refuses beyond its boundary. | Name separate first-playable, generated, and supported-core domains; do not use one constant for all three. |
| A-10 | Q674/Q677 promise only global resources, while Q749 requires a viable first founding loop. | Q734 resolves the apparent conflict: a valid golden seed/site proves the loop; impossible founder placement is a separate honest edge behavior, not a universal guarantee. | Maintain two fixtures and two expected outcomes. |
| A-11 | Q745 says reviewed source table; Q750 permits agent admission. | Q750 permits **provisional** agent admission; Q804 requires human review before release. | Keep every Earth row provisional until reviewed; citation alone is not release approval. |
| A-12 | Q663/Q693 say the initial area is bounded but exact policy deferred; Q709 says landing-centered; Q846 freezes only the vertical band. | Center is settled; radius/rectangle and horizontal extent remain open. | Define a measurable `InitialPlayableRegion` and test that outside generation does not enter first-gate scope. |
| A-13 | Q700 is summarized as “settled,” but exact performance budgets are expressly derived from missing measurements. | The measurement policy is settled; numeric budgets are open until #28 evidence. | Do not convert the policy decision into a numeric completion claim. |
| A-14 | Q777 says every wheel verb schedules future governed work, while Q817–Q820 require immediate physical consequences and possible preview reservations. | Proposal scheduling and physical execution are distinct phases of one governed pipeline; preview reservation is itself authoritative state, not a direct completion. | Specify attempt → reservation/claim → effect → adjudication ordering and rollback/correction boundaries. |
| A-15 | The first builder cache identity omitted mutable world deltas; the follow-up now adds a versioned seven-chunk `ChunkInputDigest`. | The bounded presentation dependency is resolved for the current mesher: the chunk plus six cardinal neighbour boundary/residency facts invalidate correctly. Full generator/source/table and whole-world projection identity remain separate work. | Preserve `ChunkInputDigest` as cache input, not world authority; add broader digests only when their consumers exist. |
| A-16 | Existing audit text says the asset builder/mesher/cache is still blocked, while current source now contains a partial runtime generator, external semantic builder tool, octree, controls, and exact raycast. | The audit remains historical checkpoint evidence; current source supersedes its absence claim only for the bounded features now present. | Describe the generator/tool pair as partial and re-run #22/#28 evidence; do not restore “absent” or claim full completion. |
| A-17 | All inspected capability packets are draft/partial, while some inherited rows are labeled “settled” and the implementation plan is current. | Decision settlement, packet approval, implementation evidence, and release eligibility are different states. | Preserve all four states separately; #30 and #21 remain open. |

## 9. Evidence index

- World-generator answers: `docs/GRILLING-C11.md:5068-5972` (C11-Q662–Q753).
- Replay/save corrections: `docs/GRILLING-C11.md:6199-6219` (Q775–Q776).
- Physical-effect corrections: `docs/GRILLING-C11.md:6626-6675` (Q817–Q820), with persistent transition at `:6698-6709` (Q816).
- First-target corrections: `docs/GRILLING-C11.md:6999-7161` (Q840–Q855).
- Asset-generator corrections: `docs/GRILLING-C11.md:7165-7204` (Q856–Q858).
- Concise constitution projection: `DESIGN.md:503-594,616-617,656-660,680-699`.
- Draft capability state and blockers: `docs/design/capabilities/CAP-001.json:1-131`, `CAP-002.json:1-73`, `CAP-007.json:1-131`, `CAP-008.json:1-80`, `CAP-010.json:1-74`.
- Target implementation checkpoint: `docs/design/audits/2026-09-23-target-slice-0.md:1-64`.
- Q775 fixture boundary: `prototypes/cap001-projection/README.md:1-42`.
- Provisional Earth evidence: `docs/research/earth-source-table.md:1-23,82-96,129-131`.
- Inspected source: `src/founding_day.rs`, `src/asset_generator.rs`, `src/asset_controls.rs`, `src/octree.rs`, `src/raycast.rs`, `src/bin/ala-cities-target.rs`, and `src/bin/asset-builder.rs` as cited in each cluster.

## 10. Bottom line for implementers

The world generator may be extended only along the settled dependency spine: **identity → signed coordinates → coupled fields → supported terrain/seams → conserved dynamic processes → baseline plus typed deltas → canonical projection → persistence/replay → presentation/client → measured evidence**. A later subsystem cannot silently become active, a later geometry cannot reinterpret old baseline, a presentation recipe cannot alter world truth, and a viable starter path cannot be smuggled in as a universal seed guarantee. The first target constants and autonomous-citizen slice are now concrete, but the complete generator, durable lineage, required asset path, and release evidence remain open.
