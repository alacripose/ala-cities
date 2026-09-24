# Dynamic Asset Generator: runtime presentation and external authoring

> **Subordinate visual companion.** `DESIGN.md` remains the concise product authority and `docs/GRILLING-C11.md` remains the decision record. This note explains one corrected architecture view; it does not restate or supersede either document.

## Visual argument

The red vertical line is an explicit authority boundary. CAP-001 world truth is on the left; only the required runtime read crosses the boundary. The external authoring lane is on the right and never points back into CAP-001.

### Path A — required target runtime

`CAP-001 world truth → runtime Dynamic Asset Generator library → sparse chunk octree/cache → chunk mesh → target game renderer`

The generator is required target infrastructure. It owns the runtime chunk presentation path, including sparse acceleration, materials, and manifests. The renderer is the game surface, not a builder or authoring surface. Its separate exact voxel ray-picking path is read-only game observation and returns hit evidence; it is not a world mutation or an authoring control.

**Current-code grounding.** `src/asset_generator.rs` keys the runtime chunk cache by seed/revision (the implemented generator identity), chunk coordinates, the chunk-builder recipe, and a versioned `ChunkInputDigest`. The digest covers the resident chunk plus the residency and inward-facing boundary voxels of its six cardinal neighbours. This bounds the presentation input to seven chunks; it is not full generator/source/table identity or a canonical whole-world projection digest.

### Path B — external semantic authoring

`external asset-builder tool → three separate sliders with floating orb handles → derived candidate BuilderHash → one coherent gear mesh → mechanical/human review → promoted presentation asset`

The three semantic controls are separate: **Teeth** for profile morph, **Opening** for the root-radius ratio, and **Accent** for construction. Each slider is an authoring control in the external tool. The target renderer contains no builder sliders, orb handles, raw recipe editor, or other authoring UI. Only runtime/promoted presentation assets are consumed by the game.

The authoring loop has no arrow into CAP-001. The red boundary and the separate lane make that direction impossible to misread.

## Boundary rules shown

- Shape recipe changes never divide or rescale chunk voxels; the authoring recipe is not a world-coordinate scaler.
- Raw hash bits are not editable. The BuilderHash is derived from the named recipe values.
- The external tool may produce a candidate, but the candidate is not represented as a game asset until mechanical/human review and promotion.
- The amber unresolved strip deliberately preserves open work: OpenPBR authoring/validation, the reference browser, broader LOD/culling budgets, measured performance, and release evidence (finish review, verdict, and provenance).

## Claim-to-Q856–Q858 map

The following source text is carried exactly from the corrected rows in `DESIGN.md`:

| Diagram claim | Exact decision source |
|---|---|
| The generator is required target infrastructure; it consumes world truth and owns runtime chunk meshes, sparse acceleration, materials, and manifests, while its authoring UI remains a separate tool. | **Q856** — ✔ required target infrastructure; consumes world truth and owns runtime chunk meshes, sparse acceleration, materials, and manifests, while its authoring UI remains a separate tool |
| The builder uses named float controls and a derived hash; the gear proof uses Teeth/profile, Opening/root-radius ratio, and Accent/construction, never X/Y/Z voxel scaling or raw hash-bit editing. | **Q857** — ✔ named float controls determine a derived hash; the gear-first proof uses Teeth/profile, Opening/root-radius ratio, and Accent/construction, never X/Y/Z voxel scaling or raw hash-bit editing |
| The three orb-handle sliders are in the external builder tool; the target game exposes no authoring controls and consumes only runtime/promoted presentation assets. | **Q858** — ✔ three separate sliders with floating orb handles edit semantic recipe axes in the external builder tool; the target game exposes no authoring controls and consumes only runtime/promoted presentation assets |

The exact voxel ray-picking read path and the unresolved-work strip are companion constraints carried from `target-dynamic-asset-generator.md` and `asset-builder.md`; they are not new claims attributed to Q856–Q858.

## Deliberately incomplete

This is a boundary and dependency diagram, not a completion claim. It does not assert OpenPBR completion, reference-browser completion, broader LOD/culling completion, measured performance, or release evidence.

**Native source:** `docs/design/diagrams/dynamic-asset-generator.excalidraw`
