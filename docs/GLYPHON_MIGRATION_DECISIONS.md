# Glyphon text-rendering migration

Approved 2026-09-22:

- Migrate all text paths: the game client, shared HUD/UI, and icon picker.
- Use Glyphon as the sole text renderer; remove `ab_glyph` and the hand-rolled atlas/rasterizer.
- Use the native Glyphon/cosmic-text model rather than preserving the old text API as the target architecture.
- Use OS font discovery with cosmic-text fallback; do not bundle font files.
- Keep text screen-space only; no world-space text path is required by current callers.
- Upgrade `wgpu` from 29 to 30 and use Glyphon 0.12.
- Rewrite tests around behavior: shaping, multilingual text, wrapping/measurement, clipping, placement, and dynamic atlas preparation.
- Preserve physical design sizing: `Step::px(ui)` is the physical font size, with line height derived from `LINE_ADVANCE_FACTOR`; `TextArea::scale` remains 1.0.

The migration must keep the existing measure-before-paint, scrolling, and pinned-footer guarantees while replacing the rendering implementation.
