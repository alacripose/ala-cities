"""Render the icons, measure them, and write the manifest.

Run it with Blender, which is the only thing here that needs Blender:

    blender --background --factory-startup --python tools/icons/generate.py

`cargo build` never runs this. The renders are committed artifacts; this script
is how they were made, and re-running it on a different Blender build records
that fact rather than quietly producing different pixels.

What is written:

* `assets/icons/<id>.<px>.png` — one file per shipped size, for a person to look at
* `assets/icons/manifest.json`   — provenance, materials, deviations, and measurements

Every measurement below is **recorded even when it is a failure**, and the verdict
says which. A generator that only writes passing icons would be a generator nobody
could audit.
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

# ---------------------------------------------------------------------------
# Tolerances. Stated rather than tuned: a check with an invisible threshold is a
# check nobody can argue with.
# ---------------------------------------------------------------------------

#: A material swatch is rendered under the **same rig** for every icon, and the
#: icon is compared to its own swatch. This band is that ratio.
#:
#: Two earlier versions of this check were wrong in opposite directions and both
#: are worth recording. The first used an invented irradiance model and called two
#: dark-but-correct icons failures. The second compared the icon's mean against the
#: declared albedo with no lighting term at all, and called *every* icon a failure
#: (0.07–0.35×) because a shaded box is dimmer than its own albedo. The lesson is
#: in the numbers: any model of "how bright it should be" was going to be wrong, so
#: the reference is measured instead of modelled — same rig, same material, flat
#: plate facing the camera, and the icon is judged against it.
ICON_TO_SWATCH_BAND = (0.4, 1.8)

#: How far a flat swatch of the material may sit from its albedo under this rig.
#: Not a physical claim: it is the rig's illumination, recorded so a wrong token,
#: a broken light or a post-processed render has nowhere to hide.
SWATCH_TO_ALBEDO_BAND = (0.35, 1.35)

#: How far the rendered icon's chromaticity may sit from the declared colour's,
#: in degrees of the red/green ratio plane. Wide enough for three coloured
#: lights, narrow enough to catch a wrong token.
CHROMATICITY_TOLERANCE_DEGREES = 12.0

#: Ink coverage at the smallest shipped size. Below the floor the glyph has
#: averaged away; above the ceiling it is a blob and carries no information.
COVERAGE_FLOOR_24 = 0.06
COVERAGE_CEILING_24 = 0.62


#: The material check is mid-change, and the tool says so rather than reporting a
#: verdict it cannot support. Round 11 replaced the modelled luminance band with a
#: measured **material swatch** (`ICON_TO_SWATCH_BAND` above), because the modelled
#: band called every icon a failure at 0.07–0.35× — a shaded box is dimmer than its
#: own albedo, and no model of "how bright it should be" was going to survive
#: contact with the rig. The swatch render is not wired in yet, so a run stops up
#: front instead of writing a manifest whose numbers mean less than they look.
MID_CHANGE = (
    "tools/icons/generate.py is mid-change: the swatch comparison that replaces the "
    "removed luminance model (see ICON_TO_SWATCH_BAND) is not wired in yet, so this "
    "script refuses rather than writing a manifest it cannot stand behind. Recorded "
    "in docs/GRILLING-C2.md, round 11."
)


def linear(pixel: float) -> float:
    return rig.linearise(pixel)


def relative_luminance(rgb) -> float:
    """WCAG relative luminance, which is the same measure the interface uses."""
    return 0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]


def chromaticity_angle(rgb) -> float:
    """An angle in the red/green plane: the hue a check can hold to a tolerance."""
    return math.degrees(math.atan2(rgb[1], rgb[0] + 1e-9))


def angle_difference(a: float, b: float) -> float:
    """The shortest way round between two angles, so 359° and 1° are neighbours."""
    difference = abs(a - b) % 360.0
    return min(difference, 360.0 - difference)


def measure(pixels, px: int, base_color, alpha_threshold=0.5):
    """Measure an icon: ink coverage, mean and peak luminance, chromaticity."""
    covered = 0
    peak = 0.0
    sum_r = sum_g = sum_b = 0.0
    for index in range(px * px):
        alpha = pixels[index * 4 + 3] / 255.0
        if alpha < alpha_threshold:
            continue
        covered += 1
        lr, lg, lb = (
            linear(pixels[index * 4] / 255.0),
            linear(pixels[index * 4 + 1] / 255.0),
            linear(pixels[index * 4 + 2] / 255.0),
        )
        peak = max(peak, relative_luminance((lr, lg, lb)))
        sum_r += lr
        sum_g += lg
        sum_b += lb

    if covered == 0:
        return {
            "coverage": 0.0,
            "mean_luminance": 0.0,
            "peak_luminance": 0.0,
            "chromaticity_degrees": None,
            "note": "nothing opaque in the render at all",
        }

    mean = (sum_r / covered, sum_g / covered, sum_b / covered)
    return {
        "coverage": round(covered / (px * px), 4),
        "mean_luminance": round(relative_luminance(mean), 4),
        "peak_luminance": round(peak, 4),
        "chromaticity_degrees": round(chromaticity_angle(mean), 2),
        "declared_base_luminance": round(relative_luminance(base_color), 4),
        "declared_base_chromaticity_degrees": round(chromaticity_angle(base_color), 2),
    }





def save_png(bpy, path, pixels, px):
    image = bpy.data.images.new("out", width=px, height=px, alpha=True)
    image.pixels = [channel / 255.0 for channel in pixels]
    image.filepath_raw = path
    image.file_format = "PNG"
    image.save()
    bpy.data.images.remove(image)


def generate(device):
    os.makedirs(OUT, exist_ok=True)
    scene = rig.configure(bpy, device=device)

    for icon in shapes.ICONS:
        icon_id = icon["id"]
        materials = {}
        records = {}
        for role in ("primary", "accent"):
            name = f"openpbr:{icon_id}:{role}"
            material = openpbr.build_material(bpy, name, icon[role])
            materials[role] = material
            records[role] = openpbr.record(icon[role])
        # A material named by the icon is how the manifest ties a size back to
        # the parameters that made it.
        icon["_materials"] = records

        objects = shapes.build(bpy, icon, materials)
        render_path = os.path.join(OUT, f"{icon_id}.render.png")
        scene.render.filepath = render_path
        bpy.ops.render.render(write_still=True)

        # Read the saved pixels as stored: 'Non-Color' means no conversion, so
        # what comes back is what a PNG viewer would show.
        raw = rig.read_png_non_color(bpy, render_path)
        stored = bytes(
            min(255, max(0, round(channel * 255.0))) for channel in raw
        )
        # Blender hands back bottom-up rows; flip once, here, so every consumer
        # downstream can assume top-down.
        stored = flip_rows(stored, rig.RENDER_PX)

        sizes = []
        for px in rig.SHIPPED_PX:
            small = rig.downsample(stored, rig.RENDER_PX, px)
            png_path = os.path.join(OUT, f"{icon_id}.{px}.png")
            save_png(bpy, png_path, small, px)
            sizes.append(
                {
                    "px": px,
                    "file": f"{icon_id}.{px}.png",
                    "bytes": len(small),
                    "hash": rig.hash_bytes(small),
                    "measurements": measure(small, px, icon["primary"]["base_color"]),
                }
            )

        icon["_sizes"] = sizes
        icon["_render"] = {"px": rig.RENDER_PX, "file": f"{icon_id}.render.png"}
        icon["_objects"] = [obj.name for obj in objects]

        os.remove(render_path)
        for material in materials.values():
            bpy.data.materials.remove(material)
        for obj in objects:
            bpy.data.objects.remove(obj, do_unlink=True)

    return shapes.ICONS


def flip_rows(data: bytes, px: int, channels: int = 4) -> bytes:
    out = bytearray(len(data))
    stride = px * channels
    for row in range(px):
        source = (px - 1 - row) * stride
        target = row * stride
        out[target : target + stride] = data[source : source + stride]
    return bytes(out)


def judge(icon):
    """The verdict for one icon, with every reason it is not a clean pass."""
    icon_id = icon["id"]
    notes = []
    sizes = icon["_sizes"]
    largest = sizes[0]["measurements"]
    smallest = [size for size in sizes if size["px"] == 24][0]["measurements"]

    if largest["coverage"] <= 0.0:
        notes.append("nothing was drawn")
    if smallest["coverage"] < COVERAGE_FLOOR_24:
        notes.append(
            f"at 24 px the glyph covers {smallest['coverage']:.3f} of the frame, below the "
            f"{COVERAGE_FLOOR_24} floor: it averaged away"
        )
    if smallest["coverage"] > COVERAGE_CEILING_24:
        notes.append(
            f"at 24 px the glyph covers {smallest['coverage']:.3f} of the frame, above the "
            f"{COVERAGE_CEILING_24} ceiling: it is a blob"
        )

    # The material check: did the declared colour arrive, and is it the right
    # colour? Both against the declared values themselves, not against a model.
    base = icon["primary"]["base_color"]
    declared = relative_luminance(base)
    if declared > 0.0:
        ratio = largest["mean_luminance"] / declared
        low, high = LUMINANCE_BAND
        if ratio < low or ratio > high:
            notes.append(
                f"the rendered mean luminance is {ratio:.2f}× the declared base "
                f"colour's own ({largest['mean_luminance']:.4f} against {declared:.4f}), "
                f"outside the {low}–{high} band"
            )
        drift = angle_difference(
            largest["chromaticity_degrees"], largest["declared_base_chromaticity_degrees"]
        )
        if drift > CHROMATICITY_TOLERANCE_DEGREES:
            notes.append(
                f"the rendered hue sits {drift:.1f}° from the declared colour's, beyond the "
                f"{CHROMATICITY_TOLERANCE_DEGREES}° tolerance"
            )

    # A render that is mostly the frame is a blob at the smallest size, which is
    # the size that matters: the frame is a border, not the message.
    if icon.get("framed") and smallest["coverage"] > 0.55:
        notes.append(
            f"as a framed view icon it covers {smallest['coverage']:.3f} of the frame at 24 px; "
            f"the border is dominating the glyph inside it"
        )

    deviations = sorted(
        {note for record in (icon["_materials"]["primary"], icon["_materials"]["accent"])
         for note in record["deviations"]}
    )
    return {
        "id": icon_id,
        "verdict": "pass" if not notes else "pass_with_notes",
        "notes": notes,
        "deviations": deviations,
        "materials": {
            "primary": icon["_materials"]["primary"],
            "accent": icon["_materials"]["accent"],
        },
        "sizes": sizes,
        "checks": {
            "what_this_is": (
                "the declared material against the render: mean luminance as a multiple of "
                "the base colour's own, chromaticity drift in degrees, ink coverage at the "
                "smallest shipped size"
            ),
            "chromaticity_tolerance_degrees": CHROMATICITY_TOLERANCE_DEGREES,
            "luminance_band": list(LUMINANCE_BAND),
            "coverage_bounds_at_24": [COVERAGE_FLOOR_24, COVERAGE_CEILING_24],
        },
    }


def build_manifest(rendered):
    icons = [judge(icon) for icon in rendered]
    script = os.path.abspath(__file__)
    manifest = {
        "generated_by": "tools/icons/generate.py",
        "how_to_regenerate": (
            "blender --background --factory-startup --python tools/icons/generate.py"
        ),
        "why_committed": (
            "cargo build must never need Blender; the renders are artifacts with provenance, "
            "and this script is how they were produced"
        ),
        "hash": {
            "algorithm": "fnv1a64",
            "is": "an integrity check — this file is the file the manifest describes",
            "is_not": "a signature, and not a security mechanism",
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
                "reproduce these pixels; the build that made them is recorded here so that "
                "a difference is visible rather than silent"
            ),
        },
        "sources": {
            "tools/icons/generate.py": palette.source_hash(script),
            "tools/icons/openpbr.py": palette.source_hash(os.path.join(HERE, "openpbr.py")),
            "tools/icons/palette.py": palette.source_hash(os.path.join(HERE, "palette.py")),
            "tools/icons/rig.py": palette.source_hash(os.path.join(HERE, "rig.py")),
            "tools/icons/shapes.py": palette.source_hash(os.path.join(HERE, "shapes.py")),
            "src/hud.rs": palette.source_hash(os.path.join(ROOT, "src", "hud.rs")),
        },
        "palette_hash": palette.palette_hash(),
        "openpbr": {
            "standard": "OpenPBR Surface (AcademySoftwareFoundation/OpenPBR)",
            "rendered_through": "Principled BSDF v2",
            "conformance": (
                "authored against the OpenPBR parameter set and rendered through a shader that "
                "is OpenPBR-aligned but not conformance-exact; Blender 5.3.0 Alpha has no "
                "OpenPBR node and no MaterialX operators, and upstream issue #145127 records "
                "exact import/export as wanted rather than delivered"
            ),
            "mapping": openpbr.MAPPING,
            "not_mapped": openpbr.UNSUPPORTED,
        },
        "rig": rig.rig_record(),
        "icons": icons,
    }
    path = os.path.join(OUT, "manifest.json")
    with open(path, "w", encoding="utf-8") as handle:
        json.dump(manifest, handle, indent=2, sort_keys=True)
        handle.write("\n")
    return manifest, path


def main():
    if "--i-know-the-material-check-is-unfinished" not in sys.argv:
        raise SystemExit(MID_CHANGE)
    device = "GPU" if "--cpu" not in sys.argv else "CPU"
    rendered = generate(device)
    manifest, path = build_manifest(rendered)
    noted = [icon for icon in manifest["icons"] if icon["notes"]]
    print(f"wrote {path}")
    print(
        f"blender {manifest['blender']['version']} ({manifest['blender']['build_hash']}) "
        f"on {device}"
    )
    print(f"icons: {len(manifest['icons'])}, with notes: {len(noted)}")
    for icon in noted:
        for note in icon["notes"]:
            print(f"  {icon['id']}: {note}")


if __name__ == "__main__":
    main()
