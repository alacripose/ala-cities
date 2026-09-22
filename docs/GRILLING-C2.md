# Grilling — round 7: presentation

Round 7 exists because the first build ran and the player said, in effect: *this
is hard to read, it is flat, and it does not look like the design doc.* Those are
three different problems with three different sizes, so they are asked as
separate questions rather than bundled into "make the UI better".

**Convention, as in `GRILLING.md`:** `➡️` is the recommendation. `✔` is the
answer. Where an answer overrides a recommendation, it says so.

---

## Facts gathered before asking — measurements, not impressions

| Claim under test | Measurement | Verdict |
|---|---|---|
| "Text is extremely difficult to read" | **`SIZE_SMALL` (12 px) is used at 46 of 61 text call sites**; body 14 px at 11; display 20 px at 4 | Cause found |
| "…and inconsistently sized" | A 3-size scale, but **6 off-scale literals** (11, 12, 16, 24, 32, 40) are passed at call sites | Cause found |
| | Glyph quads are **not snapped to integer pixels**, and the atlas is magnified with **`FilterMode::Linear`** — so at fractional positions every glyph is resampled and soft, worst at the smallest size | Cause found |
| | Contrast is **fine**: 14.24:1 body-on-panel, 17.02:1 body-on-desk, 8.57:1 muted, 7.02:1 on ink | Ruled out |
| | Hypothesis: the HUD ignores the OS scale factor. **Measured `scale_factor = 1.0` on this machine**, so it does not explain it here — it remains a latent bug on a scaled display | Hypothesis killed |
| "The scene is 2D top-down" | The camera is orthographic with a `zoom` and a `focus` only. No pitch, no rotation, no height | Confirmed |
| "The UI doesn't have any icons" | **Zero icons.** 61 label strings, no iconography | Confirmed |
| "…or follow the unified design doc" | Follows: tokens all have records (32/32), contrast measured, targets **56 px** high (≥ 48). Misses: no type scale, no 4/8-unit spacing scale (6, 22, 26, 30, 34 appear), mono-for-numbers is partial (7 sites) | Split verdict |

---

## ❓ Q29 — The camera and the scene

The design doc's pattern library is full of surfaces that read as a *place*; a
flat orthographic grid does not. CS2's feel is mostly tilt + rotate + zoom, with
buildings extruded as boxes.

- **(a)** Keep 2D top-down; add 90° rotation and smoother pan/zoom.
- **(b)** **Tilted camera** — keep the tile data exactly as it is, add a
  perspective/oblique projection, a depth buffer, and buildings extruded to a
  height by level, with cast shadows.
- **(c)** Full 3D mesh pipeline with real models.

➡️ **(b).** It buys almost all of CS2's spatial feel for a fraction of (c), and
it does not touch the simulation at all: the height comes from `Building::level`,
which already exists. The honest cost is a new render pipeline with depth, box
geometry per building, and picking/pan/zoom maths rewritten around a tilted
basis.

**✔ — pending**

## ❓ Q30 — Readability

Three separate causes, three separate fixes. Bundling them is how "make the text
better" turns into a repaint that changes nothing.

- **(a)** A **named type scale** in one place (e.g. 12 / 14 / 16 / 20 / 28), with
  **no literal size at any call site**, nearest-filtered atlas sampling, glyph
  positions snapped to whole pixels, and a **UI scale setting**.
- **(b)** Just increase the sizes and leave the rest.

➡️ **(a).** Raising 12 px to 16 px without fixing the sampling still renders soft
glyphs; fixing sampling without raising the scale still leaves 46 call sites at
the smallest size. The design doc also requires legibility "at every supported
size", which only a named scale plus a scale setting can honestly claim.

**✔ — pending**

## ❓ Q31 — Icons

- **(a)** An **OS icon font** (Segoe MDL2 Assets / Segoe Fluent Icons on
  Windows), loaded at runtime exactly as `consola.ttf` already is — real icons,
  no bundled asset, so §8.2's licence gate has nothing to gate.
- **(b)** Icons drawn procedurally in code, like the audio.
- **(c)** Text only, larger.

➡️ **(a), with (b) for the handful the system font lacks.** Note what the design
doc actually says an icon *is*: a **locator to a surface or record**, never a
capability grant. So the icon set is derived from surfaces that exist — tool,
ledger, ticket, district, power — and not from decoration.

**✔ — pending**

## ❓ Q32 — What "follow the design doc" binds

- **(a)** The **mechanical tier**, enforced in a build check that fails closed:
  named type scale, 4/8-unit spacing scale, target sizes, mono for numbers,
  measured contrast, every token having a record.
- **(b)** (a) plus the doc's **product patterns**: tool panel as a popit-style
  contextual surface, desk/paper surfaces, diegetic panels.
- **(c)** All of the above plus §14-style mode scoping of tools.

➡️ **(a)** plus one slice of **(b)** — the tool panel is already the popit
analogue in behaviour, so making it *look* like one costs little. **(c)** stays
out: a18 removed modes, so there is nothing to scope.

**✔ — pending**

## ❓ Q33 — What C2 does first

- **(a)** A **presentation pass**: camera, type scale, icons, the conformance
  check — before any new simulation system, plus `F12` frame capture written
  into `playtest/<stage>/` so a feedback note can carry the frame it is about.
- **(b)** Simulation first: service coverage, milestones, then presentation.

➡️ **(a).** Every simulation system added on top of unreadable text is a system
nobody can playtest. The user's own report is the evidence for the order.

**✔ — pending**

---

## Answers

*Recorded as the round proceeds. Nothing above is settled until this section says
so.*
