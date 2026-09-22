# The OpenPBR parameter table

`UNIFIED_DESIGN.md` §3.2 names its colour roles, and the icons have to *locate*
those surfaces rather than decorate them: an icon is "a locator to a surface or
record", never a capability grant. So every icon in `assets/icons/` is rendered
from a material defined here, in **OpenPBR** vocabulary, and every icon carries a
record of what the renderer could and could not honour.

## Why there is a mapping table at all

Two things are true at once, and pretending otherwise would be the whole
problem:

1. **OpenPBR Surface is the standard.** It is an ASWF specification
   (`AcademySoftwareFoundation/OpenPBR`) describing a surface shading model as a
   layered base + coat + fuzz + thin-film stack with named parameters.
2. **Blender 5.3.0 Alpha does not implement it.** Measured on this machine, in
   the build that renders these icons:

   | Probe | Result |
   |---|---|
   | `[n for n in dir(bpy.types) if 'openpbr' in n.lower()]` | `[]` — no node type |
   | `[o for o in dir(bpy.ops.wm) if 'materialx' in o.lower()]` | `[]` — no MaterialX I/O |
   | PBR-capable BSDFs present | `BsdfPrincipled`, `BsdfMetallic`, `BsdfHairPrincipled`, `BsdfVolumePrincipled` |

   Upstream is explicit that this is unfinished: `projects.blender.org` issue
   **#145127** ("OpenPBR Compatibility") says exact OpenPBR import and export is
   *wanted*, not delivered.

So the icons are **authored against the OpenPBR parameter set** and rendered
through **Principled BSDF v2**, which is OpenPBR-aligned in design but is not a
conformance-exact implementation. [`DEVIATIONS`](openpbr.py) records every place
the two differ, per parameter, and each icon's manifest entry carries the
deviations that applied to it. A claim of conformance that the toolchain cannot
demonstrate is not written down anywhere in this repository.

## The parameters in use

Only the parameters these icons actually need. The names are the specification's;
the second column is where each one goes in the shader, and the third is what the
mapping costs.

| OpenPBR parameter | Principled BSDF v2 input | Deviation |
|---|---|---|
| `base_color` | `Base Color` | none — same meaning and range |
| `base_metalness` | `Metallic` | none degenerate case: the shader's metallic is also an explicit reflector |
| `specular_roughness` | `Roughness` | **the spec separates dielectric and metallic roughness; the shader has one.** Icons never mix the two on one surface, so the collapse is exercised nowhere |
| `specular_ior` | `IOR` | none for dielectrics; the shader derives `F0` from it the same way at normal incidence |
| `specular_weight` | `Specular IOR Level` | **the spec's weight is linear on the reflectance at normal incidence; the shader's is a 0–1 artist scale.** Unused by these icons — left at default |
| `base_weight` | implicit (single base layer) | **the spec allows layering several bases; the shader does not.** Icons use exactly one base layer, which is why this is a caveat and not a defect |
| `geometry_normal` | `Normal` | none (not driven by these icons) |
| `geometry_coat_weight` | `Coat Weight` | none — but the spec's coat has its own roughness and IOR, and the shader's defaults are used unmodified |
| `emission_color` / `emission_luminance` | `Emission Color` / `Emission Strength` | **luminance in the spec is in `cd/m²`; the shader's strength is unitless and scaled by the exposure.** Emission is used for nothing in this set, deliberately: an emissive icon would read as a *state*, and state comes from data, not from artwork |

Absent from this table on purpose: `transmission_*`, `subsurface_*`, `fuzz_*`,
`thin_film_*`, `transmission_depth`. The first three are for materials these
icons do not have, and the last two are **not implemented by the target shader at
all** — using them would mean the manifest recording a parameter the renderer
ignored, which is exactly the silent-drop defect class `UNIFIED_DESIGN.md` §8.1
names.

## The palette, in the design doc's own roles

Each icon borrows its base colour from the token it locates, so the icon and the
surface agree rather than merely coexist:

| Icon | Role it locates | Base colour source |
|---|---|---|
| `tool-road` | the road tool | `Token::Road` / `Token::RoadEdge` |
| `tool-zone` | the zoning tool | `Token::ZoneResidential` |
| `tool-power` | the power tool | `Token::Powered` |
| `tool-demolish` | the demolition tool | `Token::Warning` |
| `tool-inspect` | the lens | `Token::NotObtained` |
| `ledger` | the ticket board | `Token::PanelRaised` / `Token::Plaque` |
| `ticket` | one ticket | `Token::CaseOpen` |
| `district` | a district | `Token::ZoneCommercial` |
| `power` | the power service | `Token::Nature` |
| `view-power`, `view-zoning`, `view-traffic`, `view-ledger` | the four info views that have data | as above, desaturated for the view row |

The colours are **not** hand-picked: `tools/icons/palette.py` recomputes them
from the same OKLCH values `src/hud.rs` uses, so an icon cannot drift away from
the token it stands for. If the token changes, the icon is re-rendered — the
manifest records the palette hash so that "the icons were regenerated after the
palette moved" is a fact rather than an intention.
