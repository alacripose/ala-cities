# C7 — widget layer playtest record

**What this stage was for:** the whole icon pipeline exercised by its only
final judge — a person deciding which candidates ship — and then the game
proving it received the decisions. The loop: picker → decision ledger →
shipping loader → toolbar draw, with sound on every hooked event.

---

## Session 1 — the picker, run by the reviewer

`cargo run --bin pick`, 2026-09-22 22:06 UTC. All **6 awaiting icons visited,
6 decisions recorded**, the tool closing itself after the last ("every
awaiting icon has been visited"). Every decision carried a `target: true` —
the promotion rule — and one carried a comment:

| icon | promoted | comment |
|---|---|---|
| tool-road | a-md1-plate | — |
| tool-power | a-md1-plate | "remove the extra floating geomaetry that has nothing to do twih the icon" |
| tool-inspect | b-md1-stack | — |
| tool-demolish | a-md1-plate | — |
| vocab-settings | b-md1-stack | — |
| ticket | a-md1-plate | — |

The MD1 family won five of six; the two layered-body wins are both
`b-md1-stack`. The comment is recorded verbatim, misspellings included —
it is a directive to the next authoring pass, and editing it would edit
the evidence.

**A defect was found handing the tool over, not by a test:** with the
comment field focused, digit and command keys still reached the picker —
typing "3 too flat" would have re-targeted candidate 3, and an `n` inside
a sentence would have recorded the review. Fixed before the session ran
(commit `2deb4df`); the guard has no test because the defect needs a
person typing prose.

## Session 2 — the game receives the decisions

`cargo run --bin ala-cities`, same evening, 1600×900, scale 1.0. The
startup log is the verification:

- `decided icons loaded count=6` — every promoted icon passed the loader's
  provenance checks; no text-toolbar fallback, no invented placeholder.
- `atlas_packed_to_row=4 atlas_refused=false` — the six 32 px renders
  packed into one strip and bound on the first frame.
- 26 interaction events over ~3 minutes: 16 `build_road`, 5 `zone`,
  4 `place_service`, 1 `demolish` — all four placement hooks, so the
  `Place` sound path fired on every build path that exists. Tool switches
  fired `Select`. **No refusals** (`refusals=0`), so `Refuse` has no
  on-record occurrence this session.
- Clean exit: autosave written, capture flushed
  (`session-1790114989450.jsonl`), 107 tickets on the ledger.

## What this record establishes — and what it does not

The machine record establishes: decisions were made by the picker's owner,
recorded in the ledger format, loaded with full provenance, drawn from the
atlas, and that every hooked event class occurred during play.

It does **not** establish, because only the reviewer can:

- whether the promoted icons are *legible at toolbar size* in the running
  game (the picker showed them at decision size; the toolbar draws them at
  the 32 px recognition size — that judgement is open);
- whether the sounds indicated something of use (a62) or were noise;
- whether the `Refuse` sound reads correctly — it has not fired yet.

## Open

- The comment on tool-power is a live directive: the next authoring pass
  regenerates that icon against it, and the re-decision supersedes.
- `vocab-settings` and `ticket` are promoted but have no in-game consumer
  yet — they ship in the atlas and wait for their surfaces.
- `Refuse` remains unexercised by an actual refusal.
