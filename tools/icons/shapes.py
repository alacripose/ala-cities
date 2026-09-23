"""Explicit pilot icon authoring data and procedural shape vocabulary.

The generator is a renderer, not an art director. Every pilot candidate is a named,
reviewable construction recipe with a semantic brief. Numeric checks can reject a
candidate; they cannot invent a silhouette or approve one.
"""

import math
import os

import families
import openpbr
import palette
from palette import rgba

#: The concept set these candidates belong to. Bumped by C8 a160: the silhouette
#: check's frame and the traced outline both changed, so every render in the review
#: set is a different artifact and the six promotions taken in C7 are void by the
#: standard's own rule (§8) — re-decided in the picker, by a person.
CONCEPT_SET = "pilot-six-c8-v1"
CATALOGUE_STAGE = "pilot-six"

#: The rule that settles an icon's form, settled in C3 and recorded in
#: `docs/GRILLING-C3.md`. It is aspect-based, not object-based: each reference
#: family owns a *property* of every icon rather than owning the icons it happens
#: to have a matching object for. The old object-based precedence is preserved below
#: only as the thing it replaced.
REFERENCE_STUDY = {
    "rule": (
        "one silhouette from Material Design 1, three main materials whose surface "
        "and colour intent come from TouchWiz, and one accenting record piece from "
        "iOS 6, which also sits as a skeuomorphic layer above the TouchWiz base"
    ),
    "responsibilities": {
        "silhouette": {
            "family": "material-design-icons-4.0.0",
            "pack": "assets/reference/material-design-icons-4.0.0",
            "licence": "Apache-2.0",
            "artifact": (
                "png/<category>/<glyph>/materialicons/<size>dp/<1x,2x>/"
                "baseline_<glyph>_black_<size>dp.png"
            ),
            "style": "materialicons (the `baseline` mark), 24dp grid, always",
            "why": (
                "`baseline` is the MD1-era filled mark; outlined/round/sharp/twotone "
                "are the later set, and mixing them would make six glyphts instead "
                "of one system"
            ),
        },
        "materials": {
            "family": "galaxy-s4-icon-pack",
            "pack": "assets/reference/galaxy-s4-icon-pack",
            "licence": "Samsung artwork, reference only",
            "population": "res/drawable/*.png",
            "why": (
                "the only TouchWiz artifact on disk; round 13's pillar said *early* "
                "TouchWiz and the artifact is Nature UX, which is recorded as a "
                "disagreement rather than quietly resolved"
            ),
        },
        "accent_and_layer": {
            "family": "iOS-6-Icons-main",
            "pack": "assets/reference/iOS-6-Icons-main",
            "licence": "community and third-party artwork, reference only",
            "why": (
                "iOS 6 is the accent piece and the skeuomorphic layer above the "
                "TouchWiz base, never the base itself"
            ),
        },
    },
    "sampled": {
        "method": (
            "per-class circular mean hue over the pack's TouchWiz-native rasters, "
            "weighted by saturation x value"
        ),
        "class_hues": {
            "people": 162.6, "message": 34.5, "media": 226.2,
            "system": 131.4, "store": 201.8, "nature_health": 193.0,
        },
        "finding": (
            "45.7% of chroma sits in 180-210 degrees and the class spread is a "
            "95 degree arc: the pack holds no red, no yellow and no violet, so the "
            "seven hue anchors are declared rather than sampled, four of them "
            "backed by the game's own hues and two recorded as slots"
        ),
    },
    "grammar": [
        "layered inset and rim construction",
        "softly rounded or chamfered edges with controlled highlights",
        "coherent object scale and transparent presentation",
        "density variants preserve the semantic read while reducing detail",
    ],
    "superseded_object_precedence": [
        "Samsung/TouchWiz reference grammar when a matching object exists",
        "Google Material Design silhouette grammar when Samsung has no matching object",
        "iOS 6 grammar only as the final fallback",
    ],
    "provenance": (
        "geometry is authored on the MD1 grid, or adapted from the MD1 outline "
        "where an icon declares `md1-adapted`; the manifest records which, per "
        "icon, and no TouchWiz or iOS 6 artwork is imported"
    ),
    "source_artwork_copied": False,
    "materials": "procedural only; no reference texture samples or raster fragments",
}

#: The three parts every icon has, which is the whole point of the composition
#: rule: a recipe names a *function* and the icon's palette decides the material.
COMPOSITION_ROLES = ("silhouette", "secondary", "accent")

#: Which family leads, by tier. The tier sets the lead; the pair sets the rest, so
#: an icon's palette is a reading of one decision rather than three. The three
#: families in a tier are distinct by construction, which is what makes "three main
#: colours" true of every candidate rather than of most of them.
TIERS = {
    "tool": {"lead": "metal", "pair": ("polymer", "glass")},
    "record": {"lead": "paper", "pair": ("metal", "enamel")},
    "mark": {"lead": "enamel", "pair": ("metal", "glass")},
}

#: Details are drawn in the neutral anchors and are **not** counted among the three
#: main colours: a rule on a page or a rim on a body is not a fourth material, and a
#: check that counted them would count the shading.
NEUTRAL_ROLES = ("ink", "white", "black")

#: The three pillars, and the six candidates. Each emphasis leads with one family's
#: contribution and restrains the other two, so the family spread is a *choice* the
#: picker shows rather than a note it prints, and each slot is still a whole
#: composition under the rule above.
EMPHASES = (
    {"id": "md1", "label": "silhouette-led", "ladder": 0.0,
     "why": "MD1 leads: the flattest, most legible reading of the form"},
    {"id": "touchwiz", "label": "material-led", "ladder": 0.5,
     "why": "TouchWiz leads: the surfaces carry the object, the silhouette simplifies"},
    {"id": "ios6", "label": "accent-led", "ladder": 1.0,
     "why": "iOS 6 leads: the record piece is the loudest thing in the frame"},
)

#: The three design languages as one **measured** ladder, not three opinions.
#:
#: Sampled from disk at the decision size, each archive's own marks. The values
#: below are what the corpora measure; they are the anchors the shape builder
#: interpolates and the numbers a candidate is checked against, so "this candidate
#: is TouchWiz-led" is a claim with a threshold rather than a label with a ring:
#:
#:   language   fill (median)   IQR of fill    gloss (p75)   pieces   holes
#:   md1            0.490       0.366-0.706      0.000        2.0      1.0
#:   touchwiz       0.937       0.784-0.997      0.102        1.0      0.0
#:   ios6           1.000       1.000-1.000      0.240        1.0      0.0
#:
#: **The statistic differs per parameter on purpose.** Fill and complexity are
#: central tendencies, so the median is the anchor. Gloss's median is *zero* for two
#: of the three archives -- a highlight is a minority of pixels even in a glossy
#: icon -- so gloss's anchor is its 75th percentile and using a median would have
#: declared that neither MD1 nor TouchWiz has any gloss at all.
#:
#: **The tolerance is the corpus's own spread**, not a number chosen to make the
#: check pass: fill's band is each archive's interquartile range, because that is
#: what the reference family itself spans. iOS 6's IQR is a single value because
#: every sampled iOS 6 mark fills its box -- which is the whole character of that
#: language and is worth knowing before asking a candidate to imitate it.
#:
#: Two caveats, recorded because they bound what this measures. MD1's row is
#: achromatic and gloss-free **by construction**: the population is the
#: `baseline_black` marks that C3's a18(a) adopted, and Material Design's coloured
#: product icons would move it. And the anchors are close to collinear -- fill and
#: gloss both rise while complexity falls -- so this is one axis, which is why the
#: six slots need the construction axis as well (C4's a38).
LANGUAGE_LADDER = {
    "md1": {
        "lambda": 0.0,
        "fill": 0.490, "fill_band": (0.366, 0.706),
        "gloss": 0.000, "pieces": 2.0, "holes": 1.0,
        "population": 137,
        "digest": "2fe369f8396434d8",
    },
    "touchwiz": {
        "lambda": 0.5,
        "fill": 0.937, "fill_band": (0.784, 0.997),
        "gloss": 0.102, "pieces": 1.0, "holes": 0.0,
        "population": 125,
        "digest": "c58b07e5b327bb0f",
    },
    "ios6": {
        "lambda": 1.0,
        "fill": 1.000, "fill_band": (1.000, 1.000),
        "gloss": 0.240, "pieces": 1.0, "holes": 0.0,
        "population": 97,
        "digest": "e92be36568d1ab29",
    },
}

#: How each language's sample was taken, so the numbers above can be re-derived
#: rather than trusted. The **globs are the population** and the digests are over
#: the sorted relative path and byte size of every file in it.
LADDER_SAMPLING = {
    "box_px": 96,
    "resample": "BOX (area average), on the mark's own square",
    "fill": "opaque mask area / its own bounding box area",
    "gloss": (
        "fraction of opaque pixels above 1.6x the mark's own median luminance; a "
        "tail statistic, so its anchor is p75"
    ),
    "pieces": "8-connected components of the opaque mask",
    "holes": "background components not reachable from the border",
    "glob": {
        "md1": "assets/reference/material-design-icons-4.0.0/png/*/*/materialicons/48dp/2x/baseline_*.png",
        "touchwiz": "assets/reference/galaxy-s4-icon-pack/res/drawable/*.png",
        "ios6": "assets/reference/iOS-6-Icons-main/**/*.{PNG,png}",
    },
    "population_total": {"md1": 1503, "touchwiz": 125, "ios6": 814},
    "licence": (
        "measurement only. Only the MD1 archive is derivative-safe (Apache-2.0); "
        "the TouchWiz pack is an unpacked Samsung APK and the iOS 6 set is a "
        "community archive asking for credit. No geometry is ever taken from "
        "either -- the archives supply numbers, not shapes."
    ),
}

#: A candidate's measured gloss is a single icon's number against a population's
#: p75, so it carries a floor as well as the corpus's spread: below this the
#: difference is inside the measurement's own noise.
GLOSS_TOLERANCE_FLOOR = 0.03
#: Complexity is counted in whole pieces, so its tolerance is a count.
TOPOGRAPHY_TOLERANCE = 1.0


def _ladder_at(lam: float) -> dict:
    """The interpolated anchor set at a position on the ladder.

    `lambda` is *nominal*: each language sits at its declared position because the
    three archives are anchors rather than evenly spaced samples, and saying that is
    more honest than deriving a spacing from tenuous geometry.
    """
    if not 0.0 <= lam <= 1.0:
        raise ValueError(f"ladder position {lam} is outside [0, 1]")
    points = sorted(LANGUAGE_LADDER.values(), key=lambda spec: spec["lambda"])
    for lower, upper in zip(points, points[1:]):
        if lower["lambda"] <= lam <= upper["lambda"]:
            span = upper["lambda"] - lower["lambda"]
            mix = 0.0 if span == 0.0 else (lam - lower["lambda"]) / span
            blend = lambda a, b: a + (b - a) * mix  # noqa: E731
            return {
                "lambda": lam,
                "fill": round(blend(lower["fill"], upper["fill"]), 4),
                "fill_band": (
                    round(blend(lower["fill_band"][0], upper["fill_band"][0]), 4),
                    round(blend(lower["fill_band"][1], upper["fill_band"][1]), 4),
                ),
                "gloss": round(blend(lower["gloss"], upper["gloss"]), 4),
                "gloss_tolerance": round(
                    max(
                        GLOSS_TOLERANCE_FLOOR,
                        min(lower["gloss"], upper["gloss"]) * 0.5,
                    ),
                    4,
                ),
                "pieces": round(blend(lower["pieces"], upper["pieces"]), 2),
                "holes": round(blend(lower["holes"], upper["holes"]), 2),
            }
    raise ValueError(f"ladder position {lam} fell outside every declared segment")


def ladder_point(lam: float) -> dict:
    """The anchor set a candidate at this ladder position must measure like."""
    return _ladder_at(lam)


def ladder_record() -> dict:
    """The ladder and its provenance, for the manifest."""
    return {
        "anchors": {
            name: dict(spec) for name, spec in sorted(LANGUAGE_LADDER.items())
        },
        "sampling": LADDER_SAMPLING,
        "statistics": (
            "per parameter: median for fill and complexity, p75 for gloss, because "
            "gloss's median is zero in two of the three archives"
        ),
        "tolerance": (
            "fill uses each archive's own interquartile range; gloss uses half the "
            "lower anchor with a declared floor; complexity is a whole-piece count"
        ),
    }

CONSTRUCTIONS = (
    {"id": "plate", "label": "single body",
     "why": "one extruded body with an inset face"},
    {"id": "stack", "label": "layered body",
     "why": "the same silhouette built as stacked plates, TouchWiz layered construction"},
)


def mat(token, roughness, ior=1.45, metallic=0.0, coat=0.0, finish="plain"):
    parameters = {
        "base_color": rgba(token),
        "base_metalness": metallic,
        "specular_roughness": roughness,
        "specular_ior": ior,
    }
    if coat:
        parameters["geometry_coat_weight"] = coat
    parameters["finish"] = finish
    return parameters


def B(x, z, w, h, depth=0.34, angle=0.0, role="ink", finish=None):
    part = {"shape": "box", "x": x, "z": z, "w": w, "h": h,
            "depth": depth, "angle": angle, "role": role}
    if finish:
        part["finish"] = finish
    return part


def R(x, z, major, minor, angle=0.0, role="ink", finish=None):
    part = {"shape": "ring", "x": x, "z": z, "major": major,
            "minor": minor, "angle": angle, "role": role}
    if finish:
        part["finish"] = finish
    return part


def C(x, z, radius, depth=0.34, role="ink", vertices=48, finish=None):
    part = {"shape": "cylinder", "x": x, "z": z, "radius": radius,
            "depth": depth, "role": role, "vertices": vertices}
    if finish:
        part["finish"] = finish
    return part


def G(x, z, radius, teeth=10, tooth=0.16, depth=0.34, role="metal", finish=None):
    """An extruded radial silhouette with short root lands and rounded tooth tips."""
    part = {"shape": "gear", "x": x, "z": z, "radius": radius,
            "teeth": teeth, "tooth": tooth, "depth": depth, "role": role}
    if finish:
        part["finish"] = finish
    return part


def gear_parts(teeth=10, radius=0.72, tooth=0.16, role="metal",
               detail=False, hub_role="accent", face_role="metal",
               centre_role="black"):
    # The old implementation assembled a gear from a torus and rectangular teeth.
    # That read as a primitive exploded diagram at 96px.  Keep the recipe compact,
    # but make the primary silhouette one authored, bevelable radial mesh.
    parts = [G(0.0, 0.0, radius, teeth, tooth, role=role, finish="brushed")]
    parts.append(C(0.0, 0.0, radius * 0.43, 0.16, role=hub_role, vertices=48,
                   finish="polished"))
    parts.append(C(0.0, 0.0, radius * 0.29, 0.38, role=face_role, vertices=48,
                   finish="brushed"))
    parts.append(C(0.0, 0.0, radius * 0.13, 0.42, role=centre_role, vertices=48,
                   finish="matte"))
    if detail:
        parts.extend([
            R(0.0, 0.0, radius * 0.50, radius * 0.045, role="accent", finish="polished"),
            C(0.0, 0.0, radius * 0.13, 0.46, role=centre_role, vertices=48, finish="polished"),
        ])
    return parts


def pylon_parts(variant="tower", detail=False):
    if variant == "node":
        parts = [
            C(0.0, 0.0, 0.3, 0.42, role="metal", vertices=8, finish="brushed"),
            R(0.0, 0.0, 0.68, 0.1, role="accent", finish="polished"),
            B(-0.62, 0.0, 0.22, 0.22, 0.32, role="metal", finish="brushed"),
            B(0.62, 0.0, 0.22, 0.22, 0.32, role="metal", finish="brushed"),
            B(0.0, 0.62, 0.22, 0.22, 0.32, role="metal", finish="brushed"),
            B(0.0, -0.62, 0.22, 0.22, 0.32, role="metal", finish="brushed"),
        ]
    else:
        parts = [
            B(0.0, -0.56, 0.28, 0.72, 0.36, role="metal", finish="brushed"),
            B(0.0, 0.12, 0.16, 1.0, 0.32, role="metal", finish="brushed"),
            B(0.0, 0.62, 1.28, 0.16, 0.32, role="metal", finish="brushed"),
            C(-0.48, 0.46, 0.12, 0.28, role="ceramic", vertices=24, finish="polished"),
            C(0.48, 0.46, 0.12, 0.28, role="ceramic", vertices=24, finish="polished"),
        ]
    if detail:
        parts.extend([
            B(-0.38, -0.18, 0.08, 0.62, 0.22, role="metal", finish="brushed"),
            B(0.38, -0.18, 0.08, 0.62, 0.22, role="metal", finish="brushed"),
            B(-0.7, 0.7, 0.12, 0.12, 0.26, role="accent", finish="polished"),
            B(0.7, 0.7, 0.12, 0.12, 0.26, role="accent", finish="polished"),
        ])
    return parts


def lens_parts(detail=False):
    """A layered Material-style search lens, not a torus-and-stick placeholder."""
    parts = [
        # Housing and glass are separate depth-bearing surfaces so the silhouette
        # reads as a magnifier before any highlight is added.
        C(-0.2, 0.22, 0.66, 0.3, role="metal", vertices=64, finish="polished"),
        C(-0.2, 0.22, 0.53, 0.34, role="glass", vertices=64, finish="glass"),
        R(-0.2, 0.22, 0.44, 0.045, role="accent", finish="polished"),
        C(-0.2, 0.22, 0.38, 0.37, role="glass", vertices=64, finish="glass"),
        # A collar makes the handle join the housing instead of merely touching it.
        C(0.28, -0.29, 0.17, 0.36, role="metal", vertices=32, finish="brushed"),
        B(0.55, -0.58, 0.92, 0.22, 0.34, angle=-45.0, role="polymer", finish="matte"),
    ]
    if detail:
        parts.extend([
            B(-0.43, 0.47, 0.17, 0.08, 0.24, angle=-35.0, role="glass", finish="glass"),
            B(0.75, -0.78, 0.42, 0.24, 0.36, angle=-45.0, role="polymer", finish="matte"),
            R(-0.2, 0.22, 0.31, 0.025, role="accent", finish="polished"),
        ])
    return parts


def plaque_parts(detail=False, ticket=False):
    if ticket:
        parts = [
            B(0.0, 0.0, 1.62, 1.0, 0.25, role="paper", finish="paper"),
            B(0.42, 0.0, 0.12, 0.88, 0.3, role="accent", finish="polished"),
            R(-0.62, 0.0, 0.12, 0.07, role="paper", finish="paper"),
        ]
    else:
        parts = [
            B(-0.58, 0.0, 0.18, 1.64, 0.42, role="metal", finish="brushed"),
            B(0.12, 0.52, 1.42, 0.38, 0.28, role="paper", finish="paper"),
            B(0.12, -0.02, 1.42, 0.38, 0.28, role="paper", finish="paper"),
            B(0.12, -0.56, 1.42, 0.38, 0.28, role="paper", finish="paper"),
        ]
    if detail:
        parts.extend([
            B(0.12, 0.52, 0.92, 0.08, 0.36, role="accent", finish="polished"),
            B(0.12, -0.02, 0.92, 0.08, 0.36, role="accent", finish="polished"),
            B(0.12, -0.56, 0.92, 0.08, 0.36, role="accent", finish="polished"),
        ])
    return parts


def road_parts(variant="plaque", detail=False):
    if variant == "junction":
        parts = [
            B(0.0, 0.0, 0.5, 1.74, 0.28, role="road", finish="road"),
            B(0.0, 0.0, 1.74, 0.5, 0.28, role="road", finish="road"),
            B(0.0, 0.0, 0.09, 1.46, 0.34, role="accent", finish="enamel"),
            B(-0.58, 0.0, 0.44, 0.09, 0.34, role="accent", finish="enamel"),
            B(0.58, 0.0, 0.44, 0.09, 0.34, role="accent", finish="enamel"),
        ]
    else:
        parts = [
            B(0.0, 0.0, 0.78, 1.76, 0.3, role="road", finish="road"),
            B(0.0, 0.54, 0.1, 0.3, 0.36, role="accent", finish="enamel"),
            B(0.0, 0.0, 0.1, 0.3, 0.36, role="accent", finish="enamel"),
            B(0.0, -0.54, 0.1, 0.3, 0.36, role="accent", finish="enamel"),
            B(-0.48, 0.0, 0.12, 1.9, 0.38, role="metal", finish="brushed"),
            B(0.48, 0.0, 0.12, 1.9, 0.38, role="metal", finish="brushed"),
        ]
    if detail:
        parts.extend([
            B(-0.82, 0.0, 0.08, 1.2, 0.24, role="metal", finish="brushed"),
            B(0.82, 0.0, 0.08, 1.2, 0.24, role="metal", finish="brushed"),
        ])
    return parts


def icon(icon_id, kind, meaning, source, locates, parts, materials, brief,
         lineage, forbidden, alternate=None, notes=None):
    return {
        "id": icon_id, "kind": kind, "meaning": meaning, "source": source,
        "locates": locates, "sits_on": ("Panel", "PanelRaised"),
        "identity": brief.get("identity", brief["material_family"]), "identity_as": "detail-on-ink",
        "materials": materials, "parts": parts, "framed": False,
        "brief": brief, "lineage": lineage, "forbidden_readings": forbidden,
        "notes": list(notes or ()), "alternate": alternate,
    }


# ---------------------------------------------------------------------------
# The silhouette registry
# ---------------------------------------------------------------------------

MD1_PACK = "assets/reference/material-design-icons-4.0.0"
#: `materialicons` is the style directory whose marks are named `baseline_*`: the
#: MD1-era filled glyph. The other four styles are the later set.
MD1_STYLE = "materialicons"
MD1_SIZE_DP = 48
#: 48dp at 2x is 96x96 pixels — the decision size exactly, so the silhouette check
#: compares two 96 px rasters and neither side is resampled.
MD1_SCALE = "2x"
MD1_SIDE_PX = MD1_SIZE_DP * 2


SILHOUETTES = {
    "tool-road": {"glyph": "maps/add_road", "mode": "md1-adapted"},
    "tool-power": {"glyph": "action/power_settings_new", "mode": "md1-adapted"},
    "tool-inspect": {"glyph": "action/search", "mode": "md1-adapted"},
    "tool-demolish": {"glyph": "action/delete", "mode": "md1-adapted"},
    "ticket": {"glyph": "notification/confirmation_number", "mode": "md1-adapted"},
    "vocab-settings": {"glyph": "action/settings", "mode": "md1-adapted"},
    "ledger": {
        "glyph": None, "mode": "deferred",
        "why": (
            "MD1 has no ledger glyph, so C3's gap rule would send it to iOS 6, and "
            "the record is deferred instead of an invented surface being drawn"
        ),
    },
}

#: Inventory the game declares and the catalogue does not yet draw. Recorded rather
#: than left implicit, because a missing icon is a fact about the set.
MISSING_INVENTORY = (
    {"surface": "Tool::Zone", "why": "no MD1 glyph and no zone surface in the game yet"},
)


#: Resolved from this file's own location, never from the process's working
#: directory: Blender is launched from wherever the caller happens to be, and a
#: relative path that works in a shell prompt is a path that fails in a build.
ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))


def md1_relative_path(glyph: str) -> str:
    """One MD1 glyph's reference artifact, as the manifest states it."""
    category, name = glyph.split("/", 1)
    return (f"{MD1_PACK}/png/{category}/{name}/{MD1_STYLE}/{MD1_SIZE_DP}dp/"
            f"{MD1_SCALE}/baseline_{name}_black_{MD1_SIZE_DP}dp.png")


def md1_path(glyph: str) -> str:
    """Where that artifact actually is on this machine."""
    return os.path.join(ROOT, md1_relative_path(glyph))


def md1_reference(glyph: str) -> dict:
    """A glyph's file and digest, as the manifest states it."""
    if not glyph:
        return None
    path = md1_path(glyph)
    exists = os.path.exists(path)
    return {
        "glyph": glyph,
        "style": MD1_STYLE,
        "size_dp": MD1_SIZE_DP,
        "scale": MD1_SCALE,
        "side_px": MD1_SIDE_PX,
        "file": md1_relative_path(glyph),
        "digest": palette.source_hash(path) if exists else None,
        "exists": exists,
    }


def silhouette_record(icon_id: str) -> dict:
    """The declared silhouette for one icon, with its reference resolved."""
    declared = SILHOUETTES.get(icon_id)
    if declared is None:
        raise KeyError(f"`{icon_id}` declares no silhouette; every icon needs one")
    record = dict(declared)
    record["reference"] = md1_reference(declared.get("glyph"))
    return record


#: How much of the icon's own box a silhouette's bounding box fills, in world units.
#: The rig's ortho scale is 2.25, so a tile spans 2.25 units and a mark that filled
#: it edge to edge would leave no margin at all — which is not how a locator reads.
GLYPH_SPAN = 1.72

#: The icon's own frame, in world units: the orthographic camera's scale. It is
#: stated here because the silhouette check has to express the declared mark in the
#: **same frame** the render is measured in, and the two must not disagree.
#: `generate.py` refuses to run if this and `rig.ORTHO_SCALE` differ, so the number
#: cannot drift apart from the rig it describes.
ICON_FRAME_SPAN = 2.25


def _load_glyph(bpy, glyph: str):
    """An MD1 mark's pixels, as Blender hands them back: bottom-up rows."""
    path = md1_path(glyph)
    if not os.path.exists(path):
        raise FileNotFoundError(
            f"the declared silhouette for `{glyph}` is not on disk: {path}"
        )
    image = bpy.data.images.load(path, check_existing=False)
    image.colorspace_settings.name = "Non-Color"
    pixels = list(image.pixels)
    width, height = image.size
    bpy.data.images.remove(image)
    if width != height:
        raise ValueError(f"`{glyph}` is {width}x{height}; a silhouette must be square")
    return width, height, pixels


def glyph_cells(bpy, glyph: str, alpha_threshold: float = 0.5) -> tuple:
    """Covered cells as a set of `(x, y)` with y measured **downward** from the top.

    Top-down because that is the orientation every consumer of these marks already
    agrees on: the pack's own file is top-down, the runtime sidecar is top-down, and
    the review set is top-down. Blender's bottom-up rows are flipped exactly once,
    here, and nowhere else.
    """
    width, height, pixels = _load_glyph(bpy, glyph)
    cells = set()
    for row in range(height):
        base = row * width * 4
        y = height - 1 - row
        for column in range(width):
            if pixels[base + column * 4 + 3] >= alpha_threshold:
                cells.add((column, y))
    return width, height, cells


def _components(cells):
    """Eight-connected components, largest first.

    Largest first because the body is the thing that has to read at 24 px, and a
    speck of antialiasing is not a part of the mark.
    """
    remaining = set(cells)
    found = []
    while remaining:
        seed = remaining.pop()
        stack = [seed]
        component = {seed}
        while stack:
            x, y = stack.pop()
            for dx in (-1, 0, 1):
                for dy in (-1, 0, 1):
                    neighbour = (x + dx, y + dy)
                    if neighbour in remaining:
                        remaining.discard(neighbour)
                        component.add(neighbour)
                        stack.append(neighbour)
        found.append(component)
    found.sort(key=len, reverse=True)
    return found


#: How far the traced outline is grown past the mark's own cells, in raster cells.
#: One is the measured correction, and it has two causes that do not go away:
#: `_trace` walks the **centres** of the boundary cells, so the polygon it produces
#: sits half a cell inside the mark, and `_simplify` may cut up to its own tolerance
#: (0.34 px) from every corner. At the decision size that cost the outermost declared
#: cells about 15% of their ink — measured, not assumed — which is the difference
#: between a render that *carries* the mark and one that sits just inside it.
OUTLINE_GROW_CELLS = 1


def _dilate(cells, radius: int = OUTLINE_GROW_CELLS):
    """Grow a cell set by `radius` cells in every direction."""
    grown = set()
    for x, y in cells:
        for dx in range(-radius, radius + 1):
            for dy in range(-radius, radius + 1):
                grown.add((x + dx, y + dy))
    return grown


def _trace(component) -> list:
    """Moore-neighbour boundary trace of one component, as cell coordinates.

    Returns the outer boundary only: a component's own holes are separate
    components themselves, so they are traced by their own pass and subtracted.
    """
    start = min(component, key=lambda cell: (cell[1], cell[0]))
    # Clockwise, starting from the west of the start cell, which is guaranteed to be
    # outside the component because the start is its topmost-then-leftmost cell.
    directions = ((-1, 0), (-1, -1), (0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1))
    contour = [start]
    current, entered_from = start, 0
    for _ in range(len(component) * 8):
        for step in range(8):
            direction = (entered_from + 1 + step) % 8
            dx, dy = directions[direction]
            candidate = (current[0] + dx, current[1] + dy)
            if candidate in component:
                current = candidate
                # Backtrack: begin the next scan from the direction we came in on.
                entered_from = (direction + 5) % 8
                contour.append(current)
                break
        else:
            break
        if current == start and len(contour) > 2:
            break
    return contour


def _simplify(points, tolerance: float = 0.34) -> list:
    """Douglas-Peucker, so a traced edge is a few corners rather than a pixel run."""
    if len(points) < 3:
        return list(points)
    first, last = points[0], points[-1]
    span = math.hypot(last[0] - first[0], last[1] - first[1])
    furthest, index = -1.0, 0
    for position in range(1, len(points) - 1):
        point = points[position]
        if span < 1e-9:
            distance = math.hypot(point[0] - first[0], point[1] - first[1])
        else:
            distance = abs(
                (last[0] - first[0]) * (first[1] - point[1])
                - (first[0] - point[0]) * (last[1] - first[1])
            ) / span
        if distance > furthest:
            furthest, index = distance, position
    if furthest <= tolerance:
        return [first, last]
    left = _simplify(points[: index + 1], tolerance)
    right = _simplify(points[index:], tolerance)
    return left[:-1] + right


def placement_scale(bounds, span: float = GLYPH_SPAN) -> float:
    """World units per mask pixel, for a mark whose own bounds are fitted to `span`.

    One rule, one home: the geometry builder and the silhouette check both ask this,
    so the frame the check measures in cannot drift from the frame the mesh is built
    in.
    """
    min_x, max_x, min_y, max_y = bounds
    width = max(max_x - min_x, 1e-6)
    height = max(max_y - min_y, 1e-6)
    return span / max(width, height)


def placed_frame(bounds, span: float = GLYPH_SPAN, frame: float = ICON_FRAME_SPAN):
    """Mask pixel to the icon's frame as `(u, v)`, v measured downward.

    The frame is the square the orthographic camera sees, so this is the transform
    that makes a declared mask and a rendered frame comparable: without it the
    comparison put a centred, own-bounds-fitted mark against a mask binned where it
    sits **in its own file**, which is exactly the fault `ICON_STANDARD` left open
    for `tool-road` (containment 0.3235) and `tool-road`'s mark does not sit centred.
    """
    to_world = placement(1, bounds, span)

    def to_frame(pixel):
        wx, wz = to_world(pixel)
        return ((wx + frame / 2.0) / frame, (frame / 2.0 - wz) / frame)

    return to_frame


def placement(side_px: int, bounds, span: float = GLYPH_SPAN) -> tuple:
    """Mask pixels to world `(x, z)`, fitting the mark's **own** bounds to `span`.

    Its own bounds, not the file's: the marks are not centred in their 24 dp box.
    `maps/add_road` occupies x 0.167-0.948 and `action/search` sits up-left of
    centre, so centring on the file's box would put six icons at six different
    offsets and make the smallest size read inconsistently. This is also the exact
    function the silhouette check reuses, so the comparison is against the mark as
    placed rather than against an assumption about where it would have gone.
    """
    min_x, max_x, min_y, max_y = bounds
    scale = placement_scale(bounds, span)
    centre_x = (min_x + max_x) / 2.0
    centre_y = (min_y + max_y) / 2.0

    def to_world(point):
        px, py = point
        # Mask y runs downward and world z runs up, and the world y axis is the
        # camera's depth, so the silhouette is built in the x/z plane.
        return ((px - centre_x) * scale, -(py - centre_y) * scale)

    return to_world


def _enclosed_background(cells, width: int, height: int):
    """Background regions the mark encloses, largest first.

    A hole is an *absence*, so it is not a covered component and cannot be found by
    looking for one. The first version of this helper did exactly that and reported
    zero holes for `action/settings` — a gear with a hole in its centre — so a solid
    gear would have shipped, differing from its own reference in precisely the cells
    a silhouette check exists to compare. The test is a flood fill: whatever
    background the border cannot reach is enclosed.
    """
    everything = {(x, y) for x in range(width) for y in range(height)}
    background = everything - cells
    reachable = set()
    stack = [
        cell for cell in background
        if cell[0] in (0, width - 1) or cell[1] in (0, height - 1)
    ]
    for cell in stack:
        reachable.add(cell)
    # Four-connected for the background: with eight-connected foreground this is the
    # pairing that keeps a diagonal join from closing a region that is visibly open.
    while stack:
        x, y = stack.pop()
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            neighbour = (x + dx, y + dy)
            if neighbour in background and neighbour not in reachable:
                reachable.add(neighbour)
                stack.append(neighbour)
    return _components(background - reachable)


def glyph_outline(bpy, glyph: str, minimum_cells: int = 24) -> dict:
    """The mark as placed contours: the pieces drawn, and the regions they enclose."""
    width, height, cells = glyph_cells(bpy, glyph)
    if not cells:
        raise ValueError(f"`{glyph}` has nothing opaque; it cannot be a silhouette")
    body = [component for component in _components(cells) if len(component) >= minimum_cells]
    if not body:
        raise ValueError(f"`{glyph}` has no component above {minimum_cells} cells")
    holes = [
        component for component in _enclosed_background(cells, width, height)
        if len(component) >= minimum_cells
    ]

    # Placement is taken from the union of everything that is drawn, so a mark with
    # a detached part is sized by the whole mark rather than by its biggest piece.
    drawn = [cell for component in body for cell in component]
    xs = [cell[0] for cell in drawn]
    ys = [cell[1] for cell in drawn]
    to_world = placement(width, (min(xs), max(xs) + 1, min(ys), max(ys) + 1))

    def outline(component):
        # Both the body and the enclosed holes are grown: the same centre-sampling
        # inset applies to a hole's boundary, and a hole traced from the raster is a
        # cell too *large*, which is the same error read outward.
        traced = _trace(_dilate(component))
        closed = _simplify(traced + [traced[0]], 0.34)
        # The closing point is dropped: the polygon is closed by the mesh builder
        # walking edge-to-edge, and a repeated vertex would make a degenerate quad.
        if len(closed) > 1 and closed[0] == closed[-1]:
            closed = closed[:-1]
        return [to_world(point) for point in closed]

    return {
        "glyph": glyph,
        "side_px": width,
        "cells": len(cells),
        "coverage": round(len(cells) / (width * height), 4),
        "bodies": [outline(component) for component in body],
        "holes": [outline(component) for component in holes],
        "hole_cells": sum(len(component) for component in holes),
        "enclosed_fraction": round(
            sum(len(component) for component in holes) / len(cells), 4
        ),
        "placement": "own bounds, fitted to GLYPH_SPAN",
        "span": GLYPH_SPAN,
        "simplify_tolerance_px": 0.34,
        "grow_cells": OUTLINE_GROW_CELLS,
        "grow_why": (
            "the trace walks boundary-cell centres, half a cell inside the mark, and "
            "simplification shaves its own tolerance from every corner; the outline is "
            "grown by this many cells so the render carries the mark rather than "
            "sitting inside it"
        ),
        "bounds_from": "the mark's own cells, ungrown, so placement is unchanged",
    }


#: Outlines are traced once per glyph per run: the trace is deterministic and the
#: same mark is used by six candidates, so tracing it six times would be six times
#: the work for the same polygon.
_OUTLINE_CACHE = {}


def outline_for(bpy, glyph: str) -> dict:
    """A glyph's placed outline, traced once."""
    if glyph not in _OUTLINE_CACHE:
        _OUTLINE_CACHE[glyph] = glyph_outline(bpy, glyph)
    return _OUTLINE_CACHE[glyph]


def build_silhouette(bpy, name: str, material, outline: dict,
                     depth: float = 0.36, bevel: float = 0.02, collection=None,
                     scale: float = 1.0, y_offset: float = 0.0):
    """A mesh from a placed outline: a prism per body, minus the enclosed holes.

    Holes are cut rather than filled, because a filled gear centre is a different
    silhouette from the mark it claims to be, and a check comparing the two would
    have to be told to ignore exactly the difference it exists to find. `scale`
    shrinks the outline about the mark's own centre, which is how a nested plate is
    built without a second trace.
    """
    def scaled(polygon):
        return [(point[0] * scale, point[1] * scale) for point in polygon]

    target = collection or bpy.context.collection
    bodies = []
    for index, polygon in enumerate(outline["bodies"]):
        polygon = scaled(polygon)
        if len(polygon) < 3:
            continue
        obj = _prism(bpy, f"{name} {index}", polygon, depth, target, y_offset)
        obj.data.materials.append(material)
        if bevel:
            edge = obj.modifiers.new(name="authored rounded edge", type="BEVEL")
            edge.width = bevel
            edge.segments = 3
            edge.limit_method = "ANGLE"
        bodies.append(obj)

    cutters = []
    for index, polygon in enumerate(outline["holes"]):
        polygon = scaled(polygon)
        if len(polygon) < 3:
            continue
        # Deeper than the body, so the boolean never has to resolve two coplanar
        # faces, which is the case that produces the hole-shaped artefact instead
        # of the hole.
        cutter = _prism(bpy, f"{name} hole {index}", polygon, depth * 1.6, target,
                        y_offset)
        cutters.append(cutter)
        for body in bodies:
            boolean = body.modifiers.new(name="cut hole", type="BOOLEAN")
            boolean.operation = "DIFFERENCE"
            boolean.object = cutter
    for object_ in bodies + cutters:
        _recalculate_normals(bpy, object_)
    for body in bodies:
        for modifier in [m for m in body.modifiers if m.type == "BOOLEAN"]:
            _apply_modifier(bpy, body, modifier)
    for cutter in cutters:
        bpy.data.objects.remove(cutter, do_unlink=True)
    return bodies


def _recalculate_normals(bpy, obj):
    """Point every face outward, because the boolean solver reads the winding.

    A traced hole walks the boundary in the opposite sense to the body it sits in,
    and an inside-out cutter turns a difference into a union — silently, and only in
    the middle of the icon.
    """
    view_layer = bpy.context.view_layer
    for other in view_layer.objects:
        other.select_set(False)
    obj.select_set(True)
    view_layer.objects.active = obj
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.mesh.normals_make_consistent(inside=False)
    bpy.ops.object.mode_set(mode="OBJECT")


def _prism(bpy, name, polygon, depth, collection, y_offset: float = 0.0):
    """One extruded polygon in the x/z plane, the rig's own depth axis being y."""
    count = len(polygon)
    vertices = [(point[0], y_offset - depth / 2.0, point[1]) for point in polygon]
    vertices += [(point[0], y_offset + depth / 2.0, point[1]) for point in polygon]
    # The far face, then the near face reversed, then one quad per edge. Winding is
    # stated rather than left to Blender: an inside-out prism still renders, and
    # would be a silent difference between the mesh and its own record.
    faces = [list(range(count - 1, -1, -1)), list(range(count, count * 2))]
    for edge in range(count):
        nxt = (edge + 1) % count
        faces.append([edge, nxt, count + nxt, count + edge])
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(vertices, [], faces)
    mesh.validate(verbose=False)
    obj = bpy.data.objects.new(name, mesh)
    collection.objects.link(obj)
    return obj


def _apply_modifier(bpy, obj, modifier):
    """Apply one modifier, with the object made active the way the op requires."""
    view_layer = bpy.context.view_layer
    for other in view_layer.objects:
        other.select_set(False)
    obj.select_set(True)
    view_layer.objects.active = obj
    bpy.ops.object.modifier_apply(modifier=modifier.name)


_MASK_CACHE = {}


def glyph_mask(bpy, glyph: str, alpha_threshold: float = 0.5) -> dict:
    """A glyph's mask record, measured once per run.

    Cached because the check asks for the same mark six times per icon, and the
    measurement walks every pixel of a 96x96 raster in Python.
    """
    key = (glyph, alpha_threshold)
    if key not in _MASK_CACHE:
        _MASK_CACHE[key] = _glyph_mask(bpy, glyph, alpha_threshold)
    return _MASK_CACHE[key]


def _glyph_mask(bpy, glyph: str, alpha_threshold: float = 0.5) -> dict:
    """Read an MD1 mark as coverage, centroid and an occupancy grid.

    Inside Blender because nothing in this pipeline has a PNG decoder: the runtime
    deliberately carries none, and `bpy.data.images` is the only reader there is.
    The MD1 marks are flat black on transparency, so the alpha channel is the
    silhouette and the colour channels carry no information.
    """
    import os

    width, height, pixels = _load_glyph(bpy, glyph)

    covered_at = []
    sum_x = sum_y = 0.0
    for row in range(height):
        for column in range(width):
            if pixels[(row * width + column) * 4 + 3] < alpha_threshold:
                continue
            # Blender hands back bottom-up rows; everything here is stated top-down.
            covered_at.append((column, height - 1 - row))
            sum_x += column
            sum_y += height - 1 - row
    if not covered_at:
        raise ValueError(f"`{glyph}` has nothing opaque; it cannot be a silhouette")

    covered = len(covered_at)
    xs = [cell[0] for cell in covered_at]
    ys = [cell[1] for cell in covered_at]
    min_x, max_x, min_y, max_y = min(xs), max(xs), min(ys), max(ys)
    del xs, ys
    # The same half-open bounds convention the geometry builder uses, so the mask and
    # the mesh are placed by one rule rather than by two that happen to agree today.
    bounds = (min_x, max_x + 1, min_y, max_y + 1)

    grid_side = 12
    grid = [0] * (grid_side * grid_side)
    placed = [0] * (grid_side * grid_side)
    outside = 0
    to_frame = placed_frame(bounds)
    for column, y in covered_at:
        grid[min(grid_side - 1, y * grid_side // height) * grid_side
             + min(grid_side - 1, column * grid_side // width)] += 1
        u, v = to_frame((column + 0.5, y + 0.5))
        if not (0.0 <= u < 1.0 and 0.0 <= v < 1.0):
            outside += 1
            continue
        placed[min(int(v * grid_side), grid_side - 1) * grid_side
               + min(int(u * grid_side), grid_side - 1)] += 1

    total = width * height
    cell_area = (width / grid_side) * (height / grid_side)
    # In the placed frame one mask pixel covers `scale` world units, and the frame is
    # ICON_FRAME_SPAN across, so a cell holds this many mask pixels.
    scale = placement_scale(bounds)
    placed_cell_area = (ICON_FRAME_SPAN / (grid_side * scale)) ** 2
    return {
        "glyph": glyph,
        "side_px": [width, height],
        "coverage": round(covered / total, 4),
        "centroid": [round(sum_x / covered / width, 4), round(sum_y / covered / height, 4)],
        "bounds": {"x": [min_x / width, max_x / width], "y": [min_y / height, max_y / height]},
        #: The grid the check reads: the mark as **placed** on the icon, which is the
        #: only frame a rendered frame can be compared against.
        "occupancy": [round(min(cell / placed_cell_area, 1.0), 3) for cell in placed],
        #: The reading it replaced, kept because a number that moved should stay
        #: readable: the mark binned where it sits in its own file, which is 0.3235 of
        #: the floor for `tool-road` and was never a fault in the geometry.
        "occupancy_file_frame": [round(cell / cell_area, 3) for cell in grid],
        "occupancy_grid": grid_side,
        "placed_outside_frame": outside,
        "frame_span": ICON_FRAME_SPAN,
        "placement_span": GLYPH_SPAN,
    }


# ---------------------------------------------------------------------------
# The composition: three materials, one silhouette, one accent
# ---------------------------------------------------------------------------


def lead_families(tier: str) -> dict:
    """The three distinct families an icon of this tier is made of."""
    if tier not in TIERS:
        raise KeyError(f"`{tier}` is not a declared tier: {', '.join(sorted(TIERS))}")
    lead = TIERS[tier]["lead"]
    secondary, accent = TIERS[tier]["pair"]
    return {"silhouette": lead, "secondary": secondary, "accent": accent}


def composition_materials(tier: str, hues: dict, accent_finish: str = "skeuomorph",
                          body_finish: str = None) -> dict:
    """A candidate's three materials, from the tier, its hues and the matrix.

    Exactly three base colours, and the accent is one of them told apart by finish —
    which is what keeps "three main colours" a property of every icon rather than a
    slogan about them.

    `body_finish` is the emphasis's own sheen (C8 a178): the ladder's gloss anchor is
    measured per language, so the surface that carries the body has to be a property
    of the emphasis and not of the icon. `None` leaves each family's declared surface
    in place, which is what the entry-level call wants.
    """
    families = lead_families(tier)
    materials = {}
    for role in COMPOSITION_ROLES:
        family = families[role]
        hue = hues.get(role, "natural")
        level = "edge" if role == "accent" else "body"
        finish = accent_finish if role == "accent" else body_finish
        # `family` travels with the material so the shared reference blend can own
        # the *surface* while the matrix owns the *colour*: the generator looks the
        # editable family material up in the blend and the base colour comes from
        # here, which is what "ROYGBIV versions derived from the main materials" means
        # when one of the two is editable by hand.
        parameters = {"base_color": rgba(palette.material_name(family, hue, level)),
                      "family": family}
        parameters.update(openpbr.family_surface(family, finish))
        materials[role] = parameters
    materials["ink"] = {"base_color": rgba("TextBody"), "family": "paper",
                        **openpbr.family_surface("paper", "matte")}
    return materials


#: One editable material per family, which is what the shared reference blend holds:
#: the surface an artist can tune, at the family's own untinted body colour. The
#: hue-varied materials are derived from these by the matrix rather than stored here,
#: so a change to a family's surface reaches every hue of it at once.
SURFACE_MATERIALS = {
    family: {"base_color": rgba(palette.material_name(family, "natural")),
             "family": family, **openpbr.family_surface(family)}
    for family in sorted(openpbr.FAMILY_SURFACE)
}

COMMON_MATERIALS = {
    "ink": mat("TextBody", 0.48, finish="matte"),
    "metal": mat("MetalBody", 0.28, 1.5, 0.9, 0.35, "brushed"),
    "accent": mat("MetalEdge", 0.22, 1.5, 0.9, 0.5, "polished"),
    "paper": mat("PaperBody", 0.7, 1.45, 0.0, 0.0, "paper"),
    "glass": mat("GlassLens", 0.12, 1.52, 0.0, 0.6, "glass"),
    "polymer": mat("PolymerGrip", 0.8, 1.45, 0.0, 0.0, "matte"),
    "road": mat("RoadSurface", 0.62, 1.45, 0.0, 0.0, "road"),
    "ceramic": mat("CeramicInsulator", 0.35, 1.48, 0.0, 0.25, "polished"),
    "blue": mat("IconBlue", 0.24, 1.5, 0.0, 0.45, "enamel"),
    "black": mat("IconBlack", 0.55, 1.45, 0.0, 0.08, "matte"),
}


def brief(role, cues, material, size_read, motion="static-ready"):
    return {
        "candidate_role": role,
        "identity": material,
        "semantic_cues": cues,
        "material_family": material,
        "size_read": size_read,
        "motion": {"eligibility": motion, "trigger": "store-backed only",
                   "reduced_motion": "static frame"},
        "fallback": "named solid with the same silhouette and role token",
        "acceptance": ["recognizable at 32px", "same meaning at 24px",
                       "no authority or live-state implication"],
    }


def candidate(base, candidate_id, label, role, parts, cues, material, why):
    item = dict(base)
    item["parts"] = parts
    item["generation"] = candidate_id
    item["generation_label"] = label
    item["generation_why"] = why
    item["brief"] = dict(base["brief"])
    item["brief"]["candidate_role"] = role
    item["brief"]["semantic_cues"] = cues
    item["brief"]["material_family"] = material
    item["brief"]["lineage"] = base["lineage"]
    return item


def six(base, alternate_recipes):
    """One canonical and five explicit alternate concepts.

    There is no separate "detailed" slot: every alternate brief must contain its
    own meaningful construction, and the canonical is the restrained control.
    """
    candidates = [candidate(
        base, "a-canonical", "canonical", "canonical", base["parts"],
        base["brief"]["semantic_cues"], base["brief"]["material_family"],
        "the clearest authored reading",
    )]
    for index, recipe in enumerate(alternate_recipes, 1):
        candidates.append(candidate(
            base, f"{chr(97 + index)}-alternate-{recipe['name']}",
            f"alternate {index}", "alternate", recipe["parts"],
            recipe["cues"], recipe["material"], recipe["why"],
        ))
    return candidates


PILOT_BASES = []
PILOT_CANDIDATES = {}


#: The pre-C3 pilot form, retired rather than deleted.
#:
#: C3 recorded why: the six pilots stood on no reference object at all — "no
#: knowledgable shape except for the settings icon, which was programatically
#: recreating the touchwiz icon, but badly" — so they are kept as the historical
#: record of a discarded form and nothing built here reaches the inventory. Retiring
#: rather than deleting is the same rule the game's own record uses.
RETIRED_PILOTS = {}


def add_pilot(icon_id, meaning, source, locates, parts, cues, material,
              lineage, forbidden, recipes, notes=()):
    """Retired: registers into RETIRED_PILOTS, contributes no inventory entry."""
    RETIRED_PILOTS[icon_id] = {
        "id": icon_id, "meaning": meaning, "source": source, "locates": locates,
        "parts": parts, "cues": cues, "material_family": material,
        "lineage": lineage, "forbidden_readings": forbidden, "notes": list(notes or ()),
        "recipe_count": len(recipes),
        "retired_because": (
            "C3: the pilot form had no reference object behind it; its silhouette is "
            "discarded and its recipes are kept only as history"
        ),
    }


PILOT = PILOT_BASES


add_pilot(
    "vocab-settings", "settings", "MD1 grammar / Unified Design", "settings surface",
    gear_parts(10, detail=False, role="ceramic", hub_role="blue", face_role="ceramic"),
    ["rounded radial teeth", "blue inset control disc", "white bezel", "dark centre"],
    "ceramic gear with blue enamel control",
    ["MD1 radial control grammar", "TouchWiz material gear language", "Popit rim restraint"],
    ["authority grant", "live powered state"],
    [
        {"name": "gear-detail", "parts": gear_parts(10, detail=True, role="ceramic", hub_role="blue", face_role="ceramic"),
         "cues": ["teeth", "blue control disc", "white bezel", "dark centre", "soft edge highlight"],
         "material": "ceramic and blue enamel", "why": "adds the layered control construction seen in the study grammar"},
        {"name": "control-dial", "parts": gear_parts(8, 0.7, 0.2, detail=True),
         "cues": ["radial dial", "index notch", "hub"], "material": "polished metal",
         "why": "alternate radial control silhouette"},
        {"name": "rosette", "parts": gear_parts(12, 0.68, 0.12, detail=True),
         "cues": ["fine radial rosette", "central bearing"], "material": "enamel metal",
         "why": "alternate compact radial construction"},
        {"name": "plaque-gear", "parts": [
            B(0.0, 0.0, 1.48, 1.48, 0.28, role="metal", finish="brushed"),
            G(0.0, 0.0, 0.56, 8, 0.16, 0.34, role="accent", finish="polished"),
            C(0.0, 0.0, 0.16, 0.42, role="metal", vertices=48, finish="brushed"),
         ], "cues": ["gear mounted on plaque", "layered frame"], "material": "metal plaque",
         "why": "alternate contextual plaque construction"},
        {"name": "hub-mark", "parts": [C(0, 0, 0.72, 0.34, "metal", 10), R(0, 0, 0.5, 0.1, role="accent"), C(0, 0, 0.14, 0.42, role="metal", vertices=32)],
         "cues": ["mechanical hub", "radial edge"], "material": "brushed metal",
         "why": "alternate compact mechanical emblem"},
    ],
)

add_pilot(
    "tool-road", "the road tool", "game surface: Tool::Road", "Tool::Road",
    road_parts(), ["raised lane", "edge rails", "lane markings"], "road plaque",
    ["Google Material Design road/alt-route silhouette", "iOS 6 utility locator as final fallback", "Popit contextual tool surface"],
    ["capability grant", "traffic state"],
    [
        {"name": "road-detail", "parts": road_parts(detail=True),
         "cues": ["road plaque", "lane marks", "edge rails", "layered curb"],
         "material": "road and brushed metal", "why": "adds curb construction"},
        {"name": "junction", "parts": road_parts("junction", True),
         "cues": ["four-way junction", "crossing lanes"], "material": "matte road",
         "why": "alternate plan-view junction silhouette"},
        {"name": "turn", "parts": [B(0, 0, 0.54, 1.72, 0.3, role="road"), B(0.3, 0.45, 0.72, 0.48, 0.3, angle=35, role="road"), B(0, 0.5, 0.08, 0.35, role="accent")],
         "cues": ["turning road", "lane cue"], "material": "road plaque",
         "why": "alternate turn silhouette"},
        {"name": "bridge", "parts": [B(0, 0.25, 1.45, 0.48, 0.32, role="road"), B(0, -0.45, 1.1, 0.22, 0.32, role="metal"), B(-0.62, -0.1, 0.12, 0.8, role="metal"), B(0.62, -0.1, 0.12, 0.8, role="metal")],
         "cues": ["raised crossing", "road deck"], "material": "road and metal",
         "why": "alternate infrastructure reading"},
        {"name": "lane-plaque", "parts": [B(0, 0, 1.42, 0.82, 0.28, role="road"), B(0, 0, 0.1, 0.64, role="accent"), B(-0.58, 0, 0.1, 0.64, role="metal"), B(0.58, 0, 0.1, 0.64, role="metal")],
         "cues": ["lane slab", "edge rails"], "material": "road plaque",
         "why": "alternate compact road marker"},
    ],
)

add_pilot(
    "tool-power", "the power tool", "game surface: Tool::Power", "Tool::Power",
    pylon_parts(), ["tower", "crossarm", "insulators"], "brushed metal and ceramic",
    ["Google Material Design power/bolt silhouette", "iOS 6 utility object as final fallback", "Unified Design service locator"],
    ["powered state", "authority grant"],
    [
        {"name": "pylon-detail", "parts": pylon_parts(detail=True), "cues": ["tower", "crossarm", "insulators", "cables"], "material": "metal and ceramic", "why": "adds construction members and insulators"},
        {"name": "network-node", "parts": pylon_parts("node", True), "cues": ["central transformer", "four network arms"], "material": "brushed metal", "why": "alternate service network silhouette"},
        {"name": "transformer", "parts": [B(0, 0, 0.9, 0.9, role="metal"), C(0, 0, 0.42, 0.42, role="ceramic", vertices=12), B(-0.72, 0, 0.22, 0.12, role="accent"), B(0.72, 0, 0.22, 0.12, role="accent")], "cues": ["transformer body", "line terminals"], "material": "metal and ceramic", "why": "alternate service object"},
        {"name": "crossarm", "parts": [B(0, 0, 0.18, 1.64, role="metal"), B(0, 0.58, 1.5, 0.18, role="metal"), C(-0.52, 0.58, 0.14, role="ceramic", vertices=24), C(0.52, 0.58, 0.14, role="ceramic", vertices=24)], "cues": ["crossarm", "insulators", "mast"], "material": "metal and ceramic", "why": "alternate tall pylon silhouette"},
        {"name": "service-loop", "parts": [R(0, 0, 0.62, 0.12, role="accent"), B(0, -0.52, 0.18, 0.72, role="metal"), C(0, 0.52, 0.2, role="ceramic", vertices=24)], "cues": ["service loop", "anchor", "insulator"], "material": "metal and ceramic", "why": "alternate compact network mark"},
    ],
)

add_pilot(
    "tool-inspect", "the inspect lens", "game surface: Tool::Inspect", "Tool::Inspect",
    lens_parts(), ["circular lens", "bezel", "handle"], "glass, polished metal, polymer",
    ["Google Material Design search/zoom silhouette", "iOS 6 search cue as final fallback", "Popit tool affordance"],
    ["search authority", "verified state"],
    [
        {"name": "lens-detail", "parts": lens_parts(True), "cues": ["housing", "inset lens", "glass highlight", "collar", "grip"], "material": "glass and metal", "why": "adds optical layers and a readable handle transition"},
        {"name": "loupe", "parts": [C(-0.16, 0.2, 0.68, 0.28, role="metal", vertices=64, finish="polished"), C(-0.16, 0.2, 0.48, 0.34, role="glass", vertices=64, finish="glass"), C(0.3, -0.32, 0.16, 0.34, role="metal", vertices=32), B(0.56, -0.6, 0.84, 0.2, angle=-45, role="polymer")], "cues": ["loupe housing", "glass centre", "collar", "short grip"], "material": "glass and polymer", "why": "compact Material-style search silhouette"},
        {"name": "scope", "parts": [B(-0.1, 0.2, 1.1, 0.5, angle=-12, role="metal", finish="brushed"), C(-0.58, 0.32, 0.3, 0.3, role="glass", vertices=48, finish="glass"), C(0.4, 0.05, 0.18, 0.3, role="metal", vertices=32), B(0.62, -0.38, 0.54, 0.2, angle=35, role="polymer")], "cues": ["inspection scope", "front lens", "rear collar", "handle"], "material": "glass and metal", "why": "alternate instrument silhouette with explicit optical ends"},
        {"name": "bezel", "parts": [C(-0.22, 0.2, 0.68, 0.32, role="metal", vertices=64, finish="polished"), R(-0.22, 0.2, 0.5, 0.08, role="accent", finish="polished"), C(-0.22, 0.2, 0.42, 0.34, role="glass", vertices=64, finish="glass"), B(0.38, -0.48, 0.72, 0.22, angle=-45, role="metal", finish="brushed")], "cues": ["thick bezel", "glass", "metal handle"], "material": "polished metal and glass", "why": "alternate heavy optical housing"},
        {"name": "inspection-mark", "parts": [C(-0.16, 0.2, 0.62, 0.3, role="glass", vertices=48, finish="glass"), R(-0.16, 0.2, 0.62, 0.1, role="accent", finish="polished"), C(0.3, -0.38, 0.15, 0.3, role="metal", vertices=32), B(0.48, -0.62, 0.4, 0.18, angle=-45, role="metal")], "cues": ["lens mark", "rim", "collar", "short stem"], "material": "glass and metal", "why": "alternate emblematic inspect mark with a complete optical silhouette"},
    ],
)

add_pilot(
    "ledger", "the ticket board", "game surface: the ledger panel", "the ledger panel",
    plaque_parts(detail=False), ["spine", "stacked records", "ruled pages"], "paper and brushed metal",
    ["Google Material Design description/inventory silhouette", "iOS 6 record surface as final fallback", "TouchWiz plaque construction"],
    ["promotion", "trust grade", "authority grant"],
    [
        {"name": "ledger-detail", "parts": plaque_parts(True), "cues": ["spine", "paper stack", "rules", "metal edge"], "material": "paper and brushed metal", "why": "adds ruled construction"},
        {"name": "record-stack", "parts": [B(-0.45, 0, 0.2, 1.5, role="metal"), B(0.2, 0.45, 1.2, 0.44, role="paper"), B(0.2, -0.1, 1.2, 0.44, role="paper"), B(0.2, -0.65, 1.2, 0.44, role="paper")], "cues": ["stacked cards", "spine", "records"], "material": "paper and metal", "why": "alternate card-stack silhouette"},
        {"name": "open-book", "parts": [B(-0.4, 0, 0.66, 1.36, angle=-8, role="paper"), B(0.4, 0, 0.66, 1.36, angle=8, role="paper"), B(0, 0, 0.12, 1.48, role="metal")], "cues": ["open pages", "binding", "record"], "material": "paper and metal", "why": "alternate open record silhouette"},
        {"name": "plaque-record", "parts": [B(0, 0, 1.5, 1.4, role="metal"), B(0, 0.35, 1.0, 0.16, role="paper"), B(0, 0, 0.8, 0.16, role="paper"), B(0, -0.35, 0.6, 0.16, role="paper")], "cues": ["plaque", "record rules", "inset paper"], "material": "metal and paper", "why": "alternate material-forward record"},
        {"name": "ledger-tab", "parts": [B(0, 0, 1.2, 1.5, role="paper"), B(0.38, 0.58, 0.48, 0.24, role="accent"), B(0, -0.2, 0.74, 0.12, role="metal"), B(0, 0.12, 0.74, 0.12, role="metal")], "cues": ["page", "tab", "rules"], "material": "paper and metal", "why": "alternate compact record locator"},
    ],
)

add_pilot(
    "ticket", "one ticket", "game surface: a ticket in the ledger", "a ticket in the ledger",
    plaque_parts(detail=False, ticket=True), ["card", "perforation", "seal"], "paper and polished metal",
    ["Google Material Design confirmation-number silhouette", "iOS 6 record badge as final fallback", "Unified Design record locator"],
    ["success", "trust grade", "promotion"],
    [
        {"name": "ticket-detail", "parts": plaque_parts(True, ticket=True), "cues": ["card", "perforation", "seal", "rule lines"], "material": "paper and polished metal", "why": "adds perforation and seal construction"},
        {"name": "stub", "parts": [B(0, 0, 1.62, 0.86, role="paper"), B(-0.5, 0, 0.12, 0.8, role="accent"), B(0.5, 0, 0.12, 0.8, role="accent"), C(0, 0, 0.18, role="metal", vertices=32)], "cues": ["ticket stub", "perforated sides", "seal"], "material": "paper and metal", "why": "alternate stub silhouette"},
        {"name": "pass", "parts": [B(0, 0, 1.0, 1.5, angle=0, role="paper"), B(0, 0.35, 0.62, 0.12, role="metal"), C(0, -0.35, 0.18, role="accent", vertices=32)], "cues": ["pass card", "rule", "seal"], "material": "paper and metal", "why": "alternate vertical pass silhouette"},
        {"name": "seal-card", "parts": [B(0, 0, 1.42, 0.92, role="paper"), R(0, 0, 0.3, 0.09, role="accent"), C(0, 0, 0.12, role="metal", vertices=24)], "cues": ["card", "round seal", "paper body"], "material": "paper and polished metal", "why": "alternate seal-forward construction"},
        {"name": "perforated", "parts": [B(0, 0, 1.5, 0.8, role="paper"), R(-0.48, 0, 0.1, 0.07, role="paper"), R(0.48, 0, 0.1, 0.07, role="paper"), B(0, 0, 0.7, 0.08, role="metal")], "cues": ["perforated card", "central rule"], "material": "paper and metal", "why": "alternate compact ticket mark"},
    ],
)

ICONS = PILOT_BASES
SURFACES = ICONS
VOCABULARY = []
DECLARED_SURFACES = tuple(entry["locates"] for entry in ICONS)
VOCABULARY_REFUSALS = []

# ---------------------------------------------------------------------------
# The C3 inventory, as compositions
# ---------------------------------------------------------------------------

#: An icon's three main colours, chosen per icon from the declared anchors.
#: Read down the list the choices follow the game's own meanings rather than taste:
#: red for the destructive tool because `Refused` is red, amber for power because the
#: caution cluster is amber, and the two records carry a coloured tab so a page is
#: not mistaken for a blank.
INVENTORY = (
    {
        "id": "tool-road", "tier": "tool", "locates": "Tool::Road",
        "meaning": "the road tool",
        "reading": "the road tool, in the toolbar",
        "source": "MD1 maps/add_road silhouette; TouchWiz surface intent",
        "hues": {"silhouette": "blue", "secondary": "natural", "accent": "cyan"},
        "cues": ["road ribbon", "lane dashes", "cool metal body"],
        "forbidden": ["authority grant", "live powered state"],
    },
    {
        "id": "tool-power", "tier": "tool", "locates": "Tool::Power",
        "meaning": "the power tool",
        "reading": "the power tool, in the toolbar",
        "source": "MD1 action/power_settings_new silhouette; TouchWiz surface intent",
        "hues": {"silhouette": "amber", "secondary": "natural", "accent": "red"},
        "cues": ["power ring", "open gap", "warm amber body"],
        "forbidden": ["authority grant", "a live supply reading"],
    },
    {
        "id": "tool-inspect", "tier": "tool", "locates": "Tool::Inspect",
        "meaning": "the inspect tool",
        "reading": "the inspect tool, in the toolbar",
        "source": "MD1 action/search silhouette; TouchWiz surface intent",
        "hues": {"silhouette": "cyan", "secondary": "natural", "accent": "blue"},
        "cues": ["lens ring", "open lens", "handle"],
        "forbidden": ["authority grant", "a verdict or grade"],
    },
    {
        "id": "tool-demolish", "tier": "tool", "locates": "Tool::Demolish",
        "meaning": "the demolish tool",
        "reading": "the demolish tool, in the toolbar",
        "source": "MD1 action/delete silhouette; TouchWiz surface intent",
        "hues": {"silhouette": "red", "secondary": "natural", "accent": "amber"},
        "cues": ["lid", "can", "warning-adjacent accent"],
        "forbidden": ["authority grant", "a completed refusal"],
    },
    {
        "id": "vocab-settings", "tier": "mark", "locates": "mark::settings",
        "meaning": "the settings mark",
        "reading": "the settings mark, on the panel chrome",
        "source": "MD1 action/settings silhouette; the study's own blue control face",
        "hues": {"silhouette": "natural", "secondary": "natural", "accent": "cyan"},
        "cues": ["gear teeth", "open centre", "blue enamel control face"],
        "forbidden": ["authority grant", "live powered state"],
    },
    {
        "id": "ticket", "tier": "record", "locates": "record::ticket",
        "meaning": "one ticket",
        "reading": "a ticket, in the ledger panel",
        "source": "MD1 notification/confirmation_number silhouette; TouchWiz paper intent",
        "hues": {"silhouette": "natural", "secondary": "natural", "accent": "blue"},
        "cues": ["stub", "perforation", "coloured tab"],
        "forbidden": ["success", "trust grade", "promotion"],
    },
)

#: Surfaces the game declares that the catalogue does not draw, with the reason.
#: Recorded rather than left implicit, because a missing icon is a fact about the
#: set and silence about it would be the one thing this pipeline refuses.
DEFERRED_INVENTORY = (
    {"id": "ledger", "locates": "panel::ledger", "kind": "record",
     "why": (
         "MD1 has no ledger glyph, so C3's gap rule sends it to iOS 6; the record is "
         "deferred rather than an invented silhouette being drawn"
     )},
    {"id": "tool-zone", "locates": "Tool::Zone", "kind": "tool",
     "why": "no MD1 glyph and no zone surface in the game yet"},
)

#: How much of the mark's own span each layer covers, and how far forward it sits.
#: Derived from GLYPH_SPAN rather than from any one glyph, because the traced mark is
#: fitted to its own bounding box and its world box is therefore known before the
#: mark is read: x and z both run -span/2 to +span/2 whatever the glyph is.
#: ...and the finish, because the ladder's gloss anchor is a *surface* reading (C8
#: a178). Each language's sheen is declared once here and travels with every
#: candidate of that emphasis, so "this candidate is TouchWiz-led" is a claim about
#: its surface as well as its construction. Before this the six candidates of one
#: icon shared a single material set, and the measured gloss of every slot was the
#: family's own brushed or paper surface whatever the emphasis said.
LAYERS = {
    "md1": {"depth": 0.28, "bevel": 0.014, "inset": 0.30, "second": 0.62,
            "finish": "matte"},
    "touchwiz": {"depth": 0.42, "bevel": 0.032, "inset": 0.78, "second": 0.34,
                 "finish": "polished"},
    "ios6": {"depth": 0.34, "bevel": 0.022, "inset": 0.52, "second": 0.42,
             "finish": "skeuomorph"},
}

#: The finish naming the four layers above, so the two tables cannot drift apart.
def emphasis_finish(emphasis_id: str) -> str:
    if emphasis_id not in LAYERS:
        raise KeyError(f"`{emphasis_id}` has no declared layer")
    return LAYERS[emphasis_id]["finish"]


def composition_parts(entry, emphasis: dict, construction: dict) -> list:
    """One candidate's parts: the traced silhouette, then what is laid over it."""
    glyph = silhouette_record(entry["id"])["glyph"]
    if not glyph:
        raise ValueError(f"`{entry['id']}` has no silhouette and cannot be composed")
    span = GLYPH_SPAN
    half = span / 2.0
    layer = LAYERS[emphasis["id"]]
    depth = layer["depth"]
    stack = construction["id"] == "stack"
    parts = [
        {"shape": "silhouette", "glyph": glyph, "role": "silhouette",
         "depth": depth, "bevel": layer["bevel"]},
    ]
    if stack:
        # Nested plates rather than one body: the same silhouette twice, the front
        # one inset, which is TouchWiz's layered construction and is also why the
        # second construction reads as a different object at 24 px.
        parts.append({"shape": "silhouette", "glyph": glyph, "role": "secondary",
                      "depth": depth * 0.30, "bevel": layer["bevel"] * 0.5,
                      "scale": 1.0 - layer["inset"] * 0.28, "y": depth * 0.62})
        parts.append({"shape": "silhouette", "glyph": glyph, "role": "accent",
                      "depth": depth * 0.18, "bevel": layer["bevel"] * 0.4,
                      "scale": 1.0 - layer["inset"] * 0.62, "y": depth * 0.74})
    else:
        # The single-body construction still carries all three materials: the inset
        # face is the secondary, and the accent is the record piece below. An earlier
        # version gave the inset the accent role, which left this candidate with two
        # materials and made "exactly three main colours" true of most icons rather
        # than of all of them.
        parts.append({"shape": "cylinder", "x": 0.0, "z": 0.0,
                      "radius": half * layer["second"], "depth": depth * 0.34,
                      "y": depth * 0.34, "vertices": 48, "role": "secondary"})
    # The accenting record piece: a tab or a rule, never a count. Round 11's Q52
    # settled that a count is a live text slot and is never baked into an icon.
    parts.append({"shape": "box", "x": half * 0.54, "z": -half * 0.54,
                  "w": half * 0.30, "h": half * 0.18, "depth": depth * 1.1,
                  "angle": -12.0, "y": depth * 0.30, "role": "accent"})
    if emphasis["id"] == "touchwiz":
        parts.append({"shape": "ring", "x": 0.0, "z": 0.0, "major": half * 0.86,
                      "minor": half * 0.045, "role": "secondary"})
    if emphasis["id"] == "ios6":
        parts.append({"shape": "ring", "x": 0.0, "z": 0.0, "major": half * 0.66,
                      "minor": half * 0.055, "role": "accent"})
        parts.append({"shape": "cylinder", "x": -half * 0.52, "z": half * 0.54,
                      "radius": half * 0.14, "depth": depth * 0.5,
                      "y": depth * 0.5, "vertices": 32, "role": "accent",
                      "finish": "skeuomorph"})
    return parts


def composition_brief(entry, emphasis: dict, construction: dict) -> dict:
    """The authored brief for one candidate, from the declaration it was built on."""
    hues = entry["hues"]
    families = lead_families(entry["tier"])
    return {
        "candidate_role": emphasis["id"],
        "identity": f"{families['silhouette']} {hues['silhouette']} body",
        "semantic_cues": list(entry["cues"]),
        "material_family": (
            f"{families['silhouette']} lead, {families['secondary']} secondary, "
            f"{families['accent']} accent"
        ),
        "size_read": "silhouette at 32px; the mark alone at 24px",
        "motion": {"eligibility": "static-ready", "trigger": "store-backed only",
                   "reduced_motion": "static frame"},
        "fallback": "named solid with the same silhouette and role token",
        "acceptance": [
            "recognisable at 32px",
            "the same mark at 24px",
            "no authority or live-state implication",
            "exactly three main materials",
        ],
        "silhouette": {
            "glyph": silhouette_record(entry["id"])["glyph"],
            "style": MD1_STYLE,
            "size_dp": MD1_SIZE_DP,
        },
        "emphasis": emphasis["id"],
        "construction": construction["id"],
        "surface_finish": emphasis_finish(emphasis["id"]),
        "hues": dict(hues),
    }


def six_compositions(entry) -> list:
    """The six candidates: three emphases, each in two constructions.

    The emphasis is which pillar leads, so the family spread is a choice the picker
    shows rather than a note it prints; the construction is how the body is built.
    Six slots, and every one is a whole composition under the C3 rule.
    """
    candidates = []
    letter = ord("a")
    for emphasis in EMPHASES:
        for construction in CONSTRUCTIONS:
            candidates.append({
                # `materials` travels with every candidate: the generator builds
                # the render's materials from the candidate it is rendering, and a
                # candidate without them would silently fall back to nothing.
                **{key: entry[key] for key in (
                    "id", "kind", "meaning", "source", "locates", "sits_on",
                    "identity", "identity_as", "framed", "brief", "lineage",
                    "forbidden_readings", "notes", "alternate", "tier", "hues",
                    "cues", "silhouette",
                )},
                # The emphasis's finish rides with the candidate's materials, because
                # the gloss anchor is per language (C8 a178).
                "materials": composition_materials(
                    entry["tier"], entry["hues"],
                    body_finish=emphasis_finish(emphasis["id"]),
                ),
                "parts": composition_parts(entry, emphasis, construction),
                "generation": f"{chr(letter)}-{emphasis['id']}-{construction['id']}",
                "generation_label": f"{emphasis['label']} · {construction['label']}",
                "generation_why": f"{emphasis['why']}; {construction['why']}",
                "brief": composition_brief(entry, emphasis, construction),
            })
            letter += 1
    return candidates


def add_composition(entry) -> None:
    """Register one C3 inventory entry and its six candidates."""
    glyph = silhouette_record(entry["id"])["glyph"]
    record = {
        "id": entry["id"], "kind": "surface", "meaning": entry["meaning"],
        "source": entry["source"], "locates": entry["locates"],
        "sits_on": ("Panel", "PanelRaised"),
        "identity": f"{lead_families(entry['tier'])['silhouette']} body",
        "identity_as": "detail-on-ink",
        "materials": composition_materials(entry["tier"], entry["hues"]),
        "parts": composition_parts(entry, EMPHASES[0], CONSTRUCTIONS[0]),
        "framed": False,
        "brief": {**composition_brief(entry, EMPHASES[0], CONSTRUCTIONS[0]),
                  "lineage": list(REFERENCE_STUDY["grammar"])},
        "lineage": [
            f"MD1 {glyph} silhouette ({MD1_STYLE}, {MD1_SIZE_DP}dp)",
            "TouchWiz surface and colour intent",
            "iOS 6 accent piece and layer",
        ],
        "forbidden_readings": list(entry["forbidden"]),
        "notes": [], "alternate": None,
        "tier": entry["tier"], "hues": dict(entry["hues"]),
        "cues": list(entry["cues"]),
        "silhouette": silhouette_record(entry["id"]),
    }
    PILOT_BASES.append(record)
    PILOT_CANDIDATES[entry["id"]] = six_compositions(record)


for _entry in INVENTORY:
    add_composition(_entry)

# Recomputed after the inventory is registered: the tuple above was built from an
# empty list, and a declared-surface check that cannot see the icons is the check
# C3 opened the session complaining about.
DECLARED_SURFACES = tuple(entry["locates"] for entry in ICONS)

#: The six review slots, as the manifest states them: three emphases, each in two
#: constructions. The letters are the slot order the files are named by.
TREATMENTS = [
    {
        "slot": slot, "id": f"{chr(ord('a') + slot - 1)}-{emphasis['id']}-{construction['id']}",
        "label": f"{emphasis['label']} · {construction['label']}",
        "candidate_role": emphasis["id"],
        "emphasis": emphasis["id"], "construction": construction["id"],
    }
    for slot, (emphasis, construction) in enumerate(
        [(emphasis, construction)
         for emphasis in EMPHASES
         for construction in CONSTRUCTIONS],
        start=1,
    )
]


def generations_of(entry):
    """Return the six explicit candidate records for one pilot icon."""
    candidates = PILOT_CANDIDATES[entry["id"]]
    if len(candidates) != 6:
        raise ValueError(f"{entry['id']} has {len(candidates)} concepts; exactly six are required")
    return candidates


def _smooth(obj):
    for polygon in obj.data.polygons:
        polygon.use_smooth = True
    try:
        normal = obj.modifiers.new(name="weighted authored normals", type="WEIGHTED_NORMAL")
        normal.keep_sharp = True
    except (AttributeError, TypeError):
        pass


def box(bpy, name, material, x, z, w, h, depth=0.34, angle=0.0, y=0.0):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=(x, y, z))
    obj = bpy.context.active_object
    obj.name = name
    obj.scale = (w, depth, h)
    obj.rotation_euler = (0.0, math.radians(angle), 0.0)
    obj.data.materials.append(material)
    bevel = obj.modifiers.new(name="authored rounded edge", type="BEVEL")
    bevel.width = min(w, h, depth) * 0.16
    bevel.segments = 4
    bevel.limit_method = "ANGLE"
    _smooth(obj)
    return obj


def ring(bpy, name, material, x, z, major, minor, angle=0.0, y=0.0):
    """`y` is the layer offset along the camera's depth axis, as in `box`."""
    bpy.ops.mesh.primitive_torus_add(major_radius=major, minor_radius=minor,
                                     major_segments=64, minor_segments=16,
                                     location=(x, y, z),
                                     rotation=(math.radians(90.0), 0.0, math.radians(angle)))
    obj = bpy.context.active_object
    obj.name = name
    obj.data.materials.append(material)
    _smooth(obj)
    return obj


def cylinder(bpy, name, material, x, z, radius, depth=0.34, vertices=48, y=0.0):
    """`y` is the layer offset along the camera's depth axis, as in `box`."""
    bpy.ops.mesh.primitive_cylinder_add(vertices=vertices, radius=radius,
                                        depth=depth, location=(x, y, z),
                                        rotation=(math.radians(90.0), 0.0, 0.0))
    obj = bpy.context.active_object
    obj.name = name
    obj.data.materials.append(material)
    bevel = obj.modifiers.new(name="authored rounded edge", type="BEVEL")
    bevel.width = min(radius, depth) * 0.16
    bevel.segments = 4
    _smooth(obj)
    return obj


def gear_mesh(bpy, name, material, x, z, radius, teeth=10, tooth=0.16, depth=0.34):
    """Build one radial body so bevels and highlights follow the actual silhouette."""
    import mathutils

    root = radius * 0.76
    tip = radius + tooth * 0.20
    points = []
    for index in range(teeth):
        centre = index * 2.0 * math.pi / teeth
        for fraction, ring_radius in ((-0.46, root), (-0.28, tip), (0.28, tip), (0.46, root)):
            angle = centre + fraction * math.pi / teeth
            points.append((math.cos(angle) * ring_radius, math.sin(angle) * ring_radius))
    half = depth * 0.5
    vertices = [(px + x, -half, pz + z) for px, pz in points]
    vertices += [(px + x, half, pz + z) for px, pz in points]
    count = len(points)
    faces = [tuple(range(count - 1, -1, -1)), tuple(range(count, count * 2))]
    for index in range(count):
        next_index = (index + 1) % count
        faces.append((index, next_index, count + next_index, count + index))
    mesh = bpy.data.meshes.new(f"{name} authored radial topology")
    mesh.from_pydata(vertices, [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    bevel = obj.modifiers.new(name="authored tooth and rim bevel", type="BEVEL")
    bevel.width = min(tooth * 0.32, depth * 0.22)
    bevel.segments = 5
    bevel.limit_method = "ANGLE"
    _smooth(obj)
    return obj


# ---------------------------------------------------------------------------
# The families: one body per object, from a parameter vector (C8 a179, a184)
# ---------------------------------------------------------------------------


def _bake(bpy, obj):
    """Apply every modifier already on an object, in stack order.

    A `box` arrives with its bevel already on the stack, and adding a boolean after
    it and applying the boolean alone makes Blender warn that it "was not first" —
    the operation then depends on stack order that nobody declared. Baking first
    means the body is a mesh with no modifiers before anything is cut out of it or
    welded to it, so the result is the declared one.
    """
    for modifier in list(obj.modifiers):
        _apply_modifier(bpy, obj, modifier)
    return obj


def _cut(bpy, body, cutters):
    """Subtract transient cutters from `body` and remove them.

    The cutters never render: they exist to leave a *hole* in one body, which is
    why the declared voids of a candidate (a gear's bore) are the cutters the
    builder knew about rather than something a person has to read back out of a
    raster.
    """
    _bake(bpy, body)
    for cutter in cutters:
        boolean = body.modifiers.new(name="declared void", type="BOOLEAN")
        boolean.operation = "DIFFERENCE"
        boolean.object = cutter
        boolean.solver = "EXACT"
    for modifier in [m for m in body.modifiers if m.type == "BOOLEAN"]:
        _apply_modifier(bpy, body, modifier)
    for cutter in cutters:
        bpy.data.objects.remove(cutter, do_unlink=True)
    return body


def _weld(bpy, body, others):
    """Union overlapping solids into one body, and remove the parts.

    a184 builds an object as **one profile**, and a183 refuses promotion for
    interpenetrating shells — so features that overlap the body (a marking raised
    on a road, a curb, a junction arm) cannot simply be left sitting in it. Welding
    resolves both at once: one shell, and the silhouette's piece count is the
    object's rather than the part list's.
    """
    _bake(bpy, body)
    for other in others:
        if other is None:
            continue
        boolean = body.modifiers.new(name="welded feature", type="BOOLEAN")
        boolean.operation = "UNION"
        boolean.object = other
        boolean.solver = "EXACT"
    for modifier in [m for m in body.modifiers if m.type == "BOOLEAN"]:
        _apply_modifier(bpy, body, modifier)
    for other in others:
        if other is not None:
            bpy.data.objects.remove(other, do_unlink=True)
    return body


def gear_body(bpy, name, material, vector, radius=None, depth=0.34):
    """One gear body, from a family vector.

    The tooth count is **fractional** (a187): `whole` teeth at the family's duty,
    plus one partial segment whose angular extent is the fraction, so the ring
    closes exactly and a count of 7.0 has no eighth tooth rather than a threshold
    at which one appears. The bore is a declared void cut through the body (a184),
    and `root_ratio` is measured from the reference object (0.76 in the table's
    own terms, 0.39 for its bore), not chosen here.
    """
    import math

    # The body's span comes from the composition budget (a200), not from the traced
    # mark's old 1.72: the margin the budget leaves is what a separate accent piece
    # fits into, and a fixed accent position collided at three lanes and up.
    radius = families.COMPOSITION["body_span"] * 0.5 if radius is None else radius
    count = vector["teeth"]
    duty = vector["tooth_duty"]
    root = radius * vector["root_ratio"]
    height = radius - root
    tip = radius + vector["tip_rise"] * height
    whole, fraction = families.features(count)
    pitch = 2.0 * math.pi / count
    segments = [(1.0, index) for index in range(whole)]
    if fraction > 0.0:
        segments.append((fraction, whole))

    points = []
    for position, (extent, index) in enumerate(segments):
        width = pitch * extent
        centre = index * pitch + width * 0.5
        land = duty * width * 0.5
        for offset, ring in ((-land, root), (-land * 0.55, tip),
                             (land * 0.55, tip), (land, root)):
            angle = centre + offset
            points.append((math.cos(angle) * ring, math.sin(angle) * ring))
        # The root land between two teeth **follows the root circle** instead of
        # being a straight chord across it. A chord dips inside the root radius, and
        # at nine teeth that dip left a one-pixel enclosed void for the bevel to
        # pinch -- the smoke test's only defect, and the class a196 refuses. Two
        # samples across the gap are enough to keep the land on the circle, and the
        # gap is measured so a narrow one is not over-sampled.
        if position + 1 < len(segments):
            next_extent, next_index = segments[position + 1]
            gap_start = centre + land
            gap_end = (next_index * pitch
                       + pitch * next_extent * (1.0 - duty) * 0.5)
            gap = gap_end - gap_start
            if gap > math.radians(4.0):
                for share in (1.0 / 3.0, 2.0 / 3.0):
                    angle = gap_start + gap * share
                    points.append((math.cos(angle) * root, math.sin(angle) * root))

    # The seam that closes the ring is a gap like any other, and it is the one the
    # loop above cannot see because it has no next segment. It was the smoke test's
    # only defect: at nine teeth — a whole number, where the seam lands on a pixel
    # boundary — its chord dip left a one-pixel enclosed void, the class a196 refuses.
    last_extent, last_index = segments[-1]
    last_width = pitch * last_extent
    last_end = last_index * pitch + last_width * (1.0 + duty) * 0.5
    first_width = pitch * segments[0][0]
    seam_end = 2.0 * math.pi + first_width * (1.0 - duty) * 0.5
    seam = seam_end - last_end
    if seam > math.radians(4.0):
        for share in (1.0 / 3.0, 2.0 / 3.0):
            angle = last_end + seam * share
            points.append((math.cos(angle) * root, math.sin(angle) * root))

    half = depth * 0.5
    vertices = [(x, -half, z) for x, z in points] + [(x, half, z) for x, z in points]
    side = len(points)
    faces = [tuple(range(side - 1, -1, -1)), tuple(range(side, side * 2))]
    for index in range(side):
        nxt = (index + 1) % side
        faces.append((index, nxt, side + nxt, side + index))
    mesh = bpy.data.meshes.new(f"{name} family topology")
    mesh.from_pydata(vertices, [], faces)
    mesh.update()
    body = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(body)
    # The boolean solver reads the winding, and an inside-out bore cutter turns a
    # difference into a union silently, so the normals are made consistent before
    # anything is cut out of this body.
    _recalculate_normals(bpy, body)
    body.data.materials.append(material)
    edge = body.modifiers.new(name="authored tooth bevel", type="BEVEL")
    edge.width = max(1e-4, vector["chamfer_ratio"] * height)
    edge.segments = 5
    edge.limit_method = "ANGLE"
    _smooth(body)

    bore = cylinder(bpy, f"{name} bore cutter", material,
                    0.0, 0.0, vector["bore_ratio"] * radius,
                    depth=depth * 2.0, vertices=64)
    _cut(bpy, body, [bore])
    _smooth(body)
    return body


def road_body(bpy, name, material, vector, span=None, depth=0.30):
    """One carriageway body, from a family vector.

    Lane count is fractional in the same way a tooth count is: the ribbon's width
    is `lanes * lane_width`, so the partial lane is simply narrower and appears as
    it grows. The **line type is a reading, not a switch**: `centre_dash_duty` at 0
    is one unbroken strip, and above 0 the strip is cut into dashes by that duty,
    which is how "solid", "dashed" and the rests between them stop being modes.
    Markings are *raised and welded* rather than cut, because a dash cut into the
    surface is an enclosed void and a small one is a sliver (a196).
    """
    span = families.COMPOSITION["body_span"] if span is None else span
    lanes = vector["lanes"]
    lane_width = vector["lane_width"] * span
    half = lanes * lane_width * 0.5
    length = span
    body = box(bpy, f"{name} ribbon", material, 0.0, 0.0,
               half * 2.0, length, depth=depth)
    features = []

    # Lane markings, raised on the ribbon's face and welded into it.
    boundary = 1
    while boundary < lanes - 1e-9:
        x = -half + boundary * lane_width
        if x < half - lane_width * 0.25:
            # The boundary nearest the ribbon's own centre is the centre line, and
            # it is the one the family's dash duty applies to; the others are lane
            # edges and stay solid. With an odd lane count no boundary sits at the
            # centre, so there is no dashed line at all rather than an arbitrary one.
            dashes = abs(x) < lane_width * 0.5
            duty = vector["centre_dash_duty"] if dashes else 0.0
            cycle = lane_width * 3.0
            if duty <= 0.0:
                features.append(box(bpy, f"{name} marking {boundary}", material,
                                    x, 0.0, lane_width * 0.10, length,
                                    depth=vector["marking_height"] * depth))
            else:
                step = 0
                while step < int(length / cycle) + 1:
                    z = -length / 2.0 + cycle * (step + duty * 0.5)
                    if z - cycle * duty * 0.5 < length / 2.0:
                        features.append(box(
                            bpy, f"{name} dash {boundary}.{step}", material,
                            x, z, lane_width * 0.10, cycle * duty,
                            depth=vector["marking_height"] * depth))
                    step += 1
        boundary += 1

    # Curbs, and a shoulder if the family asks for one.
    curb = vector["curb_width"] * span
    if curb > 1e-6:
        for side in (-1.0, 1.0):
            features.append(box(bpy, f"{name} curb {side:+.0f}", material,
                                side * (half + curb * 0.5), 0.0, curb, length,
                                depth=depth * 1.15))

    # Junction arms: whole arms plus one partial, whose length is the fraction, so
    # an arm grows out of the ribbon rather than appearing at a threshold.
    # The arms are read against the **margin the budget leaves** (a200), not against the
    # ribbon's span: a four-lane carriageway has almost no verge, and an arm measured
    # against the span reached 1.33 of a body budget that ends at 0.72, which pushed
    # the car's accent slot off the frame. Bounded by the budget, nothing overflows and
    # the accent keeps its room by construction.
    budget_half = families.COMPOSITION["body_span"] * 0.5
    margin = max(0.0, budget_half - half - curb)
    arm_length = vector["arm_length"] * margin
    if arm_length > 1e-6:
        whole, fraction = families.features(vector["junction_arms"])
        arms = [(1.0, index) for index in range(whole)]
        if fraction > 0.0:
            arms.append((fraction, whole))
        for extent, index in arms:
            if extent <= 0.0:
                continue
            side = 1.0 if index % 2 == 0 else -1.0
            z = (index // 2) * (span / 3.0) - span / 6.0
            features.append(box(bpy, f"{name} arm {index}", material,
                                side * (half + arm_length * extent * 0.5), z,
                                arm_length * extent, lane_width,
                                depth=depth * 0.82))

    _weld(bpy, body, features)

    # The pose (a203): the carriageway is laid out flat and then turned in its own
    # plane by `heading` and revealed by `pitch`, which is how a road reads as a
    # *tile* rather than a rectangle — the idiom the person pointed at. It is a
    # parameter rather than a fixed pose because the ladder is the axis: flat at the
    # md1 end, where the reference idiom is a flat mark, posed at the iOS 6 end. The
    # turn happens **after** the weld so one body turns, not a pile of parts.
    body.rotation_euler = (0.0, math.radians(vector["heading"]),
                           math.radians(vector["pitch"]))
    _smooth(body)
    return body


def _profile(bpy, name, material, points, depth, bevel=0.02):
    """One extruded polygon: the body of a family whose shape is not a box or a disc.

    `points` is the outline in the icon's own x/z plane, and the bevel is a fraction
    of the smallest edge, so a thin stroke and a wide slab round the same way rather
    than one of them losing its edge entirely.
    """
    half = depth * 0.5
    vertices = [(x, -half, z) for x, z in points] + [(x, half, z) for x, z in points]
    side = len(points)
    faces = [tuple(range(side - 1, -1, -1)), tuple(range(side, side * 2))]
    for index in range(side):
        nxt = (index + 1) % side
        faces.append((index, nxt, side + nxt, side + index))
    mesh = bpy.data.meshes.new(f"{name} profile")
    mesh.from_pydata(vertices, [], faces)
    mesh.update()
    body = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(body)
    _recalculate_normals(bpy, body)
    body.data.materials.append(material)
    if bevel > 1e-6:
        edge = body.modifiers.new(name="authored edge bevel", type="BEVEL")
        edge.width = bevel
        edge.segments = 4
        edge.limit_method = "ANGLE"
    _smooth(body)
    return body


def _stroke_outline(centre, miter_limit=2.0):
    """The outline of a **stroked polyline**, given `(x, z, half_width)` along it.

    A bolt is a stroke rather than a shape: it is a zigzag with a thickness, and both
    of those are parameters. Offsetting a polyline by its own half-width is what makes
    the thickness and the zigzag independent — the alternative (a stack of welded
    boxes) makes the thickness a property of the box and the corners a property of how
    the boxes happen to overlap.

    The joint reach is clamped, because a sharp zigzag's true miter fires a spike far
    outside the stroke, and a spike pinches off exactly the kind of sliver a196 refuses.
    """
    left, right = [], []
    count = len(centre)
    for index, (x, z, half_width) in enumerate(centre):
        normals = []
        if index > 0:
            dx, dz = x - centre[index - 1][0], z - centre[index - 1][1]
            length = math.hypot(dx, dz) or 1.0
            normals.append((dz / length, -dx / length))
        if index < count - 1:
            dx, dz = centre[index + 1][0] - x, centre[index + 1][1] - z
            length = math.hypot(dx, dz) or 1.0
            normals.append((dz / length, -dx / length))
        sx = sum(normal[0] for normal in normals)
        sz = sum(normal[1] for normal in normals)
        reach = math.hypot(sx, sz)
        if reach < 1e-9:
            sx, sz, reach = normals[0][0], normals[0][1], 1.0
        scale = min(miter_limit, 2.0 / reach)
        ox, oz = sx / reach * half_width * scale, sz / reach * half_width * scale
        left.append((x + ox, z + oz))
        right.append((x - ox, z - oz))
    return left + list(reversed(right))


def bolt_body(bpy, name, material, vector, height=None, depth=0.28):
    """One bolt of energy, from a family vector.

    The reference (`content/bolt`, 40x72, one piece, no voids) is a bolt drawn as a
    single solid mark, and it is re-pointed there from the power button a198 retired.
    Its three readings move independently: `stroke` is the stroke's thickness,
    `waist` the zigzag's amplitude as a fraction of the object's width, `tip_taper`
    narrows the ends so the object reads as a strike rather than a bar, and `steps`
    adds interior vertices — each new one arriving at **zero amplitude** (a187), so
    a flank that has not grown yet is a flank that is not there.
    """
    height = families.COMPOSITION["body_span"] if height is None else height
    width = vector["width_over_height"] * height
    stroke = vector["stroke"] * width
    whole, fraction = families.features(vector["steps"])
    joints = 2 + whole + (1 if fraction > 0.0 else 0)
    centre = []
    for index in range(joints + 1):
        along = index / joints
        z = height * 0.5 - height * along
        side = 1.0 if index % 2 == 0 else -1.0
        if index in (0, joints):
            # The tips sit on the opposite corners of the object's own box, which is
            # what gives the bolt its diagonal sweep at 40x72's proportion.
            half_width = stroke * 0.5 * vector["tip_taper"]
            x = side * (width * 0.5 - half_width)
        else:
            half_width = stroke * 0.5
            amplitude = vector["waist"] * width * 0.5
            if index == joints - 1 and fraction > 0.0:
                amplitude *= fraction
            x = side * amplitude
        centre.append((x, z, half_width))
    body = _profile(bpy, name, material, _stroke_outline(centre),
                    depth, bevel=min(stroke, depth) * 0.16)
    # `strike_angle` is the object leaning into its own direction of travel, the way
    # the reference's bolt leans rather than standing perfectly upright (a179).
    body.rotation_euler = (0.0, math.radians(vector["strike_angle"]), 0.0)
    _smooth(body)
    return body


def lens_body(bpy, name, material, vector, radius=None, depth=0.34):
    """One lens with a handle, from a family vector.

    Both of this family's defining ratios are read off the reference rather than
    chosen, and the ring is **thin because the measurements say it is**:
    `action/search`'s enclosing void is 1018 px against a 34.5 px box half (0.51)
    while its ink, 1374 px, is far less than the annulus that ratio would leave —
    so the ring's own outer radius is only 0.74 of the box half and its wall is the
    difference between the two. Sizing the bore against the *housing* instead was
    the first attempt, and it built a thick washer where the reference has a rim,
    which the fill reading then refused (0.479-0.721 against 0.18-0.38).
    """
    half_span = families.COMPOSITION["body_span"] * 0.5
    outer = half_span * vector["ring_ratio"] if radius is None else radius
    bore = vector["bore_ratio"] * half_span
    wall = max(1e-3, outer - bore)
    housing = cylinder(bpy, name, material, 0.0, 0.0, outer, depth=depth, vertices=64)
    rim_wall = max(1e-3, vector["ring_thickness"] * outer)
    features = []
    # `rings` counts the object's **concentric rims, the lens ring itself being the
    # first**: a count of 1 is one thin ring rather than a ring plus a decoration,
    # which is what the reference measures and what the first attempt got wrong by
    # welding a torus *on top of* a housing that was already the ring.
    cursor = outer
    whole, fraction = families.features(vector["rings"])
    rims = [(1.0, index) for index in range(1, whole)]
    if fraction > 0.0:
        rims.append((fraction, whole))
    for extent, index in rims:
        # Each rim overlaps the last (`major - minor` sits inside it) so the weld has
        # something to weld, and the partial rim's **wall is its extent** -- a rim that
        # has not grown yet has no wall, which is a187's rule applied to a rim.
        minor = rim_wall * extent
        major = cursor + minor * 0.5
        features.append(ring(bpy, f"{name} rim {index}", material, 0.0, 0.0,
                             major, minor, y=0.0))
        cursor = major + minor
    collar = vector["collar_width"] * outer
    if collar > 1e-6:
        # A collar is a rim too, and it has to be one: a *cylinder* welded around the
        # housing would fill the bore it is supposed to leave open.
        minor = collar * 0.5
        major = cursor + minor * 0.5
        features.append(ring(bpy, f"{name} collar", material, 0.0, 0.0, major, minor,
                             y=0.0))
        cursor = major + minor
    # The handle starts **inside** the ring rather than at its edge: a handle left
    # touching the rim is the floating-component class a183 refuses, and the first run
    # measured exactly that -- the body arriving in two pieces, 95 px of handle
    # detached at (63, 26). Its inner part is then removed by the bore cut, which is
    # what makes the bore a declared cutter rather than a hole the geometry implies.
    # The reach is read off the **ring**, not off the outermost rim: tying it to
    # `cursor` made the handle grow every time a ring was added, which grew the
    # object's bounds and took the accent's slot out of the frame at the iOS 6 end.
    angle = math.radians(vector["handle_angle"])
    reach = outer * vector["handle_length"]
    inset = outer * 0.4
    features.append(box(bpy, f"{name} handle", material,
                        math.cos(angle) * (inset + reach * 0.5),
                        math.sin(angle) * (inset + reach * 0.5),
                        reach, outer * 0.22, depth=depth * 0.62,
                        angle=vector["handle_angle"]))
    _weld(bpy, housing, features)
    # The bore is measured against the **object's half-span**, which is the frame the
    # reference's own 0.51 was read in, not against the ring's outer radius.
    bore_cutter = cylinder(bpy, f"{name} bore cutter", material, 0.0, 0.0, bore,
                           depth=depth * 3.0, vertices=64)
    _cut(bpy, housing, [bore_cutter])
    _smooth(housing)
    return housing


def bin_body(bpy, name, material, vector, height=None, depth=0.30):
    """One container with a lid, from a family vector.

    The reference (`action/delete`) measures its **lid as a separate component** with
    a gap between it and the body, so `lid_gap` is a measured parameter and not a
    styling choice — and the object therefore arrives in two declared parts, which is
    what the family declares rather than something the check has to guess. The mouth
    is a dip drawn into the container's top edge so it reads at a flat pose; a mouth
    cut into the top *face* would be invisible from straight on, which is a parameter
    that renders as nothing.
    """
    height = families.COMPOSITION["body_span"] * 0.74 if height is None else height
    top = families.COMPOSITION["body_span"] * 0.44
    bottom = top * vector["taper"]
    gap = vector["lid_gap"] * height
    lid_height = max(1e-3, vector["rim_thickness"] * height * 2.0)
    body_height = height - gap - lid_height
    mouth = vector["mouth_depth"] * body_height * 0.5
    shoulder = max(1e-4, top * 0.42)
    points = [
        (-bottom, -height * 0.5),
        (bottom, -height * 0.5),
        (top, -height * 0.5 + body_height),
        (shoulder, -height * 0.5 + body_height),
        (0.0, -height * 0.5 + body_height - mouth),
        (-shoulder, -height * 0.5 + body_height),
        (-top, -height * 0.5 + body_height),
    ]
    container = _profile(bpy, name, material, points, depth, bevel=0.012)

    whole, fraction = families.features(vector["ridges"])
    ridges = [(1.0, index) for index in range(whole)]
    if fraction > 0.0:
        ridges.append((fraction, whole))
    features = []
    for extent, index in ridges:
        # Ridges are **vertical ribs** on the container's face, spaced across its
        # width. The count is fractional in the same way every other count is: the
        # partial rib is narrower and arrives as it grows (a187), so a bin with no
        # ridges draws none rather than one at a threshold.
        across = (index + 0.5) / max(1.0, float(whole + (1 if fraction else 0)))
        x = (bottom + (top - bottom) * 0.5) * (across * 2.0 - 1.0)
        features.append(box(bpy, f"{name} ridge {index}", material,
                            x, -height * 0.5 + body_height * 0.5,
                            top * 0.09 * extent, body_height * 0.68 * extent,
                            depth=depth * 1.06))
    _weld(bpy, container, features)

    lip = top * (1.0 + vector["lid_lip"])
    lid = _profile(bpy, name + " lid", material, [
        (-lip, -height * 0.5 + body_height + gap),
        (lip, -height * 0.5 + body_height + gap),
        (lip * 0.96, -height * 0.5 + body_height + gap + lid_height),
        (-lip * 0.96, -height * 0.5 + body_height + gap + lid_height),
    ], depth * 1.08, bevel=0.012)
    handle = box(bpy, f"{name} handle", material, 0.0,
                 -height * 0.5 + body_height + gap + lid_height * 0.55,
                 lip * vector["handle_width"], lid_height * 0.9,
                 depth=depth * 0.55)
    _weld(bpy, lid, [handle])
    _smooth(container)
    # The lid is a declared second part (the family says so); it is welded to its own
    # handle and left gap-separated from the container, exactly as the reference draws
    # it, so the object's piece count is a reading of the declaration rather than of
    # how the parts happen to have been stacked. Both are already linked by the
    # builder that made them, which is why nothing re-links them here.
    return container, lid


def plaque_body(bpy, name, material, vector, width=None, depth=0.20):
    """One punched card, from a family vector.

    The notches are **declared voids** cut through the card, and their size is not a
    taste: a199 settled that the ticket's notches are drawn *above the 3.5 % floor*
    the sliver rule gates, even though the reference's own notches measure 1.37 % —
    so `notch_diameter` starts just above the smallest diameter that clears the floor
    for a card of this aspect, and the divergence from the reference is recorded
    rather than the rule softened.
    """
    width = families.COMPOSITION["body_span"] if width is None else width
    height = width / vector["aspect"]
    card = _profile(bpy, name, material, [
        (-width * 0.5, -height * 0.5), (width * 0.5, -height * 0.5),
        (width * 0.5, height * 0.5), (-width * 0.5, height * 0.5),
    ], depth, bevel=vector["corner_rounding"] * height * 0.25)
    features = []
    whole, fraction = families.features(vector["rules"])
    rules = [(1.0, index) for index in range(whole)]
    if fraction > 0.0:
        rules.append((fraction, whole))
    stub = width * vector["stub_width"]
    for extent, index in rules:
        x = -width * 0.5 + stub * (index + 1.0) / max(1.0, float(whole + 1))
        features.append(box(bpy, f"{name} rule {index}", material,
                            x, 0.0, height * 0.045, height * 0.82 * extent,
                            depth=depth * 1.04))
    tab = width * vector["tab_width"]
    if tab > 1e-6:
        features.append(box(bpy, f"{name} tab", material, width * 0.5 + tab * 0.35,
                            0.0, tab, height * 0.5, depth=depth * 0.9))
    _weld(bpy, card, [feature for feature in features if feature is not None])

    whole, fraction = families.features(vector["notches"])
    cutters = []
    slots = [(1.0, index) for index in range(whole)]
    if fraction > 0.0:
        slots.append((fraction, whole))
    for extent, index in slots:
        across = width * (-0.30 + 0.30 * index)
        edge = height * 0.30 if index % 2 == 0 else -height * 0.30
        cutters.append(cylinder(bpy, f"{name} notch cutter {index}", material,
                                across, edge,
                                vector["notch_diameter"] * height * 0.5 * extent,
                                depth=depth * 3.0, vertices=32))
    if cutters:
        _cut(bpy, card, cutters)
    _smooth(card)
    return card


def build(bpy, entry, materials):
    objects = []
    for index, part in enumerate(entry["parts"]):
        role = part.get("role", "ink")
        material = materials.get(role, materials.get("ink"))
        shape = part["shape"]
        if shape == "box":
            objects.append(box(bpy, f"{entry['id']} {index}", material, **{
                key: value for key, value in part.items() if key not in ("shape", "role", "finish")
            }))
        elif shape == "ring":
            objects.append(ring(bpy, f"{entry['id']} {index}", material, **{
                key: value for key, value in part.items() if key not in ("shape", "role", "finish")
            }))
        elif shape == "cylinder":
            objects.append(cylinder(bpy, f"{entry['id']} {index}", material, **{
                key: value for key, value in part.items() if key not in ("shape", "role", "finish")
            }))
        elif shape == "gear":
            objects.append(gear_mesh(bpy, f"{entry['id']} {index}", material, **{
                key: value for key, value in part.items() if key not in ("shape", "role", "finish")
            }))
        elif shape == "silhouette":
            # The MD1 mark, traced and placed, in whatever role the composition gave
            # it. `y` is the layer offset the composition declared.
            outline = outline_for(bpy, part["glyph"])
            objects.extend(build_silhouette(
                bpy, f"{entry['id']} {index}", material, outline,
                depth=part.get("depth", 0.36), bevel=part.get("bevel", 0.02),
                scale=part.get("scale", 1.0), y_offset=part.get("y", 0.0),
            ))
        else:
            raise ValueError(f"unknown authored shape {shape!r}")
    return objects


def swatch_geometry(bpy, material):
    return [box(bpy, "swatch plate", material, 0.0, 0.0, 1.5, 1.5, depth=0.06)]
