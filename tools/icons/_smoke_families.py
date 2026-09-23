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
import mathutils

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


#: The smoke test builds through `shapes.build_family` — the same entry point the
#: review pipeline uses — rather than through geometry calls of its own. Two callers
#: deriving the accent's slot separately is exactly how the harness passed while the
#: pipeline's first family run refused the overlap that its own stale bounds had
#: created.


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


def gate_self_test():
    """Feed the gate the four things it exists to refuse, and one clean candidate.

    A gate that passes everything is not evidence, so it is tested against its own
    refusals rather than only against the family set it happens to approve of. The
    inputs are crafted measurements, not renders: what is under test is the judgement,
    not the renderer.
    """
    cases = (
        ("an undeclared component", dict(
            family="gear", lam=0.0, fill=0.479,
            topography={"pieces": 4, "hole_shares": [0.2]},
            sites=[(1803, 47, 35), (5, 92, 86), (4, 88, 94), (1, 83, 94)])),
        ("a one-pixel sliver", dict(
            family="gear", lam=0.0, fill=0.479,
            topography={"pieces": 1, "hole_shares": [0.0004]},
            void_sites=[(1, 61, 34)])),
        ("interpenetration", dict(
            family="gear", lam=0.0, fill=0.479,
            topography={"pieces": 1, "hole_shares": []},
            parts=[("body", "accent", 0.02)])),
        ("a fill outside the envelope", dict(
            family="gear", lam=0.0, fill=0.99,
            topography={"pieces": 1, "hole_shares": []})),
    )
    print("\ngate self-test — every one of these has to be refused:")
    failures = 0
    for label, arguments in cases:
        verdict = generate.geometry_gate(**arguments)
        refused = verdict["verdict"] == "refuse"
        failures += 0 if refused else 1
        print(f"  {'refused' if refused else 'PASSED (should have refused)'}: {label}")
        if refused:
            print(f"      {verdict['notes'][0]}")
    clean = generate.geometry_gate(
        family="bin", lam=0.5, fill=0.757,
        topography={"pieces": 2, "hole_shares": []})
    if clean["verdict"] != "pass":
        failures += 1
        print(f"  refused a clean candidate: {clean['notes']}")
    print(f"  {'ok' if not failures else 'FAILED'}: {len(cases)} refusals, 1 pass, "
          f"{failures} unexpected")


def main():
    scene = rig.configure(bpy, device="GPU")
    # One body material and one for the accent, so the renders are readable rather
    # than flat: the smoke test is about silhouette and topology, not surface. They are
    # handed to `shapes.build_family` under the roles the pipeline's own materials use.
    materials = {
        "silhouette": material(bpy, "metal", "natural"),
        "accent": material(bpy, "glass", "natural"),
    }

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
            built = shapes.build_family(bpy, family, lam, materials, f"smoke {family} {lam}")
            objects = built["parts"] + [built["accent"]]
            bounds = built["bounds"]
            # The span budget (a200) is a claim about the **frame**, not a hope: an
            # object plus its accent piece has to fit the square the camera sees, and
            # a posed family can grow without any single parameter looking wrong. The
            # run says so rather than leaving it to be noticed in a render.
            frame_half = families.COMPOSITION["frame_span"] * 0.5
            reach = max(abs(bounds[0]), abs(bounds[2]), abs(bounds[1]), abs(bounds[3]))
            slot_x, slot_z = built["slot"]
            accent_edge = max(abs(slot_x), abs(slot_z)) + built["radius"]
            if reach > frame_half or accent_edge > frame_half:
                print(f"    ! frame half {frame_half:.2f}: body reaches {reach:.2f}, "
                      f"accent edge {accent_edge:.2f}")
            # The accent's pixels are known rather than guessed: the camera is
            # orthographic, so its disc maps to a raster disc. A run prints its area
            # beside the accent's own so a wrong coordinate reads as a broken exclusion
            # instead of as a cleaner render.
            radius = built["radius"]
            disc = generate.raster_disc(slot_x, slot_z, radius, generate.DECISION_PX)
            scale = generate.DECISION_PX / rig.ORTHO_SCALE
            path = os.path.join(generate.REVIEW, f"_smoke.{family}.{lam}.render.png")
            found, topo, small = render_and_read(scene, path, exclude=[disc])
            # Anything under the 3.5 % rule's floor is a sliver, and a sliver is only
            # fixable if the run says *where* it is (a196).
            voids = generate.void_sites(small, generate.DECISION_PX, exclude=[disc])
            # The gate the pipeline itself runs (a183-a204), called here rather than
            # reimplemented: a smoke test that judges by its own rules proves the
            # harness works, not the gate.
            boxes = []
            for part in objects:
                corners = [part.matrix_world @ mathutils.Vector(corner)
                           for corner in part.bound_box]
                boxes.append((part.name,
                              (min(c[0] for c in corners), min(c[1] for c in corners),
                               min(c[2] for c in corners), max(c[0] for c in corners),
                               max(c[1] for c in corners), max(c[2] for c in corners))))
            gate = generate.geometry_gate(
                family, lam, found["fill"], topo,
                sites=topo.get("piece_sites") or [], void_sites=voids,
                parts=generate.interference(boxes),
                excluded_px=found["excluded_px"])
            for note in gate["notes"]:
                print(f"    ! {note}")
            counter = counters[0]["name"] if counters else declaration["parameters"][0]["name"]
            counts = (families.features(vector[counter])
                      if counters else round(vector[counter], 2))
            shares = topo.get("hole_shares") or []
            printed = " ".join(f"{share * 100:5.2f}%" for share in shares[:4]) or "none"
            low, high = gate["fill_envelope"]
            judged = "in" if gate["verdict"] == "pass" else "OUT"
            # The body alone is what a202 judges, and the accent's own pixels are out
            # of the reading, so these are the *body's* pieces and holes.
            print(f"{family:8} {lam:6.2f}  {found['coverage']:6.3f} "
                  f"{found['fill']:6.3f} {judged:>3} ({low:.2f}-{high:.2f}) "
                  f"{topo.get('pieces'):5} {topo.get('holes'):5}  {printed:32} "
                  f"excl {found['excluded_px']:5} of ~{3.14159 * (radius * scale) ** 2:6.0f}  "
                  f"{counter}={vector[counter]:.2f} -> {counts}")
            for item in objects:
                bpy.data.objects.remove(item, do_unlink=True)
    gate_self_test()
    print("\nThe rendered frame is measured with the pipeline's own functions, so a\n"
          "number here means the same thing it will mean in the review set.")


main()
