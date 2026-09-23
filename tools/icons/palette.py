"""The colour algebra, over a declaration this file no longer owns.

The icons must agree with the surfaces they locate, and the only way to be sure of
that is to derive them from the same numbers rather than to copy them. Every value
here comes from `tools/materials/declare.py`, which `src/materials/generated.rs` is
also generated from — so the icons and the game read **one** table, and the old
transcription of `src/hud.rs` is gone rather than kept in step by a hash.

What lives here is the algebra: OKLCH to linear sRGB, the gamut search, the matrix
resolution, and the contrast a component owes. The declaration is data and lives in
one file; this one computes with it.
"""

import hashlib
import os
import sys

# The declaration is one directory over, so the generator, the icon pipeline and
# this file can never disagree about where the values come from.
_MATERIALS = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "materials")
if _MATERIALS not in sys.path:
    sys.path.insert(0, _MATERIALS)

import declare

# ---------------------------------------------------------------------------
# OKLab -> linear sRGB (the same algebra as hud.rs::oklch)
# ---------------------------------------------------------------------------


def _oklch_raw(lightness: float, chroma: float, hue_degrees: float):
    """OKLCH to linear sRGB **without** clamping, so a gamut miss is visible."""
    import math

    hue = math.radians(hue_degrees)
    a = chroma * math.cos(hue)
    b = chroma * math.sin(hue)

    l_ = lightness + 0.39633778 * a + 0.21580376 * b
    m_ = lightness - 0.10556135 * a - 0.06385417 * b
    s_ = lightness - 0.08948418 * a - 1.2914855 * b

    l3, m3, s3 = l_ ** 3, m_ ** 3, s_ ** 3

    r = 4.0767417 * l3 - 3.3077116 * m3 + 0.23096993 * s3
    g = -1.268438 * l3 + 2.6097574 * m3 - 0.3413194 * s3
    b_ = -0.00419609 * l3 - 0.7034186 * m3 + 1.7076147 * s3

    return (r, g, b_)


def oklch(lightness: float, chroma: float, hue_degrees: float):
    """OKLCH to **linear** sRGB, which is what a renderer wants."""
    return tuple(min(max(channel, 0.0), 1.0) for channel in _oklch_raw(lightness, chroma, hue_degrees))


def out_of_gamut(lightness: float, chroma: float, hue_degrees: float) -> tuple:
    """Which channels of a triple fall outside linear sRGB.

    A declared colour the conversion had to clamp is not the colour that was
    declared, and a check that silently accepted it would be measuring a different
    material from the one in the record. So the matrix is asked this question at
    import time instead.
    """
    return tuple(
        index for index, channel in enumerate(_oklch_raw(lightness, chroma, hue_degrees))
        if channel < -1e-9 or channel > 1.0 + 1e-9
    )


# ---------------------------------------------------------------------------
# The declared values, imported rather than transcribed
# ---------------------------------------------------------------------------

#: The colours this pipeline measures against, declared once in
#: `tools/materials/declare.py` and emitted for the game from the same source.
TOKENS = dict(declare.SHARED_TOKENS)
TOKENS.update(declare.ICON_COLOURS)
#: Derived in `hud.rs` from Nature with alpha; declared here as a name so an icon
#: brief may still refer to it without carrying a second value.
TOKENS["Powered"] = None
#: The authored icon materials are the matrix below, derived from the families
#: rather than listed, and the old role names are aliases into it, so one value has
#: one home. What survives from those roles is the part that was measured rather
#: than guessed: Galaxy-era metal reads as a light cool object against a dark panel,
#: and the earlier 0.62 body token rendered like charcoal under the fixed rig.


# ---------------------------------------------------------------------------
# The material matrix: seven families, seven declared hue anchors
# ---------------------------------------------------------------------------
#
# An icon is a silhouette from Material Design 1, three main materials whose
# depicted surface and colour intent come from TouchWiz, and one accenting record
# piece in the iOS 6 manner. That leaves one open question for every icon: *which
# three materials*. This table is the answer space, and it is bounded on purpose.
# Every material an icon may use is a named family at a named hue, so "the icon is
# made of a metal" and "the icon is made of a hue nobody declared" can be told
# apart by a script.
#
# The anchors are DECLARED, and each records what backs it. Two of the seven are
# slots: positions the hue ring needs and neither reference supports. Saying so is
# the point — an anchor invented to fill a slot would carry a measurement's
# authority with none of its evidence.

HUE_ANCHORS = declare.HUE_ANCHORS

#: The seven physical families, at the study's own character. The pack's median
#: saturation is 0.53-0.92 and its median value 0.69-0.92: TouchWiz colour is
#: saturated and bright, which is what the pre-matrix bodies (chroma 0.018) were
#: not, and what let a metal icon render as charcoal under the fixed rig.
#:
#: `variant_chroma` is the number the matrix turns on. A hue-varied body needs more
#: chroma than a neutral one or rotating the hue changes nothing — at the old 0.018
#: the seven "hues" of a metal are the same colour seven times. `natural` is the
#: family's own untinted body, and it is what the old role tokens became.
#: The icon set's seven families. The world's extra families are declared beside
#: them and deliberately **not** merged in: `ICON_STANDARD.md` states seven as a
#: property of the icon set, so a material the world needs cannot widen what an icon
#: may be made of.
FAMILIES = declare.ICON_FAMILIES

#: The world's own families: declared, resolvable, and not available to an icon.
WORLD_FAMILIES = declare.WORLD_FAMILIES

#: How colour physically arrives, per family -- and the most chroma that mechanism
#: can carry. This is the difference between tinting a material and pretending a
#: material is a pigment: a metal body's colour comes from an oxide film on its
#: surface, so a *blue metal* is not a metal, it is a coating, and the coating is
#: a different family. Asking for more chroma than a family's ceiling is therefore
#: not asking for a stronger colour, it is asking for the wrong material, and
#: `within_ceiling` says so instead of quietly producing a saturated metal.
#:
#: The numbers are the family's declared `variant_chroma`, and `colour_mechanisms()`
#: refuses a table where the two disagree -- one value, one home, and the ceiling
#: cannot drift away from the chroma that is actually used.
FAMILY_COLOUR = declare.FAMILY_COLOUR

#: Every family the world may name, icon or world-only. The matrix below resolves
#: the icon set; the world's entries are resolved for the generator.
ALL_FAMILIES = {**declare.ICON_FAMILIES, **declare.WORLD_FAMILIES}


#: A level is a declared lightness offset on the family's own lightness, so a body
#: and its edge are one material read twice rather than two materials. The offset is
#: applied and then held inside these bounds: an edge on a family already at L 0.90
#: would otherwise sit at exactly 1.00, where the only colour available is pure
#: white and the family's whole identity is lost.
MATERIAL_LEVELS = declare.MATERIAL_LEVELS
MATERIAL_LIGHTNESS_BOUNDS = declare.MATERIAL_LIGHTNESS_BOUNDS

#: What the anchors and the families were derived from, recorded so the palette hash
#: covers the provenance and not just the numbers.
REFERENCE_SAMPLING = declare.REFERENCE_SAMPLING

HUE_ANCHOR_NAMES = declare.HUE_ANCHOR_NAMES


def _anchor_degrees(hue: str) -> float:
    if hue == "natural":
        raise KeyError("`natural` carries the family's own hue, not an anchor")
    for name, degrees, _backing in HUE_ANCHORS:
        if name == hue:
            return degrees
    raise KeyError(f"`{hue}` is not a declared hue anchor: {', '.join(HUE_ANCHOR_NAMES)}")


def _material_value(family: str, hue: str, level: str, table: dict | None = None) -> tuple:
    """Resolve one entry. `table` widens the families that may be named, which is
    how a **world** family is resolved without joining the icon matrix."""
    table = FAMILIES if table is None else table
    if family not in table:
        raise KeyError(f"`{family}` is not a declared family: {', '.join(sorted(table))}")
    if level not in MATERIAL_LEVELS:
        raise KeyError(f"`{level}` is not a declared level: {', '.join(MATERIAL_LEVELS)}")
    if hue not in HUE_ANCHOR_NAMES:
        raise KeyError(f"`{hue}` is not a declared hue: {', '.join(HUE_ANCHOR_NAMES)}")
    lightness, variant_chroma, natural_chroma, natural_hue, _note = table[family]
    low, high = MATERIAL_LIGHTNESS_BOUNDS
    value = min(max(lightness + MATERIAL_LEVELS[level], low), high)
    if hue == "natural":
        degrees, chroma = natural_hue, natural_chroma
    else:
        degrees, chroma = _anchor_degrees(hue), variant_chroma
    return (value, _bounded_chroma(value, chroma, degrees), degrees)


def _bounded_chroma(lightness: float, requested: float, degrees: float,
                    steps: int = 40) -> float:
    """The largest chroma at or below `requested` that this lightness can hold.

    Floored rather than rounded, and floored *after* the search: rounding up by half
    a ten-thousandth is enough to put an entry back outside the gamut it was just
    measured inside, which is how the first version of this helper reported 26
    misses while believing it had solved them.
    """
    import math

    if not out_of_gamut(lightness, requested, degrees):
        return requested
    low, high = 0.0, requested
    for _ in range(steps):
        middle = (low + high) / 2.0
        if out_of_gamut(lightness, middle, degrees):
            high = middle
        else:
            low = middle
    return math.floor(low * 1e4) / 1e4


def chromatic_adjustments() -> dict:
    """Every entry whose chroma had to be reduced to reach linear sRGB.

    Recorded rather than silently applied: the declared cap and the achieved value
    are both true, and the difference between them is the interesting part.
    """
    out = {}
    for name, (lightness, chroma, degrees) in sorted(MATERIALS.items()):
        family, hue = name.split(":")[0], name.split(":")[1]
        requested = (FAMILIES[family][2] if hue == "natural" else FAMILIES[family][1])
        if abs(chroma - requested) > 1e-9:
            out[name] = {"declared_cap": requested, "in_gamut": chroma}
    return out


def material_name(family: str, hue: str, level: str = "body") -> str:
    """The token name of one matrix entry, e.g. `metal:amber:edge`."""
    _material_value(family, hue, level)
    return f"{family}:{hue}" if level == "body" else f"{family}:{hue}:{level}"


def material_value(family: str, hue: str, level: str = "body") -> tuple:
    """One icon matrix entry's OKLCH triple."""
    return _material_value(family, hue, level)


def world_material_value(family: str, hue: str, level: str = "body") -> tuple:
    """One **world** family entry's OKLCH triple, resolved the same way.

    A world family is resolvable but is not part of the icon matrix, so the icon set
    stays exactly seven while the world may name water, organic matter and soil.
    """
    return _material_value(family, hue, level, table=ALL_FAMILIES)


MATERIALS = {
    material_name(family, hue, level): _material_value(family, hue, level)
    for family in FAMILIES
    for hue in HUE_ANCHOR_NAMES
    for level in MATERIAL_LEVELS
}

#: The old role names, kept so an existing recipe still resolves and each one an
#: alias into the matrix rather than a second copy of a value.
MATERIAL_ALIASES = declare.MATERIAL_ALIASES

TOKENS.update(MATERIALS)
for _alias, _target in MATERIAL_ALIASES.items():
    TOKENS[_alias] = MATERIALS[_target]


def colour_mechanisms() -> dict:
    """The declared mechanism per family, checked against the chroma actually used.

    Empty when the two agree. A non-empty result means the file is claiming one
    ceiling and rendering another, which is the silent-drop defect class: the
    manifest would record a mechanism whose number the matrix does not honour.
    """
    return {
        family: {
            "mechanism": FAMILY_COLOUR[family][0],
            "declared_ceiling": FAMILY_COLOUR[family][1],
            "variant_chroma_in_use": FAMILIES[family][1],
        }
        for family in sorted(FAMILIES)
        if family not in FAMILY_COLOUR or FAMILY_COLOUR[family][1] != FAMILIES[family][1]
    }


def within_ceiling(family: str, chroma: float) -> bool:
    """Whether a chroma is still the family's own colour mechanism."""
    if family not in FAMILY_COLOUR:
        raise KeyError(
            f"`{family}` has no declared colour mechanism; declared: "
            f"{', '.join(sorted(FAMILY_COLOUR))}"
        )
    return chroma <= FAMILY_COLOUR[family][1] + 1e-9


def hue_warrants() -> dict:
    """Each anchor and what backs it, so a hue can be *warranted* rather than picked.

    The game's own declared tokens come first: where a surface already has a hue in
    the interface, the icon restates a meaning rather than inventing one. The two
    declared slots are recorded as slots -- which is what lets a hue chosen to fill
    one be reported as an author choice instead of borrowing a measurement's
    authority.
    """
    return {name: backing for name, _degrees, backing in HUE_ANCHORS}


def gamut_misses() -> list:
    """Every declared entry whose conversion to linear sRGB had to be clamped."""
    return [
        name
        for name, value in sorted(MATERIALS.items())
        if value is not None and out_of_gamut(*value)
    ]


def material_contrasts() -> dict:
    """Each family's natural body against every declared host.

    Recorded rather than gated: the 3:1 floor is owed by an *icon* against the
    surface it declares, and the 75th percentile of a rendered glyph is not the flat
    value of its material. This is the number that says whether a family is
    plausible on a host before any geometry exists.
    """
    out = {}
    for family in sorted(FAMILIES):
        rgb = linear(f"{family}:natural")
        out[family] = {
            host: round(contrast_ratio(relative_luminance(rgb), host_luminance(host)), 2)
            for host in HOSTS
        }
    return out


def reference_digest() -> str:
    """A digest of the sampling population: the files the anchors were read from.

    Names and sizes rather than pixel bytes: the population is what a re-derivation
    has to find again, and this makes a changed or re-fetched pack visible without
    hashing three megabytes of someone else's artwork.
    """
    import os

    root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    population = os.path.join(root, "assets", "reference", "galaxy-s4-icon-pack",
                              "res", "drawable")
    if not os.path.isdir(population):
        return "absent"
    digest = hashlib.sha256()
    for name in sorted(os.listdir(population)):
        if not name.lower().endswith(".png"):
            continue
        digest.update(name.encode())
        digest.update(str(os.path.getsize(os.path.join(population, name))).encode())
    return digest.hexdigest()[:16]


def matrix_record() -> dict:
    """The matrix as the manifest states it: anchors, families, provenance."""
    return {
        "anchors": [
            {"hue": name, "degrees": degrees, "backing": backing}
            for name, degrees, backing in HUE_ANCHORS
        ],
        "families": {
            family: {
                "lightness": values[0],
                "variant_chroma": values[1],
                "natural_chroma": values[2],
                "natural_hue": values[3],
                "note": values[4],
            }
            for family, values in sorted(FAMILIES.items())
        },
        "levels": MATERIAL_LEVELS,
        "aliases": MATERIAL_ALIASES,
        "lightness_bounds": MATERIAL_LIGHTNESS_BOUNDS,
        "entries": len(MATERIALS),
        "reference": REFERENCE_SAMPLING,
        "reference_digest": reference_digest(),
        "gamut_misses": gamut_misses(),
        "chromatic_adjustments": chromatic_adjustments(),
        "natural_contrasts": material_contrasts(),
    }


def linear(token: str):
    """A token's linear RGB."""
    if token == "Powered":
        return oklch(*TOKENS["Nature"])
    if token not in TOKENS:
        raise KeyError(f"no palette entry for token `{token}`; add it before using it")
    return oklch(*TOKENS[token])


def rgba(token: str, alpha: float = 1.0):
    """A token as the four floats the renderer takes."""
    linear_rgb = linear(token)
    return (linear_rgb[0], linear_rgb[1], linear_rgb[2], alpha)


# ---------------------------------------------------------------------------
# Hosts, and the contrast a non-text component owes
# ---------------------------------------------------------------------------

#: The surfaces an icon is allowed to sit on, as the tokens that fill them. An icon
#: must declare which of these it lands on, and the pipeline measures it against
#: each one: a glyph that passes every other check and is invisible on the panel it
#: was put on has still failed.
#:
#: Only surfaces that exist today are listed. `Ground` and `Water` are deliberately
#: absent: the world palette is not in `hud.rs` yet, so their values would have to
#: be invented here, and an invented value is exactly what this file exists to
#: avoid.
HOSTS = ("Desk", "Panel", "PanelRaised")

#: WCAG 2.2 1.4.11 Non-text Contrast, AA: a UI component needs 3:1 against what it
#: sits on. Body text's 4.5:1 floor is measured separately, in `src/hud.rs`.
NON_TEXT_MIN_CONTRAST = 3.0


def relative_luminance(linear_rgb) -> float:
    """WCAG relative luminance of a **linear** RGB triple."""
    return 0.2126 * linear_rgb[0] + 0.7152 * linear_rgb[1] + 0.0722 * linear_rgb[2]


def contrast_ratio(a: float, b: float) -> float:
    """WCAG contrast between two relative luminances."""
    lighter, darker = max(a, b), min(a, b)
    return (lighter + 0.05) / (darker + 0.05)


def host_luminance(token: str) -> float:
    """The relative luminance of a host surface's own fill."""
    if token not in HOSTS:
        raise KeyError(
            f"`{token}` is not a declared host; an icon may only sit on a surface "
            f"this file can measure it against ({', '.join(HOSTS)})"
        )
    return relative_luminance(linear(token))


def palette_hash() -> str:
    """A stable digest of every value in the table, and of the matrix's provenance.

    Recorded in the manifest so "the icons were re-rendered after the palette
    moved" is checkable rather than asserted. The anchors, the families and the
    sampling record are included because moving an anchor changes every material
    derived from it — a hash that covered only the resulting numbers would make the
    same change invisible.
    """
    import json

    digest = hashlib.sha256()
    for name in sorted(TOKENS):
        value = TOKENS[name]
        digest.update(name.encode())
        if value is not None:
            digest.update(repr(value).encode())
    digest.update(json.dumps({
        "anchors": HUE_ANCHORS,
        "families": {family: values for family, values in sorted(FAMILIES.items())},
        "levels": MATERIAL_LEVELS,
        "aliases": MATERIAL_ALIASES,
        "sampling": REFERENCE_SAMPLING,
        "reference_digest": reference_digest(),
    }, sort_keys=True, default=str).encode())
    return digest.hexdigest()[:16]


def source_hash(path: str) -> str:
    """A digest of the authority this table transcribes."""
    with open(path, "rb") as handle:
        return hashlib.sha256(handle.read()).hexdigest()[:16]
