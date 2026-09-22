"""The icons: geometry, and what each one locates.

`UNIFIED_DESIGN.md` §9 and the round-9 answers fix the rule this file obeys: an
icon is **a locator to a surface or record**, never a grant of authority and
never decoration. So the set is derived from surfaces that exist — the five
tools, the ledger, one ticket, a district, the power service, and the four info
views that have data behind them — and not from a list of things that would look
nice.

Everything is built in the X/Z plane facing a camera that looks along +Y, inside
±1.0 of the origin, so `ortho_scale` frames it with a margin. Silhouettes are
deliberately heavy: an icon has to survive being averaged down to 24 px, and fine
detail does not.
"""

import math

from palette import rgba

# ---------------------------------------------------------------------------
# Primitives
# ---------------------------------------------------------------------------


def box(bpy, name, material, x, z, w, h, depth=0.34, angle=0.0, y=0.0):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=(x, y, z))
    obj = bpy.context.active_object
    obj.name = name
    obj.scale = (w, depth, h)
    obj.rotation_euler = (0.0, math.radians(angle), 0.0)
    obj.data.materials.append(material)
    return obj


def ring(bpy, name, material, x, z, major, minor, angle=0.0):
    bpy.ops.mesh.primitive_torus_add(
        major_radius=major,
        minor_radius=minor,
        major_segments=48,
        minor_segments=12,
        location=(x, 0.0, z),
        rotation=(math.radians(90.0), 0.0, math.radians(angle)),
    )
    obj = bpy.context.active_object
    obj.name = name
    obj.data.materials.append(material)
    return obj


#: How much of the frame a view glyph occupies. See `build`.
FRAMED_GLYPH_SCALE = 0.6


def frame(bpy, materials, name, inset=0.0, thickness=0.1, half=0.94):
    """A square border. Four bars, because there is no line primitive."""
    parts = []
    span = half * 2.0 - inset * 2.0
    for index, (dx, dz, w, h) in enumerate(
        [
            (0.0, half - inset, span, thickness),
            (0.0, -half + inset, span, thickness),
            (-half + inset, 0.0, thickness, span - thickness),
            (half - inset, 0.0, thickness, span - thickness),
        ]
    ):
        parts.append(
            box(bpy, f"{name} frame {index}", materials["accent"], dx, dz, w, h, depth=0.26)
        )
    return parts


# ---------------------------------------------------------------------------
# Builders
# ---------------------------------------------------------------------------


def road(bpy, materials):
    parts = [box(bpy, "carriageway", materials["primary"], 0.0, 0.0, 0.92, 1.92)]
    for index, z in enumerate((0.62, 0.0, -0.62)):
        parts.append(
            box(bpy, f"centre line {index}", materials["accent"], 0.0, z, 0.11, 0.34, 0.4)
        )
    return parts


def zoning(bpy, materials):
    parts = []
    for index, (x, z) in enumerate(((-0.42, 0.42), (0.42, 0.42), (-0.42, -0.42), (0.42, -0.42))):
        # Three tiles at one density and one taller, because a zone is a
        # density, not a colour.
        h = 0.92 if index == 3 else 0.62
        parts.append(box(bpy, f"tile {index}", materials["primary"], x, z, 0.72, h, 0.3))
    return parts


def power_tool(bpy, materials):
    return [
        box(bpy, "mast", materials["primary"], 0.0, -0.1, 0.16, 1.6),
        box(bpy, "crossarm", materials["primary"], 0.0, 0.5, 1.24, 0.14, 0.3),
        box(bpy, "insulator west", materials["accent"], -0.55, 0.32, 0.11, 0.3, 0.22),
        box(bpy, "insulator east", materials["accent"], 0.55, 0.32, 0.11, 0.3, 0.22),
    ]


def demolish(bpy, materials):
    return [
        box(bpy, "slab", materials["primary"], 0.12, 0.34, 0.95, 1.16, 0.34, angle=18.0),
        box(bpy, "rubble", materials["accent"], -0.34, -0.62, 0.86, 0.44, 0.3),
    ]


def inspect(bpy, materials):
    return [
        ring(bpy, "lens", materials["primary"], -0.2, 0.2, 0.56, 0.15),
        box(bpy, "handle", materials["accent"], 0.52, -0.52, 0.78, 0.19, 0.3, angle=-45.0),
    ]


def ledger(bpy, materials):
    parts = [box(bpy, "spine", materials["accent"], -0.66, 0.0, 0.16, 1.62, 0.4)]
    for index, z in enumerate((0.52, -0.02, -0.56)):
        parts.append(box(bpy, f"plate {index}", materials["primary"], 0.12, z, 1.44, 0.42, 0.36))
    return parts


def ticket(bpy, materials):
    return [
        box(bpy, "stub", materials["primary"], -0.06, 0.0, 1.66, 0.94, 0.3),
        box(bpy, "perforation", materials["accent"], 0.42, 0.0, 0.1, 0.94, 0.36),
        box(bpy, "check digit", materials["accent"], -0.62, -0.62, 0.3, 0.16, 0.36),
    ]


def district(bpy, materials):
    parts = []
    for index, (dx, dz, w, h) in enumerate(
        [(-0.4, 0.82, 1.72, 0.16), (-0.4, -0.82, 1.72, 0.16), (-0.82, -0.4, 0.16, 1.4), (0.82, -0.4, 0.16, 1.4)]
    ):
        parts.append(box(bpy, f"boundary {index}", materials["primary"], dx, dz, w, h, 0.28))
    parts.append(box(bpy, "block", materials["accent"], -0.3, -0.32, 0.7, 0.7, 0.34))
    return parts


def power_service(bpy, materials):
    return [
        box(bpy, "bolt upper", materials["primary"], 0.1, 0.42, 0.5, 0.98, 0.32, angle=26.0),
        box(bpy, "bolt lower", materials["primary"], -0.1, -0.42, 0.5, 0.98, 0.32, angle=26.0),
        box(bpy, "busbar", materials["accent"], 0.0, -0.92, 1.3, 0.16, 0.3),
    ]


def traffic(bpy, materials):
    parts = [box(bpy, "carriageway", materials["accent"], 0.0, -0.42, 1.78, 0.5, 0.3)]
    for index, x in enumerate((-0.56, 0.0, 0.56)):
        parts.append(box(bpy, f"vehicle {index}", materials["primary"], x, 0.28, 0.42, 0.56, 0.34))
    return parts


# ---------------------------------------------------------------------------
# The set
# ---------------------------------------------------------------------------

#: Every icon: the surface it locates, the token its colour comes from, and the
#: OpenPBR parameters it is made of. `framed` icons are the info views, which
#: carry a border so a view glyph is distinguishable from a tool glyph at 24 px.
ICONS = [
    {
        "id": "tool-road",
        "surface": "the road tool",
        "locates": "Token::Road",
        "builder": road,
        "primary": {"base_color": rgba("Road"), "base_metalness": 0.0,
                    "specular_roughness": 0.55, "specular_ior": 1.45},
        "accent": {"base_color": rgba("RoadEdge"), "base_metalness": 0.0,
                   "specular_roughness": 0.62, "specular_ior": 1.45},
    },
    {
        "id": "tool-zone",
        "surface": "the zoning tool",
        "locates": "Token::ZoneResidential",
        "builder": zoning,
        "primary": {"base_color": rgba("ZoneResidential"), "base_metalness": 0.0,
                    "specular_roughness": 0.48, "specular_ior": 1.45},
        "accent": {"base_color": rgba("ZoneResidential", 0.9), "base_metalness": 0.15,
                   "specular_roughness": 0.4, "specular_ior": 1.45},
    },
    {
        "id": "tool-power",
        "surface": "the power tool",
        "locates": "Token::Powered",
        "builder": power_tool,
        "primary": {"base_color": rgba("Powered"), "base_metalness": 0.1,
                    "specular_roughness": 0.42, "specular_ior": 1.5},
        "accent": {"base_color": rgba("Retired"), "base_metalness": 0.9,
                   "specular_roughness": 0.3, "specular_ior": 1.5},
    },
    {
        "id": "tool-demolish",
        "surface": "the demolition tool",
        "locates": "Token::Warning",
        "builder": demolish,
        "primary": {"base_color": rgba("Warning"), "base_metalness": 0.0,
                    "specular_roughness": 0.6, "specular_ior": 1.45},
        "accent": {"base_color": rgba("Refused"), "base_metalness": 0.0,
                   "specular_roughness": 0.66, "specular_ior": 1.45},
    },
    {
        "id": "tool-inspect",
        "surface": "the inspect lens",
        "locates": "Token::NotObtained",
        "builder": inspect,
        "primary": {"base_color": rgba("NotObtained"), "base_metalness": 0.85,
                    "specular_roughness": 0.28, "specular_ior": 1.5,
                    "geometry_coat_weight": 0.4},
        "accent": {"base_color": rgba("NotObtained", 0.8), "base_metalness": 0.2,
                   "specular_roughness": 0.45, "specular_ior": 1.45},
    },
    {
        "id": "ledger",
        "surface": "the ticket board",
        "locates": "Token::PanelRaised",
        "builder": ledger,
        "primary": {"base_color": rgba("PanelRaised"), "base_metalness": 0.0,
                    "specular_roughness": 0.58, "specular_ior": 1.45},
        "accent": {"base_color": rgba("Plaque"), "base_metalness": 0.7,
                   "specular_roughness": 0.34, "specular_ior": 1.5},
    },
    {
        "id": "ticket",
        "surface": "one ticket",
        "locates": "Token::CaseOpen",
        "builder": ticket,
        "primary": {"base_color": rgba("CaseOpen"), "base_metalness": 0.0,
                    "specular_roughness": 0.5, "specular_ior": 1.45},
        "accent": {"base_color": rgba("Warning"), "base_metalness": 0.0,
                   "specular_roughness": 0.5, "specular_ior": 1.45},
    },
    {
        "id": "district",
        "surface": "a district",
        "locates": "Token::ZoneCommercial",
        "builder": district,
        "primary": {"base_color": rgba("ZoneCommercial"), "base_metalness": 0.0,
                    "specular_roughness": 0.5, "specular_ior": 1.45},
        "accent": {"base_color": rgba("ZoneCommercial", 0.95), "base_metalness": 0.0,
                   "specular_roughness": 0.44, "specular_ior": 1.45},
    },
    {
        "id": "power",
        "surface": "the power service",
        "locates": "Token::Nature",
        "builder": power_service,
        "primary": {"base_color": rgba("Nature"), "base_metalness": 0.0,
                    "specular_roughness": 0.42, "specular_ior": 1.5},
        "accent": {"base_color": rgba("Nature", 0.85), "base_metalness": 0.0,
                   "specular_roughness": 0.5, "specular_ior": 1.45},
    },
    {
        "id": "view-power",
        "surface": "the power info view",
        "locates": "Token::Nature",
        "builder": power_service,
        "framed": True,
        "primary": {"base_color": rgba("Nature"), "base_metalness": 0.0,
                    "specular_roughness": 0.44, "specular_ior": 1.5},
        "accent": {"base_color": rgba("Nature", 0.7), "base_metalness": 0.0,
                   "specular_roughness": 0.5, "specular_ior": 1.45},
    },
    {
        "id": "view-zoning",
        "surface": "the zoning and demand view",
        "locates": "Token::ZoneResidential",
        "builder": zoning,
        "framed": True,
        "primary": {"base_color": rgba("ZoneResidential"), "base_metalness": 0.0,
                    "specular_roughness": 0.48, "specular_ior": 1.45},
        "accent": {"base_color": rgba("ZoneResidential", 0.75), "base_metalness": 0.0,
                   "specular_roughness": 0.5, "specular_ior": 1.45},
    },
    {
        "id": "view-traffic",
        "surface": "the traffic info view",
        "locates": "Token::Agent",
        "builder": traffic,
        "framed": True,
        "primary": {"base_color": rgba("Agent"), "base_metalness": 0.0,
                    "specular_roughness": 0.46, "specular_ior": 1.45},
        "accent": {"base_color": rgba("Road"), "base_metalness": 0.0,
                   "specular_roughness": 0.6, "specular_ior": 1.45},
    },
    {
        "id": "view-ledger",
        "surface": "the ledger info view",
        "locates": "Token::TextMuted",
        "builder": ledger,
        "framed": True,
        "primary": {"base_color": rgba("TextMuted"), "base_metalness": 0.0,
                    "specular_roughness": 0.52, "specular_ior": 1.45},
        "accent": {"base_color": rgba("Plaque"), "base_metalness": 0.5,
                   "specular_roughness": 0.4, "specular_ior": 1.5},
    },
]


def build(bpy, icon, materials):
    """Build one icon's geometry, framed if it is an info view.

    A framed icon's glyph is built at **60 %** and inset, because the first run of
    this pipeline measured the framed views at 0.72–0.74 ink coverage at 24 px:
    the border was carrying the icon and the glyph inside it had nowhere to be.
    The border says "this is a view"; the glyph is what the view is of, and at
    24 px the glyph has to win.
    """
    parts = icon["builder"](bpy, materials)
    if icon.get("framed"):
        for obj in parts:
            obj.location.x *= FRAMED_GLYPH_SCALE
            obj.location.z *= FRAMED_GLYPH_SCALE
            obj.scale.x *= FRAMED_GLYPH_SCALE
            obj.scale.z *= FRAMED_GLYPH_SCALE
        parts.extend(frame(bpy, materials, icon["id"]))
    return parts
