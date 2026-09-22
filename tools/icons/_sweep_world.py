"""Throwaway: how much world radiance does each family's body need?

Run under Blender, then deleted. It answers one question -- what a grey
environment at radiance `v` does to a flat swatch of each material -- so the
world can be chosen from measurement instead of taste.

    blender --background --factory-startup --python tools/icons/_sweep_world.py
"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import bpy

import generate
import palette
import rig
import shapes

FAMILIES = ["metal", "enamel", "paper", "polymer", "ceramic", "glass", "road"]
FAMILY_TOKEN = {
    "metal": "MetalBody",
    "enamel": "IconBlue",
    "paper": "PaperBody",
    "polymer": "PolymerGrip",
    "ceramic": "CeramicInsulator",
    "glass": "GlassLens",
    "road": "RoadSurface",
}
LEVELS = [0.0, 0.05, 0.10, 0.15, 0.20, 0.30, 0.40, 0.60]

PANEL = palette.host_luminance("Panel")
RAISED = palette.host_luminance("PanelRaised")
FLOOR = palette.NON_TEXT_MIN_CONTRAST


def set_world(scene, value):
    for node in scene.world.node_tree.nodes:
        if node.type == "BACKGROUND":
            node.inputs["Color"].default_value = (value, value, value, 1.0)
            node.inputs["Strength"].default_value = 1.0


def recolour(material, linear_rgb):
    for node in material.node_tree.nodes:
        if node.type == "BSDF_PRINCIPLED":
            node.inputs["Base Color"].default_value = (*linear_rgb, 1.0)


def main():
    scene = rig.configure(bpy, device="GPU")
    pixel_count = generate.DECISION_PX * generate.DECISION_PX
    print()
    print(f"Panel luminance {PANEL:.5f} / PanelRaised {RAISED:.5f}; "
          f"floor {FLOOR}:1")
    print(f"needs p75 >= {FLOOR * (PANEL + 0.05) - 0.05:.4f} on Panel, "
          f"{FLOOR * (RAISED + 0.05) - 0.05:.4f} on PanelRaised")
    print()

    header = "family   " + "".join(f"  v={v:<5.2f}" for v in LEVELS)
    print(header)
    print("         " + "".join(f"{'p75 / ctr':>9}" for _ in LEVELS))

    for family in FAMILIES:
        cells = []
        for value in LEVELS:
            set_world(scene, value)
            material = rig.reference_material(family)
            if material is None:
                cells.append("  n/a    ")
                continue
            material = material.copy()
            recolour(material, palette.linear(FAMILY_TOKEN[family]))
            objects = shapes.swatch_geometry(bpy, material)
            pixels = generate.render_to(
                bpy, scene, os.path.join(generate.REVIEW, "_sweep.render.png"))
            small = rig.downsample(pixels, rig.RENDER_PX, generate.DECISION_PX)
            measured = generate.measure(small, generate.DECISION_PX)
            for obj in objects:
                bpy.data.objects.remove(obj, do_unlink=True)
            bpy.data.materials.remove(material)
            p75 = measured["p75_luminance"]
            ratio = palette.contrast_ratio(p75, PANEL)
            clipped = sum(1 for i in range(0, len(small), 4)
                          if small[i] / 255.0 > 0.98) / pixel_count
            cells.append(f"{p75:.3f}/{ratio:.2f}")
            if clipped > 0.02:
                cells[-1] += "!"
        print(f"{family:<9}" + "".join(f"{c:>9}" for c in cells))

    print()
    print("! = more than 2% of the mark's pixels clipped above 0.98")
    set_world(scene, 0.0)
    for material in list(bpy.data.materials):
        if material.name.startswith("icon_ref:material:"):
            bpy.data.materials.remove(material)


main()
