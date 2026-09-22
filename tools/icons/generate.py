"""Render the inventory in generations, measure everything, and write the record.

Run it with Blender, which is the only thing here that needs Blender:

    blender --background --factory-startup --python tools/icons/generate.py -- --review surface

What it writes:

* `assets/icons/review/<id>.<generation>.<px>.png` — the three generations of one
  icon, which is what a person chooses between. This is the review set, not the
  shipping set.
* `assets/icons/review/<id>.<generation>.96.rgba` — the decision size as raw RGBA,
  which is what the picker and the game read. The runtime carries **no PNG
  decoder**: PNG is for people, RGBA is for the program, and the manifest says
  which is which.
* `assets/icons/review.json` — the picker's input: what awaits a decision, the
  three generations' declared transforms, their measurements, and the identity of
  the build that made them.
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
COVERAGE_CEILING_24 = 0.62

#: A framed view icon carries a border, and the border must not become the icon.
FRAMED_COVERAGE_CEILING_24 = 0.55

#: An accent drawn against the icon's own body needs at least this much separation
#: from it, or the detail is decoration that cannot be read.
INTERNAL_MIN_CONTRAST = 1.5

#: Below this much separation between a colour's channels, the hue check is noise.
NEUTRAL_CHROMA_THRESHOLD = 0.06


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


def measure(pixels, px: int, alpha_threshold: float = 0.5) -> dict:
    """Coverage, luminance distribution and chromaticity of a rendered icon.

    The **75th percentile** is the number the contrast check uses, and it is the
    honest middle: the mean of a shaded solid is dragged down by its own shadow
    side, and the peak is one highlight pixel. What §1.4.11 asks is whether the
    component can be *perceived*, so the check asks what most of the glyph is
    doing, and the mean and peak are recorded beside it so a reader can disagree.
    """
    covered = 0
    luminances = []
    sum_r = sum_g = sum_b = 0.0
    for index in range(px * px):
        alpha = pixels[index * 4 + 3] / 255.0
        if alpha < alpha_threshold:
            continue
        covered += 1
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
        }

    luminances.sort()
    mean = (sum_r / covered, sum_g / covered, sum_b / covered)
    return {
        "coverage": round(covered / (px * px), 4),
        "mean_luminance": round(relative_luminance(mean), 4),
        "p75_luminance": round(luminances[int(0.75 * (covered - 1))], 4),
        "peak_luminance": round(luminances[-1], 4),
        "chromaticity_degrees": round(chromaticity_angle(mean), 2),
    }


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


def read_render(bpy, path: str) -> bytes:
    """Read a saved render back as stored, then put the rows the right way up."""
    raw = rig.read_png_non_color(bpy, path)
    stored = bytes(min(255, max(0, round(channel * 255.0))) for channel in raw)
    return flip_rows(stored, rig.RENDER_PX)


# ---------------------------------------------------------------------------
# Rendering
# ---------------------------------------------------------------------------


def build_materials(bpy, entry):
    """One Blender material per declared role, plus the record for the manifest."""
    materials = {}
    records = {}
    for role, parameters in entry["materials"].items():
        name = f"openpbr:{entry['id']}:{entry['generation']}:{role}"
        materials[role] = openpbr.build_material(bpy, name, parameters)
        records[role] = openpbr.record(parameters)
    return materials, records


def material_key(parameters) -> str:
    """A stable key for a material, so identical materials share one swatch render."""
    return json.dumps(openpbr.record(parameters)["openpbr"], sort_keys=True)


def render_to(bpy, scene, path: str) -> bytes:
    scene.render.filepath = path
    bpy.ops.render.render(write_still=True)
    pixels = read_render(bpy, path)
    os.remove(path)
    return pixels


def measure_swatch(bpy, scene, material, key, cache, counter):
    """The material's own brightness under the same rig, measured once per material."""
    if key in cache:
        return cache[key]
    objects = shapes.swatch_geometry(bpy, material)
    path = os.path.join(REVIEW, f"_swatch.{counter[0]}.render.png")
    counter[0] += 1
    pixels = render_to(bpy, scene, path)
    small = rig.downsample(pixels, rig.RENDER_PX, DECISION_PX)
    cache[key] = measure(small, DECISION_PX)
    for obj in objects:
        bpy.data.objects.remove(obj, do_unlink=True)
    return cache[key]


def generate(device: str, tier: str, limit=None):
    os.makedirs(REVIEW, exist_ok=True)
    scene = rig.configure(bpy, device=device)
    decisions = read_decisions()
    swatches = {}
    counter = [0]
    entries = [e for e in shapes.ICONS if tier == "all" or e["kind"] == tier]
    if limit:
        entries = entries[:limit]
    rendered = []

    for entry in entries:
        chosen = decisions.get(entry["id"], {}).get("generation")
        generations = []
        for generation in shapes.generations_of(entry):
            gen_id = generation["generation"]
            materials, material_records = build_materials(bpy, generation)
            objects = shapes.build(bpy, generation, materials)

            pixels = render_to(
                bpy, scene, os.path.join(REVIEW, f"{entry['id']}.{gen_id}.render.png")
            )

            sizes = []
            for px in (DECISION_PX, CONTEXT_PX):
                small = rig.downsample(pixels, rig.RENDER_PX, px)
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
                sizes.append({"px": px, "measurements": measure(small, px)})

            swatch_measurements = {}
            for role, parameters in generation["materials"].items():
                key = material_key(parameters)
                swatch_measurements[role] = measure_swatch(
                    bpy, scene, materials[role], key, swatches, counter
                )

            promoted = None
            if chosen == gen_id:
                promoted = promote(bpy, entry, generation, pixels)

            for obj in objects:
                bpy.data.objects.remove(obj, do_unlink=True)
            for material in materials.values():
                bpy.data.materials.remove(material)

            record = {
                "generation": gen_id,
                "generation_label": generation["generation_label"],
                "generation_why": generation["generation_why"],
                "transform": transform_record(entry, generation),
                "materials": material_records,
                "swatches": swatch_measurements,
                "sizes": sizes,
                "promoted": promoted,
            }
            generations.append(record)

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
                "notes": entry["notes"],
                "chosen_generation": chosen,
                "generations": generations,
            }
        )
    return rendered


def transform_record(entry, generation) -> dict:
    """The declared transform that made this generation, not a diff of pixels."""
    treatment = next(
        (t for t in shapes.TREATMENTS if t["id"] == generation["generation"]), None
    )
    if treatment is None:
        return {"note": "no declared treatment matched this generation"}
    return {
        "scale": treatment["scale"],
        "stroke_weight": treatment["weight"],
        "material_treatment": treatment["material"],
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
    notes = []
    sharp = next(s for s in generation["sizes"] if s["px"] == DECISION_PX)["measurements"]
    small = next(s for s in generation["sizes"] if s["px"] == CONTEXT_PX)["measurements"]
    ink = generation["materials"]["ink"]["openpbr"]["base_color"]["linear_rgb"]
    identity = generation["materials"].get("identity", {}).get("openpbr", {}).get(
        "base_color", {}
    ).get("linear_rgb")

    # --- is anything there, and is there too much -------------------------
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
    swatch = generation["swatches"]["ink"]
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
        if not low <= arrival["swatch_to_albedo"] <= high:
            notes.append(
                f"a flat swatch of the ink renders at {arrival['swatch_to_albedo']}x "
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

    return {"notes": notes, "contrasts": contrasts, "internal_contrast": internal,
            "arrival": arrival}


def judge_icon(record) -> dict:
    """The icon's verdict, which is every generation's checks in one place."""
    notes = []
    if record["kind"] == "surface" and record["locates"] not in shapes.DECLARED_SURFACES:
        notes.append(
            f"this is a surface icon that locates `{record['locates']}`, which is not a "
            f"surface the game declares: an icon may only locate something that exists"
        )
    for generation in record["generations"]:
        for note in generation["checks"]["notes"]:
            notes.append(f"[{generation['generation']}] {note}")
    if record["chosen_generation"] is None:
        notes.append(
            "no generation has been marked as the target: awaiting a decision in the "
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


def read_decisions() -> dict:
    """The last `target: true` line per icon, from the picker's own log."""
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
            if record.get("target"):
                chosen[record.get("icon")] = record
    return chosen


def read_directives() -> dict:
    """Every comment a person left, per icon, oldest first.

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
        "src/hud.rs": os.path.join(ROOT, "src", "hud.rs"),
        "src/design.rs": os.path.join(ROOT, "src", "design.rs"),
    }
    return {name: palette.source_hash(path) for name, path in files.items()}


def build_record(tier: str) -> dict:
    return {
        "generated_by": "tools/icons/generate.py",
        "how_to_regenerate": (
            "blender --background --factory-startup --python "
            "tools/icons/generate.py -- --review surface"
        ),
        "why_committed": (
            "cargo build must never need Blender; the renders are artifacts with "
            "provenance, and this script is how they were produced"
        ),
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
        "palette_hash": palette.palette_hash(),
        "hosts": list(palette.HOSTS),
        "non_text_contrast_floor": palette.NON_TEXT_MIN_CONTRAST,
        "sizes_px": list(rig.SHIPPED_PX),
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
        "vocabulary_refusals": shapes.VOCABULARY_REFUSALS,
        "checks": {
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


def build_manifest(rendered) -> tuple:
    record = build_record("manifest")
    icons = []
    directives = read_directives()
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
    path = os.path.join(OUT, "manifest.json")
    with open(path, "w", encoding="utf-8") as handle:
        json.dump(record, handle, indent=2, sort_keys=True)
        handle.write("\n")
    return record, path


def build_review(rendered) -> tuple:
    """What the picker reads: three generations per icon, and what it may claim."""
    icons = []
    awaiting = []
    decided = {}
    directives = read_directives()
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
                    "transform": generation["transform"],
                    "files": {
                        "sharp_png": f"review/{entry['id']}.{gen_id}.{DECISION_PX}.png",
                        "raw": f"review/{entry['id']}.{gen_id}.{DECISION_PX}.rgba",
                        "context_png": f"review/{entry['id']}.{gen_id}.{CONTEXT_PX}.png",
                        "context_raw": f"review/{entry['id']}.{gen_id}.{CONTEXT_PX}.rgba",
                    },
                    "measurements": sizes,
                    "checks": judge_generation(entry, generation),
                }
            )
        if entry["chosen_generation"] is None:
            awaiting.append(entry["id"])
        else:
            decided[entry["id"]] = entry["chosen_generation"]
        icons.append(
            {
                "id": entry["id"],
                "kind": entry["kind"],
                "meaning": entry["meaning"],
                "source": entry["source"],
                "locates": entry["locates"],
                "sits_on": entry["sits_on"],
                "identity": entry["identity"],
                "identity_as": entry["identity_as"],
                # What was asked for last time, in the words it was asked in. The
                # picker shows these beside the candidates so the next generation is
                # authored against a person's own sentences rather than a summary.
                "directives": directives.get(entry["id"], []),
                "generations": generations,
            }
        )
    record = {
        "what_this_is": (
            "three generations of each icon, to be chosen between by a person; the "
            "choice is the only thing that puts an icon into the shipping set"
        ),
        "how_to_decide": (
            "cargo run --release --bin pick — check one generation and press Enter to "
            "make it the target, and type in the comment box to say what the next "
            "generation should change (a comment is recorded with or without a target)"
        ),
        "directives_log": DECISIONS,
        "decision_size_px": DECISION_PX,
        "context_size_px": CONTEXT_PX,
        "raw": {
            "format": "RGBA8, unpremultiplied",
            "layout": "row-major, top-down, origin top-left",
            "side_px": DECISION_PX,
        },
        "hosts": list(palette.HOSTS),
        # The picker paints with these, taken from the same token table the icons
        # were rendered against: a background colour retyped in the tool would be a
        # second source of truth for the surface an icon is judged on.
        "fills": {
            name: list(palette.linear(name))
            for name in ("Desk", "Panel", "PanelRaised", "Ink", "Warning", "Nature", "TextBody")
        },
        "host_fills": {host: list(palette.linear(host)) for host in palette.HOSTS},
        "generations": shapes.TREATMENTS,
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
    tier = "surface"
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
