"""The inventory: what each icon means, and the geometry data that draws it.

Two tiers, and the difference is the whole point of the file:

* **surface** — an icon that locates something the game actually has: a tool, a
  record, a service, a view. It must name the surface it locates, and the pipeline
  refuses an icon whose surface is not on that list. In the design document's own
  words (section 12.9), an icon is *a locator to a surface or record*, and its
  forbidden reading is *a capability grant*.
* **vocabulary** — an icon that carries a meaning from the design grammar: save,
  add, close, warning. The user asked for *"a list of icons from the relevant
  design specs to recreate within the new system"*. What is recreated is the
  **grammar** — a gear means settings — and never the artwork: sections 1 and 0.5
  put "icon cloning" and "brand replication" in the *not adopted* column, and this
  repository is public, so reproducing a company's marks would be a trademark
  problem rather than merely a lapse of taste.

One rule the first render pass did not have, and the contrast check forced:

    INK CARRIES THE SILHOUETTE.  IDENTITY CARRIES THE COLOUR.

The parts that decide whether the glyph is visible are drawn in a token bright
enough to reach 3:1 on every host the icon declares (WCAG 2.2 1.4.11, measured,
not asserted). The token that says *what this locates* appears in parts big enough
to survive the smallest shipped size. A dark token may not carry a silhouette on a
dark host: an icon that is invisible on the panel it was placed on has failed even
if every other number about it is right.

Geometry is deliberately heavy — silhouette is what survives being averaged down
to 24 px — and everything is built inside ±1.0 of the origin in the X/Z plane,
facing a camera that looks along +Y.
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


# ---------------------------------------------------------------------------
# Authoring helpers, so an icon is data rather than a function
# ---------------------------------------------------------------------------


def B(x, z, w, h, depth=0.34, angle=0.0, role="ink"):
    """A box part."""
    return {"shape": "box", "x": x, "z": z, "w": w, "h": h, "depth": depth,
            "angle": angle, "role": role}


def R(x, z, major, minor, angle=0.0, role="ink"):
    """A ring part. There is no line primitive in the primitives available, so a
    circle is a torus and a stroke is a long, thin box."""
    return {"shape": "ring", "x": x, "z": z, "major": major, "minor": minor,
            "angle": angle, "role": role}


def mat(token, roughness, ior=1.45, metallic=0.0, coat=0.0):
    """A material from a token, as OpenPBR parameters.

    Deliberately only parameters this set actually expresses: an unused parameter
    is a claim the manifest would carry and the render would not.
    """
    parameters = {
        "base_color": rgba(token),
        "base_metalness": metallic,
        "specular_roughness": roughness,
        "specular_ior": ior,
    }
    if coat:
        parameters["geometry_coat_weight"] = coat
    return parameters


def icon(icon_id, kind, meaning, source, sits_on, materials, parts,
         identity=None, identity_as="detail", framed=False, notes=None,
         locates=None):
    """One inventory entry. Everything the pipeline and the manifest need."""
    return {
        "id": icon_id,
        "kind": kind,
        "meaning": meaning,
        "source": source,
        "locates": locates or source,
        "sits_on": tuple(sits_on),
        "identity": identity,
        "identity_as": identity_as,
        "materials": materials,
        "parts": parts,
        "framed": framed,
        "notes": list(notes or ()),
    }


#: The two inks. Both are `TextBody`, and that is a measured decision rather than
#: a taste one: the first run of the contrast check put `TextMuted` at **2.1-2.7:1**
#: on the panels the icons declare, under the 3:1 non-text floor, so a softer ink is
#: a different *surface finish* here (roughness), never a darker colour. A darker
#: ink is not a softer icon, it is an icon nobody can see.
INK_STRONG = mat("TextBody", 0.52, 1.45)
INK_MID = mat("TextBody", 0.66, 1.45)


# ---------------------------------------------------------------------------
# Tier 1 — surface locators
# ---------------------------------------------------------------------------

SURFACES = [
    icon(
        "tool-road",
        "surface",
        "the road tool",
        "game surface: Tool::Road",
        ("Panel", "PanelRaised"),
        {"ink": INK_MID, "identity": mat("Road", 0.55, 1.45)},
        [
            B(0.0, 0.0, 0.92, 1.92),
            B(0.0, 0.62, 0.11, 0.34, 0.4, role="identity"),
            B(0.0, 0.0, 0.11, 0.34, 0.4, role="identity"),
            B(0.0, -0.62, 0.11, 0.34, 0.4, role="identity"),
        ],
        identity="Road",
        identity_as="detail-on-ink",
        locates="Tool::Road",
        notes=["centre markings are the identity colour: the carriageway is ink so "
               "the glyph is visible, and the markings say which tool it is"],
    ),
    icon(
        "tool-zone",
        "surface",
        "the zoning tool",
        "game surface: Tool::Zone",
        ("Panel", "PanelRaised"),
        {"ink": INK_MID, "identity": mat("ZoneResidential", 0.5, 1.45)},
        [
            B(-0.42, 0.42, 0.72, 0.62, 0.3),
            B(0.42, 0.42, 0.72, 0.62, 0.3),
            B(-0.42, -0.42, 0.72, 0.62, 0.3),
            B(0.42, -0.42, 0.72, 0.92, 0.3),
            B(0.42, -0.42, 0.72, 0.2, 0.36, role="identity"),
        ],
        identity="ZoneResidential",
        identity_as="detail-on-ink",
        locates="Tool::Zone",
        notes=["three tiles at one density and one taller, because a zone is a "
               "density rather than a colour"],
    ),
    icon(
        "tool-power",
        "surface",
        "the power tool",
        "game surface: Tool::Power",
        ("Panel", "PanelRaised"),
        {"ink": INK_MID, "identity": mat("Powered", 0.45, 1.5)},
        [
            B(0.0, -0.1, 0.16, 1.6),
            B(0.0, 0.5, 1.24, 0.14, 0.3),
            B(-0.55, 0.32, 0.11, 0.3, 0.22, role="identity"),
            B(0.55, 0.32, 0.11, 0.3, 0.22, role="identity"),
        ],
        identity="Powered",
        identity_as="detail-on-ink",
        locates="Tool::Power",
    ),
    icon(
        "tool-demolish",
        "surface",
        "the demolition tool",
        "game surface: Tool::Demolish",
        ("Panel", "PanelRaised"),
        {"ink": mat("Warning", 0.6, 1.45), "identity": mat("Refused", 0.66, 1.45)},
        [
            B(0.12, 0.34, 0.95, 1.16, 0.34, angle=18.0),
            B(-0.3, -0.15, 0.86, 0.44, 0.3, role="identity"),
        ],
        identity="Warning",
        identity_as="detail-on-ink",
        locates="Tool::Demolish",
        notes=[
            "Warning is this tool's identity, not a state: the material never "
            "changes with state, and state marks elsewhere are additive",
            "demolition retires rather than deletes, which is why the slab is "
            "tipped and the rubble is held in the refused colour",
        ],
    ),
    icon(
        "tool-inspect",
        "surface",
        "the inspect lens",
        "game surface: Tool::Inspect",
        ("Panel", "PanelRaised"),
        {"ink": INK_MID, "identity": mat("NotObtained", 0.3, 1.5, 0.85, 0.4)},
        [
            R(-0.2, 0.2, 0.56, 0.15),
            B(0.52, -0.52, 0.78, 0.19, 0.3, angle=-45.0),
            B(-0.2, 0.2, 0.24, 0.24, 0.42, role="identity"),
        ],
        identity="NotObtained",
        identity_as="detail-on-ink",
        locates="Tool::Inspect",
        notes=["the lens is the same grammar as the vocabulary's search icon, "
               "which is why search is refused there: one meaning, one glyph"],
    ),
    icon(
        "ledger",
        "surface",
        "the ticket board",
        "game surface: the ledger panel",
        ("Panel", "PanelRaised"),
        {"ink": INK_STRONG, "identity": mat("Plaque", 0.34, 1.5, 0.7)},
        [
            B(-0.66, 0.0, 0.16, 1.62, 0.4, role="identity"),
            B(0.12, 0.52, 1.44, 0.42, 0.36),
            B(0.12, -0.02, 1.44, 0.42, 0.36),
            B(0.12, -0.56, 1.44, 0.42, 0.36),
            B(0.12, 0.52, 1.0, 0.1, 0.4, role="identity"),
            B(0.12, -0.02, 1.0, 0.1, 0.4, role="identity"),
            B(0.12, -0.56, 1.0, 0.1, 0.4, role="identity"),
        ],
        identity="Plaque",
        identity_as="detail-on-ink",
        locates="the ledger panel",
        notes=["the plates are ink and the rules across them are metal: a plaque is "
               "metal and a page is paper, which is honest material language rather "
               "than decoration — and a dark metal spine beside the host was "
               "invisible, which the contrast check said in numbers"],
    ),
    icon(
        "ticket",
        "surface",
        "one ticket",
        "game surface: a ticket in the ledger",
        ("Panel", "PanelRaised"),
        {"ink": INK_STRONG, "identity": mat("CaseOpen", 0.5, 1.45)},
        [
            B(-0.06, 0.0, 1.66, 0.94, 0.3),
            B(0.42, 0.0, 0.1, 0.94, 0.36, role="identity"),
            B(-0.62, -0.62, 0.3, 0.16, 0.36, role="identity"),
        ],
        identity="CaseOpen",
        identity_as="detail-on-ink",
        locates="a ticket in the ledger",
        notes=["a ticket is paper: ink, with the perforation and the check digit "
               "carrying the colour of an open case"],
    ),
    icon(
        "district",
        "surface",
        "a district",
        "game surface: a named district",
        ("Panel", "PanelRaised"),
        {"ink": INK_MID, "identity": mat("ZoneCommercial", 0.5, 1.45)},
        [
            B(-0.4, 0.82, 1.72, 0.16, 0.28),
            B(-0.4, -0.82, 1.72, 0.16, 0.28),
            B(-0.82, -0.4, 0.16, 1.4, 0.28),
            B(0.82, -0.4, 0.16, 1.4, 0.28),
            B(-0.3, -0.32, 0.7, 0.7, 0.34),
            B(-0.3, -0.32, 0.34, 0.34, 0.4, role="identity"),
        ],
        identity="ZoneCommercial",
        identity_as="detail-on-ink",
        locates="a named district",
    ),
    icon(
        "power",
        "surface",
        "the power service",
        "game surface: the electricity network",
        ("Panel", "PanelRaised"),
        {"ink": INK_MID, "identity": mat("Nature", 0.45, 1.5)},
        [
            B(0.1, 0.42, 0.5, 0.98, 0.32, angle=26.0),
            B(-0.1, -0.42, 0.5, 0.98, 0.32, angle=26.0),
            B(0.0, -0.78, 1.3, 0.16, 0.3, role="identity"),
        ],
        identity="Nature",
        identity_as="detail-on-ink",
        locates="the electricity network",
    ),
    icon(
        "view-power",
        "surface",
        "the power info view",
        "game surface: view `power`",
        ("Panel", "PanelRaised"),
        {"ink": INK_MID, "identity": mat("Nature", 0.45, 1.5)},
        [
            B(0.1, 0.42, 0.5, 0.98, 0.32, angle=26.0),
            B(-0.1, -0.42, 0.5, 0.98, 0.32, angle=26.0),
            B(0.0, -0.78, 1.3, 0.16, 0.3, role="identity"),
        ],
        identity="Nature",
        identity_as="detail-on-ink",
        locates="view `power`",
        framed=True,
    ),
    icon(
        "view-zoning",
        "surface",
        "the zoning and demand view",
        "game surface: view `zoning`",
        ("Panel", "PanelRaised"),
        {"ink": INK_MID, "identity": mat("ZoneResidential", 0.5, 1.45)},
        [
            B(-0.42, 0.42, 0.72, 0.62, 0.3),
            B(0.42, 0.42, 0.72, 0.62, 0.3),
            B(-0.42, -0.42, 0.72, 0.62, 0.3),
            B(0.42, -0.42, 0.72, 0.92, 0.3),
            B(0.42, -0.42, 0.72, 0.2, 0.36, role="identity"),
        ],
        identity="ZoneResidential",
        identity_as="detail-on-ink",
        locates="view `zoning`",
        framed=True,
    ),
    icon(
        "view-traffic",
        "surface",
        "the traffic info view",
        "game surface: view `traffic`",
        ("Panel", "PanelRaised"),
        {"ink": INK_MID, "identity": mat("Road", 0.55, 1.45)},
        [
            B(0.0, -0.42, 1.78, 0.5, 0.3),
            B(-0.56, 0.28, 0.42, 0.56, 0.34),
            B(0.0, 0.28, 0.42, 0.56, 0.34),
            B(0.56, 0.28, 0.42, 0.56, 0.34),
            B(0.0, -0.42, 1.78, 0.14, 0.38, role="identity"),
        ],
        identity="Road",
        identity_as="detail-on-ink",
        locates="view `traffic`",
        framed=True,
        notes=["the vehicles are ink and the lane they run in carries the colour: an "
               "agent-coloured vehicle was near-white on near-white ink, and the "
               "design record had already settled that traffic is drawn where the "
               "road is"],
    ),
    icon(
        "view-ledger",
        "surface",
        "the ledger view",
        "game surface: view `ledger`",
        ("Panel", "PanelRaised"),
        {"ink": INK_STRONG, "identity": mat("Plaque", 0.34, 1.5, 0.7)},
        [
            B(-0.66, 0.0, 0.16, 1.62, 0.4, role="identity"),
            B(0.12, 0.52, 1.44, 0.42, 0.36),
            B(0.12, -0.02, 1.44, 0.42, 0.36),
            B(0.12, -0.56, 1.44, 0.42, 0.36),
            B(0.12, 0.52, 1.0, 0.1, 0.4, role="identity"),
            B(0.12, -0.02, 1.0, 0.1, 0.4, role="identity"),
            B(0.12, -0.56, 1.0, 0.1, 0.4, role="identity"),
        ],
        identity="Plaque",
        identity_as="detail-on-ink",
        locates="view `ledger`",
        framed=True,
    ),
]

# ---------------------------------------------------------------------------
# Tier 2 — the design grammar, recreated
# ---------------------------------------------------------------------------

#: Where the grammar comes from, stated once. These are the meanings the
#: interfaces this project draws from put on these forms; the forms themselves are
#: this rig's own.
GRAMMAR = "universal interface grammar, as carried by the iOS 6 era and MD1"

VOCABULARY = [
    icon("vocab-settings", "vocabulary", "settings", GRAMMAR, ("Panel", "PanelRaised"),
         {"ink": INK_STRONG, "identity": INK_MID},
         [R(0.0, 0.0, 0.58, 0.16),
          B(0.0, 0.78, 0.2, 0.32, 0.34),
          B(0.0, -0.78, 0.2, 0.32, 0.34),
          B(0.78, 0.0, 0.32, 0.2, 0.34),
          B(-0.78, 0.0, 0.32, 0.2, 0.34),
          B(0.55, 0.55, 0.3, 0.19, 0.34, angle=45.0),
          B(-0.55, -0.55, 0.3, 0.19, 0.34, angle=45.0),
          B(0.55, -0.55, 0.3, 0.19, 0.34, angle=-45.0),
          B(-0.55, 0.55, 0.3, 0.19, 0.34, angle=-45.0)],
         identity="TextMuted", identity_as="body",
         notes=["a gear is the form family the user named for animated icons: this "
                "is the one that turns while the sim is working"]),
    icon("vocab-save", "vocabulary", "save", GRAMMAR, ("Panel", "PanelRaised"),
         {"ink": INK_STRONG, "identity": mat("Plaque", 0.34, 1.5, 0.7)},
         [B(0.0, 0.0, 1.5, 1.5, 0.3),
          B(0.0, 0.45, 0.72, 0.6, 0.36, role="identity"),
          B(0.0, -0.42, 1.0, 0.46, 0.36)],
         identity="Plaque",
         notes=["a sheet of paper with a metal shutter: the same material language "
                "as the ledger, so a save reads as a page and not as a shape"]),
    icon("vocab-load", "vocabulary", "load", GRAMMAR, ("Panel", "PanelRaised"),
         {"ink": INK_STRONG, "identity": INK_MID},
         [B(0.0, -0.72, 1.5, 0.2, 0.32),
          B(-0.62, -0.62, 0.2, 0.44, 0.32),
          B(0.62, -0.62, 0.2, 0.44, 0.32),
          B(0.0, 0.1, 0.22, 1.1, 0.3),
          B(-0.3, 0.6, 0.5, 0.2, 0.3, angle=-40.0),
          B(0.3, 0.6, 0.5, 0.2, 0.3, angle=40.0)],
         identity="TextMuted", identity_as="body"),
    icon("vocab-add", "vocabulary", "add", GRAMMAR, ("Panel", "PanelRaised"),
         {"ink": INK_STRONG, "identity": INK_MID},
         [B(0.0, 0.0, 0.3, 1.5, 0.32), B(0.0, 0.0, 1.5, 0.3, 0.32)],
         identity="TextMuted", identity_as="body"),
    icon("vocab-close", "vocabulary", "close", GRAMMAR, ("Panel", "PanelRaised"),
         {"ink": INK_STRONG, "identity": INK_MID},
         [B(0.0, 0.0, 0.3, 1.55, 0.32, angle=45.0),
          B(0.0, 0.0, 0.3, 1.55, 0.32, angle=-45.0)],
         identity="TextMuted", identity_as="body"),
    icon("vocab-check", "vocabulary", "accepted, verified", GRAMMAR,
         ("Panel", "PanelRaised"),
         {"ink": mat("Nature", 0.5, 1.45), "identity": INK_MID},
         [B(-0.36, -0.18, 0.26, 0.62, 0.32, angle=-45.0),
          B(0.28, 0.22, 0.26, 1.05, 0.32, angle=42.0)],
         identity="Nature", identity_as="body",
         notes=["a tick is the one form that may only mean verified or accepted: "
                "it is never used for 'the request succeeded' without a record"]),
    icon("vocab-warning", "vocabulary", "warning", GRAMMAR, ("Panel", "PanelRaised"),
         {"ink": mat("Warning", 0.5, 1.45), "identity": INK_MID},
         [B(0.0, 0.16, 0.34, 1.16, 0.34), B(0.0, -0.66, 0.34, 0.34, 0.34)],
         identity="Warning", identity_as="body"),
    icon("vocab-blocked", "vocabulary", "refused, not obtained", GRAMMAR,
         ("Panel", "PanelRaised"),
         {"ink": mat("Refused", 0.55, 1.45), "identity": INK_MID},
         [R(0.0, 0.0, 0.66, 0.16), B(0.0, 0.0, 0.26, 1.42, 0.3, angle=45.0)],
         identity="Refused", identity_as="body",
         notes=["refused is deliberately distinct from error, and this is the "
                "form it borrows: a struck-through circle, never a red cross"]),
    icon("vocab-info", "vocabulary", "what this is", GRAMMAR, ("Panel", "PanelRaised"),
         {"ink": INK_STRONG, "identity": INK_MID},
         [B(0.0, -0.42, 0.3, 0.3, 0.34), B(0.0, 0.24, 0.32, 1.0, 0.34)],
         identity="TextMuted", identity_as="body"),
    icon("vocab-list", "vocabulary", "a list of records", GRAMMAR,
         ("Panel", "PanelRaised"),
         {"ink": INK_STRONG, "identity": INK_MID},
         [B(-0.1, 0.52, 1.4, 0.24, 0.32),
          B(0.02, 0.0, 1.16, 0.24, 0.32),
          B(0.14, -0.52, 0.92, 0.24, 0.32)],
         identity="TextMuted", identity_as="body"),
    icon("vocab-clock", "vocabulary", "time, elapsed", GRAMMAR, ("Panel", "PanelRaised"),
         {"ink": INK_STRONG, "identity": INK_MID},
         [R(0.0, 0.0, 0.68, 0.14), B(0.0, 0.22, 0.18, 0.56, 0.3),
          B(0.22, -0.06, 0.5, 0.18, 0.3)],
         identity="TextMuted", identity_as="body"),
    icon("vocab-document", "vocabulary", "a record, a page", GRAMMAR,
         ("Panel", "PanelRaised"),
         {"ink": INK_STRONG, "identity": INK_MID},
         [B(0.0, 0.06, 1.14, 1.4, 0.3),
          B(0.4, 0.66, 0.42, 0.42, 0.36, angle=45.0, role="identity"),
          B(0.0, -0.3, 0.72, 0.16, 0.36, role="identity"),
          B(0.0, -0.02, 0.72, 0.16, 0.36, role="identity")],
         identity="Plaque"),
    icon("vocab-folder", "vocabulary", "a group of locators", GRAMMAR,
         ("Panel", "PanelRaised"),
         {"ink": INK_STRONG, "identity": INK_MID},
         [B(-0.3, 0.62, 0.6, 0.26, 0.32),
          B(0.0, -0.14, 1.5, 1.0, 0.32)],
         identity="TextMuted", identity_as="body",
         notes=["in the design document a folder organises locators and never "
                "organises authority, and the form has to say the same"]),
    icon("vocab-trash", "vocabulary", "retire, remove", GRAMMAR, ("Panel", "PanelRaised"),
         {"ink": INK_MID, "identity": mat("Refused", 0.6, 1.45)},
         [B(0.0, 0.62, 1.3, 0.22, 0.32),
          B(0.24, -0.36, 0.3, 0.4, 0.32, angle=45.0),
          B(0.0, -0.22, 1.04, 1.14, 0.3),
          B(-0.34, -0.22, 0.16, 0.86, 0.36, role="identity"),
          B(0.34, -0.22, 0.16, 0.86, 0.36, role="identity")],
         identity="Refused",
         notes=["a bin is not a deletion: things retired here keep their record, "
                "so the bin is ink and only its ribs carry the colour"]),
]

#: Forms deliberately **not** in the inventory, each with the reason. A refusal
#: list is worth more than a silence: it is the difference between a library that
#: grows on purpose and one that grows by accident.
VOCABULARY_REFUSALS = [
    {
        "form": "a magnifier for `search`",
        "reason": "the inspect tool already holds that grammar; one meaning, one glyph",
    },
    {
        "form": "four equal squares for `grid`",
        "reason": "the zoning tool already holds it, and a zone is a density rather than a grid",
    },
    {
        "form": "brand marks (a compass for navigation, a company's storefront glyph, a camera, a play triangle)",
        "reason": "UNIFIED_DESIGN.md section 1 puts `icon cloning` in the not-adopted column and "
                  "section 0.5 puts `brand replication` there; the grammar is recreated, the "
                  "artwork is not, and this repository is public",
    },
    {
        "form": "a padlock for `locked`",
        "reason": "nothing in the interface is locked; drawing it would imply an access "
                  "control that does not exist",
    },
    {
        "form": "a coin or a banknote for `money`",
        "reason": "the budget is a real number with a record behind it, and a decorative "
                  "coin next to it would be a fiction standing beside a fact",
    },
]

#: Every icon, both tiers.
ICONS = SURFACES + VOCABULARY

#: The surfaces the game declares. An icon of kind `surface` must be on this list,
#: so the inventory can never describe a surface the game does not have.
DECLARED_SURFACES = (
    "Tool::Road", "Tool::Zone", "Tool::Power", "Tool::Demolish", "Tool::Inspect",
    "the ledger panel", "a ticket in the ledger", "a named district",
    "the electricity network", "view `power`", "view `zoning`", "view `traffic`",
    "view `ledger`",
)

#: How much of the frame a framed icon's glyph occupies. The first run measured the
#: framed views at 0.72-0.74 ink coverage at 24 px: the border was carrying the icon
#: and the glyph inside it had nowhere to be. The border says "this is a view"; the
#: glyph is what the view is of, and at 24 px the glyph has to win.
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
            box(bpy, f"{name} frame {index}", materials["identity"], dx, dz, w, h,
                depth=0.26)
        )
    return parts


def build(bpy, entry, materials):
    """Build one icon's geometry, framed if it is a view."""
    parts = []
    for index, part in enumerate(entry["parts"]):
        role = part.get("role", "ink")
        kwargs = {k: v for k, v in part.items() if k not in ("shape", "role")}
        maker = {"box": box, "ring": ring}[part["shape"]]
        parts.append(maker(bpy, f"{entry['id']} {index}", materials[role], **kwargs))

    if entry.get("framed"):
        for obj in parts:
            obj.location.x *= FRAMED_GLYPH_SCALE
            obj.location.z *= FRAMED_GLYPH_SCALE
            obj.scale.x *= FRAMED_GLYPH_SCALE
            obj.scale.z *= FRAMED_GLYPH_SCALE
        parts.extend(frame(bpy, materials, entry["id"]))
    return parts


# ---------------------------------------------------------------------------
# Generations, for the review loop
# ---------------------------------------------------------------------------

#: Three declared generations of the same icon. A generation is **not** random: it
#: is a transform — scale, stroke weight, and a material treatment — so that when a
#: choice is recorded, the *reason* travels with it and the winner can become the
#: house style instead of one lucky render.
TREATMENTS = [
    {
        "id": "a-authored",
        "label": "as authored",
        "scale": 1.0,
        "weight": 1.0,
        "material": "plain",
        "why": "the geometry as written, proportions tuned for the largest size",
    },
    {
        "id": "b-bold",
        "label": "bold",
        "scale": 1.06,
        "weight": 1.35,
        "material": "plain",
        "why": "heavier strokes and a slightly larger glyph: the silhouette the "
               "smallest size can still read",
    },
    {
        "id": "c-detailed",
        "label": "detailed",
        "scale": 0.96,
        "weight": 0.82,
        "material": "treated",
        "why": "finer strokes with a material treatment on the identity parts: "
               "more surface at 96 px, less at 24",
    },
]


def apply_treatment(entry, treatment):
    """The same icon drawn at one of the three declared weights.

    Copies rather than mutates: two generations of one icon are compared side by
    side, so neither may write over the other.
    """
    parts = []
    for part in entry["parts"]:
        part = dict(part)
        if part["shape"] == "box":
            w, h = part["w"] * treatment["scale"], part["h"] * treatment["scale"]
            # Stroke weight scales the thin dimension, which is what "heavier"
            # means for a bar.
            if w < h:
                w *= treatment["weight"]
            else:
                h *= treatment["weight"]
            part["w"], part["h"] = w, h
        else:
            part["major"] *= treatment["scale"]
            part["minor"] *= treatment["scale"] * treatment["weight"]
        parts.append(part)

    materials = {}
    for role, parameters in entry["materials"].items():
        parameters = dict(parameters)
        if treatment["material"] == "treated":
            if role == "identity":
                parameters["base_metalness"] = max(parameters.get("base_metalness", 0.0), 0.55)
                parameters["specular_roughness"] = min(
                    parameters.get("specular_roughness", 0.5), 0.34
                )
            else:
                parameters["specular_roughness"] = (
                    parameters.get("specular_roughness", 0.5) * 0.8
                )
        materials[role] = parameters

    treated = dict(entry)
    treated["parts"] = parts
    treated["materials"] = materials
    treated["generation"] = treatment["id"]
    treated["generation_label"] = treatment["label"]
    treated["generation_why"] = treatment["why"]
    return treated


def generations_of(entry):
    """Every generation of one icon, in declared order."""
    return [apply_treatment(entry, treatment) for treatment in TREATMENTS]


def swatch_geometry(bpy, material):
    """A flat plate facing the camera, for measuring a material under the same rig.

    This is how the material check stops being a model. Two earlier versions were
    wrong in opposite directions: one used an invented irradiance model and called
    two correct icons failures, the other compared against the albedo with no
    lighting term and called *every* icon a failure. Neither survived contact with
    the rig, so the reference is now measured rather than reasoned about — same
    lights, same camera, same material, flat plate.
    """
    return [box(bpy, "swatch plate", material, 0.0, 0.0, 1.5, 1.5, depth=0.06)]
