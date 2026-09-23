"""Render the gear and road families at the five sampled λ points (C8 a179–a194).

The point of the rework is that *intermediate* points are legitimate objects, so the
only honest smoke test is to render all five and look at the numbers: the count
changing by a feature growing from zero width (a187), one welded body per object
(a184), and no enclosed void except the declared ones (a183/a196).

Run it the way the pipeline runs anything:

    blender --background --factory-startup --python tools/icons/_smoke_families.py
"""

import os
import sys

import bpy

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import families  # noqa: E402
import generate  # noqa: E402
import openpbr  # noqa: E402
import palette  # noqa: E402
import rig  # noqa: E402
import shapes  # noqa: E402

#: The five sampled points (a194), kept in step with the review sampler.
SAMPLES = (0.0, 0.25, 0.5, 0.75, 1.0)

#: The two families asked for first: a gear and a road (the person's own examples).
SMOKE = (
    ("gear", "vocab-settings"),
    ("road", "tool-road"),
)


def material(bpy, family, hue, finish=None):
    parameters = {
        "base_color": palette.rgba(palette.material_name(family, hue, "body")),
        "family": family,
        **openpbr.family_surface(family, finish),
    }
    return openpbr.build_material(bpy, f"smoke:{family}:{hue}", parameters)


def render_and_read(scene, path, px=generate.DECISION_PX):
    """Render at the rig's own resolution and read it back at the judged size.

    Exactly the sequence the review pipeline uses (`render_to` then
    `rig.downsample`), so a number printed here means what it will mean in the
    review set rather than being a second, friendlier measurement.
    """
    pixels = generate.render_to(bpy, scene, path)
    small = rig.downsample(pixels, rig.RENDER_PX, px)
    return generate.measure(small, px), generate.topography(small, px), small


def main():
    scene = rig.configure(bpy, device="GPU")
    # One body material and one for the accent, so the renders are readable rather
    # than flat: the smoke test is about silhouette and topology, not surface.
    body = material(bpy, "metal", "natural")
    accent = material(bpy, "glass", "natural")

    print(f"\n{'family':8} {'lambda':>6}  {'cover':>6} {'fill':>6} judged          "
          f"{'envelope':14} {'pieces':>6} {'voids':>5}  "
          f"{'void shares of the object':32} count")
    for family, icon in SMOKE:
        for lam in SAMPLES:
            vector = families.vector(family, lam)
            if family == "gear":
                obj = shapes.gear_body(bpy, f"smoke gear {lam}", body, vector)
            else:
                obj = shapes.road_body(bpy, f"smoke road {lam}", body, vector)
            # The accent's slot is **derived from the body's own bounds** (a200) and
            # sits clear by the declared clearance. Overlapping it into the body is
            # what produced the only sliver this smoke test found — a one-pixel void
            # at (68, 76), in the corner where the two touched — the same defect class
            # the committed review set carries (16 voids in one candidate), now with
            # a cause on the record and a rule that removes it.
            corners = [tuple(corner) for corner in obj.bound_box]
            bounds = (min(c[0] for c in corners), min(c[2] for c in corners),
                      max(c[0] for c in corners), max(c[2] for c in corners))
            slot_x, slot_z = families.accent_slot(bounds)
            accent_obj = shapes.cylinder(
                bpy, f"smoke accent {lam}", accent, slot_x, slot_z,
                families.COMPOSITION["accent_radius"], depth=0.44, y=0.30)
            path = os.path.join(generate.REVIEW, f"_smoke.{family}.{lam}.render.png")
            found, topo, small = render_and_read(scene, path)
            # Anything under the 3.5 % rule's floor is a sliver, and a sliver is only
            # fixable if the run says *where* it is (a196).
            slivers = [site for site in generate.void_sites(small, generate.DECISION_PX)
                       if site[0] / max(1, topo["covered_px"]) < 0.035]
            if slivers:
                print(f"    slivers at {slivers[:4]} px, top-down, of "
                      f"{topo['covered_px']} covered px")
            counts = families.features(vector["teeth"] if family == "gear"
                                       else vector["lanes"])
            shares = topo.get("hole_shares") or []
            printed = " ".join(f"{share * 100:5.2f}%" for share in shares[:4]) or "none"
            low, high = families.fill_envelope(family)
            judged = "in" if low <= found["fill"] <= high else "OUT"
            print(f"{family:8} {lam:6.2f}  {found['coverage']:6.3f} "
                  f"{found['fill']:6.3f} {judged:>3} ({low:.2f}-{high:.2f}) "
                  f"{topo.get('pieces'):6} {topo.get('holes'):5}  {printed:32} "
                  f"{families.declaration(family)['parameters'][0]['name']}="
                  f"{vector[families.declaration(family)['parameters'][0]['name']]:.2f} "
                  f"-> {counts}")
            for item in (obj, accent_obj):
                bpy.data.objects.remove(item, do_unlink=True)
    print("\nThe rendered frame is measured with the pipeline's own functions, so a\n"
          "number here means the same thing it will mean in the review set.")


main()
