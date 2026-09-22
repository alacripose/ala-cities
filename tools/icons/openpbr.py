"""The OpenPBR parameter set, mapped onto the shader Blender actually has.

The icons are authored in OpenPBR vocabulary because that is the standard named
in the brief, and rendered through Principled BSDF v2 because that is what
Blender 5.3.0 Alpha implements. The two are aligned in design and **not** equal,
so the mapping lives in one table here, every difference is named, and each
icon's manifest entry carries the deviations that applied to it.

Two rules this module enforces rather than documents:

* an unknown parameter is a **KeyError**, never a silent skip. `UNIFIED_DESIGN.md`
  §8.1: "an unknown token that is silently dropped is a defect class".
* a parameter that exists in OpenPBR but not in the shader must be listed in
  `UNSUPPORTED` and must be *unused* by every icon. If an icon needs it, the run
  fails with the reason, which is the honest outcome.
"""

# ---------------------------------------------------------------------------
# The mapping
# ---------------------------------------------------------------------------

#: OpenPBR parameter name -> Principled BSDF v2 input name.
MAPPING = {
    "base_color": "Base Color",
    "base_metalness": "Metallic",
    "specular_roughness": "Roughness",
    "specular_ior": "IOR",
    "geometry_coat_weight": "Coat Weight",
}

#: Parameters the specification defines for the materials this set uses.
#: Present here so a typo in an icon definition is an error rather than a
#: parameter that quietly never made it into the render.
ALLOWED = {"base_color", "base_metalness", "specular_roughness", "specular_ior",
           "geometry_coat_weight"}

#: OpenPBR parameters with no counterpart in the target shader. Using any of
#: these would mean writing a parameter into the manifest that the renderer
#: ignored, so `build_material` refuses instead.
UNSUPPORTED = {
    "specular_weight": "the shader's 'Specular IOR Level' is an artist scale, not a linear weight on F0",
    "base_weight": "the shader has a single base layer and no layer stack",
    "coat_roughness": "the shader has coat roughness, but not independently of the coat's own defaults",
    "thin_film_thickness": "not implemented by the target shader",
    "thin_film_ior": "not implemented by the target shader",
    "transmission_weight": "not used by an opaque icon; deliberately unpinned rather than approximated",
    "subsurface_weight": "not used by an opaque icon",
    "fuzz_weight": "not used by an opaque icon",
}

#: Per-parameter deviations between the specification and the shader. Recorded
#: per icon, not summarised away: a reader of the manifest should be able to see
#: which claim each icon is standing on.
DEVIATIONS = {
    "base_color": [],
    "base_metalness": [
        "OpenPBR's metallic is defined as a weight between dielectric and conductor; the shader's 'Metallic' is the same range, and these icons use it at 0.0 or 1.0 only, where both readings agree"
    ],
    "specular_roughness": [
        "the specification carries separate dielectric and metallic roughness; the shader has one 'Roughness'. No icon mixes the two on one surface, so the collapse is never exercised, but it is real"
    ],
    "specular_ior": [],
    "geometry_coat_weight": [
        "the shader's coat has its own roughness and IOR which this table does not set, so a coated icon inherits Blender's defaults rather than the specification's"
    ],
}


def blender_input(parameter: str) -> str:
    """The shader input a parameter maps onto, or a refusal."""
    if parameter in UNSUPPORTED:
        raise KeyError(
            f"OpenPBR parameter `{parameter}` has no counterpart in the target shader "
            f"({UNSUPPORTED[parameter]}); an icon using it would put a parameter in the "
            f"manifest that the renderer ignored"
        )
    if parameter not in MAPPING:
        raise KeyError(f"unknown OpenPBR parameter `{parameter}`")
    return MAPPING[parameter]


def deviations_for(parameters) -> list:
    """Every deviation that applies to a set of parameters, in table order."""
    found = []
    for parameter in parameters:
        if parameter in UNSUPPORTED:
            raise KeyError(
                f"`{parameter}` is unsupported and cannot be declared: {UNSUPPORTED[parameter]}"
            )
        for note in DEVIATIONS.get(parameter, []):
            if note not in found:
                found.append(note)
    return found


def validate(material: dict) -> dict:
    """Check a material definition before it reaches the renderer."""
    if not material:
        raise ValueError("a material with no parameters is not a material")
    for parameter in material:
        blender_input(parameter)  # raises on anything unmapped
    return material


def build_material(bpy, name: str, parameters: dict):
    """Build a Blender material from OpenPBR parameters.

    The node tree is deliberately a single Principled BSDF, because that is what
    the mapping table describes. Anything else would make the table a fiction.
    """
    validate(parameters)
    material = bpy.data.materials.new(name=name)
    material.use_nodes = True
    tree = material.node_tree
    bsdf = None
    for node in tree.nodes:
        if node.type == "BSDF_PRINCIPLED":
            bsdf = node
            break
    if bsdf is None:
        raise RuntimeError(
            f"no Principled BSDF in {name}; the mapping table describes that shader and "
            f"nothing else, so a different node tree would make this table wrong"
        )

    for parameter, value in parameters.items():
        input_name = blender_input(parameter)
        if input_name not in bsdf.inputs:
            raise RuntimeError(
                f"shader input `{input_name}` does not exist in this Blender build, so "
                f"OpenPBR parameter `{parameter}` would be silently dropped"
            )
        bsdf.inputs[input_name].default_value = value
    return material


def record(parameters: dict) -> dict:
    """What goes into the manifest about a material: parameters, mapping, deviations."""
    validate(parameters)
    return {
        "openpbr": _jsonable(parameters),
        "mapping": {parameter: blender_input(parameter) for parameter in parameters},
        "deviations": deviations_for(parameters),
    }


def _jsonable(parameters: dict):
    """Colours as hex, numbers as numbers: a manifest a person can read."""
    out = {}
    for parameter, value in parameters.items():
        if isinstance(value, (tuple, list)) and len(value) in (3, 4):
            out[parameter] = {
                "linear_rgb": [round(float(channel), 6) for channel in value[:3]],
                "hex": _hex(value),
            }
        else:
            out[parameter] = round(float(value), 6)
    return out


def _hex(value) -> str:
    """Linear RGB to an sRGB hex string, for a human reading the manifest."""
    channels = []
    for channel in value[:3]:
        linear = max(0.0, min(1.0, float(channel)))
        encoded = (
            12.92 * linear
            if linear <= 0.0031308
            else 1.055 * (linear ** (1 / 2.4)) - 0.055
        )
        channels.append(round(encoded * 255))
    return "#{:02x}{:02x}{:02x}".format(*channels)
