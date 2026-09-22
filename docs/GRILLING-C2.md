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

**✔ — tilted camera, extruded boxes.** Height comes from `Building::level`; the
simulation is untouched.

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

**✔ — scale + sampling + UI scale.**

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

**✔ — overridden, and it changes the pipeline substantially.** Verbatim:

> Icons rendered in headless blender, mathematically/photgraphically correct
> designs that look good in an openpbr standard, that conform to the unified
> design doc

So icons are **authored and rendered**, not drawn: Blender as the authoring tool,
OpenPBR as the material standard, the design doc as the visual authority. Carried
into round 8 below, along with two consequences worth stating up front.

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

**✔ — everything, including mode scoping.** This **overrides** the recommendation,
and it **reinstates** the §14.3 absence claim that round 6 withdrew when a18
removed modes. Both are recorded as corrections:

| Was | Now | Because |
|---|---|---|
| §14.3 absence claim withdrawn (round 6) | **Reinstated** — with modes defined, the absence rule has something to scope | The user asked for the full doc |
| "a18 means no modes at all" | **a18 means the sim never gates the player** — not that the interface has no modes | Re-read of the answer: building *asynchronously* is about the city continuing, not about mode-free chrome |

## ❓ Q33 — What C2 does first

- **(a)** A **presentation pass**: camera, type scale, icons, the conformance
  check — before any new simulation system, plus `F12` frame capture written
  into `playtest/<stage>/` so a feedback note can carry the frame it is about.
- **(b)** Simulation first: service coverage, milestones, then presentation.

➡️ **(a).** Every simulation system added on top of unreadable text is a system
nobody can playtest. The user's own report is the evidence for the order.

**✔ — "all of it, grill me for more."** Both passes; presentation first. And an
explicit instruction to keep asking, which is what round 8 below is.

---

# Round 8 — the icon pipeline, modes, and the camera in detail

Round 8 exists because round 7's answers each opened a new decision. Facts
gathered first:

- **Blender is not installed** — not on `PATH`, no `Blender Foundation`
directory. Python is present (a WindowsApps shim).
- **Native OpenPBR support in Blender is an open issue**, not a feature:
`projects.blender.org` issue **#145127**, "OpenPBR Compatibility", opened
November 2025, states plainly that exact OpenPBR import/export is *wanted*. Blender's
Principled BSDF v2 is **OpenPBR-aligned in design, not conformance-exact**.
- Consequence: "rendered in an OpenPBR standard" is achievable as *authored
against the OpenPBR parameter set*, with the mapping onto Blender's shader
recorded — but **not** as a claim of exact spec conformance, and I will not write
that claim into a document.
- **The licence gate now has content.** §8.2 has had nothing to gate while every
asset was a named solid or a system font. Rendered PNGs change that: each one
needs a provenance record, and a procedurally generated render has the best
provenance available — the script that made it.

## ❓ Q34 — The icon toolchain

- **(a)** Install Blender (LTS) and render icons with **Cycles**; each icon is a
  scripted scene; materials are authored against the **OpenPBR parameter set** and
  mapped onto Blender's Principled BSDF v2, with **the mapping and any deviation
  recorded per icon**.
- **(b)** Drive MaterialX/OpenPBR through its reference implementation —
  spec-exact, much heavier toolchain, and it renders through something other than
  Blender.
- **(c)** Drop the renderer; procedural icons in code.

➡️ **(a).** It is the only option that gets a real renderer, keeps the OpenPBR
vocabulary, and stays honest about what Blender can and cannot certify today.

**✔ — overridden on the install, kept on the substance.** Verbatim: *"I have
blender 5.3 alpha installed on steam"* — and it is: `Blender 5.3.0 Alpha`, build
hash **`5eaad57cfabe`**, built 2026-09-10, at
`…/Steam/steamapps/common/Blender/blender.exe`. Verified headless in this session:

| Probe | Result |
|---|---|
| Version | 5.3.0 **Alpha**, branch `main`, Windows |
| Cycles | **available and working** — a 64×64 render completed headless |
| Devices | `NVIDIA GeForce RTX 2070 SUPER` (×2 entries), `AMD Ryzen 7 5700G` |
| OpenPBR node types | **none** — `[n for n in dir(bpy.types) if 'openpbr' in n.lower()]` is empty |
| MaterialX operators | **none** — no `wm.materialx_*` / `wm.mtlx_*` operators exist |
| PBR-relevant BSDFs | `BsdfPrincipled`, `BsdfMetallic`, `BsdfHairPrincipled`, `BsdfVolumePrincipled` |

So the mapping-and-deviation route is not a preference, it is the only one
available: author against the OpenPBR parameter set, drive Principled BSDF v2,
and **record the deviation per icon** rather than implying a conformance the
build cannot demonstrate.

## ❓ Q35 — What makes a rendered icon *correct*

"Photographically correct" has to become checkable or it is a mood.

- **(a)** Mechanical: the rendered PNG is checked against the material's own
  declared parameters within a stated tolerance; luminance response follows the
  declared albedo/roughness/metalness; contrast against every surface the icon
  sits on meets §1.4.11 (≥ 3:1 non-text); and the icon stays legible at the
  smallest shipped size. Deviations are reported, not smoothed over.
- **(b)** Judged by eye.

➡️ **(a)**, because the whole point of a spec-named standard is that something can
read it back.

**✔ — mechanical checks against the material.**

## ❓ Q36 — Where the renders live, and what the build needs

- **(a)** Renders are **committed artifacts** with a provenance manifest (script
  hash, Blender version, material parameters), plus a regeneration script.
  `cargo build` never needs Blender. A check **fails closed** if a manifest entry
  and its file disagree, or if provenance is missing.
- **(b)** Blender runs as part of the build.

➡️ **(a).** Committing the output keeps the build tool-chain-free, and the
regeneration script keeps it reproducible; a build that needs Blender cannot be
built by anyone who does not have Blender.

**✔ — commit the renders, script regenerates.**

## ❓ Q37 — What the modes actually are

- **(a)** Three: **Build**, **Inspect**, **Ledger**. Build tools are **absent**
  outside Build — absent, not greyed, per §14.3. Inspect is the lens panel;
  Ledger is the board. The simulation never pauses and never gates the player,
  which is what a18 actually required.
- **(b)** One mode per CS2-style info view (many modes).

➡️ **(a).** Three modes are enough for the absence rule to mean something without
turning every readout into a mode.

**✔ — overridden: one mode per info view.** The modes are the city's own data
layers, not an arbitrary triple. This has a consequence worth stating: in C1 only
**power** and **zoning/demand** have data behind them, so most CS2 info views
cannot exist yet without becoming controls that look like grants — which §0.1
refuses. Round 9 asks how to handle that.

## ❓ Q38 — The type scale, the spacing scale, and the UI scale

- **(a)** **micro 12 · small 14 · body 16 · title 20 · display 26**, every step a
  multiple of two, no literal size at any call site; spacing on the doc's 4-unit
  scale (4/8/12/16/24/32); targets ≥ 48 px; UI scale **100/125/150/200 %** applied
  as a device-pixel multiplier with the UI re-laid out rather than stretched.
- **(b)** Keep 12/14/20 and only add the scale setting.

➡️ **(a).** Body text moves 14 → 16 and the floor moves 12 → 12-only-for-micro,
which together address the measured cause: 12 px was doing 46 of 61 jobs.

**✔ — 12 / 14 / 16 / 20 / 26 with a UI scale.**

## ❓ Q39 — The tilted camera, in numbers

- **(a)** Fixed tilt in the **45–60°** range, rotation in **90° steps**, free zoom,
  extrusion of 1 level = a fixed height step, and contact shadows for honest
  depth. 90° steps keep the tile grid axis-aligned, so picking stays exact and
  roads stay straight on screen.
- **(b)** Free tilt and free rotation with smoothing.

➡️ **(a).** A freely rotated grid puts every road on a non-integer angle, which
makes picking, placement previews and legibility all worse for the sake of the
camera — and the design doc's own preference is that chrome does not compete with
content.

**✔ — overridden: free tilt and free rotation.** Worth being precise about what
that does and does not cost, because the recommendation above was partly wrong:
directly-drawn roads and buildings can be transformed fine. What it makes harder
is anything that resolves a click back to a tile or draws grid-aligned chrome,
which stops being a screen-space approximation and must become a real inverse
projection. Two consequences, carried into round 9: **picking must be exact at
any angle** (ray against the ground plane, not a shifted rect), and the tile grid
overlay needs a policy of its own, since at arbitrary angles it stops reading as
"the grid" and starts reading as noise.

---

## Round 8 at a glance

| # | Question | ➡️ | ✔ |
|---|---|---|---|
| Q34 | The icon toolchain | Blender + Cycles, deviations recorded | ✔ install already present; substance kept |
| Q35 | What makes an icon correct | mechanical checks against the material | ✔ |
| Q36 | Where renders live | committed artifacts + regeneration script | ✔ |
| Q37 | Which modes exist | three: Build / Inspect / Ledger | **overridden: one mode per info view** |
| Q38 | The type scale | 12/14/16/20/26 + UI scale | ✔ |
| Q39 | Camera numbers | fixed tilt, 90° rotation steps | **overridden: free tilt and rotation** |

---

# Round 9 — the gaps round 8 opened

## ❓ Q40 — The OpenPBR gap, now that it is measured

Blender 5.3.0 Alpha has **no OpenPBR node** and **no MaterialX operators**. So
"renders in an OpenPBR standard" can only mean one of these:

- **(a)** An **OpenPBR parameter table in the repo** — spec parameter names
  mapped onto Principled BSDF v2 inputs — from which scenes are generated, with a
  **per-icon deviation record** (what the spec says, what this shader does, the
delta).
- **(b)** Author materials by eye in Blender, no table, no deviation record.
- **(c)** Wait for a Blender with native OpenPBR, and ship no icons until then.

➡️ **(a).** It is the only one where "OpenPBR" stays a checkable word rather than
a mood, and (c) would block the user's own request on someone else's roadmap.

**✔ — the OpenPBR parameter table plus deviation records.**

## ❓ Q41 — An alpha build, and re-render fidelity

5.3.0 **Alpha** (`5eaad57cfabe`) will change between builds; a re-render on a
later alpha will not reproduce these pixels.

- **(a)** Pin the exact build hash in the provenance manifest, and when a render
  is regenerated on a different build, record **`reproduced: false`** beside it
  rather than quietly replacing the file.
- **(b)** Accept any 5.x as equivalent.

➡️ **(a).** The whole reason to keep a provenance manifest is to be able to say
when a render is no longer the render that was reviewed.

**✔ — overridden: accept any Blender 5.x.** Recorded with its cost, which is the
part that matters: if builds are interchangeable, a re-render on a later alpha
silently produces different pixels and nothing anywhere says so. Round 10 asks for
the cheap half of the original recommendation back — still *record* the hash,
just never fail on it — because recording costs nothing and is what makes a
difference visible later.

## ❓ Q41b — (round 10) record the hash anyway?

**✔ — pending**

## ❓ Q42 — The icon render rig

- **(a)** Orthographic camera, a fixed two-light rig (key + fill), transparent
  background, rendered at **4×** the largest shipped size and downsampled with a
  **recorded** filter. One rig for every icon.
- **(b)** Perspective camera with an HDRI environment.

➡️ **(a).** A fixed rig is what makes the mechanical check meaningful: with a
known, unchanging light setup, the rendered luminance actually reflects the
declared material parameters. An HDRI environment adds a variable that would have
to be recorded and controlled anyway, and buys nothing at 24 px.

**✔ — overridden in detail, kept in substance.** Verbatim: *"orthographic with
fixed three light rig"*. So: orthographic, **three lights** (key, fill, rim),
transparent background, all three recorded with their angles, energies and
colours so the rig itself is part of the provenance and not folklore.

## ❓ Q42b — (round 10) what the three lights are

**✔ — pending**

## ❓ Q43 — Info views, given that most of them have no data yet

- **(a)** A **view registry** fixed now, populated only with the views C1's data
  supports — **Power**, **Zoning/Demand**, **Traffic**, **Ledger** — each one
  declaring its own available tools. Later views arrive with the systems that give
  them data, one at a time.
- **(b)** Declare the full CS2 view list now (water, sewage, garbage, land value,
  noise, pollution, education, healthcare…) and stub them.

➡️ **(a).** §0.1: nothing on a surface may look more authorised than it is. A view
with no data behind it is a control that looks like a grant, and (b) would create
a dozen of them in one pass.

**✔ — overridden: stub the full CS2 view list**, verbatim, *"appending new
features into the game where needed"*. Read as: the **registry** holds the whole
roadmap, and the **interface shows only the views that have data**. That
reconciliation is the whole point, because the naive reading of the answer — draw
a dozen view buttons now — is exactly what §0.1 refuses. Round 10 confirms it, and
asks where a not-yet-live view is visible instead.

## ❓ Q43b — (round 10) where an undeclared view is visible

**✔ — pending**

## ❓ Q44 — Free rotation, and the grid

Free tilt and free rotation make tile picking and grid-aligned chrome the two
things that must change shape.

- **(a)** Picking becomes a **true inverse projection** — the cursor ray against
  the ground plane, exact at any angle — and the tile **grid overlay appears only
  while a build tool is active**, so a freely-rotated grid is never decoration.
- **(b)** Picking stays a screen-space approximation, and the grid is always
  drawn.

➡️ **(a).** (b) would put the click a tile or two off the moment the camera tilts,
which is the kind of bug that reads as "the game is broken" rather than "the
picking is approximate".

**✔ — overridden on the grid half.** Verbatim: *"exact ray picking, with an
intelligent system that always allows the user to see where they're on the grid"*.
Exact ray picking: settled. The grid policy is now a requirement — orientation must
be readable at every camera angle, always, and a grid that simply disappears
outside Build mode does not satisfy it. Round 10 asks what that system is made of.

## ❓ Q44b — (round 10) what "always see where you are on the grid" is

**✔ — pending**

## ❓ Q45 — How the runtime gets the renders

- **(a)** `assets/icons/*.png` + `assets/icons/manifest.json` + `tools/icons/*.py`,
  embedded with `include_bytes!` so the shipped runtime reads no files and cannot
  disagree with its manifest.
- **(b)** Load the PNGs from disk at runtime.

➡️ **(a).** It keeps the icons and their provenance in exactly one place, and
matches how the game already treats its own record as the only writable thing.

**✔ — embedded at build time.**

---

# Round 10 — closing round 9's reconciliations

## ❓ Q46 — Any 5.x is accepted; do we still write down which build made the file?

- **(a)** Record the build hash **always**, and never fail on a mismatch — so a
  re-render from a different build is *visible* in the manifest without becoming
  an error.
- **(b)** Record nothing, since any 5.x is accepted.

➡️ **(a).** It costs one field and it is the only thing that would let anyone,
later, tell a reviewed render from an unreviewed one.

**✔ — record the hash, never fail on it.**

## ❓ Q47 — A view that has no data yet: where does it live?

- **(a)** In the **registry** (visible in the Ledger as planned work) but **absent
  from the interface** until it has data — the design doc's own preference,
  absence over a disabled control.
- **(b)** In the interface, greyed out, with a "no data" tooltip.
- **(c)** In the interface, openable, showing an empty panel.

➡️ **(a).** (b) and (c) both put a control on screen that a player will read as a
capability; (a) puts it on the roadmap where it belongs.

**✔ — overridden: in the UI, openable, empty.** Recorded with the rule that makes
it liveable, because taken naively this is the one answer in the whole round that
brushes against §0.1. The rule:

> **An openable view with no data must say it has none, and name what will fill
> it. It must never render a zero that reads as a measurement.**

An empty panel that says *"power — nothing measured yet; this view fills when
service coverage lands in C2"* is honest and openable. An empty panel showing
`0 MW · 0 buildings · 0 connected` is a false measurement wearing a real number's
clothes, and it is refused. The full CS2 list becomes a visible roadmap without
becoming a wall of fabricated zeros.

## ❓ Q48 — What "always see where you are on the grid" is made of

- **(a)** Three indicators: a **persistent single-tile highlight** under the cursor
  in every mode; a **distance-faded local grid patch** that fades in when a build
  tool is active; and a fixed **axis/north indicator with tile coordinates** in a
  corner. Together they work at any tilt and rotation.
- **(b)** The whole-map grid drawn at all times.
- **(c)** A compass only.

➡️ **(a).** Each answers a different question — *which tile*, *which way is the
lattice*, *where am I* — and only drawing the full grid always (b) answers none of
them well at a tilted angle.

**✔ — tile highlight + local grid patch + axis and coordinate readout.**

## ❓ Q49 — The icon inventory and shipped sizes

- **(a)** One render per **surface** — the five tools, ledger, ticket, district,
  power, and each info view — at the sizes the type scale implies, each rendered
  at 4× and downsampled.
- **(b)** Icons for the toolbar only.

➡️ **(a)**, scoped by the rule the design doc already gives: an icon is a
**locator to a surface**, so the set is derived from surfaces that exist rather
than from a desired look. Sizes to confirm: **24 / 32 / 48 px**, matching title
(20) and display (26) on the type scale.

**✔ — one per surface, 24 / 32 / 48 px.** Authoring at 4× means rendering at
**96 px** and downsampling, so the shipped 48 is a shrinking rather than a guess.

## ❓ Q50 — What the presentation pass has to prove before simulation work resumes

- **(a)** Mechanical: no literal type sizes or off-scale spacing left; contrast,
  target sizes, icon provenance and icon material checks all running in a build
  check that **fails closed**; exact picking verified by a test under free tilt and
  rotation; UI scale at 100/125/150/200 %; and a playtest record with the player's
  own verdict attached.
- **(b)** It looks good enough; move on.

➡️ **(a).** It is the difference between "we improved the UI" and "the UI can no
longer silently regress", and the second one is the only version worth the round.

**✔ — mechanical checks that fail closed.**

---

## Round 10 at a glance

| # | Question | ➡️ | ✔ |
|---|---|---|---|
| Q46 | Record the build hash under any-5.x? | yes | ✔ |
| Q47 | Where a dataless view lives | registry only, absent from UI | **overridden: openable and empty**, with the no-fabricated-zeros rule |
| Q48 | What grid sense is made of | three coupled indicators | ✔ |
| Q49 | Icon inventory and sizes | one per surface, 24/32/48 | ✔ |
| Q50 | The pass gate | mechanical checks that fail closed | ✔ |

## Corrections ledger for C2's grilling

| Was | Now | Because |
|---|---|---|
| "No modes at all" (round 6, from a18) | **Modes return as info views**, and a18 is re-read as *the sim never gates the player* | The user asked for the doc's full §14 handling |
| §14.3 absence claim withdrawn (round 6) | **Reinstated**, with the openable-empty exception above | Modes exist again to scope against |
| Recommendation: 3 modes | **One mode per info view** | Same reason |
| Recommendation: fixed tilt, 90° rotation | **Free tilt and free rotation** | Same reason |
| Recommendation: pin the Blender build | **Any 5.x accepted**, hash recorded but never enforced | Same reason |
| Recommendation: 2-light rig | **3-light rig** (key, fill, rim), all recorded | Same reason |
| ".picking stays exact with a fixed grid" | Half wrong: **roads and buildings transform fine at any angle**; what needs rebuilding is click-to-tile resolution and grid-aligned chrome | The original claim was broader than it should have been |

## The settled presentation pass

Put back for sign-off, because it is what implementation follows:

1. **Camera.** Free tilt and free rotation, orthographic tilt-referenced projection
   with a depth buffer, buildings extruded one height step per level, contact
   shadows. Tile picking by **inverse projection against the ground plane**.
2. **Grid sense.** Persistent tile highlight under the cursor; a distance-faded
   local grid patch while a build tool is active; a fixed axis indicator with tile
   coordinates.
3. **Type and spacing.** `micro 12 · small 14 · body 16 · title 20 · display 26`,
   spacing on the 4-unit scale, targets ≥ 48 px, **no literal size at any call
   site**, nearest-filtered atlas, glyph positions snapped to whole pixels, UI
   scale 100/125/150/200 % with real re-layout.
4. **Icons.** Authored from an **OpenPBR parameter table** mapped onto Principled
   BSDF v2, rendered headless in **Blender 5.3 Alpha** (`5eaad57cfabe`,
   orthographic, fixed three-light rig, 96 px downsampled to 24/32/48), committed
   with a provenance manifest that records the build hash and the deviation from
   the spec, embedded with `include_bytes!`. `cargo build` never needs Blender.
5. **Info views.** The full CS2 list in the registry, one mode per view, each
   declaring its tool set; a view without data is **openable and honestly empty**
   — it names what will fill it and fabricates no numbers.
6. **Gate.** A build check that **fails closed**: type and spacing scales, contrast,
   target sizes, icon provenance, icon material checks, and a test proving picking
   is exact under free tilt and rotation.

The one thing still unmeasured after all of this: whether it is readable *to the
player*. That is what the playtest record is for, and it is the only check in the
list that no build can run.

---

# Built — the presentation pass, and what it was measured against

Items 1, 2, 3 and 6 of the settled list are **in the build**. Item 4 (the icon
pipeline) is the next task; item 5's info-view registry depends on the data
C2's systems bring.

## What changed, against what was measured

| Measured defect | What replaced it |
|---|---|
| 46 of 61 text call sites at **12 px** | a closed type scale — micro 12 · small 14 · body 16 · title 20 · display 26 — with `body` at 16 px, and `draw_step` taking a `Step` so a literal cannot reach a call site |
| six **off-scale size literals** (11, 12, 16, 24, 32, 40) | gone; a test reads the sources back and fails if the old constants reappear |
| glyphs **unsnapped**, atlas sampled with `Linear` | positions snapped to whole pixels; atlas sampled `Nearest`. The atlas is rasterised at the size it is drawn at, so there is nothing for a filter to interpolate |
| **no pitch, no rotation** | free orbit camera: free yaw, free tilt clamped to 20°–88°, orthographic projection, free zoom |
| flat rectangles for buildings | extruded boxes, one height step per level, faces culled against the camera and shaded by the fixed light, with contact shadows cast from that same light |
| a screen-rectangle tile guess | **inverse projection against the ground plane**, tested across 12 yaws × 7 pitches × 8 yaws × 4 pitches sweeps |
| one hovered-tile hint | tile highlight in every mode + distance-faded local grid patch while placing + projected north/east arms with a bearing, cursor coordinates and the interface scale |
| no enforced conformance | `design::verify`, which **refuses to start** on any defect and reports all of them |
| contrast asserted in prose | measured at startup: 14.24:1 body-on-panel, 17.02:1 on desk, 8.57:1 muted, 7.02:1 on ink |

## Evidence from the running build

```
design: ui scale 100% · 5 type steps · 6 spacing steps · 11 targets
design: type micro 12 → 12 px · small 14 → 14 · body 16 → 16 · title 20 → 20 · display 26 → 26
design: smallest target: 48 px (floor 24, comfortable 48)
design: style table: 32 tokens, 32 records, 0 with no record
renderer ready … depth=Depth32Float
first frame world_opaque=26167 world_overlay=49 interface=410 yaw=0° pitch=52° zoom=0.70
```

Tests: **57 library + 28 client, all passing**, 0 clippy warnings. The sweeps
that matter are the camera ones — `the_camera_round_trips_a_ground_point_at_any_
angle` checks 84 yaw/pitch combinations, and `a_tile_picks_back_the_tile_it_was_
drawn_at_at_any_angle` checks 32 — because "exact picking" is only a claim until
it has been tried at angles nobody wrote the test for.

## Three things the work turned up

1. **An orthographic camera has no horizon.** A test asserted that a ray at the
top of the screen picks nothing "into the sky". It failed: under orthographic
projection every screen point meets the ground plane, so the honest behaviour is
that the top of the screen is *far away*, not empty. The test was rewritten to
check that instead, and a second test now verifies the bearing claim against the
projection rather than trusting it.
2. **A leftover process locks the build.** `kill $PID` in Git Bash does not end
a Windows process; a stale `ala-cities.exe` was still holding the binary, which
is why one build failed with `os error 5`. `taskkill //F //IM` is the version
that actually stops it — and a run that is killed rather than closed still keeps
its record, because the capture flushes mid-session.
3. **The comfortable target floor has to scale too.** At 125 %, a 48 px target is
no longer 48 logical pixels. The check caught its own test doing exactly that,
which is the check earning its keep before it ever ran against the game.

## Still open, and honestly so

- **Whether it is legible to you.** Every mechanical cause of the old softness is
gone, but that is an argument, not a reading. This is the item no build can run.
- **The interface scale at 125/150/200 %** has been checked by test at each
scale, not by eye at each scale.
- **Frame timing under load** with 26 k world quads and a depth buffer is
unmeasured at 240 Hz output.
- **Composite contrast on a rendered frame** — token pairs are measured, pixels
are not.
- **Icons** — next task, per the round-9 answers.

---

## Answers

*Rounds 7 and 8 are settled above. Round 9 is pending, and nothing in it is
settled until its answers are filled in here.*
