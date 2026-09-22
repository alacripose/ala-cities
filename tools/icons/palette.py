"""The interface's colours, recomputed for the renderer.

The icons must agree with the surfaces they locate, and the only way to be sure
of that is to derive them from the same numbers rather than to copy them. Every
value here is the OKLCH triple that `src/hud.rs` carries for the same token, run
through the same OKLab conversion, so a token change that is not followed by a
re-render shows up as a palette hash mismatch in the manifest.

`src/hud.rs` is the authority; this file is a transcription of it, and
`tests.rs`-style duplication is avoided by hashing the transcription into the
manifest so the two can be compared by eye or by script.
"""

import hashlib

# ---------------------------------------------------------------------------
# OKLab -> linear sRGB (the same algebra as hud.rs::oklch)
# ---------------------------------------------------------------------------


def oklch(lightness: float, chroma: float, hue_degrees: float):
    """OKLCH to **linear** sRGB, which is what a renderer wants."""
    import math

    hue = math.radians(hue_degrees)
    a = chroma * math.cos(hue)
    b = chroma * math.sin(hue)

    l_ = lightness + 0.39633778 * a + 0.21580376 * b
    m_ = lightness - 0.10556135 * a - 0.06385417 * b
    s_ = lightness - 0.08948418 * a - 1.2914855 * b

    l3, m3, s3 = l_ ** 3, m_ ** 3, s_ ** 3

    r = 4.0767417 * l3 - 3.3077116 * m3 + 0.23096993 * s3
    g = -1.268438 * l3 + 2.6097574 * m3 - 0.3413194 * s3
    b_ = -0.00419609 * l3 - 0.7034186 * m3 + 1.7076147 * s3

    return (min(max(r, 0.0), 1.0), min(max(g, 0.0), 1.0), min(max(b_, 0.0), 1.0))


# ---------------------------------------------------------------------------
# The tokens these icons borrow from, verbatim from src/hud.rs
# ---------------------------------------------------------------------------

TOKENS = {
    # name: (l, c, h) exactly as the style table builds them
    "Desk": (0.17, 0.012, 260.0),
    "Panel": (0.25, 0.014, 260.0),
    "Ink": (0.45, 0.130, 250.0),
    "Nature": (0.70, 0.150, 150.0),
    "Warning": (0.74, 0.160, 70.0),
    "Refused": (0.62, 0.180, 25.0),
    "NotObtained": (0.55, 0.020, 260.0),
    "TextBody": (0.96, 0.005, 260.0),
    "TextMuted": (0.80, 0.010, 260.0),
    "PanelRaised": (0.31, 0.016, 260.0),
    "Plaque": (0.42, 0.030, 80.0),
    "Road": (0.42, 0.005, 260.0),
    "RoadEdge": (0.52, 0.005, 260.0),
    "Scaffold": (0.60, 0.090, 80.0),
    "ZoneResidential": (0.55, 0.130, 150.0),
    "ZoneCommercial": (0.55, 0.130, 250.0),
    "ZoneIndustrial": (0.55, 0.130, 60.0),
    "CaseOpen": (0.74, 0.160, 70.0),
    "Powered": None,  # derived from Nature with alpha in hud.rs
    "Retired": (0.65, 0.020, 260.0),
    "Agent": (0.92, 0.020, 260.0),
}


def linear(token: str):
    """A token's linear RGB."""
    if token == "Powered":
        return oklch(*TOKENS["Nature"])
    if token not in TOKENS:
        raise KeyError(f"no palette entry for token `{token}`; add it before using it")
    return oklch(*TOKENS[token])


def rgba(token: str, alpha: float = 1.0):
    """A token as the four floats the renderer takes."""
    linear_rgb = linear(token)
    return (linear_rgb[0], linear_rgb[1], linear_rgb[2], alpha)


# ---------------------------------------------------------------------------
# Hosts, and the contrast a non-text component owes
# ---------------------------------------------------------------------------

#: The surfaces an icon is allowed to sit on, as the tokens that fill them. An icon
#: must declare which of these it lands on, and the pipeline measures it against
#: each one: a glyph that passes every other check and is invisible on the panel it
#: was put on has still failed.
#:
#: Only surfaces that exist today are listed. `Ground` and `Water` are deliberately
#: absent: the world palette is not in `hud.rs` yet, so their values would have to
#: be invented here, and an invented value is exactly what this file exists to
#: avoid.
HOSTS = ("Desk", "Panel", "PanelRaised")

#: WCAG 2.2 1.4.11 Non-text Contrast, AA: a UI component needs 3:1 against what it
#: sits on. Body text's 4.5:1 floor is measured separately, in `src/hud.rs`.
NON_TEXT_MIN_CONTRAST = 3.0


def relative_luminance(linear_rgb) -> float:
    """WCAG relative luminance of a **linear** RGB triple."""
    return 0.2126 * linear_rgb[0] + 0.7152 * linear_rgb[1] + 0.0722 * linear_rgb[2]


def contrast_ratio(a: float, b: float) -> float:
    """WCAG contrast between two relative luminances."""
    lighter, darker = max(a, b), min(a, b)
    return (lighter + 0.05) / (darker + 0.05)


def host_luminance(token: str) -> float:
    """The relative luminance of a host surface's own fill."""
    if token not in HOSTS:
        raise KeyError(
            f"`{token}` is not a declared host; an icon may only sit on a surface "
            f"this file can measure it against ({', '.join(HOSTS)})"
        )
    return relative_luminance(linear(token))


def palette_hash() -> str:
    """A stable digest of every value in the table.

    Recorded in the manifest so "the icons were re-rendered after the palette
    moved" is checkable rather than asserted.
    """
    digest = hashlib.sha256()
    for name in sorted(TOKENS):
        value = TOKENS[name]
        digest.update(name.encode())
        if value is not None:
            digest.update(repr(value).encode())
    return digest.hexdigest()[:16]


def source_hash(path: str) -> str:
    """A digest of the authority this table transcribes."""
    with open(path, "rb") as handle:
        return hashlib.sha256(handle.read()).hexdigest()[:16]
