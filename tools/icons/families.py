"""The six object families, declared once (C8 a179–a199).

The ladder is the axis. λ = 0 is md1 (the mark is the object), λ = 0.5 is TouchWiz,
λ = 1 is iOS 6, and every family's parameters move along it, so a candidate at any
point is a legitimate object rather than a labelled variant. `vector(family, lam)`
is the only way geometry gets its numbers: nothing downstream picks a variant by
name.

Three rules this file exists to enforce, each of them a settled decision rather than
a preference:

* **The reference object is measured, and the measurement is recorded beside the
  endpoint** (a188/a192). Where the reference gives a number — the settings gear's
  six teeth, the magnifier's 0.51 bore, the bin's 0.68 taper — the md1 endpoint
  *is* that number and says so. Where it does not, the endpoint is marked authored,
  because an authored number presented as a measurement is the one thing this
  pipeline refuses.
* **An integer count changes by a feature growing from zero width** (a187). A count
  parameter interpolates as a real number and `features()` splits it into whole
  features plus one partial one, whose angular or linear width is the fraction — so
  nothing switches on at a threshold and `floor` is a reading, not a decision.
* **A parameter that the object's identity pins is `fixed`**, with the measurement
  that pins it. The ticket is the same ticket at every point on the ladder; what
  moves is its detailing, and pretending otherwise would be inventing variation.

`check()` is the gate on this table: a family without a reference, a parameter
without a source, or a count whose endpoints are not whole numbers all fail it.
"""

#: How wide a family's fill envelope is, as a fraction of its own measurement.
#:
#: a201 makes each family's envelope its **own** reference measurement rather than the
#: corpus's interquartile range, because that band is a *population* statistic and
#: four of the six reference objects fall outside it -- the road (0.283) and the lens
#: (0.281) below its 0.366 floor, the bin (0.795) and the ticket (0.913) above its
#: 0.706 ceiling. An object is what it is; the band says what the *catalogue* is.
#:
#: One reference mark gives one number, so the spread has to come from somewhere that
#: is not taste: it is the md1 population's own relative spread (median 0.490, IQR
#: 0.366-0.706, so a half-width of 0.170 over a median of 0.490). Applied to each
#: object's own measurement, it is the same claim the ladder already makes -- "this is
#: what that archive's marks look like" -- read at the size of one object instead of
#: 1503 of them.
POPULATION_RELATIVE_SPREAD = 0.347

#: The composition budget (a200): what the object occupies of the frame the rig's
#: camera sees, and what is left for a separate accent piece. Declared here rather
#: than discovered in a render, because a fixed accent position collided with the
#: road at three lanes and up and welded the two into one piece.
#:
#: `body_span` is deliberately smaller than the 1.72 the traced marks occupied: with
#: 1.44 the margin is 0.405 a side, which leaves room for an accent of radius 0.15
#: plus its clearance at every family and every point on the ladder.
COMPOSITION = {
    "frame_span": 2.25,
    "body_span": 1.44,
    "accent_radius": 0.15,
    "clearance": 0.06,
    "note": (
        "the accent sits outside the body's own bounds by the declared clearance, so "
        "it cannot interpenetrate (a183) or pinch a sliver against it (a196); the "
        "placement is derived from the body's bounds because a fixed one collides"
    ),
}

#: What each family is, which icon it belongs to, and the mark it was read from.
#: `measured` holds the numbers `tools/icons/_measure_marks.py` printed, so an
#: endpoint's provenance can be checked against the artefact rather than trusted.
FAMILIES = {
    "gear": {
        "icon": "vocab-settings",
        "object": "a toothed control disc",
        "reference": {
            "glyph": "action/settings",
            "measured": {"teeth": 6, "bore_ratio": 0.39, "pieces": 1, "voids": 1,
                         "fill": 0.5665},
            "note": "6 runs at 0.80 and 0.92 of R; one 646 px void ≈ 0.39 R",
        },
        "parameters": (
            {"name": "teeth", "type": "count", "md1": 6, "ios6": 12,
             "unit": "whole teeth",
             "source": "measured: 6 runs at 0.80 and 0.92 of R in the reference"},
            {"name": "tooth_duty", "type": "continuous", "md1": 0.46, "ios6": 0.28,
             "unit": "fraction of the pitch",
             "source": "authored: the reference's land fraction is not separable "
                       "from its own antialiasing"},
            {"name": "root_ratio", "type": "continuous", "md1": 0.76, "ios6": 0.84,
             "unit": "fraction of the outer radius", "source": "authored"},
            {"name": "tip_rise", "type": "continuous", "md1": 0.20, "ios6": 0.10,
             "unit": "fraction of the tooth height", "source": "authored"},
            {"name": "bore_ratio", "type": "continuous", "md1": 0.39, "ios6": 0.22,
             "unit": "fraction of the outer radius",
             "source": "measured: the reference's 646 px void ≈ 0.39 R"},
            {"name": "hub_ratio", "type": "continuous", "md1": 0.55, "ios6": 0.42,
             "unit": "fraction of the outer radius", "source": "authored"},
            {"name": "chamfer_ratio", "type": "continuous", "md1": 0.10, "ios6": 0.06,
             "unit": "fraction of the tooth height", "source": "authored"},
        ),
    },
    "road": {
        "icon": "tool-road",
        "object": "a ribbon of carriageway",
        "reference": {
            "glyph": "maps/add_road",
            "measured": {"box": [76, 76], "pieces": 6, "voids": 0, "fill": 0.2825},
            "note": "the ribbon arrives in 6 components; its centre column carries "
                    "2 runs at 89 % ink",
        },
        "parameters": (
            {"name": "lanes", "type": "count", "md1": 2, "ios6": 4,
             "unit": "whole lanes",
             "source": "authored: the reference does not separate its lanes, so "
                       "there is nothing to measure"},
            {"name": "lane_width", "type": "continuous", "md1": 0.14, "ios6": 0.20,
             "unit": "fraction of the ribbon span",
             "source": "measured: the reference's carriageway is ~21 px of its 76 px "
                       "box, so two lanes are 0.28 of the span between them. The "
                       "declared 0.30 a lane was wrong in the direction the "
                       "envelope caught -- it made the object 0.60 of its span"},
            {"name": "centre_dash_duty", "type": "continuous", "md1": 0.55, "ios6": 0.0,
             "unit": "fraction of the dash cycle",
             "source": "authored: 0 is a solid centre line, so the *line type* is a "
                       "reading of this number rather than a switch"},
            {"name": "marking_height", "type": "continuous", "md1": 0.06, "ios6": 0.03,
             "unit": "of the ribbon depth",
             "source": "authored: raised rather than cut, because a cut dash is an "
                       "enclosed void and would be a sliver (a196)"},
            {"name": "curb_width", "type": "continuous", "md1": 0.06, "ios6": 0.0,
             "unit": "fraction of the ribbon span",
             "source": "authored: 0 is none. Kept small because a wide verge makes the "
                       "object's own box a property of its edges rather than of the "
                       "carriageway, which is what the envelope measures"},
            {"name": "junction_arms", "type": "count", "md1": 0, "ios6": 4,
             "unit": "whole arms", "source": "authored"},
            {"name": "arm_length", "type": "continuous", "md1": 0.0, "ios6": 0.72,
             "unit": "fraction of the room the budget leaves",
             "source": "authored: an arm whose length is 0 is absent, which is how "
                       "the junction count moves (a187). Read against the *margin* "
                       "the composition budget leaves rather than the ribbon's span, "
                       "because a wide four-lane carriageway has almost no verge "
                       "and an arm measured against the span overflowed the frame"},
            {"name": "corner_rounding", "type": "continuous", "md1": 0.0, "ios6": 0.14,
             "unit": "fraction of the ribbon span", "source": "authored"},
            {"name": "heading", "type": "continuous", "md1": 45.0, "ios6": 24.0,
             "unit": "degrees in the icon's own plane",
             "source": "measured in effect: the reference mark is diagonal (its box is "
                       "76x76 while its ink is 17.7 % of it, so it cannot be an "
                       "axis-aligned ribbon), which is also why its fill is 0.28"},
            {"name": "pitch", "type": "continuous", "md1": 0.0, "ios6": 38.0,
             "unit": "degrees about the carriageway's length",
             "source": "authored (a203): the fake-isometric pose the person asked "
                       "for, which is what reveals the carriageway's top face and "
                       "its side — flat at the md1 end, where the reference idiom is "
                       "a flat mark, and posed at the iOS 6 end"},
        ),
    },
    "bolt": {
        "icon": "tool-power",
        "object": "a bolt of energy",
        "reference": {
            "glyph": "content/bolt",
            "measured": {"box": [40, 72], "aspect": 0.56, "pieces": 1, "voids": 0,
                         "fill": 0.3767},
            "note": "re-pointed from action/power_settings_new (a198): a power "
                    "button asserts the switched-on state this icon forbids, and "
                    "the pylon the game means has no reference object behind it",
        },
        "parameters": (
            {"name": "width_over_height", "type": "continuous", "md1": 0.56,
             "ios6": 0.78, "unit": "aspect",
             "source": "measured: the reference's box is 40x72, aspect 0.56"},
            {"name": "strike_angle", "type": "continuous", "md1": 0.0, "ios6": 16.0,
             "unit": "degrees", "source": "authored"},
            {"name": "tip_taper", "type": "continuous", "md1": 0.35, "ios6": 0.20,
             "unit": "fraction of the width", "source": "authored"},
            {"name": "waist", "type": "continuous", "md1": 0.55, "ios6": 0.80,
             "unit": "fraction of the width", "source": "authored"},
            {"name": "stroke", "type": "continuous", "md1": 0.30, "ios6": 0.40,
             "unit": "fraction of the width", "source": "authored"},
            {"name": "steps", "type": "count", "md1": 0, "ios6": 2,
             "unit": "whole steps in the flank", "source": "authored"},
        ),
    },
    "lens": {
        "icon": "tool-inspect",
        "object": "a lens with a handle",
        "reference": {
            "glyph": "action/search",
            "measured": {"box": [69, 70], "bore_ratio": 0.51, "pieces": 1, "voids": 1,
                         "fill": 0.2812},
            "note": "one 1018 px void ≈ 0.51 R: the bore is what defines the object",
        },
        "parameters": (
            {"name": "rings", "type": "count", "md1": 1, "ios6": 3,
             "unit": "whole rings", "source": "authored"},
            {"name": "bore_ratio", "type": "continuous", "md1": 0.51, "ios6": 0.30,
             "unit": "fraction of the outer radius",
             "source": "measured: the reference's 1018 px void ≈ 0.51 R"},
            {"name": "ring_thickness", "type": "continuous", "md1": 0.06, "ios6": 0.13,
             "unit": "fraction of the outer radius", "source": "authored"},
            {"name": "glass_depth", "type": "continuous", "md1": 0.10, "ios6": 0.28,
             "unit": "of the housing depth", "source": "authored"},
            {"name": "handle_angle", "type": "continuous", "md1": 45.0, "ios6": 34.0,
             "unit": "degrees", "source": "authored"},
            {"name": "handle_length", "type": "continuous", "md1": 0.75, "ios6": 0.55,
             "unit": "fraction of the outer radius", "source": "authored"},
            {"name": "collar_width", "type": "continuous", "md1": 0.0, "ios6": 0.16,
             "unit": "fraction of the outer radius",
             "source": "authored: 0 is none, which is how the collar appears"},
        ),
    },
    "bin": {
        "icon": "tool-demolish",
        "object": "a container with a lid",
        "reference": {
            "glyph": "action/delete",
            "measured": {"box": [56, 72], "taper": 0.68, "pieces": 2, "voids": 0,
                         "lid_share": 0.17, "fill": 0.7946},
            "note": "the lid is a separate component (540 px of 3204) with a gap "
                    "between it and the body; the reference draws no ridges",
        },
        "parameters": (
            {"name": "taper", "type": "continuous", "md1": 0.68, "ios6": 0.86,
             "unit": "bottom width over widest",
             "source": "measured: the reference tapers to 0.68"},
            {"name": "lid_gap", "type": "continuous", "md1": 0.08, "ios6": 0.03,
             "unit": "fraction of the body height",
             "source": "measured: the reference's lid is a separate component"},
            {"name": "lid_lip", "type": "continuous", "md1": 0.10, "ios6": 0.16,
             "unit": "overhang beyond the body width", "source": "authored"},
            {"name": "ridges", "type": "count", "md1": 0, "ios6": 6,
             "unit": "whole ridges",
             "source": "measured: the reference draws none, so md1 is 0"},
            {"name": "rim_thickness", "type": "continuous", "md1": 0.05, "ios6": 0.09,
             "unit": "fraction of the body width", "source": "authored"},
            {"name": "handle_width", "type": "continuous", "md1": 0.22, "ios6": 0.30,
             "unit": "fraction of the lid width", "source": "authored"},
            {"name": "mouth_depth", "type": "continuous", "md1": 0.06, "ios6": 0.14,
             "unit": "fraction of the body height", "source": "authored"},
        ),
    },
    "plaque": {
        "icon": "ticket",
        "object": "a punched card",
        "reference": {
            "glyph": "notification/confirmation_number",
            "measured": {"box": [80, 64], "notches": 3, "notch_px": 64,
                         "notch_share": 0.0137, "fill": 0.9125},
            "note": "three notches of 64 px each, 1.37 % of the ink — the class the "
                    "3.5 % rule condemns, which is why the notches are drawn larger "
                    "(a199) and the containment divergence is recorded",
        },
        "parameters": (
            {"name": "aspect", "type": "fixed", "md1": 1.25, "ios6": 1.25,
             "unit": "width over height",
             "source": "measured: the reference's box is 80x64, aspect 1.25"},
            {"name": "notches", "type": "fixed", "md1": 3, "ios6": 3,
             "unit": "whole notches",
             "source": "measured: three voids of 64 px in the reference"},
            {"name": "notch_diameter", "type": "continuous", "md1": 0.46, "ios6": 0.36,
             "unit": "fraction of the card height",
             "source": "authored **above the 3.5 % floor**: the reference's own "
                       "1.37 % is condemned by a196, so the notch is drawn larger "
                       "and the containment divergence is recorded rather than "
                       "the rule softened"},
            {"name": "rules", "type": "count", "md1": 3, "ios6": 1,
             "unit": "whole rules drawn on the card", "source": "authored"},
            {"name": "stub_width", "type": "continuous", "md1": 0.30, "ios6": 0.44,
             "unit": "fraction of the card width", "source": "authored"},
            {"name": "tab_width", "type": "continuous", "md1": 0.0, "ios6": 0.18,
             "unit": "fraction of the card width",
             "source": "authored: 0 is no tab"},
            {"name": "corner_rounding", "type": "continuous", "md1": 0.0, "ios6": 0.07,
             "unit": "fraction of the card height", "source": "authored"},
        ),
    },
}

#: Which family every reviewed icon's object comes from. `tool-zone` is absent
#: because its locator is still deferred (a166), and `ledger` because it has no mark.
ICON_FAMILY = {declaration["icon"]: name for name, declaration in FAMILIES.items()}


def declaration(family: str) -> dict:
    if family not in FAMILIES:
        raise KeyError(
            f"`{family}` is not a declared family: {', '.join(sorted(FAMILIES))}"
        )
    return FAMILIES[family]


def family_of(icon_id: str) -> str:
    """The family an icon's object belongs to, by icon id."""
    if icon_id not in ICON_FAMILY:
        raise KeyError(f"`{icon_id}` has no declared family")
    return ICON_FAMILY[icon_id]


def features(value: float) -> tuple:
    """A fractional count as `(whole, partial)` (a187).

    A count of 7.5 is seven whole features and one at half width, which is what
    "an integer count changes by a feature growing from zero width" means when it
    is written down: the partial feature's extent is the fraction, so a count of
    7.0 has no eighth feature at all rather than a threshold at which one appears.
    """
    if value < 0.0:
        raise ValueError(f"a count of {value} is negative")
    whole = int(value)
    return whole, value - whole


def vector(family: str, lam: float) -> dict:
    """The parameter vector at a point on the ladder.

    Linear in λ over each parameter's declared endpoints. Deliberately linear: a
    curve would need a reason, and the reason would have to be a measurement of
    *intermediate* objects, which no archive holds — the three anchors are the
    evidence and everything between them is an interpolation, stated as such.
    """
    if not 0.0 <= lam <= 1.0:
        raise ValueError(f"λ {lam} is outside [0, 1]")
    out = {}
    for parameter in declaration(family)["parameters"]:
        md1, ios6 = parameter["md1"], parameter["ios6"]
        if parameter["type"] == "fixed":
            out[parameter["name"]] = md1
        else:
            out[parameter["name"]] = md1 + (ios6 - md1) * lam
    return out


def fill_envelope(family: str) -> tuple:
    """The fill range a candidate of this family is judged against (a201).

    Its own reference's measurement, widened by the population's relative spread. The
    corpus band is *not* used as the gate: it is recorded by the check that the
    catalogue as a whole spans the archives, which is the claim it can actually
    support.
    """
    fill = declaration(family)["reference"]["measured"].get("fill")
    if fill is None:
        raise KeyError(f"`{family}` records no measured fill, so it has no envelope")
    spread = fill * POPULATION_RELATIVE_SPREAD
    return (round(max(0.0, fill - spread), 4), round(min(1.0, fill + spread), 4))


def accent_slot(bounds: tuple, radius: float = None, clearance: float = None) -> tuple:
    """Where a separate accent piece goes, from the body's own bounds (a200).

    Diagonally outside the body's lower-right corner by the declared clearance, so
    the accent cannot interpenetrate the body (a183 refuses that) and cannot pinch a
    sliver against it (a196). Derived rather than fixed, because a fixed position
    collided with the road at three lanes and up and the two welded into one piece.

    `bounds` is `(min_x, min_z, max_x, max_z)` in the icon's own plane.
    """
    gap = COMPOSITION["clearance"] if clearance is None else clearance
    size = COMPOSITION["accent_radius"] if radius is None else radius
    return (bounds[2] + gap + size, bounds[1] - gap - size)


def describe() -> dict:
    """The families and their provenance, for the manifest."""
    return {
        "axis": (
            "λ = 0 md1, 0.5 TouchWiz, 1 iOS 6; every parameter moves along it and "
            "nothing downstream selects a variant by name"
        ),
        "integer_counts": (
            "a count interpolates as a real number and is read as whole features "
            "plus one partial whose extent is the fraction (a187)"
        ),
        "families": {
            name: {
                "icon": item["icon"],
                "object": item["object"],
                "reference": dict(item["reference"]),
                "parameters": [dict(parameter) for parameter in item["parameters"]],
            }
            for name, item in sorted(FAMILIES.items())
        },
    }


def check() -> dict:
    """Whether the table still says what it claims to.

    A family with no reference object, a parameter with no source, a count whose
    endpoints are not whole numbers, or an icon claimed by two families all fail
    here rather than in a render nobody reads.
    """
    problems = []
    seen_icons = {}
    for name, item in sorted(FAMILIES.items()):
        if not item.get("reference", {}).get("glyph"):
            problems.append(f"`{name}` declares no reference mark")
        if not item.get("reference", {}).get("measured"):
            problems.append(f"`{name}` records no measurement of its reference")
        fill = item.get("reference", {}).get("measured", {}).get("fill")
        if fill is None:
            problems.append(f"`{name}` records no measured fill, so a201 has nothing "
                            f"to build its envelope from")
        else:
            low, high = fill_envelope(name)
            if not low <= fill <= high:
                problems.append(f"`{name}`'s own measurement {fill} falls outside the "
                                f"envelope {low}-{high} derived from it")
        icon = item["icon"]
        if icon in seen_icons:
            problems.append(f"`{icon}` is claimed by both `{seen_icons[icon]}` and `{name}`")
        seen_icons[icon] = name
        for parameter in item["parameters"]:
            label = f"`{name}.{parameter['name']}`"
            if not parameter.get("source"):
                problems.append(f"{label} has no source, so nobody can tell whether "
                                f"it was measured or chosen")
            if parameter["type"] == "count":
                for end in ("md1", "ios6"):
                    if float(parameter[end]) != int(parameter[end]):
                        problems.append(f"{label}'s {end} end is "
                                        f"{parameter[end]}, and a count endpoint "
                                        f"has to be a whole number")
            if parameter["type"] == "fixed" and parameter["md1"] != parameter["ios6"]:
                problems.append(f"{label} is fixed but its ends differ")
    return {
        "families": len(FAMILIES),
        "parameters": sum(len(item["parameters"]) for item in FAMILIES.values()),
        "measured_endpoints": sum(
            1 for item in FAMILIES.values() for parameter in item["parameters"]
            if parameter["source"].startswith("measured")
        ),
        "fill_envelopes": {name: fill_envelope(name) for name in sorted(FAMILIES)},
        "composition": dict(COMPOSITION),
        "problems": problems,
    }
