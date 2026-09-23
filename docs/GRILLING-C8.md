# Grilling — C8: the material truth, the external validator, and the world

C8 exists because the material half of the project was built for **one consumer**
and is now asked to serve three: the icon pipeline, the game world, and an external
model that reads both the renders and the world's claims. The session's first act
was measurement again, and it turned up four things the brief could not have
assumed — the external validator does not exist, the model that made the renders is
on this machine after all, the icon checks measure against a **transcription** of
the game's own colours, and the game world has no material vocabulary whatsoever.

**Convention, as in `GRILLING.md` and `GRILLING-C2.md` … `GRILLING-C7.md`:** `➡️`
is the recommendation made. `✔` is the answer received. Question numbers continue
C7's sequence; C7 ended at Q138.

---

## What prompted it

> take the material and shape icon builder system, and repair it using an external
> validator model, and apply the material science to the game world itself

**Facts established before the first question**, because the questions had to be
asked against the tree that exists:

| Premise | What the tree says |
|---|---|
| There is an external validator to repair *with* | **There is none.** `tools/` contains no HTTP client, no API key handling, no model call of any kind; the only "oracle" is `src/bin/verify.rs`, which re-reads the season record against the world |
| A model that could validate is out of reach | **Ollama is running locally** with `hf.co/ggml-org/Qwen2.5-VL-3B-Instruct-GGUF:Q4_K_M`, whose capabilities include `vision` — an external model, offline, already on the machine |
| The renders were made by a Blender that is gone | **It is here.** `C:/Program Files (x86)/Steam/steamapps/common/Blender/blender.exe` reports `Blender 5.3.0 Alpha, build 2026-09-10` — the recorded `build_hash 5eaad57cfabe` line. The manifest's own policy already accepts any Blender 5.x and records the build that made each render |
| The icon set is six pilots awaiting decisions | **Six pilots decided.** `review-decisions.jsonl` carries 6 lines, all with `target: true` (C7's session). `review.json` on disk was written **before** those decisions and still lists all six as `awaiting_decision` — measured defect: a decision recorded in the ledger does not clear the review surface until the pipeline regenerates it |
| The deferred inventory is `ledger` and `tool-zone` | Confirmed in the manifest's `deferred_inventory`, with `ledger`'s reason ("MD1 has no ledger glyph, so C3's gap rule sends it to iOS 6"). **`help` is in neither `ICONS` nor `deferred_inventory`**, though C6 promised it into the inventory |
| The icon pipeline and the game share one colour truth | They do not. `palette.py` states that `src/hud.rs` "is the authority; this file is a transcription of it", and separately notes "the world palette is not in `hud.rs` yet" — so the icon checks measure contrast against a **copy** of the host colours, and the world's colours are the missing half |
| The game world has material science to apply | **It has none.** `grep -i material src/` returns nothing. `render.rs` has one `LIGHT`, one `shade()` (`0.55 + 0.45·lambert`), and `hud.rs` has 32 OKLCH tokens with no families, mechanisms or ceilings |
| A save can carry new state | `saves/world.ron` is a **versionless** RON dump of `World` (4.49 MB measured, 30.7 ms). There is no format version to migrate from |
| There is an economy for materials to price | `Economy` exists (`credits: 25_000`, three tax rates, loans, month income/expense), `BuildingKind::build_cost()` is fixed per kind (120/240/400/1800), and a test asserts that road upkeep on a nearly empty city exceeds its tax take |
| The sim has mechanisms for materials to drive | `Tile.powered`, brownout, `BuildingKind::capacity()`, a road graph whose `edge_cost()` is explicitly stubbed "for the traffic model in C2", and a `Demand` vector |

Two staleness defects were found in the documents while measuring, and are recorded
here rather than edited away: `ICON_STANDARD.md` §7 says the rig "is currently
untracked" (it has been tracked since `b5488e5`), and `README.md` says "**no icon
has a target yet**" (there are six).

---

# Round 1 — the validator, the scope, the rig, the world

## ❓ Q139 — Which model is the external validator?

Candidates measured: the offline Ollama vision model, a hosted vision LLM API
(needs a key), a non-model independent re-measurement.

➡️ One sealed validator interface (an OpenAI-compatible HTTP endpoint) with the
local Ollama model as today's default, so swapping in a hosted model is config, not
a rewrite — and the validator stays an **external** oracle the pipeline cannot fake.

**✔ (verbatim):** `Local Ollama VLM default`

## ❓ Q140 — What does "repair" cover?

➡️ The builder must be able to produce a conformant six-slot set again: the plate
construction, the accent forms, the layered slivers, `tool-road`'s 0.3235 and
`tool-power`'s 0.857 against the 0.90 floor, `enamel` in the blend, and the arrival
bands re-derived by measurement.

**✔ (verbatim):** `Measured faults + unbuilt mechanisms`

## ❓ Q141 — Blender: the renders were made by 5.3.0 Alpha

➡️ Point me at the 5.3 Alpha build if it still exists, so the rig's numbers stay
comparable; if it is gone, re-render under 5.0.1 and supersede without erasing.

**✔ (verbatim):** `Point me at 5.3 Alpha`

**Q146 resolves it, and the answer is better than the plan assumed** — see below.

## ❓ Q142 — What does the validator's verdict do, and what may it see?

➡️ The validator reads the render and the icon's **declaration, blind** — never the
scripted checks' results — so the two readings stay independent; its verdict lands
in an append-only ledger and a `refused` verdict refuses promotion as a **note**;
it never promotes, and code never averages the two readings.

**✔ (verbatim):** `Blind verdict, note-only gate`

## ❓ Q143 — What does "apply the material science to the game world" mean?

➡️ The matrix becomes the world's material vocabulary (family + hue anchor + level,
ceilings enforced by a fail-closed gate), the renderer shades per mechanism, and
each surface's family is carried in the world state for a later sim rule.

**✔ (verbatim):** `Plus sim mechanics now` — **expanded**, not merely accepted: the
sim effects are in this campaign, not deferred to a later one.

## ❓ Q144 — Are the world's material claims part of the evidence record?

➡️ Yes, at the gate's grain: the material table is a digested artifact and the
gate's result lands in the season record, so "this build's surfaces are these
materials" is checkable by `verify.exe`.

**✔ (verbatim):** `Gate output as evidence`

## ❓ Q145 — What happens to the uncommitted tree?

➡️ Commit the existing work partitioned first (tooling → assets → runtime records →
docs), as C7 Q132 established, so anything this pass breaks is attributable.

**✔ (verbatim):** `Commit partitioned first`

---

# Round 2 — the build, the owner, the absence, the mechanics

## ❓ Q146 — The Blender build

**✔ (verbatim):** `"C:\Program Files (x86)\Steam\steamapps\common\Blender\blender-launcher.exe" goes to my blender 5.3`

**Verified, not taken on trust:** that path's `blender.exe` reports `Blender 5.3.0
Alpha` — the same cycle the manifest records, from a Steam install beside the
5.0.1 build in `D:/SteamLibrary`. **The renders stay comparable, the six promotions
survive, and no re-decision is owed *for that reason*.** The manifest's recorded
policy already covers the residual risk: any Blender 5.x is accepted, and the build
that made each render is written into it, so a difference is visible rather than
silent.

## ❓ Q147 — Where does the matrix live now that two consumers use it?

Today it is Python-only (`palette.py`: `FAMILIES`, `FAMILY_COLOUR` mechanisms and
ceilings, `MATERIAL_LEVELS`, `HUE_ANCHORS`), and `hud.rs` gamut-**clips** while
Python gamut-**searches**.

➡️ Python stays the single owner and generator; the game reads emitted data.

**✔ (verbatim):** `Python generates, Rust reads`

## ❓ Q148 — What happens when the material table is absent?

➡️ Fail closed, like `governor.json` ("a governance layer that fails open is worse
than none").

**✔ (verbatim):** `Fail closed, like the governor`

**Superseded in form by Q174** — see the corrections ledger. The principle stands;
the artifact is a generated Rust source, so the "absent file" case becomes a
**staleness** case caught at build time rather than a runtime parse.

## ❓ Q149 — Which sim mechanics do materials drive this pass?

**✔ (verbatim):** `Deterioration + repair tickets, Family as placement
precondition, Build cost + upkeep, Performance effects, even the grass and the
water, every material for every part`

All four, and **every world part gets a material** — terrain, water, roads, zone
paint, every structure, its parts, power lines, scaffolds, ruins.

## ❓ Q150 — At what grain is a world material a claim?

➡️ Per structure, matching the road-ticket pattern already in `gov`: filed before
the build, closed only by re-reading the tile.

**✔ (verbatim):** `Per-structure ticket`

## ❓ Q151 — Does the external validator also judge the world?

➡️ One validator, two subjects.

**✔ (verbatim):** `Icons only` — **overrode.** The world's material truth is checked
*structurally* (record vs world, by `verify.exe`, per Q150 and Q144). The visual
question — does the world read as those materials to a person — is therefore
**open by construction** and is printed as such (Q172), rather than half-answered by
a 3B model.

---

# Round 3 — the world's materials, in data

## ❓ Q152 — How does a material arrive in a world that has none?

`saves/world.ron` is versionless and must keep loading.

➡️ `#[serde(default)]` with the material derived once from the part's own tile, then
recorded on the next save.

**✔ (verbatim):** `Format version + migration` — **overrode.** The world format gets
a version; versionless snapshots read as v1, their materials are derived once by a
named rule, and the file is rewritten as v2. The derivation is reported on load, not
performed silently.

## ❓ Q153 — Which families may the world use?

The matrix is seven families, and `ICON_STANDARD` states seven as a property of the
icon set — but grass and water have no family.

➡️ Declare **world-only** families with their own mechanism and ceiling, the icon
set staying exactly seven, the anchor and level vocabulary shared, and the same
ceiling discipline.

**✔ (verbatim):** `Declare world-only families`

## ❓ Q154 — Which performance effects?

**✔ (verbatim):** all four — `Power conduction + capacity`, `Road speed`,
`Desirability`, `Noise and pollution`.

## ❓ Q155 — What does deterioration actually change?

➡️ A condition value decays by family; a repair ticket restores it; the recorded
material claim stays immutable, because a claim that weathers would make `verify.exe`
report the city's own ageing as a contradiction.

**✔ (verbatim):** `Both` — **overrode, and the risk is accepted explicitly.** A
condition value drives the level used for **drawing**, so the world visibly ages in
the material vocabulary, *and* the recorded claim stays at the **as-built** level.
That is only checkable if the two are separate fields: `material_as_built` is what
`verify.exe` re-reads; `condition` (and the display level derived from it) is what
the frame shows. Verify's rule becomes: the as-built claim must hold; the condition
must be within its declared bounds; a drawn level that disagrees with the condition
is a **renderer** finding, never a claim contradiction.

## ❓ Q156 — How does a family price a structure?

➡️ A declared per-family multiplier on the existing base cost, so the existing
numbers and the road-upkeep test keep their meaning.

**✔ (verbatim):** `Price the parts` — **overrode.** Each part carries its own price
and a structure's cost is the sum of its parts' prices plus its upkeep rate.

## ❓ Q157 — Where does the surface→material mapping live?

➡️ Rust declares which material each of the game's own surfaces is; Python owns the
vocabulary and the resolution; the gate validates the mapping.

**✔ (verbatim):** `Both, cross-checked` — **overrode.** Rust declares the mapping
*and* the generator emits a copy, with a digest check that they agree. The reason
this is not merely redundant: a mapping that exists twice without a check is the
defect Q159 exists to remove, so the second copy is admissible **only** with the
digest that makes drift a build failure.

## ❓ Q158 — What does the world's material gate do?

➡️ Modelled on `design::verify`: refuses to start, reports every defect rather than
the first, and prints what it cannot check.

**✔ (verbatim):** `materials::verify, fails closed`

---

# Round 4 — one home, the promotions, and who decides

## ❓ Q159 — Where does a shared colour live?

`ICON_STANDARD` says no number in two places; `palette.py` transcribes `hud.rs`.

➡️ The generated table becomes the single home for the world's material vocabulary
*and* the world-facing token colours; `hud.rs` resolves those from it instead of
hardcoding them; interface-only tokens stay in `hud.rs`; `palette.py` stops
transcribing and reads the same generated values; both sides record the digest and
fail closed on disagreement.

**✔ (verbatim):** `Generated file is the one home`

## ❓ Q160 — Do the six C7 promotions survive the repair?

`ICON_STANDARD` §8: a concept-set change reopens every target, "reported, never
enforced, because the trigger stays a human decision".

➡️ Bump the concept set; the six re-decide in the picker after the repair.

**✔ (verbatim):** `Bump, re-decide all six`

## ❓ Q161 — Which icons does this pass touch?

➡️ The six, plus `help` — a promise made and not kept is the one that rots most
quietly.

**✔ (verbatim):** `The six pilots (already promoted), ledger, help, tool-zone,
everything` — **widened**, then narrowed again at Q166 by the deferral's own reason.

## ❓ Q162 — At what grain does the validator read?

➡️ Per candidate, narrow declared questions — one blind call per candidate per
question — because that is what a 3B model can answer, and what the repair consumes.

**✔ (verbatim):** `Per candidate, narrow questions`

## ❓ Q163 — Which validator readings may refuse promotion?

`ICON_STANDARD` §6: a judged failure is a note, and a note refuses promotion;
recorded-only items carry their reason.

➡️ Silhouette match and undeclared geometry as judged facts; material reading and
24 px legibility as notes.

**✔ (verbatim):** `Undeclared geometry` only.

## ❓ Q164 — How do we spend Blender time?

➡️ Re-render one repaired icon first (`tool-road`, the 0.3235 failure), measure it,
then re-render the whole reviewed set — a wrong geometry change then costs one
render, not thirty-six.

**✔ (verbatim):** `Smoke test, then the set`

## ❓ Q165 — Who decides which material each part is?

➡️ I propose the whole table (each part of each `BuildingKind`, road, zone paint,
terrain, water, power line, scaffold, ruin → family + anchor + level) in this
record, and you override any line.

**✔ (verbatim):** `Agent proposes, you override`

---

# Round 5 — the zone, the order, the tool, the campaign

## ❓ Q166 — "Everything" includes `tool-zone`, which C7 deferred on its locator

C7 Q133's reason: the tool exists, the locator does not.

➡️ Build the locator, then the icon.

**✔ (verbatim):** `Keep the deferral` — **overrode the agent.** `tool-zone` stays
deferred, so "everything" (Q161) resolves to **eight icons**: the six pilots,
`ledger`, and `help`. The deferral's reason is unchanged and is re-recorded, not
quietly dropped: the zone tool places zones, but *which* zone is in hand is still a
HUD mode rather than a surface with a locator, and an icon may only exist for a
surface that has one.

## ❓ Q167 — What order does this run in?

➡️ Table → icons → world → sim → playtest, because the table is the only piece both
halves must agree on.

**✔ (verbatim):** `Table → icons → world → sim → playtest`

## ❓ Q168 — Where does the validator run from?

➡️ A separate `tools/icons/validate.py` after a render pass, writing an append-only
verdict ledger; `generate.py` reads those as notes and directives; Blender stays out
of the model's loop.

**✔ (verbatim):** `Separate tool, generate.py reads it`

## ❓ Q169 — How many of the four questions does the model get asked?

➡️ All four asked and recorded; only undeclared geometry judged; a disagreement
between the model's reading and the scripted measurement recorded as a **finding**,
never averaged.

**✔ (verbatim):** `Judged plus silhouette` — a label that read as *two* judged
questions against Q163's one. Carried to Round 6 rather than resolved by assumption.

## ❓ Q170 — One campaign or two?

➡️ One campaign with two playtest stages, each playtested before the next starts.

**✔ (verbatim):** `One campaign, two playtest stages`

---

# Round 6 — the last four forks

## ❓ Q171 — Which of the two questions refuses promotion?

Stated plainly, because Q163 and Q169 disagreed on this one point.

**✔ (verbatim):** `Only undeclared geometry refuses` — the silhouette reading is
asked and **recorded as an independent second reading** beside the scripted
containment number; a disagreement is a finding, not a veto, and the script remains
the silhouette judge.

## ❓ Q172 — What does the world's material gate declare as open?

**✔ (verbatim):** all four — whether the world reads as those materials to a person;
frame timing with the new shading; whether the four sim effects are balanced; and
composite contrast on a rendered frame.

## ❓ Q173 — What happens to the existing numbers and tests?

Home 120 / Shop 240 / Factory 400 / PowerPlant 1800, and a test asserting road
upkeep exceeds a young city's tax take.

➡️ Price the parts so the default family's totals equal today's numbers exactly.

**✔ (verbatim):** `Re-tune, correct the tests` — **overrode.** Parts are priced on
their own merits, the balance changes, and every affected test is updated **with a
recorded correction** naming the measurement that no longer holds. The old numbers
are superseded, not silently overwritten: the difference between "the price changed"
and "the price was always this" is exactly what this project keeps records for.

## ❓ Q174 — Where does the generated table land?

➡️ `config/materials.json`, beside `governor.json`.

**✔ (verbatim):** `Generated Rust source` — **overrode.** The generator emits a
`.rs` file compiled into the game: no runtime parse, and no absent-file case. What
that costs is a new failure to design for — a **stale** generated file — and it is
answered in §7 below by a digest test that refuses a build whose generated source no
longer matches the Python that declared it. Q148's principle (fail closed) is
intact; only the artifact changed.

## ❓ Q175 — How does a material claim enter the ticket board?

➡️ Extend the existing claim shape; no new record kind.

**✔ (verbatim):** `New material ticket kind` — **overrode.** `MAT-*` gets its own
kind and its own terminal vocabulary, so material claims can be drained and reported
separately from placement and zone claims instead of hiding inside them.

---

## Corrections ledger for C8

| Was | Now | Because |
|---|---|---|
| The brief's implied premise: an external validator exists to be used | **No validator, no HTTP client, no model call anywhere in `tools/`** — it is built in this campaign | Measured |
| The agent's plan assumed the 5.3 Alpha renderer was gone and prepared to void the six promotions | **5.3 Alpha is installed** (Steam, verified by version call): the render numbers stay comparable and the promotions survive | a146 |
| Q148: a config file read at runtime, absent means refuse | A **generated Rust source**: the absence case becomes a **staleness** check at build time | a174 |
| Q143: sim mechanics "later, once the material truth is proven" | The four sim effects are **in this campaign** | a143, a149 |
| Q151: the validator reads icons and world frames | **Icons only**; the world is checked structurally, and the visual question is printed open | a151 |
| Q152: `serde(default)` and derive the material quietly | **Format version + named migration**; the derivation is reported, not silent | a152 |
| Q155: one condition value, material immutable | **Both**: the drawn level follows the condition, and the as-built claim stays immutable — which is only checkable if the two are separate fields | a155 |
| Q156: a per-family multiplier on the existing base | **Parts carry their own prices**; the structure's cost is the sum | a156 |
| Q157: the mapping lives in Rust only | Rust **and** a generated copy, with a digest that makes drift a build failure | a157 |
| Q161: eight icons — the six plus `help` and `ledger` | `tool-zone` **stays deferred**; "everything" means eight icons, not nine | a166 |
| Q169's label read "judged plus silhouette" | **One judged question** (undeclared geometry); silhouette is a recorded second reading | a171, against a163 |
| Q173: the existing balance preserved by construction | **Re-tuned, with the tests corrected by recorded correction** | a173 |
| Q175: material claims ride the existing kinds | A **new `MAT-*`** kind | a175 |
| `ICON_STANDARD.md` §7: the rig "is currently untracked" | It has been tracked since `b5488e5` | `git ls-files` |
| `README.md`: "no icon has a target yet" | Six targets were recorded in C7 | `review-decisions.jsonl` |
| The documents imply a decision clears the review surface | `review-decisions.jsonl` has 6 targets while `review.json` still lists all six as `awaiting_decision` | Measured; moot for the six once the concept set is bumped (a160), but the loader and the picker disagree until then |

## What each answer settles

1. **One sealed validator interface, local Ollama by default**, invoked as a
   separate tool after a render pass, reading per-candidate and **blind** to the
   scripted checks. Four questions asked and recorded; **one** of them — undeclared
   geometry, the shape the live `tool-power` directive names — may refuse promotion
   as a note. It never promotes.
2. **Repair means the builder can produce a conformant set again**: the plate
   construction, the accent forms, the sliver-enclosing layered body, the two
   containment faults, `enamel` in the blend, and the arrival bands re-derived by
   measurement — over **eight** icons (six pilots + `ledger` + `help`), under
   Blender 5.3.0 Alpha, carried by a smoke render before the set.
3. **A concept-set bump voids the six C7 targets**; the picker is the re-decision
   surface, and the decisions are re-taken by a person.
4. **The material truth has one home** — declared in Python, emitted as a generated
   Rust source, with the world-facing token colours in the same generated source and
   `palette.py` no longer transcribing `hud.rs`. Digest recorded on both sides; a
   stale generated source is a build failure.
5. **The world gets materials**: world-only families beside the icon set's seven,
   every part of every surface assigned one, the mapping declared in Rust and
   emitted for cross-check, a versioned save with a named v1→v2 migration, and a
   `materials::verify` that fails closed like `governor.json` and prints its own
   blind spots.
6. **All four sim effects land this pass** — cost and upkeep (parts priced, balance
   re-tuned, tests corrected by record), deterioration with repair tickets, family
   as a placement precondition, and performance (conduction and capacity, road
   speed, desirability, noise and pollution).
7. **Every placed structure files an `MAT-*` ticket** carrying family, anchor and
   level, closed only by reading the structure back; `verify.exe` learns to
   re-derive the claim, distinguishing the as-built claim from the present
   condition.
8. **One campaign, two playtest stages** — icons and validator, then world
   materials and sim — in the order table → icons → world → sim → playtest.
9. **The uncommitted tree is committed partitioned first.**

---

## 7. Specified by the agent, not asked (and why)

Per C6 Q129/Q130's lesson — *an agent that invented a mechanism tends to hand the
user the specification of it as a question* — the following are **specified here
with their reasoning**, and shown rather than argued. Each is overridable line by
line; an override is recorded as an override.

### 7.1 The one-home direction, which the answers imply but do not name

`hud.rs` stays the home of the **interface** tokens (a148/a159: interface-only
tokens stay). The **world-facing** colours move to the Python declaration, which is
what `palette.py`'s own comment anticipates ("the world palette is not in `hud.rs`
yet"). The generator then emits:

* `src/materials/generated.rs` — the world's families, mechanisms, ceilings, levels,
  anchors, the resolved family×anchor×level table, the world surface→material
  mapping (a157's emitted copy), and the world-facing token colours;
* nothing else. There is no second JSON: the Rust source **is** the artifact, and
  the icon checks read the values through the generated digest.

Because the Python side still needs the **interface** colours to measure contrast
against (Panel, PanelRaised, Desk, Ink, Warning, Nature, TextBody), the direction for
those is reversed: a small Rust binary prints every interface token's resolved
linear values and luminance, and `palette.py` reads that instead of transcribing
`hud.rs`. One home per value, in both directions, with the digest recorded.

### 7.2 The world-only families

The icon set stays exactly seven (a153). The world declares these beside them, each
with a mechanism and a ceiling, in the same shape as `FAMILY_COLOUR`:

| Family | Mechanism | Ceiling | Why it exists |
|---|---|---|---|
| `water` | body volume — colour held in the volume, with depth absorption | 0.100 | water is not glass: a pane and a lake are not one material |
| `organic` | pigment in living tissue — never uniform, always slightly varied | 0.120 | grass, vegetation, the ground's planted half |
| `soil` | aggregate and mineral — near-neutral, matte, granular | 0.060 | bare ground, unpaved terrain, ruins |

`aggregate` (`road`) already covers paving; `soil` is its unpaved neighbour and is
deliberately a different family for the same reason metal and enamel are different:
the mechanism is different.

### 7.3 The parts table (the override surface, a165)

Each structure is priced and drawn as parts. The proposal, to be overridden line by
line:

| Structure | Parts | Family lead | Anchor |
|---|---|---|---|
| Home | walls, roof, window | ceramic, polymer, glass | natural / amber secondary / cyan accent |
| Shop | walls, frontage, window | ceramic, enamel, glass | blue |
| Factory | frame, cladding, vent | metal, enamel, polymer | amber |
| Power plant | frame, stack, insulator, core | metal, ceramic, ceramic, enamel | amber |
| Road | surface, edge | road (aggregate), soil | natural |
| Zone paint | a tint over the surface's own material | inherited from what it paints on | zone anchor |
| Terrain | ground, water, vegetation | soil, water, organic | green / cyan / natural |
| Power line | conductor, pylon | metal, metal (deep) | natural |
| Scaffold | frame, deck | metal, paper | amber |
| Ruin | the retired structure's own parts at `deep` | inherited | inherited |
| Interface hosts | Panel, PanelRaised, Desk | enamel, enamel, polymer | (unchanged; measured by the icon checks) |

**Every entry is a claim**: `family + anchor + level`, resolvable by the gate, and
readable back out of the world by `verify.exe`.

### 7.4 The four sim effects, at their declared grain

* **Cost and upkeep** — a part's price is declared per unit of material; a
  structure's build cost is the sum of its parts' prices, and its monthly upkeep is
  a declared per-family rate. The re-tune (a173) is recorded, with the corrected
  tests naming the measurement that no longer holds.
* **Deterioration** — a `condition` in [0,1] per structure, decaying per day by a
  declared per-family rate and fed by upkeep; below a floor, a **repair ticket** is
  filed and closes by reading the structure's condition back. The drawn level
  follows the condition; the as-built material never moves (a155).
* **Placement precondition** — the declared material of a placement is a
  **precondition** the governor checks before filing the placement ticket; a refusal
  names the family and the rule that refused it (never a silent drop), and is
  recorded as a refusal.
* **Performance** — `conduction` and `capacity` per family (power lines, plants);
  `speed` per road surface on the graph's `edge_cost()`; `desirability` as a
  multiplier on `Demand`; `nuisance` (noise and pollution) as a per-source value
  sampled at read points, so a 65 536-tile field is never recomputed per tick and
  the sim stays tick-deterministic.

### 7.5 The validator's frozen question set

Four questions, asked per candidate, blind, temperature 0, recorded with the model
name, the prompt, the image digest and the render's measurements:

1. *Is the drawn form the declared silhouette?* — **recorded** as a second
   independent reading; a disagreement with the scripted containment number is
   recorded as a finding.
2. *Is anything drawn that the declaration does not name?* — **judged**; a `refused`
   verdict refuses promotion as a note. (This is the live `tool-power` directive,
   turned into a check.)
3. *Are exactly three materials visible, and are they the declared ones?* —
   **recorded**.
4. *Does the mark survive at 24 px?* — **recorded**; the judgement stays the
   person's, and C7's playtest left exactly that open.

### 7.6 The plate construction and the accent forms

Specified as geometry and shown as renders (C6 Q129): the plate is a **tile-filling
body with the mark carried on its face**, and the silhouette check reads against λ —
containment where the mark is the object, occupancy agreement of the mark's region
where it is not (C6 Q130). The six declared accent forms (tab / seal / ribbon / notch
/ band / corner) are built once each, so the sixth axis stops being declared-and-absent.

---

## Still open, and deliberately so

* **`tool-zone`** — the locator half, unchanged by this campaign (a166).
* **Motion** — still settled as store-driven and still unimplemented.
* **Whether the world reads as those materials to a person** — the gate will print
  it as open every run (a172).
* **Frame timing with the new shading** — the material shade term is a change to a
  26 k-quad pass, and it is unmeasured until a playtest measures it.
* **Whether the four sim effects are balanced** — declared, computed, and only a
  playtest can say whether the city is fair.
* **The picker does not clear its own awaiting list** — `review.json` lagged the
  decision ledger; whether the picker should rewrite it or the pipeline should
  regenerate before opening is not settled here, because the concept-set bump makes
  the six awaiting again anyway.
* **A hosted validator model** — the interface accepts one; no key exists and none
  is being bought in this campaign.
