# Grilling — C5: the picker, and where the doctrine was missing

C5 exists because the icon review picker — the tool every icon decision is made in —
was declared unusable after the C4 conformance pass, and because the design doctrine
it was compiled against had no section governing it.

Two things came out of the session. A **measured diagnosis** of the tool, which was
not a styling problem, and a **document audit** that found the repository holding
three incompatible partitions of its own design sources. Both are recorded here, and
both produced changes to `UNIFIED_DESIGN.md` rather than to code alone.

**Convention, as in `GRILLING.md`, `GRILLING-C2.md`, `GRILLING-C3.md` and
`GRILLING-C4.md`:** `➡️` is the recommendation made. `✔` is the answer received.
Where an answer **overrode** a recommendation it says so, and one answer in this
session overrode the *approach* rather than the content — recorded because that is
the more useful correction. Question numbers continue C4's sequence; C4 ended at
Q112.

---

## What prompted it

> this ui is fucked up it needs to be redone using the new grilling documents grill me

The fact-finding that preceded the first question, from `src/bin/pick.rs` (2081
lines), `src/design.rs` (450) and `Cargo.toml` — evaluated at 100 % scale with the
real design numbers (`Space::Xs/Sm/Md/Lg/Xl = 4/8/12/16/24`, `Step::Small/Title =
14/20`, `tile = 96 × 3 = 288`):

| Claim under test | Measurement | Verdict |
|---|---|---|
| "the ui is fucked up" | `Layout::new` reserves **494 px** for a row; `row_offset()` advances **712 px** | Two different numbers for one thing |
| …which is what makes it look wrong | row 2's tiles are at 806–1094; `comment_top` is **1078** | The comment box is drawn **over** the bottom of the second row's images |
| can the user scroll to the rest? | No scroll handling exists anywhere in the file — `wheel`, `scroll` and `offset_y` have **zero** occurrences | Row 2's checkbox (1102–1150) and context plate (1174–1246) are past the 900 px window and unreachable |
| …so the comment field? | It sits at y = **1078**, 178 px below the window edge | The mechanism for recording *what the next generation should change* has never been visible |
| is the design system the problem? | `design.rs` implements closed type and space scales, 48 px targets, and a `verify` that fails closed | **No** — the scales are sound; the layout engine is three ad-hoc constants that never agreed |
| is it a framework-free choice? | No UI crate in `Cargo.toml`; `cosmic-text` recorded as "considered and deferred" | Confirmed — there are no primitives to fall back on |
| latent defect | `row_offset` calls `UiScale::default()` (1.0) inside a calculation whose other terms used the real scale | The row pitch is wrong by construction at 125/150/200 % |

**And the document audit**, because a114 asked for exactly this. `UNIFIED_DESIGN.md`
was 693 lines across 17 sections:

| Finding | Count |
|---|---|
| `icon` | 5 occurrences, **all incidental** (a grid line, "icons jiggle", the §12.9 locator table) |
| `chroma` | **0** — no colour-mechanism doctrine anywhere |
| `corpus` / `corpora` | **0** — the ladder and its checks were not doctrine |
| `picker` / `review tool` | **0** — the tool was compiled against AGENTS.md and governed by no section |

And the structural finding: **three competing partitions of the design sources** —
this file's "four lineages", the README's "five pillars", and C3/C4's three roles per
icon — with nothing reconciling them. The document also contradicted *itself*: §1
demoted Popit and Advanced Warfare to "pattern libraries, not extra lineages that can
outrank the four", while §17's own crosswalk gave both of them columns equal to every
other source.

---

# Round 25 — what the picker is for, and what it is made of

## ❓ Q113 — What is the picker's job, in one sentence — and therefore what must be on screen at once?

The implicit answer was "everything, always, at 288 px", which is what produced
1172 px of content in a 900 px window. **(a)** a **chooser** — six images always
visible because comparison *is* the job, everything but a one-line label on demand;
**(b)** a **review console** — a document per icon, read top to bottom; **(c)** a
**workbench** — one candidate large, the other five as thumbnails.

➡️ **(a)**, with (c)'s focus behaviour for the detail: six tiles stay, the
per-candidate note text moves to one detail region for the selected candidate, and
the comment box becomes a fixed footer — about 700 px at 100 % scale.

**✔ (verbatim):**

> ➡️ (a), with (c)'s focus behaviour for the detail. The task is picking between six,
> and the picker's own header already says so; everything that competes with the six
> images for pixels is text about candidates you are not choosing. Concretely: six
> 288 px tiles stay, the per-candidate note text moves to one detail region for the
> selected candidate, and the comment box becomes a fixed footer. That puts the
> content height around 700 px at 100% scale and leaves room for a scroll for the rest.

## ❓ Q114 — Which pillar owns which part of the picker?

**(a)** MD1 owns metrics and grid, iOS 6 owns behaviour and clarity, Impeccable owns
craft, Popit owns the picker as a tool surface, and TouchWiz is **not** a picker
pillar — it is the icons' material language, which the picker displays; **(b)** all
five apply, including TouchWiz material language in the picker's own chrome; **(c)**
the picker is an internal tool and exempt.

➡️ **(a)** — the only reading where the pillars are a division of labour rather than a
mood board, and it says the picker's chrome should be quiet and flat so the six
images are the material-rich thing on screen.

**✔ (verbatim):**

> all, double check the unified design.md doc for the missing entries past the
> currently outlined design specs and integrate them into the system's and icons designs

**Override, and scope added.** All six apply, not five — and the answer adds a piece
of work the recommendation did not contain: audit the design document for what is
missing beyond the outlined specs, and integrate it into **both** the system design
and the icon design. That instruction produced the audit table above and, later,
`ICON_STANDARD.md`.

## ❓ Q115 — Substrate: build the missing primitives by hand, or take a UI crate?

**(a)** keep the hand-rolled wgpu stack and build the primitives, because the game HUD
needs the same ones; **(b)** an immediate-mode crate for the picker only; **(c)** a
crate for both, eventually replacing the HUD's hand-drawn panels.

➡️ **(a) if the HUD needs them, (b) if not** — a checkable fact rather than a
decision.

**✔ (verbatim):**

> c both, the old rust ui is sloppy and dumb, it was supposed to be web gpu but it's
> not supposed to look like garbage

**Settled.** Both surfaces share one layer. And the constraint inside the answer is
the load-bearing part: it is *supposed* to be WebGPU (which `wgpu` already is) and it
is **not** supposed to look like a developer tool — so whatever is adopted must serve
the visual language, not replace it.

---

# Round 26 — the doctrine, and the missing entries

## ❓ Q116 — How do the three taxonomies reconcile?

**(a)** they are three *levels* of one statement: **lineages** are where rules come
from, **pillars** are who owns an aspect of a surface, **roles** are who owns a part
of one artefact; **(b)** promote Popit and Advanced Warfare to full lineages, making
six; **(c)** collapse to one list and lose the role partition.

➡️ **(a)** — the only reading where all three documents are true at once.

**✔ (verbatim):**

> a

## ❓ Q117 — Where do the missing entries live?

**(a)** `UNIFIED_DESIGN.md` gains normative sections for icons and for tool surfaces
while `ICON_STANDARD.md` holds the specifics; **(b)** everything icon-related in the
icon standard, with a link; **(c)** fold icons into §3 and give tools a new section.

➡️ **(a)**, splitting at normative-versus-specific.

**✔ (verbatim):**

> a: but with the demoted popit and advanced warfare elemets, ignoring what the
> document says about promoting or demoting those styles, since those are what we are
> looking for as the final key

**Read as:** the split is accepted, **and** Popit and Advanced Warfare are promoted —
not because a ranking says so, but because they are the **target** the product is
aiming at. This is the answer that turned §1's hierarchy into a correction object
rather than an amendment.

## ❓ Q118 — What does "a UI crate" mean, given it must still look like the doctrine?

**(a)** adopt an immediate-mode toolkit and theme it hard; **(b)** adopt **mechanism**
crates — `cosmic-text` for shaping, wrapping and measured line heights — and keep the
wgpu pipeline drawing, adding a layout pass that computes height before it draws;
**(c)** build our own retained widget tree on wgpu.

➡️ **(b)** — the failure was never drawing, it was that nothing could *measure* text
or scroll; `wgpu` is already WebGPU, so that part is intact.

**✔ (verbatim):**

> b

## ❓ Q119 — The four declared UI scales double all content. What bounds the layout?

**(a)** the layout computes its own height and always scrolls, designed so nothing
*needs* scrolling at 100 % on 1400×900; **(b)** a declared minimum window with a
fit-to-window zoom below it; **(c)** design for 100 % only and drop the other scales.

➡️ **(a)** — (c) contradicts §2 and the scales the design system already declares.

**✔ (verbatim):**

> a

## ❓ Q120 — What is in the detail region for the selected candidate?

**(a)** the brief, its notes, and its **measured numbers against the anchor** — fill,
gloss, containment, contrast; **(b)** brief and notes only; **(c)** notes only.

➡️ **(a)**, with the six candidates' fill and gloss compared side by side, because
"two slots, one reading" is invisible in prose and obvious in a four-number row.

**✔ (verbatim):**

> a

---

# Round 27 — ownership, and the picker's obligations

## ❓ Q121 — What is the mechanical form of "no ranking"?

**(a)** one table of six sources at equal rank, each row stating *what it owns* and
*what it may not decide*, the bounds column kept; **(b)** six sources with no bounds
column; **(c)** keep the rank language and move Popit and Advanced Warfare up.

➡️ **(a)** — bounds are a division of labour, not a rank, and several bans in that
column are the only place a rule is written down.

**✔ (verbatim):**

> man I'm just asking you to include the popit and advanced warfare design conventions
> into the same system with ios 6 touchwiz md1 and apple hig not deliberate over prose
> docs

**Override — of the approach rather than the content.** The question was answered by
being *rejected*: the instruction was to include six sets of conventions in one
system, and turning that into a deliberation about how to word a hierarchy was the
wrong move. Recorded as a correction, because the failure mode is general: an agent
asked to unify sources can spend a round debating the prose of the unification
instead of unifying them. **The content was still (a)** — six peers, bounds kept — but
it did not need a question.

## ❓ Q122 — What does Advanced Warfare own?

**(a)** legibility and status hierarchy **under load** — escalation, ping, warning
priority, the diegetic HUD; **(b)** industrial material language, which collides with
TouchWiz; **(c)** loadout and point-budget surfaces.

➡️ **(a)**, with (c) as a bounded second claim; (b) goes into AW's *bounds*, since
choosing materials is TouchWiz's to decide.

**✔ (verbatim):**

> ➡️ (a), with (c) as a bounded second claim, because a "what is unreadable under
> stress" owner is a job nothing else in the set does.

## ❓ Q123 — Does the picker inherit Popit's tool-surface semantics, and which ones?

**(a)** the **structural** half — a tool opened on a thing, one thing at a time,
actions in the surface that shows the subject — and not the game-facing half; **(b)**
§14 in full, cursor discipline included; **(c)** exempt because it is not
player-facing.

➡️ **(a)** — this is the rule that says the six candidate images *are* the surface,
the detail region is the bag, and the comment footer is the tool's one action, which
is the layout a113 had already chosen.

**✔ (verbatim):**

> ➡️ (a), and this is not a small point for the rebuild: it's the rule that says the
> six candidate images are the surface, the detail region is the bag, and the comment
> footer is the tool's one action — which is exactly the layout a113 already chose.
> (b) would put a game cursor on a desktop window.

---

## What the session changed

### In `UNIFIED_DESIGN.md` (693 → 752 lines, recorded as correction objects per §16)

| # | Change |
|---|---|
| **C1** | §1 is now **"the six sources of design conventions"**, at equal standing. The sentence calling Popit and Advanced Warfare "pattern libraries, not extra lineages that can outrank the four" is gone, and the evidence that it was wrong is recorded: **§17's own crosswalk already gave both columns equal to every other source** |
| **C2** | New **§3.2.1**, the colour-mechanism rule: a colour comes from a named mechanism with a declared ceiling, so exceeding one means asking for a different material |
| **C3** | New **§5.7**, tool surfaces: the subject *is* the surface; the detail region carries brief, notes and measured numbers; the tool's one action is a fixed footer that can never be below the fold or overlapped; **layout computes its height before it draws**; content beyond the viewport scrolls, at every scale |
| **C4** | New **§1.0**, separating **aspect ownership** (which part of a surface a source decides) from **role ownership** (which part of one artefact), which is what reconciles the lineages, the pillars and the icon's three roles |
| §1 | Six-row source table, with Popit and Advanced Warfare added and Popit's bounds written to include "a game cursor on a desktop surface" and "hearting as a review mechanism" |
| §1.0 | Aspect-ownership table, including AW's slot with its reasoning: iOS 6 owns clarity at rest, AW owns readability under stress, and loadout/point-budget is a bounded second claim |

### New: `ICON_STANDARD.md`

The specific half, so no number lives in two places: the three roles (silhouette,
surface, accent); the MD1 `baseline`/`48dp/2x` = 96 px reference and the 0.90
containment floor; the seven-family matrix with every mechanism and ceiling; the
seven anchors with what backs each, including the two marked honestly as **declared
slots**; the corpus ladder with per-parameter statistics, tolerances, digests and the
measurement-only licence rule; the six slots and their λ; a judged-versus-recorded
table that says *why* piece count is recorded and not judged; the rig; the artefact
list; and the open items.

### Corrections ledger for C5

| Was | Now | Because |
|---|---|---|
| "the ui is fucked up" — read as a styling problem | A **layout** defect: 712 px of row pitch against 494 px reserved, the comment box drawn over the second row's images, and no scrolling in 2081 lines | Measured before the first question |
| Agent recommended the picker follow MD1 + iOS 6 + Impeccable + Popit and leave TouchWiz out | All six apply | a114 |
| Agent recommended keeping the hand-rolled stack "if the HUD needs it" | Both surfaces share one layer, and the look is a hard constraint: WebGPU is right, the *look* is what failed | a115 |
| Agent's Q115 recommendation implied building primitives by hand | **Mechanism** crates — `cosmic-text` for measurement — with our own wgpu draw | a118 |
| Agent asked Q121 as a question about wording a hierarchy | The instruction was to unify six sets of conventions; the prose was not the deliverable | a121 |
| `UNIFIED_DESIGN.md` §1 demoted Popit and Advanced Warfare | Six sources at equal standing, with AW owning legibility under load | a117, a122 |
| The doctrine had no colour-mechanism rule, no tool-surface rule, and no icon section | §3.2.1, §5.7, and `ICON_STANDARD.md` | The audit above; a114 |

## Still open, and deliberately so

- **The picker rebuild itself.** Not started, deliberately: `src/bin/pick.rs` is 2081
  lines with a hand-rolled glyph cache, and a half-applied layout and text refactor
  would leave the binary unbuildable, which is worse than the current state — it at
  least runs. The measured defect list and the plan are in §1 of this record.
- **`tool-road`'s containment** (0.3235) and `tool-power`'s (0.857) against a 0.90
  floor — trace fault or comparison normalisation, not yet isolated.
- **The plate construction**, without which the material-led and accent-led slots
  cannot be conformant: fill cannot rise from 0.49 to 0.94 by decorating a thin glyph.
- **The layered construction's enclosed slivers** (20 holes where 0–1 is predicted).
- **The arrival bands**, still the pre-world values.
- **`Tool::Zone`**, the accent forms, the loader, and motion.

---

Continues in **`docs/GRILLING-C6.md`** — the implementation plan for all four builds,
and the atlas defect that turns out to be the reason the interface is unreadable.

**Correction recorded there:** this session diagnosed the picker as a *layout* defect
and led with it. The layout defect is real and measured, but the fault the user was
describing — an unreadable interface — is in `src/render.rs`'s shared coverage atlas,
which drives text, panels and outlines from one buffer.
