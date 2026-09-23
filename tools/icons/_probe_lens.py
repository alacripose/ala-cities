"""Probe: what the lens's rim stack and handle actually measure, at each end.

Written because a clamp I added to make the handle protrude changed nothing in the
render, and guessing a third time is not a method. This prints the numbers the recipe
recorded on the object plus the world-space bounds, so the next change is aimed.
"""
import os
import sys

import bpy

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import families  # noqa: E402
import palette  # noqa: E402
import openpbr  # noqa: E402
import shapes  # noqa: E402


def material(family, hue):
    params = {
        "base_color": palette.rgba(palette.material_name(family, hue, "body")),
        "family": family,
        **openpbr.family_surface(family, None),
    }
    return openpbr.build_material(bpy, f"probe:{family}:{hue}", params)


# A bisect rather than a guess: build λ = 0.5 with one feature at a time turned off, so
# the step that destroys the housing is the one that is named.
CASES = [
    ("angle 39.00 (declared)", {}),
    ("angle 39.01", {"handle_angle": 39.01}),
    ("angle 38.50", {"handle_angle": 38.50}),
    ("angle 39.50", {"handle_angle": 39.50}),
    ("angle 40.00", {"handle_angle": 40.00}),
    ("handle wider", {"handle_length": 1.29}),
]
plain_vector = families.vector
for label, overrides in CASES:
    def patched(family, lam, _overrides=overrides, _plain=plain_vector):
        vector = dict(_plain(family, lam))
        vector.update(_overrides)
        return vector

    families.vector = patched
    for lam in (0.5,):
        for obj in list(bpy.data.objects):
            bpy.data.objects.remove(obj, do_unlink=True)
        built = shapes.build_family(bpy, "lens", lam, material("glass", "cyan"),
                                    "probe-lens")
        verts = [o.matrix_world @ v.co for o in built["parts"] for v in o.data.vertices]
        if verts:
            print(f"    {label}: x [{min(v.x for v in verts):.4f}, "
                  f"{max(v.x for v in verts):.4f}] z "
                  f"[{min(v.z for v in verts):.4f}, {max(v.z for v in verts):.4f}] "
                  f"verts {len(verts)}")
families.vector = plain_vector

for lam in ():
    for obj in list(bpy.data.objects):
        bpy.data.objects.remove(obj, do_unlink=True)
    built = shapes.build_family(bpy, "lens", lam, material("glass", "cyan"), "probe-lens")
    vector = built["vector"]
    objs = built["parts"]
    # λ goes on every line: without it, interleaved output makes a reader guess which
    # sample a bounds line belongs to, which is how a wrong number gets believed.
    print(f"    lam={lam}: fit {built['fit']}")
    print(f"    lam={lam}: bounds {[round(value, 4) for value in built['bounds']]}"
          f" slot {[round(value, 4) for value in built['slot']]}"
          f" accent radius {built['radius']:.4f}")
    print(f"--- λ={lam}")
    print(f"    vector: rings={vector['rings']} ring_ratio={vector['ring_ratio']} "
          f"bore_ratio={vector['bore_ratio']} ring_thickness={vector['ring_thickness']} "
          f"handle_length={vector['handle_length']} collar={vector['collar_width']}")
    for obj in objs:
        note = obj.get("handle_stack")
        if note:
            print(f"    {obj.name}: {note}")
    import math
    # The truth by vertex, because `bound_box` and the mesh can disagree: a modifier that
    # is baked but not re-evaluated leaves a box behind that no vertex is inside, and the
    # accent slot is derived from that box.
    for obj in objs:
        verts = [obj.matrix_world @ vertex.co for vertex in obj.data.vertices]
        if verts:
            print(f"    lam={lam}: {obj.name} verts x [{min(v.x for v in verts):.4f}, "
                  f"{max(v.x for v in verts):.4f}] z "
                  f"[{min(v.z for v in verts):.4f}, {max(v.z for v in verts):.4f}] "
                  f"modifiers {[m.type for m in obj.modifiers]} "
                  f"loc {tuple(round(c, 3) for c in obj.location)} "
                  f"scale {tuple(round(c, 3) for c in obj.scale)}")

    for obj in objs:
        radii = []
        for vertex in obj.data.vertices:
            point = obj.matrix_world @ vertex.co
            radii.append((math.hypot(point.x, point.z),
                          math.degrees(math.atan2(point.z, point.x))))
        if radii:
            top = max(radii)
            angle = math.radians(vector["handle_angle"])
            spoke = [r for r, a in radii
                     if abs(((a - math.degrees(angle) + 180) % 360) - 180) < 12]
            print(f"    {obj.name}: max reach {top[0]:.4f} at {top[1]:.1f} deg · "
                  f"along the handle spoke {max(spoke) if spoke else 0:.4f}")
