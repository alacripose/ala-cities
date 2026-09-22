"""The interface's colours, recomputed for the renderer.

The icons must agree with the surfaces they locate, and the only way to be sure
of that is to derive them from the same numbers rather than to copy them. Every
value here is the OKLCH triple that `src/hud.rs` carries for the same token, run
through the same OKLab conversion, so a token change that is not followed by a
re-render shows up as a palette hash mismatch in the manifest.

`src/hud.rs` is the authority; this file is a transcription of it, and
`tests.rs`-style duplication is avoided by hashing the transcription into the
manifest so the two can be compared by eye or by script.
"""

import hashlib

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
# The tokens these icons borrow from, verbatim from src/hud.rs
# ---------------------------------------------------------------------------

TOKENS = {
    # name: (l, c, h) exactly as the style table builds them
    "Desk": (0.17, 0.012, 260.0),
    "Panel": (0.25, 0.014, 260.0),
    "Ink": (0.45, 0.130, 250.0),
    "Nature": (0.70, 0.150, 150.0),
    "Warning": (0.74, 0.160, 70.0),
    "Refused": (0.62, 0.180, 25.0),
    "NotObtained": (0.55, 0.020, 260.0),
    "TextBody": (0.96, 0.005, 260.0),
    "TextMuted": (0.80, 0.010, 260.0),
    "PanelRaised": (0.31, 0.016, 260.0),
    "Plaque": (0.42, 0.030, 80.0),
    "Road": (0.42, 0.005, 260.0),
    "RoadEdge": (0.52, 0.005, 260.0),
    "Scaffold": (0.60, 0.090, 80.0),
    "ZoneResidential": (0.55, 0.130, 150.0),
    "ZoneCommercial": (0.55, 0.130, 250.0),
    "ZoneIndustrial": (0.55, 0.130, 60.0),
    "CaseOpen": (0.74, 0.160, 70.0),
    "Powered": None,  # derived from Nature with alpha in hud.rs
    "Retired": (0.65, 0.020, 260.0),
    "Agent": (0.92, 0.020, 260.0),
    # The authored icon materials no longer live here. They are the matrix below,
    # derived from the families rather than listed, and the old role names are
    # aliases into it, so one value has one home. What survives from those roles is
    # the part that was measured rather than guessed: Galaxy-era metal reads as a
    # light cool object against a dark panel, and the earlier 0.62 body token
    # rendered like charcoal under the fixed rig.
    "IconBlack": (0.16, 0.018, 250.0),
}


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

HUE_ANCHORS = (
    ("red", 25.0, "backed by Token::Refused (25) — destructive"),
    ("amber", 62.0, "backed by ZoneIndustrial (60) / Warning (70) / Scaffold (80) — caution, power"),
    ("yellow", 115.0, "declared slot — no reference and no game meaning occupies it"),
    ("green", 150.0, "backed by Nature / ZoneResidential (150) — nature, residential"),
    ("cyan", 200.0, "partially backed — the study's people (162.6) and store (201.8) classes; no game token"),
    ("blue", 250.0, "backed by Ink / ZoneCommercial (250) — data, commercial"),
    ("violet", 300.0, "declared slot — no reference and no game meaning occupies it"),
)

#: The seven physical families, at the study's own character. The pack's median
#: saturation is 0.53-0.92 and its median value 0.69-0.92: TouchWiz colour is
#: saturated and bright, which is what the pre-matrix bodies (chroma 0.018) were
#: not, and what let a metal icon render as charcoal under the fixed rig.
#:
#: `variant_chroma` is the number the matrix turns on. A hue-varied body needs more
#: chroma than a neutral one or rotating the hue changes nothing — at the old 0.018
#: the seven "hues" of a metal are the same colour seven times. `natural` is the
#: family's own untinted body, and it is what the old role tokens became.
FAMILIES = {
    # family:   (lightness, variant_chroma, natural_chroma, natural_hue, note)
    "metal":   (0.78, 0.060, 0.018, 250.0, "the light cool object the study's chrome reads as"),
    "paper":   (0.88, 0.060, 0.035, 82.0, "matte warm sheet"),
    "ceramic": (0.90, 0.050, 0.025, 72.0, "glazed insulator"),
    "glass":   (0.66, 0.100, 0.090, 220.0, "lens and pane"),
    "polymer": (0.34, 0.050, 0.018, 250.0, "dark grip"),
    "road":    (0.38, 0.040, 0.018, 250.0, "matte surface"),
    "enamel":  (0.62, 0.190, 0.190, 245.0, "the study's blue control face; coated colour"),
}

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
FAMILY_COLOUR = {
    "metal": (
        "anodised film: an oxide layer on the substrate, so a metal body stays a metal",
        0.060,
    ),
    "glass": (
        "body-tinted: colour held in the glass itself, which also tints what shows through it",
        0.100,
    ),
    "ceramic": (
        "fired glaze: a mineral glaze over the body, opaque rather than transparent",
        0.050,
    ),
    "polymer": (
        "pigmented resin: colour compounded into the plastic itself",
        0.050,
    ),
    "paper": (
        "dyed stock: pigment in the sheet, which is why it stays matte and pale",
        0.060,
    ),
    "road": (
        "aggregate: asphalt and stone, near-neutral because that is what it is made of",
        0.040,
    ),
    "enamel": (
        "painted colour: a pigmented coating over a substrate -- the family paint belongs to",
        0.190,
    ),
}


#: A level is a declared lightness offset on the family's own lightness, so a body
#: and its edge are one material read twice rather than two materials. The offset is
#: applied and then held inside these bounds: an edge on a family already at L 0.90
#: would otherwise sit at exactly 1.00, where the only colour available is pure
#: white and the family's whole identity is lost.
MATERIAL_LEVELS = {"body": 0.0, "edge": 0.10, "deep": -0.18}
MATERIAL_LIGHTNESS_BOUNDS = (0.05, 0.97)

#: What the anchors and the families were derived from, recorded so the palette hash
#: covers the provenance and not just the numbers.
REFERENCE_SAMPLING = {
    "pack": "assets/reference/galaxy-s4-icon-pack",
    "population": "res/drawable/*.png — the TouchWiz-native raster icons",
    "population_files": 125,
    "population_note": (
        "the APK also carries Material Components resources, which are not the "
        "artistic-intent family; measured, the two populations give identical "
        "anchors, so this is a provenance statement rather than a correction"
    ),
    "method": (
        "per-class circular mean hue, weighted by saturation x value, over opaque "
        "pixels with saturation above 0.25 and value above 0.15"
    ),
    "measured_class_hues": {
        "people": 162.6, "message": 34.5, "media": 226.2,
        "system": 131.4, "store": 201.8, "nature_health": 193.0,
    },
    "finding": (
        "the study's classes occupy a 95 degree arc from green through cyan to blue "
        "with one orange outlier: it contains no red, no yellow and no violet, so "
        "seven anchors cannot be sampled from it. Four are backed by the game's own "
        "declared hues, one is partial, and two are declared slots"
    ),
    "character": {
        "median_saturation": (0.53, 0.92),
        "median_value": (0.69, 0.92),
        "reading": (
            "TouchWiz colour is saturated and bright, so variant chroma follows that "
            "rather than the near-neutral bodies the first pass used"
        ),
    },
}

HUE_ANCHOR_NAMES = tuple(name for name, _degrees, _backing in HUE_ANCHORS) + ("natural",)


def _anchor_degrees(hue: str) -> float:
    if hue == "natural":
        raise KeyError("`natural` carries the family's own hue, not an anchor")
    for name, degrees, _backing in HUE_ANCHORS:
        if name == hue:
            return degrees
    raise KeyError(f"`{hue}` is not a declared hue anchor: {', '.join(HUE_ANCHOR_NAMES)}")


def _material_value(family: str, hue: str, level: str) -> tuple:
    if family not in FAMILIES:
        raise KeyError(f"`{family}` is not a declared family: {', '.join(sorted(FAMILIES))}")
    if level not in MATERIAL_LEVELS:
        raise KeyError(f"`{level}` is not a declared level: {', '.join(MATERIAL_LEVELS)}")
    if hue not in HUE_ANCHOR_NAMES:
        raise KeyError(f"`{hue}` is not a declared hue: {', '.join(HUE_ANCHOR_NAMES)}")
    lightness, variant_chroma, natural_chroma, natural_hue, _note = FAMILIES[family]
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
    """One matrix entry's OKLCH triple."""
    return _material_value(family, hue, level)


MATERIALS = {
    material_name(family, hue, level): _material_value(family, hue, level)
    for family in FAMILIES
    for hue in HUE_ANCHOR_NAMES
    for level in MATERIAL_LEVELS
}

#: The old role names, kept so an existing recipe still resolves and each one an
#: alias into the matrix rather than a second copy of a value.
MATERIAL_ALIASES = {
    "MetalBody": "metal:natural",
    "MetalEdge": "metal:natural:edge",
    "PaperBody": "paper:natural",
    "CeramicInsulator": "ceramic:natural",
    "GlassLens": "glass:natural",
    "PolymerGrip": "polymer:natural",
    "RoadSurface": "road:natural",
    "IconBlue": "enamel:natural",
}

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
