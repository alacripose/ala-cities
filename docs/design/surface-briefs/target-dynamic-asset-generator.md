# Target Dynamic Asset Generator surface brief

## Scope and visitor mode

- **Surface:** target native game client, not the asset-authoring tool.
- **Mode:** Experience.
- **Primary job:** render the real generated chunk produced by the Dynamic Asset Generator and support exact world observation/picking.
- **Audience:** a player or reviewer inspecting the target game presentation path.
- **Constraints:** real CAP-001 world truth, real chunk mesher, sparse octree/cache, and real camera ray; no legacy-client bridge, raw hash-bit editing, builder recipe controls, or OpenPBR completion claim.

## Direction contract

### THESIS

The city is the work, not a preview of the work. The target client proves that the game renders the Dynamic Asset Generator's real chunk output; authoring controls remain outside the player experience.

### OWN-WORLD

The established restrained industrial game surface: matte blue-black panels, quiet grid/line work, compact measurements, and terrain/material color doing the visual work. The generator has no ornamental control chrome in this surface.

### STORY

The client loads canonical world truth, asks the required generator for a chunk asset, and renders that asset. A camera ray returns voxel read-back from the same sparse octree. The resulting frame proves the production presentation dependency without exposing an authoring workflow.

### FIRST VIEWPORT

The generated chunk occupies the native viewport. A compact upper-left readout identifies the Dynamic Asset Generator chunk path, shows build identity and cache state, and reports the exact ray hit. Orbiting, panning, zooming, and clicking remain game-view interactions.

### FORM

A normal target-game observation surface. The Dynamic Asset Generator is infrastructure behind the image, not a settings panel or modal. No asset-authoring controls appear in the game.

### FINISH

unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance

## Unresolved work

- OpenPBR material authoring and validation remain the next material pass.
- Greedy meshing, broader sparse-culling/LOD budgets, and measured performance remain open under issue #22/#28.
- The target client remains a new binary alongside the legacy client until the target route earns replacement authority.
