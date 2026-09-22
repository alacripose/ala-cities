//! The shared widget layer's first landing: **measurement and layout only**.
//!
//! C7 a128(a) scoped this layer to text measurement and layout, with every
//! pixel still drawn by the existing pipeline; widgets and theme resolution
//! arrive when a second kind of consumer needs them. What makes it a layer
//! rather than a helper is the rule it enforces — **layout computes its height
//! before anything draws** (`UNIFIED_DESIGN.md` §5.7), so a panel can never be
//! caught reserving one number and advancing another. C5's picker failed on
//! exactly that: 494 px reserved, 712 px advanced, and a comment box drawn
//! over the row below it.
//!
//! The layout is explicit about scale: every call takes the real `UiScale`.
//! Reading `UiScale::default()` inside a calculation whose other terms used
//! the real scale was C5's latent defect, and this layer is shaped so the
//! mistake cannot be made again by construction.
//!
//! Content beyond the viewport scrolls, at every scale (a119). The footer is
//! pinned: never below the fold, never overlapped, which is what §5.7
//! requires of a tool's one action.

// The picker rebuild is this module's first consumer and lands immediately
// after it; until that commit everything here is deliberately unused by the
// running binaries, and the dead-code lint says so rather than hiding it.
#![allow(dead_code)]

use crate::design::{Space, Step, UiScale};
use crate::render::{Batcher, Face, Screen, Text};

/// Baseline-to-baseline distance as a multiple of the step's pixel size.
/// Chosen once, here, rather than improvised per surface.
pub const LINE_ADVANCE_FACTOR: f32 = 1.35;

/// The horizontal inset of content from the window edge, per side.
pub fn content_inset(ui: UiScale) -> f32 {
    Space::Md.px(ui)
}

/// The width flowing content is measured and wrapped at.
pub fn content_width(screen: &Screen, ui: UiScale) -> f32 {
    (screen.w - 2.0 * content_inset(ui)).max(0.0)
}

/// One block of content, in flow order. A block declares what it is; the
/// measurer decides how tall it is from the font's own metrics.
#[derive(Clone, Debug)]
pub enum Block {
    /// A single paragraph of text at a named type step. No literal sizes
    /// here: that is what the type-scale gate is for.
    Line {
        face: Face,
        step: Step,
        text: String,
        color: [f32; 4],
    },
    /// A horizontal rule spanning the content width.
    Rule,
    /// Vertical space from the spacing scale — never a literal.
    Gap(Space),
    /// A region the caller draws itself (a grid of tiles, a canvas). The layer
    /// reserves its height; the pixels stay with the caller.
    Fixed { height: f32 },
    /// The tool's one action, pinned to the bottom of the viewport. Never
    /// below the fold, never overlapped, at any scale.
    Footer { height: f32, color: [f32; 4] },
}

impl Block {
    fn is_footer(&self) -> bool {
        matches!(self, Block::Footer { .. })
    }
}

/// The measured geometry of a list of blocks, before anything draws.
pub struct Measured {
    /// The blocks, in order.
    pub blocks: Vec<Block>,
    /// Each block's top edge in *content space*, where 0 is the top of the
    /// content and y grows downward. Footers are absent here: they do not
    /// flow, they pin.
    pub tops: Vec<f32>,
    /// Each block's measured height.
    pub heights: Vec<f32>,
    /// The total height of the flowing content.
    pub content_height: f32,
    /// The width content was measured at.
    pub width: f32,
    /// The viewport height this was measured against, in device pixels.
    pub viewport: f32,
    /// The footer's height, if any — carried separately because it pins.
    pub footer_height: f32,
}

impl Measured {
    /// The footer's top edge in *viewport space*: pinned to the bottom.
    pub fn footer_top(&self) -> f32 {
        (self.viewport - self.footer_height).max(0.0)
    }

    /// The height actually available to flowing content: the viewport minus
    /// the footer's reservation, because the footer is never overlapped.
    pub fn visible_height(&self) -> f32 {
        self.footer_top()
    }

    /// Whether the content extends past what the viewport can show.
    pub fn needs_scroll(&self) -> bool {
        self.content_height > self.visible_height() + f32::EPSILON
    }

    /// The furthest a scroll offset may reach. Zero when everything fits.
    pub fn max_scroll(&self) -> f32 {
        (self.content_height - self.visible_height()).max(0.0)
    }

    /// Where a block landed, for callers that draw inside a `Fixed` region:
    /// `(top_in_content_space, height)`.
    pub fn region(&self, index: usize) -> Option<(f32, f32)> {
        let block = self.blocks.get(index)?;
        if block.is_footer() {
            return None;
        }
        Some((self.tops[index], self.heights[index]))
    }
}

/// Measure a list of blocks against a content width and a viewport height,
/// using the font's own metrics for every height. Wraps each `Line` that
/// needs it — a wrapped line is several baselines, and the measurer counts
/// them *before* drawing, which is the whole point of the layer.
pub fn measure(
    text: &mut Text,
    ui: UiScale,
    width: f32,
    viewport: f32,
    blocks: &[Block],
) -> Measured {
    let mut tops = Vec::with_capacity(blocks.len());
    let mut heights = Vec::with_capacity(blocks.len());
    let mut y = 0.0f32;
    let mut footer_height = 0.0f32;

    for block in blocks {
        let height = match block {
            Block::Line { face, step, text: line, .. } => {
                let step_px = step.px(ui) as f32;
                let lines = wrap(text, *face, *step, line, width).len().max(1);
                lines as f32 * step_px * LINE_ADVANCE_FACTOR
            }
            Block::Rule => Space::Xs.px(ui),
            Block::Gap(space) => space.px(ui),
            Block::Fixed { height } => *height,
            Block::Footer { height, .. } => {
                footer_height = *height;
                // Reserved against the viewport, not flowed.
                0.0
            }
        };
        tops.push(y);
        heights.push(height);
        if !block.is_footer() {
            y += height;
        }
    }

    Measured {
        blocks: blocks.to_vec(),
        tops,
        heights,
        content_height: y,
        width,
        viewport,
        footer_height,
    }
}

/// Word-wrap a line to a pixel width using the font's real advance widths.
/// Returns the wrapped lines; a word wider than the box sits on its own line
/// rather than being clipped or hyphenated into a lie about its length.
pub fn wrap(
    text: &mut Text,
    face: Face,
    step: Step,
    s: &str,
    max_width: f32,
) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_width = 0.0f32;

    for word in s.split(' ') {
        let word_width = text.measure_step(face, word, step);
        let space_width = if current.is_empty() {
            0.0
        } else {
            text.measure_step(face, " ", step)
        };
        if !current.is_empty() && current_width + space_width + word_width > max_width {
            lines.push(std::mem::take(&mut current));
            current_width = 0.0;
        }
        if !current.is_empty() {
            current_width += space_width;
            current.push(' ');
        }
        // A single word wider than the box still occupies its own line whole.
        current.push_str(word);
        current_width += word_width;
    }
    lines.push(current);
    lines
}

/// A scroll region over measured content. Holds the offset, clamps it, and
/// answers the two questions drawing and input have: where does this content
/// pixel sit on screen, and which content pixel did this click land on.
pub struct Scroll {
    offset: f32,
}

impl Scroll {
    pub fn new() -> Self {
        Self { offset: 0.0 }
    }

    /// Scroll by a wheel delta (positive delta scrolls toward the top, the
    /// direction a wheel's roll away from the user reads as). Clamped, never
    /// overscrolled into emptiness.
    pub fn scroll_by(&mut self, delta: f32, measured: &Measured) {
        self.offset = (self.offset + delta).clamp(0.0, measured.max_scroll());
    }

    pub fn offset(&self) -> f32 {
        self.offset
    }

    /// The content-space y of a viewport-space y, or `None` when the point
    /// lands on the footer band — the footer is the tool's one action, not a
    /// part of the content, and it is never hit through the scroll.
    pub fn to_content_y(&self, measured: &Measured, viewport_y: f32) -> Option<f32> {
        if viewport_y >= measured.footer_top() {
            return None;
        }
        Some(self.offset + viewport_y)
    }

    /// The viewport-space y of a content-space y, when that line is visible.
    pub fn to_viewport_y(&self, measured: &Measured, content_y: f32) -> Option<f32> {
        let y = content_y - self.offset;
        if y < 0.0 || y >= measured.footer_top() {
            None
        } else {
            Some(y)
        }
    }
}

impl Default for Scroll {
    fn default() -> Self {
        Self::new()
    }
}

/// Paint flowing content and the pinned footer. `Fixed` regions are *not*
/// painted: callers read `Measured::region` and draw their own pixels,
/// because this landing owns layout, not widgets.
///
/// Every position is snapped to whole device pixels before it reaches the
/// batcher, keeping the atlas sampling honest at every scale. Lines are
/// anchored by the font's own ascent — read from a flat-cap `H` at the drawn
/// size, the same font-true trick `Text::em_px` uses for advances — so a line
/// whose top the measurer promised is the line whose top arrives.
pub fn paint(
    measured: &Measured,
    scroll: &Scroll,
    batcher: &mut Batcher,
    screen: &Screen,
    text: &mut Text,
) {
    let ui = text.ui_scale();
    let inset = content_inset(ui);

    // The footer first, in its own declared colour: content may then draw
    // over nothing, because the scroll mapping never hands out a line below
    // `footer_top`.
    if let Some(Block::Footer { color, .. }) =
        measured.blocks.iter().find(|block| block.is_footer())
    {
        batcher.screen_rect(
            screen,
            0.0,
            measured.footer_top(),
            screen.w,
            screen.h - measured.footer_top(),
            *color,
            Text::solid_uv(),
        );
    }

    for (index, block) in measured.blocks.iter().enumerate() {
        if block.is_footer() {
            continue;
        }
        let Some(top) = scroll.to_viewport_y(measured, measured.tops[index]) else {
            continue;
        };
        let height = measured.heights[index];
        if top + height <= 0.0 {
            continue;
        }
        match block {
            Block::Line { face, step, text: line, color } => {
                let step_px = step.px(ui) as f32;
                let advance = step_px * LINE_ADVANCE_FACTOR;
                // The font's own ascent at this size, via a flat-cap glyph.
                let cap_offset = text.ascent(*face, *step);
                for (i, part) in wrap(text, *face, *step, line, measured.width)
                    .into_iter()
                    .enumerate()
                {
                    let line_top = top + i as f32 * advance;
                    // `draw` hangs glyphs below the baseline it is given; the
                    // measurer promised the line's *top*, so the baseline sits
                    // above it by the flat-cap offset the font itself reports.
                    text.draw_step(
                        *face,
                        batcher,
                        screen,
                        snap(inset),
                        snap(line_top - cap_offset),
                        *step,
                        *color,
                        &part,
                    );
                }
            }
            Block::Rule => {
                batcher.screen_rect(
                    screen,
                    snap(inset),
                    snap(top),
                    measured.width,
                    1.0,
                    RULE_COLOR,
                    Text::solid_uv(),
                );
            }
            Block::Gap(_) | Block::Fixed { .. } => {}
            Block::Footer { .. } => unreachable!("footers are handled above"),
        }
    }
}

const RULE_COLOR: [f32; 4] = [0.35, 0.35, 0.38, 1.0];

fn snap(y: f32) -> f32 {
    y.round()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ui() -> UiScale {
        UiScale(1.0)
    }

    fn blocks() -> Vec<Block> {
        vec![
            Block::Line {
                face: Face::Body,
                step: Step::Title,
                text: "pick a candidate".into(),
                color: [1.0; 4],
            },
            Block::Gap(Space::Sm),
            Block::Fixed { height: 600.0 },
        ]
    }

    #[test]
    fn the_height_is_computed_before_anything_draws() {
        let mut text = Text::new();
        if text.missing_font {
            eprintln!("no system font available; the measurement is not exercised");
            return;
        }
        text.set_ui_scale(ui());
        let mut batch = Batcher::default();
        let measured = measure(&mut text, ui(), 600.0, 900.0, &blocks());
        // One title line + one small gap + a 600 px region, in that order.
        let title = Step::Title.px(ui()) as f32 * LINE_ADVANCE_FACTOR;
        let gap = Space::Sm.px(ui());
        assert!((measured.content_height - (title + gap + 600.0)).abs() < 0.5);
        // Measuring emitted no quads: layout precedes drawing, literally.
        assert_eq!(batch.instances.len(), 0, "measuring drew something");
        // Painting what was measured draws, and only what was measured.
        let scroll = Scroll::new();
        paint(&measured, &scroll, &mut batch, &screen(), &mut text);
        assert!(batch.instances.len() > 0, "painting drew nothing");
    }

    fn screen() -> Screen {
        Screen { w: 900.0, h: 700.0 }
    }

    #[test]
    fn a_wrapped_line_takes_the_height_of_all_its_baselines() {
        let mut text = Text::new();
        if text.missing_font {
            eprintln!("no system font available; the wrap check is not exercised");
            return;
        }
        text.set_ui_scale(ui());
        let long = "the six candidate images are the surface and everything else is text about candidates you are not choosing";
        let block = Block::Line {
            face: Face::Body,
            step: Step::Body,
            text: long.into(),
            color: [1.0; 4],
        };
        let measured = measure(&mut text, ui(), 300.0, 900.0, &[block]);
        let body = Step::Body.px(ui()) as f32 * LINE_ADVANCE_FACTOR;
        let lines = wrap(&mut text, Face::Body, Step::Body, long, 300.0).len() as f32;
        assert!(lines > 1.0, "the test needs a line that actually wraps");
        assert!((measured.content_height - lines * body).abs() < 0.5);
    }

    #[test]
    fn the_footer_is_never_overlapped_and_never_below_the_fold() {
        let mut text = Text::new();
        if text.missing_font {
            eprintln!("no system font available; the footer check is not exercised");
            return;
        }
        text.set_ui_scale(ui());
        let mut b = blocks();
        b.push(Block::Footer { height: 64.0, color: [0.2; 4] });
        let measured = measure(&mut text, ui(), 600.0, 400.0, &b);
        // Footer pinned to the bottom of the viewport, whatever the content does.
        assert!((measured.footer_top() - (400.0 - 64.0)).abs() < f32::EPSILON);
        // Scrolled to the end, the last flowing block stops at the footer's top.
        let mut scroll = Scroll::new();
        scroll.scroll_by(f32::INFINITY, &measured);
        let last = measured.tops.len() - 2; // the footer is last and does not flow
        let bottom = measured.tops[last] + measured.heights[last];
        let visible_bottom = scroll.offset() + measured.visible_height();
        assert!(
            bottom <= visible_bottom + 0.5,
            "the end of content may not sit under the footer"
        );
        // And a paint pass under that scroll never hands a line to the footer
        // band: every drawn text quad's top stays above the footer's top.
        let mut batch = Batcher::default();
        let screen = Screen { w: 700.0, h: 400.0 };
        paint(&measured, &scroll, &mut batch, &screen, &mut text);
        for instance in &batch.instances {
            // Screen-space y back out of clip space, top-left origin.
            let sy = (1.0 - instance.pos[1]) / 2.0 * screen.h;
            // Quads starting *at* the footer's top are the footer itself
            // (pinned, half-pixel of snap adjacency tolerated); content is
            // never handed a line below it.
            assert!(
                sy <= measured.footer_top() + 0.5,
                "a content quad drew at y={sy}, under the footer's top {}",
                measured.footer_top()
            );
        }
    }

    #[test]
    fn scrolling_clamps_and_hit_tests_through_the_same_numbers_it_drew_with() {
        let mut text = Text::new();
        if text.missing_font {
            eprintln!("no system font available; the scroll check is not exercised");
            return;
        }
        text.set_ui_scale(ui());
        let mut b = blocks();
        b.push(Block::Footer { height: 64.0, color: [0.2; 4] });
        let measured = measure(&mut text, ui(), 600.0, 400.0, &b);
        assert!(measured.needs_scroll());

        let mut scroll = Scroll::new();
        scroll.scroll_by(10_000.0, &measured);
        assert!((scroll.offset() - measured.max_scroll()).abs() < f32::EPSILON);
        scroll.scroll_by(-10_000.0, &measured);
        assert_eq!(scroll.offset(), 0.0);

        // A click 100 px down reads as content y 100 at offset 0 — and as
        // nothing at all when it lands on the footer band.
        assert_eq!(scroll.to_content_y(&measured, 100.0), Some(100.0));
        assert_eq!(scroll.to_content_y(&measured, measured.footer_top() + 1.0), None);
        // Content that has scrolled above the top is not on screen.
        scroll.scroll_by(200.0, &measured);
        assert_eq!(scroll.to_viewport_y(&measured, 0.0), None);
    }

    #[test]
    fn every_scale_lays_out_from_the_scale_it_was_given() {
        let mut text = Text::new();
        if text.missing_font {
            eprintln!("no system font available; the scale check is not exercised");
            return;
        }
        let ui = UiScale(2.0);
        text.set_ui_scale(ui);
        let measured = measure(&mut text, ui, 600.0, 900.0, &blocks());
        let title = Step::Title.px(ui) as f32 * LINE_ADVANCE_FACTOR;
        let gap = Space::Sm.px(ui);
        assert!((measured.content_height - (title + gap + 600.0)).abs() < 0.5);
    }
}
