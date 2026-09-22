# Grilling — C7: the implementation plan, resumed

C7 exists because C6 left an implementation plan and the tree had moved under
it. The session's first act was measurement, not questions: several premises in
C6's "still open" list were stale, one defect was already half-fixed, and the
uncommitted tree held a whole session's product.

**Convention, as throughout the record:** `➡️` is the recommendation made. `✔`
is the answer received. Question numbers continue C6's sequence; C6 ended at
Q131.

---

## What prompted it

> grill me through the grilling docs, starting at c6 and ending at grilling.
> Then begin the implementation

**Facts established before the first question**, because the questions had to
be asked against the tree that exists, not the one C6 described:

| C6 said | The tree says |
|---|---|
| The atlas defect must be fixed before any layout work | **Half-fixed.** Solid fills got a point-sampled `solid_uv` and a packing gutter — but the glyph UV rect still spans exactly `w` texels, landing its far edge *on* the zero-coverage gutter. C6's mechanism 1 is unfixed. |
| The plate + λ-dependent check are owed | The C3 `INVENTORY` tuple carries the set; `help` was recorded but never registered; no plate construction, no λ-dependent check. `review-decisions.jsonl` has never existed — zero promotions ever taken. |
| The picker is 2081 lines with no scrolling | Unchanged since C5 — zero `scroll`/`wheel` occurrences. |
| The loader is owed with the icons | Still does not exist; only `pick.rs` reads `assets/icons`. |
| The debugger needs a ledger to read | **No gov ledger exists.** But the season store already carries `BLD-*` and `CSE-*` tickets — a74's "one store" is an *extension* of the season record, not a new invention. Only the agent-side ledger (the sole `in_progress` author) is genuinely new. |
| a24 adopted `rodio` | Not in `Cargo.toml`; no audio code. |
| `Tool::Zone` deferred: "no zone surface in the game yet" | **Stale premise** — `main.rs` has real zone placement (`Tool::Zone`, `act_zone`, three `Zone` types). The other half stands: MD1 has no zone glyph, which per C3's gap rule sends the silhouette to iOS 6. |

And the tree itself: **618 untracked paths, 194 deletions, 14 modifications** —
the re-rendered review sets under the new canonical scheme, `ICON_STANDARD.md`,
the manifest/`review.json` rework, and the icon tooling — all uncommitted since
before C6.

---

# Round 1 — the tree, the deferral, the audio, the rebuild

## ❓ Q132 — What happens to the uncommitted tree?

**(a)** commit the existing work first, partitioned; **(b)** leave it and stack
new work on it; **(c)** discard and re-derive.

➡️ **(a)** — the review renders are provenance-tracked artifacts the conformance
pass must read; (c) would delete work the record counts as done; (b) makes
nothing attributable.

**✔ (verbatim):** `a`

## ❓ Q133 — Does the Zone icon deferral stand?

➡️ **(b)** — the surface exists in code but the tool's identity (which zone type
is being painted) is still a HUD mode, not a surface with a locator; a
half-defined locator is how invented surfaces crept in before. Sequenced after
`help`.

**✔ (verbatim):** `B`

## ❓ Q134 — Is audio in this campaign, and where?

a24 adopted `rodio` "now"; C6's order left it off. **(a)** audio ships in this
campaign, inside the game-integration step; **(b)** sequenced out entirely.

**✔ (verbatim):** `a`

## ❓ Q135 — Picker rebuild: fresh file or in place?

**(a)** rewrite from a clean file against the widget layer, porting only the
decision recording, old file retired; **(b)** refactor in place; **(c)** rebuild
fresh but keep the old binary runnable until the new one passes a smoke test.

➡️ **(c)** — (b) is how three disagreeing constants survive a refactor; (a)
trades a runnable tool for a clean file during the stretch where the tool is
needed for every icon decision.

**✔ (verbatim):** `c`

---

# Round 2 — the packs, the audio scope, the second consumer

## ❓ Q136 — Do the reference packs go into the public repo?

The repo is public per a65. MD1 is Apache-2.0; the TouchWiz pack is Samsung
artwork and iOS 6 is community/third-party brand work — reference-grade only,
per C3 round 17. The checks read MD1 *content*; the others ride on digests.

**(a)** ignore `assets/reference/` entirely; **(b)** vendor MD1 only; **(c)**
vendor all three.

➡️ **(b)**.

**✔ (verbatim):** `a` — **overrode.** The packs stay local, recorded by digest.
The manifest's digest mechanism is what carries the provenance; nothing in the
repository claims redistribution.

## ❓ Q137 — What does the first audio pass cover?

**(a)** UI feedback only (tool select, place, refuse, notification); **(b)** plus
store-driven ambient bed; **(c)** plus generative music.

➡️ **(b)** — a sound, like an animation, must indicate something of use (a62).

**✔ (verbatim):** `a` — **overrode.** UI feedback only, this pass.

## ❓ Q138 — Does the game HUD migrate onto the widget layer this campaign?

**(a)** fully, right after the picker ships, before game integration; **(b)**
shared primitives only; **(c)** not this campaign; **(d)** fully, but during
game integration.

➡️ **(a)** — the icon loader is the forcing function; icons through the old path
while text goes through the new one is exactly the two-surface drift a128
named.

**✔ (verbatim):** `a`

---

## Corrections ledger for C7

| Was | Now | Because |
|---|---|---|
| C6: "no zone surface in the game yet" | Zone placement is real and working; the deferral stands on the *locator* half, not the surface half | Measured in `main.rs` |
| C6: the atlas defect is wholly open | Half-fixed already: point-sampled solid UV and packing gutter in; the glyph UV far edge still lands on the gutter | Measured in `render.rs` |
| Agent recommended vendoring MD1 | **No packs are vendored**; digests carry provenance | a136 |
| Agent recommended store-driven ambient audio | **UI feedback only** this pass | a137 |
| C6: the debugger needs a ledger to read | The season store already carries `BLD-*`/`CSE-*` tickets — the store exists; the agent-side ledger is the new piece | Measured in `saves/season_2026_s1/` |

## What each answer settles

1. **The tree was committed first**, partitioned: tooling → assets → runtime
   records → docs, with the reference-pack ignore landing first so the packs
   could never be swept in.
2. **Zone stays deferred** — recorded with its *current* reason: the tool exists,
   the locator does not yet.
3. **Audio ships in this campaign**, in the game-integration step, scope: UI
   feedback sounds.
4. **The picker is rebuilt fresh** (`pick.rs` → new file on the widget layer),
   old binary kept runnable until the new one passes a smoke test.

## Still open, and deliberately so

- The atlas glyph-UV fix (this campaign's first code task).
- The widget layer, picker rebuild, HUD migration, loader, game integration
  with audio, and the debugger pane + agent ledger — in that order (C6 Q124,
  unchanged).
- `tool-road` containment (0.3235), `tool-power` (0.857); the plate
  construction; the layered-construction slivers; the arrival bands; the accent
  forms; `help`'s registration into the inventory.
