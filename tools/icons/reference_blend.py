"""Create the shared icon authoring reference blend.

This file deliberately contains settings only: three named lights, the camera/world
defaults, and one named material for each generator role. Icon geometry remains in
`shapes.py`, so artists can tune the look without creating one .blend per icon.

Run with Blender:
    blender --background --factory-startup --python tools/icons/reference_blend.py
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
if HERE not in sys.path:
    sys.path.insert(0, HERE)

import bpy  # noqa: E402

import openpbr  # noqa: E402
import rig  # noqa: E402
import shapes  # noqa: E402


def main():
    scene = rig.configure(
        bpy,
        device="CPU",
        render_px=rig.RENDER_PX,
        load_reference_settings=False,
    )

    # The reference scene must remain settings-only. Remove the generated camera
    # and lights, then recreate them with stable artist-facing names.
    for obj in list(bpy.data.objects):
        if obj.type in {"CAMERA", "LIGHT"}:
            bpy.data.objects.remove(obj, do_unlink=True)

    camera_data = bpy.data.cameras.new("icon_ref:camera")
    camera_data.type = "ORTHO"
    camera_data.ortho_scale = rig.ORTHO_SCALE
    camera = bpy.data.objects.new("icon_ref:camera", camera_data)
    camera.location = (0.0, -8.0, 0.0)
    camera.rotation_euler = (1.5707963267948966, 0.0, 0.0)
    bpy.context.collection.objects.link(camera)
    scene.camera = camera

    for name, spec in rig.RIG.items():
        light_data = bpy.data.lights.new(f"icon_ref:light:{name}", type="AREA")
        light_data.energy = spec["energy"]
        light_data.size = spec["size"]
        light_data.color = spec["color"]
        light = bpy.data.objects.new(f"icon_ref:light:{name}", light_data)
        position = rig.light_position(spec)
        light.location = position
        direction = (-position[0], -position[1], -position[2])
        length = sum(component * component for component in direction) ** 0.5 or 1.0
        light.rotation_euler = rig._look_along([component / length for component in direction])
        light["icon_light_role"] = name
        light["icon_light_editable"] = True
        bpy.context.collection.objects.link(light)

    # Material names are the stable API between this file and shapes.py. Editing
    # Principled inputs in Blender is enough; the generator reads them back.
    for material in list(bpy.data.materials):
        if material.name.startswith("icon_ref:material:"):
            bpy.data.materials.remove(material)
    materials = {}
    for role, parameters in shapes.COMMON_MATERIALS.items():
        material = openpbr.build_material(
            bpy,
            f"icon_ref:material:{role}",
            parameters,
        )
        material["icon_material_role"] = role
        material["icon_finish"] = parameters.get("finish", "plain")
        material["icon_material_editable"] = True
        materials[role] = material

    # Give every material a visible owner in Blender. These are deliberately
    # preview swatches, not icon meshes: the generator never imports this
    # collection, but an artist can select a role, edit its Principled inputs,
    # and immediately see the result under the shared rig.
    preview_collection = bpy.data.collections.new("icon_ref:previews")
    scene.collection.children.link(preview_collection)
    for index, role in enumerate(sorted(materials)):
        column = index % 5
        row = index // 5
        bpy.ops.mesh.primitive_uv_sphere_add(
            segments=32,
            ring_count=16,
            radius=0.34,
            location=((column - 2) * 0.92, 0.0, 0.72 - row * 0.92),
        )
        preview = bpy.context.object
        preview.name = f"icon_ref:preview:{role}"
        for collection in list(preview.users_collection):
            collection.objects.unlink(preview)
        preview_collection.objects.link(preview)
        preview.data.materials.append(materials[role])
        preview["icon_preview_role"] = role
        preview["icon_preview_only"] = True

    bpy.ops.mesh.primitive_plane_add(
        size=5.2,
        location=(0.0, 0.28, -1.1),
        rotation=(1.5707963267948966, 0.0, 0.0),
    )
    preview_floor = bpy.context.object
    preview_floor.name = "icon_ref:preview-surface"
    for collection in list(preview_floor.users_collection):
        collection.objects.unlink(preview_floor)
    preview_collection.objects.link(preview_floor)
    preview_floor.data.materials.append(materials["ink"])
    preview_floor["icon_preview_only"] = True

    scene["icon_reference_schema"] = "icon-reference-v1"
    scene["icon_reference_purpose"] = "shared lights and materials only; geometry stays procedural"
    scene["icon_generator_contract"] = "tools/icons/generate.py reads icon_ref:light:* and icon_ref:material:*"
    scene["icon_material_roles"] = sorted(shapes.COMMON_MATERIALS)
    scene["icon_light_roles"] = sorted(rig.RIG)
    scene["icon_reference_study"] = shapes.REFERENCE_STUDY["pack"]

    target = rig.reference_path()
    output = target + ".new"
    os.makedirs(os.path.dirname(output), exist_ok=True)
    bpy.ops.wm.save_as_mainfile(filepath=output)
    os.replace(output, target)
    print(f"wrote {target}")
    print(f"materials: {', '.join(sorted(shapes.COMMON_MATERIALS))}")
    print(f"lights: {', '.join(sorted(rig.RIG))}")


if __name__ == "__main__":
    main()
