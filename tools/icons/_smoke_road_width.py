"""How the road's fill responds to its own carriageway, at the md1 end.

The road's md1 anchor is 0.2825, measured from its own reference (`maps/add_road`),
while the built object measures 0.502 — and a184/a202 say the body is one piece, so
"fragment it until the number matches" is not available. This asks the narrower
question that decides it: can a **single-piece** solid ribbon reach its reference's
range at all, and by which parameter?

Scaffolding rather than pipeline: the numbers it prints belong in the record and in
`families.py`'s sources, and it is the thing to delete once they are there.

    blender --background --factory-startup --python tools/icons/_smoke_road_width.py
"""

import os
import sys

import bpy
import mathutils

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import families  # noqa: E402
import generate  # noqa: E402
import openpbr  # noqa: E402
import palette  # noqa: E402
import rig  # noqa: E402
import shapes  # noqa: E402

#: (lanes, lane_width, corner_rounding, curb_width) at the md1 end, with what each is
#: asking: the declared pair, then one lane, then progressively thinner carriageways.
CASES = (
    (2, 0.14, 0.0, 0.06),
    (1, 0.14, 0.0, 0.06),
    (2, 0.10, 0.0, 0.06),
    (2, 0.08, 0.0, 0.06),
    (1, 0.10, 0.0, 0.06),
    (2, 0.14, 0.0, 0.0),
)


def main():
    scene = rig.configure(bpy, device="GPU")
    body = openpbr.build_material(bpy, "smoke:road:body", {
        "base_color": palette.rgba(palette.material_name("metal", "natural", "body")),
        "family": "metal",
        **openpbr.family_surface("metal"),
    })
    accent = openpbr.build_material(bpy, "smoke:road:accent", {
        "base_color": palette.rgba(palette.material_name("glass", "natural", "body")),
        "family": "glass",
        **openpbr.family_surface("glass"),
    })
    low, high = families.fill_envelope("road", 0.0)
    print(f"\nroad at λ = 0; its own reference measures 0.2825, envelope {low}-{high}")
    print(f"{'lanes':>5} {'lane_w':>7} {'corner':>7} {'curb':>5}  {'fill':>6} judged "
          f"{'pieces':>6} {'voids':>5}")
    for lanes, lane_width, corner, curb in CASES:
        vector = families.vector("road", 0.0)
        vector.update(lanes=lanes, lane_width=lane_width, corner_rounding=corner,
                      curb_width=curb)
        obj = shapes.road_body(bpy, f"width probe {lanes}/{lane_width}", body, vector)
        corners = [obj.matrix_world @ mathutils.Vector(corner_v)
                   for corner_v in obj.bound_box]
        bounds = (min(c[0] for c in corners), min(c[2] for c in corners),
                  max(c[0] for c in corners), max(c[2] for c in corners))
        slot_x, slot_z = families.accent_slot(bounds)
        radius = families.COMPOSITION["accent_radius"]
        accent_obj = shapes.cylinder(
            bpy, "width probe accent", accent, slot_x, slot_z, radius, depth=0.44,
            y=0.30)
        disc = generate.raster_disc(slot_x, slot_z, radius, generate.DECISION_PX)
        pixels = generate.render_to(bpy, scene, os.path.join(
            generate.REVIEW, "_smoke.road_width.render.png"))
        small = rig.downsample(pixels, rig.RENDER_PX, generate.DECISION_PX)
        found = generate.measure(small, generate.DECISION_PX, exclude=[disc])
        topo = generate.topography(small, generate.DECISION_PX, exclude=[disc])
        judged = "in" if low <= found["fill"] <= high else "OUT"
        print(f"{lanes:5} {lane_width:7.2f} {corner:7.2f} {curb:5.2f}  "
              f"{found['fill']:6.3f} {judged:>3} {topo['pieces']:6} {topo['holes']:5}")
        for item in (obj, accent_obj):
            bpy.data.objects.remove(item, do_unlink=True)


main()
