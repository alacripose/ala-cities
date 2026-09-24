# Licensed cubic-chunk architecture references

## Intent

The Dynamic Asset Generator uses the following upstream projects as licensed architectural references for cubic 3D chunking and concurrent chunk management. This is an implementation reference, not a claim that the Rust engine is a Minecraft clone.

## C2ME — concurrent chunk management

- Project: `PhantomG27249/C2ME` (C^2M-Engine)
- License: MIT
- Canonical repository: https://github.com/PhantomG27249/C2ME
- Reference architecture: concurrent chunk management for world generation, chunk I/O, and loading; parallel CPU work; deterministic behavior and vanilla-parity checks.
- Adaptation boundary: use the ideas as a Rust design reference for deterministic staged generation, residency-aware scheduling, and bounded work. Do not import Minecraft-specific code or assume its thread model is correct for this engine.

## Cubic Chunks — true cubic vertical sections

- Project: `OpenCubicChunks/CubicChunks`
- License: MIT
- Canonical repository: https://github.com/OpenCubicChunks/CubicChunks
- Reference architecture: true cubic vertical chunks, signed coordinate space, and 32-bit coordinate limits rather than a fixed-height 2D column.
- Adaptation boundary: use the cubic-section model with this project's settled first-target contract: 1 m voxels, 32³ chunks, signed `i32` coordinates, and the first playable vertical band `z=-16..+47`. The first playable band is not the long-term terminal core boundary.

## Current implementation consequences

- The target preview uses a named full 32³ cubic fixture to prove volume occupancy, section meshing, octree behavior, and camera culling.
- The fixture is not canonical generator truth and does not replace the sparse, field-coupled world generator.
- Runtime presentation consumes immutable chunk truth and emits disposable indexed triangle meshes; it does not write back to the world.
- The user's clarification is recorded here: the references are intended for licensed architectural use. “Do not copy their architecture” means do not paste or imitate code without respecting the applicable license and provenance; it does not prohibit adapting their architectural ideas under MIT terms.
- Any future direct adaptation of upstream source must preserve the applicable MIT notice and add a source/provenance record before the code is accepted.
