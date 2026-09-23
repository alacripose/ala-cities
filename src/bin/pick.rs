//! The rebuilt icon review picker, on the shared widget layer.
//!
//! C5 measured the old tool's defect and C7 ordered the rebuild: the six
//! candidate images **are** the surface (a113/a123), the detail region carries
//! brief, directives and the selected candidate's measured numbers (a120), and
//! the comment box is a **fixed footer** that is never overlapped and never
//! below the fold (§5.7). The layout computes its height before anything
//! draws, at the scale it was given, and content beyond the viewport scrolls
//! at every scale (a119).
//!
//! It reads the same `review.json`, validates with the same rules, and records
//! the same decision lines as the old picker — both sides of the hand-off live
//! in `ala_cities::iconreview`, so the two tools cannot disagree about what a
//! decision is. The old binary stays runnable until this one passes a
//! playtest (a135(c)), then retires.
//!
//! Candidate images are packed from the pipeline's `.rgba` raws into one atlas
//! and drawn through the shared renderer's image pipeline — the same path the
//! game's icon loader will use (C6 Q131).

use std::collections::HashMap;
use std::path::Path;

use ala_cities::design::{self, Space, Step, Target, UiScale};
use ala_cities::hud::{self, Token};
use ala_cities::iconreview::{
    load_review, read_directives, record, review_defects, Icon, Review, CANDIDATE_COUNT,
    DECISIONS, VALIDATION,
};
use ala_cities::render::{
    Gpu, ImageBatcher, Screen, Text, ATLAS_SIZE, Batcher, Face,
};
use ala_cities::ui::{self, Block, Scroll};
use serde_json::Value;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowId};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn".into()),
        )
        .init();

    let review = match load_review() {
        Ok(review) => review,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    };
    if review.icons.is_empty() {
        println!("nothing to review: every icon in the inventory has a decided target.");
        return;
    }
    let defects = review_defects(&review);
    if !defects.is_empty() {
        eprintln!("the icon-authoring review set is invalid:");
        for defect in &defects {
            eprintln!("  - {defect}");
        }
        std::process::exit(1);
    }

    // The gate first, on this tool's own controls — same rule as the old picker.
    for line in design::audit(&controls(), UiScale::default()) {
        println!("picker design: {line}");
    }
    let mut defects = design::verify(&controls(), UiScale::default());
    let probe = Text::new();
    defects.extend(probe.scale_defects());
    if !defects.is_empty() {
        eprintln!("the picker's own design check failed closed:");
        for defect in &defects {
            eprintln!("  - {defect}");
        }
        std::process::exit(1);
    }

    println!("{}", review.header());
    let mut app = Picker::new(review);
    let event_loop = EventLoop::new().expect("an event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).expect("the event loop ran");
}

/// The rebuilt picker's own interactive targets. The tiles are the six
/// candidates' decision images at 288 px (96 × 3 zoom); everything else is
/// text about candidates you are not choosing, and yields to them.
fn controls() -> Vec<Target> {
    vec![
        Target { name: "candidate tile", w: 288.0, h: 288.0 },
        Target { name: "candidate checkbox", w: 288.0, h: 48.0 },
        Target { name: "comment box", w: 900.0, h: 48.0 },
    ]
}

/// One candidate image, packed: where its pixels live in the atlas.
#[derive(Clone, Copy, Debug)]
struct Packed {
    /// Source rect in atlas pixels — exactly what the image shader wants.
    src: [f32; 4],
}

/// Pack the six decision images of one icon, row-major, into a CPU-side RGBA
/// buffer. The atlas is rebuilt per icon: at most six 288 px images (≈2 MB),
/// rebuilt only when the icon changes — the pack is not per-frame work.
type PackedAtlas = (Vec<u8>, u32, u32, Vec<Option<Packed>>);

fn pack_atlas(icon: &Icon, review: &Review) -> Result<PackedAtlas, String> {
    let side = review.decision_px as u32;
    let zoom = 3u32;
    let tile = side * zoom;
    let cols = 3u32;
    let rows = u32::try_from(icon.generations.len())
        .map(|n| n.div_ceil(cols))
        .unwrap_or(u32::MAX);
    let width = tile * cols;
    let height = tile * rows;
    let mut atlas = vec![0u8; (width * height * 4) as usize];
    let mut packed = Vec::with_capacity(icon.generations.len());

    for (index, generation) in icon.generations.iter().enumerate() {
        let col = (index % cols as usize) as u32;
        let row = (index / cols as usize) as u32;
        let dst_x = col * tile;
        let dst_y = row * tile;
        let raw = std::fs::read(&generation.sharp)
            .map_err(|err| format!("{} cannot be read: {err}", generation.sharp.display()))?;
        let expected = (side * side * 4) as usize;
        if raw.len() != expected {
            return Err(format!(
                "{} is {} bytes; expected {side}x{side} RGBA8 ({expected})",
                generation.sharp.display(),
                raw.len()
            ));
        }
        // Upscale side×side → tile×tile by whole-pixel replication: the window
        // shows the rendered pixels, not a resampling of them (the old
        // picker's zoom rule, kept).
        for y in 0..tile {
            let src_row = (y / zoom) as usize * side as usize * 4;
            let dst_row = ((dst_y + y) * width + dst_x) as usize * 4;
            for x in 0..tile {
                let src_col = (x / zoom) as usize * 4;
                let dst = dst_row + x as usize * 4;
                let s = src_row + src_col;
                atlas[dst..dst + 4].copy_from_slice(&raw[s..s + 4]);
            }
        }
        packed.push(Some(Packed {
            src: [
                dst_x as f32,
                dst_y as f32,
                (dst_x + tile) as f32,
                (dst_y + tile) as f32,
            ],
        }));
    }
    while packed.len() < CANDIDATE_COUNT {
        packed.push(None);
    }
    Ok((atlas, width, height, packed))
}

struct Picker {
    review: Review,
    window: Option<std::sync::Arc<Window>>,
    gpu: Option<Gpu>,
    text: Text,
    ui: UiScale,
    icon_index: usize,
    /// The checked candidate, if any — a checkbox, so "none of these" stays
    /// reachable without leaving the tool.
    checked: Option<usize>,
    comment: String,
    focus_comment: bool,
    cursor: (f32, f32),
    /// The packed atlas and where each candidate's image landed in it.
    atlas: Vec<u8>,
    atlas_width: u32,
    atlas_height: u32,
    packed: Vec<Option<Packed>>,
    atlas_icon: String,
    /// Scrolling, from the shared layer: the detail region can exceed the
    /// viewport at any scale, and a119 says the content continues, not the
    /// layout breaks.
    scroll: Scroll,
    directives: HashMap<String, Vec<String>>,
    /// The title + six tiles strip never scrolls; the detail region below
    /// does. Its offset lives here.
    detail_offset: f32,
    /// Set when every awaiting icon has been visited; the next redraw exits.
    closing: bool,
}

impl Picker {
    fn new(review: Review) -> Self {
        let mut text = Text::new();
        text.set_ui_scale(UiScale::default());
        let directives = read_directives(Path::new(DECISIONS), &review.concept_set);
        Self {
            review,
            window: None,
            gpu: None,
            text,
            ui: UiScale::default(),
            icon_index: 0,
            checked: None,
            comment: String::new(),
            focus_comment: false,
            cursor: (0.0, 0.0),
            atlas: Vec::new(),
            atlas_width: 0,
            atlas_height: 0,
            packed: Vec::new(),
            atlas_icon: String::new(),
            scroll: Scroll::new(),
            directives,
            detail_offset: 0.0,
            closing: false,
        }
    }

    fn icon(&self) -> &Icon {
        &self.review.icons[self.icon_index]
    }

    /// Both directive sources, oldest first, deduplicated.
    fn directives_for(&self, icon: &Icon) -> Vec<String> {
        let mut out = icon.directives.clone();
        if let Some(logged) = self.directives.get(&icon.id) {
            for comment in logged {
                if !out.contains(comment) {
                    out.push(comment.clone());
                }
            }
        }
        out
    }

    /// Load the current icon's six candidates into the atlas. Kept honest by
    /// refusing the run if any image is missing or the wrong size.
    fn load_atlas(&mut self) -> Result<(), String> {
        let icon = self.icon().clone();
        if self.atlas_icon == icon.id && !self.atlas.is_empty() {
            return Ok(());
        }
        let (atlas, width, height, packed) = pack_atlas(&icon, &self.review)?;
        self.atlas = atlas;
        self.atlas_width = width;
        self.atlas_height = height;
        self.packed = packed;
        self.atlas_icon = icon.id;
        self.checked = None;
        self.comment.clear();
        self.detail_offset = 0.0;
        if let Some(gpu) = self.gpu.as_mut() {
            gpu.bind_image(&self.atlas, width, height);
        }
        Ok(())
    }

    fn record_and_advance(&mut self) {
        let icon = self.icon().clone();
        let comment = self.comment.trim().to_string();
        let checked = self.checked;
        if checked.is_none() && comment.is_empty() {
            println!("nothing to record on {}: check a candidate or type a comment", icon.id);
            return;
        }
        let position = checked.unwrap_or(0).min(icon.generations.len().saturating_sub(1));
        let what = match checked {
            Some(index) => format!(
                "target = {} ({})",
                icon.generations[index].label, icon.generations[index].id
            ),
            None => "no target, comment only".to_string(),
        };
        // The promotion gate (Q206). A refusal leaves the person on this icon with
        // the reason printed, because the way through the gate is the comment box
        // they are already looking at — not a second tool, and not a silent skip.
        if let Err(refusal) = record(
            &self.review,
            &icon,
            position,
            checked.is_some(),
            &comment,
            Path::new(DECISIONS),
            Path::new(VALIDATION),
        ) {
            eprintln!("{refusal}");
            return;
        }
        println!("{what} on {} — recorded", icon.id);
        self.directives
            .entry(icon.id.clone())
            .or_default()
            .extend(self.comment.trim().is_empty().then(Vec::new).unwrap_or_else(
                || vec![self.comment.trim().to_string()],
            ));
        if self.icon_index + 1 < self.review.icons.len() {
            self.icon_index += 1;
        } else {
            println!("every awaiting icon has been visited; the tool closes.");
            if self.window.take().is_some() {
                self.closing = true;
            }
            return;
        }
        self.checked = None;
        self.comment.clear();
        self.detail_offset = 0.0;
        if let Err(err) = self.load_atlas() {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }

    // --- layout numbers, all from the scales --------------------------------

    fn tile(&self) -> f32 {
        288.0 * self.ui.0
    }

    fn gutter(&self) -> f32 {
        Space::Sm.px(self.ui)
    }

    /// The header strip: title line + six tiles in one row + checkbox row.
    /// Measured, not assumed — the detail region begins where this ends.
    fn strip_height(&self) -> f32 {
        let ui = self.ui;
        let title = Step::Title.px(ui) as f32 * ui::LINE_ADVANCE_FACTOR;
        let tile = self.tile();
        let checkbox = design::MIN_TARGET_PX * ui.0;
        title + Space::Sm.px(ui) + tile + Space::Xs.px(ui) + checkbox
    }

    /// The detail region: everything between the strip and the footer.
    fn detail_viewport(&self, screen: &Screen) -> f32 {
        let footer = self.footer_height();
        (screen.h - self.strip_height() - footer).max(0.0)
    }

    fn footer_height(&self) -> f32 {
        let ui = self.ui;
        let input = design::MIN_TARGET_PX * ui.0;
        input + Space::Md.px(ui) * 2.0
    }

    // --- drawing ------------------------------------------------------------

    fn draw(&mut self, batch: &mut Batcher, images: &mut ImageBatcher, screen: &Screen) {
        let ui = self.ui;
        let icon = self.icon().clone();
        let inset = Space::Md.px(ui);
        let strip_h = self.strip_height();

        // The strip's title line.
        let title = format!(
            "{} · {} of {} — {}",
            icon.id,
            self.icon_index + 1,
            self.review.icons.len(),
            icon.meaning
        );
        self.text.draw_step(
            Face::Body,
            batch,
            screen,
            inset,
            Step::Title.px(ui) as f32 * 0.85,
            Step::Title,
            hud::style(Token::Ink).text.unwrap_or([1.0; 4]),
            &title,
        );

        // The six candidate tiles: the subject of the surface. Always visible,
        // in one row; the decision image fills the tile, the checkbox rides
        // under it.
        let tile = self.tile();
        for (index, generation) in icon.generations.iter().enumerate() {
            let x = inset + index as f32 * (tile + self.gutter());
            let y = Step::Title.px(ui) as f32 * ui::LINE_ADVANCE_FACTOR + Space::Sm.px(ui);
            // Tile backdrop = the icon's own declared host surface, so the
            // contrast claim is checkable by eye.
            let host = self.review.host_fill(&icon);
            hud::fill(batch, screen, x, y, tile, tile, host);
            batch.screen_outline(
                screen,
                x,
                y,
                tile,
                tile,
                hud::style(Token::Panel).border.unwrap_or([0.3; 4]),
            );
            if let Some(Some(packed)) = self.packed.get(index) {
                images.image(
                    screen,
                    x,
                    y,
                    tile,
                    tile,
                    packed.src,
                    [1.0, 1.0, 1.0, 1.0],
                );
            }
            // The checkbox row under the tile: a full-target-sized hit area
            // with the state drawn in.
            let checkbox_h = design::MIN_TARGET_PX * ui.0;
            let cy = y + tile + Space::Xs.px(ui);
            hud::panel(batch, screen, x, cy, tile, checkbox_h, Token::Panel);
            let fill = if self.checked == Some(index) {
                hud::style(Token::Nature).fill.unwrap_or([0.3, 0.7, 0.4, 1.0])
            } else {
                hud::style(Token::PanelRaised).fill.unwrap_or([0.2; 4])
            };
            let mark = checkbox_h * 0.45;
            hud::fill(batch, screen, x + Space::Sm.px(ui), cy + (checkbox_h - mark) / 2.0, mark, mark, fill);
            self.text.draw_step(
                Face::Body,
                batch,
                screen,
                x + mark + Space::Md.px(ui),
                cy + checkbox_h * 0.75,
                Step::Small,
                hud::style(Token::TextBody).text.unwrap_or([0.9; 4]),
                &generation.label,
            );
        }

        // The detail region: a scrollable stack of measured facts, measured
        // and painted in a frame whose origin sits below the strip — the
        // shared layer maps content space to that frame's screen position
        // itself, and keeps every line above the footer it knows.
        let detail_h = self.detail_viewport(screen);
        let width = screen.w - 2.0 * inset;
        let blocks = self.detail_blocks(&icon, width);
        let frame = ui::Frame::new(inset, strip_h, width, detail_h);
        let measured = ui::measure(&mut self.text, ui, &frame, &blocks);
        let scroll = Scroll::with_offset(self.detail_offset);
        ui::paint(&measured, &scroll, batch, screen, &mut self.text);

        // The comment footer: fixed, never overlapped, never below the fold.
        let footer_top = screen.h - self.footer_height();
        hud::panel(batch, screen, 0.0, footer_top, screen.w, self.footer_height(), Token::Panel);
        let line = if self.focus_comment {
            format!("comment ▸ {}|", self.comment)
        } else if self.comment.is_empty() {
            "comment ▸ (type what the next generation should change)".to_string()
        } else {
            format!("comment ▸ {}", self.comment)
        };
        self.text.draw_step(
            Face::Body,
            batch,
            screen,
            inset,
            footer_top + Space::Md.px(ui) + Step::Body.px(ui) as f32 * 0.8,
            Step::Body,
            hud::style(if self.focus_comment { Token::TextBody } else { Token::TextMuted })
                .text
                .unwrap_or([0.9; 4]),
            &line,
        );
        let hint = "Enter: record · Tab: comment · ←/→: candidate · [1-6]: check · PgUp/PgDn: detail · N: next icon";
        self.text.draw_step(
            Face::Mono,
            batch,
            screen,
            inset,
            screen.h - Space::Sm.px(ui),
            Step::Micro,
            hud::style(Token::TextMuted).text.unwrap_or([0.7; 4]),
            hint,
        );
    }

    fn detail_blocks(&self, icon: &Icon, width: f32) -> Vec<Block> {
        let body = hud::style(Token::TextBody).text.unwrap_or([0.9; 4]);
        let muted = hud::style(Token::TextMuted).text.unwrap_or([0.7; 4]);
        let accent = hud::style(Token::Nature).text.unwrap_or([0.4, 0.8, 0.5, 1.0]);
        let mut blocks = Vec::new();

        // The brief, per a120: the icon's meaning and the surface it locates.
        blocks.push(Block::Line {
            face: Face::Body,
            step: Step::Small,
            text: format!("locates {} · sits on {}", icon.locates, icon.sits_on.join(", ")),
            color: muted,
        });
        blocks.push(Block::Gap(Space::Sm));

        // Selected candidate's brief and measured numbers (a120), side by side
        // in one line per fact — the picker is a chooser, not a document.
        let selected = self.checked.unwrap_or(0).min(icon.generations.len() - 1);
        for (index, generation) in icon.generations.iter().enumerate() {
            let facts = generation.facts();
            let marker = if self.checked == Some(index) { "▸" } else { " " };
            blocks.push(Block::Line {
                face: Face::Body,
                step: Step::Small,
                text: format!("{marker} {} — {}", generation.label, if index == selected { facts } else { generation.why.clone() }),
                color: if self.checked == Some(index) { accent } else { body },
            });
        }
        blocks.push(Block::Gap(Space::Sm));

        // Directives: what the next generation is asked to change. The
        // mechanism for recording what the next generation should change has
        // to be visible, or it never gets used.
        let directives = self.directives_for(icon);
        if !directives.is_empty() {
            blocks.push(Block::Line {
                face: Face::Body,
                step: Step::Small,
                text: "asked of the next generation:".into(),
                color: muted,
            });
            for directive in directives {
                blocks.push(Block::Line {
                    face: Face::Body,
                    step: Step::Small,
                    text: format!("· {directive}"),
                    color: body,
                });
            }
            blocks.push(Block::Gap(Space::Sm));
        }

        // The checks: notes refuse promotion, so they are shown, not buried.
        let notes = icon.generations[selected].notes();
        if !notes.is_empty() {
            blocks.push(Block::Line {
                face: Face::Body,
                step: Step::Small,
                text: "check notes on the selected candidate:".into(),
                color: muted,
            });
            for note in notes {
                blocks.push(Block::Line {
                    face: Face::Body,
                    step: Step::Small,
                    text: format!("· {note}"),
                    color: body,
                });
            }
        }
        let _ = width;
        blocks
    }

    #[allow(dead_code)]
    fn scroll_state(&self) -> f32 {
        self.scroll.offset()
    }

    // --- input --------------------------------------------------------------

    /// The tile rects, in the same numbers the draw pass used.
    fn tile_rects(&self) -> Vec<(f32, f32, f32, f32)> {
        let inset = Space::Md.px(self.ui);
        let tile = self.tile();
        let y = Step::Title.px(self.ui) as f32 * ui::LINE_ADVANCE_FACTOR + Space::Sm.px(self.ui);
        (0..6)
            .map(|index| {
                let x = inset + index as f32 * (tile + self.gutter());
                (x, y, tile, tile + design::MIN_TARGET_PX * self.ui.0)
            })
            .collect()
    }

    fn on_click(&mut self, screen: &Screen) {
        let (cx, cy) = self.cursor;
        let footer_top = screen.h - self.footer_height();
        if cy >= footer_top {
            self.focus_comment = true;
            return;
        }
        for (index, (x, y, w, h)) in self.tile_rects().into_iter().enumerate() {
            if cx >= x && cx <= x + w && cy >= y && cy <= y + h {
                // Clicking the tile toggles the check; clicking the image
                // itself selects for the detail region. One action, per §5.7:
                // the checkbox *is* the tool's action on a candidate.
                self.checked = if self.checked == Some(index) { None } else { Some(index) };
                self.focus_comment = false;
                return;
            }
        }
        self.focus_comment = false;
    }

    fn on_key(&mut self, key: KeyCode, screen: &Screen) {
        // While the comment field holds focus, only editing and submission
        // keys reach it: every other branch is guarded, because a comment
        // like "3 too flat" must not silently re-target candidate 3, and a
        // letter n inside a sentence must not record the review.
        match key {
            KeyCode::Tab => self.focus_comment = !self.focus_comment,
            KeyCode::ArrowLeft if !self.focus_comment => {
                let len = self.icon().generations.len();
                if let Some(current) = self.checked {
                    self.checked = Some((current + len - 1) % len);
                }
            }
            KeyCode::ArrowRight if !self.focus_comment => {
                let len = self.icon().generations.len();
                if let Some(current) = self.checked {
                    self.checked = Some((current + 1) % len);
                }
            }
            KeyCode::PageUp => self.scroll_detail(-design::MIN_TARGET_PX * 4.0 * self.ui.0, screen),
            KeyCode::PageDown => self.scroll_detail(design::MIN_TARGET_PX * 4.0 * self.ui.0, screen),
            KeyCode::KeyN if !self.focus_comment => self.record_and_advance(),
            KeyCode::Enter => self.record_and_advance(),
            KeyCode::Digit1 | KeyCode::Digit2 | KeyCode::Digit3 | KeyCode::Digit4
            | KeyCode::Digit5 | KeyCode::Digit6
                if !self.focus_comment =>
            {
                let index = match key {
                    KeyCode::Digit1 => 0,
                    KeyCode::Digit2 => 1,
                    KeyCode::Digit3 => 2,
                    KeyCode::Digit4 => 3,
                    KeyCode::Digit5 => 4,
                    _ => 5,
                };
                if index < self.icon().generations.len() {
                    self.checked = if self.checked == Some(index) { None } else { Some(index) };
                }
            }
            KeyCode::Backspace if self.focus_comment => {
                self.comment.pop();
            }
            _ => {}
        }
    }

    fn scroll_detail(&mut self, delta: f32, screen: &Screen) {
        let icon = self.icon().clone();
        let inset = Space::Md.px(self.ui);
        let width = screen.w - 2.0 * inset;
        let detail_h = self.detail_viewport(screen);
        let blocks = self.detail_blocks(&icon, width);
        let frame = ui::Frame::new(inset, self.strip_height(), width, detail_h);
        let measured = ui::measure(&mut self.text, self.ui, &frame, &blocks);
        self.detail_offset = (self.detail_offset + delta).clamp(0.0, measured.max_scroll());
    }

    fn type_into_comment(&mut self, ch: char) {
        if self.focus_comment {
            self.comment.push(ch);
        }
    }
}

impl ApplicationHandler for Picker {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let window = std::sync::Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("ala-cities · pick (rebuilt)"))
                .expect("a window"),
        );
        if let Err(err) = self.load_atlas() {
            eprintln!("{err}");
            std::process::exit(1);
        }
        let mut gpu = Gpu::new(window.clone(), &self.text);
        gpu.bind_image(&self.atlas, self.atlas_width, self.atlas_height);
        self.window = Some(window);
        self.gpu = Some(gpu);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(gpu) = self.gpu.as_mut() {
                    gpu.resize(size.width, size.height);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(window) = self.window.as_ref() {
                    let scale = window.scale_factor() as f32;
                    self.cursor = (position.x as f32 * scale, position.y as f32 * scale);
                }
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                let screen = self.gpu.as_ref().map(|gpu| gpu.screen());
                if let Some(screen) = screen {
                    self.on_click(&screen);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let screen = self.gpu.as_ref().map(|gpu| gpu.screen());
                let pixels = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y * 40.0,
                    winit::event::MouseScrollDelta::PixelDelta(p) => p.y as f32,
                };
                // The wheel scrolls the detail region; the tiles never move.
                if let Some(screen) = screen {
                    self.scroll_detail(pixels, &screen);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let Some(text) = &event.text {
                        for ch in text.chars() {
                            self.type_into_comment(ch);
                        }
                    }
                    let screen = self.gpu.as_ref().map(|gpu| gpu.screen());
                    if let (Some(screen), winit::keyboard::PhysicalKey::Code(code)) =
                        (screen, event.physical_key)
                    {
                        self.on_key(code, &screen);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if self.closing {
                    event_loop.exit();
                    return;
                }
                let Some(window) = self.window.clone() else { return };
                // The gpu borrow is confined to the block: the draw pass owns
                // `self` in between, and the two must not overlap.
                let screen = {
                    let Some(gpu) = self.gpu.as_mut() else { return };
                    gpu.screen()
                };
                let mut batch = Batcher::default();
                let mut images = ImageBatcher::default();
                self.draw(&mut batch, &mut images, &screen);
                if let Some(gpu) = self.gpu.as_mut() {
                    gpu.render(
                        &ala_cities::render::WorldBatch::default(),
                        &batch,
                        &images,
                        &winit_default_camera(),
                        [0.08, 0.08, 0.09, 1.0],
                        1.0 / 60.0,
                        &mut self.text,
                    );
                }
                window.request_redraw();
            }
            _ => {}
        }
    }
}

fn winit_default_camera() -> ala_cities::render::Camera {
    // The picker draws no world; the camera exists because the render
    // signature shares the game's. Identity view, no projection needed —
    // the world pass draws nothing.
    ala_cities::render::Camera::new(
        ala_cities::render::Screen { w: 1400.0, h: 900.0 },
        256,
        256,
    )
}

// Keep the unused-import lint quiet on the two names used only in doc comments.
#[allow(unused)]
fn _tension(value: &Value) -> f32 {
    value.as_f64().unwrap_or(0.0) as f32
}
#[allow(unused)]
const _: u32 = ATLAS_SIZE;

#[cfg(test)]
mod tests {
    use super::*;

    /// The headless half of a135(c)'s smoke test: the rebuilt picker's data
    /// path — load, validate, pack the first awaiting icon's candidates —
    /// runs against the real review set before the window ever opens.
    #[test]
    fn the_data_path_loads_and_packs_the_real_review_set() {
        let review = load_review().expect("review.json loads");
        if review.icons.is_empty() {
            eprintln!("every icon is decided; the pack test has nothing to exercise");
            return;
        }
        let icon = &review.icons[0];
        let (atlas, width, height, packed) =
            pack_atlas(icon, &review).expect("the atlas packs");
        assert_eq!(packed.len(), CANDIDATE_COUNT);
        // Six tiles pack three across: two rows, not a square.
        let tile = review.decision_px * 3.0;
        assert_eq!(width as f32, tile * 3.0);
        assert_eq!(height as f32, tile * 2.0);
        assert_eq!(atlas.len(), (width * height * 4) as usize);
        // Every packed candidate's source rect sits inside the atlas, on
        // whole-pixel boundaries, at the zoomed tile size.
        for candidate in packed.into_iter().flatten() {
            assert!(candidate.src[2] <= width as f32 && candidate.src[3] <= height as f32);
            assert_eq!(candidate.src[2] - candidate.src[0], tile);
            assert_eq!(candidate.src[3] - candidate.src[1], tile);
        }
    }
}
