# Grilling — C4: what makes six candidates six readings

C4 exists because the picker showed six candidates per icon and the user could see
that they were not six readings. The complaint was specific: the TouchWiz-led and
iOS 6-led variants "only slightly change the position of a couple of circles on the
icon". The pipeline had a model of *what an icon is* (C3, Q78) but no model of what
its six candidates are *for*.

The complaint was correct, and measuring it turned up two defects — one in the
model and one in the code — plus a structural fact about the three reference
archives that changed the shape of the answer twice.

**Convention, as in `GRILLING.md`, `GRILLING-C2.md` and `GRILLING-C3.md`:** `➡️` is
the recommendation made. `✔` is the answer received. Where an answer **overrode** a
recommendation it says so. Question numbers continue C3's sequence; C3 ended at Q98.

**Note on labels.** Rounds 21–23 were asked under short labels in the conversation
(`Q30`–`Q41`), because the session had moved to a new subject and the numbering was
restarted by mistake. They are recorded here as **Q100–Q111** so the sequence stays
continuous, and the mapping is one-to-one and in order.

---

## What prompted it

Before the first question, three measurements — the user's complaint, the model,
and a defect nobody had asked about:

| Claim under test | Measurement | Verdict |
|---|---|---|
| "the variants only slightly change a couple of circles" | `composition_materials(tier, hues)` is called once per **icon**, not per candidate | Confirmed — all six candidates of `tool-road` carry the identical material set `{accent: glass, ink: paper, secondary: polymer, silhouette: metal}` at identical values |
| `EMPHASES`'s own docstring — *"each emphasis leads with one family's contribution and restrains the other two"* | Nothing leads: the emphasis only appends geometry (`touchwiz` one ring, `ios6` a ring and a bead) | **False in code** |
| how far apart the six actually are | fraction of pixels differing between candidate pairs at 96 px: `ticket` 42.8% mean / 38.4% min; `tool-road` 27.2 / 18.0; `tool-inspect` 20.4 / 13.1; `vocab-settings` 9.6 / **5.1**; `tool-demolish` 9.2 / **3.7** | Two pairs differ by ~4% of pixels: rounding, not a reading |
| the declared per-icon hues | `tool-road` blue, `tool-power` amber, `tool-inspect` cyan, `tool-demolish` red — all four render the identical `[0.4288, 0.4824, 0.5442]` | **Defect: the declared hue never reaches a pixel** |

The hue defect's mechanism: `build_materials` copies the family's material out of
the blend and then called `openpbr.from_blender_material`, which reads `Base Color`
**out of that material** and only falls back to the declared colour when the socket
is missing. So the blend owned the colour as well as the surface, and the matrix's
hue was recorded as the declared colour while the render used the blend's. Every
family member rendered the same grey whatever it declared. This is the silent-drop
defect class `UNIFIED_DESIGN.md` §8.1 names.

---

# Round 20b — the metal body, reopened

## ❓ Q99 — How should the metal surface be resolved?

Metal is `#afb9c3` at metalness 0.9 under a black world. **(a)** lower the metalness
in the blend so the family is a painted/brushed body with a diffuse term; **(b)**
stop the world being black, so a mirror has radiance to reflect; **(c)** change the
tools' lead family to a bright dielectric; **(d)** leave it and ship nothing for the
tools.

➡️ **(a)**, on the grounds that the reference pack's own metal reads as a bright
painted surface (median V 0.69–0.92), and that a black world is load-bearing.

**✔ (verbatim):**

> Give the world something to reflect

**Override.** The recommendation was (a); the answer is (b). Recorded because the
reason matters: a black world makes a mirror's body unreadable, and the user chose
to give the world radiance rather than repaint the metal.

**Measured before implementing.** Flat swatch of each family at the decision size,
grey world at strength 1.0, read back through the same `measure()` the checks use:

| family | v=0.00 | v=0.10 | v=0.20 | v=0.30 | v=0.40 | v=0.60 |
|---|---|---|---|---|---|---|
| metal | 0.030 | 0.076 | 0.123 | 0.169 | 0.216 | 0.307 |
| paper | 0.569 | 0.625 | 0.683 | 0.739 | 0.794 | 0.892 |
| ceramic | 0.325 | 0.386 | 0.449 | 0.509 | 0.571 | 0.694 |
| glass | 0.096 | 0.134 | 0.172 | 0.210 | 0.249 | 0.326 |
| polymer | 0.034 | 0.039 | 0.043 | 0.048 | 0.052 | 0.061 |
| road | 0.039 | 0.046 | 0.052 | 0.059 | 0.066 | 0.079 |

Two facts turn this table into a decision. Every icon declares **both** `Panel` and
`PanelRaised`, and the floor is set by the *brighter* host, not the darker one —
`PanelRaised` has luminance 0.0298 against `Panel`'s 0.0156, so the ink needs
p75 ≥ **0.1894** there. Metal crosses that between 0.30 and 0.40. And by 0.40 the
world has lifted `paper` to 0.794, where it begins to clip.

**Settled.** Grey world at strength **0.40** — 3.33:1 on PanelRaised, 4.06:1 on
Panel. `rig.py` reads `icon_ref:world` from the blend (blend wins, constant is the
fallback), and the declared `black` claim is gone from `RIG_INFO`.

**Consequences, all recorded rather than hidden:**

1. **`clip_fraction` becomes a check.** A clipped highlight is detail that never
   reaches the shipped file, and a brighter world can produce one. Measured per
   size and compared against a declared ceiling.
2. **Q71(a) is retired.** The near-metal swatch skip existed because a black world
   made a metal swatch a reading of the rig rather than of the material. With the
   world lit, the same swatch measures the material again (metal lands at ~0.45,
   inside the existing band), so the skip's premise is gone and a rule whose premise
   is gone would hide the next real metal failure.
3. **The blend carries six of seven families.** `enamel` has no
   `icon_ref:material:enamel`, so its surface has been coming from the Python
   fallback while the manifest implied the reference. `family_coverage()` could not
   see this because it compares two dicts in code. The fallback is now recorded per
   material as `surface_source`.

---

# Round 21 — what the six slots are for

## ❓ Q100 — What is the six-slot model asked to vary?

a9 fixes each pillar to a *role* — MD1 owns the silhouette always, TouchWiz owns
colour and surface always, iOS 6 owns the accent always — so "which pillar leads"
has no compositional degree of freedom left. **(a)** six colourways; **(b)** six
constructions; **(c)** keep the emphasis axis as a declared, measured balance; **(d)**
make the axis the accent form, the only form a9 leaves free.

➡️ **(c) with (d) folded in** — keeps the six slots, keeps a9 intact, and turns the
complaint into a rule: two candidates whose measured balances are within a few
percent are not two readings and should not both be shown.

**✔ (verbatim):**

> c with d folded in

## ❓ Q101 — Can a candidate ever be TouchWiz-led in *form*, under a9?

If the silhouette is MD1's in every slot, a "material-led" candidate can only differ
in surface and balance. **(a)** a9 stands and the emphasis is surface + balance +
construction; **(b)** amend a9 so a TouchWiz-led candidate draws its mark from a
TouchWiz object; **(c)** drop the emphasis labels.

➡️ **(a)**, with the cost stated: under (a) no candidate is recognisably TouchWiz,
because the pack's identity is partly its shapes.

**✔ (verbatim):**

> the shape builder is supposed to provide variants, why not use all three design
> languages as a corpus for it to interpolate from

**Override, and the second reframing of the session.** Neither (a) nor (b) as
posed: the answer moves the question to a *corpus* model, where the three languages
are not three voices competing for the lead but one space the shape builder
interpolates. That is what Q104–Q107 then had to make concrete.

## ❓ Q102 — How much must a declared hue actually show?

**(a)** raise each family's variant chroma until the hue is identifiable, and
re-check every host floor; **(b)** keep near-neutral metal for the silhouette and
let hue ride the secondary and accent; **(c)** record hue as intent and drop the
claim that a silhouette *is* blue.

➡️ **(b)** — a blue mirror is neither blue nor a mirror, and raising metal's chroma
until it reads is how you get grey icons failing the contrast floor again by another
route.

**✔ (verbatim):**

> b

## ❓ Q103 — Is "conformant to its pillar" a check or a label?

**(a)** build the three touchstones plus a minimum separation between slots; **(b)**
keep the labels and let the eye be the only judge; **(c)** build only the separation
floor.

➡️ **(a) with (c) inside it** — (c) alone stops the collapse without saying what each
slot is for; (a) alone lets six candidates each be nominally conformant while
remaining near-identical.

**✔ (verbatim):**

> a with c

---

# Round 22 — the corpus

## The measurement that reframed it

Before asking, each language was sampled from disk at 96 px: 126 MD1 marks, 125
TouchWiz drawables, 113 iOS 6 files.

| language | pieces | holes | fill (mark ÷ own box) | gloss | chroma |
|---|---|---|---|---|---|
| MD1 `baseline` marks | 2.6 | 1.2 | 0.525 | 0.000 | 0.000 |
| TouchWiz (galaxy pack) | 1.6 | 0.1 | 0.865 | 0.063 | 0.387 |
| iOS 6 | 1.0 | 0.0 | 0.995 | 0.137 | 0.456 |

Fill, gloss and chroma all rise MD1 → TouchWiz → iOS 6; complexity falls. Nothing
is non-monotone. So the three languages are **one ordered ladder**, not three
independent voices — which means interpolating them buys a single scalar, and the
freedom to make six *different* readings has to come from somewhere the corpus does
not order.

**Two caveats recorded against the measurement itself.** MD1's row is achromatic and
gloss-free *by construction*, because the sample is the `baseline_black` marks that
a18(a) adopted; MD1's coloured product icons would move that row, and that is the
agent's sampling choice, not a fact about Material Design. And because the anchors
are collinear, the obvious way to get six slots from three languages — three
vertices plus three midpoints of a ternary simplex — puts the TouchWiz vertex and
the MD1↔iOS 6 midpoint at the *same* point. Two of six slots would have been the
same icon again, which is precisely the failure under discussion.

## ❓ Q104 — Which axes does the shape builder interpolate, and where do the anchors come from?

**(a)** fill + gloss + complexity, anchors measured from the corpora with population
and method recorded, chroma left to the palette; **(b)** add chroma to the axes;
**(c)** declare the anchors by eye.

➡️ **(a)** — the only version where "conformant to TouchWiz" is a number; (b)
appeared to collide with a9's three materials, since MD1's measured character is
achromatic.

**✔ (verbatim):**

> a, with b, just to be sure that you stick to material science, so a gear isn't
> painted blue for no reason

**Override.** Chroma **is** an axis — but guarded by material science rather than
free. The apparent collision with a9 was real and is resolved by Q109: a material's
chroma is bounded by the mechanism that produces its colour, so the three-material
rule and the measured ladder agree.

## ❓ Q105 — Where do the six slots sit, given the corpus is one ladder?

**(a)** six declared positions along the measured axis; **(b)** the ternary simplex,
accepting the duplicate slot; **(c)** two axes, adding a construction axis the corpus
does not have; **(d)** keep the emphasis names but derive them from the parameter
values.

➡️ **(a)** at the ends plus **(c)** for the middle — one measured ladder for surface
character, one independent axis for construction.

**✔ (verbatim):**

> a with c but the layers can't be very thick at all for material transmission
> properties

## ❓ Q106 — What carries the freedom the corpus doesn't give?

**(a)** the palette triad per icon; **(b)** the construction axis; **(c)** the accent
form; **(d)** the glyph source — at the TouchWiz end the mark is drawn from a
TouchWiz object.

➡️ **(a)+(b)+(c)**; (d) held as its own decision because it has a licence attached.

**✔ (verbatim):**

> (a)+(b)+(c), all three, crossed with the ladder

**(d) is not taken**, which is what Q107 then confirms.

## ❓ Q107 — What may the corpora legally be used for?

Only MD1 is derivative-safe: `material-design-icons-4.0.0` is Google's official set
under **Apache-2.0**. `galaxy-s4-icon-pack` is an unpacked Samsung APK, and
`iOS-6-Icons-main` is a community set whose own README asks for credit for
third-party and Reddit-made icons. **(a)** measurement only; **(b)** geometry too,
with an attribution record; **(c)** geometry, falling back if the licence bites.

➡️ **(a)** — measured character is an idea, a copied outline is a copy.

**✔ (verbatim):**

> a, the ideas from the other packs is what we're keeping

---

# Round 23 — the six slots, the colour, and the layers

## ❓ Q108 — Which axes make the six slots, and which are per-icon constants?

Six slots cannot be a cross of all three carriers of a36 (palette × construction ×
accent = twelve). **(a)** three ladder anchors × two constructions, palette and
accent per icon; **(b)** three anchors × two accent forms; **(c)** two constructions
× three palettes; **(d)** six λ positions on the ladder, everything else per icon.

➡️ **(a)**, with (d) folded in behind it: the builder takes a continuous λ, the slots
sit at the three anchors, and a later icon can use any point on the ladder without
new code.

**✔ (verbatim):**

> A with D

## ❓ Q109 — How does colour physically arrive, per family?

**(a)** a declared **chroma ceiling** per family with its mechanism named, and the
mechanism recorded per material; **(b)** any family, any chroma, nothing recorded;
**(c)** no hue on metal or glass at all, all colour on the pigmented families as
separate parts.

➡️ **(a)** — and it strengthens the three-material claim rather than weakening it:
substrate + pigment + record piece is a *physical* reading of the three, and the
manifest gains a field saying *why* the blue is blue.

**✔ (verbatim):**

> A

## ❓ Q110 — What is the warrant for a given hue on a given icon?

**(a)** semantic — the hue comes from the game's own declared token for that surface;
**(b)** material — chosen per icon from the matrix, recorded in the brief; **(c)**
both, (a) taking precedence wherever the game has a token.

➡️ **(c)**, which costs nothing: of the seven anchors, four are backed by game
tokens, one is partial, and two are declared slots.

**✔ (verbatim):**

> c

## ❓ Q111 — What is the thin-layer rule, and how is "not very thick" checked?

Today the layered construction adds layers at 0.30 and 0.18 of the body depth,
displaced forward by 0.62 and 0.74 of it — so the stacked object protrudes to
**1.74×** the body's thickness, which is a sandwich rather than a finish. **(a)** a
declared maximum per-layer thickness and total protrusion, checked; **(b)** a render
check — the substrate must remain identifiable in a stated share of the stacked
area; **(c)** an absolute thinness in pixels.

➡️ **(a) with (b) as the check that matters**; (c) is wrong because it would make the
96 px master and the 24 px icon physically different objects.

**✔ (verbatim):**

> (a) with (b) as the check that matters: (a) says what the builder may do, (b) says
> what the result must still show.

---

# Round 24 — the build

## ❓ Q112 — How should the settled tree be built?

**(a)** all in one pass; **(b)** conformance machinery first, then re-render to see
whether the slots stop collapsing; **(c)** write the standard document first.

**✔ (verbatim):**

> Stage it: conformance first

**Settled.** The build order is: chroma ceilings and hue warrants → the hue
precedence fix → the corpus ladder → the conformance and separation checks →
re-render → picker. Accent forms, the thin-layer builder bound and the standard
document follow the staged result.

---

## Corrections ledger for C4

| Was | Now | Because |
|---|---|---|
| C3's "What each answer settles" implies the palette matrix landed | The declared hue **never reached a pixel** — `from_blender_material` read `Base Color` out of the blend, so the blend owned the colour as well as the surface | Measured: four tools declaring four hues rendered one grey |
| C3 said the arrival bands are "re-derived by measurement" | **Not done**; still owed | Reading the file while fixing the precedence bug |
| The six-slot model was recorded as implemented (emphasis × construction) | The emphasis could only change geometry: material sets identical across all six candidates | Measured; `EMPHASES`' docstring was false in code |
| Agent implemented the Q71(a) near-metal swatch skip | **Retired** — its premise was the black world, and the world now has radiance | a99; leaving it would hide the next real metal failure |
| Agent recommended "a9 stands, surface only" for Q101 | Overridden by the corpus-interpolation model | a101 |
| Agent recommended excluding chroma from the interpolation axes | Overridden: chroma is an axis, guarded by a per-family mechanism | a104, resolved by a109 |
| Agent's Q105 recommendation was the two-axis version | Confirmed, with the added constraint that layers stay thin for material transmission | a105 |
| `rig.py`'s claim that the world is black, and that the three lights are the only illumination | No longer true by design | a99 |
| `rig.py`'s comment claiming the energies were raised 2.5× so a swatch would clear 0.20× | False in both directions: nothing there is what rendered, and the blend's key is 7× *below* it | C3's a5, corrected while editing the file |
| `reference.blend` assumed to carry all seven families | **Six**; `enamel` fell back to the Python table silently | Measured by the world sweep |
| "`tool-power` is the only candidate failing silhouette containment" | **Wrong** — `tool-road` fails at **0.3235**, far worse | Per-icon containment measured after the conformance pass; the earlier claim came from reading note text rather than querying the field |
| Agent's Q101 recommendation was "a9 stands, emphasis is surface and balance" | Insufficient: the staged pass measured the consequence — surface decoration cannot move fill from 0.49 to 0.94, so the *construction* has to change | The stage result above |

## What each answer settles, in one place

1. **The world is the rig's ambient half**: grey, radiance **0.40**, measured,
   blend-preferred, with `clip_fraction` as a check and the near-metal skip retired.
2. **Colour is physical**: every family has a declared **chroma ceiling** and a named
   mechanism (metal = anodised film 0.060, glass = body-tinted 0.100, enamel =
   painted colour 0.190). Above a ceiling is a different material, not a stronger
   colour, and `within_ceiling()` says so.
3. **Every hue is warranted**: the game's own declared token first (`Refused` 25°,
   `ZoneIndustrial` 60°, `Ink` 250°), author choice recorded *as* a gap-fill for the
   two declared slots and for `Tool::Zone`.
4. **The shape builder interpolates a measured corpus ladder** — fill, gloss,
   complexity — with λ continuous; the six slots sit at the three anchors × two
   constructions.
5. **The corpora are measurement-only.** Ideas, never geometry; only MD1 is
   derivative-safe.
6. **Conformance is the corpus anchors plus a separation floor**, plus the
   substrate-survives check for the layered construction, plus declared layer maxima.
7. **Freedom beyond the ladder** comes from the per-icon palette triad, the
   construction, and the accent form — crossed with λ.
8. **The layered construction must stay thin**: declared maxima bound the builder,
   and the substrate must still be identifiable in the render.

## What has landed so far

- **`palette.py`** — `FAMILY_COLOUR` (mechanism + ceiling per family) with
  `colour_mechanisms()` refusing a table that disagrees with the chromas in use;
  `within_ceiling()`; `hue_warrants()`. Verified: no drift, no gamut misses,
  `metal` rejects 0.19 and accepts 0.06.
- **`openpbr.py`** — `apply_base_color()`, so the matrix's colour is *written onto*
  the material before rendering, and `from_blender_material` now takes the base
  colour from the declared parameters while still reading the surface from the
  blend.
- **`rig.py`** — the world is loaded from the blend and applied; `WORLD` declares
  radiance 0.40 with the sweep as its reason; `reference_roles()` exposes which
  families the blend actually carries.
- **`generate.py`** — `clip_fraction` measured per size; the near-metal skip
  retired; `surface_source` recorded per material so a Python fallback stops
  impersonating the reference.
- Not yet re-rendered, so every current render is stale against the moved palette,
  the world, and the hue fix.

## Stage result: the conformance pass, as measured

Q112's staged build was run. What it answers is the question the session opened
with: **do the six slots separate?**

**The hue fix is verified — the declared hue now reaches the pixels.** The four
tools render four different colours, each recorded with `colour_source: matrix
(metal at the declared hue)`:

| icon | declared hue | rendered silhouette |
|---|---|---|
| `tool-road` | blue | `[0.327, 0.497, 0.723]` |
| `tool-power` | amber | `[0.658, 0.431, 0.281]` |
| `tool-inspect` | cyan | `[0.252, 0.549, 0.568]` |
| `tool-demolish` | red | `[0.712, 0.397, 0.372]` |

**The slots do not separate, and the measurement says so in numbers.** The ladder
spans 0.510 of fill; the six candidates of each icon span:

| icon | fill spread | gloss spread | closest pairs within the floor |
|---|---|---|---|
| `tool-power` | 0.177 | 0.108 | 2 |
| `tool-road` | 0.146 | 0.085 | 2 |
| `tool-inspect` | 0.144 | 0.073 | 2 |
| `vocab-settings` | 0.115 | 0.030 | **7** |
| `tool-demolish` | 0.096 | 0.033 | **7** |
| `ticket` | 0.073 | 0.144 | 1 |

Worse than the small spread is that it does not **track λ**: `tool-road`'s two
`ios6` candidates measure fill 0.455 and 0.459, *below* its `md1` candidate's 0.508,
where the ios6 anchor demands 1.000. The readings are not merely unmet, they are
unordered.

**Why, mechanically.** The silhouette is an MD1 glyph traced at a fixed span in all
six slots, and a thin-line glyph occupies roughly 0.4–0.5 of its own box however much
decoupage is added around it. No ring, inset or layer depth can move fill from 0.49
to 0.94: reaching the TouchWiz end requires the object to **become a plate** — a
solid body filling the tile with the mark on or in it — which is a change to the
*construction*, not a decoration. This is exactly the prediction recorded in Q100
and Q101, and the staged pass is what turned it into a number.

**Two defects the new measurement found that no earlier check could see:**

1. **The layered construction creates unintended enclosed gaps.**
   `tool-power.d-touchwiz-stack` reports **20** holes and
   `vocab-settings.c-touchwiz-plate` 12, where the anchors predict 0 or 1. Slivers
   between stacked plates are being enclosed and counted; a real geometry defect in
   the stack, not a threshold problem.
2. **`tool-road` fails silhouette containment at 0.3235** — worse than
   `tool-power`'s 0.857. Containment is computed from the alpha channel alone, so
   none of this stage's changes could have moved it: it is pre-existing, and the
   earlier "still open" entry named only `tool-power` because that was the only
   failure whose note text had been read. Whether the fault is the trace or the
   comparison's normalisation for a mark whose ink does not span its own tile cannot
   be told from the outside, and that is the next stage's first task.

**A scoping decision, recorded rather than hidden.** Piece count and hole count are
measured and stored per candidate but **not judged**, because under the composition
rule an icon is a mark *plus* an accent piece, so its piece count is the mark's plus
the composition's and can never equal a bare glyph's. Judging it would fail every
compliant candidate, and a check that cannot pass trains everyone to ignore the
notes. The mark's own topology is already compared by the silhouette check.

## Still open, and deliberately so

- **The icon-standard document**, which the settled tree now fully determines.
- **`tool-power`'s trace** (0.857) and **`tool-road`'s** (0.3235) against a 0.90
  containment floor — the open ring in `power_settings_new`, and for `add_road` a
  six-piece mark whose comparison cannot yet be told from a trace fault. The
  earlier entry here named only `tool-power`; the per-icon measurement above
  corrects that.
- **The plate construction** the conformance pass proved is needed: fill cannot rise
  from 0.49 to 0.94 by adding decoration to a thin glyph.
- **The layered construction's enclosed slivers**, which the topography count now
  makes visible.
- **The arrival bands**: still the pre-world values, and C3 already owes their
  re-derivation by measurement.
- **`enamel` missing from `reference.blend`**, and **`reference.blend` still
  untracked**, so the rig that made every pixel is not in the repository.
- **`Tool::Zone`** (no MD1 glyph, no game token) and **`ledger`**, both deferred.
- **The accent forms** (tab / seal / ribbon / notch / band / corner) and the
  **thin-layer builder bound**, both staged after the conformance pass.
- **Motion** (C2's a46, C3's Q62) and the **loader** that would consume the set.

---

Continues in **`docs/GRILLING-C5.md`** — the picker, the design-document audit, and
the reconciliation of the lineages, the pillars and the icon's three roles.
