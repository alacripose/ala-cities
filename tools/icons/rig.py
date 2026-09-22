"""The render rig, fixed and recorded.

A three-light rig, an orthographic camera and a transparent film, all constants
kept here so the manifest can name them. The point of fixing the rig is that the
material becomes the only variable: with known lights, the rendered luminance is
a *reading* of the declared base colour rather than a look somebody tuned.

The camera is orthographic because an icon is a symbol: a perspective projection
would foreshorten the parts nearest the camera and make the smallest shipped size
(the one that matters) harder to read for no gain.
"""

import hashlib
import math

# 4x the largest shipped size. 192 -> 48 is a box average of 16 pixels; 192 -> 32
# averages 36; 192 -> 24 averages 64. All exact integer factors, so the
# downsample is a plain mean and no filter has to be invented or trusted.
RENDER_PX = 192
SHIPPED_PX = (48, 32, 24)
SAMPLES = 256
ORTHO_SCALE = 2.25

#: The three lights. Azimuth is degrees around the vertical, elevation degrees
#: above the camera's horizon, both measured from the icon's own centre.
RIG = {
    "key": {
        "azimuth": 38.0,
        "elevation": 42.0,
        "distance": 6.0,
        "energy": 320.0,
        "size": 3.0,
        "color": (1.0, 0.97, 0.93),
        "role": "the light the shading model is checked against",
    },
    "fill": {
        "azimuth": -62.0,
        "elevation": 14.0,
        "distance": 6.0,
        "energy": 110.0,
        "size": 4.5,
        "color": (0.86, 0.91, 1.0),
        "role": "lifts the shadow side without flattening the form",
    },
    "rim": {
        "azimuth": 165.0,
        "elevation": 58.0,
        "distance": 6.0,
        "energy": 170.0,
        "size": 2.0,
        "color": (1.0, 1.0, 1.0),
        "role": "separates the silhouette from the surface behind it",
    },
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


def configure(bpy, device="GPU"):
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

    scene.render.resolution_x = RENDER_PX
    scene.render.resolution_y = RENDER_PX
    scene.render.resolution_percentage = 100
    scene.render.film_transparent = True
    scene.render.image_settings.file_format = "PNG"
    scene.render.image_settings.color_mode = "RGBA"
    scene.render.image_settings.color_depth = "8"
    scene.view_settings.view_transform = "Standard"
    scene.view_settings.look = "None"
    scene.view_settings.exposure = 0.0
    scene.view_settings.gamma = 1.0

    # A black world: the three lights in the manifest are the only light there
    # is, which is what makes their recorded energies mean something.
    if scene.world is None:
        scene.world = bpy.data.worlds.new("icon world")
    scene.world.use_nodes = True
    for node in scene.world.node_tree.nodes:
        if node.type == "BACKGROUND":
            node.inputs["Color"].default_value = (0.0, 0.0, 0.0, 1.0)
            node.inputs["Strength"].default_value = 1.0

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
    return scene


def _look_along(direction):
    """Euler angles that point an object's -Z along `direction` (Blender's convention)."""
    import mathutils  # noqa: F401  (Blender's own; present inside the renderer)

    x, y, z = direction
    return (math.atan2(math.sqrt(x * x + y * y), -z), 0.0, math.atan2(y, x))


def rig_record() -> dict:
    """The rig as the manifest states it."""
    return {
        "camera": {
            "type": "orthographic",
            "ortho_scale": ORTHO_SCALE,
            "position": [0.0, -8.0, 0.0],
            "looks_along": "+Y",
            "why": "an icon is a symbol; perspective would foreshorten the parts nearest the camera",
        },
        "lights": {
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
        },
        "world": "black; the three lights above are the only illumination",
        "film": "transparent",
        "view_transform": "Standard (no filmic or AgX transform, so the saved pixels encode radiance)",
        "denoising": "off; a denoiser is a post-process and would have to be declared",
        "samples": SAMPLES,
        "render_px": RENDER_PX,
        "shipped_px": list(SHIPPED_PX),
        "downsample": (
            "box average in linear light over exact integer factors "
            "(192->48 averages 16 px, 192->32 averages 36, 192->24 averages 64)"
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
