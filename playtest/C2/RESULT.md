# C2 — why the text was unreadable

The player reported, twice, that the text was *"extremely difficult to read and
inconsistently sized"*, and then that the picker's text was *"similarly bugged as
hell as the main game"*, and supplied two screenshots.

**The cause was one missing line of arithmetic, and it had been there from the
beginning.** `ab_glyph`'s `PxScale` is not the em size. The crate says so in its own
definition — *"By definition, this is the pixel-height of a font"* — and its
`pt_to_px_scale` states the conversion:

```rust
let px_per_em = pt_size * (96.0 / 72.0);
let height = self.height_unscaled();
PxScale::from(px_per_em * height / units_per_em)
```

The code passed the design size straight in: `PxScale::from(size as f32)`. Segoe UI's
height is 1.33 em, so **every step on the type scale rendered at 75 % of the size it
declared, and advanced the pen at 75 % too**:

| step | declared | rendered | what it is |
|---|---|---|---|
| micro | 12 px | **9.02 px** | below the 12 px floor the design document sets |
| small | 14 px | **10.53 px** | — |
| body | 16 px | **12.03 px** | the size the previous build was *reading* as body |
| title | 20 px | **15.04 px** | — |
| display | 26 px | **19.55 px** | — |

So the interface drew 12 px text where the design said 16, and 9 px text where it
said 12 — beneath the legibility floor, at a size nothing had ever been measured at.
The type scale was correct in the source, correct in the gate, correct in every
contrast measurement, and **25 % smaller on the screen than any of those checks
assumed**. That is the whole of *"difficult to read and inconsistently sized"*: not
inconsistency between labels, but between what the design said and what was drawn.

## How it was found, in order, including the dead end

1. **The atlas upload — a real bug, not this one.** The picker gated the upload on
   `text.data.len()`, which for a fixed 1024x1024 atlas never changes, so the texture
   was uploaded exactly once and every glyph rasterised later sampled empty texels;
   the game gated the same upload on a hand-set `atlas_dirty` flag that no glyph ever
   set. Fixed, and the flag deleted rather than corrected — `Text::revision` counts
   the atlas's own content now.
2. **The screenshots, measured.** The player's capture was read as luminance, banded
   into text rows, and rendered as ASCII so the glyphs could be read. Comparing the
   drawn label against the same font rendered independently (PIL, `segoeui.ttf`, same
   nominal size) showed the *positions* correct and the *letterforms* wrong — letters
   touching, gaps filled, strokes doubled.
3. **The composite test.** The decisive step, and it is now a permanent test: draw a
   phrase into a `Batcher`, then composite the quads on the CPU by sampling the atlas
   through the same UVs the shader would use. That composite is what the GPU is
   *asked* to paint. It read `Hamburgefons gyp 12.5:1` — readable, correctly
   baselined — and **its width was 0.75x the width the font's own metrics demand**.
4. **The ratio, named.** 0.75 at every size, exactly. Segoe UI: ascent + descent =
   2724 units over a 2048-unit em = 1.33, and 1 / 1.33 = 0.75. One cast, one
   concept: `PxScale` is a *height*, not an em.

## The fix

`em_scale` — the inverse of the crate's own conversion, with the em given in pixels:

```rust
PxScale::from(px as f32 * height_unscaled / units_per_em)
```

After it, the drawn widths agree with the font's own advances at every step:

| step | before | after | font's own metrics |
|---|---|---|---|
| 12 px | 102.7 | **136.6** | 135.0 |
| 14 px | 119.8 | **159.3** | 160.0 |
| 16 px | 136.9 | **182.1** | 180.0 |
| 20 px | 171.1 | **227.6** | 226.0 |
| 26 px | 222.5 | **295.9** | 295.0 |

## The gate, and proof that it closes

Two checks, both at startup and both failing closed:

* `Text::scale_defects()` — each step's *effective* em, read back out of the rendered
  advance rather than recomputed from the conversion, must equal its declared size;
* the independent width check in the composite test — a phrase's drawn width must
  agree with the sum of the font's *unscaled* advances at the declared em size, to
  within half a pixel. A scale bug that fooled both the rasteriser and the
  measurement would pass the first and fail this one.

Proof, run deliberately: with `em_scale` reverted to the broken conversion, the gate
refused with

```
type step `micro` declares 12 px and renders at 9.02 px, which is 75 % of the size
every measurement assumes; … `body` declares 16 px and renders at 12.03 px, …
```

and the test failed with *"the type scale does not render at the sizes it declares"*.
Restored, it passes. The numbers in that refusal are the same numbers in the table at
the top of this record: the defect, named by the check that exists to catch it.

## What this pass did not establish

* **Whether it is readable to the player now.** That is the judgement this record
  cannot make, and it is the one that matters.
* **Whether the 240 Hz frame budget is met with the extra quads.** Text is now 33 %
  taller and wider on screen; the first-frame report is `394` interface quads in the
  game, `579` in the picker, and no frame timing under load was taken.
* **Whether anything else was sized against the wrong number.** Every layout constant
  that was *measured* from text — panel heights, line spacing, the bottom-anchor
  offsets — was measured through the same wrong conversion, so the layout is
  internally consistent but was tuned while everything was 25 % small. Worth a look
  at the chrome now that text is the size it claims.

## Playing it

```
cargo run --release                                  # the game
cargo run --release --bin pick                       # the icon review picker
```
