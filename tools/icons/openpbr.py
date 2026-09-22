"""OpenPBR parameter table mapped onto Blender's Principled BSDF v2.

The renderer is honest about the boundary: OpenPBR parameters are validated and
recorded, while the small `finish` field is explicit authoring metadata that drives
visible procedural treatment in Blender. It is never mistaken for an OpenPBR
parameter.

C3 added the family table below. A material is no longer a roughness and an IOR
picked per recipe: it is a named family, whose surface is declared once here and
whose colour comes from `palette`'s matrix. That is what makes "the icon is made of
a metal" a claim a script can check, and what stopped the previous pass from giving
five different objects five different metals by accident.
"""

import palette

MAPPING = {
    "base_color": "Base Color",
    "base_metalness": "Metallic",
    "specular_roughness": "Roughness",
    "specular_ior": "IOR",
    "geometry_coat_weight": "Coat Weight",
}
#: `finish` drives visible procedural treatment; `family` names which row of
#: FAMILY_SURFACE the material came from, so the shared reference blend can own the
#: surface while the palette matrix owns the colour. Neither is an OpenPBR
#: parameter and neither is ever presented as one.
ALLOWED = set(MAPPING)
METADATA = {"finish", "family"}
UNSUPPORTED = {
    "specular_weight": "the shader's Specular IOR Level is not OpenPBR's linear F0 weight",
    "base_weight": "the shader has one base layer",
    "coat_roughness": "the shader does not expose an independent OpenPBR coat roughness",
    "thin_film_thickness": "not implemented by the target shader",
    "thin_film_ior": "not implemented by the target shader",
    "transmission_weight": "not used by opaque icon authoring",
    "subsurface_weight": "not used by opaque icon authoring",
    "fuzz_weight": "not used by opaque icon authoring",
}
DEVIATIONS = {
    "base_color": [],
    "base_metalness": ["Principled Metallic is used at the same 0..1 weight; icon roles use stable endpoints or explicit authored values"],
    "specular_roughness": ["OpenPBR separates dielectric and metallic roughness; Principled exposes one Roughness input"],
    "specular_ior": [],
    "geometry_coat_weight": ["Principled coat roughness and IOR remain Blender defaults because this alpha exposes no matching OpenPBR controls"],
}
FINISH_NOTES = {
    "plain": "single Principled surface",
    "matte": "high roughness surface; no texture claim",
    "brushed": "procedural directional wave varies roughness across the surface",
    "polished": "low roughness with coat for a controlled highlight",
    "paper": "matte dielectric paper role",
    "road": "matte road role with restrained roughness",
    "glass": "glass-like coat and low roughness; transmission is not claimed as OpenPBR conformance",
    "enamel": "coated colored metal role",
    "skeuomorph": (
        "the iOS 6 layer: a heavy coat over the family's own surface, "
        "which is the half of that layer a swatch can see — the geometry half is "
        "declared per icon in shapes.py"
    ),
}

#: The surface of each family, declared once. Keys must stay in step with
#: `palette.FAMILIES`, and `family_coverage()` is what says whether they have.
#:
#: These are the values the first pass reached per recipe — a metal at metalness
#: 0.9 with brushed roughness, a lens at 0.12 — promoted from per-object numbers to
#: one row each, because a family that shimmers differently in two icons is two
#: materials wearing one name.
FAMILY_SURFACE = {
    "metal":   {"base_metalness": 0.9, "specular_roughness": 0.28, "specular_ior": 1.50, "geometry_coat_weight": 0.35, "finish": "brushed"},
    "paper":   {"base_metalness": 0.0, "specular_roughness": 0.70, "specular_ior": 1.45, "geometry_coat_weight": 0.0, "finish": "paper"},
    "ceramic": {"base_metalness": 0.0, "specular_roughness": 0.35, "specular_ior": 1.48, "geometry_coat_weight": 0.35, "finish": "polished"},
    "glass":   {"base_metalness": 0.0, "specular_roughness": 0.12, "specular_ior": 1.52, "geometry_coat_weight": 0.6, "finish": "glass"},
    "polymer": {"base_metalness": 0.0, "specular_roughness": 0.80, "specular_ior": 1.45, "geometry_coat_weight": 0.0, "finish": "matte"},
    "road":    {"base_metalness": 0.0, "specular_roughness": 0.62, "specular_ior": 1.45, "geometry_coat_weight": 0.0, "finish": "road"},
    "enamel":  {"base_metalness": 0.0, "specular_roughness": 0.24, "specular_ior": 1.50, "geometry_coat_weight": 0.45, "finish": "enamel"},
}


def family_surface(family: str, finish: str = None) -> dict:
    """One family's surface, optionally wearing a different finish.

    The finish override exists for the accent piece: it reuses one of the icon's
    three materials and is told apart by *finish* rather than by hue, which is what
    keeps "three main colours" literally true.
    """
    if family not in FAMILY_SURFACE:
        raise KeyError(
            f"`{family}` has no declared surface; declared: {', '.join(sorted(FAMILY_SURFACE))}"
        )
    surface = dict(FAMILY_SURFACE[family])
    if finish is not None:
        if finish not in FINISH_NOTES:
            raise KeyError(f"unknown authored finish `{finish}`")
        surface["finish"] = finish
    return surface


def family_coverage() -> dict:
    """Whether the surface table and the colour matrix still describe one set."""
    colours = set(palette.FAMILIES)
    surfaces = set(FAMILY_SURFACE)
    return {
        "families": sorted(colours | surfaces),
        "colours_without_a_surface": sorted(colours - surfaces),
        "surfaces_without_a_colour": sorted(surfaces - colours),
    }


def blender_input(parameter: str) -> str:
    if parameter in UNSUPPORTED:
        raise KeyError(f"OpenPBR parameter `{parameter}` is unsupported: {UNSUPPORTED[parameter]}")
    if parameter not in MAPPING:
        raise KeyError(f"unknown OpenPBR parameter `{parameter}`")
    return MAPPING[parameter]


def _shader_parameters(material: dict) -> dict:
    return {key: value for key, value in material.items() if key not in METADATA}


def validate(material: dict) -> dict:
    if not material:
        raise ValueError("a material with no parameters is not a material")
    finish = material.get("finish", "plain")
    if finish not in FINISH_NOTES:
        raise KeyError(f"unknown authored finish `{finish}`")
    for parameter in _shader_parameters(material):
        blender_input(parameter)
    return material


def _finish_nodes(tree, bsdf, finish):
    if finish == "brushed":
        # A single roughness wave looked like plastic striping. Use a restrained
        # directional grain for roughness, then a much smaller bump signal so the
        # highlight breaks across the surface without becoming noise at 24px.
        texcoord = tree.nodes.new("ShaderNodeTexCoord")
        mapping = tree.nodes.new("ShaderNodeMapping")
        mapping.inputs["Scale"].default_value = (1.0, 7.0, 1.0)
        wave = tree.nodes.new("ShaderNodeTexWave")
        wave.wave_type = "BANDS"
        wave.bands_direction = "X"
        wave.inputs["Scale"].default_value = 42.0
        wave.inputs["Distortion"].default_value = 2.0
        wave.inputs["Detail"].default_value = 2.0
        roughness = tree.nodes.new("ShaderNodeMapRange")
        roughness.inputs["From Min"].default_value = 0.28
        roughness.inputs["From Max"].default_value = 0.72
        roughness.inputs["To Min"].default_value = 0.20
        roughness.inputs["To Max"].default_value = 0.34
        bump = tree.nodes.new("ShaderNodeBump")
        bump.inputs["Strength"].default_value = 0.08
        bump.inputs["Distance"].default_value = 0.025
        tree.links.new(texcoord.outputs["Generated"], mapping.inputs["Vector"])
        tree.links.new(mapping.outputs["Vector"], wave.inputs["Vector"])
        tree.links.new(wave.outputs["Color"], roughness.inputs["Value"])
        tree.links.new(wave.outputs["Color"], bump.inputs["Height"])
        tree.links.new(roughness.outputs["Result"], bsdf.inputs["Roughness"])
        tree.links.new(bump.outputs["Normal"], bsdf.inputs["Normal"])
    elif finish in {"polished", "enamel"}:
        if "Coat Weight" in bsdf.inputs:
            bsdf.inputs["Coat Weight"].default_value = max(bsdf.inputs["Coat Weight"].default_value, 0.35)
    elif finish == "skeuomorph":
        # The iOS 6 layer, as far as a surface can carry it: a coat heavy enough to
        # read as glass over the family's own body. Nothing here is a post-process,
        # because the rig has none and says so.
        if "Coat Weight" in bsdf.inputs:
            bsdf.inputs["Coat Weight"].default_value = 0.85
        if "Coat Roughness" in bsdf.inputs:
            bsdf.inputs["Coat Roughness"].default_value = 0.05
        if "Specular IOR Level" in bsdf.inputs:
            bsdf.inputs["Specular IOR Level"].default_value = 0.8
    elif finish == "glass":
        if "Coat Weight" in bsdf.inputs:
            bsdf.inputs["Coat Weight"].default_value = 0.6
        if "Transmission Weight" in bsdf.inputs:
            bsdf.inputs["Transmission Weight"].default_value = 0.18


def build_material(bpy, name: str, parameters: dict):
    validate(parameters)
    material = bpy.data.materials.new(name=name)
    material.use_nodes = True
    tree = material.node_tree
    bsdf = next((node for node in tree.nodes if node.type == "BSDF_PRINCIPLED"), None)
    if bsdf is None:
        raise RuntimeError(f"no Principled BSDF in {name}")
    for parameter, value in _shader_parameters(parameters).items():
        input_name = blender_input(parameter)
        if input_name not in bsdf.inputs:
            raise RuntimeError(f"shader input `{input_name}` does not exist in this Blender build")
        bsdf.inputs[input_name].default_value = value
    _finish_nodes(tree, bsdf, parameters.get("finish", "plain"))
    material["icon_finish"] = parameters.get("finish", "plain")
    material["icon_family"] = parameters.get("family", "")
    return material


def deviations_for(parameters) -> list:
    found = []
    for parameter in _shader_parameters(parameters):
        for note in DEVIATIONS.get(parameter, []):
            if note not in found:
                found.append(note)
    finish = parameters.get("finish", "plain")
    found.append(f"authored finish `{finish}`: {FINISH_NOTES[finish]}")
    return found


def apply_base_color(material, linear_rgb, alpha: float = 1.0) -> bool:
    """Write the matrix's colour onto a material, so the render uses it.

    The division of ownership is the point: the blend owns the **surface** (metalness,
    roughness, IOR, coat) and the palette matrix owns the **colour**. Reading the
    colour back out of the blend, which is what this pipeline did, silently made the
    blend the owner of both -- every icon of a family rendered the same grey whatever
    hue it declared, and the manifest recorded that grey as the declared colour. So
    the colour is *assigned* here, and reading it back is only a verification.
    """
    bsdf = next((node for node in material.node_tree.nodes if node.type == "BSDF_PRINCIPLED"), None)
    if bsdf is None:
        return False
    socket = bsdf.inputs.get("Base Color")
    if socket is None:
        return False
    socket.default_value = (linear_rgb[0], linear_rgb[1], linear_rgb[2], alpha)
    return True


def from_blender_material(material, fallback: dict) -> dict:
    """Read editable Principled inputs back into the manifest's OpenPBR record.

    The **surface** parameters come from the material, because the blend is the rig
    and the reference material is where a family's surface is authored. The **base
    colour** comes from the declared parameters, because the matrix owns it -- see
    `apply_base_color` for what reading it from the material cost.
    """
    bsdf = next((node for node in material.node_tree.nodes if node.type == "BSDF_PRINCIPLED"), None)
    if bsdf is None:
        return fallback

    def value(name, default):
        socket = bsdf.inputs.get(name)
        return socket.default_value if socket is not None else default

    parameters = {
        "base_color": tuple(fallback["base_color"]),
        "base_metalness": float(value("Metallic", fallback["base_metalness"])),
        "specular_roughness": float(value("Roughness", fallback["specular_roughness"])),
        "specular_ior": float(value("IOR", fallback["specular_ior"])),
        "finish": material.get("icon_finish", fallback.get("finish", "plain")),
        "family": material.get("icon_family", fallback.get("family", "")),
    }
    coat = bsdf.inputs.get("Coat Weight")
    if coat is not None:
        parameters["geometry_coat_weight"] = float(coat.default_value)
    return parameters


def record(parameters: dict) -> dict:
    validate(parameters)
    return {
        "openpbr": _jsonable(_shader_parameters(parameters)),
        "mapping": {parameter: blender_input(parameter) for parameter in _shader_parameters(parameters)},
        "finish": parameters.get("finish", "plain"),
        "finish_note": FINISH_NOTES[parameters.get("finish", "plain")],
        "family": parameters.get("family"),
        "deviations": deviations_for(parameters),
    }


def _jsonable(parameters):
    out = {}
    for parameter, value in parameters.items():
        if isinstance(value, (tuple, list)) and len(value) in (3, 4):
            out[parameter] = {"linear_rgb": [round(float(channel), 6) for channel in value[:3]], "hex": _hex(value)}
        else:
            out[parameter] = round(float(value), 6)
    return out


def _hex(value) -> str:
    channels = []
    for channel in value[:3]:
        linear = max(0.0, min(1.0, float(channel)))
        encoded = 12.92 * linear if linear <= 0.0031308 else 1.055 * (linear ** (1 / 2.4)) - 0.055
        channels.append(round(encoded * 255))
    return "#{:02x}{:02x}{:02x}".format(*channels)
