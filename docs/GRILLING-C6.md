# Grilling — C6: the implementation plan, and the atlas

C6 exists because four builds had to be planned together — the full icon inventory,
the picker's new UI, the game, and the debugger — and checking the plan against the
old documents turned up two things the plan could not have assumed: the debugger is
**fully specified and completely unbuilt**, and the thing making the interface
unreadable is **not** the thing the previous session diagnosed.

The session also contains the clearest correction so far about *what to ask the
user*, and it is recorded in full because it has a general shape.

**Convention, as in `GRILLING.md`, `GRILLING-C2.md`, `GRILLING-C3.md`,
`GRILLING-C4.md` and `GRILLING-C5.md`:** `➡️` is the recommendation made. `✔` is
the answer received. Question numbers continue C5's sequence; C5 ended at Q123.

---

## What prompted it

> okay now grill me on the implementation plan for compiling all the new icons, the
> new ui for the picker, the game, and the debugger against the old documents

Fact-finding before the first question:

| Build | State | Waits on |
|---|---|---|
| The new icons | 6 pilots; `ICONS = PILOT_BASES`; `MISSING_INVENTORY` holds `ledger` and `tool-zone`; **`help` is in neither**, though `show_help` is a real surface and `action/help` a real MD1 mark | The plate construction — without it λ = 0.5 and λ = 1.0 cannot be conformant |
| The picker UI | 2081 lines, three constants that disagree, no scrolling (C5, round 25) | The widget layer |
| The game | `main.rs`, `render.rs`, `hud.rs`, `session.rs`, `sim/`, `gov/` | The widget layer; and the **icon loader**, which does not exist — every read of `assets/icons` is still `pick.rs` reading `review.json` |
| The debugger | **No code.** `src/bin/` contains only `pick.rs` and `verify.rs` | The widget layer, plus a ledger to read |

**The debugger is specified in `GRILLING-C2.md` (a71–a78, Q64) and has no
implementation.** The constraints those answers settled, which a plan has to carry:

- **a78 — a pane, not a window.** "A pane that lives horizontally next to the main
  window in windowed mode", reachable from the pause menu or a function key. Kept
  constraint: when the pane opens the world viewport narrows, so the picking sweeps
  must cover the narrowed aspect as well as the full one.
- **a71/a73 — agent presence is a claim about reality.** It reports what a **record**
  says, with **the age of that record**, and **degrades to Unknown** when the record
  goes quiet. The agent in the console is this agent, so the ledger must be kept
  truthful rather than dressed.
- **a74 — one store, `origin: sim | playtest`**; states `filed → acknowledged →
  in_progress → blocked / closed / retired / superseded`; **only the agent's ledger
  may set `in_progress`**.
- **a75 — playtests are the agent's and never overlap the user playing**, so *time
  left* shows a countdown only while a playtest runs and otherwise a **named
  absence, never a zero**.
- **a76/a77 — build identity** shows commit, branch, tag, dirty flag and the files
  changed since session start; a **dirty tree disqualifies release labelling**; git
  is **read-only** from inside the game; the pane is a **design-framework
  extension** — same tokens, same scales, the player's UI scale, the same element
  registry — with a **fault annotation path for reporting the debugger itself**.
- **a64 — deliberately off-centre** for players, with a contextual "report something
  wrong" action.
- **a69 — *"Simple code sounds like a death knell for this advanced project."***
  Where a capability is real, complexity is paid for. An MVP-shaped plan contradicts
  a standing answer.

And one consequence of a117/a121 that the plan had not been using: with Popit and
Advanced Warfare promoted to peers, **§5.7 governs the picker *and* the debugger** —
both are tool surfaces with a subject, a detail region and one action — which is a
reason to build one layer rather than two.

---

# Round 28 — order, scope, and boundaries

## ❓ Q124 — What is the build order?

**(a)** icons first, then the widget layer, then picker → game → debugger; **(b)**
widget layer first; **(c)** vertical slices of all four.

➡️ **(a)** — the icons are the only build with a *measurement* gate, they unblock
the game's loader, and the widget layer is the riskiest piece so it should not start
in parallel with anything that will churn under it.

**✔ (verbatim):** `a`

## ❓ Q125 — Do icon decisions wait for the new picker, or are they taken in the old one?

**(a)** plate lands first, then all decisions in the new picker; **(b)** decide the
pilots in the old picker so the loader has something to consume; **(c)** take
decisions now and import them into the rebuild.

➡️ **(a)** — (b) spends clicks on candidates the conformance pass already refused;
(c) imports decisions taken in a surface §5.7 says was not fit to take them.

**✔ (verbatim):** `A`

## ❓ Q126 — Scope of "all the new icons", and does the loader come with it?

**(a)** full inventory, Zone still deferred, **and** the loader built; **(b)**
inventory only; **(c)** re-validate the six pilots first.

➡️ **(a)** — the loader is what turns the icon work from a record into a product, and
the only way to learn whether the shipped sizes, the sidecars and the atlas are
actually usable.

**✔ (verbatim):** `A`

## ❓ Q127 — What is the debugger in code terms, and who may set `in_progress`?

**(a)** a pane inside the game binary reading the ledger read-only, with `in_progress`
set only by the agent's own ledger; **(b)** a third binary with its own window; **(c)**
a pane plus a separate agent-side tool that writes the ledger.

➡️ **(a) with (c) split by direction** — the pane reads, the agent's tool writes, and
the two never share a process, which is what makes "only the agent's ledger may set
`in_progress`" enforceable rather than a convention.

**✔ (verbatim):**

> ➡️ (a) with (c) split by direction: the pane reads, the agent's tool writes, and the
> two never share a process — which is what makes "only the agent's ledger may set
> in_progress" enforceable rather than a convention. It also keeps the
> release-labelling rule honest, since the dirty-tree check needs the game to see git
> but never touch it.

## ❓ Q128 — What does the shared widget layer own, and does the HUD adopt it wholesale?

**(a)** text measurement and layout only, every pixel still drawn by our pipeline, HUD
adopting panel by panel; **(b)** layout *and* widgets, both surfaces migrating; **(c)**
layout, widgets and theme resolution.

➡️ **(a) for the first landing, (b) as the target** — the defect that broke the picker
was a measurement failure, so (a) fixes the real fault with the smallest surface area,
while (c) before two surfaces exist is a design system built for one consumer.

**✔ (verbatim):**

> ➡️ (a) for the first landing, (b) as the target. The defect that broke the picker
> was purely a measurement failure, so (a) fixes the real fault with the smallest
> surface area; (b) is where this ends up once two surfaces need the same checkbox
> and the same focus ring, and (c) is where token drift gets impossible — but (c)
> before two surfaces exist is a design system built for one consumer.

---

# Round 29 — the plate, the check, and the atlas

## ❓ Q129 — What *is* the plate?

**(a)** tile-filling body with the mark **cut into** it; **(b)** tile-filling plate
with the mark carried **on** its face; **(c)** the mark extruded with a bezel.

➡️ **(b)** — the roles stay simultaneous; **(a)** makes the mark an absence, and an
absence vanishes first at 24 px; **(c)** cannot reach 0.937 with any stroke thickness
a glyph can have.

**✔ (verbatim):**

> You came up with the plate in the first place. I have no idea

**Override — and a correction about *what to ask*.** The user is right: the plate was
the agent's invention in C4, so asking them to choose its geometry asked them to
decide something only the person who proposed it could specify. The question was
**wrong to ask**, not merely hard. Decision taken back by the agent, to be shown as
renders rather than argued in prose.

## ❓ Q130 — What happens to the silhouette check when the mark sits on a plate?

At λ = 1.0 a full-tile plate covers every cell the declared mark covers, so
containment goes to 1.0 **trivially** while occupancy agreement collapses: the check
as written would report the material-led and accent-led candidates as perfectly
conformant silhouettes.

➡️ **(a)** a **λ-dependent reading**: containment where the mark is the object,
occupancy agreement of the mark's region where it is not.

**✔ (verbatim):**

> ????

**The same failure, worse.** Not merely a question the user declined to answer — a
question they could not parse. The mechanism is a fact about the check, discoverable
by reading it, and it never needed to be put to anyone. Decision taken back by the
agent: the check reads against λ, and the reasoning is shown as a measured table.

**The general shape, recorded because it will recur:** an agent that has *invented* a
mechanism tends to hand the user the specification of it as a question. A question is
only legitimate when the answer changes what is built **and** the user is better
placed to know it than the agent is. Neither held for Q129 or Q130.

## ❓ Q131 — How does the game address an icon, and does `help` join the inventory?

**(a)** one RGBA atlas per shipped size, packed like the text atlas, addressed by
manifest `id` through a generated lookup, so a draw site asks for `Tool::Road` and
never a path; **(b)** sidecars by path; **(c)** atlas for small sizes, files at 96.

➡️ **(a)**, with **`help` added** in the same pass — it has a real surface and a real
MD1 mark, which is exactly the test the inventory rule uses.

**✔ (verbatim):**

> your best guess. the text atlassing is causing the ui to be unreadable since it's
> not just driving text properly

**Delegated, and it carried a bug report that reframed the diagnosis.** Two separate
readings came out of it:

1. **Delegation:** (a) stands, with `help` added.
2. **The readability fault is the atlas, not the layout.** C5 diagnosed the picker as
   a *layout* defect — which is real and measured (712 px of row pitch against 494
   reserved, no scrolling, the comment box drawn over row 2) — but the user's report
   names a different cause of *unreadability*: `src/render.rs`'s coverage atlas drives
   **text, panels and outlines** from one buffer, so a defect there corrupts every
   filled rectangle in the game *and* the picker, not just the words.

### The atlas, from the code

```rust
// src/render.rs — the reserved solid region, for panels and outlines:
for y in 0..2 { for x in 0..2 { data[y * ATLAS_SIZE + x] = 255; } }   // 2×2 texels at (0,0)

// glyph packing starts here:
pen_x: 4, pen_y: 0                                                    // same row, two texels away
...
uv: [ at_x/ATLAS_SIZE, at_y/ATLAS_SIZE,
      (at_x + w)/ATLAS_SIZE, (at_y + h)/ATLAS_SIZE ]                   // far edge on a texel boundary
```

**Two mechanisms follow, and both match the reported symptom:**

1. **The UV rect is half-open in the wrong direction for a Nearest sampler.** It spans
   exactly `w` texels, so a quad drawn across it puts its far column and far row *on*
   the boundary at `at_x + w` — one texel past the glyph, in the 1-pixel gutter, which
   is **zero**. The last column and bottom row of every glyph sample transparent
   instead of the glyph, so strokes lose their edges. Whenever the drawn size differs
   from the rasterised size — every UI scale but 100 % — this happens on every glyph.
   The sampler is Nearest and clamped (`render.rs`), so there is no filtering to hide it.
2. **Solid fills sample the 2×2 patch by the same convention, and glyph packing begins
   in the same row.** The region meaning "solid coverage" sits two texels of background
   from glyph data, with nothing in the shader or the packer enforcing the boundary. A
   fill whose UV lands a hair off gets a zero texel — an edge that disappears — or glyph
   coverage — a panel speckled with letter fragments.

That is why it reads as a font bug and is not one: the atlas is not driving text
properly, and it is not only driving text.

### What the atlas changed about the plan

The three fixes are **one job**, and they move to the head of the widget layer rather
than into the picker rebuild: the solid region becomes a real reserved block with an
enforced boundary and a constant fill UV; the glyph UV rect stops on the last texel
instead of the boundary; and the layout pass is then built on text that measures and
renders correctly. Building a layout on top of text that renders wrong would have
produced a well-arranged, unreadable tool.

---

## Corrections ledger for C6

| Was | Now | Because |
|---|---|---|
| C5: "this ui is fucked up" diagnosed as a layout defect, and led with that | The **readability** fault is the shared coverage atlas in `render.rs`; the layout defect is real, measured and *separate* | a131. Both are true; the agent led with the wrong one |
| Agent asked the user to choose the plate's geometry | The plate was the agent's invention; it specifies it and **shows renders** | a129 — the question was wrong to ask |
| Agent asked the user to choose how the silhouette check should read a plate | A fact about the check, discoverable by reading it; no question was needed | a130 — the question could not even be parsed |
| Agent assumed the picker's rebuild was the first UI work | The atlas is upstream of it, and of the game and the debugger too | All three draw text through `render.rs` |
| The debugger assumed to need its own surface | a78 already settled a pane inside the window; a74/a127 split read and write across processes | Reading C2 for the plan |
| `help` absent from both `ICONS` and `MISSING_INVENTORY` | Added to the inventory | It passes the inventory rule: a real surface and a real MD1 mark |

## What each answer settles

1. **Order:** icons → widget layer (atlas first) → picker → game → debugger.
2. **Icon decisions are taken in the new picker only.** Targets recorded in the old
   one are void by design.
3. **Scope:** the full inventory with Zone deferred, `help` added, **and** the icon
   loader, addressed by manifest `id` from a generated atlas lookup.
4. **The debugger is a pane in the game binary that reads; the agent's tool writes.**
   Separate processes, so `in_progress` has exactly one author.
5. **The widget layer owns text measurement and layout first**, widgets second, theme
   resolution third — each when a second consumer exists.
6. **The plate is a tile-filling body with the mark on its face**, and the silhouette
   check reads against λ.
7. **The atlas is fixed before any layout work**, because it is the actual cause of
   the interface being unreadable.

## Still open, and deliberately so

- **The picker rebuild**, still not started — now correctly ordered behind the atlas.
- **The icon loader and atlas packing**, with the text atlas's own defect as the
  worked example of what not to repeat.
- **`tool-road`'s containment** (0.3235) and `tool-power`'s (0.857) against a 0.90
  floor.
- **The layered construction's enclosed slivers** (20 holes where 0–1 is predicted).
- **The arrival bands**, still the pre-world values.
- **`Tool::Zone`**, the accent forms, motion, and the agent-side ledger writer that
  the debugger's read side will depend on.
