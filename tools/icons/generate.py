"""Render the inventory in generations, measure everything, and write the record.

Run it with Blender, which is the only thing here that needs Blender:

    blender --background --factory-startup --python tools/icons/generate.py -- --review surface

What it writes:

* `assets/icons/review/<id>.<generation>.<px>.png` — six authored concepts of one
  icon (one canonical plus five named alternates), which is what a person chooses
  between. This is the review set, not the shipping set.
* `assets/icons/review/<id>.<generation>.96.rgba` — the decision size as raw RGBA,
  which is what the picker and the game read. The runtime carries **no PNG
  decoder**: PNG is for people, RGBA is for the program, and the manifest says
  which is which.
* `assets/icons/review.json` — the picker's input: what awaits a decision, the
  six candidates' authored briefs, measurements, quality record, and the identity
  of the build that made them.
* `assets/icons/manifest.json` — provenance, materials, deviations, measurements.
* `assets/icons/<id>.<px>.png|rgba` — the shipping assets, written **only** for an
  icon whose chosen generation is recorded in `review-decisions.jsonl`. Nothing
  ships that a person has not marked as the target.

Every measurement is recorded even when it is a failure, and the verdict says
which. A generator that only writes passing icons is a generator nobody can audit.
"""

import json
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
if HERE not in sys.path:
    sys.path.insert(0, HERE)

import bpy  # noqa: E402  (only available inside Blender)

import openpbr  # noqa: E402
import palette  # noqa: E402
import rig  # noqa: E402
import shapes  # noqa: E402

ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
OUT = os.path.join(ROOT, "assets", "icons")
REVIEW = os.path.join(OUT, "review")
DECISIONS = os.path.join(OUT, "review-decisions.jsonl")

#: The size a decision is made at: the largest shipped size, because a generation
#: is judged on what it looks like when it has room, and the smallest size is
#: recorded beside it so a choice that dies at 24 px can be seen to.
DECISION_PX = max(rig.SHIPPED_PX)
RECOGNITION_PX = 32
CONTEXT_PX = min(rig.SHIPPED_PX)

# ---------------------------------------------------------------------------
# Tolerances. Stated rather than tuned: a check with an invisible threshold is a
# check nobody can argue with.
# ---------------------------------------------------------------------------

#: A material swatch is rendered under the same rig for every icon, and an icon is
#: compared to its own swatch. Two earlier versions of this check were wrong in
#: opposite directions and both are worth remembering: one used an invented
#: irradiance model and called two correct icons failures; the other compared
#: against the albedo with no lighting term and called *every* icon a failure at
#: 0.07-0.35x, because a shaded box is dimmer than its own albedo. The lesson is in
#: those numbers — any model of "how bright it should be" was going to be wrong —
#: so the reference is measured instead of reasoned about.
ICON_TO_SWATCH_BAND = (0.4, 1.8)

#: How far a flat swatch of a material may sit from its own albedo under this rig.
#: Not a physical claim: it is the rig's illumination, recorded so a wrong token, a
#: broken light or a post-processed render has nowhere to hide.
SWATCH_TO_ALBEDO_BAND = (0.35, 1.35)

#: How far the rendered chromaticity may sit from the declared colour's, in degrees
#: of the red/green ratio plane. Wide enough for three coloured lights, narrow
#: enough to catch a wrong token.
CHROMATICITY_TOLERANCE_DEGREES = 12.0

#: Ink coverage at the smallest shipped size. Below the floor the glyph has
#: averaged away; above the ceiling it is a blob and carries no information.
COVERAGE_FLOOR_24 = 0.06
#: How far apart two candidates must measure to count as two readings, as a fraction
#: of the span between the two extreme corpus anchors. Ten percent: closer than that
#: and the "difference" is inside the ladder's own resolution, because the two
#: anchors being compared differ by far more than the noise between them.
LADDER_SEPARATION_FRACTION = 0.10
COVERAGE_CEILING_24 = 0.62

#: A framed view icon carries a border, and the border must not become the icon.
FRAMED_COVERAGE_CEILING_24 = 0.55

#: An accent drawn against the icon's own body needs at least this much separation
#: from it, or the detail is decoration that cannot be read.
INTERNAL_MIN_CONTRAST = 1.5

#: Below this much separation between a colour's channels, the hue check is noise.
NEUTRAL_CHROMA_THRESHOLD = 0.06
#: Above this metalness a material has essentially no diffuse term, so its flat
#: swatch is not a reading of the declared colour at all: it is a reading of how
#: much the rig happens to be reflecting. Under this project's black world with
#: area lights, every metal family member measures the same 0.075-0.088x whatever
#: its light/level is, which is a number about the rig and not about the icon. So
#: the swatch/icon-source band is skipped there, with the reason recorded -- the
#: same treatment the near-neutral hue case gets -- and the host-contrast check
#: still does the work of saying whether the material reads against its surface.

#: Surface locator identity colours describe the role of a surface, not its
#: current state. State tokens are reserved for store-backed badges and status
#: marks; using one as the base identity makes a static icon falsely look live.
STATE_IDENTITY_TOKENS = {
    "Nature", "Powered", "Warning", "Refused", "NotObtained", "CaseOpen",
    "Agent", "Verified", "Correction", "Retired",
}

#: A surface icon must have enough authored parts to survive as a symbol rather
#: than a single block. This is a floor, not a style target; the detail pass may
#: add more parts but may not make the authoring set simpler.
MIN_SURFACE_PARTS = 3


def linear(pixel: float) -> float:
    return rig.linearise(pixel)


def relative_luminance(rgb) -> float:
    """WCAG relative luminance, the same measure the interface uses."""
    return palette.relative_luminance(rgb)


def chromaticity_angle(rgb) -> float:
    """An angle in the red/green plane: a hue a check can hold to a tolerance."""
    return math.degrees(math.atan2(rgb[1], rgb[0] + 1e-9))


def angle_difference(a: float, b: float) -> float:
    """The shortest way round between two angles, so 359 and 1 are neighbours."""
    difference = abs(a - b) % 360.0
    return min(difference, 360.0 - difference)


def raster_disc(x: float, z: float, radius: float, px: int, span: float = None,
                margin_px: float = 1.0) -> tuple:
    """A world-space `(x, z)` disc as a raster-space `(cx, cy, r)` exclusion.

    The camera is **orthographic** with `rig.ORTHO_SCALE` across the frame, so the
    world-to-raster map is an affine and a disc stays a disc — which is what makes it
    possible to read the *body* the way the reference mark was read, with a declared
    accent piece's pixels removed (a202/a204). `y` runs top-down, matching
    `flip_rows`, so a coordinate printed here matches the PNG a person opens.

    Without this, `fill` counts the accent: the render's own box widens to hold it,
    and the gear measured 0.327 against a bare reference mark's 0.5665 — two numbers
    that were never describing the same object.
    """
    span = rig.ORTHO_SCALE if span is None else span
    scale = px / span
    return (
        round((x / span + 0.5) * px - 0.5, 3),
        round((0.5 - z / span) * px - 0.5, 3),
        # Grown by one **judged pixel** (`margin_px`), which is the raster's own
        # resolution rather than a tolerance: an object's edge covers about one pixel,
        # so a disc cut exactly at the declared radius leaves the accent's antialiased
        # fringe behind as a halo. That halo measured 1-11 px fragments at the same
        # corner in every smoke run, and the topology walk honestly reported it as the
        # *body* arriving in four pieces -- an artefact that looked like the defect the
        # rule exists to catch, which is the worst kind of measurement error.
        round(radius * scale + margin_px, 3),
    )


def with_exclusions(pixels, px: int, exclude) -> tuple:
    """`(pixels, hidden_px)` with every pixel inside a declared exclusion cleared.

    The count is returned rather than kept private because an exclusion that landed
    somewhere other than the accent would otherwise read as a cleaner render instead
    of as a broken coordinate: a run can compare it against the accent's own area.
    """
    if not exclude:
        return pixels, 0
    buffer = bytearray(pixels)
    hidden = 0
    for index in range(px * px):
        x, y = index % px, index // px
        for centre_x, centre_y, radius in exclude:
            if (x + 0.5 - centre_x) ** 2 + (y + 0.5 - centre_y) ** 2 <= radius * radius:
                if buffer[index * 4 + 3]:
                    hidden += 1
                buffer[index * 4 + 3] = 0
                break
    return bytes(buffer), hidden


def measure(pixels, px: int, alpha_threshold: float = 0.5, exclude=None) -> dict:
    """Coverage, luminance distribution and chromaticity of a rendered icon.

    `exclude` holds raster-space discs whose pixels are read as background (`a204`),
    which is how a body is measured on its own while its accent is still in the
    frame: the reference marks are bare objects, so the comparison is only honest if
    the accent's pixels are removed rather than averaged in.

    The **75th percentile** is the number the contrast check uses, and it is the
    honest middle: the mean of a shaded solid is dragged down by its own shadow
    side, and the peak is one highlight pixel. What §1.4.11 asks is whether the
    component can be *perceived*, so the check asks what most of the glyph is
    doing, and the mean and peak are recorded beside it so a reader can disagree.
    """
    pixels, excluded_px = with_exclusions(pixels, px, exclude)
    covered = 0
    clipped = 0
    luminances = []
    sum_r = sum_g = sum_b = 0.0
    min_x = min_y = px
    max_x = max_y = -1
    for index in range(px * px):
        alpha = pixels[index * 4 + 3] / 255.0
        if alpha < alpha_threshold:
            continue
        covered += 1
        x, y = index % px, index // px
        min_x, max_x = min(min_x, x), max(max_x, x)
        min_y, max_y = min(min_y, y), max(max_y, y)
        # Blown means blown *in the asset*: the saved byte saturated at 255 in
        # any channel. The world is an environment now, so a bright family can
        # clip against it, and a clipped highlight is detail that never reaches
        # the shipped file -- a defect the luminance percentile cannot see.
        if pixels[index * 4] == 255 or pixels[index * 4 + 1] == 255 or pixels[index * 4 + 2] == 255:
            clipped += 1
        lr = linear(pixels[index * 4] / 255.0)
        lg = linear(pixels[index * 4 + 1] / 255.0)
        lb = linear(pixels[index * 4 + 2] / 255.0)
        luminances.append(relative_luminance((lr, lg, lb)))
        sum_r += lr
        sum_g += lg
        sum_b += lb

    if covered == 0:
        return {
            "coverage": 0.0,
            "mean_luminance": 0.0,
            "p75_luminance": 0.0,
            "peak_luminance": 0.0,
            "chromaticity_degrees": None,
            "note": "nothing opaque in the render at all",
            "clip_fraction": 0.0,
            "excluded_px": excluded_px,
        }

    luminances.sort()
    mean = (sum_r / covered, sum_g / covered, sum_b / covered)
    return {
        "coverage": round(covered / (px * px), 4),
        "mean_luminance": round(relative_luminance(mean), 4),
        "p75_luminance": round(luminances[int(0.75 * (covered - 1))], 4),
        "peak_luminance": round(luminances[-1], 4),
        "chromaticity_degrees": round(chromaticity_angle(mean), 2),
        "clip_fraction": round(clipped / covered, 4),
        # Fill is how much of its **own box** the mark occupies, which is what the
        # corpus anchors measure and is scale-invariant, unlike coverage (which is
        # against the frame). Gloss is the tail of the luminance distribution -- the
        # fraction of the mark brighter than 1.6x its own median -- which is the
        # only reading of "is there a highlight on this" that a single icon supports.
        "fill": round(covered / max(1, (max_x - min_x + 1) * (max_y - min_y + 1)), 4),
        "gloss": round(
            float(sum(1 for l in luminances if l > 1.6 * luminances[len(luminances) // 2]))
            / covered,
            4,
        ),
        "excluded_px": excluded_px,
    }


def topography(pixels, px: int, alpha_threshold: float = 0.5, exclude=None,
               with_sites: bool = False) -> dict:
    """Pieces and holes of the mark, counted the way the corpus was counted.

    8-connected for both, because `shapes.LADDER_SAMPLING` states 8-connectivity
    and an anchor measured with one connectivity compared against a render measured
    with another is a comparison of two different numbers.

    A *hole* is background the border cannot reach -- the gear's centre, the lens of
    a magnifier. Counting components of the covered mask instead, which is the
    obvious mistake, reports zero holes for a gear, because a hole is an absence and
    never a covered component.

    **`hole_shares` is the number the sliver rule reads** (a183, a196): each enclosed
    void's area as a share of the object's covered area, largest first. A count alone
    cannot separate a declared feature from a sliver, because the ticket's notch is
    1.37 % and a rendering sliver can be 0.59 % — both are "one hole". Recorded here
    rather than judged, because the gate belongs with the checks that act on it and a
    measurement that is only printed is still a measurement.
    """
    pixels, excluded_px = with_exclusions(pixels, px, exclude)
    covered = _components(pixels, px, alpha_threshold, want_covered=True)
    voids = _components(pixels, px, alpha_threshold, want_covered=False)
    # The denominator is the *area* the object covers, not the number of pieces it
    # arrives in: dividing by the piece count printed a one-pixel void as 100 % and
    # would have made every share meaningless in exactly the case the rule exists.
    area = covered["area"]
    out = {
        "pieces": covered["total"],
        "holes": voids["enclosed"],
        "covered_px": area,
        "excluded_px": excluded_px,
        "hole_shares": [
            round(void / area, 4) if area else 0.0
            for void in sorted(voids["enclosed_areas"], reverse=True)
        ],
    }
    if with_sites:
        # A floating component refuses promotion (a183/a202), and a refusal has to
        # name the fix: "4 pieces" cannot be acted on, "a 70 px piece at (61, 22)"
        # can. Sites are top-down, matching the PNG a person opens.
        out["piece_sites"] = [
            (area_px, x, y)
            for area_px, x, y in sorted(component_sites(pixels, px, alpha_threshold),
                                        reverse=True)
        ]
    return out


def component_sites(pixels, px: int, alpha_threshold: float = 0.5) -> list:
    """Every covered component as `(area_px, x, y)`, largest first.

    Same walk as `_components`, with the site kept: the largest cell of the component
    and the middle of the run through it, so two pieces that both report "at (61, 22)"
    cannot be confused for one another.
    """
    seen = bytearray(px * px)
    found = []
    for start in range(px * px):
        if seen[start] or pixels[start * 4 + 3] / 255.0 < alpha_threshold:
            continue
        stack = [start]
        area = 0
        best_x = best_y = 0
        best_row = -1
        while stack:
            index = stack.pop()
            if seen[index] or pixels[index * 4 + 3] / 255.0 < alpha_threshold:
                continue
            seen[index] = 1
            area += 1
            x, y = index % px, index // px
            # The site is the widest run's middle: a component's own extent, not its
            # first pixel, so the number is stable across antialiasing at its edge.
            row = 0
            left = x
            while left - 1 >= 0 and pixels[(y * px + left - 1) * 4 + 3] / 255.0 >= alpha_threshold:
                left -= 1
            right = x
            while right + 1 < px and pixels[(y * px + right + 1) * 4 + 3] / 255.0 >= alpha_threshold:
                right += 1
            row = right - left + 1
            if row > best_row:
                best_row = row
                best_x, best_y = (left + right) // 2, y
            for dy in (-1, 0, 1):
                for dx in (-1, 0, 1):
                    nx, ny = x + dx, y + dy
                    if 0 <= nx < px and 0 <= ny < px:
                        j = ny * px + nx
                        if not seen[j] and pixels[j * 4 + 3] / 255.0 >= alpha_threshold:
                            stack.append(j)
        found.append((area, best_x, best_y))
    return found


def void_sites(pixels, px: int, alpha_threshold: float = 0.5, exclude=None) -> list:
    """Every enclosed void as `(area_px, x, y)`, largest first.

    A refusal has to name the fix, and for a sliver the fix is a *place* in the
    frame: "a one-pixel void at (61, 34)" can be looked at, while "2 holes" cannot.
    Top-down coordinates on the judged raster, so it matches the PNG a person opens.
    """
    pixels, _ = with_exclusions(pixels, px, exclude)
    seen = bytearray(px * px)
    stack = []
    for index in range(px * px):
        x, y = index % px, index // px
        if pixels[index * 4 + 3] / 255.0 >= alpha_threshold:
            continue
        if x in (0, px - 1) or y in (0, px - 1):
            if not seen[index]:
                seen[index] = 1
                stack.append(index)
    while stack:
        index = stack.pop()
        x, y = index % px, index // px
        for dy in (-1, 0, 1):
            for dx in (-1, 0, 1):
                nx, ny = x + dx, y + dy
                if not (0 <= nx < px and 0 <= ny < px):
                    continue
                j = ny * px + nx
                if not seen[j] and pixels[j * 4 + 3] / 255.0 < alpha_threshold:
                    seen[j] = 1
                    stack.append(j)
    sites = []
    for start in range(px * px):
        if seen[start] or pixels[start * 4 + 3] / 255.0 >= alpha_threshold:
            continue
        stack = [start]
        seen[start] = 1
        area = 0
        sum_x = sum_y = 0
        while stack:
            index = stack.pop()
            x, y = index % px, index // px
            area += 1
            sum_x += x
            sum_y += y
            for dy in (-1, 0, 1):
                for dx in (-1, 0, 1):
                    nx, ny = x + dx, y + dy
                    if not (0 <= nx < px and 0 <= ny < px):
                        continue
                    j = ny * px + nx
                    if not seen[j] and pixels[j * 4 + 3] / 255.0 < alpha_threshold:
                        seen[j] = 1
                        stack.append(j)
        sites.append((area, round(sum_x / area), round(sum_y / area)))
    return sorted(sites, reverse=True)


def _components(pixels, px: int, alpha_threshold: float, want_covered: bool) -> dict:
    def wanted(index):
        covered = pixels[index * 4 + 3] / 255.0 >= alpha_threshold
        return covered if want_covered else not covered

    seen = bytearray(px * px)
    total = 0
    enclosed = 0
    enclosed_areas = []
    area_total = 0
    for start in range(px * px):
        if seen[start] or not wanted(start):
            continue
        total += 1
        stack = [start]
        touches_border = False
        area = 0
        while stack:
            index = stack.pop()
            if seen[index] or not wanted(index):
                continue
            seen[index] = 1
            area += 1
            x, y = index % px, index // px
            if x == 0 or y == 0 or x == px - 1 or y == px - 1:
                touches_border = True
            for dy in (-1, 0, 1):
                for dx in (-1, 0, 1):
                    nx, ny = x + dx, y + dy
                    if 0 <= nx < px and 0 <= ny < px:
                        j = ny * px + nx
                        if not seen[j] and wanted(j):
                            stack.append(j)
        if not want_covered and not touches_border:
            enclosed += 1
            enclosed_areas.append(area)
        area_total += area
    return {"total": total, "enclosed": enclosed, "enclosed_areas": enclosed_areas,
            "area": area_total}


# ---------------------------------------------------------------------------
# Pixels out
# ---------------------------------------------------------------------------


def flip_rows(data: bytes, px: int, channels: int = 4) -> bytes:
    """Blender hands back bottom-up rows; flip once, here, for everyone downstream."""
    out = bytearray(len(data))
    stride = px * channels
    for row in range(px):
        source = (px - 1 - row) * stride
        target = row * stride
        out[target : target + stride] = data[source : source + stride]
    return bytes(out)


def save_png(bpy, path: str, pixels: bytes, px: int) -> None:
    image = bpy.data.images.new("out", width=px, height=px, alpha=True)
    image.pixels = [channel / 255.0 for channel in pixels]
    image.filepath_raw = path
    image.file_format = "PNG"
    image.save()
    bpy.data.images.remove(image)


def write_raw(path: str, pixels: bytes) -> None:
    """The runtime sidecar: RGBA8, row-major, top-down. Stated in the manifest."""
    with open(path, "wb") as handle:
        handle.write(pixels)


def read_render(bpy, path: str, source_px: int = rig.RENDER_PX) -> bytes:
    """Read a saved render back as stored, then put the rows the right way up."""
    raw = rig.read_png_non_color(bpy, path)
    stored = bytes(min(255, max(0, round(channel * 255.0))) for channel in raw)
    return flip_rows(stored, source_px)


# ---------------------------------------------------------------------------
# Rendering
# ---------------------------------------------------------------------------


def build_materials(bpy, entry):
    """Clone editable reference materials, falling back to the source recipe."""
    materials = {}
    records = {}
    for role, parameters in entry["materials"].items():
        name = f"openpbr:{entry['id']}:{entry['generation']}:{role}"
        # The blend owns the *surface* and the matrix owns the *colour*: the
        # editable reference material is looked up by family, not by the role the
        # composition happened to give it, so "silhouette" and "secondary" in two
        # icons resolve to the same editable surface when they are the same family.
        family = parameters.get("family") or role
        reference = rig.reference_material(family)
        if reference is not None:
            materials[role] = reference.copy()
            materials[role].name = name
            # The blend owns the surface, the matrix owns the colour. Without this
            # the copied material keeps the blend's own base colour, which is how
            # four icons declaring four different hues all rendered one grey.
            if not openpbr.apply_base_color(
                materials[role], parameters["base_color"][:3],
                parameters["base_color"][3] if len(parameters["base_color"]) > 3 else 1.0,
            ):
                raise RuntimeError(
                    f"`{family}` has no Principled Base Color socket, so the declared "
                    f"colour for `{role}` would not reach the render"
                )
            # The emphasis's finish is applied **over** the blend's family surface
            # (C8 a178). Without this the blend owned the whole surface and a
            # candidate's declared sheen never reached the render: every slot of an
            # icon measured the family's own finish whatever its emphasis said.
            declared_finish = parameters.get("finish")
            finished = bool(declared_finish) and openpbr.apply_finish(
                materials[role], declared_finish)
            effective = openpbr.from_blender_material(materials[role], parameters)
            records[role] = openpbr.record(effective)
            records[role]["surface_source"] = (
                f"reference-blend ({family}) + declared finish ({declared_finish})"
                if finished else f"reference-blend ({family})"
            )
            records[role]["colour_source"] = f"matrix ({family} at the declared hue)"
        else:
            materials[role] = openpbr.build_material(bpy, name, parameters)
            records[role] = openpbr.record(parameters)
            # Not silent. The blend carries no `icon_ref:material:<family>`, so the
            # surface is the declared table rather than the reference -- which is a
            # different claim about where the pixels came from.
            records[role]["surface_source"] = (
                f"python-fallback; the blend has no icon_ref:material:{family}"
            )
            # The colour is the matrix's either way -- the fallback is a *surface*
            # path, not a colour path -- so it is recorded here too rather than
            # leaving the only family without a blend material looking as though its
            # colour came from somewhere else.
            records[role]["colour_source"] = f"matrix ({family} at the declared hue)"
    return materials, records


def material_key(parameters) -> str:
    """A stable key for a material, so identical materials share one swatch render."""
    return json.dumps(openpbr.record(parameters)["openpbr"], sort_keys=True)


def render_to(bpy, scene, path: str, source_px: int = rig.RENDER_PX) -> bytes:
    scene.render.filepath = path
    bpy.ops.render.render(write_still=True)
    pixels = read_render(bpy, path, source_px)
    os.remove(path)
    return pixels


def measure_swatch(bpy, scene, material, key, cache, counter, source_px=rig.RENDER_PX):
    """The material's own brightness under the same rig, measured once per material."""
    if key in cache:
        return cache[key]
    objects = shapes.swatch_geometry(bpy, material)
    path = os.path.join(REVIEW, f"_swatch.{counter[0]}.render.png")
    counter[0] += 1
    pixels = render_to(bpy, scene, path, source_px)
    small = rig.downsample(pixels, source_px, DECISION_PX)
    cache[key] = measure(small, DECISION_PX)
    for obj in objects:
        bpy.data.objects.remove(obj, do_unlink=True)
    return cache[key]


def generate(device: str, tier: str, limit=None):
    os.makedirs(REVIEW, exist_ok=True)
    scene = rig.configure(bpy, device=device)
    # A concept-set change reopens old targets. The append-only log remains the
    # historical record, but only records from this set may promote an asset.
    decisions = read_decisions(shapes.CONCEPT_SET)
    swatches = {}
    counter = [0]
    entries = [
        e for e in shapes.ICONS
        if tier in ("all", "stage-1") or e["kind"] == tier
    ]
    if limit:
        entries = entries[:limit]
    rendered = []

    for entry in entries:
        chosen = decisions.get(entry["id"], {}).get("generation")
        generations = []
        generation_entries = {}
        generation_pixels = {}
        for generation in shapes.generations_of(entry):
            gen_id = generation["generation"]
            generation_entries[gen_id] = generation
            materials, material_records = build_materials(bpy, generation)
            objects = shapes.build(bpy, generation, materials)

            pixels = render_to(
                bpy, scene, os.path.join(REVIEW, f"{entry['id']}.{gen_id}.render.png")
            )
            generation_pixels[gen_id] = pixels

            sizes = []
            decision_small = None
            for px in (DECISION_PX, RECOGNITION_PX, CONTEXT_PX):
                small = rig.downsample(pixels, rig.RENDER_PX, px)
                if px == DECISION_PX:
                    decision_small = small
                save_png(
                    bpy,
                    os.path.join(REVIEW, f"{entry['id']}.{gen_id}.{px}.png"),
                    small,
                    px,
                )
                # Raw for **every** reviewed size, not just the decision size: the
                # picker shows the smallest size beside the largest, and the runtime
                # has no PNG decoder to reach for. PNG is for people; RGBA is for the
                # program; both are written, and the manifest names which is which.
                write_raw(
                    os.path.join(REVIEW, f"{entry['id']}.{gen_id}.{px}.rgba"), small
                )
                measurements = measure(small, px)
                if px == DECISION_PX:
                    # Counted at the decision size only: it is the size the corpus
                    # anchors were measured at, and topology at 24 px is a property
                    # of the downsampler rather than of the mark.
                    measurements.update(topography(small, px))
                sizes.append({"px": px, "measurements": measurements})

            swatch_measurements = {}
            for role, parameters in generation["materials"].items():
                key = material_key(parameters)
                swatch_measurements[role] = measure_swatch(
                    bpy, scene, materials[role], key, swatches, counter
                )

            record = {
                "generation": gen_id,
                "generation_label": generation["generation_label"],
                "part_count": len(generation["parts"]) + (4 if entry.get("framed") else 0),
                "primary_role": primary_role(generation),
                "generation_why": generation["generation_why"],
                "brief": generation.get("brief", {}),
                "lineage": generation.get("lineage", entry.get("lineage", [])),
                "transform": transform_record(entry, generation),
                "materials": material_records,
                "swatches": swatch_measurements,
                "sizes": sizes,
                # C3's two additions, measured against the declared mark at the
                # decision size rather than at 24 px, because MD1's largest raster
                # is 96 px and neither side should have to be resampled.
                "silhouette": silhouette_check(bpy, entry, decision_small),
                "composition": composition_check(entry, generation),
                "authored_under": "c3-composition",
                "promoted": None,
            }
            record["checks"] = judge_generation(entry, record)

            for obj in objects:
                bpy.data.objects.remove(obj, do_unlink=True)
            for material in materials.values():
                bpy.data.materials.remove(material)

            generations.append(record)

        for generation in generations:
            generation["selection_notes"] = candidate_gate_notes(
                entry, generation, generations
            )
        chosen_record = next(
            (generation for generation in generations if generation["generation"] == chosen),
            None,
        )
        if chosen_record is not None and chosen_record["selection_notes"]:
            chosen_record["promoted"] = {
                "refused": True,
                "reason": "candidate failed the hard selection gate",
                "notes": chosen_record["selection_notes"],
            }
        elif chosen_record is not None:
            chosen_record["promoted"] = promote(
                bpy,
                entry,
                generation_entries[chosen],
                generation_pixels[chosen],
            )

        rendered.append(
            {
                "id": entry["id"],
                "kind": entry["kind"],
                "meaning": entry["meaning"],
                "source": entry["source"],
                "locates": entry["locates"],
                "sits_on": list(entry["sits_on"]),
                "identity": entry["identity"],
                "identity_as": entry["identity_as"],
                "framed": bool(entry.get("framed")),
                "concept_set": shapes.CONCEPT_SET,
                "tier": entry.get("tier"),
                "hues": entry.get("hues"),
                "silhouette": entry.get("silhouette"),
                "notes": entry["notes"],
                "brief": entry["brief"],
                "lineage": entry["lineage"],
                "forbidden_readings": entry["forbidden_readings"],
                "chosen_generation": chosen,
                "generations": generations,
            }
        )
    return rendered


# ---------------------------------------------------------------------------
# C3's checks: the silhouette, and the three materials
# ---------------------------------------------------------------------------

#: How much of the declared mark's own coverage the render has to cover. Not 1.0:
#: the mark is traced at 96 px and rendered through a bevelled extrusion, so a rim
#: of antialiasing is expected to fall outside it at the edges. Below this the
#: render is a different shape from the one it claims to be.
SILHOUETTE_CONTAINMENT_FLOOR = 0.90

#: The accent piece is allowed to leave the mark's box — that is what an accent is —
#: so whole-icon occupancy overlap is recorded rather than gated. It would otherwise
#: measure the tab as a failure to be the gear.
SILHOUETTE_OVERLAP_RECORDED_NOT_GATED = True


def alpha_occupancy(pixels, px: int, alpha_threshold: float = 0.5, grid_side: int = 12):
    """A 12x12 coverage grid from a rendered RGBA buffer, top-down."""
    cells = [0] * (grid_side * grid_side)
    area = (px / grid_side) ** 2
    for index in range(px * px):
        if pixels[index * 4 + 3] / 255.0 < alpha_threshold:
            continue
        x, y = index % px, index // px
        cell_x = min(grid_side - 1, x * grid_side // px)
        cell_y = min(grid_side - 1, y * grid_side // px)
        cells[cell_y * grid_side + cell_x] += 1
    return [cell / area for cell in cells]


def silhouette_check(bpy, entry, decision_pixels) -> dict:
    """Does the render actually carry the silhouette the brief declares?

    Containment, not equality: every cell the declared mark covers must be covered
    by the render, and the render is allowed to add — the accent piece leaves the
    mark's box by design. The whole-icon overlap is recorded beside it so a reader
    can see how much was added rather than having to trust that it was the accent.

    **Both sides are measured in the icon's own frame.** The declared mark is placed
    by the same transform the mesh is built with, so the comparison is "did the render
    carry the mark", not "did two framings agree". That distinction is the fault C8
    isolated: the mask used to be binned where the mark sits **in its own file**, so a
    mark that is not centred in its file (`maps/add_road` occupies x 0.167-0.948) was
    compared against a mesh placed on its own bounds and centred — 0.3235 of the floor
    for reasons that were never about the geometry. The old reading is still recorded,
    as `containment_file_frame`, because a number that moved should stay readable.
    """
    if abs(shapes.ICON_FRAME_SPAN - rig.ORTHO_SCALE) > 1e-9:
        raise SystemExit(
            f"the silhouette check's frame ({shapes.ICON_FRAME_SPAN}) and the rig's "
            f"orthographic scale ({rig.ORTHO_SCALE}) disagree; the check would measure "
            f"a frame nothing is rendered into"
        )
    declared = entry.get("silhouette") or {}
    glyph = declared.get("glyph")
    reference = declared.get("reference")
    if not glyph or not reference or not reference.get("exists"):
        return {
            "judged": False,
            "why": f"`{entry['id']}` declares no silhouette on disk; nothing to compare",
        }
    mark = shapes.glyph_mask(bpy, glyph)
    render_cells = alpha_occupancy(decision_pixels, DECISION_PX)
    covered, missing = 0, 0
    for index, cell in enumerate(mark["occupancy"]):
        if cell < 0.5:
            continue
        if render_cells[index] >= 0.5:
            covered += 1
        else:
            missing += 1
    total = covered + missing
    strict = round(covered / total, 4) if total else 0.0
    agreement = 0
    compared = 0
    for cell, render_cell in zip(mark["occupancy"], render_cells):
        compared += 1
        if (cell >= 0.5) == (render_cell >= 0.5):
            agreement += 1

    # The judged reading (C8 a176): the share of the declared mark's **ink** the
    # render actually carries. The strict cell count above is kept as a gate and a
    # recorded number, because a yes/no reading on an 8-pixel cell turns on where the
    # cell boundary happens to fall: `tool-power`'s thin ring has cells the mark fills
    # 0.55 of and the render fills 0.40 of, and 0.40 is 73% of that cell's ink. The
    # weighted reading says so; the strict count calls it a miss. Both are printed,
    # and a strict miss still refuses promotion — so neither reading can hide the
    # other.
    declared_cells = [
        (cell, render_cell)
        for cell, render_cell in zip(mark["occupancy"], render_cells)
        if cell >= 0.5
    ]
    declared_ink = sum(cell for cell, _ in declared_cells)
    carried_ink = sum(min(cell, render_cell) for cell, render_cell in declared_cells)
    weighted = round(carried_ink / declared_ink, 4) if declared_ink else 0.0
    ratios = sorted(render_cell / cell for cell, render_cell in declared_cells if cell > 0)
    worst_cell = round(ratios[0], 4) if ratios else 0.0
    below = sum(1 for cell, render_cell in declared_cells if render_cell < 0.5)
    containment = weighted

    # Recorded, never judged: what the check said while it was comparing two
    # different frames. It is kept so the correction above is auditable rather than
    # a silent renumbering of every icon's history.
    file_covered = file_missing = 0
    for cell, render_cell in zip(mark["occupancy_file_frame"], render_cells):
        if cell < 0.5:
            continue
        if render_cell >= 0.5:
            file_covered += 1
        else:
            file_missing += 1
    file_total = file_covered + file_missing

    return {
        "judged": True,
        "glyph": glyph,
        "reference": reference,
        "containment": containment,
        "containment_kind": "weighted ink",
        "containment_strict": strict,
        "containment_strict_is_a_gate": (
            "any declared cell under 0.5 covered refuses promotion, however the "
            "weighted reading scores"
        ),
        "declared_cells": len(declared_cells),
        "cells_below_threshold": below,
        "worst_cell_ratio": worst_cell,
        "containment_file_frame": round(file_covered / file_total, 4) if file_total else 0.0,
        "frame": {
            "icon_frame_span": mark["frame_span"],
            "placement_span": mark["placement_span"],
            "mark_pixels_outside_frame": mark["placed_outside_frame"],
            "note": (
                "both sides of the comparison are measured in the icon's own frame; "
                "containment_file_frame is the reading this replaced, kept for audit"
            ),
        },
        "occupancy_agreement": round(agreement / compared, 4),
        "mark_coverage_at_96": mark["coverage"],
        "render_coverage_at_96": round(
            sum(1 for i in range(DECISION_PX * DECISION_PX)
                if decision_pixels[i * 4 + 3] / 255.0 >= 0.5) / (DECISION_PX * DECISION_PX), 4
        ),
        "grid": mark["occupancy_grid"],
        "floor": SILHOUETTE_CONTAINMENT_FLOOR,
        "occupancy_is_recorded_not_gated": SILHOUETTE_OVERLAP_RECORDED_NOT_GATED,
    }


def composition_check(entry, generation) -> dict:
    """Three main materials, one silhouette, one accent — said in a script.

    The roles the parts name are the *functions*; the amount of colour is what the
    composition is. So the check counts distinct base colours across the three roles
    rather than counting the parts, because a candidate with six parts in three
    colours is a composition and a candidate with three parts in two is not.
    """
    roles = sorted({part.get("role", "ink") for part in generation.get("parts", [])})
    materials = generation.get("materials", {}) or {}
    missing = [role for role in shapes.COMPOSITION_ROLES if role not in roles]
    colours = {}
    for role in shapes.COMPOSITION_ROLES:
        parameters = materials.get(role)
        if not parameters:
            continue
        colours[role] = tuple(round(channel, 6) for channel in parameters["base_color"][:3])
    distinct = len(set(colours.values()))
    notes = []
    if missing:
        notes.append(f"the composition is missing a main part: {', '.join(missing)}")
    if distinct != len(shapes.COMPOSITION_ROLES):
        notes.append(
            f"the three main materials resolve to {distinct} distinct colour(s), not "
            f"{len(shapes.COMPOSITION_ROLES)}: two of them are the same material wearing "
            f"two role names"
        )
    accidental = [role for role in roles if role not in shapes.COMPOSITION_ROLES and role != "ink"]
    if accidental:
        notes.append(f"undeclared roles in the composition: {', '.join(accidental)}")
    return {"roles": roles, "missing_roles": missing, "distinct_colours": distinct,
            "colours": {role: list(value) for role, value in colours.items()},
            "notes": notes}


def primary_role(generation) -> str:
    """The role carrying the silhouette, rather than the HUD's text token.

    C3 compositions name their parts by function, so the answer is the silhouette
    role itself. The legacy priority list is kept behind it for a retired recipe,
    and either way the caller falls back to `ink` rather than to a wrong material:
    the material checks compare against this role's swatch, so returning the wrong
    one would measure a material the candidate is not made of.
    """
    present = {part.get("role", "ink") for part in generation.get("parts", [])}
    for role in shapes.COMPOSITION_ROLES:
        if role in present:
            return role
    legacy = ("metal", "road", "paper", "glass", "polymer", "ceramic", "accent", "ink")
    return next((role for role in legacy if role in present), "ink")


def transform_record(entry, generation) -> dict:
    """Record the authored candidate brief, never an invented transform."""
    return {
        "mode": "explicit-authored-recipe",
        "candidate_role": generation.get("brief", {}).get("candidate_role"),
        "semantic_cues": generation.get("brief", {}).get("semantic_cues", []),
        "material_family": generation.get("brief", {}).get("material_family"),
    }


def promote(bpy, entry, generation, pixels) -> dict:
    """Write the shipping assets for a generation a person marked as the target."""
    written = []
    for px in rig.SHIPPED_PX:
        small = rig.downsample(pixels, rig.RENDER_PX, px)
        png = os.path.join(OUT, f"{entry['id']}.{px}.png")
        save_png(bpy, png, small, px)
        write_raw(os.path.join(OUT, f"{entry['id']}.{px}.rgba"), small)
        written.append({"px": px, "file": f"{entry['id']}.{px}.png",
                        "hash": rig.hash_bytes(small)})
    return {"generation": generation["generation"], "sizes": written}


# ---------------------------------------------------------------------------
# Checks
# ---------------------------------------------------------------------------


def judge_generation(entry, generation) -> dict:
    """Every check for one generation, with every reason it is not a clean pass."""
    notes = list((generation.get("composition") or {}).get("notes", []))
    silhouette = generation.get("silhouette") or {}
    if silhouette and not silhouette.get("judged"):
        notes.append(
            silhouette.get("why", "the declared silhouette could not be compared")
        )
    elif silhouette.get("judged") and silhouette["containment"] < silhouette["floor"]:
        notes.append(
            f"the render carries {silhouette['containment']} of the declared "
            f"`{silhouette['glyph']}` mark, below the {silhouette['floor']} containment "
            f"floor: it is not the silhouette the brief names"
        )
    # The strict reading is a **gate of its own** (a176): a declared cell the render
    # leaves under half covered is a note whether or not the weighted number passes,
    # so the weighted reading cannot be used to walk past a hole in the mark.
    if silhouette.get("judged") and silhouette.get("cells_below_threshold", 0) > 0:
        notes.append(
            f"{silhouette['cells_below_threshold']} of {silhouette['declared_cells']} "
            f"declared cells of `{silhouette['glyph']}` are under half covered "
            f"(worst carries {silhouette['worst_cell_ratio']} of its own ink): the mark "
            f"is carried on average but not cell for cell"
        )
    sharp = next(s for s in generation["sizes"] if s["px"] == DECISION_PX)["measurements"]
    recognition = next(s for s in generation["sizes"] if s["px"] == RECOGNITION_PX)["measurements"]
    small = next(s for s in generation["sizes"] if s["px"] == CONTEXT_PX)["measurements"]
    primary = generation.get("primary_role", "ink")
    ink = generation["materials"].get(primary, generation["materials"]["ink"])["openpbr"]["base_color"]["linear_rgb"]
    identity = generation["materials"].get("identity", {}).get("openpbr", {}).get(
        "base_color", {}
    ).get("linear_rgb")

    # --- is anything there, and is there too much -------------------------
    if recognition["coverage"] < COVERAGE_FLOOR_24:
        notes.append(f"at {RECOGNITION_PX}px the authored mark covers {recognition['coverage']:.3f}, below the recognition floor")
    if small["coverage"] <= 0.0:
        notes.append("nothing was drawn")
    if small["coverage"] < COVERAGE_FLOOR_24:
        notes.append(
            f"at {CONTEXT_PX} px the glyph covers {small['coverage']:.3f} of the frame, "
            f"below the {COVERAGE_FLOOR_24} floor: it averaged away"
        )
    ceiling = FRAMED_COVERAGE_CEILING_24 if entry["framed"] else COVERAGE_CEILING_24
    if small["coverage"] > ceiling:
        notes.append(
            f"at {CONTEXT_PX} px the glyph covers {small['coverage']:.3f} of the frame, "
            f"above the {ceiling} ceiling: it is a blob"
        )

    # --- can the component be perceived on each surface it declares -------
    contrasts = {}
    for host in entry["sits_on"]:
        host_luminance = palette.host_luminance(host)
        ink_ratio = palette.contrast_ratio(sharp["p75_luminance"], host_luminance)
        contrasts[host] = {"ink": round(ink_ratio, 2)}
        if ink_ratio < palette.NON_TEXT_MIN_CONTRAST:
            notes.append(
                f"the ink reaches {ink_ratio:.2f}:1 on {host}, below the "
                f"{palette.NON_TEXT_MIN_CONTRAST}:1 non-text floor (WCAG 1.4.11): "
                f"the glyph would be hard to see on the surface it declares"
            )
        if identity is not None:
            identity_ratio = palette.contrast_ratio(
                relative_luminance(identity), host_luminance
            )
            contrasts[host]["identity"] = round(identity_ratio, 2)
            # Three cases, because where a colour is drawn decides what it owes. A
            # colour that carries the silhouette owes the host its own contrast; one
            # drawn **on the icon's own ink** owes the ink, because it is never seen
            # against the panel at all. The earlier version demanded host contrast
            # from both, which flagged accents that are legible by construction.
            if identity_ratio < palette.NON_TEXT_MIN_CONTRAST:
                if entry["identity_as"] == "detail-on-ink":
                    contrasts[host]["identity_why_recorded"] = (
                        "recorded, not judged: this colour is drawn on the icon's own "
                        "ink and never against the host"
                    )
                else:
                    notes.append(
                        f"the identity colour is drawn against the host but reaches "
                        f"only {identity_ratio:.2f}:1 on {host}"
                    )

    # --- does the accent read against the icon's own body ----------------
    internal = None
    if identity is not None:
        internal = palette.contrast_ratio(relative_luminance(identity),
                                          relative_luminance(ink))
        if internal < INTERNAL_MIN_CONTRAST:
            notes.append(
                f"the accent sits {internal:.2f}:1 against the ink body, below the "
                f"{INTERNAL_MIN_CONTRAST}:1 needed to read as a detail"
            )

    # --- did the declared material arrive --------------------------------
    swatch = generation["swatches"].get(primary, generation["swatches"]["ink"])
    declared = relative_luminance(ink)
    arrival = None
    if swatch["mean_luminance"] > 0.0 and declared > 0.0:
        arrival = {
            "icon_to_swatch": round(sharp["mean_luminance"] / swatch["mean_luminance"], 3),
            "swatch_to_albedo": round(swatch["mean_luminance"] / declared, 3),
            "declared_albedo_luminance": round(declared, 4),
        }
        low, high = ICON_TO_SWATCH_BAND
        if not low <= arrival["icon_to_swatch"] <= high:
            notes.append(
                f"the icon renders at {arrival['icon_to_swatch']}x its own material "
                f"swatch, outside the {low}-{high} band"
            )
        low, high = SWATCH_TO_ALBEDO_BAND
        # This band used to be skipped for near-metal materials, because under a
        # black world a mirror swatch measured the rig rather than the albedo: all
        # 31 metal candidates returned the identical 0.306, later 0.075-0.088. The
        # world now has radiance (`rig.WORLD`), which is the thing that made the
        # skip unnecessary -- a metal swatch is a reading of the material again, at
        # ~0.45x. So the skip is retired rather than left in place: a rule whose
        # premise no longer holds would hide the next real metal failure.
        if not low <= arrival["swatch_to_albedo"] <= high:
            notes.append(
                f"a flat swatch of the primary `{primary}` material renders at {arrival['swatch_to_albedo']}x "
                f"the declared albedo's own luminance, outside the {low}-{high} band"
            )
        # The hue check only means something for a colour that *has* a hue. Near a
        # neutral, the red/green angle is hypersensitive: the first run flagged a
        # 17 degree "drift" on a grey that had not moved, because the ratio of two
        # almost-equal channels is mostly noise. So the check is skipped, with the
        # reason recorded rather than a pass implied.
        chroma = max(ink) - min(ink)
        if chroma < NEUTRAL_CHROMA_THRESHOLD:
            arrival["chromaticity"] = (
                f"not judged: the declared colour is near-neutral (chroma {chroma:.3f} "
                f"below the {NEUTRAL_CHROMA_THRESHOLD} threshold), where the red/green "
                f"angle is noise rather than a reading"
            )
        else:
            drift = angle_difference(
                sharp["chromaticity_degrees"], chromaticity_angle(ink)
            )
            arrival["chromaticity_drift_degrees"] = round(drift, 2)
            if drift > CHROMATICITY_TOLERANCE_DEGREES:
                notes.append(
                    f"the rendered hue sits {drift:.1f} degrees from the declared "
                    f"colour's, beyond the {CHROMATICITY_TOLERANCE_DEGREES} degree "
                    f"tolerance"
                )

    # --- does it measure like the language it claims ----------------------
    #
    # The corpus ladder (`shapes.LANGUAGE_LADDER`) is the anchor set: fill and gloss
    # are whole-mark properties, so a candidate can be compared to them directly.
    #
    # **Piece count is recorded but not judged, on purpose.** An icon under the
    # composition rule is a mark *plus* an accent piece (and, layered, plus a second
    # body), so its piece count is necessarily the mark's plus the composition's.
    # Comparing it to a bare glyph's would fail every compliant candidate, and a
    # check that cannot pass is worse than no check: it would train everyone to
    # ignore the notes. The mark's own topology is already compared by the
    # silhouette check, against the MD1 mask it was traced from.
    character = None
    emphasis_id = (generation.get("brief") or {}).get("emphasis")
    lam = next(
        (spec["ladder"] for spec in shapes.EMPHASES if spec["id"] == emphasis_id), None
    )
    if lam is not None:
        anchor = shapes.ladder_point(lam)
        measured = {
            "fill": sharp["fill"],
            "gloss": sharp["gloss"],
            "pieces": sharp["pieces"],
            "holes": sharp["holes"],
        }
        character = {
            "emphasis": emphasis_id,
            "ladder": lam,
            "anchor": anchor,
            "measured": measured,
            "topography_judged": False,
            "topography_why_recorded": (
                "recorded, not judged: an icon is a mark plus an accent piece, so its "
                "piece count is the mark's plus the composition's and cannot equal a "
                "bare glyph's"
            ),
        }
        low, high = anchor["fill_band"]
        if not low <= measured["fill"] <= high:
            notes.append(
                f"measures fill {measured['fill']} at {DECISION_PX}px, outside the "
                f"{low}-{high} the `{emphasis_id}` end of the corpus ladder spans: it "
                f"is not the reading it claims to be"
            )
        if abs(measured["gloss"] - anchor["gloss"]) > anchor["gloss_tolerance"]:
            notes.append(
                f"measures gloss {measured['gloss']} at {DECISION_PX}px against the "
                f"`{emphasis_id}` anchor's {anchor['gloss']} (tolerance "
                f"{anchor['gloss_tolerance']}): the surface character is not that "
                f"language's"
            )

    return {"notes": notes, "contrasts": contrasts, "internal_contrast": internal,
            "arrival": arrival, "character": character}


def candidate_gate_notes(entry, generation, all_generations) -> list:
    """The hard gate for selecting one candidate for shipping."""
    notes = list(generation.get("checks", {}).get("notes", []))
    if entry["kind"] == "surface":
        if entry.get("locates") not in shapes.DECLARED_SURFACES:
            notes.append(f"surface `{entry['locates']}` is not declared by the pilot registry")
        if entry.get("identity") in STATE_IDENTITY_TOKENS:
            notes.append(f"static identity `{entry['identity']}` is a live state token")
        if generation.get("part_count", 0) < MIN_SURFACE_PARTS:
            notes.append(f"candidate has {generation.get('part_count', 0)} authored parts, below the {MIN_SURFACE_PARTS}-part floor")
        if len(all_generations) != 6:
            notes.append("the candidate set must contain exactly six authored concepts")
        brief = generation.get("brief", {})
        for field in ("candidate_role", "semantic_cues", "material_family", "motion", "fallback", "acceptance"):
            if not brief.get(field):
                notes.append(f"candidate brief is missing `{field}`")
        cues = brief.get("semantic_cues", [])
        if not cues:
            notes.append("candidate has no declared semantic cues")
    return notes


def _ladder_span() -> tuple:
    """How far apart the two extreme anchors are, per judged property."""
    fills = [spec["fill"] for spec in shapes.LANGUAGE_LADDER.values()]
    glosses = [spec["gloss"] for spec in shapes.LANGUAGE_LADDER.values()]
    return max(fills) - min(fills), max(glosses) - min(glosses)


def _separation_notes(generations) -> list:
    """No two candidates may be the same reading.

    The floor is a fraction of the ladder's own span rather than a number chosen by
    feel: if two candidates are closer together than that on **both** judged axes,
    the picker is showing one icon twice and asking a human to choose between it and
    itself -- which is the failure that opened this session.
    """
    fill_span, gloss_span = _ladder_span()
    floor = (fill_span * LADDER_SEPARATION_FRACTION, gloss_span * LADDER_SEPARATION_FRACTION)
    measured = []
    for generation in generations:
        character = (generation.get("checks") or {}).get("character")
        if character:
            measured.append((generation["generation"], character["measured"]))
    notes = []
    for index, (name_a, a) in enumerate(measured):
        for name_b, b in measured[index + 1:]:
            if abs(a["fill"] - b["fill"]) < floor[0] and abs(a["gloss"] - b["gloss"]) < floor[1]:
                notes.append(
                    f"`{name_a}` and `{name_b}` measure within {floor[0]:.3f} fill and "
                    f"{floor[1]:.3f} gloss of each other: two slots, one reading"
                )
    return notes


def judge_icon(record) -> dict:
    """The icon's verdict, which is every candidate's checks in one place."""
    notes = []
    for generation in record["generations"]:
        for note in generation.get("selection_notes", generation["checks"]["notes"]):
            notes.append(f"[{generation['generation']}] {note}")
    for note in _separation_notes(record["generations"]):
        notes.append(f"[separation] {note}")
    if record["chosen_generation"] is None:
        notes.append(
            "no candidate has been marked as the target: awaiting a decision in the "
            "picker, so nothing for this icon ships yet"
        )
    return {
        "id": record["id"],
        "kind": record["kind"],
        "verdict": "pass" if not notes else "open",
        "notes": notes,
        "chosen_generation": record["chosen_generation"],
    }


# ---------------------------------------------------------------------------
# Decisions
# ---------------------------------------------------------------------------


def read_decisions(concept_set: str) -> dict:
    """The last target for this concept set; older targets remain historical."""
    chosen = {}
    if not os.path.exists(DECISIONS):
        return chosen
    with open(DECISIONS, "r", encoding="utf-8") as handle:
        for line in handle:
            line = line.strip()
            if not line:
                continue
            try:
                record = json.loads(line)
            except json.JSONDecodeError:
                continue
            if record.get("target") and record.get("concept_set") == concept_set:
                chosen[record.get("icon")] = record
    return chosen


def read_directives(concept_set: str | None = None) -> dict:
    """Every comment for the current concept set, oldest first.

    This is the half of the review loop that shapes the *next* generation: a
    comment is recorded with or without a target, so "none of these, change this"
    is a usable answer rather than a dead end. The log is the source, so a comment
    counts the moment it is written, before this script has run again.
    """
    directives = {}
    if not os.path.exists(DECISIONS):
        return directives
    with open(DECISIONS, "r", encoding="utf-8") as handle:
        for line in handle:
            line = line.strip()
            if not line:
                continue
            try:
                record = json.loads(line)
            except json.JSONDecodeError:
                continue
            if concept_set is not None and record.get("concept_set") != concept_set:
                continue
            comment = (record.get("comment") or "").strip()
            if comment:
                directives.setdefault(record.get("icon"), []).append(comment)
    return directives


# ---------------------------------------------------------------------------
# The records
# ---------------------------------------------------------------------------


def source_hashes() -> dict:
    files = {
        "tools/icons/generate.py": os.path.abspath(__file__),
        "tools/icons/openpbr.py": os.path.join(HERE, "openpbr.py"),
        "tools/icons/palette.py": os.path.join(HERE, "palette.py"),
        "tools/icons/rig.py": os.path.join(HERE, "rig.py"),
        "tools/icons/shapes.py": os.path.join(HERE, "shapes.py"),
        "assets/icons/reference.blend": rig.reference_path(),
        "src/hud.rs": os.path.join(ROOT, "src", "hud.rs"),
        "src/design.rs": os.path.join(ROOT, "src", "design.rs"),
        # C3 named the design spec as part of the standard the renders are judged
        # against, and it was not in the set at all: a change to the spec could not
        # have been seen, let alone reported.
        "UNIFIED_DESIGN.md": os.path.join(ROOT, "UNIFIED_DESIGN.md"),
    }
    return {
        name: palette.source_hash(path)
        for name, path in files.items()
        if os.path.exists(path)
    }


def build_record(tier: str) -> dict:
    return {
        "generated_by": "tools/icons/generate.py",
        "how_to_regenerate": (
            "blender --background --factory-startup --python "
            "tools/icons/generate.py -- --review stage-1"
        ),
        "why_committed": (
            "cargo build must never need Blender; the renders are artifacts with "
            "provenance, and this script is how they were produced"
        ),
        "concept_set": shapes.CONCEPT_SET,
        "catalogue_stage": shapes.CATALOGUE_STAGE,
        "this_run": {"tier": tier, "inventory_entries": len(shapes.ICONS)},
        "tiers": {
            "surface": "locates something the game has; must name a declared surface",
            "vocabulary": "carries a meaning from the design grammar; recreates the "
                          "grammar, never the artwork",
        },
        "hash": {
            "algorithm": "fnv1a64",
            "is": "an integrity check — this file is the file the manifest describes",
            "is_not": "a signature, and not a security mechanism",
        },
        "raw_format": {
            "what": "the runtime sidecar for each icon",
            "format": "RGBA8, unpremultiplied",
            "layout": "row-major, top-down, origin top-left",
            "why": "the runtime carries no PNG decoder: PNG is for people, RGBA is for "
                   "the program",
        },
        "blender": {
            "version": bpy.app.version_string,
            "build_hash": bpy.app.build_hash.decode()
            if isinstance(bpy.app.build_hash, bytes)
            else str(bpy.app.build_hash),
            "branch": str(bpy.app.build_branch),
            "cycle": bpy.app.version_cycle,
            "policy": (
                "any Blender 5.x is accepted, so a re-render on a later build will not "
                "reproduce these pixels; the build that made them is recorded here so "
                "that a difference is visible rather than silent"
            ),
        },
        "sources": source_hashes(),
        "reference_settings": rig.reference_info(),
        "reference_study": shapes.REFERENCE_STUDY,
        "palette_hash": palette.palette_hash(),
        "hosts": list(palette.HOSTS),
        "non_text_contrast_floor": palette.NON_TEXT_MIN_CONTRAST,
        "sizes_px": list(rig.SHIPPED_PX),
        "dpi": {"base_px": 96, "scale_factors": [1, 2, 4, 8, 16], "master_px": 1536,
                 "outputs_px": [96, 192, 384, 768, 1536],
                 "ui_scale_mapping": "density metadata is separate from logical display size",
                 "downsample": "linear-light box average for integer factors"},
        "ladder": (
            "every shipped size divides the 192 px render by a whole number, so the "
            "downsample is a box mean rather than a resampling filter; the ladder "
            "reaches 96 px because the interface scale reaches 200 %"
        ),
        "openpbr": {
            "standard": "OpenPBR Surface (AcademySoftwareFoundation/OpenPBR)",
            "rendered_through": "Principled BSDF v2",
            "conformance": (
                "authored against the OpenPBR parameter set and rendered through a "
                "shader that is OpenPBR-aligned but not conformance-exact; Blender "
                "5.3.0 Alpha has no OpenPBR node and no MaterialX operators, and "
                "upstream issue #145127 records exact import/export as wanted rather "
                "than delivered"
            ),
            "mapping": openpbr.MAPPING,
            "not_mapped": openpbr.UNSUPPORTED,
        },
        "rig": rig.rig_record(),
        "generations": shapes.TREATMENTS,
        "candidate_model": (
            "six total: three emphases — MD1-led, TouchWiz-led and iOS 6-led — each in "
            "two constructions, so the family spread is a choice rather than a note"
        ),
        "composition": {
            "roles": list(shapes.COMPOSITION_ROLES),
            "tiers": shapes.TIERS,
            "emphases": [dict(emphasis) for emphasis in shapes.EMPHASES],
            "constructions": [dict(construction) for construction in shapes.CONSTRUCTIONS],
            "neutral_roles_not_counted": list(shapes.NEUTRAL_ROLES),
        },
        "material_matrix": palette.matrix_record(),
        "silhouette_policy": {
            "reference": "Material Design Icons 4.0.0",
            "style": shapes.MD1_STYLE,
            "size_dp": shapes.MD1_SIZE_DP,
            "scale": shapes.MD1_SCALE,
            "side_px": shapes.MD1_SIDE_PX,
            "why_this_size": (
                "48dp at 2x is 96x96, which is the decision size, so the comparison "
                "resamples neither side"
            ),
            "containment_floor": SILHOUETTE_CONTAINMENT_FLOOR,
            "gate": (
                "every cell the declared mark covers must be covered by the render; "
                "the render may add, because the accent piece leaves the mark's box "
                "by design"
            ),
            "not_gated": "whole-icon occupancy agreement is recorded, never enforced",
        },
        "deferred_inventory": [dict(item) for item in shapes.DEFERRED_INVENTORY],
        "retired_pilots": {
            icon_id: record["retired_because"] for icon_id, record in shapes.RETIRED_PILOTS.items()
        },
        "quality_bar": {
            "reference_grammar": "original translation of the Galaxy TouchWiz study pack",
            "presentation": "transparent object render; no shared frame",
            "materials": "procedural only",
            "pilot_gate": "settings must be visually and mechanically accepted before the catalogue is promoted",
        },
        "vocabulary_refusals": shapes.VOCABULARY_REFUSALS,
        "checks": {
            "high_dpi_policy": "generated only after a human target; master failure is visible but not shippable",
            "what_this_is": (
                "each generation against the declared material (measured swatch), the "
                "ink against every host surface it declares (WCAG 1.4.11 non-text "
                "contrast, 75th percentile of covered pixels), the accent against the "
                "icon's own body, ink coverage at the smallest shipped size, and the "
                "hue against the declared colour"
            ),
            "icon_to_swatch_band": list(ICON_TO_SWATCH_BAND),
            "swatch_to_albedo_band": list(SWATCH_TO_ALBEDO_BAND),
            "chromaticity_tolerance_degrees": CHROMATICITY_TOLERANCE_DEGREES,
            "coverage_bounds_at_24": [COVERAGE_FLOOR_24, COVERAGE_CEILING_24],
            "framed_coverage_ceiling_at_24": FRAMED_COVERAGE_CEILING_24,
            "internal_min_contrast": INTERNAL_MIN_CONTRAST,
        },
    }


def read_previous_sources(path: str) -> dict:
    """The inputs the last render was made from, before this run overwrites them."""
    try:
        with open(path, "r", encoding="utf-8") as handle:
            return json.load(handle).get("sources", {})
    except (OSError, json.JSONDecodeError):
        return {}


def divergence_record(previous: dict, current: dict) -> dict:
    """Which inputs moved since the last render.

    C3's answer for "what makes a rebuild happen" was a human bump of the concept
    set. A human can only bump what they can see, so the manifest **reports** — it
    never enforces, and the report is a statement about the last run rather than a
    gate on this one. The design spec is in the set now, so a change to the standard
    is visible too.
    """
    moved = [
        name for name in sorted(set(previous) | set(current))
        if previous.get(name) != current.get(name)
    ]
    return {
        "what_this_is": (
            "inputs that differ from the render this manifest replaced; reported, "
            "never enforced"
        ),
        "inputs_that_moved": moved,
        "inputs_hash_matches_previous_run": not moved,
        "then": previous,
        "now": current,
    }


def build_manifest(rendered) -> tuple:
    path = os.path.join(OUT, "manifest.json")
    previous_sources = read_previous_sources(path)
    record = build_record("manifest")
    record["input_divergence"] = divergence_record(previous_sources, record["sources"])
    record["silhouettes"] = {
        entry["id"]: entry.get("silhouette") for entry in rendered if entry.get("silhouette")
    }
    # The three design languages as measured anchors, with their population, method
    # and digests, so "this candidate is TouchWiz-led" is checkable against a number
    # that came from the archive rather than from a label.
    record["languages"] = shapes.ladder_record()
    record["palette_mechanisms"] = palette.colour_mechanisms()
    icons = []
    directives = read_directives(shapes.CONCEPT_SET)
    for entry in rendered:
        # The generations are checked first, because the icon's verdict is their
        # notes in one place: a verdict assembled before the checks would be a
        # summary of nothing.
        generations = []
        for generation in entry["generations"]:
            generation = dict(generation)
            generation["checks"] = judge_generation(entry, generation)
            generations.append(generation)
        judged = judge_icon({**entry, "generations": generations})
        icons.append(
            {
                **{k: v for k, v in entry.items() if k != "generations"},
                "verdict": judged["verdict"],
                "verdict_notes": judged["notes"],
                # Carried into the shipping manifest as well as the review set: an
                # icon that shipped with an unanswered comment is a promise the
                # record can still see.
                "directives": directives.get(entry["id"], []),
                "generations": generations,
            }
        )
    record["icons"] = icons
    with open(path, "w", encoding="utf-8") as handle:
        json.dump(record, handle, indent=2, sort_keys=True)
        handle.write("\n")
    return record, path


def build_review(rendered) -> tuple:
    """What the picker reads: six explicit concepts per icon and their briefs."""
    icons = []
    awaiting = []
    decided = {}
    directives = read_directives(shapes.CONCEPT_SET)
    for entry in rendered:
        generations = []
        for generation in entry["generations"]:
            gen_id = generation["generation"]
            sizes = {str(size["px"]): size["measurements"] for size in generation["sizes"]}
            generations.append(
                {
                    "id": gen_id,
                    "label": generation["generation_label"],
                    "why": generation["generation_why"],
                "part_count": generation["part_count"],
                "primary_role": generation.get("primary_role", "ink"),
                "transform": generation["transform"],
                "quality": {
                    "reference_grammar": "Galaxy TouchWiz study grammar, original geometry",
                    "source_artwork_copied": False,
                    "procedural_materials": True,
                },
                    "files": {
                        "sharp_png": f"review/{entry['id']}.{gen_id}.{DECISION_PX}.png",
                        "raw": f"review/{entry['id']}.{gen_id}.{DECISION_PX}.rgba",
                        "recognition_png": f"review/{entry['id']}.{gen_id}.{RECOGNITION_PX}.png",
                        "recognition_raw": f"review/{entry['id']}.{gen_id}.{RECOGNITION_PX}.rgba",
                        "context_png": f"review/{entry['id']}.{gen_id}.{CONTEXT_PX}.png",
                        "context_raw": f"review/{entry['id']}.{gen_id}.{CONTEXT_PX}.rgba",
                    },
                    "measurements": sizes,
                    "checks": judge_generation(entry, generation),
                    "selection_notes": generation.get("selection_notes", []),
                    "brief": generation.get("brief", {}),
                }
            )
        chosen = entry["chosen_generation"]
        chosen_record = next(
            (generation for generation in generations if generation["id"] == chosen),
            None,
        )
        if chosen is None or not chosen_record or chosen_record["selection_notes"]:
            awaiting.append(entry["id"])
        else:
            decided[entry["id"]] = chosen
        icons.append(
            {
                "id": entry["id"],
                "concept_set": shapes.CONCEPT_SET,
                "chosen_generation": entry["chosen_generation"],
                "kind": entry["kind"],
                "meaning": entry["meaning"],
                "source": entry["source"],
                "locates": entry["locates"],
                "sits_on": entry["sits_on"],
                "identity": entry["identity"],
                "identity_as": entry["identity_as"],
                "brief": entry["brief"],
                "lineage": entry["lineage"],
                "forbidden_readings": entry["forbidden_readings"],
                # What was asked for last time, in the words it was asked in. The
                # picker shows these beside the candidates so the next generation is
                # authored against a person's own sentences rather than a summary.
                "directives": directives.get(entry["id"], []),
                "generations": generations,
            }
        )
    record = {
        "what_this_is": (
            "six explicit authored concepts of each pilot icon; a person must accept "
            "one before anything enters the shipping set"
        ),
        "concept_set": shapes.CONCEPT_SET,
        "catalogue_stage": shapes.CATALOGUE_STAGE,
        "how_to_decide": (
            "cargo run --release --bin pick — check one concept and press Enter to "
            "make it the target, and type in the comment box to say what the next "
            "generation should change (a comment is recorded with or without a target)"
        ),
        "directives_log": DECISIONS,
        "decision_size_px": DECISION_PX,
        "recognition_size_px": RECOGNITION_PX,
        "context_size_px": CONTEXT_PX,
        "raw": {
            "format": "RGBA8, unpremultiplied",
            "layout": "row-major, top-down, origin top-left",
            "side_px": DECISION_PX,
            "recognition_side_px": RECOGNITION_PX,
            "context_side_px": CONTEXT_PX,
        },
        "hosts": list(palette.HOSTS),
        "reference_study": shapes.REFERENCE_STUDY,
        "quality_bar": {
            "reference_grammar": "original translation of the Galaxy TouchWiz study pack",
            "presentation": "transparent object render; no shared frame",
            "materials": "procedural only",
            "pilot_gate": "settings first",
        },
        # The picker paints with these, taken from the same token table the icons
        # were rendered against: a background colour retyped in the tool would be a
        # second source of truth for the surface an icon is judged on.
        "fills": {
            name: list(palette.linear(name))
            for name in ("Desk", "Panel", "PanelRaised", "Ink", "Warning", "Nature", "TextBody")
        },
        "host_fills": {host: list(palette.linear(host)) for host in palette.HOSTS},
        "generations": shapes.TREATMENTS,
        "candidate_count": 6,
        "awaiting_decision": awaiting,
        "decided": decided,
        "blender": {
            "version": bpy.app.version_string,
            "build_hash": bpy.app.build_hash.decode()
            if isinstance(bpy.app.build_hash, bytes)
            else str(bpy.app.build_hash),
        },
        "palette_hash": palette.palette_hash(),
        "icons": icons,
    }
    path = os.path.join(OUT, "review.json")
    with open(path, "w", encoding="utf-8") as handle:
        json.dump(record, handle, indent=2, sort_keys=True)
        handle.write("\n")
    return record, path


def main():
    device = "GPU" if "--cpu" not in sys.argv else "CPU"
    tier = "stage-1"
    if "--review" in sys.argv:
        tier = sys.argv[sys.argv.index("--review") + 1]
    limit = None
    if "--limit" in sys.argv:
        limit = int(sys.argv[sys.argv.index("--limit") + 1])

    rendered = generate(device, tier, limit)
    manifest, manifest_path = build_manifest(rendered)
    review, review_path = build_review(rendered)

    noted = [icon for icon in manifest["icons"] if icon["verdict_notes"]]
    print(f"wrote {manifest_path}")
    print(f"wrote {review_path}")
    print(
        f"blender {manifest['blender']['version']} ({manifest['blender']['build_hash']}) "
        f"on {device}, tier {tier}"
    )
    print(
        f"icons: {len(manifest['icons'])} "
        f"({len(review['decided'])} decided, {len(review['awaiting_decision'])} awaiting)"
    )
    for icon in noted:
        for note in icon["verdict_notes"]:
            print(f"  {icon['id']}: {note}")

    # Reported, never enforced: the concept set is bumped by a person, and this is
    # the part that tells that person what moved since the last render.
    divergence = manifest.get("input_divergence", {})
    if divergence.get("inputs_hash_matches_previous_run"):
        print("inputs: nothing moved since the last render")
    elif divergence:
        print(f"inputs that moved since the last render ({len(divergence['inputs_that_moved'])}):")
        for name in divergence["inputs_that_moved"]:
            print(f"  {name}")

    # What a person asked for, in their own words, so the next generation is
    # authored against the log rather than against memory.
    outstanding = [
        (icon["id"], directive)
        for icon in manifest["icons"]
        for directive in icon["directives"]
    ]
    if outstanding:
        print(f"directives from the review log ({len(outstanding)}):")
        for icon_id, directive in outstanding:
            print(f"  {icon_id}: {directive}")


if __name__ == "__main__":
    main()
