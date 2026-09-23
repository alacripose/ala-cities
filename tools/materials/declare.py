"""The material declaration: one home for every value both halves read.

This module is **data only**. The colour algebra lives in `tools/icons/palette.py`,
which imports this file, and the generator `tools/materials/emit.py` resolves it
into `src/materials/generated.rs`.

Why it exists at all: the icon pipeline used to *transcribe* `src/hud.rs`'s tokens
("`src/hud.rs` is the authority; this file is a transcription of it"), so a change
to a panel colour could leave the icons measuring contrast against a stale copy and
nothing would say so. And the game world had no material vocabulary at all. Both
halves now read one declared table, and a divergence is a build failure rather than
a silent drift.

Two families sets, deliberately separate:

* `ICON_FAMILIES` — exactly seven. `ICON_STANDARD.md` states seven as a property of
  the icon set, so a family the world needs does not silently widen what an icon may
  be made of.
* `WORLD_FAMILIES` — the world's extra materials, with the same mechanism/ceiling
  discipline. Grass and water are not glass and paper, and saying so is cheaper than
  pretending a lake is a pane.

Every number here is a proposal recorded in `docs/GRILLING-C8.md` §7 and is
overridable line by line; an override is recorded as an override, not applied
quietly.
"""

# ---------------------------------------------------------------------------
# The seven icon families, verbatim from the matrix C4 derived
# ---------------------------------------------------------------------------

#: family: (lightness, variant_chroma, natural_chroma, natural_hue, note)
ICON_FAMILIES = {
    "metal":   (0.78, 0.060, 0.018, 250.0, "the light cool object the study's chrome reads as"),
    "paper":   (0.88, 0.060, 0.035, 82.0, "matte warm sheet"),
    "ceramic": (0.90, 0.050, 0.025, 72.0, "glazed insulator"),
    "glass":   (0.66, 0.100, 0.090, 220.0, "lens and pane"),
    "polymer": (0.34, 0.050, 0.018, 250.0, "dark grip"),
    "road":    (0.38, 0.040, 0.018, 250.0, "matte surface"),
    "enamel":  (0.62, 0.190, 0.190, 245.0, "the study's blue control face; coated colour"),
}

# ---------------------------------------------------------------------------
# The world's own families (C8 a153: the icon set stays seven)
# ---------------------------------------------------------------------------

#: Each world family's `natural` entry is set to the value the game already ships
#: for that surface, so declaring the material does not quietly restyle the city.
#: Where a surface had no material before (`organic`, `soil`), the natural entry is
#: the nearest shipping colour and the move is recorded as a divergence.
WORLD_FAMILIES = {
    "water":   (0.42, 0.100, 0.070, 240.0,
                "body volume: colour held in the water itself, with depth absorption"),
    "organic": (0.30, 0.120, 0.030, 140.0,
                "pigment in living tissue: never uniform, always slightly varied"),
    "soil":    (0.52, 0.060, 0.005, 260.0,
                "aggregate and mineral: near-neutral, matte, granular"),
}

#: How colour physically arrives, per family, and the most chroma that mechanism
#: carries. The number equals the family's `variant_chroma`; `colour_mechanisms()`
#: refuses a table where the two disagree, so a ceiling cannot drift away from the
#: chroma actually in use.
FAMILY_COLOUR = {
    "metal": (
        "anodised film: an oxide layer on the substrate, so a metal body stays a metal",
        0.060,
    ),
    "glass": (
        "body-tinted: colour held in the glass itself, which also tints what shows through it",
        0.100,
    ),
    "ceramic": (
        "fired glaze: a mineral glaze over the body, opaque rather than transparent",
        0.050,
    ),
    "polymer": (
        "pigmented resin: colour compounded into the plastic itself",
        0.050,
    ),
    "paper": (
        "dyed stock: pigment in the sheet, which is why it stays matte and pale",
        0.060,
    ),
    "road": (
        "aggregate: asphalt and stone, near-neutral because that is what it is made of",
        0.040,
    ),
    "enamel": (
        "painted colour: a pigmented coating over a substrate -- the family paint belongs to",
        0.190,
    ),
    "water": (
        "body volume: light travels through the material, so its colour is a property of depth",
        0.100,
    ),
    "organic": (
        "pigment in living tissue: grown rather than applied, and never uniform across a body",
        0.120,
    ),
    "soil": (
        "aggregate and mineral grain: a mixture, so it reads near-neutral and matte",
        0.060,
    ),
}

#: A level is a declared lightness offset on the family's own lightness, so a body
#: and its edge are one material read twice rather than two materials.
MATERIAL_LEVELS = {"body": 0.0, "edge": 0.10, "deep": -0.18}
MATERIAL_LIGHTNESS_BOUNDS = (0.05, 0.97)

# ---------------------------------------------------------------------------
# Hue anchors
# ---------------------------------------------------------------------------

HUE_ANCHORS = (
    ("red", 25.0, "backed by Token::Refused (25) — destructive"),
    ("amber", 62.0, "backed by ZoneIndustrial (60) / Warning (70) / Scaffold (80) — caution, power"),
    ("yellow", 115.0, "declared slot — no reference and no game meaning occupies it"),
    ("green", 150.0, "backed by Nature / ZoneResidential (150) — nature, residential"),
    ("cyan", 200.0, "partially backed — the study's people (162.6) and store (201.8) classes; no game token"),
    ("blue", 250.0, "backed by Ink / ZoneCommercial (250) — data, commercial"),
    ("violet", 300.0, "declared slot — no reference and no game meaning occupies it"),
)

HUE_ANCHOR_NAMES = tuple(name for name, _degrees, _backing in HUE_ANCHORS) + ("natural",)

# ---------------------------------------------------------------------------
# The tokens this table owns
# ---------------------------------------------------------------------------

#: Every colour the icon pipeline measures against, plus the world's surface
#: colours. `hud.rs` no longer hardcodes these: it resolves them from the emitted
#: table, so an icon cannot be measured against a copy of a colour.
#:
#: Values are unchanged from the shipping table. Where a token is the *reading of a
#: material* (`Ground`, `Water`, `Road`, `RoadEdge`), `WORLD_SURFACES` names the
#: material and the generator checks the two agree, so a token cannot drift away
#: from the material it claims to be.
SHARED_TOKENS = {
    # Host surfaces an icon may sit on, and the inks the checks pair with them.
    "Desk":        (0.17, 0.012, 260.0),
    "Panel":       (0.25, 0.014, 260.0),
    "PanelRaised": (0.31, 0.016, 260.0),
    "Plaque":      (0.42, 0.030, 80.0),
    "Ink":         (0.45, 0.130, 250.0),
    "Nature":      (0.70, 0.150, 150.0),
    "Warning":     (0.74, 0.160, 70.0),
    "Refused":     (0.62, 0.180, 25.0),
    "NotObtained": (0.55, 0.020, 260.0),
    "TextBody":    (0.96, 0.005, 260.0),
    "TextMuted":   (0.80, 0.010, 260.0),
    # World surfaces, each the reading of a declared material.
    "Ground":      (0.30, 0.030, 140.0),
    "Water":       (0.42, 0.070, 240.0),
    "Road":        (0.42, 0.005, 260.0),
    "RoadEdge":    (0.52, 0.005, 260.0),
    "Grid":        (0.35, 0.010, 260.0),
    # World and record state the frame draws.
    "Scaffold":    (0.60, 0.090, 80.0),
    "ZoneResidential": (0.55, 0.130, 150.0),
    "ZoneCommercial":  (0.55, 0.130, 250.0),
    "ZoneIndustrial":  (0.55, 0.130, 60.0),
    "CaseOpen":    (0.74, 0.160, 70.0),
    "Retired":     (0.65, 0.020, 260.0),
    "Agent":       (0.92, 0.020, 260.0),
}

#: Colours the icon authoring side uses that are not interface tokens: the near-black
#: a recipe may name, kept so an existing recipe still resolves.
ICON_COLOURS = {
    "IconBlack": (0.16, 0.018, 250.0),
}

#: The old role names, kept so an existing recipe still resolves — each one an alias
#: into the matrix rather than a second copy of a value.
MATERIAL_ALIASES = {
    "MetalBody": "metal:natural",
    "MetalEdge": "metal:natural:edge",
    "PaperBody": "paper:natural",
    "CeramicInsulator": "ceramic:natural",
    "GlassLens": "glass:natural",
    "PolymerGrip": "polymer:natural",
    "RoadSurface": "road:natural",
    "IconBlue": "enamel:natural",
}

#: What the anchors and the families were derived from, recorded so the palette hash
#: covers the provenance and not just the numbers.
REFERENCE_SAMPLING = {
    "pack": "assets/reference/galaxy-s4-icon-pack",
    "population": "res/drawable/*.png — the TouchWiz-native raster icons",
    "population_files": 125,
    "population_note": (
        "the APK also carries Material Components resources, which are not the "
        "artistic-intent family; measured, the two populations give identical "
        "anchors, so this is a provenance statement rather than a correction"
    ),
    "method": (
        "per-class circular mean hue, weighted by saturation x value, over opaque "
        "pixels with saturation above 0.25 and value above 0.15"
    ),
    "measured_class_hues": {
        "people": 162.6, "message": 34.5, "media": 226.2,
        "system": 131.4, "store": 201.8, "nature_health": 193.0,
    },
    "finding": (
        "the study's classes occupy a 95 degree arc from green through cyan to blue "
        "with one orange outlier: it contains no red, no yellow and no violet, so "
        "seven anchors cannot be sampled from it. Four are backed by the game's own "
        "declared hues, one is partial, and two are declared slots"
    ),
    "character": {
        "median_saturation": (0.53, 0.92),
        "median_value": (0.69, 0.92),
        "reading": (
            "TouchWiz colour is saturated and bright, so variant chroma follows that "
            "rather than the near-neutral bodies the first pass used"
        ),
    },
}

# ---------------------------------------------------------------------------
# The world's surfaces, as claims
# ---------------------------------------------------------------------------

#: part: (family, hue, level, note). Every entry is a claim a reader can check:
#: the family exists, the hue is a declared anchor (or the family's own `natural`),
#: and the level is one of the three declared offsets.
#:
#: `inherits` is not a material: a ruin is the retired structure's own parts at
#: `deep`, so it is listed as a rule rather than as ten copies of the table above.
WORLD_SURFACES = {
    "home.walls":        ("ceramic", "natural", "body", "the home's body"),
    "home.roof":         ("polymer", "amber", "edge", "a different material from the walls, read once"),
    "home.window":       ("glass", "cyan", "body", "the one transparent part"),
    "shop.walls":        ("ceramic", "natural", "body", "shares the residential body"),
    "shop.frontage":     ("enamel", "blue", "body", "the commercial face paint belongs to"),
    "shop.window":       ("glass", "cyan", "body", ""),
    "factory.frame":     ("metal", "natural", "body", "structural metal"),
    "factory.cladding":  ("enamel", "amber", "body", "coated industrial panel"),
    "factory.vent":      ("polymer", "natural", "deep", "the dark recessed part"),
    "power.frame":       ("metal", "natural", "body", ""),
    "power.stack":       ("ceramic", "natural", "edge", "a fired stack, not a painted one"),
    "power.insulator":   ("ceramic", "amber", "body", "the insulator is the part that must not conduct"),
    "power.core":        ("enamel", "amber", "body", "painted caution face"),
    "road.surface":      ("road", "natural", "body", "aggregate"),
    "road.edge":         ("soil", "natural", "body", "the shoulder: mineral, not asphalt"),
    "terrain.ground":    ("organic", "natural", "body", "the grass is a living surface, not a mineral one"),
    "terrain.water":     ("water", "natural", "body", "a body of water is a volume, not a pane"),
    "powerline.conductor": ("metal", "natural", "body", "what a conductor must be made of"),
    "powerline.pylon":   ("metal", "natural", "deep", "the same metal read darker"),
    "scaffold.frame":    ("metal", "amber", "edge", "under construction, and painted to say so"),
    "scaffold.deck":     ("paper", "natural", "body", "a board, not a beam"),
    "zone.paint":        ("enamel", "__anchor__", "body", "a tint over whatever it paints on; the anchor is the zone's"),
    "ruin.parts":        ("__inherits__", "__inherits__", "deep", "the retired structure's own parts, darker"),
}

#: The tokens that **are** a surface's material read out, rather than a separate
#: colour that happens to look similar. The generator resolves each token from its
#: material, so the two cannot drift; where the material's value differs from the
#: shipping literal, the move is emitted as a recorded divergence.
SURFACE_TOKENS = {
    "Ground": "terrain.ground",
    "Water": "terrain.water",
    "Road": "road.surface",
    "RoadEdge": "road.edge",
}

#: A surface the world does not have yet, declared so the absence is a record rather
#: than an invention. Same pattern as the icon inventory's deferred list.
DEFERRED_SURFACES = {
    "terrain.vegetation": (
        "the world draws one green ground; a second vegetation surface does not exist, "
        "so `organic` has one consumer and not two"
    ),
    "soil.paving": (
        "declared slot: `soil` is the road shoulder today; unpaved ground and yards "
        "would be its second consumer"
    ),
}

# ---------------------------------------------------------------------------
# The four sim effects, as declared numbers
# ---------------------------------------------------------------------------

#: What a family costs and how it behaves. Every one of these is a proposal from
#: `docs/GRILLING-C8.md` §7.4 — overridable line by line, and the balance is
#: re-tuned rather than preserved (a173), so the tests that assert the old numbers
#: are corrected by recorded correction.
PART_PRICE = {
    # part: credits. A structure's build cost is the sum of its parts.
    "home.walls": 60, "home.roof": 40, "home.window": 40,
    "shop.walls": 70, "shop.frontage": 90, "shop.window": 80,
    "factory.frame": 180, "factory.cladding": 120, "factory.vent": 60,
    "power.frame": 700, "power.stack": 400, "power.insulator": 250, "power.core": 700,
    "road.surface": 3, "road.edge": 1,
    "powerline.conductor": 12, "powerline.pylon": 30,
    "scaffold.frame": 8, "scaffold.deck": 4,
}

#: Credits per month, per part, by family.
UPKEEP_PER_MONTH = {
    "metal": 2.0, "enamel": 1.6, "glass": 1.5, "ceramic": 1.2, "polymer": 0.8,
    "paper": 0.3, "road": 0.4, "water": 0.0, "organic": 0.05, "soil": 0.1,
}

#: Condition lost per sim-day, by the part's family. Repaired, never patched quietly.
DECAY_PER_DAY = {
    "metal": 0.0008, "enamel": 0.0006, "glass": 0.0005, "ceramic": 0.0004,
    "polymer": 0.0012, "paper": 0.0018, "road": 0.0006, "water": 0.0,
    "organic": 0.0015, "soil": 0.0009,
}

#: Below this condition a repair ticket is filed. Above the ceiling a structure is
#: fully maintained and nothing is owed.
REPAIR_FLOOR = 0.35
REPAIR_CEILING = 1.0
#: What a repair costs: a share of the structure's own build cost.
REPAIR_SHARE = 0.4

#: Whether a family carries power, and how much of it.
CONDUCTION = {
    "metal": 1.0, "enamel": 0.0, "glass": 0.0, "ceramic": 0.0, "polymer": 0.0,
    "paper": 0.0, "road": 0.0, "water": 0.2, "organic": 0.0, "soil": 0.0,
}

#: Travel speed over a surface, as a factor on the road graph's own step cost. A
#: tile the world cannot build on is 0.0 rather than "slow".
SURFACE_SPEED = {
    "road": 1.0, "soil": 0.6, "organic": 0.8, "water": 0.0,
    "metal": 0.0, "enamel": 0.0, "glass": 0.0, "ceramic": 0.0, "polymer": 0.0,
    "paper": 0.0,
}

#: How a structure moves demand on the tiles around it, per family.
DESIRABILITY = {
    "metal": -0.02, "enamel": 0.03, "glass": 0.04, "ceramic": 0.02, "polymer": -0.01,
    "paper": 0.0, "road": -0.04, "water": 0.03, "organic": 0.05, "soil": -0.01,
}

#: Noise and pollution emitted per structure, per family, in the family's own units.
#: Sampled at read points rather than diffused across 65 536 tiles every tick, so the
#: sim stays tick-deterministic and cheap.
NUISANCE = {
    "metal": 0.3, "enamel": 0.1, "glass": 0.0, "ceramic": 0.05, "polymer": 0.1,
    "paper": 0.0, "road": 0.25, "water": 0.0, "organic": 0.0, "soil": 0.1,
}

#: What this declaration refuses to claim. Printed by the gate every run, in the
#: same spirit as `design::verify`'s open list.
OPEN = (
    "whether the world reads as these materials to a person",
    "frame timing with the material shade term",
    "whether the four sim effects are balanced",
    "composite contrast on a rendered frame",
)
