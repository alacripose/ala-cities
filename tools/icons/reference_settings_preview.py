"""Create a settings-gear preview copy of the shared reference blend.

The source reference.blend is opened and never saved over. The output copy keeps
its edited lights, materials, camera, and swatches, then adds one authored
canonical settings gear using the same procedural recipe as the generator.

Run with Blender:
    blender --background --factory-startup --python tools/icons/reference_settings_preview.py
"""

import os
import sys

import bpy

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
if HERE not in sys.path:
    sys.path.insert(0, HERE)

import shapes  # noqa: E402
import rig  # noqa: E402

SOURCE = os.path.join(ROOT, "assets", "icons", "reference.blend")
OUTPUT = os.path.join(ROOT, "assets", "icons", "reference-settings-preview.blend")


def main():
    if not os.path.exists(SOURCE):
        raise FileNotFoundError(SOURCE)

    bpy.ops.wm.open_mainfile(filepath=SOURCE)

    entry = shapes.ICONS[0]
    candidate = shapes.generations_of(entry)[0]
    materials = {}
    for role in shapes.COMMON_MATERIALS:
        material = bpy.data.materials.get(f"icon_ref:material:{role}")
        if material is None:
            raise RuntimeError(f"reference blend is missing material role {role!r}")
        materials[role] = material

    collection = bpy.data.collections.new("icon_ref:settings-gear")
    bpy.context.scene.collection.children.link(collection)
    objects = shapes.build(bpy, candidate, materials)

    for obj in objects:
        for old_collection in list(obj.users_collection):
            old_collection.objects.unlink(obj)
        collection.objects.link(obj)
        obj["icon_preview_only"] = True
        obj["icon_preview_candidate"] = candidate["generation"]
        obj["icon_preview_source"] = "tools/icons/shapes.py"

    scene = bpy.context.scene
    scene["icon_preview_schema"] = "icon-settings-preview-v1"
    scene["icon_preview_candidate"] = candidate["generation"]
    scene["icon_preview_meaning"] = entry["meaning"]
    scene["icon_preview_source_blend"] = "assets/icons/reference.blend"
    scene["icon_preview_geometry_source"] = "tools/icons/shapes.py"
    scene["icon_preview_note"] = "procedural canonical settings gear; source reference blend remains unchanged"

    os.makedirs(os.path.dirname(OUTPUT), exist_ok=True)
    bpy.ops.wm.save_as_mainfile(filepath=OUTPUT)
    print(f"wrote {OUTPUT}")
    print(f"candidate: {candidate['generation']}")
    print(f"objects: {len(objects)}")
    print(f"materials: {', '.join(sorted(materials))}")
    print(f"lights: {', '.join(sorted(obj.name for obj in bpy.data.objects if obj.name.startswith('icon_ref:light:')))}")


if __name__ == "__main__":
    main()
