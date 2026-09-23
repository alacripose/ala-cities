//! Glyphon-backed screen-space text.
//!
//! The application submits shaped cosmic-text buffers as text areas during its
//! layout pass. Glyphon owns rasterization, dynamic etagere atlas packing, and
//! GPU rendering; this module only owns application text buffers and placement.

use glyphon::{
    Attrs, Buffer, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea,
    TextAtlas, TextBounds, TextRenderer, Viewport, Wrap,
};

use crate::design::{Step, UiScale};
use crate::render::{Batcher, Face, Screen};

const LINE_ADVANCE_FACTOR: f32 = 1.35;

/// A shaped buffer retained for the current frame.
#[derive(Clone, Copy, Debug)]
pub struct TextLayout {
    buffer: usize,
    pub width: f32,
    pub height: f32,
    pub lines: usize,
}

#[derive(Clone, Copy, Debug)]
struct Area {
    buffer: usize,
    left: f32,
    top: f32,
    bounds: TextBounds,
    color: Color,
}

/// Application-side text state. Glyphon owns the GPU atlas and renderer in
/// `crate::render::Gpu`; this object owns only CPU-side cosmic-text buffers.
pub struct Text {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    buffers: Vec<Buffer>,
    areas: Vec<Area>,
    ui: UiScale,
    /// Kept public for the existing startup diagnostic. FontSystem performs
    /// fallback internally, so a missing primary font is not a failure state.
    pub missing_font: bool,
    pub refused: bool,
}

impl Text {
    pub fn new() -> Self {
        Self {
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
            buffers: Vec::new(),
            areas: Vec::new(),
            ui: UiScale::default(),
            missing_font: false,
            refused: false,
        }
    }

    pub fn set_ui_scale(&mut self, ui: UiScale) {
        self.ui = ui;
    }

    pub fn ui_scale(&self) -> UiScale {
        self.ui
    }

    /// cosmic-text's Metrics are the source of truth now: Step::px(ui) is the
    /// physical font size and the line height is the design advance.
    pub fn scale_defects(&self) -> Vec<String> {
        Vec::new()
    }

    /// Frame diagnostics: number of shaped buffers submitted so far. Glyphon
    /// itself owns the atlas occupancy and intentionally does not expose it.
    pub fn occupancy(&self) -> (usize, u32) {
        (self.buffers.len(), 0)
    }

    pub fn px(&self, step: Step) -> u32 {
        step.px(self.ui)
    }

    pub fn solid_uv() -> [f32; 4] {
        [0.5, 0.5, 0.5, 0.5]
    }

    fn attrs(face: Face, color: Color) -> Attrs<'static> {
        let family = match face {
            Face::Body => Family::SansSerif,
            Face::Mono => Family::Monospace,
        };
        Attrs::new().family(family).color(color)
    }

    pub fn color(color: [f32; 4]) -> Color {
        Color::rgba(
            (color[0].clamp(0.0, 1.0) * 255.0).round() as u8,
            (color[1].clamp(0.0, 1.0) * 255.0).round() as u8,
            (color[2].clamp(0.0, 1.0) * 255.0).round() as u8,
            (color[3].clamp(0.0, 1.0) * 255.0).round() as u8,
        )
    }

    fn make_buffer(&mut self, face: Face, step: Step, text: &str, color: Color) -> Buffer {
        let px = self.px(step) as f32;
        let mut buffer = Buffer::new(
            &mut self.font_system,
            Metrics::new(px, px * LINE_ADVANCE_FACTOR),
        );
        buffer.set_text(
            text,
            &Self::attrs(face, color),
            Shaping::Advanced,
            None,
        );
        buffer
    }

    fn shape(&mut self, buffer: &mut Buffer) {
        buffer.shape_until_scroll(&mut self.font_system, false);
    }

    /// Shape and retain a buffer. Width and wrapping are part of the buffer,
    /// not a second hand-written word-width algorithm, so complex scripts and
    /// fallback glyphs participate in the same layout as Latin text.
    pub fn layout(
        &mut self,
        face: Face,
        step: Step,
        text: &str,
        color: [f32; 4],
        width: f32,
        wrap: Wrap,
    ) -> TextLayout {
        let mut buffer = self.make_buffer(face, step, text, Self::color(color));
        buffer.set_size(Some(width.max(1.0)), None);
        buffer.set_wrap(wrap);
        self.shape(&mut buffer);
        let metrics = buffer.metrics();
        let mut measured_width: f32 = 0.0;
        let mut lines = 0;
        for run in buffer.layout_runs() {
            measured_width = measured_width.max(run.line_w);
            lines += 1;
        }
        let lines = lines.max(1);
        let height = lines as f32 * metrics.line_height;
        self.buffers.push(buffer);
        TextLayout {
            buffer: self.buffers.len() - 1,
            width: measured_width,
            height,
            lines,
        }
    }

    /// Measure without retaining a frame area or buffer.
    pub fn measure_step(&mut self, face: Face, text: &str, step: Step) -> f32 {
        let mut buffer = self.make_buffer(face, step, text, Self::color([1.0; 4]));
        buffer.set_wrap(Wrap::None);
        self.shape(&mut buffer);
        buffer
            .layout_runs()
            .map(|run| run.line_w)
            .fold(0.0, f32::max)
    }

    /// Glyphon positions from the top of the TextArea, so the old font-specific
    /// baseline offset no longer belongs in the application layer.
    pub fn ascent(&mut self, _face: Face, _step: Step) -> f32 {
        0.0
    }

    pub fn paint_layout(&mut self, layout: TextLayout, x: f32, y: f32, bounds: TextBounds, color: [f32; 4]) {
        self.areas.push(Area {
            buffer: layout.buffer,
            left: x.round(),
            top: y.round(),
            bounds,
            color: Self::color(color),
        });
    }

    /// Direct labels outside the shared flow layout. New UI code should prefer
    /// `layout` + `paint_layout`; this method keeps the small HUD primitives
    /// concise while still submitting a native Glyphon TextArea.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_step(
        &mut self,
        face: Face,
        _batch: &mut Batcher,
        screen: &Screen,
        x: f32,
        y: f32,
        step: Step,
        color: [f32; 4],
        text: &str,
    ) -> f32 {
        let layout = self.layout(face, step, text, color, screen.w.max(1.0), Wrap::None);
        self.paint_layout(
            layout,
            x,
            y,
            TextBounds {
                left: 0,
                top: 0,
                right: screen.w.max(1.0) as i32,
                bottom: screen.h.max(1.0) as i32,
            },
            color,
        );
        layout.width
    }

    /// Remove submitted areas and buffers after Glyphon has rendered the frame.
    pub fn pending_area_count(&self) -> usize {
        self.areas.len()
    }

    pub fn clear(&mut self) {
        self.buffers.clear();
        self.areas.clear();
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atlas: &mut TextAtlas,
        renderer: &mut TextRenderer,
        viewport: &mut Viewport,
        screen: Screen,
    ) -> Result<(), glyphon::PrepareError> {
        viewport.update(
            queue,
            Resolution {
                width: screen.w.max(1.0) as u32,
                height: screen.h.max(1.0) as u32,
            },
        );
        let areas = self.areas.iter().map(|area| TextArea {
            buffer: &self.buffers[area.buffer],
            left: area.left,
            top: area.top,
            scale: 1.0,
            bounds: area.bounds,
            default_color: area.color,
            custom_glyphs: &[],
        });
        renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            atlas,
            viewport,
            areas,
            &mut self.swash_cache,
        )
    }
}

impl Default for Text {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosmic_text_shapes_multilingual_content() {
        let mut text = Text::new();
        let latin = text.measure_step(Face::Body, "Café", Step::Body);
        let mixed = text.measure_step(Face::Body, "東京 · العربية · café", Step::Body);
        assert!(latin > 0.0);
        assert!(mixed > latin, "fallback shaping should account for every script");
    }

    #[test]
    fn metrics_use_the_physical_design_step_and_line_advance() {
        let mut text = Text::new();
        text.set_ui_scale(UiScale(1.25));
        let layout = text.layout(Face::Body, Step::Body, "one\ntwo", [1.0; 4], 400.0, Wrap::Word);
        let metrics = text.buffers[layout.buffer].metrics();
        assert_eq!(metrics.font_size, Step::Body.px(UiScale(1.25)) as f32);
        assert!((metrics.line_height - metrics.font_size * LINE_ADVANCE_FACTOR).abs() < 0.01);
        assert_eq!(layout.lines, 2);
    }

    #[test]
    fn submitted_text_is_frame_owned_and_clears_after_render() {
        let mut text = Text::new();
        let mut batch = Batcher::default();
        let layout = text.layout(Face::Mono, Step::Small, "42", [1.0; 4], 200.0, Wrap::None);
        text.paint_layout(
            layout,
            10.4,
            20.8,
            TextBounds { left: 0, top: 0, right: 200, bottom: 100 },
            [1.0; 4],
        );
        assert_eq!(text.areas.len(), 1);
        assert_eq!(text.buffers.len(), 1);
        let _ = &mut batch;
        text.clear();
        assert!(text.areas.is_empty());
        assert!(text.buffers.is_empty());
    }
}
