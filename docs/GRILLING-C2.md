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

**✔ — one per surface**, and (round 11) the sizes are **24 / 32 / 48 / 64 / 96 px**.
Authoring at 4× means rendering at **192 px** and downsampling, so the shipped 48 is
a shrinking rather than a guess — and every shelves size divides 192 by a whole
number (192÷96=2, ÷64=3, ÷48=4, ÷32=6, ÷24=8), which is what makes "downsampled"
mean *box-filtered* rather than *resampled by luck*. **Correction:** this line said
96 px until round 11; the pipeline renders at 192.

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
   orthographic, fixed three-light rig, 192 px downsampled to 24/32/48/64/96), committed
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

The gate was then **made to fail on purpose** — a 20×20 target added to the
list — and the build refused to start:

```
the design check failed closed:
  - target `temporary probe` is 20×20 px, below the WCAG 2.5.8 floor of 24
fix these and restart; nothing will be drawn until the scales hold.
exit=1
```

That is the difference between a check and a decoration, and it is why the probe
was run rather than assumed: a gate that has never been seen to close is a
comment.

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

*Rounds 7–11 are settled above and in round 11 below. Where a paraphrase and a
verbatim answer disagree, the verbatim answer wins.*

---

# Round 11 — the icons as a library, and the materials they claim

Asked after checking the pipeline, the manifest, the spec and the HUD rather than
the plan. Three facts framed the round:

1. **The icons are not in the game.** `grep -rn "icons\|assets/icons\|manifest"
   src/*.rs` finds nothing but an unrelated `CARGO_MANIFEST_DIR`. There is **no view
   registry in Rust** either, so round 9's *stub the full CS2 view list* is recorded
   and unimplemented.
2. **Five icons borrow a state token as their identity colour** — `tool-demolish`
   (`Warning`+`Refused`), `ticket` (`CaseOpen`), `tool-inspect` (`NotObtained`),
   `power`/`view-power` (`Nature`, which `Token::Powered` derives from),
   `view-traffic` (`Agent`). §0.1's list opens with *"a plaque, a lamp, **a green
   state**"*.
3. **`Cargo.toml` declares `license = "Apache-2.0"` and there is no `LICENSE`
   file**, and §8.2's gate names CC0/PD/CC BY(-SA) only — Apache-2.0 is absent from
   that list. The manifest has no `artist` or `licence` field at all.

## ❓ Q45 — Are icons "primary surfaces"?

**✔ (verbatim)** — *"Icons are for the hud, but we can also use the blender material
renders for more things. Maybe making a library of icons based on the unified design
doc and other grilling sessions that we can go back to for other things. Grill me
more if you need"*

**Read as:** icons are HUD locator chrome (so §3.1's *named solid fallback* is the
right description of them), **and** the material renders are a reusable **library**
with more consumers than the HUD. Consequence recorded: the icons **depict**
materials — the ledger is metal, the ticket is paper — while the interface **applies**
none. Those are different claims and the manifest must not blur them. The lineage
table bans the alternative anyway: *"anti-slop bans; real textures over pure
generative fill"* and *"Generative texture alone is not sufficient without a
human-selected physical reference"*, so noise textures written to satisfy the word
"texture" would be the exact anti-pattern.

## ❓ Q46 — May an icon's colour mean *state*?

**✔ (verbatim)** — *"I don't mind animating the icons if we have to - if it makes
the ui more responsive to the user. Ask in feedback sections for playtests about
this"*

**Read as:** the question was about colour and the answer is about motion, so both
are recorded and the colour half stays open in round 12. Recorded as settled: icon
motion is **permitted when it makes the UI more responsive**, and the playtest
feedback form must **ask about it** — that question is now a required field, not an
optional one.

## ❓ Q47 — Size, and where an icon may sit

**✔ (verbatim)** — *"Probably your best guess, biggest icon that reasonably fits
scaled down into wherever it goes. Grill me for more if you have to"*

**Read as:** the glyph fills the tile as far as it reasonably can, and the source is
scaled **down**, never up. The delegation is taken, and round 12 makes it mechanical
rather than a judgement: a glyph size derived from the spacing scale, snapped to the
shipped ladder, with a measured floor and a declared host surface per icon.

## ❓ Q48 — The size ladder

**✔ (verbatim)** — *"Ship/24/32/48/64/96"*

Settled. The reason the ladder matters: `design::UiScale` reaches **200 %**, where
`design::target()` is **96 px** — so the old 24/32/48 set would upscale a 48 px icon
2×, reintroducing in icons exactly the softness that was just removed from text.

## ❓ Q49 — More icons, under what constraint

**✔ (verbatim)** — *"We can make more icons it's okay just as long as they follow
unified design and the three fixed sources of light & openpbr schemas we discussed.
Grill me more if you want"*

**Read as:** the set may grow, and the constraint is not taste but **three fixed
sources of light + the OpenPBR schema + the design doc**. That makes the rig a
*contract*, not a setting: an icon whose legibility needs a different rig is
**refused** rather than special-cased, and the refusal is recorded.

## ❓ Q50 — The licence on the renders

**✔ (verbatim)** — *"These are rendered through my machine - they belong to me. The
unified design spec is ambiguous"*

**Read as:** authorship is the user's, and the spec's gate is silent on first-party
assets. Recorded as a **stated ambiguity**, resolved in round 12 by choosing an
option the gate actually recognises rather than by leaning on its silence.

## ❓ Q51 — Assets or no assets

**✔ (verbatim)** — *"Hopefully the game can be played without assets.. but the
pipeline I have in mind later will require assets anyways, since we'll want
buildings- that will require whole 3D models"*

**Read as:** two requirements, in this order. The game must stay **playable with
zero assets** (the current procedural extrusions are not a placeholder to be
replaced — they are the named fallback), and a **model pipeline is coming** for
buildings. That is the first real second consumer of the asset machinery, so round
12 asks whether the provenance schema is shared now or duplicated later.

## ❓ Q52 — Does a count live on an icon?

**✔ (verbatim)** — *"Not baked into the icon, it would have to be it's own text slot
that lives as a popit/ios6 notification. grill me more about unified design"*

Settled: **never baked**. And the doc has an exact row for the rest of it — §12.9's
SpringBoard mapping:

| iOS 6 element | Product meaning | Forbidden reading |
|---|---|---|
| Badge | **Count or state copied from store** | Trust grade or promotion |

So a badge is *copied*, never computed, and its forbidden reading is a grade. Round
12 asks which surface each count belongs to, because §12.7's Notification Center and
a Popit page are different objects with different authority: one aggregates events,
the other describes a record.

## Round 11 at a glance

| # | Question | ➡️ | ✔ |
|---|---|---|---|
| Q45 | Are icons primary surfaces? | locator chrome, named solids | **icons are HUD chrome, and the renders become a reusable library** |
| Q46 | May icon colour mean state? | constant per surface; states additive | **motion permitted for responsiveness; playtest must ask; colour half held over** |
| Q47 | Size and placement | declared host, measured contrast | **biggest glyph that reasonably fits, scaled down — made mechanical in round 12** |
| Q48 | Size ladder | 24/32/48/64/96 | ✔ |
| Q49 | More icons? | one per declared surface | **yes, bound to the design doc + the three lights + the OpenPBR schema** |
| Q50 | Licence on the renders | CC0 or CC BY with named author | **user owns them; spec ambiguous; resolved in round 12** |
| Q51 | Assets or no assets | assets never required | **playable without assets; a 3D model pipeline is coming for buildings** |
| Q52 | Counts on icons | live text slot, never baked | ✔ **as a §12.9 badge: count copied from store** |

## Corrections fixed with this round

| Was | Now | Why |
|---|---|---|
| "rendered at 96 px, downsampled to 24/32/48" | **rendered at 192 px, shipped at 24/32/48/64/96** | `rig.py` says `RENDER_PX = 192`; the doc said 96 |
| Q43b marked *pending* at line 379 | **answered** (line 429) | a doc that says pending next to an answered question teaches a reader to distrust the rest |

---

# Round 12 — open

*Nothing here is settled until its answer is written above the heading.*

## ❓ Q53 — What the library is *for*

**(a)** An internal asset source, structured so publishing is a copy rather than a
rewrite: one directory, one manifest, one provenance schema, machine-readable.
**(b)** A shippable design-system artifact now, with a public catalogue and its own
version. **(c)** Icons only; no other asset family ever joins it.

## ❓ Q54 — The contract for adding an icon

What must a new icon declare, and where is it written? Proposal: `id`, the
**surface it locates** (which must already exist in the registry), its **role token**
(identity, never state), the **hosts** it may sit on, its builder, its OpenPBR
parameters, its framed-or-not, and its sizes. Refusals: an icon needing a second rig,
an icon for a surface the game does not declare, an icon whose identity colour is a
state token.

## ❓ Q55 — How an icon moves

Quad-level transform in the renderer (no new assets) versus pre-rendered frame
sequences (a new asset family, N× the files). Plus the §4/§8.4 collision: *"Reduced
motion is the default"* would turn off the responsiveness that motivated motion.

## ❓ Q56 — "Biggest that reasonably fits", made mechanical

Glyph = tile − 2×`Space::Sm`, snapped to the shipped ladder, never upscaled; the
drawn glyph never below 24 px; contrast measured against **each declared host** and
failing closed at 3:1 (§6.2, §1.4.11).

## ❓ Q57 — The licence string, and the hollow declaration

Which licence goes in the provenance record, under which author name, and does the
repo get a `LICENSE` file — or does `Cargo.toml` stop declaring Apache-2.0?

## ❓ Q58 — Where a count lives, and what it may look like

§12.9's badge row says *copied from store*; §7.3 says refused must never look like
success. Does a zero state show anything at all?

## ❓ Q59 — The coming model pipeline, and the schema it needs

One provenance schema for every asset family, or one per family? Does the named
fallback stay permanent? Are building models generated parametrically like the
icons, or authored?

---

# Round 12 — recorded

## Answers, verbatim

| # | Answer |
|---|---|
| a53 | *"a + c"* |
| a54 | *"I do not honestly care. I just want to see a lot of neat icons rendered throught the mathemetical constraints"* |
| a55 | *"a+b, the rendering system itself should support advanced raster image features where necessary (like in the ios 6 or popit unified design spec) as well as animated icons themselves (like gears turning, reminiscent of ps2 save icons) The ui should be as lively as the world if not more."* |
| a56 | *"When I playtest it I will let you know if an icon is too big. Better yet, while a playtest is open, run a live debugger at the same time that analyzes the data output from the program, and let me log ui elements with comments"* |
| a57 | *"I don't care, this should be publicly accessible and playable through github so cc-by works for me"* |
| a58 | *"your best guess from the documentations"* |
| a59 | *"The models will also be mostly procedural made the same way the icons should be, so aside from the mechanical, mathematial, and unified design & grilling design docs we've discussed, we'll have to work through the models just like for the icons. Grill me for more"* |

## What each answer settles

**a53 — (a) + (c).** The library is **internal**, structured so publishing is a copy
rather than a rewrite, and it stays **icons only**. Models are therefore a *separate
family with its own library* rather than an extension of this one — which a59
confirms by giving them their own grilling round.

**a54 — delegated, and the requirement is volume.** *"A lot of neat icons rendered
throught the mathemetical constraints."* So the contract written in `CONTRACT.md`
changes purpose: it is what makes volume **safe**, not what makes it slow. The
constraints are named in the answer itself — the fixed three-light rig, the OpenPBR
parameter table, the design tokens, and (from Q48/Q56) integer-divisible sizes,
coverage bounds and per-host contrast.

**a55 — (a) + (b), and a live UI.** Quad transforms *and* pre-rendered animation
frames; the renderer must gain real raster features "where necessary"; animated
icons in the **PS2 memory-card tradition** (the answer's own example is gears
turning); and *"the ui should be as lively as the world if not more"*. This
**collides with §4 and §8.4**, which both say *"reduced motion is the default"*. The
collision is recorded as **open** rather than resolved quietly, because the
motion-forward reading and the doc's default cannot both be first: round 13 asks it
once, sharply.

**a56 — the playtest harness grows a debugger.** A live process that consumes the
program's own data output during a playtest, plus **UI element logging with
comments** from the user. Two consequences recorded now: an annotation must bind to
a stable **element identity** (a registry), not to a pixel; and an annotation is a
**lesson with tentative confidence**, never a claim — the shape C1's Q27 already
set for feedback, with build identity and session evidence attached.

**a57 — public and playable through GitHub, CC BY accepted.** The licence decision
falls to me: a `LICENSE` file for the code (the declaration in `Cargo.toml` is
currently unbacked), and **CC BY 4.0 with a named author** for the renders. §8.2's
*"UI copy that refers to assets states what is true"* becomes a user-visible
obligation, because strangers will read it.

**a58 — delegated.** Taken as recommended: a §12.9 badge (count copied from store),
non-zero only, opening the Popit page; the Notification Center carries **events**,
never rollups; a zero shows nothing; and a badge never recolours the glyph.

**a59 — models are procedural, and they get their own round.** Settled now: the
named fallback stays **permanent** (playable with zero assets is a property, not a
coincidence), the **provenance shape is shared** even though the libraries are not,
the **icon rig does not apply** to world-lit models, and the model campaign opens
with its own grilling the way the icons did.

## A reconciliation recorded while writing this round

§12.1 requires grid contents to be **"loaded, not compiled into the binary"**;
round 9 settled that the icons are **embedded with `include_bytes!`** so `cargo
build` never needs Blender. Both hold at once: the **manifest is the loaded data**
(§5.1's version + digest, with the UI able to refuse a mismatch), the **bytes are
embedded** as the shipping copy, and a dev/playtest mode may point at
`assets/icons/` on disk instead — which is also what lets a re-render appear in a
running playtest without a rebuild.

---

# Round 13 — open

*Nothing here is settled until its answer is written above the heading.*

## ❓ Q60 — The reduced-motion default, which a55 contradicts

§4: *"Reduced motion is the default"*. §8.4: *"Default is reduced."* a55: *"the ui
should be as lively as the world if not more"*. Either the doc's default governs
chrome (and the first launch is still, with liveliness opt-in) or it is corrected
for this product (motion is the default, reduced is reachable from chrome).

## ❓ Q61 — Which "advanced raster image features" the renderer gains

Named features to accept or refuse: atlas sub-rects with a transparent gutter
(neareast-filtered sampling without bleeding); **nine-slice** stretchable plates so
"a plaque is metal" (§3.1, TouchWiz lineage) survives any size; a multiply/tint path;
a **soft-glow or neon rim layer** (§3.2 names the Popit rim as "flavour only; never
authority"); and multi-frame sprite animation with a declared cadence.

## ❓ Q62 — What animates, and what drives it

Which icons animate, at which sizes, how many frames, loop-seamless or transition;
and the rule that animation play state is **read from the store** (a gear turning
must mean work is actually happening, per §12.9's "state copied from store") rather
than being ambient decoration.

## ❓ Q63 — The inventory: how many icons, from which list

"A lot" needs a source of truth. Does the **registry come first** in Rust (from the
full CS2 view list plus tools, records and chrome objects) with the generator reading
it, and how do variants multiply (per zone type, per service, per state mark, per
density)?

## ❓ Q64 — The playtest debugger and the element logger

In-game overlay versus a separate process; how an element gets a stable identity; how
an annotation is written, keyed and later read; and the bound that it **never writes
to the sim** and never becomes an authority surface.

## ❓ Q65 — What "playable through GitHub" means mechanically

(a) Releases with prebuilt binaries per stage tag; (b) a wasm/WebGPU build on Pages,
playable in a browser; (c) clone and `cargo run`. Plus where the licence and
attribution surfaces live, given §8.2's no-stale-counts rule.

## ❓ Q66 — The model campaign

When it starts relative to the icon library, what the shared provenance shape is, and
what the first deliverable of its grilling round must be.

---

# Round 13 — recorded

## Answers, verbatim

| # | Answer |
|---|---|
| a60 | *"We're aiming for something that looks good according to the pillars of design in unified design. Impeccable, md1, early touchwiz, ios 6, and the popit. My word over the document where it matters"* |
| a61 | *"All five"* |
| a62 | *(the recommendation quoted back verbatim)* **+** *"Just make sure that blocked threads don't block animations where it wouldn't indicate anything of use to the user"* |
| a63 | *"Your best bet, with a list of icons from the relevant design specs to recreate within the new system"* |
| a64 | *"Some of the playtests you would open would record as no user interacted with. As long as the debugger tools aren't just front and center to the user it's good, but with some kind of contextual action to report something is wrong. A side by side debugger for the agent would be nice"* |
| a65 | *"Push the project to github with release packages and a readme that extensively documents and covers the project"* |
| a66 | *(the recommendation quoted back verbatim)* |

## What each answer settles

**a60 — the quality bar is the pillars, and the user's word outranks the doc.** The
five pillars named are the four lineages of §1 (TouchWiz, MD1, Apple HIG of the iOS 6
era, Impeccable) plus Popit as a pattern library. The override is recorded as a
**governance fact**: where the document and the user disagree about what makes the
game good, the user's word wins. Round 14 defines the override's scope once, so it is
not relitigated per question, and asks whether reduced motion survives — because
accessibility and taste are different kinds of requirement.

**a61 — all five raster features.** Atlas sub-rects with a gutter, nine-slice plates,
tint/multiply, a glow/rim layer, and multi-frame animation. Recorded with the note
that only the last two change the renderer's data model.

**a62 — animation stays store-driven, and gains a thread-independence rule.**
The addition is the one that matters: *blocked threads must not block animations that
would have indicated something useful.* A second rule follows from the user's own
words — where an animation indicates *nothing* of use, it does not run — which is also
the honest reading of §4's *"motion provides meaning, not noise"*. And a measured
hazard lands squarely here: the world snapshot is **4.49 MB written in 30.7 ms**, so
at 240 Hz the autosave drops roughly seven frames — visible in any animation running
when it fires.

**a63 — the inventory gains a recreation list, and collides with §1.** The user asked
for *"a list of icons from the relevant design specs to recreate"*. §1's TouchWiz row
lists **"icon cloning"** among what is **not adopted**, and §0.5's iOS 6 row lists
**"brand replication"** the same way. The collision is real and is put to the user in
round 14 rather than solved quietly in the generator.

**a64 — the debugger is deliberately off-centre, and gets a side-by-side pane.**
Three requirements: not front and centre; a **contextual** action to report something
wrong rather than a permanent form; and a **side-by-side agent pane**. Plus the
honesty rule about unattended sessions, which this project has already got wrong once
— C1's first draft claimed *nobody clicked anything* and the capture disproved it.

**a65 — publish to GitHub with release packages and a real README.** Facts checked
rather than assumed: `gh` 2.100.0 is installed and authenticated as **`alacripose`**
with `repo` and `workflow` scopes; there is **no remote**; `main` carries one tag
(`c1-sim-core`); `target/release/ala-cities.exe` is 8.7 MB and `verify.exe` 708 KB;
the README is **130 lines**, so *"extensively documents"* means a rewrite. The push
itself is explicitly authorised.

**a66 — models after the icons, as recommended**, with the named fallback permanent,
the provenance shape shared, the libraries separate, and the icon rig not applying to
world-lit buildings.

## Two collisions recorded, both answered in round 14

| Collision | Where it lands |
|---|---|
| *"recreate icons from the design specs"* vs §1's **"icon cloning"** and §0.5's **"brand replication"** | Q70 |
| *"as lively as the world if not more"* vs iOS 6's **"motion for decoration"** and §4/§8.4's **reduced-motion default** | Q67, Q68 |

---

# Round 14 — open

## ❓ Q67 — The five pillars, ranked, and how "looks good" becomes judgeable

What wins when the pillars disagree, and what makes the quality bar traceable rather
than a matter of taste at review time?

## ❓ Q68 — The override's scope, and whether reduced motion survives

*"My word over the document where it matters"* — matters per decision, or generally?
And does reduced motion still exist as a chrome-reachable preference even though
liveliness is now the default?

## ❓ Q69 — Animation honesty, and the measured save hitch

Meaningful animation must survive blocked threads; animation must not keep implying
progress when work is blocked; and the autosave writes 4.49 MB / 30.7 ms on the frame
path.

## ❓ Q70 — The recreation line

Semantics and grammar recreated with original geometry, or the specifications' artwork
reproduced? And what is the refusal list?

## ❓ Q71 — The debugger's shape

Split pane in the same window versus a second window; what the agent pane shows; what
"contextual" means for reporting something wrong; and how an unattended session is
recorded.

## ❓ Q72 — GitHub mechanics

Repository name and visibility; what a release package contains; tag and release
naming; and what the rewritten README has to cover.

---

# Round 14 — recorded

## Answers, verbatim

| # | Answer |
|---|---|
| a67 | *"exactly what you said- and this applies to the icons, materials in buildings etc."* |
| a68 | *(the per-decision paragraph quoted back verbatim)* |
| a69 | *"all three of these options sound right together. Simple code sounds like a death knell for this advanced project"* |
| a70 | *(the semantics-and-grammar answer quoted back verbatim)* |
| a71 | *"Your best guess. There should be a secondary window that informs me of the agent's presence and if they've begun work on my ticketed items, my ticketed items from every playtest, and wether those items are closed or in another state. The debugger itself should be an extension of the design framework and allow the user to report if itself is not working. There should be detailed information about the logging happening when the debugger view is open, playtest time left, build release information, and git tracking built in to the debugger"* |
| a72 | *"your best guess"* |

## What each answer settles

**a67 — the pillar ranking and its rubric apply to icons, building materials, and
everything after them.** The rubric is not an icon document; it is the quality bar.

**a68 — the override is per decision.** Your word wins where given; where it is not,
the document stands until it is corrected on the record. Recorded so this is never
relitigated. One part of my answer was *not* quoted back and is therefore recorded as
my call rather than yours, reversible on request: **reduced motion survives** as a
chrome-reachable preference, because it is accessibility rather than taste (§6.1's
floor sits above the pillars). Motion is still the default.

**a69 — all three animation rules stand together, and a standing value is set:**
*"Simple code sounds like a death knell for this advanced project."* This corrects my
own bias on the record. Where a capability is real, complexity is paid for; the
doctrine's anti-over-engineering instinct is subordinate to the user's word per a68.
The three rules: chrome animates on its own clock; store-driven animation stops when
its store field stops advancing; the autosave leaves the frame path (4.49 MB / 30.7 ms
on the frame path is ~7 dropped frames at 240 Hz).

**a70 — semantics and grammar only.** Original geometry on our rig with our tokens; a
named **refusal list** in the manifest for brand marks and trade dress.

**a71 — the debugger becomes a session console, and grows a second window.** Six
requirements, one of which is a doctrine hazard:

1. a **secondary window** informing the user of the **agent's presence**;
2. whether the agent has **begun work** on the user's ticketed items;
3. those items **from every playtest**, and their state;
4. the debugger as an **extension of the design framework**, able to **report its own
   malfunction**;
5. detailed information while open: **what is being logged**, **playtest time left**,
   **build release information**;
6. **git tracking built in**.

The hazard: *"the agent's presence"* and *"has begun work"* are claims about reality,
and §0.1 forbids a surface that looks more authorised than it is. A green lamp that
says an agent is working when nothing is happening is exactly the defect class this
project keeps catching. It is put to the user in round 15 with the mechanism that
makes it honest: **the window reports what a record says, with the age of that
record, and degrades to Unknown when the record goes quiet.**

**a72 — delegated.** Taken as recommended: `ala-cities`, public, one release per
stage tag marked pre-release, README rewritten. The push is authorised.

## Standing item recorded, not asked

**Controls for desktop, mobile and controller** — *"Grill me later about controls for
desktop, mobile and controller."* Recorded as a **named future round**, with the one
structural thing taken now under a72's delegation: input is written as an
**action layer** (actions bound to inputs) rather than three input paths bolted on
later, because mobile implies touch and controller implies focus navigation with no
cursor (§5.4, §2.1), and both would otherwise arrive as rewrites.

---

# Round 15 — open

## ❓ Q73 — What an "agent presence" indicator may claim, and what it reads

A file the agent writes (a work ledger with ticket id + heartbeat) plus git state as
corroboration, versus inferring presence from commits, versus a live connection. And
the rule that presence is timestamped and **self-expiring** (Working → Idle → Unknown)
rather than a lamp that stays green.

## ❓ Q74 — Ticket lifecycle across playtests

Where playtest-filed tickets are stored, how they are grouped by playtest, the state
vocabulary shown, and who may set `in_progress` — the agent's ledger, or anything else.

## ❓ Q75 — Playtest time left

Who declares a session's duration, whether the clock pauses with the game, and what
happens at zero.

## ❓ Q76 — Build identity and git tracking

What is displayed, whether a **dirty** tree disqualifies a build from release
labelling, and the rule that the game only ever runs read-only git commands.

## ❓ Q77 — The debugger as a design-framework extension

Whether the console uses the player's UI scale or its own density, how its elements
register, and what its self-report path writes.

## ❓ Q78 — Where the secondary window lives

A genuine second OS window (a second surface and swapchain in the renderer), an
always-on-top overlay, or a pane — and what happens on a single display.

---

# Round 15 — recorded

## Answers, verbatim

| # | Answer |
|---|---|
| a73 | *"you are currently devloping this - you are the agent, so you must do that work when the user uses the debugging panel"* |
| a74 | *(the recommendation quoted back verbatim)* |
| a75 | *"the agent determines playstests during development, which don't happen when the user is actually playing the game."* |
| a76 | *"your best guess"* |
| a77 | *"exactly what you said"* |
| a78 | *"a pane that lives horizontally next to the main window in windowed mode that can be accessed from the pause menu or a dedicated function key for debugging"* |

## What each answer settles

**a73 — the agent in the console is me, and the work is real.** *"you are currently
 devloping this - you are the agent, so you must do that work when the user uses the
debugging panel"*. So the console's presence display is not a metaphor and not a
mock: the ledger it reads is the record of **this** agent's actual work, and it is my
job to keep it truthful. A figure that says the agent is working while nothing is
happening is the defect class this project keeps catching, so the honest mechanism is
settled: **the console reports what a record says, with the age of that record, and
degrades to Unknown when the record goes quiet.**

**a74 — one store, `origin: sim | playtest`.** Sharing an id space is fine; sharing a
meaning is not. States: `filed → acknowledged → in_progress → blocked / closed /
retired / superseded`, and **only the agent's ledger may set `in_progress`**.

**a75 — playtests are the agent's, and they do not overlap the user playing.**
*"the agent determines playstests during development, which don't happen when the
user is actually playing the game."* This splits two things that had been blurring
into one: a **playtest** is a development activity with a declared script and
duration, run by the agent; **the user playing** is not a playtest, is untimed, and
is still the primary feedback path. Consequence for the console: *playtest time left*
shows a countdown only while a playtest is running, and otherwise shows a **named
absence**, never a zero.

**a76, a77 — taken as recommended.** Build identity displays commit, branch, tag, a
dirty flag and the files changed since the session started; a **dirty tree
disqualifies a build from release labelling**; git is read-only from inside the game.
The console is a design-framework extension: same tokens, same scales, the player's
UI scale, elements registered in the same registry, and a **fault annotation** path
for reporting the debugger itself being wrong.

**a78 — not a second OS window: a horizontal pane beside the main view.** Available in
windowed mode, reachable from the pause menu or a dedicated function key. Kept from
the recommendation as the fallback shape: when the pane opens, the world viewport
narrows, so the picking sweeps have to cover the narrowed aspect as well as the full
one.

---

# Round 16 — open

## ❓ Q79 — What an agent-run playtest *is*, mechanically

A declared launch (stage + duration) driven by **scripted, deterministic input**, so
the run is reproducible from a seed — versus the agent launching the build and only
observing instrumented output. And what the console shows when no playtest is running.

## ❓ Q80 — Replay from capture

User sessions already capture every interaction. Does that capture become a
**replayable** re-run now — so an annotation or a bug report becomes a reproducible
repro the agent can step through — or does it wait for the deferred observer-mode
round?

## ❓ Q81 — The ledger as the interface between us

What fields it carries, what it may claim, who may write it, and what the console
shows when no agent is attached at all.

## ❓ Q82 — Whether the console shows the agent's *plan*, and whether the user replies there

A work-queue view (claimed, working, blocked, closed with evidence) versus a presence
light only — and whether design conversation leaves the game entirely for the docs.
