# World-generation grilling structure

This is the short visual companion to `world-generation-grilling-structure.excalidraw`. The long engineering argument and all 91 WG rows live in [`../world-generation-grilling-map.md`](../world-generation-grilling-map.md); this page is a decision-flow overview, not a second authority.

## Visual argument

```mermaid
flowchart TB
    AM1["Q775–Q776<br/>tick/epoch replay · v3 refused"] -. sharpens .-> A
    AM2["Q816–Q820<br/>persistent citizen · real effect · reservation"] -. sharpens .-> G
    AM3["Q840–Q855<br/>clock · 1 m/32³/i32 · first band<br/>autonomous citizen · estate transfer"] -. sharpens .-> K
    AM4["Q856–Q858<br/>required runtime · external semantic sliders"] -. sharpens .-> I

    A["A Authority + determinism<br/>identity · pure stages · projection · clock"]
    B["B Signed coordinates + chunks<br/>seams · sparse state · first band · support limit"]
    C["C Regional fields + geology + materials<br/>global first · prehistory · sourced tables"]
    D["D Terrain + elevation + seams<br/>regional scope · landing region · core limit"]
    E["E Hydrology + climate + ecology + atmosphere<br/>typed fluids · closed accounts · fields"]
    F["F Caves + erosion + tectonics + clocks<br/>cadence · order · tick zero · cross-scale"]
    G["G Deltas + mutation + conservation<br/>immutable baseline · lineage · reservations"]

    J["J Save + replay + migration<br/>new lineage · v3 refusal · canonical projection"]
    I["I Presentation + asset cache<br/>builder · meshes · culling · one-way only"]
    K["K Landing + first night<br/>autonomous citizen · inspect/request/observe"]
    H["H Streaming + workers + performance<br/>fixed merge · deterministic LOD · measure first"]
    L["L Evidence + release gate<br/>projection · seams · mass · refusal · migration<br/>performance · autonomous human-observed night"]

    A --> B --> C --> D --> E --> F --> G
    G --> J
    G --> I
    I --> K
    K --> H
    J -. readings .-> H
    H --> L

    X["EXPLICIT AMBIGUITIES<br/>DESIGN Q721/Q722 stale projections<br/>Q717 vs Q746 first-gate scope<br/>Q693/Q719 landing domain<br/>Q701/Q731/Q846 extent split"]
    X -. blocks silent resolution .-> L

    R["RETIRED / DEMOTED<br/>flat World·Tile truth · cosmetic edge road · 8 m as voxel scale<br/>cache as save · v3 as target lineage · direct editor mutation<br/>player-completion prerequisite · optional asset path<br/>hard-label biomes · water-only cases · late dynamics activation"]
    R -. forbidden shortcuts .-> L

    classDef settled fill:#a7f3d0,stroke:#047857,color:#065f46,stroke-width:2px;
    classDef amended fill:#ddd6fe,stroke:#6d28d9,color:#4c1d95,stroke-width:2px;
    classDef downstream fill:#dbeafe,stroke:#1e40af,color:#1e3a8a,stroke-width:2px;
    classDef measure fill:#fef3c7,stroke:#b45309,color:#92400e,stroke-width:2px;
    classDef blocked fill:#fee2e2,stroke:#b91c1c,color:#991b1b,stroke-width:3px;
    class A,B,C,D,E,F,G settled;
    class AM1,AM2,AM3,AM4 amended;
    class J,I,K downstream;
    class H,X,R measure;
    class L blocked;
```

## Reading order

1. **Purple rail:** later C11 target decisions amend the original WG pass; they do not announce implementation completion.
2. **Green spine:** identity and supported coordinates precede coupled fields, terrain, Earth systems, dynamic processes, and conserved deltas.
3. **Blue branches:** CAP-007 persistence, CAP-010 presentation, and founding-day UX consume the world/delta contract through separate authority boundaries.
4. **Amber floor:** worker/performance behavior and all known record ambiguities remain explicit; measurements do not choose semantics.
5. **Red floor:** the generator/new-save/first-night gate remains blocked until every named evidence class exists.

## Critical forks

- **No universal seed guarantee:** a valid golden seed proves the founding loop; an impossible founder site is a separate honest edge case (WG-Q13, WG-Q16, WG-Q57, WG-Q72).
- **First band is not the long-term core:** Q846 freezes the first target's `z=-16..+47`; Q731 still requires a finite supported-core refusal for the longer target.
- **First-slice order is not gate order:** early substrate/dynamics prototypes may precede broad generator work, but Q746 and Q724 prohibit treating a world without correct tick-zero dynamics as accepted.
- **Presentation is downstream:** the Dynamic Asset Generator is required, but recipe/hash/builder-tool/cache changes never alter CAP-001 truth (WG-Q82, WG-Q85, WG-Q89; Q856–Q858).
- **Human gate changed:** the first human observes an autonomous citizen's attempt; player physical completion is no longer a prerequisite (Q852).

## Rendering and review guide

- Open the native source in Excalidraw: `docs/design/diagrams/world-generation-grilling-structure.excalidraw`.
- Read the diagram left-to-right across the green spine, then follow the blue authority branches to the amber and red evidence floors.
- Green means a settled contract, not completed code. Purple means a later amendment. Blue is downstream persistence/presentation/UX. Amber is measured or disputed. Red is blocked.
- The canonical full-Q traceability and disputed mappings are in `docs/design/world-generation-grilling-map.md`; do not use the cluster nodes as a substitute for that matrix.
- Optional official render command from the repository root:

```text
uv run --with playwright python docs/design/diagrams/render_excalidraw.py docs/design/diagrams/world-generation-grilling-structure.excalidraw docs/design/diagrams/rendered/world-generation-grilling-structure.png --scale 2
```

**Native source:** `docs/design/diagrams/world-generation-grilling-structure.excalidraw`
**Detailed synthesis:** `docs/design/world-generation-grilling-map.md`
