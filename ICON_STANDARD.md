# ICON STANDARD

The specific half of the icon doctrine. [`UNIFIED_DESIGN.md`](UNIFIED_DESIGN.md)
states what binds *any* surface — that an icon is MD1's silhouette, TouchWiz's
surface and colour, iOS 6's accent, and that a colour must be physical. This file
states the numbers, the file names and the checks, so the two documents are read
together: doctrine in one, implementation in the other, and no number in two places.

The rules here were derived in `docs/GRILLING-C3.md` (Q70–Q98) and
`docs/GRILLING-C4.md` (Q99–Q112). Where this file and those records disagree, the
records are the history and this file is the current rule.

---

## 1. What an icon is

One icon is three simultaneous roles, not three competing designs:

| Role | Source | What it decides |
|---|---|---|
| **Silhouette** | MD1 `materialicons` / `baseline` | The form. Traced from the reference mark, never redrawn by eye |
| **Surface and colour** | TouchWiz | The material a body is made of, and why it is that colour |
| **Accent** | iOS 6 | The record piece — expressed as **both** material parameters and geometry |

Two supporting rules. Every icon carries **exactly three** main materials, so
"three materials" is a property of the set rather than a slogan about it. And no
count is ever baked into an icon: a number rides a live text slot (§12.9 of the
design doc), because a baked number cannot update and would read as state.

**Forbidden readings** are declared per icon and checked as text: an icon is a
locator to a surface or a record, never a capability grant and never a live
powered/refused state.

---

## 2. The silhouette

| Item | Value |
|---|---|
| Reference | `assets/reference/material-design-icons-4.0.0/png/<category>/<glyph>/materialicons/` |
| Style | `materialicons` — the filled `baseline` mark |
| Density | `48dp/2x` — **96×96 px**, the decision size exactly, so neither side is scaled |
| Checked at | 96 px |
| Containment floor | 0.90 — the share of the declared mark's cells the render must cover |

The check is **containment, not equality**: every cell the declared mark covers must
be covered by the render, and the render may add, because the accent piece leaves the
mark's box by design. Occupancy agreement is recorded beside it so a reader can see
how much was added.

The reference file and its digest are recorded per icon in the manifest.

---

## 3. The material matrix

Seven families × seven declared hue anchors × three levels, plus ink / white / black
as neutrals. Every material an icon may use is a *named* family at a *named* hue, so
"made of metal" and "made of a hue nobody declared" can be told apart by a script.

**Levels** are declared lightness offsets on the family's own lightness —
`body 0.0`, `edge +0.10`, `deep −0.18` — held inside `(0.05, 0.97)`, so a body and
its edge are one material read twice rather than two materials.

**How colour arrives.** Each family has a named mechanism and a **ceiling**: the most
chroma that mechanism supports. Exceeding a ceiling is not a stronger colour, it is a
different material.

| Family | Mechanism | Ceiling |
|---|---|---|
| metal | anodised film — an oxide layer on the substrate | 0.060 |
| glass | body-tinted — colour held in the glass itself | 0.100 |
| ceramic | fired glaze over the body | 0.050 |
| polymer | pigmented resin, compounded in | 0.050 |
| paper | dyed stock, pigment in the sheet | 0.060 |
| road | aggregate — near-neutral because that is what it is | 0.040 |
| enamel | painted colour — the family paint belongs to | 0.190 |

A metal body therefore stays near-neutral and stays a metal; a saturated body is a
pigmented coating and belongs to `enamel`. `colour_mechanisms()` refuses a table
whose ceilings disagree with the chromas actually in use, so the claim and the
number cannot drift apart.

**Hue anchors** are declared, and each records what backs it — the product's own
declared hue where one exists, and honestly a *slot* where none does:

| Anchor | Backed by |
|---|---|
| 25° red | `Token::Refused` — destructive |
| 62° amber | `ZoneIndustrial` / `Warning` / `Scaffold` — caution, power |
| 115° yellow | **declared slot** — no reference, no product meaning |
| 150° green | `Nature` / `ZoneResidential` |
| 200° cyan | partially — the TouchWiz `people`/`store` classes; no product token |
| 250° blue | `Ink` / `ZoneCommercial` — data, commercial |
| 300° violet | **declared slot** — no reference, no product meaning |

**Gamut.** Every entry is resolved inside linear sRGB and every chroma reduction is
recorded. A declared colour that had to be clamped was never the colour declared:
the shipped settings blue declared chroma 0.19 and clamped a negative red channel to
zero, i.e. 0.034 of chroma it never had.

---

## 4. The corpus ladder

The three design languages are **one measured ladder**, not three opinions. Sampled
from the archives on disk at 96 px — 137 MD1 marks, 125 TouchWiz drawables, 97 iOS 6
files:

| Language | fill (median) | fill IQR | gloss (p75) | pieces | holes | digest |
|---|---|---|---|---|---|---|
| MD1 | 0.490 | 0.366–0.706 | 0.000 | 2.0 | 1.0 | `2fe369f8396434d8` |
| TouchWiz | 0.937 | 0.784–0.997 | 0.102 | 1.0 | 0.0 | `c58b07e5b327bb0f` |
| iOS 6 | 1.000 | 1.000–1.000 | 0.240 | 1.0 | 0.0 | `e92be36568d1ab29` |

**The statistic differs per parameter on purpose.** Fill and complexity are central
tendencies, so their anchor is the median. Gloss's median is *zero* in two of the
three archives — a highlight is a minority of pixels even in a glossy icon — so
gloss's anchor is its 75th percentile.

**The tolerance is the corpus's own spread**, not a number chosen to make a check
pass: fill uses each archive's interquartile range, because that is what the
reference family itself spans.

Two caveats bound what this measures. MD1's row is achromatic and gloss-free **by
construction**, because the population is the `baseline_black` marks; Material
Design's coloured product icons would move it. And the anchors are close to
collinear — fill and gloss rise while complexity falls — so this is *one* axis, which
is why the six slots need a second, independent construction axis.

**Licence: measurement only.** Only the MD1 archive is derivative-safe (Apache-2.0).
The TouchWiz archive is an unpacked Samsung APK and the iOS 6 set is a community
archive asking for credit. The archives supply **numbers, not shapes** — no geometry
is ever taken from either. `UNIFIED_DESIGN.md` §1 independently bans "icon cloning"
for the same reason.

---

## 5. The six slots

Six candidates per icon: **three ladder anchors × two constructions**.

| Slot | Emphasis | Ladder | Construction |
|---|---|---|---|
| a, b | silhouette-led | λ = 0.0 (MD1) | single body / layered |
| c, d | material-led | λ = 0.5 (TouchWiz) | single body / layered |
| e, f | accent-led | λ = 1.0 (iOS 6) | single body / layered |

`λ` is *nominal*: each language sits at its declared position because the three
archives are anchors rather than evenly spaced samples. The builder takes a
continuous `λ`, so a later icon can use any point on the ladder without new code.

The layered construction must stay **thin**: layers are bounded as a fraction of the
body depth, and the check that matters is rendered rather than declared — the
substrate material must remain identifiable in a stated share of the stacked area,
because a stack that hides the material beneath it is a sandwich, not a finish.

A third of the six is the *separations* rule: no two candidates may be one reading.

---

## 6. What is judged, and what is only recorded

**Judged** — a failure is a note, and a note refuses promotion:

| Check | Floor / band |
|---|---|
| Silhouette containment at 96 px | ≥ 0.90 |
| Non-text contrast on every declared host | ≥ 3.0:1 (WCAG 1.4.11) |
| Fill at 96 px, against the slot's anchor | the anchor's own IQR |
| Gloss at 96 px | the anchor's value ± tolerance (floor 0.03) |
| Clipped pixels at 96 px | declared ceiling — a blown highlight is detail not in the file |
| Candidate separation | no pair within 10% of the ladder's span on **both** judged axes |
| Hue drift from the declared colour | within the declared angular tolerance |
| Coverage at 24 px | above the recognition floor, below the blob ceiling |

**Recorded, not judged**, with the reason written into the manifest:

- **Piece count and hole count.** An icon is a mark *plus* an accent piece, so its
  piece count is the mark's plus the composition's and can never equal a bare
  glyph's. Judging it would fail every compliant candidate, and a check that cannot
  pass teaches everyone to ignore the notes.
- **Chromaticity of a near-neutral colour**, where the red/green angle is noise.
- **Swatch-to-albedo for a near-metal material under a black world** — retired, not
  skipped: the world now has radiance, so a metal swatch measures the material again.
- **An accent drawn on the icon's own ink**, which owes contrast to the ink, not to
  the host.

---

## 7. The rig

`assets/icons/reference.blend` **is the rig**. It must be committed: the renders are
not verifiable without it, and it is currently untracked.

The world is a grey environment at strength **0.40**, and that number is measured,
not chosen. Under a black world a mirror has nothing to reflect, so the metal family
rendered at 0.075–0.088× its own albedo whatever the lights did. Every icon declares
`Panel` *and* `PanelRaised`, and the floor is set by the **brighter** host, not the
darker one: `PanelRaised` needs p75 ≥ 0.1894, which a grey world reaches at 0.40
(3.33:1 there, 4.06:1 on `Panel`).

The blend owns the **surface** and the matrix owns the **colour**. Reading the
colour back out of the blend, which this pipeline did, made the blend the owner of
both: four icons declaring four hues rendered one grey, and the manifest recorded
that grey as the declared colour.

The blend currently carries **six of seven families** — `enamel` has no
`icon_ref:material:enamel` and has been falling back to the declared table. Every
material records `surface_source` and `colour_source` so a fallback cannot
impersonate the reference.

---

## 8. The artefacts

| Path | What it is |
|---|---|
| `assets/icons/reference.blend` | The rig. Materials and lights only, never icon geometry |
| `assets/icons/review.json` | What the picker reads: candidates, briefs, measurements, checks |
| `assets/icons/manifest.json` | The full record, including the rig, the ladder, the matrix provenance and every check |
| `assets/icons/review/` | The renders: PNG for people, RGBA8 for the runtime, at every reviewed size |
| `assets/icons/review-decisions.jsonl` | Append-only: every target and every comment, with the concept set it belongs to |

**A concept-set change reopens every target.** Only records from the current set may
promote an asset, which is what makes a materials, spec or rig change void the
targets recorded under the old one — *reported, never enforced*, because the trigger
stays a human decision.

---

## 9. Still open

- **`Tool::Zone`** has no MD1 glyph and no product token; it stays deferred until
  the game has a zone surface to locate.
- **`tool-road`'s containment** fails at 0.3235 and `tool-power`'s at 0.857, against
  a 0.90 floor. Whether road's is a trace fault or the comparison's normalisation for
  a mark whose ink does not span its own tile is not yet isolated.
- **The plate construction.** The conformance pass proved that fill cannot rise from
  0.49 to 0.94 by adding decoration to a thin glyph: reaching the TouchWiz end needs
  the object to *become* a plate. Until that exists, the material-led and accent-led
  slots cannot be conformant.
- **The layered construction encloses slivers** — 20 holes on one candidate where the
  anchors predict 0–1.
- **The arrival bands** are still the pre-world values and are owed a re-derivation by
  measurement.
- **The accent forms** (tab / seal / ribbon / notch / band / corner) are declared as
  the sixth axis and not yet built.
- **The loader** that would consume a promoted set.
