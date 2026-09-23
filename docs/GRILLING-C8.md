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

### 7.7 What the containment floor measures (Q176, a176)

The floor's reading was left open by `ICON_STANDARD` §2, which says "the share of
the declared mark's cells the render must cover" against a 0.90 floor. Measuring
the committed renders showed the check and the geometry were talking about
different frames, and that the two obvious readings of the sentence disagree in
opposite directions. Settled as **both**:

* **Judged — weighted ink.** For every declared cell the mark's region claims,
the render must carry at least the declared share of that cell's *own ink*, and
the icon's number is the ink-weighted mean over the claim. This is the number the
90 % floor applies to, and it is the honest reading of "the share of the mark".
* **Gated — strict cells.** Any declared cell carrying at least half its own ink
in the mark must have at least half of its area covered by the render. This is
binary per cell, is what "cell for cell" means, and **`tool-power` fails it** —
which is a repair, not a reason to soften the gate.

Two defects found while measuring, both in the check rather than the artwork:

* **The mask was binned in file space while the render is placed.** The mark's
  mask was computed in the glyph file's own frame, then compared against a render
  placed by the rig's transform, so the two were only coincident when the glyph
  happened to fill its frame. Fixed by binning the mask through the placement the
  geometry already uses. `tool-road`'s recorded 0.3235 was **this**, not geometry:
  the same render reads 0.9167 once the frames agree. The old file-frame number is
  kept beside it as a recorded divergence rather than deleted.
* **The trace walked pixel centres and Douglas–Peucker shaved its tolerance off
  every corner**, leaving the polygon half a pixel inside the mark. Compensated
  before tracing, so the outline sits on the mark's edge rather than inside it. A
  bevel-removal probe showed the bevel was **not** the cause (strict 0.6667 with
  and without it), which is why the fix is in the trace and not the material.

After both: every icon is containment-clean at every reviewed size, `tool-power`
strict 1.0000 / weighted 0.9933 and `tool-road` 1.0000 / 1.0000.

### 7.8 What the fresh render measured, and what it means

All six icons re-rendered under Blender 5.3.0 Alpha (5eaad57cfabe) in 82 s, four
sizes each, concept set `pilot-six-c8-v1`, so C7's six decisions are void and the
picker awaits again as recorded. Containment notes: **none**. What remains is the
plate and the finish, and the numbers say why:

| icon | glyph | fill at the `plate` slots (md1 / touchwiz / ios6) |
|---|---|---|
| `tool-road` | thin ribbon | 0.54 / 0.47 / 0.50 |
| `tool-power` | bolt | 0.62 / 0.45 / 0.57 |
| `tool-inspect` | ring + handle | 0.50 / 0.46 / 0.48 |
| `tool-demolish` | solid | 0.83 / 0.80 / 0.85 |
| `vocab-settings` | gear | 0.70 / 0.72 / 0.70 |
| `ticket` | blocky | **0.96** / 0.88 / 0.94 |

`fill` is covered ÷ the render's **own box**, so for a mark fitted to its span it
is the mark's own ink. Every `plate` candidate measures its glyph's ink, which is
the proof that **no plate body exists** (7.6, C6 a129): a candidate named `plate`
is the traced mark plus a cylinder. The corpus ladder's three ends are therefore
three *constructions*, not three shades of one:

* **`md1` 0.49, band [0.366, 0.706]** — the object is the mark. Reached today by
  the thin marks (road, power, inspect), not by the solid ones.
* **`touchwiz` [0.784, 0.997]** — a body whose own box it fills to ≈0.95: a
  squircle plate, whose cut corners are the missing area.
* **`ios6` [1.0, 1.0]** — a body that fills its box exactly: a square plate.
* **finish** — the gloss anchors (md1 0.0 ±0.03, touchwiz 0.102 ±0.03, ios6 0.24
  ±0.051) are a property of the material, and the measured candidates sit *below*
  each: touchwiz slots read 0.038–0.064 against 0.072–0.132, ios6 slots 0.094–0.137
  against 0.189–0.291. The finish is a mechanism to set (roughness and coat per
  language), measured rather than asserted.
* **`vocab-settings` hue drift 12.9–14.0°** against the 12.0° tolerance in all six
  generations — a real fault: the mean chromaticity over the covered area is dragged
  by the accent's share, so either the accent's area or the declared colour moves.

A correction owed and recorded: my option text for Q176 described "weighted judged,
strict as a gate too" as letting `tool-power` through. It does not — the strict gate
is exactly what it fails (6 cells at 0.7778). What was chosen stands; the geometry
is what has to change.

### 7.9 A solid glyph and its `md1` slot (Q177, a177)

The `md1` end of the ladder is fill 0.366–0.706 (anchor 0.49). Two of the six
glyphs are intrinsically solid — `ticket` measures 0.956 and `tool-demolish`
0.833 at every `plate` slot — so no body construction brings their marks into
band. Settled: **the `md1` candidate draws that glyph hollowed** (the mark's
interior shows the body through it), which is a form MD1's own archive ships. The
mark's identity is unchanged, the other slots keep the solid form, and the fill
band stays the promise it was rather than becoming a recorded exception.

### 7.10 What a declared hue is measured on (Q178, a178)

`vocab-settings` drifts 12.9–14.0° against the 12.0° tolerance in all six
generations. The check took the mean chromaticity over **every covered pixel**,
which includes the accent's cyan — declared, and a different hue on purpose — so
a legitimate accent dragged the reading. This is the same class of defect as the
containment frame error and is settled the same way: **the hue reading for a
declared colour is taken over the pixels of the material that declares it**, and
the whole-render composite is kept beside it as a recorded number, so a bad
composite is visible rather than silent. No 12° reading in the set is moved by a
material that is not the one being declared.

### 7.11 The finish, wired and measured

The emphasis now owns a **finish** (`LAYERS[...]["finish"]`: md1 `matte`, touchwiz
`polished`, ios6 `skeuomorph`), and the candidate's materials travel with it, so the
six candidates of an icon no longer share one surface. Two things had to be found by
measuring rather than by writing the parameter down:

* **The finish never reached the render.** When a family is in `reference.blend` the
generator *copies* that material and overrides only its base colour, so the blend
owned the whole surface and a declared finish was a name on nothing. `apply_finish`
now applies it to the copy, and the record says which claim owns which part:
`surface_source: reference-blend (metal) + declared finish (matte)`.
* **`matte` had no branch at all**, and would not have bitten even with one: metal's
brushed surface drives Roughness from a node, and a linked socket ignores a default
value. A matte finish therefore *unlinks* the family's variation, which is what
"matte" claims, and the link goes rather than the claim.

tool-road, measured under the same rig, before and after:

| slot | before | after | band |
|---|---|---|---|
| `b-md1-stack` gloss | 0.0602 | **0.0305** | 0.0 ± 0.03 |
| `c`/`d-touchwiz` gloss | 0.0619 / 0.0595 | **0.068 / 0.0688** | 0.072–0.132 |
| `e`/`f-ios6` gloss | 0.1105 / 0.1037 | **0.114 / 0.1176** | 0.189–0.291 |

So the mechanism is live and directional, and **only one of the three anchors is
reached**: md1's, and not everywhere (six md1 notes remain). The reason is now
measured rather than argued: gloss counts the share of pixels above 1.6× the median,
which rewards a **broad lobe**, not a mirror — sharpening the surface covers *fewer*
pixels, so the ios6 distance is not a roughness number away. Closing it needs a
**declared sweep** of the rig's response the way `rig.py` carries its environment
sweep, not another guess. Until then the ios6 slots carry their own note and no icon
is promoted, which is the state the notes exist to make visible.

Still to wire from the decisions above: **7.10's per-material hue reading** (four
`vocab-settings` notes remain, all still measured on the composite), and the plate
body and the hollowed md1 marks (7.6, 7.8, 7.9).

# 8. The shape builder: geometry that interpolates (Q179–Q186)

The directive, in the words it was given: *"there are still other geometry artifacts
in the icons, the shape builder is really bad. when I said interpolate, I mean like
the geometry itself interpolates. So gears can have variable teeth, spacing, etc,
roads can have different line types and lanes."*

## 8.1 What the builder is, measured

Before the round, the builder was measured rather than described:

* **The reviewed icons are not built from parametric geometry at all.**
  `add_pilot(...)` is a *retirement* record — "Retired: registers into
  RETIRED_PILOTS, contributes no inventory entry… its silhouette is discarded and
  its recipes are kept only as history" — so `gear_parts`, `road_parts`,
  `lens_parts`, `pylon_parts` and the `gear_mesh` that already takes `teeth`,
  `tooth` and `radius` are history. `six()` is dead code (defined, never called;
  all icons go through `add_composition` → `six_compositions`). What renders is a
  traced MD1 bitmap plus primitives bolted to it.
* **The gear's spacing is baked as fractions, not declared**: `root =
  radius * 0.76`, `tip = radius + tooth * 0.20`, tooth land at ±0.46/±0.28 of the
  pitch. The three authored gear variants are three hand-picked points
  (8/0.20, 10/0.16, 12/0.12), not a curve.
* **The ladder already claims to interpolate and does not**: it moves four scalars
  (`depth`, `bevel`, `inset`, `finish`) and `construction` is two discrete modes.
* **No geometric validity machinery exists of any kind** — no watertight,
  intersection, manifold or sliver check — and topography is *explicitly*
  recorded, not judged, "because an icon is a mark plus an accent piece, so its
  piece count is the mark's plus the composition's and cannot equal a bare
  glyph's".
* **The measured artifacts**, pieces/holes at 96 px:

  | icon | a-md1-plate | b-md1-stack | c-touchwiz-plate | d-touchwiz-stack | e-ios6-plate | f-ios6-stack |
  |---|---|---|---|---|---|---|
  | `tool-power` | 1/0 | 2/0 | 1/1 | 2/1 | **1/10** | 1/1 |
  | `vocab-settings` | 1/0 | 1/1 | **1/9** | **1/8** | 1/0 | 1/1 |
  | `tool-inspect` | 1/1 | 1/1 | 1/4 | 1/5 | 1/4 | 1/3 |

  plus the standing directive on `tool-power`, in the person's own words:
  *"remove the extra floating geomaetry that has nothing to do twih the icon"*.
* **And the ladder's complexity anchors point the other way**: md1 is 2.0 pieces /
  1.0 hole, touchwiz and ios6 are 1.0 / 0.0. Ten holes is not a near miss.
* The builder can already do what the repair needs: `from_pydata` polygon meshes, a
  **BOOLEAN DIFFERENCE** path (used today to cut the mark's own holes), bevel and
  weighted normals.

## 8.2 The decisions

* **a179 — the geometry is an authored parametric family, and the MD1 mark is its
  reference.** The mark stops being the geometry and becomes the object the
  parameters are measured against. This **supersedes C3's retirement** of the
  authored recipes (whose reason was "the pilot form had no reference object behind
  it") — the reference object now exists and is measured, and the old retirement
  record stays readable rather than being deleted.
* **a180 — the ladder drives everything**: geometry, surface and construction, so
  md1→touchwiz→ios6 is one claim about the object rather than a label on a
  variant.
* **a181 — free topology morph.** Every point on the continuum is a legitimate
  object, and the topology is a consequence of the parameters rather than a switch
  between named stages. (This overrides the declared-stage recommendation.)
* **a182 — all six families approved**: gear, road, lens, pylon, plaque/ticket, each
  parameter declared with its type and its endpoints on λ.
* **a183 — all three defects gate**: an undeclared floating component, an enclosed
  sliver below a measured threshold, and interpenetration each refuse promotion.
* **a184 — boolean cut with clearance, built as one profile**, "similar to how app
  icons are actually made in ios": the object reads as a single profile, and the
  cuts are what make that true.
* **a185 — pieces and holes are judged**, against the anchor at the candidate's λ.
* **a186 — this round is recorded in C8**, which already owns repairing the icon
  builder.

## 8.3 Mechanisms specified rather than asked (each overridable by a word)

* **A part is present because its own parameters are not degenerate.** With a free
  morph (a181) nothing may switch on at a threshold, so every part is always
  declared and its presence is a *consequence*: a plate whose spread reaches zero,
  a ring whose thickness reaches zero, a layer whose depth reaches zero is absent
  at that point on the continuum and grows out of it continuously. Nothing in the
  builder asks "which stage is this".
* **One profile, cut not stacked.** Each object is a single body; the declared
  regions are boolean-cut into it at a declared clearance, and the cutters are
  transient geometry that never renders. This is how *interpenetration gates*
  (a183) and *boolean cut* (a184) coexist without contradiction: the gate judges
  the built shells, and the cut is the mechanism that leaves none to judge.
* **The sliver threshold is measured, not chosen** — from the reference archives'
  own enclosed regions, the way the ladder was sampled, so "sliver" is a corpus
  fact rather than a taste.

## 8.4 Round 2: what the free morph implies (Q187–Q192)

* **a187 — an integer count changes by a feature growing from zero width.** A tooth,
  lane, ring, rule or perforation whose width reaches zero is *absent*, so the count
  is continuous in geometry and the declared integer is the number of
  non-degenerate features. This is a181's degeneracy rule applied to counts, and it
  removes the need for a snap and for a recorded discontinuity.
* **a188 — every λ endpoint is authored**, and the reference archive is recorded
  beside it rather than pinning it. (This overrides the round's recommendation to
  measure the md1 end from the corpus.)
* **a189 — md1's end is declared and checked, not fitted.** The mark is recorded
  beside the authored point and containment stays a **gate**, not a fit target.
  (This too overrides the round's recommendation.)
  *Consequence, stated rather than discovered later:* with the endpoints authored
  and no fitting, each icon's parameters have to be tuned by hand until the
  containment gate passes — which a184's own wording allows ("overlap and iterate"),
  and which is work rather than a decision.
* **a190 — the body is one profile and the accent is the second piece**, so the
  judged complexity is 2 pieces at md1 and 1 at touchwiz/ios6, matching the anchors
  with no exception carved out.
* **a191 — the `plate` | `stack` axis is dropped**, and layering is a consequence of
  λ, because the anchors already measure it that way (md1 1.0 hole / 2.0 pieces;
  touchwiz and ios6 0 holes / 1 piece). The reviewed slots therefore fall from six
  to **three** per icon — the sample points are round 3's question, not assumed here.
* **a192 — the bin's vocabulary is derived from the corpus before it is declared.**
  This is a measurement owed, not a decision taken: `action/delete` is a bin, so its
  own marks are the evidence the family's parameters are read off.

## 8.5 Round 3, and the measurement that challenges one answer

* **a193 — the sliver threshold is an authored number**, recorded as authored, not a
  figure read off the archives. (§8.3 had specified it as measured.)
* **a194 — five reviewed samples per icon**: λ = 0, 0.25, 0.5, 0.75, 1, so the
  interpolations between the three named languages are reviewed as objects. Thirty
  renders; the intermediate anchors come from `_ladder_at`, which already computes
  fill, gloss, pieces and holes at any point on the ladder.
* **a195 — one parametric accent, with the six names as declared parameter windows**
  (C6 a129's tab / seal / ribbon / notch / band / corner become *regions* of one
  continuum: a tab that widens becomes a band, a seal that stretches becomes a
  ribbon). The six names and their windows are kept; the six discrete forms are
  superseded, and this too is a supersession rather than a deletion.

**And the measurement that a193 cannot be implemented as stated.** Void areas in the
committed renders, as a share of the object's covered area at 96 px:

| class | measured examples |
|---|---|
| real openings | `tool-power.c-touchwiz` 69.5%, `tool-road.c-touchwiz` 60.8%, `tool-inspect.b-md1-stack` 39.1%, `vocab-settings.f-ios6-stack` 8.7% |
| defects | `tool-inspect.c-touchwiz` 0.04% (two 1-px voids), `tool-power.e-ios6` 0.16–0.28% (16 voids), `vocab-settings.c-touchwiz` 0.10–0.59% (12 voids) |
| **declared features inside the defect range** | `ticket` perforations 0.67%, 1.14%, 1.36%, 1.43% |

The two classes are bimodal *and overlapping*, and the overlap is exactly where a
perforation sits. So **an area threshold alone cannot separate a declared hole from
a sliver**: set low it passes 16 real defects, set at 3–4% it condemns the ticket's
own perforations. The number has to be a second condition on top of a declaration,
not a substitute for one — put back to the person in round 4.

## 8.6 Round 4 — the frontier closes

* **a196 — the rule is the threshold alone, with no declaration condition**, authored
  in the asked-for band at **3.5% of the object's covered area at the decision
  size**: every enclosed void below it is a sliver, declared or not. The
  consequences are accepted and recorded, not softened:
  * `vocab-settings.c-touchwiz` (3.06%) and `tool-demolish`'s touchwiz slots
    (2.68 / 1.92 / 1.87%) become slivers, so those candidates have to be repaired;
  * **`ticket`'s perforations (0.67–1.43%) are condemned, so they are redesigned
    above the floor** — a visible change to that icon, which is what choosing the
    threshold-only rule buys;
  * the number is authored, lives in one place, and is overridable in a word.
* **a197 — geometry, then re-render and re-decide, then the validator.** This is
  knowingly two concept-set bumps and two decisions for one geometry change, taken
  so that promotion is possible early rather than only after `validate.py` exists.

**A consequence flagged for measurement, not assumed.** Enlarging a hole *removes
ink the mark has*, and containment is measured over the mark's declared cells, so a
larger perforation can lower containment on the very candidate the sliver rule just
cleared. The containment floor and the sliver threshold therefore have to be
reconciled on the same candidate at implementation time, with numbers, rather than
believed to be independent.

The frontier is empty: every branch of this tree has been visited and recorded, and
nothing is left silently assumed. Implementation starts with the measurement owed
(a192's bin vocabulary, read off `action/delete`'s own marks).

## 8.7 The reference marks, measured (the evidence a192 owed)

`tools/icons/_measure_marks.py` reads each icon's declared mark through the same
loader the checks use (`shapes._load_glyph`, alpha ≥ 0.5, top-down) and prints what
the object behind the form actually measures. All six at 96 px:

| icon | mark | ink | box | pieces | voids | what the object measures |
|---|---|---|---|---|---|---|
| `vocab-settings` | `action/settings` | 3186 (34.6 %) | 74×76 | 1 | 1 (646 px) | **6 teeth** (6 runs at 0.80 and 0.92 of R), bore ≈ 0.39 R |
| `tool-road` | `maps/add_road` | 1632 (17.7 %) | 76×76 | 6 | 0 | the ribbon arrives in 6 components; centre column 2 runs, 89 % ink |
| `tool-power` | `action/power_settings_new` | 1548 (16.8 %) | 72×72 | 2 (ring 1228 + stem 320) | 0 | **a power symbol**: a ring and a stem — no arms, insulators or mast |
| `tool-inspect` | `action/search` | 1358 (14.7 %) | 69×70 | 1 | 1 (1018 px) | a magnifier: a thin ring, bore ≈ 0.51 R |
| `tool-demolish` | `action/delete` | 3204 (34.8 %) | 56×72 | 2 (body 2664 + lid 540) | 0 | lid separated from the body by a gap, taper 0.68, **no ridges** |
| `ticket` | `confirmation_number` | 4672 (50.7 %) | 80×64 | 1 | 3 × 64 px | three notches, each **1.37 %** of the ink |

**Four places where this corrects the vocabulary I proposed in round 1:**

* **gear** — I proposed teeth 8–14, and the reference object has **6**. A range that
does not contain the object it is read from is a range that fails its own gate.
* **lens** — my list had no bore ratio at all, and the bore (0.51 R with a thin ring)
is the measurement that defines the object.
* **bin** — the reference has **no ridges** and its lid is separated by a gap, so "lid
gap" is a parameter rather than styling.
* **ticket, and this one is load-bearing** — the reference's three notches each
measure **1.37 %** of the ink, so **the reference object itself fails a196's 3.5 %
threshold**. Enlarging them to clear the floor means departing from the mark that
containment compares against, and enlarging a notch *removes ink the mark has*: the
interaction flagged in 8.6 now has numbers, and it is a real redesign of that icon
(≈ 64 px → ≥ 150 px per notch) rather than a check to adjust.

**And one conflict that has to be put back to the person (round 5, Q198).**
`tool-power`'s declared mark is a **power button** — a ring with a gap and a stem, two
components, 72×72, nothing else. Two problems follow. First, the retired recipe for
that icon was a **pylon**, which is also what the game means by `Tool::Power` (it
places power plants and lines), and a pylon has no reference object behind it at all
— which is precisely C3's stated reason for retiring the authored recipes. Second,
a power button *is* the reading that icon's own declaration forbids: its
`forbidden_readings` are "authority grant" and "a live supply reading", and a
symbol whose whole meaning is "switched on" reads as exactly the second one. So the
mark cannot simply be pointed at as the reference for that icon's shape.

## 8.8 Round 5 — the reference conflicts, resolved

* **a198 — `tool-power`'s object is the bolt, and its declared silhouette is
  re-pointed** from `action/power_settings_new` to **`content/bolt`** (measured:
  40×72 px, aspect 0.56, 1 piece, 0 voids). The family becomes a **bolt**. Two
  reasons, both recorded on the icon: the power button *is* the switched-on reading
  that icon's own `forbidden_readings` name ("a live supply reading"), and the pylon
  the game means had no reference object behind it — which is C3's own stated reason
  for retiring the authored recipes, so choosing it would have re-opened the fault
  the rework exists to close. From the next render the containment check for that
  icon compares against the bolt; the old mark's measurements stay readable in
  `_measure_marks.py`'s output rather than being overwritten.
* **a199 — the 3.5 % rule keeps no exemption, and the ticket's notches are enlarged
  above the floor.** The reference's own notches (64 px, 1.37 % of its ink) are
  condemned by the rule, so the geometry draws them **≥ 150 px at 96 px** (≈ 2.3×
  MD1's own) — a visible departure from the reference object, accepted as such — and
  the **containment divergence that follows is recorded, not fixed**: enlarging a
  notch removes ink the mark has, so the ticket's candidates carry a named
  divergence the way `Token::Road`'s and `Ink`'s do, with the number beside it.
* The vocabulary corrections in 8.7 stand as measured: the gear's teeth range must
  contain the object it is read from (MD1 draws **6**, so the range opens there
  rather than at 8), the lens gains the **bore ratio** (≈ 0.51 R) that defines it,
  and the bin gains the **lid gap** the reference actually has (ridge count 0 at the
  md1 end, since MD1's mark draws none).

## 8.9 The supersessions, written as records

Two decisions in this campaign replace earlier ones. Both are written here as
*supersessions* — what covered what, on what evidence — because the earlier records
are the reason the new ones can be justified, and deleting them would erase the
argument:

* **C3's retirement of the authored geometry recipes, superseded by a179.**
  `add_pilot(...)` retired `gear_parts`, `road_parts`, `lens_parts`, `pylon_parts`
  and their siblings into `RETIRED_PILOTS` with the reason *"the pilot form had no
  reference object behind it; its silhouette is discarded and its recipes are kept
  only as history"*. That reason was correct then and is answered now: the reference
  objects exist and measure (8.7), so the recipes return as the live geometry, read
  off those objects rather than invented. `RETIRED_PILOTS` stays exactly as it is —
  a retirement record is a record of why something stopped, and this one is what
  stops the same fault from being re-introduced silently.
* **C6 a129's six discrete accent forms, superseded by a195.** Tab, seal, ribbon,
  notch, band and corner were declared as six forms; they become six **named
  windows** on one parametric accent, so a tab that widens is a band and nobody has
  to decide where one ends. The six names, their intent and their renders stay
  readable; what changes is that they stop being six objects.

Both are recorded here rather than in a changelog, because the standard's own rule
is that a record is superseded by naming it, never by overwriting it (§8).

## 8.10 The families, built and smoked (Q179–a199 realised)

**Built.** `tools/icons/families.py` declares the six families — 42 parameters, each
with a type, a source and its endpoints on λ; `check()` is clean and reports 9
endpoints that are measurements of the reference object rather than choices.
`shapes.gear_body` and `shapes.road_body` build one body from a vector, with the
bore cut as a **declared void** (`_cut`) and overlapping features **welded**
(`_weld`), both after baking the modifier stack so the result is the declared one
rather than a function of stack order. `generate.topography` now records
`hole_shares` (each enclosed void's area over the object's coverage) and
`void_sites` names where a void is — because a refusal has to name the fix, and for
a sliver the fix is a place in the frame.

**Smoked**, both families at the five sampled λ, rendered under the real rig and
measured with the pipeline's own functions:

| family | λ | cover | fill | pieces | voids | void shares | count |
|---|---|---|---|---|---|---|---|
| gear | 0.00 | 0.291 | 0.459 | 2 | 1 | 23.55 % | 6.00 → (6, 0.0) |
| gear | 0.25 | 0.308 | 0.473 | 2 | 1 | 17.51 % | 7.50 → (7, 0.5) |
| gear | 0.50 | 0.320 | 0.485 | 2 | 1 | 13.00 % | 9.00 → (9, 0.0) |
| gear | 0.75 | 0.337 | 0.517 | 2 | 1 | 8.89 % | 10.50 → (10, 0.5) |
| gear | 1.00 | 0.349 | 0.557 | 2 | 1 | 6.46 % | 12.00 → (12, 0.0) |
| road | 0.00 | 0.494 | 0.739 | 1 | 0 | none | 2.00 → (2, 0.0) |
| road | 0.25 | 0.519 | 0.765 | 1 | 0 | none | 2.50 → (2, 0.5) |
| road | 0.50 | 0.573 | 0.730 | 1 | 0 | none | 3.00 → (3, 0.0) |
| road | 0.75 | 0.601 | 0.750 | 1 | 0 | none | 3.50 → (3, 0.5) |
| road | 1.00 | 0.609 | 0.759 | 1 | 0 | none | 4.00 → (4, 0.0) |

So the interpolation is real: the count moves through a **partial feature** (7.50
reads as seven teeth and one at half width), the gear's declared bore shrinks from
23.6 % to 6.5 % along λ — tracking its measured 0.39 → 0.22 bore ratio — and no
family produces a sliver in its own body at any point.

**Two findings, both measured, both other people's decisions:**

1. **The only sliver the smoke test found was the accent's, not the object's.** A
   one-pixel void at (68, 76) — outside the gear's 37 px radius — was the accent
   cylinder overlapping the body. That is the *same defect class the current review
   set carries* (one committed candidate has 16 voids), so the cause is now on the
   record: **the accent's clearance**. And a fixed accent position does not work:
   at three lanes and up the road grows into it and the two weld into one piece
   (pieces 1, not 2). The placement has to come from the body's own bounds with a
   declared clearance, or the body has to cut a declared socket for it.
2. **The md1 fill band is a population statistic, and three of the six reference
   objects fall outside it.** The road's own reference mark is a thin ribbon —
   ~28 % of its box, fill ≈ 0.28, *below* the band's 0.366 floor — while the bin
   (0.80) and the ticket (0.95) are *above* its 0.706 ceiling. The authored slab
   measures 0.74–0.83 for the same reason the traced candidates did: it fills its
   own box because it is a slab. So judging an individual object against the
   corpus's interquartile range asks the road to be a stroke drawing and the bin to
   be something other than a bin. Put to the person in round 6.

## 8.11 a200 and a201 wired, and what the wiring exposed

**a201 is implemented.** `POPULATION_RELATIVE_SPREAD = 0.347` (the md1 population's
IQR half-width over its median: 0.170 / 0.490) widens each family's **own measured
fill** into an envelope, `fill_envelope(family)` is what a candidate is judged
against, and `check()` refuses a family that records no fill. The corpus band stays
recorded as the catalogue-wide claim. The envelopes, against the reference
measurements:

| family | reference fill | envelope |
|---|---|---|
| bin | 0.7946 | 0.519–1.000 |
| bolt | 0.3767 | 0.246–0.507 |
| gear | 0.5665 | 0.370–0.763 |
| lens | 0.2812 | 0.184–0.379 |
| plaque | 0.9125 | 0.596–1.000 |
| road | 0.2825 | 0.184–0.381 |

**Four of the six reference objects fall outside the corpus band** (road 0.283 and
lens 0.281 below its 0.366 floor; bin 0.795 and ticket 0.913 above its 0.706
ceiling), which is the measurement that made a201 the right answer rather than a
convenience.

**a200 is implemented.** `COMPOSITION` declares the frame (2.25), the body's span
(1.44), the accent radius (0.15) and the clearance (0.06); `accent_slot(bounds)`
derives the placement from the body's own bounds; the family builders take the span
from that declaration. Re-smoked: **pieces = 2 at every λ for both families and no
sliver anywhere** — the defect class that produced 16 voids in one committed
candidate is now structurally absent rather than repaired per icon.

**Two things the run exposed, both put back to the person (round 7):**

1. **A straight slab cannot be inside its own envelope.** The road's `fill` was
   0.556 → 0.719 while its reference's is **0.2825** — and the reason is now clear:
   `fill` is measured against the object's *own box*, and the reference mark is a
   **fragmented, angled** carriageway whose box is the full square while its ink is
   17.7 % of it (6 pieces, dashes drawn separately). A solid axis-aligned ribbon
   fills its own box almost completely whatever its width, so no amount of thinning
   reaches the envelope. Matching the object means drawing the dashes as **full-width
gaps cut into the carriageway** — which connect to the background, so they are not
   enclosed voids at all, and which is also what fragments the mark into its six
   pieces.
2. **a190's piece counts and a200 cannot both hold.** a190 promised 2 pieces at md1
   and **1 at touchwiz/ios6** (the anchors: 2.0 / 1.0 / 1.0), while a200's chosen
   description says a separate accent has room at **every** λ. A separate accent is a
   separate component in a raster, so 2 at every λ is what the geometry gives — and
   the reference objects themselves carry 1, 2, **6** and 3 pieces, so the anchor's
   piece counts are a population reading in exactly the way the fill band was.

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
