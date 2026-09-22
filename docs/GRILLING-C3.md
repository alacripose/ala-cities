# Grilling — C3: the rules that shape an icon

C3 exists because the icon pipeline shipped a review set it could not promote.
Every candidate that carried a metal body failed the material check, and the
failure was not a bug in a recipe: the pipeline had no rule saying what an icon
*is*. Round 11 settled that an icon is "a locator to a surface or record"; rounds
12 and 13 settled the size ladder, the licence and the contract for adding one.
None of them settled form, colour, or where either comes from.

The user's own diagnosis opened the session, and it was half right — which is
recorded below, because confirming a cause by measurement is the part of this
document that later sessions can build on.

**Convention, as in `GRILLING.md` and `GRILLING-C2.md`:** `➡️` is the
recommendation made. `✔` is the answer received. Where an answer **overrode** a
recommendation it says so. Question numbers continue C2's sequence; C2 ended at
Q69.

---

## What prompted it

Asked to regenerate the icons, the generator ran clean and produced a set that
**nothing could ship**: no `review-decisions.jsonl` existed, so no candidate was
promoted, and four of the six icons had *no* candidate that could pass the hard
selection gate. The numbers, gathered before the first question:

| Claim under test | Measurement | Verdict |
|---|---|---|
| "the icons are too dark" | Three `Tool::*` icons render at p75 luminance 0.037 / 0.055 / 0.058. `Panel` has luminance **0.0012**; passing 3:1 there needs p75 ≥ **0.1036** | Cause found — 2–3× short |
| "…because I changed the blender file reference" | The manifest records `reference_settings.source: "reference-blend"`, i.e. the blend wins at render time. Its lights are key **500 W**, fill **100 W**, rim **4000 W** against `rig.py`'s declared **3600 / 2500 / 2200** | Confirmed — key 7.2× dimmer, fill 25× dimmer |
| the swatch check | All **31** metal-primary candidates measure `swatch_to_albedo = 0.306` to three decimals — identical, so the swatch is measuring the material, not the icon | The check is also mis-calibrated for a mirror in a black world |
| `ledger` / `ticket` pass anyway | 9.23:1 and 8.45:1 on the same Panel | The floor is achievable; it is the metal body that is dark |
| "the icons are not in the game" | `grep -rn "assets/icons" src config Cargo.toml` finds only `pick.rs`, which reads `review.json` | Nothing consumes the shipping set yet |
| the preview blend | `assets/icons/reference.blend` is **untracked** (`??`) | The rig that made every pixel is not in the repository |

---

# Round 15 — the promotion, the rig, and the trigger

## ❓ Q70 — What is the shipping asset for?

**(a)** build the loader now and ship into it; **(b)** promote now to advance the
pipeline's own pilot gate, loader as its own piece; **(c)** don't promote, treat
the pipeline as an audit surface.

➡️ **(b)** — the pilot's job is to prove the chain, and the manifest is the
artifact worth having; 60 files nothing reads is the counter-argument.

**✔ (verbatim):**

> the icons are for the game but we haven't gotten to that point yet

**Read as:** the game is the consumer, later. Promotion is a staging step toward a
loader that does not exist yet, and the record says so rather than implying a
consumer.

## ❓ Q71 — Is `0.306×` a metal that failed to arrive, or a swatch that cannot see a metal?

A metal at metalness 0.9 under a black world has no diffuse and almost nothing to
reflect, so a flat swatch reads dark *by construction*. But the icons are dark
too. **(a)** skip the swatch judgement for near-metal materials with the reason
recorded — the precedent is already in `generate.py` for the neutral-chroma case;
**(b)** give metal swatches a second declared illumination; **(c)** change the
art; **(d)** change nothing and accept four icons shipping nothing.

➡️ **(a) for the check, (c) for the icons** — the swatch fix rescues no icon,
because the contrast failure survives it.

**✔ (verbatim):**

> the icons are too dark because I changed the blender file reference but we
> haven't adjusted the rules yet

**Read as:** the cause is the rig change, not the metalness, and the thing that
has not caught up is *the rules*. That answer moved the session from "fix the
materials" to "re-evaluate the rules", which is what C3 became.

## ❓ Q72 — Do band changes and material changes count as doctrine?

`read_decisions` filters by `concept_set`, so anything that bumps it voids every
recorded target. **(a)** any material or check change bumps the set; **(b)** a
check band is a claim about *measurement* and may move without a new set, recorded
with its reason; a material change is art and does bump it; **(c)** both move
freely, the log is append-only anyway.

➡️ **(b)** — changing how we measure is not changing what we are judging.

**✔ (verbatim):**

> any changes to the materials, unified deisgn spec or reference blend file should
> trigger a rebuild of the icons through the picker

**Read as:** the trigger is not a band edit at all. Materials, the design spec and
the reference blend are **inputs**, and an input change reopens the review. The
concept set is the mechanism; the question of *who notices* is Q75.

## ❓ Q73 — The surface check cannot fail — do we care?

`candidate_gate_notes` refuses a candidate whose `locates` is not in
`shapes.DECLARED_SURFACES`, but that tuple is built from the same entries
(`DECLARED_SURFACES = tuple(entry["locates"] for entry in ICONS)`). It is
definitionally always true. **(a)** hold promotion until `locates` comes from the
game's own registry; **(b)** ship now, leave the check unenforceable; **(c)** point
it at the game's real declarations in this session and accept what it refuses.

➡️ **(c)** — a check that cannot fail is the thing this pipeline argues against.

**✔ (verbatim):**

> I don't know

**Read as:** not an answer, so the facts were gathered and asked again as Q76.

---

# Round 16 — the rig, the standard, and the question underneath

## ❓ Q74 — Which file is the rig?

**(a)** `rig.py` is authoritative and the blend is regenerated from it; **(b)** the
blend is authoritative, and the dict stops pretending; **(c)** both stay, with
declared-vs-effective recorded and promotion refused past a divergence; **(d)**
keep both and just re-derive the bands.

➡️ **(b) with (c)'s guard** — the blend was edited on purpose, so making `rig.py`
win would undo the edit; but an editable blend that silently overrides a recorded
dict is how a manifest starts lying.

**✔ (verbatim):**

> the reference blend file is the rig - but the unified design spec and the openpbr
> is the standard to follow - read docs grilling c2 for reference

**What each answer settles.** The blend is the rig. `UNIFIED_DESIGN.md` and the
OpenPBR parameter set are the *standard the renders are judged against*. Three
consequences, all recorded: `rig.py`'s comment claiming the energies were raised
2.5× is now false and the dict is fallback-only; the bands are a claim about *this*
rig, so they are re-derived by measurement rather than edited to taste; and
`reference.blend` must be committed, because pixels are not verifiable against a
rig that is not in the repository.

## ❓ Q75 — What makes a rebuild happen?

`source_hashes()` already records `generate.py`, `openpbr.py`, `palette.py`,
`rig.py`, `shapes.py`, `reference.blend`, `src/hud.rs` and `src/design.rs` — and
**nothing compares them to disk**; `UNIFIED_DESIGN.md` is not in the set at all.
**(a)** automatic, with the design spec added; **(b)** declared — a human bumps
`concept_set`; **(c)** derive `concept_set` from the hashes, so a change mints a
new set and voids targets by construction.

➡️ **(c)** — it needs no new enforcement machinery.

**✔:** **`b`** — declared. The next generation of this rule was Q83.

## ❓ Q76 — The surface check, asked again with the facts

`src/main.rs:44` declares `enum Tool { Road, Zone, Power, Demolish, Inspect }`, and
the app shows exactly two panels — `show_ledger` (key L) and `show_help` (key H).
So `Tool::Road` / `Tool::Power` / `Tool::Inspect` name real variants, `the ledger
panel` names the real `show_ledger`, `a ticket in the ledger` names a real
`TicketKind`, and **`settings surface` names nothing**. The inverse gap: **Zone and
Demolish exist in the game with no icon at all.** **(a)** resolve `locates` against
the game's declarations and refuse what will not resolve; **(b)** keep the prose
locators and drop the check; **(c)** split the field — machine-resolvable `locates`
for the gate, free prose `reading` for people.

➡️ **(c)** — the prose is useful in the review set; it just cannot be what a check
reads.

**✔ (verbatim):**

> read the other two grilling docs for reference

Both were read. They supplied the rule the whole round needed — `UNIFIED_DESIGN.md`
line 458, *"Icon / tile — Locator to a surface or record. Forbidden reading:
capability grant"*; round 10's Q49, one icon per **surface**; and round 13's a63,
*"a list of icons from the relevant design specs to recreate within the new
system"*. The locator question was then answerable and became Q82.

## ❓ Q77 — Given the rig change, do we promote anything today?

**(a)** hold until the rules are re-derived and the set re-rendered; **(b)** promote
the two promotable icons now, accepting the marks go stale; **(c)** promote all six
anyway.

➡️ **(a)** — the clicks are the scarce thing, and (b) spends them on a review set
that Q74 and Q75 will invalidate.

**✔ (verbatim):**

> I need you to reevaluate the rules that shape the icons - I have added the
> material design icons and ios 6 icons to assets/reference - so all in all -
> touchwiz defines the artistic intent, material design 1 defines the silhouette,
> ios 6 lands where the two don't meet

**This is the session's centre of gravity.** It overrides (a): nothing is promoted,
and instead the rules are re-authored against three reference families with
declared responsibilities. Everything from here is downstream of that sentence.

---

# Round 17 — what each reference family decides

Facts gathered first, because the answer depended on what the three packs actually
are:

| family | what it is | licence |
|---|---|---|
| `material-design-icons-4.0.0` | Google's official set, 17 named categories, five optical styles, `png/<cat>/<glyph>/materialicons/<size>dp/<1x,2x>/baseline_<glyph>_black_<size>dp.png` | **Apache-2.0** — derivative use is available |
| `iOS-6-Icons-main` | 549 community files; the README asks credit for third-party and Reddit-made icons | reference-grade only, not licence-grade |
| `galaxy-s4-icon-pack` | an unpacked APK; 125 TouchWiz-native rasters in `res/drawable/*.png`, plus Material Components chrome | Samsung artwork, reference only |

Only MD1 is licensed for derivative use, and `baseline` is literally the MD1-era
filled mark — so "MD1 defines the silhouette" names a concrete artifact that a
check can be run against.

## ❓ Q78 — What each family is allowed to determine

`shapes.py`'s `REFERENCE_STUDY.lineage_precedence` is *object*-based (TouchWiz
wherever it has a matching object, MD1 only where Samsung does not, iOS 6 last).
The new instruction is *aspect*-based. **(a)** rewrite precedence per aspect;
**(b)** keep object-based and read the instruction as description; **(c)** make the
glyph *name* the join — look the object up in MD1's catalogue for the silhouette,
apply TouchWiz finish, fall to iOS 6 when the name is absent.

➡️ **(c)** — the only version where "MD1 defines the silhouette" is checkable.

**✔ (verbatim):**

> c, but make sure every icon has three main colors, a main silhoutte, and a
> accenting ledger piece, silhouette from the md1 icon, three main colors/materials
> similar to touchwiz, and the accent piece like an ios 6 icon

**Settled — the composition.** Every icon is: **one silhouette** (MD1), **three
main materials** (TouchWiz in surface and colour intent), **one accenting record
piece** (iOS 6). That is the first mechanical definition of an icon this pipeline
has had, and it is what makes "exactly three materials" checkable at all.

## ❓ Q79 — Which TouchWiz is the intent?

Round 13's a60 names the pillar *"early touchwiz"* — 2008–2010, heavier gloss. The
only TouchWiz artifact on disk is a Galaxy S4 Nature UX pack, one generation later
and markedly flatter. **(a)** the pack on disk is the intent; **(b)** "early" is the
intent and the pack is a material reference only; **(c)** commit an early pack
before deciding.

➡️ **(a) plus a recorded disagreement** — a pillar named in a verdict that no
artifact supports is the unbacked claim the record exists to prevent.

**✔ (verbatim):**

> the pack on the disk with ios6 as the skeumorphic layer above it

**Settled.** The galaxy pack is the base family; iOS 6 is a **layer above** it, not
a parallel family. The "early TouchWiz" phrasing in a60 is superseded by the
artifact.

## ❓ Q80 — What MD1 may supply — the system, or the outlines?

**(a)** only the system (24 dp grid, keylines, 2 dp stroke), geometry original;
**(b)** outlines may be adapted with attribution and the provenance claim changes;
**(c)** per icon, declared in the manifest.

➡️ **(a)** — one provenance story, and the grid is what a check can hold.

**✔ (verbatim):**

> per icon, the icons before had no knowledgable shape except for the settings icon,
> which was programatically recreating the touchwiz icon, but badly

**Overridden to (c)**, and the answer carries a correction about the existing set:
the six pilots were **not** authored under the old precedence — they were ad-hoc,
with no reference object behind any of them, and only the settings gear had one.
So the pilots are discarded as *form*, and their recipes survive only as history.
Per icon, the manifest declares whether MD1 supplied only the grid or the outline
was adapted.

## ❓ Q81 — Does this void the six pilots?

**(a)** re-author under the new precedence, bump the concept set, promote nothing
from the old rule; **(b)** re-render only; **(c)** keep the portraits and spend the
six slots on a family spread.

➡️ **(a)** — (c) would turn the picker into a comparison of families rather than of
readings of one icon.

**✔:** **`a with c if you can`** — re-author, *and* express the family spread in the
six slots if it can be done without breaking the picker. Resolved as Q89.

## ❓ Q82 — The inventory

**(a)** from the game's real surfaces, prose out of `locates`, check allowed to
fail; **(b)** from the design specs' own named lists, filtered to what the game has,
each entry naming its MD1 counterpart; **(c)** keep the hand list, add Zone and
Demolish, drop `settings surface`.

➡️ **(b) with (a)'s key.**

**✔:** **`b with a`** — settled as recommended. The join table, measured:

| surface | MD1 glyph |
|---|---|
| `Tool::Road` | `maps/add_road` |
| `Tool::Power` | `action/power_settings_new`, `content/bolt` |
| `Tool::Inspect` | `action/search` |
| `Tool::Demolish` | `action/delete` |
| `show_help` | `action/help` |
| settings mark | `action/settings` |
| ticket | `notification/confirmation_number` |
| **`Tool::Zone`** | **none** |
| **ledger** | **none** |

## ❓ Q83 — Who notices a concept-set change?

**(a)** nothing; the discipline is the user's; **(b)** the manifest gains a
*report*: recompute the hashes and show divergence, with `UNIFIED_DESIGN.md` added
to the set, while enforcement stays a human bump; **(c)** make the picker refuse on
mismatch.

➡️ **(b)** — report, never enforce.

**✔:** **`b`** — settled as recommended.

---

# Round 18 — the three, the accent, and the layer

## ❓ Q84 — Which three, and are they a system?

Ten material roles existed (`metal`, `paper`, `ceramic`, `glass`, `polymer`,
`road`, `ink`, `accent`, `blue`, `black`) against ~32 interface tokens, and round
11 recorded that icons **depict** materials while the interface **applies** none.
**(a)** a fixed triad, identity-only, never a state token; **(b)** per-icon three
chosen for the object; **(c)** three tokens, applied.

➡️ **(a)** — makes "three main colours" checkable and closes round 11's open
colour question.

**✔ (verbatim):**

> b - use the reference materials, create roygbiv versions of each material type
> derived from the main materials, so each icon can have its own color pallete while
> sticking to the reference materials

**Overridden.** Per-icon palettes, from a **material matrix**: each reference
material expanded into a hue family, so an icon chooses its own three while every
material remains a real reference material. This is the largest single consequence
of the session and became round 20.

## ❓ Q85 — What is the "accenting ledger piece"?

The doc's only use of *ledger* is the record that "holds" a balance (line 633).
**(a)** an accent that reads as *a piece of the record* — tab, seal, ribbon, badge;
**(b)** simply an accenting iOS 6-style glossy detail.

➡️ **(a)** — an accent that means a record is a locator; a decorative gloss would be
the "adornment" §8 warns about.

**✔:** **`a with b`** — both: it reads as a record piece *and* it is the accenting
detail. Constraint carried in: round 11's Q52 stands — a count is **never baked**
into the icon.

## ❓ Q86 — How is the iOS 6 layer expressed?

The rig has **no post-processing** and says so, so a layer can only be geometry or
OpenPBR parameters. **(a)** material parameters; **(b)** geometry; **(c)** both,
declared per icon.

➡️ **(c)** — the two produce different measurements, and a gloss that exists only
as mesh is a finish the swatch check never sees.

**✔:** **`c`** — settled as recommended, with the declaration requirement recorded.

## ❓ Q87 — Which MD1 style and density?

Five optical styles ship; `baseline` is the MD1-era filled mark and 24 dp is the
grid it was drawn on. **(a)** baseline at 24 dp, always; **(b)** baseline by
default, another style where it reads badly at 24 px; **(c)** per icon.

➡️ **(a)** — one style is what makes the silhouettes a system.

**✔:** **`a`** — settled as recommended. The artifact is
`materialicons/<size>dp/<1x,2x>/baseline_<glyph>_black_<size>dp.png`.

## ❓ Q88 — Zone, and the settings mark

**(a)** Zone waits until the game has a zone surface and is recorded as deferred;
the settings mark is rebuilt on `action/settings` as the vocabulary mark it always
was; **(b)** Zone gets an iOS 6 map-style gap icon now; **(c)** drop both.

➡️ **(a)** — same reasoning that retired the invented surface name.

**✔:** **`a`** — settled as recommended.

## ❓ Q89 — How do six slots carry a reading *and* the family spread?

**(a)** the three pillars lead in different proportions — MD1-led (silhouette-
dominant, flattest materials), TouchWiz-led (material-dominant), iOS 6-led (accent-
dominant) — ×2 each; **(b)** six readings of one composition with the spread
recorded as a brief field; **(c)** six readings plus a separate family field shown
beside each.

➡️ **(a)** — the spread becomes a choice rather than a note, and each slot is still
a whole composition under Q78's rule.

**✔:** **`a`** — settled as recommended. The earlier objection (that a family spread
turns the picker into a different tool) is resolved by emphasis ×2 rather than by
picking one.

---

# Round 19 — the derivation, the accent, and the check

## ❓ Q90 — How are the seven derived, and at what chroma?

The families' declared chromas were near-neutral — 0.018 for metal and polymer —
and **rotating a hue at chroma 0.018 changes nothing**. **(a)** declare a variant
chroma per family, held across seven declared hue anchors, with every variant
re-checked against the hosts; **(b)** hand-author all seven per family; **(c)**
sample the anchors from the pack.

➡️ **(a)**, with (c) used only to *choose* anchors.

**✔:** **`a with c`** — the derivation is a rule at a declared per-family cap, and
the anchors are informed by the pack. Q96 is where this collided with what the pack
actually contains.

## ❓ Q91 — What keeps per-icon palettes a system?

**(a)** free per icon; **(b)** each icon declares a *lead* material and the other
two follow by a declared relation; **(c)** a rule per tier.

➡️ **(b)** — checkable, where "free choice" is not.

**✔:** **`b with c`** — the tier sets the lead, the relation sets the pair. Became
Q92's tiers.

## ❓ Q92 — What counts as a material type?

`MetalBody`, `PaperBody`, `GlassLens`, `PolymerGrip`, `RoadSurface`,
`CeramicInsulator` are materials; `IconBlue` and `IconBlack` are colours wearing
material names; the manifest's roles (`accent`, `ink`, `blue`, `black`) are roles.
**(a)** six physical families × seven hues, with ink/white/black kept as the
neutral anchors and blue/black retired as families; **(b)** expand all ten × seven;
**(c)** keep blue/black as a seventh family, "enamel".

➡️ **(a)**, with `IconBlue` re-expressed as enamel.

**✔:** **`a with b`** — which cannot be read literally, so it was asked again as
Q95.

## ❓ Q93 — Is the accent a fourth colour?

**(a)** the accent reuses one of the three, distinguished by *finish*, so the icon
has exactly three colours; **(b)** three main + a fourth accent colour; **(c)** the
silhouette is a neutral that does not count.

➡️ **(a)** — makes "three main colours" a check rather than a slogan; iOS 6's gloss
does most of its work through reflection, not hue.

**✔:** **`a`** — settled as recommended.

## ❓ Q94 — The silhouette metric

**(a)** coverage band + centroid tolerance + occupancy overlap on a stated grid,
all recorded; **(b)** occupancy overlap only; **(c)** coverage + centroid, no
overlap.

➡️ **(a)**, with the MD1 file and digest recorded per icon; the honest limit being
that `baseline_*_black_*.png` is a flat black glyph, so the comparison is purely of
form.

**✔:** **`c with b`** — read as: check at the decision size rather than at 24 px,
with the glyph recorded as provenance. Became Q98, and the reference for it was
found to exist.

---

# Round 20 — closing the ambiguities

## ❓ Q95 — Q92 cannot be read literally: does the matrix cover the non-physical roles?

**(a1)** six families × 7, plus a declared neutral ramp that is deliberately not
hue-varied; **(a2)** six families × 7 **plus** a seventh family for the object's own
coloured enamel = 49, with ink/white/black as the declared neutrals; **(a3)** all
ten × 7, with ink and black as seven very low chroma "natural ink" variants.

➡️ **(a2)** — satisfies both halves without pretending a neutral has a hue.

**✔:** **`a2`** — settled as recommended: **seven families × seven hue anchors ×
three levels = 168 entries**, plus `ink`/`white`/`black` as the declared neutral
anchors that the silhouette and the accent draw from.

## ❓ Q96 — Where do the seven anchors come from?

**(a)** sample as-is and accept a blue-dominant "seven"; **(b)** sample the pack for
its chroma and lightness character, and *declare* the seven hues with the method and
digest recorded; **(c)** sample per object class for a semantic mapping; **(d)**
take the drop-the-background variant.

➡️ **(b)** — the pack is evidence for how TouchWiz colour *sits*, not for seven hues
it does not use.

**✔ (verbatim):**

> I don't understand

**Not an answer, so the question was re-put with the measurement that made it
concrete — and the measurement is the finding of the round.**

Two populations, sampled with the same method:

| population | files | anchors |
|---|---|---|
| `res/drawable/*.png` | 125 | 18, 42, 78, 178, 202, 228, 272 |
| every `.png` in the pack | 327 | **18, 42, 78, 178, 202, 228, 272** |

Identical. The warning that Material Components chrome would poison the anchors —
made by the agent, and recorded as **wrong** — does not materialise: the chrome is
XML vectors and the density buckets are downscaled copies. But the sampling did
find something the question had not anticipated:

| class | n | circular-mean hue | median S | median V |
|---|---|---|---|---|
| people | 29 | 162.6 | 0.53 | 0.69 |
| message | 4 | 34.5 | 0.92 | 0.92 |
| media | 19 | 226.2 | 0.87 | 0.84 |
| system | 32 | **131.4** | 0.88 | 0.85 |
| store | 20 | 201.8 | 0.78 | 0.85 |
| nature/health | 6 | 193.0 | 0.88 | 0.91 |

**45.7% of all chroma-weighted pixels sit in 180–210°.** The class spread is a
95° arc from green through cyan to blue, plus one orange outlier on four icons.
There is no red, no yellow and no violet in the pack, and the game's own declared
hues tell the same story: `Refused` 25, `ZoneIndustrial` 60, `Warning` 70,
`Scaffold` 80, `Nature`/`ZoneResidential` 150, `Ink`/`ZoneCommercial` 250,
`Agent`/`Retired` 260.

**So a ROYGBIV matrix cannot be sampled from either reference.** The class-wise
systematic reading of your own example — *green for nature, blue for chrome* —
found chrome is **green** (131°), not blue; blue is media and store.

## ❓ Q96b — the restated question

**(a)** accept a blue-dominant seven; **(b)** declare the seven hues with the pack
supplying character; **(c)** semantic anchors by object class; **(d)** the
drop-the-background variant.

➡️ **(b)**, with (c) as the interesting rival.

**✔:** **`c + b`** — semantic anchors from the class-wise sample, declared as
constants, with the pack setting the character. The result, as implemented:

| anchor | backing |
|---|---|
| 25° red | ✅ `Refused` — destructive, `Tool::Demolish` |
| 62° amber | ✅ `ZoneIndustrial` / `Warning` / `Scaffold` — caution, power |
| 115° yellow | ⬜ declared slot |
| 150° green | ✅ `Nature` / `ZoneResidential` |
| 200° cyan | ⚠️ partially backed — study people 162.6, store 201.8; no game token |
| 250° blue | ✅ `Ink` / `ZoneCommercial` — data, `Tool::Inspect` |
| 300° violet | ⬜ declared slot |

Four evidenced, one partial, **two declared as slots** — because declaring a slot is
honest and inventing backing for it is not.

## ❓ Q97 — The tiers of Q91(c), and what leads each

**(a)** the HUD's three kinds — tools (Road, Power, Inspect, Demolish), records
(ledger, ticket), marks (the settings/vocabulary glyph) — leading metal, paper,
enamel; **(b)** tier by game object only, lead chosen per icon; **(c)** tier by the
three pillars.

➡️ **(a)**, with the derived pair written per tier so Q91(b) still governs the pair.

**✔ (verbatim):**

> a with b yeah

**Settled.** The tier sets the lead family; the relation sets the other two.

## ❓ Q98 — The silhouette check at the decision size

**(a)** coverage band + centroid tolerance + 12×12 occupancy, any miss a note;
**(b)** occupancy only; **(c)** coverage + centroid.

➡️ **(a)**.

**✔:** **`a`** — settled as recommended. The reference exists at that size: MD1's
largest PNG is `48dp/2x` = **96×96 px**, exactly the decision size, and the render
(192 px) reaches 96 by an exact ÷2 box mean, so neither side is resampled and no
reference has to be invented.

## Final confirmation

> **Confirmed**

---

## Corrections ledger for C3

| Was | Now | Because |
|---|---|---|
| "the icons are too dark" — the agent read this as a materials problem | The rig changed and **the rules** had not | The user's own correction in a2; confirmed by the light energies |
| Agent: Material Components chrome would poison the anchor sampling | **Wrong** — the populations give identical anchors | Measured both; recorded as wrong |
| Agent recommended a fixed triad for icon colour | Per-icon palettes from a ROYGBIV material matrix | a15 |
| Agent recommended the silhouette check at 24 px | At the decision size (96 px) | a25/a29, and the reference exists there |
| Agent recommended MD1 supply the grid only | Per icon, declared in the manifest | a11 |
| Agent: a family spread in six slots turns the picker into a different tool | Emphasis ×2 resolves it | a20 |
| Round 13 a60's pillar *"early touchwiz"* | The galaxy pack on disk, with iOS 6 layered above | a10 — the artifact supersedes the phrase |
| `settings surface` as a locator | A settings **mark**, on `action/settings` | No settings surface exists in `src/` |
| Six pilots assumed to follow an older precedence | They followed none: "no knowledgable shape except the settings icon" | a11 |
| The shipped settings blue was `(0.62, 0.19, 245)` | Out of linear-sRGB gamut; it clamped a **negative red channel to zero** and lost 0.034 chroma that was never real | Found while implementing the matrix |

## What each answer settles, in one place

1. **The asset is for the game, later.** Promotion stages for a loader that does
   not exist yet, and the record says so.
2. **The blend is the rig; `UNIFIED_DESIGN.md` + OpenPBR are the standard.** The
   bands are a claim about this rig and are re-derived by measurement.
3. **Any change to materials, the design spec or the reference blend reopens the
   review**, by a human bump of the concept set — reported, never enforced.
4. **An icon is a silhouette (MD1 `baseline`, 24 dp), three main materials
   (TouchWiz surface and colour intent), and one accenting record piece (iOS 6).**
5. **The inventory comes from the design specs joined to real game surfaces.**
6. **The palette is a matrix**: seven families × seven declared anchors × three
   levels, with ink/white/black as the neutrals, variant chroma declared per family
   as a cap, and every reduction to reach gamut recorded.
7. **Per icon**: the tier sets the lead family, a declared relation sets the pair,
   the accent reuses one of the three and is distinguished by finish.
8. **The iOS 6 layer is parameters and geometry**, declared per icon.
9. **The silhouette is checked** against MD1 `48dp/2x` at the decision size, with
   the file and its digest recorded.
10. **Zone is deferred**; the settings mark is rebuilt on `action/settings`.

## Still open, and deliberately so

- **`Tool::Zone`** and **`ledger`** have no MD1 glyph, so Q78's gap rule sends them
  to iOS 6 — and Zone is deferred until the game has a zone surface to locate.
- **The two declared slots** (115° yellow, 300° violet) have no reference and no
  game meaning. They exist because the hue ring needs them, and they are marked.
- **The iOS 6 layer's geometry half** is declared per icon but the renderer's
  ability to show it at 24 px is not yet measured.
- **Motion** (round 11's a46, round 13's Q62) is untouched by this session.
- The **loader** that would consume the promoted set.

---

## Superseded where C4 disagrees

Two items in this record were corrected by the next session, and the correction
matters more than the original claim, so it is named here rather than left standing:

- **"The palette is a matrix" with a declared variant chroma** stands, but the
  matrix's hue **never reached a pixel**: `from_blender_material` read the base
  colour out of the reference blend, so the blend owned the colour as well as the
  surface and every icon of a family rendered one grey. Fixed in C4's staged build.
- **The world is black and the three lights are the only illumination** — no longer
  true by design: the world carries radiance 0.40, measured, because a mirror in a
  black room cannot be made legible by any light energy. See `GRILLING-C4.md`, Q99.

Continues in **`docs/GRILLING-C4.md`**.
