//! The shared widget layer: **measured layout, painted through the one
//! pipeline**.
//!
//! What makes it a layer rather than a helper is the rule it enforces —
//! **layout computes its height before anything draws** (`UNIFIED_DESIGN.md`
//! §5.7), so a panel can never be caught reserving one number and advancing
//! another. C5's picker failed on exactly that: 494 px reserved, 712 px
//! advanced, and a comment box drawn over the row below it.
//!
//! The layout is explicit about scale and about place: every call takes the
//! real `UiScale`, and every call takes a [`Frame`] — the content box in
//! screen space — instead of assuming content starts at some layer-chosen
//! inset. Reading `UiScale::default()` inside a calculation whose other terms
//! used the real scale was C5's latent defect; assuming a single canonical
//! content origin would have been the same mistake in space. This layer is
//! shaped so neither can be made again by construction.
//!
//! Content beyond the viewport scrolls, at every scale (a119). The footer is
//! pinned: never below the fold, never overlapped, which is what §5.7
//! requires of a tool's one action. Paint draws only blocks that are *fully*
//! visible: this renderer has no scissor, so a discrete row (a ledger line, a
//! menu item) either fits or waits for the scroll that reveals it — cut
//! glyphs over whatever sits below the region are not an option.
//!
//! Colours arrive as values, never as tokens: the theme lives in
//! [`crate::hud`], and this layer stays theme-agnostic so a surface can use
//! it before the token table knows its name.

// The picker and the HUD are this module's consumers; game integration comes
// next and may still want a primitive a consumer has not yet exercised, so
// the dead-code lint stays explicitly acknowledged rather than silently
// tripping later.
#![allow(dead_code)]

use crate::design::{Space, Step, UiScale};
use glyphon::{TextBounds, Wrap};

use crate::render::{Batcher, Face, Screen, Text};
use crate::text::TextLayout;

/// Baseline-to-baseline distance as a multiple of the step's pixel size.
/// Chosen once, here, rather than improvised per surface.
pub const LINE_ADVANCE_FACTOR: f32 = 1.35;

/// Where a two-sided row's bar (if it has one) begins, as a fraction of the
/// content width. One number, here, instead of a literal per panel.
const BAR_START: f32 = 0.38;

/// A content box in screen space: where content sits and how much room it
/// has. `x`/`w` are the *content* edges — already inset; paint never insets
/// again. `viewport` is the height visible for flowing content (scrolling
/// clamps against it); `y` is where viewport-space 0 lands on screen.
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub viewport: f32,
}

impl Frame {
    pub fn new(x: f32, y: f32, w: f32, viewport: f32) -> Self {
        Self {
            x,
            y,
            w,
            viewport,
        }
    }
}

/// One block of content, in flow order. A block declares what it is; the
/// measurer decides how tall it is from the font's own metrics.
#[derive(Clone, Debug)]
pub enum Block {
    /// A single paragraph of text at a named type step. No literal sizes
    /// here: that is what the type-scale gate is for. Wraps when too wide.
    Line {
        face: Face,
        step: Step,
        text: String,
        color: [f32; 4],
    },
    /// A two-sided row: a body-face label on the left, a monospace value on
    /// the right edge. The workhorse of a state panel. One line tall — the
    /// caller truncates label and value when building the block, because
    /// ellipsis is a content decision, not a layout one.
    Row {
        label: String,
        value: String,
        step: Step,
        color: [f32; 4],
        value_color: [f32; 4],
    },
    /// A labelled fraction bar: label on the left, a track running to the
    /// right edge, filled to `fraction`. Demand gauges and progress.
    Bar {
        label: String,
        step: Step,
        label_color: [f32; 4],
        color: [f32; 4],
        fraction: f32,
    },
    /// A keyed row: a monospace key at the left edge, prose at a declared
    /// column. The help surface's shape. `key_col` is the caller's column
    /// width in device pixels — declared once at the call site, like any
    /// other column the surface owns.
    Keyed {
        key: String,
        text: String,
        step: Step,
        key_color: [f32; 4],
        text_color: [f32; 4],
        key_col: f32,
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
    /// The frame this was measured in: paint lands content exactly here.
    pub frame: Frame,
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
    /// Glyphon layouts created during measurement, one vector per block.
    pub layouts: Vec<Vec<TextLayout>>,
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

/// Measure a list of blocks in a frame, using the font's own metrics for
/// every height. Wraps each `Line` that needs it — a wrapped line is several
/// baselines, and the measurer counts them *before* drawing, which is the
/// whole point of the layer.
pub fn measure(text: &mut Text, ui: UiScale, frame: &Frame, blocks: &[Block]) -> Measured {
    let mut tops = Vec::with_capacity(blocks.len());
    let mut heights = Vec::with_capacity(blocks.len());
    let mut layouts = Vec::with_capacity(blocks.len());
    let mut y = 0.0f32;
    let mut footer_height = 0.0f32;

    for block in blocks {
        let (height, block_layouts) = match block {
            Block::Line { face, step, text: line, color } => {
                let layout = text.layout(*face, *step, line, *color, frame.w, Wrap::WordOrGlyph);
                (layout.height, vec![layout])
            }
            Block::Row { label, value, step, color, value_color } => {
                let label_layout = text.layout(Face::Body, *step, label, *color, frame.w, Wrap::None);
                let value_layout = text.layout(Face::Mono, *step, value, *value_color, frame.w, Wrap::None);
                (label_layout.height.max(value_layout.height), vec![label_layout, value_layout])
            }
            Block::Bar { label, step, label_color, .. } => {
                let layout = text.layout(Face::Body, *step, label, *label_color, frame.w, Wrap::None);
                (layout.height, vec![layout])
            }
            Block::Keyed { key, text: prose, step, key_color, text_color, .. } => {
                let key_layout = text.layout(Face::Mono, *step, key, *key_color, frame.w, Wrap::None);
                let prose_layout = text.layout(Face::Body, *step, prose, *text_color, frame.w, Wrap::None);
                (key_layout.height.max(prose_layout.height), vec![key_layout, prose_layout])
            }
            Block::Rule => (Space::Xs.px(ui), Vec::new()),
            Block::Gap(space) => (space.px(ui), Vec::new()),
            Block::Fixed { height } => (*height, Vec::new()),
            Block::Footer { height, .. } => {
                footer_height = *height;
                (0.0, Vec::new())
            }
        };
        tops.push(y);
        heights.push(height);
        layouts.push(block_layouts);
        if !block.is_footer() {
            y += height;
        }
    }

    Measured {
        frame: *frame,
        blocks: blocks.to_vec(),
        tops,
        heights,
        content_height: y,
        width: frame.w,
        viewport: frame.viewport,
        footer_height,
        layouts,
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

    /// A scroll region resuming at a caller-held offset. The offset stays
    /// private so it can only ever be read back through `offset()` and moved
    /// through `scroll_by`, which clamps against the measured content.
    pub fn with_offset(offset: f32) -> Self {
        Self { offset }
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

/// Paint flowing content and the pinned footer, into the frame that was
/// measured. `Fixed` regions are *not* painted: callers read
/// `Measured::region` and draw their own pixels, because this layer owns
/// layout, not widgets.
///
/// Only fully visible blocks are drawn: this pipeline has no scissor test,
/// so a discrete block that straddles the viewport's bottom edge waits for
/// the scroll that reveals it whole instead of leaking cut glyphs over
/// whatever sits below the region.
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
    let frame = measured.frame;

    // The footer first, in its own declared colour: content may then draw
    // over nothing, because the scroll mapping never hands out a line below
    // `footer_top`.
    if let Some(Block::Footer { color, .. }) =
        measured.blocks.iter().find(|block| block.is_footer())
    {
        batcher.screen_rect(
            screen,
            frame.x,
            frame.y + measured.footer_top(),
            frame.w,
            measured.footer_height,
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
        // Fully visible or not at all: no scissor means no straddlers.
        if top + height <= 0.0 || top + height > measured.viewport + 0.5 {
            continue;
        }
        match block {
            Block::Line { color, .. } => {
                for layout in &measured.layouts[index] {
                    let line_y = frame.y + top;
                    text.paint_layout(
                        *layout,
                        frame.x,
                        line_y,
                        TextBounds {
                            left: frame.x.round() as i32,
                            top: frame.y.round() as i32,
                            right: (frame.x + frame.w).round() as i32,
                            bottom: (frame.y + measured.footer_top()).round() as i32,
                        },
                        *color,
                    );
                }
            }
            Block::Row { .. } => {
                let label = &measured.layouts[index][0];
                let value = &measured.layouts[index][1];
                text.paint_layout(
                    *label,
                    frame.x,
                    frame.y + top,
                    TextBounds {
                        left: frame.x.round() as i32,
                        top: frame.y.round() as i32,
                        right: (frame.x + frame.w).round() as i32,
                        bottom: (frame.y + measured.footer_top()).round() as i32,
                    },
                    match block { Block::Row { color, .. } => *color, _ => unreachable!() },
                );
                text.paint_layout(
                    *value,
                    frame.x + frame.w - value.width,
                    frame.y + top,
                    TextBounds {
                        left: frame.x.round() as i32,
                        top: frame.y.round() as i32,
                        right: (frame.x + frame.w).round() as i32,
                        bottom: (frame.y + measured.footer_top()).round() as i32,
                    },
                    match block { Block::Row { value_color, .. } => *value_color, _ => unreachable!() },
                );
            }
            Block::Bar { color, fraction, .. } => {
                let label = &measured.layouts[index][0];
                text.paint_layout(
                    *label,
                    frame.x,
                    frame.y + top,
                    TextBounds {
                        left: frame.x.round() as i32,
                        top: frame.y.round() as i32,
                        right: (frame.x + frame.w).round() as i32,
                        bottom: (frame.y + measured.footer_top()).round() as i32,
                    },
                    match block { Block::Bar { label_color, .. } => *label_color, _ => unreachable!() },
                );
                let line_h = measured.heights[index];
                // Track from the row's bar origin to the right edge, centred
                // on the text line; the track is the colour at low alpha.
                let track_h = Space::Xs.px(ui);
                let track_y = frame.y + top + (line_h - track_h) * 0.5;
                let track_x = frame.x + frame.w * BAR_START;
                let track_w = frame.w * (1.0 - BAR_START);
                let track = [color[0], color[1], color[2], color[3] * 0.18];
                batcher.screen_rect(
                    screen,
                    snap(track_x),
                    snap(track_y),
                    snap(track_w),
                    snap(track_h),
                    track,
                    Text::solid_uv(),
                );
                let filled = (track_w * fraction.clamp(0.0, 1.0)).max(0.0);
                if filled > 0.5 {
                    batcher.screen_rect(
                        screen,
                        snap(track_x),
                        snap(track_y),
                        snap(filled),
                        snap(track_h),
                        *color,
                        Text::solid_uv(),
                    );
                }
            }
            Block::Keyed { key_col, .. } => {
                let key = &measured.layouts[index][0];
                let prose = &measured.layouts[index][1];
                text.paint_layout(
                    *key,
                    frame.x,
                    frame.y + top,
                    TextBounds {
                        left: frame.x.round() as i32,
                        top: frame.y.round() as i32,
                        right: (frame.x + frame.w).round() as i32,
                        bottom: (frame.y + measured.footer_top()).round() as i32,
                    },
                    match block { Block::Keyed { key_color, .. } => *key_color, _ => unreachable!() },
                );
                text.paint_layout(
                    *prose,
                    frame.x + *key_col,
                    frame.y + top,
                    TextBounds {
                        left: frame.x.round() as i32,
                        top: frame.y.round() as i32,
                        right: (frame.x + frame.w).round() as i32,
                        bottom: (frame.y + measured.footer_top()).round() as i32,
                    },
                    match block { Block::Keyed { text_color, .. } => *text_color, _ => unreachable!() },
                );
            }
            Block::Rule => {
                batcher.screen_rect(
                    screen,
                    snap(frame.x),
                    snap(frame.y + top),
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

    fn frame(w: f32, viewport: f32) -> Frame {
        Frame::new(0.0, 0.0, w, viewport)
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
        let measured = measure(&mut text, ui(), &frame(600.0, 900.0), &blocks());
        // One title line + one small gap + a 600 px region, in that order.
        let title = Step::Title.px(ui()) as f32 * LINE_ADVANCE_FACTOR;
        let gap = Space::Sm.px(ui());
        assert!((measured.content_height - (title + gap + 600.0)).abs() < 0.5);
        // Measuring emitted no quads: layout precedes drawing, literally.
        assert_eq!(batch.instances.len(), 0, "measuring drew something");
        // Painting submits exactly the measured block as a Glyphon area.
        let scroll = Scroll::new();
        paint(&measured, &scroll, &mut batch, &screen(), &mut text);
        assert_eq!(text.pending_area_count(), 1, "painting submitted the wrong text areas");
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
        let measured = measure(&mut text, ui(), &frame(300.0, 900.0), &[block]);
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
        let measured = measure(&mut text, ui(), &frame(600.0, 400.0), &b);
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
        let measured = measure(&mut text, ui(), &frame(600.0, 400.0), &b);
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
        let measured = measure(&mut text, ui, &frame(600.0, 900.0), &blocks());
        let title = Step::Title.px(ui) as f32 * LINE_ADVANCE_FACTOR;
        let gap = Space::Sm.px(ui);
        assert!((measured.content_height - (title + gap + 600.0)).abs() < 0.5);
    }

    #[test]
    fn a_row_measures_one_line_and_paints_its_value_at_the_right_edge() {
        let mut text = Text::new();
        if text.missing_font {
            eprintln!("no system font available; the row check is not exercised");
            return;
        }
        text.set_ui_scale(ui());
        let block = Block::Row {
            label: "population".into(),
            value: "4120".into(),
            step: Step::Small,
            color: [1.0; 4],
            value_color: [1.0; 4],
        };
        let measured = measure(&mut text, ui(), &frame(300.0, 900.0), &[block]);
        let small = Step::Small.px(ui()) as f32 * LINE_ADVANCE_FACTOR;
        assert!((measured.content_height - small).abs() < 0.5);

        let mut batch = Batcher::default();
        paint(&measured, &Scroll::new(), &mut batch, &screen(), &mut text);
        // Glyphon receives two shaped areas: the label and right-aligned value.
        assert_eq!(text.pending_area_count(), 2);
    }

    #[test]
    fn a_bar_fills_from_its_origin_by_its_fraction() {
        let mut text = Text::new();
        if text.missing_font {
            eprintln!("no system font available; the bar check is not exercised");
            return;
        }
        text.set_ui_scale(ui());
        let block = Block::Bar {
            label: "residential".into(),
            step: Step::Small,
            label_color: [1.0; 4],
            color: [0.2, 0.8, 0.3, 1.0],
            fraction: 0.5,
        };
        let measured = measure(&mut text, ui(), &frame(300.0, 900.0), &[block]);
        let mut batch = Batcher::default();
        paint(&measured, &Scroll::new(), &mut batch, &screen(), &mut text);
        // Quads come in no particular order; look for the fill: a quad whose
        // right edge lands at half the track (origin 0.38 * 300, width 0.62
        // * 300, filled 0.5 -> right edge at 0.38 + 0.31 = 0.69 of 300).
        let expected_right = 300.0 * (BAR_START + (1.0 - BAR_START) * 0.5);
        let hit = batch.instances.iter().any(|i| {
            let right = (i.pos[0] + i.size[0] + 1.0) / 2.0 * screen().w;
            (right - expected_right).abs() <= 1.5
        });
        assert!(hit, "no quad's right edge sat at the bar's fill edge {expected_right}");
    }

    #[test]
    fn paint_lands_content_in_the_frame_it_was_measured_in() {
        let mut text = Text::new();
        if text.missing_font {
            eprintln!("no system font available; the frame check is not exercised");
            return;
        }
        text.set_ui_scale(ui());
        let block = Block::Line {
            face: Face::Body,
            step: Step::Small,
            text: "panel line".into(),
            color: [1.0; 4],
        };
        let frame = Frame::new(50.0, 40.0, 300.0, 900.0);
        let measured = measure(&mut text, ui(), &frame, &[block]);
        let mut batch = Batcher::default();
        paint(&measured, &Scroll::new(), &mut batch, &screen(), &mut text);
        assert_eq!(text.pending_area_count(), 1, "the line was not submitted to Glyphon");
    }

    #[test]
    fn a_block_straddling_the_viewport_bottom_waits_instead_of_leaking() {
        let mut text = Text::new();
        if text.missing_font {
            eprintln!("no system font available; the visibility check is not exercised");
            return;
        }
        text.set_ui_scale(ui());
        let blocks = [
            Block::Line { face: Face::Body, step: Step::Small, text: "top".into(), color: [1.0; 4] },
            Block::Line { face: Face::Body, step: Step::Small, text: "straddler".into(), color: [1.0; 4] },
        ];
        // Viewport clearly between one and two lines: the second block
        // straddles the bottom edge and must wait rather than leak.
        let small = Step::Small.px(ui()) as f32 * LINE_ADVANCE_FACTOR;
        let measured = measure(&mut text, ui(), &frame(300.0, small * 1.3), &blocks);
        let mut batch = Batcher::default();
        paint(&measured, &Scroll::new(), &mut batch, &screen(), &mut text);
        // Only the first line was submitted; the straddling line waits for scroll.
        assert_eq!(text.pending_area_count(), 1, "a straddling block leaked into the text areas");
    }
}
