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
