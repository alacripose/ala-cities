# C2 — the text defect, located and fixed

The player reported, in their own words, that *"the picker's text is similarly bugged
as hell as the main game"*. They were right, and the cause was not in the text at all.

This record exists because the previous presentation pass **verified the wrong end of
the pipeline** and said so in a way that sounded like the whole path was covered.

---

## What the report was

The player played the presentation build (commit `ee09d7a`, then `a333c7f`) and
described the interface text as **extremely difficult to read, inconsistently sized,
and bugged**. The presentation pass had measured:

* the type scale — 5 steps, 12/14/16/20/26 px, body 16 px, enforced by `draw_step`
  taking a `Step` so a literal cannot reach a text call site;
* rasterisation — glyphs snapped to whole pixels, atlas sampled `Nearest`;
* contrast — TextBody on Panel 14.24:1, on Desk 17.02:1.

All of that was true. None of it was the complaint.

## The measurement

The picker is where it became visible, because the picker draws a *lot* of text, and
it re-drew the atlas through its own path. Reading that path:

```
if text.data.len() == self.uploaded_atlas_len && self.uploaded_atlas_len != 0 { return; }
```

`Text::data` is a **fixed 1024 x 1024 R8 buffer** (1,048,576 bytes). Its length never
changes, so after the first frame this comparison was always true and the texture was
uploaded **exactly once** — at startup, when the atlas held a handful of glyphs.

The game had the same defect by a different route:

```
atlas_dirty: true            // set in Gpu::new
atlas_dirty = true           // set when the UI scale changes
if self.atlas_dirty { gpu.upload_atlas(&self.text); }
```

No glyph rasterisation ever set the flag. Every glyph first drawn **after** the first
frame — a toast, a longer label, a digit as the sim counted past a value the first
frame never showed, the whole picker's vocabulary — was rasterised on the CPU and
never sent to the GPU. The sampler then read the atlas at coordinates the texture did
not yet have content for: **empty texels, or whatever glyph already owned that
region.** Mixed with the text that *did* arrive, that reads exactly as the player
described it: text that is somehow both hard to read and inconsistently sized, in
both binaries, for the same reason.

## The fix, and the rule it leaves behind

The upload is now driven by the atlas's own content. `Text::revision` counts every
glyph rasterised and every UI-scale change; `Gpu::sync_atlas` uploads when the GPU's
copy is behind and skips the transfer when it is not.

```
pub revision: u64,        // bumped in slot() on a cache miss, and in set_ui_scale
pub fn sync_atlas(&mut self, text: &Text) -> bool { if text.revision == self.atlas_revision { return false; } ... }
```

Two consequences, both deliberate:

* **the flag is deleted, not corrected** — `atlas_dirty` is gone from `main.rs`. A
  hand-set flag was a second source of truth for something the atlas already knew,
  and this is the second time in this project that a hand-maintained flag has been
  the bug rather than the guard;
* **the picker's length comparison is gone too** — it could never have been right,
  because the thing it compared does not vary.

`the_atlas_revision_follows_the_atlas_and_nothing_else` pins the property:
a new glyph moves it, a cached glyph does not, measuring moves it (measuring
rasterises), and a UI-scale change moves it exactly once.

## What is on the record from the running build

| Fact | Value |
|---|---|
| Game first frame | `world_opaque=26167 world_overlay=50 interface=410 glyph_slots=103 atlas_packed_to_row=0 atlas_refused=false` |
| Picker first frame | `575 interface quads · 6 icon quads · atlas 1048576 bytes · 96 glyph slots packed to row 0/1024` |
| Atlas capacity risk | not present in this build: 103 slots, packed to row 0 of 1024 |
| Tests | 115 passing, 1 ignored |
| Clippy | 0 warnings across all targets |

The atlas-occupancy figure is reported on the first frame for a specific reason: a
**full** atlas refuses to draw new glyphs, and the symptom of that is *missing* text,
which reads as a font bug and is not one. Now the number is on the record rather than
being a thing to guess at later.

## What this pass did *not* establish

* **Whether the text is now readable to the player.** That is the judgement this
  record cannot make, and it is the one the player has to make.
* **Frame cost of the upload.** It now happens on frames where the atlas changed,
  which is most frames during a burst of new text. The transfer is 1 MB. Not measured
  under load.
* **Whether other surfaces beyond text were drawing from stale GPU state.** The
  world and the interface batches are rebuilt per frame from CPU-side data, so this
  class of staleness was specific to the atlas — but that is an argument, not a
  measurement, and it is stated as one.

## Playing it

```
cargo run --release                                  # the game
cargo run --release --bin pick                       # the icon review picker
```

In the picker: click a generation's checkbox (or press 1/2/3), type in the comment
box for the next generation, press Enter to record. Every decision is appended to
`assets/icons/review-decisions.jsonl`.
