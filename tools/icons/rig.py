"""The render rig, fixed and recorded.

A three-light rig, a grey environment, an orthographic camera and a transparent
film, all constants kept here so the manifest can name them. The point of fixing
the rig is that the material becomes the only variable: with known lights, the
rendered luminance is a *reading* of the declared base colour rather than a look
somebody tuned.

The world is **not** black. It was, and the consequence was structural rather
than dim: a mirror in a black room has nothing to reflect, so the metal family
rendered at 0.075-0.088x its albedo no matter what the lights did -- brightening
a key moves a highlight, never a body. See `WORLD` for the measurement that
chose the radiance.

The camera is orthographic because an icon is a symbol: a perspective projection
would foreshorten the parts nearest the camera and make the smallest shipped size
(the one that matters) harder to read for no gain.
"""

import hashlib
import math
import os

# Twice the largest shipped size, and every shipped size divides it by a whole
# number: 192 -> 96 averages 4 pixels, 192 -> 64 averages 9, 192 -> 48 averages 16,
# 192 -> 32 averages 36, 192 -> 24 averages 64. Exact integer factors, so the
# downsample is a plain box mean and no filter has to be invented or trusted — and
# the interface scale (100/125/150/200 %) is why the ladder reaches 96 at all: a
# 48 px icon drawn in a 96 px tile would be upscaled, which is exactly the softness
# this pipeline exists to avoid.
RENDER_PX = 192
SHIPPED_PX = (96, 64, 48, 32, 24)
# Finalized targets are generated only after a human selection. The 1536 px
# master downscales by exact integer factors to every requested density.
FINAL_PX = (96, 192, 384, 768, 1536)
FINAL_MASTER_PX = 1536
SAMPLES = 256
ORTHO_SCALE = 2.25
REFERENCE_BLEND = "assets/icons/reference.blend"
_REFERENCE_MATERIALS = {}
_REFERENCE_LIGHTS = {}
_REFERENCE_WORLD = {}
_REFERENCE_INFO = {"source": "python-defaults", "path": REFERENCE_BLEND}

#: The three lights. Azimuth is degrees around the vertical, elevation degrees
#: above the camera's horizon, both measured from the icon's own centre.
#:
#: These are the **fallback** values. `reference.blend` is the rig, and its lights
#: win whenever it has them (it does): the run that made every pixel in the
#: manifest used key 500 W / fill 100 W / rim 4000 W, not the numbers below. An
#: earlier revision of this comment claimed the energies here had been raised 2.5x
#: so that a flat swatch would clear 0.20x its albedo; that was false in both
#: directions -- nothing here is what rendered, and the blend's key is 7x *below*
#: this value, not above it. The divergence is not silent: `source_hashes()`
#: records both files and `divergence()` reports what moved.
#:
#: The rig is fixed rather than tuned, which is why a number that no longer
#: describes the render belongs in this comment as a correction rather than left
#: standing as a claim.
RIG = {
    "key": {
        "azimuth": 38.0,
        "elevation": 42.0,
        "distance": 6.0,
        "energy": 3600.0,
        "size": 3.0,
        "color": (1.0, 0.97, 0.93),
        "role": "the light the shading model is checked against",
    },
    "fill": {
        "azimuth": -62.0,
        "elevation": 14.0,
        "distance": 6.0,
        "energy": 2500.0,
        "size": 4.5,
        "color": (0.86, 0.91, 1.0),
        "role": "lifts the shadow side without flattening the form",
    },
    "rim": {
        "azimuth": 165.0,
        "elevation": 58.0,
        "distance": 6.0,
        "energy": 2200.0,
        "size": 2.0,
        "color": (1.0, 1.0, 1.0),
        "role": "separates the silhouette from the surface behind it",
    },
}


#: The world: a grey environment, declared rather than black, and **measured**.
#:
#: With `film_transparent` and no environment a mirror has nothing to reflect, so
#: the metal family's flat swatch rendered at 0.075-0.088x its own albedo whatever
#: the lights did. That is not a dim rig to be turned up: a key light brightens the
#: highlight it appears in, never the body that reflects a black room. The sweep
#: below is a flat swatch of each family at the decision size, grey world at
#: strength 1.0, read back through the same `measure()` the checks use:
#:
#:   radiance     0.00   0.05   0.10   0.15   0.20   0.30   0.40   0.60
#:   metal        0.030  0.053  0.076  0.099  0.123  0.169  0.216  0.307
#:   paper        0.569  0.597  0.625  0.654  0.683  0.739  0.794  0.892
#:   ceramic      0.325  0.356  0.386  0.418  0.449  0.509  0.571  0.694
#:   glass        0.096  0.115  0.134  0.153  0.172  0.210  0.249  0.326
#:   polymer      0.034  0.036  0.039  0.041  0.043  0.048  0.052  0.061
#:   road         0.039  0.043  0.046  0.049  0.052  0.059  0.066  0.079
#:
#: The floor is set by the *brighter* of the two hosts every icon declares, not
#: the darker: PanelRaised has luminance 0.0298 against Panel's 0.0156, so the ink
#: needs p75 >= 0.1894 there. Metal crosses that between 0.30 and 0.40, and 0.40
#: clears it at 3.33:1 with Panel at 4.06:1. The cost is recorded, not hidden:
#: by 0.40 paper is at 0.794 and begins to clip, which is why `clip_fraction` is a
#: check rather than an observation -- a paper-led icon has to choose a lower level
#: instead of relying on the world.
WORLD = {
    "color": (1.0, 1.0, 1.0),
    "strength": 0.40,
    "role": (
        "grey environment; a mirror needs something to reflect, and the floor is "
        "set by the brighter host so the radiance is set by metal on PanelRaised"
    ),
}


def world_settings() -> dict:
    """The world as it will actually be applied, and where that came from.

    The blend is the rig, so a declared `icon_ref:world` wins and the constant
    above is the fallback -- the same precedence the lights already have.
    """
    if _REFERENCE_WORLD:
        return {
            "color_linear_rgb": list(_REFERENCE_WORLD["color_linear_rgb"]),
            "strength": _REFERENCE_WORLD["strength"],
            "source": f"reference-blend ({_REFERENCE_WORLD['name']})",
            "role": WORLD["role"],
        }
    return {
        "color_linear_rgb": list(WORLD["color"]),
        "strength": WORLD["strength"],
        "source": "python-default",
        "role": WORLD["role"],
    }


def light_position(spec: dict):
    """Where a light sits, from its own recorded angles."""
    azimuth = math.radians(spec["azimuth"])
    elevation = math.radians(spec["elevation"])
    distance = spec["distance"]
    return (
        math.sin(azimuth) * math.cos(elevation) * distance,
        -math.cos(azimuth) * math.cos(elevation) * distance,
        math.sin(elevation) * distance,
    )


def reference_path():
    return os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", REFERENCE_BLEND))


def load_reference(bpy):
    """Load light/material settings from the shared blend, never icon geometry."""
    global _REFERENCE_MATERIALS, _REFERENCE_LIGHTS, _REFERENCE_INFO, _REFERENCE_WORLD
    path = reference_path()
    _REFERENCE_MATERIALS = {}
    _REFERENCE_LIGHTS = {}
    _REFERENCE_WORLD = {}
    _REFERENCE_INFO = {"source": "python-defaults", "path": REFERENCE_BLEND}
    if not os.path.exists(path):
        return

    with bpy.data.libraries.load(path, link=False) as (data_from, data_to):
        data_to.objects = [name for name in data_from.objects if name.startswith("icon_ref:light:")]
        data_to.materials = [name for name in data_from.materials if name.startswith("icon_ref:material:")]
        # The world is part of the rig, so it is read from the blend alongside the
        # lights. Without it there is a silent split: the blend would own the
        # lights and this file would own the environment.
        data_to.worlds = [name for name in data_from.worlds if name.startswith("icon_ref:world")]

    for obj in data_to.objects:
        if obj is None or obj.type != "LIGHT":
            continue
        light_name = obj.name.removeprefix("icon_ref:light:")
        target = bpy.data.objects.get(f"icon {light_name}")
        if target is None:
            continue
        # Copy world-space placement and the complete light datablock. Reading
        # only local location/rotation loses parent/constraint-relative edits and
        # silently drops editable Area-light fields such as shape and spread.
        target.parent = None
        target.location = obj.location.copy()
        target.rotation_mode = obj.rotation_mode
        if target.rotation_mode == "QUATERNION":
            target.rotation_quaternion = obj.rotation_quaternion.copy()
        elif target.rotation_mode == "AXIS_ANGLE":
            target.rotation_axis_angle = obj.rotation_axis_angle[:]
        else:
            target.rotation_euler = obj.rotation_euler.copy()
        target.scale = obj.scale.copy()
        old_data = target.data
        target.data = obj.data.copy()
        if old_data.users == 0:
            bpy.data.lights.remove(old_data)
        _REFERENCE_LIGHTS[light_name] = {
            "type": target.data.type,
            "position": [round(value, 6) for value in target.location],
            "rotation_euler": [round(value, 6) for value in target.rotation_euler],
            "energy": target.data.energy,
            "size": target.data.size,
            "color": [round(value, 6) for value in target.data.color],
            "shape": getattr(target.data, "shape", None),
            "size_x": getattr(target.data, "size", None),
            "size_y": getattr(target.data, "size_y", None),
            "spread": getattr(target.data, "spread", None),
        }
        bpy.data.objects.remove(obj, do_unlink=True)

    for material in data_to.materials:
        if material is None:
            continue
        role = material.name.removeprefix("icon_ref:material:")
        _REFERENCE_MATERIALS[role] = material

    for world in data_to.worlds:
        if world is None:
            continue
        color = None
        strength = None
        if world.use_nodes and world.node_tree is not None:
            for node in world.node_tree.nodes:
                if node.type == "BACKGROUND":
                    color = [round(value, 6) for value in node.inputs["Color"].default_value[:3]]
                    strength = node.inputs["Strength"].default_value
                    break
        if color is None:
            color = [round(value, 6) for value in world.color[:3]]
            strength = 1.0
        _REFERENCE_WORLD = {
            "name": world.name,
            "color_linear_rgb": color,
            "strength": strength,
        }
        break

    _REFERENCE_INFO = {
        "source": "reference-blend",
        "path": REFERENCE_BLEND,
        "absolute_path": path,
        "materials": sorted(_REFERENCE_MATERIALS),
        "world_declared": bool(_REFERENCE_WORLD),
    }


def reference_material(role):
    return _REFERENCE_MATERIALS.get(role)


def reference_roles():
    """Which families the blend actually carries a reference material for.

    `openpbr.family_coverage()` compares the declared surface table against the
    declared colour matrix, and both of those are code. Neither notices that the
    blend is missing a family, which is how `enamel` came to fall back to the
    dict silently: its surface was the fallback, not the reference. This is what
    lets that be reported instead.
    """
    return sorted(_REFERENCE_MATERIALS)


def reference_info():
    info = dict(_REFERENCE_INFO)
    if _REFERENCE_LIGHTS:
        info["lights"] = {
            name: dict(values) for name, values in sorted(_REFERENCE_LIGHTS.items())
        }
    return info


def configure(bpy, device="GPU", render_px=RENDER_PX, load_reference_settings=True):
    """A factory-clean scene with the rig's render settings applied.

    `view_transform = 'Standard'` is the load-bearing setting: with a filmic or
    AgX transform the saved pixels would no longer be an encoding of the rendered
    radiance, and the material check below would be measuring a look instead of a
    surface. 'Standard' keeps the pipeline honest and is recorded in the manifest.
    """
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = bpy.context.scene

    scene.render.engine = "CYCLES"
    scene.cycles.samples = SAMPLES
    # No denoiser: it is a post-process, and a post-process is a thing the
    # manifest would then have to call out. 256 samples on flat geometry is
    # clean enough at 192 px without one.
    scene.cycles.use_denoising = False
    scene.cycles.device = device

    scene.render.resolution_x = render_px
    scene.render.resolution_y = render_px
    scene.render.resolution_percentage = 100
    scene.render.film_transparent = True
    scene.render.image_settings.file_format = "PNG"
    scene.render.image_settings.color_mode = "RGBA"
    scene.render.image_settings.color_depth = "8"
    scene.view_settings.view_transform = "Standard"
    scene.view_settings.look = "None"
    scene.view_settings.exposure = 0.0
    scene.view_settings.gamma = 1.0

    # The world is the rig's ambient half and is applied from the recorded
    # settings, not forced black. It used to be forced black here, which was the
    # load-bearing half of the metal problem: see `WORLD` for the measurement.
    world = world_settings()
    if scene.world is None:
        scene.world = bpy.data.worlds.new("icon world")
    scene.world.use_nodes = True
    has_background = False
    for node in scene.world.node_tree.nodes:
        if node.type == "BACKGROUND":
            node.inputs["Color"].default_value = (*world["color_linear_rgb"], 1.0)
            node.inputs["Strength"].default_value = world["strength"]
            has_background = True
    if not has_background:
        raise RuntimeError(
            "the icon world has no Background node, so the declared environment "
            f"({world['source']}) cannot be applied and the render would be "
            "unlit by it without saying so"
        )

    camera_data = bpy.data.cameras.new("icon camera")
    camera_data.type = "ORTHO"
    camera_data.ortho_scale = ORTHO_SCALE
    camera = bpy.data.objects.new("icon camera", camera_data)
    camera.location = (0.0, -8.0, 0.0)
    camera.rotation_euler = (math.radians(90.0), 0.0, 0.0)
    bpy.context.collection.objects.link(camera)
    scene.camera = camera

    for name, spec in RIG.items():
        light_data = bpy.data.lights.new(f"icon {name}", type="AREA")
        light_data.energy = spec["energy"]
        light_data.size = spec["size"]
        light_data.color = spec["color"]
        light = bpy.data.objects.new(f"icon {name}", light_data)
        position = light_position(spec)
        light.location = position
        # Aim at the icon's centre.
        direction = (-position[0], -position[1], -position[2])
        length = math.sqrt(sum(component ** 2 for component in direction)) or 1.0
        light.rotation_euler = _look_along([component / length for component in direction])
        bpy.context.collection.objects.link(light)

    if load_reference_settings:
        load_reference(bpy)
    scene["icon_reference_blend"] = REFERENCE_BLEND
    scene["icon_reference_source"] = _REFERENCE_INFO["source"] if load_reference_settings else "reference-builder"
    return scene


def _look_along(direction):
    """Euler angles that point an object's -Z along `direction` (Blender's convention)."""
    import mathutils  # noqa: F401  (Blender's own; present inside the renderer)

    x, y, z = direction
    return (math.atan2(math.sqrt(x * x + y * y), -z), 0.0, math.atan2(y, x))


def rig_record(render_px=RENDER_PX) -> dict:
    """The rig as the manifest states it."""
    return {
        "reference_blend": REFERENCE_BLEND,
        "reference_source": _REFERENCE_INFO["source"],
        "camera": {
            "type": "orthographic",
            "ortho_scale": ORTHO_SCALE,
            "position": [0.0, -8.0, 0.0],
            "looks_along": "+Y",
            "why": "an icon is a symbol; perspective would foreshorten the parts nearest the camera",
        },
        "lights": (
            {
                name: {
                    **values,
                    "role": RIG.get(name, {}).get("role", "reference blend light"),
                }
                for name, values in sorted(_REFERENCE_LIGHTS.items())
            }
            if _REFERENCE_LIGHTS
            else {
                name: {
                    "azimuth_degrees": spec["azimuth"],
                    "elevation_degrees": spec["elevation"],
                    "distance": spec["distance"],
                    "energy_watts": spec["energy"],
                    "size": spec["size"],
                    "color_linear_rgb": list(spec["color"]),
                    "role": spec["role"],
                }
                for name, spec in RIG.items()
            }
        ),
        "world": world_settings(),
        "film": "transparent",
        "view_transform": "Standard (no filmic or AgX transform, so the saved pixels encode radiance)",
        "denoising": "off; a denoiser is a post-process and would have to be declared",
        "samples": SAMPLES,
        "render_px": render_px,
        "shipped_px": list(SHIPPED_PX),
        "finalized_px": list(FINAL_PX),
        "final_master_px": FINAL_MASTER_PX,
        "downsample": (
            "box average in linear light over exact integer factors "
            "(192->96 averages 4 px, 192->64 averages 9, 192->48 averages 16, "
            "192->32 averages 36, 192->24 averages 64)"
        ),
    }


# ---------------------------------------------------------------------------
# Pixels
# ---------------------------------------------------------------------------


def linearise(encoded: float) -> float:
    """sRGB-encoded channel to linear light."""
    if encoded <= 0.04045:
        return encoded / 12.92
    return ((encoded + 0.055) / 1.055) ** 2.4


def encode(linear: float) -> float:
    """Linear light back to sRGB-encoded."""
    if linear <= 0.0031308:
        return linear * 12.92
    return 1.055 * (linear ** (1 / 2.4)) - 0.055


def downsample(pixels, source_px: int, target_px: int, channels: int = 4):
    """Average in linear light, exactly.

    Averaging the encoded values instead would darken edges — the classic
    "my downscaled icons look muddy" bug — so the mean is taken on linear
    radiance and re-encoded afterwards.
    """
    if source_px % target_px != 0:
        raise ValueError(
            f"{source_px} does not divide by {target_px}; the downsample is a box mean "
            f"only at whole-number factors, and a resampling filter is something the "
            f"manifest would have to name"
        )
    factor = source_px // target_px
    out = bytearray(target_px * target_px * channels)
    for ty in range(target_px):
        for tx in range(target_px):
            for channel in range(channels):
                total = 0.0
                for sy in range(factor):
                    for sx in range(factor):
                        index = ((ty * factor + sy) * source_px + (tx * factor + sx)) * channels
                        total += linearise(pixels[index + channel] / 255.0)
                value = total / (factor * factor)
                out[(ty * target_px + tx) * channels + channel] = round(
                    max(0.0, min(1.0, encode(value))) * 255
                )
    return bytes(out)


def hash_bytes(data: bytes) -> str:
    """The digest the runtime also computes.

    FNV-1a 64-bit, which the Rust side implements in a few lines. It is an
    **integrity** check — "this file is the file the manifest describes" — and
    not a security mechanism, and it is named that way in the manifest rather
    than being called a signature.
    """
    value = 0xCBF29CE484222325
    for byte in data:
        value ^= byte
        value = (value * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f"fnv1a64:{value:016x}"


def read_png_non_color(bpy, path: str):
    """Decode a saved PNG back to its stored 8-bit values, without conversion."""
    image = bpy.data.images.load(path, check_existing=False)
    image.colorspace_settings.name = "Non-Color"
    pixels = list(image.pixels)
    bpy.data.images.remove(image)
    return pixels
