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

#: Every family, so the smoke covers the whole table rather than the two that were
#: built first. The icon beside each name is the icon the family belongs to.
SMOKE = (
    ("gear", "vocab-settings"),
    ("road", "tool-road"),
    ("bolt", "tool-power"),
    ("lens", "tool-inspect"),
    ("bin", "tool-demolish"),
    ("plaque", "ticket"),
)


def family_objects(bpy, family, vector, material, name):
    """The objects one family's body is made of, at one point on the ladder.

    Most families are a single object; the bin's own reference measures its lid as a
    separate component with a gap, so that family declares **two** parts and returns
    both. Nothing downstream reads a part count out of this list as a *claim*: the
    count that gets judged is the one the family declares.
    """
    if family == "gear":
        return [shapes.gear_body(bpy, name, material, vector)]
    if family == "road":
        return [shapes.road_body(bpy, name, material, vector)]
    if family == "bolt":
        return [shapes.bolt_body(bpy, name, material, vector)]
    if family == "lens":
        return [shapes.lens_body(bpy, name, material, vector)]
    if family == "bin":
        return list(shapes.bin_body(bpy, name, material, vector))
    if family == "plaque":
        return [shapes.plaque_body(bpy, name, material, vector)]
    raise ValueError(f"`{family}` has no geometry entry point")


def material(bpy, family, hue, finish=None):
    parameters = {
        "base_color": palette.rgba(palette.material_name(family, hue, "body")),
        "family": family,
        **openpbr.family_surface(family, finish),
    }
    return openpbr.build_material(bpy, f"smoke:{family}:{hue}", parameters)


def render_and_read(scene, path, exclude=None, px=generate.DECISION_PX):
    """Render at the rig's own resolution and read it back at the judged size.

    Exactly the sequence the review pipeline uses (`render_to` then
    `rig.downsample`), so a number printed here means what it will mean in the
    review set rather than being a second, friendlier measurement.

    `exclude` is the accent's declared disc (a202/a204): the body is judged on its
    own, because the reference mark is a bare object and a reading that averaged the
    accent in would not be the same quantity.
    """
    pixels = generate.render_to(bpy, scene, path)
    small = rig.downsample(pixels, rig.RENDER_PX, px)
    return (generate.measure(small, px, exclude=exclude),
            generate.topography(small, px, exclude=exclude, with_sites=True),
            small)


def main():
    scene = rig.configure(bpy, device="GPU")
    # One body material and one for the accent, so the renders are readable rather
    # than flat: the smoke test is about silhouette and topology, not surface.
    body = material(bpy, "metal", "natural")
    accent = material(bpy, "glass", "natural")

    print(f"\n{'family':8} {'lambda':>6}  {'cover':>6} {'fill':>6} judged          "
          f"{'envelope':14} {'parts':>5} {'voids':>5}  "
          f"{'void shares of the object':32} count")
    for family, icon in SMOKE:
        declaration = families.declaration(family)
        declared = declaration["declared_parts"]["count"]
        counters = [parameter for parameter in declaration["parameters"]
                    if parameter["type"] == "count"]
        for lam in SAMPLES:
            vector = families.vector(family, lam)
            built = family_objects(bpy, family, vector, body, f"smoke {family} {lam}")
            obj = built[0]
            # The accent's slot is **derived from the body's own bounds** (a200) and
            # sits clear by the declared clearance. Overlapping it into the body is
            # what produced the only sliver this smoke test found — a one-pixel void
            # at (68, 76), in the corner where the two touched — the same defect class
            # the committed review set carries (16 voids in one candidate), now with
            # a cause on the record and a rule that removes it.
            # World bounds, not local ones: a posed body (a203) is turned after it is
            # welded, and a slot derived from its unposed box would sit inside it.
            # (The same is true of the exclusion disc below, which is why the slot is
            # read back in world space rather than from the body's local origin.)
            import mathutils
            corners = [part.matrix_world @ mathutils.Vector(corner)
                       for part in built for corner in part.bound_box]
            bounds = (min(c[0] for c in corners), min(c[2] for c in corners),
                      max(c[0] for c in corners), max(c[2] for c in corners))
            # The span budget (a200) is a claim about the **frame**, not a hope: an
            # object plus its accent piece has to fit the square the camera sees, and
            # a posed family can grow without any single parameter looking wrong. The
            # run says so rather than leaving it to be noticed in a render.
            frame_half = families.COMPOSITION["frame_span"] * 0.5
            reach = max(abs(bounds[0]), abs(bounds[2]), abs(bounds[1]), abs(bounds[3]))
            slot_x, slot_z = families.accent_slot(bounds)
            accent_edge = max(abs(slot_x), abs(slot_z)) + \
                families.COMPOSITION["accent_radius"]
            if reach > frame_half or accent_edge > frame_half:
                print(f"    ! frame half {frame_half:.2f}: body reaches {reach:.2f}, "
                      f"accent edge {accent_edge:.2f}")
            accent_obj = shapes.cylinder(
                bpy, f"smoke accent {lam}", accent, slot_x, slot_z,
                families.COMPOSITION["accent_radius"], depth=0.44, y=0.30)
            # The accent's pixels are known rather than guessed: the camera is
            # orthographic, so its disc maps to a raster disc. A run prints its area
            # beside the accent's own so a wrong coordinate reads as a broken exclusion
            # instead of as a cleaner render.
            radius = families.COMPOSITION["accent_radius"]
            disc = generate.raster_disc(slot_x, slot_z, radius, generate.DECISION_PX)
            scale = generate.DECISION_PX / rig.ORTHO_SCALE
            path = os.path.join(generate.REVIEW, f"_smoke.{family}.{lam}.render.png")
            found, topo, small = render_and_read(scene, path, exclude=[disc])
            # Anything under the 3.5 % rule's floor is a sliver, and a sliver is only
            # fixable if the run says *where* it is (a196).
            slivers = [site for site in generate.void_sites(
                           small, generate.DECISION_PX, exclude=[disc])
                       if site[0] / max(1, topo["covered_px"]) < 0.035]
            if slivers:
                print(f"    slivers at {slivers[:4]} px, top-down, of "
                      f"{topo['covered_px']} covered px")
            if topo["pieces"] != declared:
                # The object has to arrive in the number of parts its family declares
                # (a183/a184/a202), and naming where the extra ones are is the
                # difference between a number and a fix.
                print(f"    ! declared {declared} part(s), measured {topo['pieces']}: "
                      f"{topo['piece_sites'][:6]} px at (x, y) top-down")
            counter = counters[0]["name"] if counters else declaration["parameters"][0]["name"]
            counts = (families.features(vector[counter])
                      if counters else round(vector[counter], 2))
            shares = topo.get("hole_shares") or []
            printed = " ".join(f"{share * 100:5.2f}%" for share in shares[:4]) or "none"
            low, high = families.fill_envelope(family, lam)
            judged = "in" if low <= found["fill"] <= high else "OUT"
            # The body alone is what a202 judges, and the accent's own pixels are out
            # of the reading, so these are the *body's* pieces and holes.
            print(f"{family:8} {lam:6.2f}  {found['coverage']:6.3f} "
                  f"{found['fill']:6.3f} {judged:>3} ({low:.2f}-{high:.2f}) "
                  f"{topo.get('pieces'):5} {topo.get('holes'):5}  {printed:32} "
                  f"excl {found['excluded_px']:5} of ~{3.14159 * (radius * scale) ** 2:6.0f}  "
                  f"{counter}={vector[counter]:.2f} -> {counts}")
            for item in built + [accent_obj]:
                bpy.data.objects.remove(item, do_unlink=True)
    print("\nThe rendered frame is measured with the pipeline's own functions, so a\n"
          "number here means the same thing it will mean in the review set.")


main()
