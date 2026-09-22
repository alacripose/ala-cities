//! The review picker: six explicit pilot concepts of one icon, and what to change next.
//!
//! Run it after the icon pipeline has rendered a review set:
//!
//!     cargo run --release --bin pick
//!
//! What it shows is the **same icon drawn six authored ways in a 2x3 grid** — the
//! declared candidates from `tools/icons/shapes.py`: one canonical and five
//! explicit alternate recipes. Each candidate is drawn at the largest shipped
//! size, whole-number scaled so no filter invents pixels, with 32px recognition
//! and the *smallest* shipped size beside it,
//! because a choice that dies at 24 px should be seen to die while it is being made.
//! Every candidate sits on a surface the icon **declares** — painted in that
//! surface's own token colour, read out of `review.json` rather than retyped here.
//!
//! Two controls, and both are the point of the tool:
//!
//! * a **checkbox per candidate** — check one, press Enter, and that candidate
//!   becomes the icon's target: the one the pipeline promotes into the shipping
//!   set. Nothing ships that a person has not checked. A checkbox, not a radio
//!   button, because *none of these* has to be reachable;
//! * a **comment box for the next generation** — what is wrong, what to change,
//!   what to try next. A comment is recorded whether or not anything is checked,
//!   so "none of these, make the teeth longer" is a usable answer, and the next
//!   rendering pass is authored against it.
//!
//! This is deliberately *not* the game: no simulation, no record, no governance.
//! It writes one thing — an append-only line in `assets/icons/review-decisions.jsonl`
//! — and deciding again supersedes the earlier choice without erasing it, which is
//! the same rule the game's own record uses.
//!
//! The interface is an **extension of the design framework**, not a separate
//! dialect: the same tokens, the same type steps, the same spacing scale, the same
//! 48 px target floor, and the same fail-closed check at startup. Its own controls
//! are registered as targets and go through `design::verify` like everything else.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde_json::Value;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};
use wgpu::util::DeviceExt;

// The framework, included as source rather than as a crate: the picker draws with
// the game's own tokens, scales and text, and a shared crate would be a second
// place for those to be defined. Not every function is used here, and that is the
// point of a shared framework.
#[path = "../design.rs"]
#[allow(dead_code)]
mod design;
#[path = "../hud.rs"]
#[allow(dead_code)]
mod hud;
#[path = "../render.rs"]
#[allow(dead_code)]
mod render;

use design::{Space, Step, UiScale};
use hud::Token;
use render::{Batcher, Face, Screen, Text};

/// Where the pipeline's review set and its records live.
const REVIEW_JSON: &str = "assets/icons/review.json";
const DECISIONS: &str = "assets/icons/review-decisions.jsonl";
const ASSETS: &str = "assets/icons";
const AUTHORING_PHASE: &str = "icon-authoring-review";
const CANDIDATE_COUNT: usize = 6;

/// Whole pixels of window per pixel of icon, so the window shows the rendered
/// pixels and not a resampling of them.
const DECISION_ZOOM: f32 = 3.0;
const CONTEXT_ZOOM: f32 = 3.0;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let review = match load_review() {
        Ok(review) => review,
        Err(err) => {
            eprintln!("{err}");
            eprintln!(
                "render the review set first:\n  blender --background --factory-startup \
                 --python tools/icons/generate.py -- --review stage-1"
            );
            std::process::exit(1);
        }
    };

    if args.iter().any(|arg| arg == "--selftest") {
        selftest(&review);
        return;
    }

    if review.icons.is_empty() {
        println!("nothing to review: every icon in the inventory has a decided target.");
        return;
    }

    let review_defects = review_defects(&review);
    if !review_defects.is_empty() {
        eprintln!("the icon-authoring review set is invalid:");
        for defect in &review_defects {
            eprintln!("  - {defect}");
        }
        std::process::exit(1);
    }

    // The gate first, on this tool's own controls: a review surface that ignores
    // the floor it asks the game to keep is not an extension of the framework.
    let targets = controls();
    for line in design::audit(&targets, UiScale::default()) {
        println!("picker design: {line}");
    }
    let mut defects = design::verify(&targets, UiScale::default());
    // And the check the design gate cannot make, because it holds no font: that the
    // type scale renders at the sizes it declares. The last build passed every
    // design check while drawing every step 25 % small.
    let probe = render::Text::new();
    defects.extend(probe.scale_defects());
    if !defects.is_empty() {
        eprintln!("the picker's own design check failed closed:");
        for defect in &defects {
            eprintln!("  - {defect}");
        }
        std::process::exit(1);
    }

    let mut app = Picker::new(review);
    let event_loop = EventLoop::new().expect("an event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).expect("the event loop ran");
    app.report();
}

/// The picker's own interactive targets, in the design framework's terms. These go
/// through the same `design::verify` the game does, at startup, and the tool
/// refuses to open if its own controls are below the floor.
fn controls() -> Vec<design::Target> {
    vec![
        design::Target {
            name: "candidate tile",
            w: 288.0,
            h: 288.0,
        },
        design::Target {
            name: "generation checkbox",
            w: 288.0,
            h: 48.0,
        },
        design::Target {
            name: "comment box",
            w: 900.0,
            h: 48.0,
        },
    ]
}

// ---------------------------------------------------------------------------
// What the pipeline wrote
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct Review {
    concept_set: String,
    decision_px: f32,
    recognition_px: f32,
    context_px: f32,
    fills: HashMap<String, [f32; 4]>,
    host_fills: HashMap<String, [f32; 4]>,
    blender: String,
    palette_hash: String,
    icons: Vec<Icon>,
}

#[derive(Clone, Debug)]
struct Icon {
    id: String,
    kind: String,
    meaning: String,
    locates: String,
    sits_on: Vec<String>,
    identity: Option<String>,
    identity_as: String,
    /// Comments the pipeline already carries forward for this icon.
    directives: Vec<String>,
    generations: Vec<Generation>,
    brief: Value,
    lineage: Vec<String>,
    forbidden_readings: Vec<String>,
}

#[derive(Debug, Clone)]
struct Generation {
    id: String,
    label: String,
    why: String,
    sharp: PathBuf,
    recognition: PathBuf,
    context: PathBuf,
    measurements: Value,
    checks: Value,
    gate: Value,
    brief: Value,
}

fn load_review() -> Result<Review, String> {
    let text = std::fs::read_to_string(REVIEW_JSON)
        .map_err(|err| format!("could not read {REVIEW_JSON}: {err}"))?;
    let root: Value = serde_json::from_str(&text)
        .map_err(|err| format!("{REVIEW_JSON} is not valid JSON: {err}"))?;

    let number = |value: &Value, key: &str| value.get(key).and_then(Value::as_f64).unwrap_or(0.0) as f32;
    let colour_map = |key: &str| {
        let mut out = HashMap::new();
        if let Some(map) = root.get(key).and_then(Value::as_object) {
            for (name, value) in map {
                out.insert(name.clone(), rgba(value));
            }
        }
        out
    };

    // Only icons still awaiting a decision are put in front of a person:
    // re-deciding one is deliberate, not the default.
    let awaiting: Vec<String> = root
        .get("awaiting_decision")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();

    let mut icons = Vec::new();
    for entry in root.get("icons").and_then(Value::as_array).unwrap_or(&Vec::new()) {
        let id = string(entry, "id");
        if !awaiting.contains(&id) {
            continue;
        }
        let directives = entry
            .get("directives")
            .and_then(Value::as_array)
            .map(|list| {
                list.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        let mut generations = Vec::new();
        for generation in entry
            .get("generations")
            .and_then(Value::as_array)
            .unwrap_or(&Vec::new())
        {
            let files = generation.get("files").cloned().unwrap_or(Value::Null);
            generations.push(Generation {
                id: string(generation, "id"),
                label: string(generation, "label"),
                why: string(generation, "why"),
                sharp: Path::new(ASSETS).join(string(&files, "raw")),
                recognition: Path::new(ASSETS).join(string(&files, "recognition_raw")),
                context: Path::new(ASSETS).join(string(&files, "context_raw")),
                measurements: generation.get("measurements").cloned().unwrap_or(Value::Null),
                checks: generation.get("checks").cloned().unwrap_or(Value::Null),
                gate: generation.get("selection_notes").cloned().unwrap_or(Value::Array(Vec::new())),
                brief: generation.get("brief").cloned().unwrap_or(Value::Null),
            });
        }
        if generations.is_empty() {
            continue;
        }
        icons.push(Icon {
            id,
            kind: string(entry, "kind"),
            meaning: string(entry, "meaning"),
            locates: string(entry, "locates"),
            sits_on: entry
                .get("sits_on")
                .and_then(Value::as_array)
                .map(|list| {
                    list.iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect()
                })
                .unwrap_or_default(),
            identity: entry.get("identity").and_then(Value::as_str).map(str::to_string),
            identity_as: string(entry, "identity_as"),
            brief: entry.get("brief").cloned().unwrap_or(Value::Null),
            lineage: strings(entry.get("lineage")),
            forbidden_readings: strings(entry.get("forbidden_readings")),
            directives,
            generations,
        });
    }

    let blender = root.get("blender").cloned().unwrap_or(Value::Null);
    let palette_hash = string(&root, "palette_hash");
    Ok(Review {
        concept_set: string(&root, "concept_set"),
        decision_px: number(&root, "decision_size_px"),
        recognition_px: number(&root, "recognition_size_px"),
        context_px: number(&root, "context_size_px"),
        fills: colour_map("fills"),
        host_fills: colour_map("host_fills"),
        blender: format!(
            "Blender {} ({})",
            string(&blender, "version"),
            string(&blender, "build_hash")
        ),
        palette_hash,
        icons,
    })
}

fn string(value: &Value, key: &str) -> String {
    value.get(key).and_then(Value::as_str).unwrap_or_default().to_string()
}

fn strings(value: Option<&Value>) -> Vec<String> {
    value.and_then(Value::as_array)
        .map(|list| list.iter().filter_map(Value::as_str).map(str::to_string).collect())
        .unwrap_or_default()
}

fn rgba(value: &Value) -> [f32; 4] {
    let channel = |index: usize| value.get(index).and_then(Value::as_f64).unwrap_or(0.0) as f32;
    [channel(0), channel(1), channel(2), 1.0]
}

/// Validate the authoring hand-off before opening a window. A malformed manifest
/// must refuse here instead of turning into a missing texture, an out-of-bounds
/// generation, or a misleading blank candidate in the review phase.
fn review_defects(review: &Review) -> Vec<String> {
    let mut defects = Vec::new();
    if review.concept_set.is_empty() {
        defects.push("review.json has no concept_set; refusing unversioned targets".to_string());
    }
    if review.decision_px <= 0.0 || review.recognition_px <= 0.0 || review.context_px <= 0.0 {
        defects.push("decision, recognition, and context sizes must be positive".to_string());
    }
    for icon in &review.icons {
        if icon.generations.len() != CANDIDATE_COUNT {
            defects.push(format!(
                "icon `{}` declares {} candidates; the pilot review requires exactly {}",
                icon.id, icon.generations.len(), CANDIDATE_COUNT
            ));
        }
        if icon.brief.is_null() || icon.lineage.is_empty() || icon.forbidden_readings.is_empty() {
            defects.push(format!("icon `{}` is missing its full brief, lineage, or forbidden readings", icon.id));
        }
        if icon.sits_on.is_empty() {
            defects.push(format!("icon `{}` declares no host surface", icon.id));
        }
        for generation in &icon.generations {
            for (label, path, expected) in [
                ("decision", &generation.sharp, review.decision_px),
                ("recognition", &generation.recognition, review.recognition_px),
                ("context", &generation.context, review.context_px),
            ] {
                let expected_bytes = (expected as usize)
                    .saturating_mul(expected as usize)
                    .saturating_mul(4);
                match std::fs::metadata(path) {
                    Ok(metadata) if metadata.len() == expected_bytes as u64 => {}
                    Ok(metadata) => defects.push(format!(
                        "{} `{}` for {} is {} bytes; expected {}x{} RGBA8 ({})",
                        label,
                        path.display(),
                        icon.id,
                        metadata.len(),
                        expected,
                        expected,
                        expected_bytes
                    )),
                    Err(err) => defects.push(format!(
                        "{} `{}` for {} cannot be read: {err}",
                        label,
                        path.display(),
                        icon.id
                    )),
                }
            }
        }
    }
    defects
}

impl Review {
    fn fill(&self, name: &str) -> [f32; 4] {
        self.fills.get(name).copied().unwrap_or([0.1, 0.1, 0.12, 1.0])
    }

    /// The surface the icon declares first. Painting every candidate on the same
    /// surface is what makes the three comparable; painting it on *its own* declared
    /// surface is what makes the contrast claim checkable by eye.
    fn host_fill(&self, icon: &Icon) -> [f32; 4] {
        let host = icon
            .sits_on
            .first()
            .cloned()
            .unwrap_or_else(|| "PanelRaised".to_string());
        self.host_fills
            .get(&host)
            .copied()
            .unwrap_or_else(|| self.fill("PanelRaised"))
    }

    fn header(&self) -> String {
        let short = &self.palette_hash[..8.min(self.palette_hash.len())];
        format!(
            "\n{} icon(s) awaiting a decision · six candidates · decision size {} px · context {} px\n\
             rendered by {} · palette {short}\n\n\
             concept set: {} · phase: {AUTHORING_PHASE}\n\
             check a candidate and press Enter to make it the target · type in the \
             comment box to say what the next generation should change\n\
             a comment is recorded with or without a target, and only a checked \
             generation is promoted into the shipping set\n\
             every decision is appended to {DECISIONS}\n",
            self.icons.len(),
            self.decision_px,
            self.context_px,
            self.concept_set,
            self.blender,
        )
    }
}

impl Generation {
    /// The measured facts in one line: contrast on each declared surface, what the
    /// glyph covers at each shipped size, and how the accent reads against the ink.
    fn facts(&self) -> String {
        let mut parts = Vec::new();
        if let Some(contrasts) = self.checks.get("contrasts").and_then(Value::as_object) {
            for (host, values) in contrasts {
                let ink = values.get("ink").and_then(Value::as_f64).unwrap_or_default();
                parts.push(format!("{host} {ink:.2}:1"));
            }
        }
        let coverage = |px: &str| {
            self.measurements
                .get(px)
                .and_then(|value| value.get("coverage"))
                .and_then(Value::as_f64)
                .unwrap_or_default()
        };
        if let Some(role) = self.brief.get("candidate_role").and_then(Value::as_str) {
            parts.push(format!("role {role}"));
        }
        parts.push(format!(
            "cover 24 {:.2} / {} {:.2}",
            coverage("24"),
            self.measurements
                .get("96")
                .map(|_| "96")
                .unwrap_or("96"),
            coverage("96")
        ));
        if let Some(internal) = self.checks.get("internal_contrast").and_then(Value::as_f64) {
            parts.push(format!("accent {internal:.2}:1"));
        }
        parts.join(" · ")
    }

    fn notes(&self) -> Vec<String> {
        self.checks
            .get("notes")
            .and_then(Value::as_array)
            .map(|list| {
                list.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// The decisions log
// ---------------------------------------------------------------------------

/// One line, appended. A **mark** carries a target and the checked generation; a
/// **note** carries only a comment, which is how "none of these, change this" gets
/// recorded. The pipeline reads the last `target: true` per icon, so deciding again
/// supersedes without erasing — and every comment becomes a directive the next
/// generation is authored against.
fn record(
    review: &Review,
    icon: &Icon,
    position: usize,
    target: bool,
    comment: &str,
    path: &Path,
) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let generation = icon.generations.get(position);
    let shaped_comment = if comment.trim().is_empty() {
        Value::Null
    } else {
        Value::String(comment.trim().to_string())
    };
    let line = serde_json::json!({
        "icon": icon.id,
        "meaning": icon.meaning,
        "locates": icon.locates,
        "sits_on": icon.sits_on,
        "generation": generation.map(|g| g.id.clone()),
        "generation_label": generation.map(|g| g.label.clone()),
        "generation_why": generation.map(|g| g.why.clone()),
        "target": target,
        "comment": shaped_comment,
        "at_unix_seconds": now,        "by": "ala-cities pick",
        "phase": AUTHORING_PHASE,
        "concept_set": review.concept_set,
        "concept_set": review.concept_set,
            "concept_set": review.concept_set,
            "build": {
            "blender": review.blender,
            "palette_hash": review.palette_hash,
            "decision_size_px": review.decision_px,
        },
        "measured": generation.map(|g| g.measurements.clone()).unwrap_or(Value::Null),
        "checks": generation.map(|g| g.checks.clone()).unwrap_or(Value::Null),
    });
    let mut line = serde_json::to_string(&line).unwrap_or_default();
    line.push('\n');

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match std::fs::OpenOptions::new().create(true).append(true).open(path) {
        Ok(mut file) => {
            use std::io::Write;
            if let Err(err) = file.write_all(line.as_bytes()) {
                eprintln!("could not append the decision: {err}");
            }
        }
        Err(err) => eprintln!("could not open {}: {err}", path.display()),
    }
    line
}

/// Every comment recorded for an icon, oldest first, from the picker's own log — so
/// the comments shown next to an icon include the newest ones even before the
/// pipeline has re-read them.
fn read_directives(path: &Path, concept_set: &str) -> HashMap<String, Vec<String>> {
    let mut out: HashMap<String, Vec<String>> = HashMap::new();
    let Ok(text) = std::fs::read_to_string(path) else {
        return out;
    };
    for line in text.lines() {
        let Ok(record) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let Some(icon) = record.get("icon").and_then(Value::as_str) else {
            continue;
        };
        if record.get("concept_set").and_then(Value::as_str) != Some(concept_set) {
            continue;
        }
        if let Some(comment) = record.get("comment").and_then(Value::as_str) {
            out.entry(icon.to_string()).or_default().push(comment.to_string());
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Layout: computed once per frame, from the scales
// ---------------------------------------------------------------------------

/// Everything the frame needs to place, in device pixels. Built from the design
/// scales so the picker cannot drift into its own metrics.
struct Layout {
    margin: f32,
    header: f32,
    title_h: f32,
    /// The height of a `Step::Small` line, used to place the caption under each
    /// context plate.
    small_h: f32,
    body_h: f32,
    tile: f32,
    context: f32,
    gap: f32,
    left: f32,
    tile_top: f32,
    checkbox_top: f32,
    checkbox_h: f32,
    checkbox_box: f32,
    context_top: f32,
    comment_top: f32,
    comment_h: f32,
    hint_top: f32,
    comment_w: f32,
}

impl Layout {
    fn new(review: &Review, width: f32, ui: UiScale) -> Self {
        let margin = Space::Xl.px(ui);
        let title_h = Step::Title.px(ui) as f32;
        let small_h = Step::Small.px(ui) as f32;
        let body_h = Step::Body.px(ui) as f32;
        let header = Space::Sm.px(ui) + title_h + Space::Xs.px(ui) + small_h + Space::Sm.px(ui);
        let tile = (review.decision_px * DECISION_ZOOM).round();
        let context = (review.context_px * CONTEXT_ZOOM).round();
        let gap = Space::Xl.px(ui);
        let columns = 3.0_f32.min(review.icons.first().map(|i| i.generations.len()).unwrap_or(CANDIDATE_COUNT) as f32);
        let row_width = columns * tile + (columns - 1.0) * gap;
        let left = ((width - row_width) / 2.0).max(margin);
        let tile_top = margin + header + Space::Lg.px(ui);
        let checkbox_top = tile_top + tile + Space::Sm.px(ui);
        let checkbox_h = design::MIN_TARGET_PX * ui.0;
        let context_top = checkbox_top + checkbox_h + Space::Xl.px(ui);
        let row_step = tile + Space::Sm.px(ui) + checkbox_h + Space::Xl.px(ui)
            + context + Space::Xl.px(ui) + small_h + Space::Lg.px(ui);
        let comment_top = context_top + context + Space::Xl.px(ui) + small_h + Space::Md.px(ui)
            + row_step;
        let comment_h = design::MIN_TARGET_PX * ui.0;
        let hint_top = comment_top + comment_h + Space::Sm.px(ui);
        Self {
            margin,
            header,
            title_h,
            small_h,
            body_h,
            tile,
            context,
            gap,
            left,
            tile_top,
            checkbox_top,
            checkbox_h,
            checkbox_box: 24.0 * ui.0,
            context_top,
            comment_top,
            comment_h,
            hint_top,
            comment_w: (width - margin * 2.0).max(320.0),
        }
    }

    fn column_x(&self, position: usize) -> f32 {
        self.left + (position % 3) as f32 * (self.tile + self.gap)
    }

    fn row_offset(&self, position: usize) -> f32 {
        (position / 3) as f32 * (self.comment_top - self.context_top + self.context + Space::Xl.px(UiScale::default()))
    }

    fn tile_y(&self, position: usize) -> f32 {
        self.tile_top + self.row_offset(position)
    }

    fn checkbox_y(&self, position: usize) -> f32 {
        self.checkbox_top + self.row_offset(position)
    }

    fn context_y(&self, position: usize) -> f32 {
        self.context_top + self.row_offset(position)
    }

    fn context_x(&self, position: usize) -> f32 {
        self.column_x(position) + (self.tile - self.context) / 2.0
    }

    fn comment(&self) -> (f32, f32) {
        (self.margin, self.comment_top)
    }
}

// ---------------------------------------------------------------------------
// The picker
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    None,
    Comment,
}

struct Picker {
    review: Review,
    window: Option<Arc<Window>>,
    gpu: Option<Gpu>,
    ui: UiScale,
    text: Text,
    icon_index: usize,
    /// The checked concept, if any. A checkbox, so it can be unchecked: "none of
    /// these" has to be reachable without leaving the tool.
    checked: Option<usize>,
    comment: String,
    focus: Focus,
    cursor: (f32, f32),
    hover: Option<usize>,
    /// Comments read back out of the decisions log, merged with the ones the
    /// pipeline already carries, so the newest feedback is visible immediately.
    directives: HashMap<String, Vec<String>>,
    started: Instant,
    last: Option<String>,
    decided_here: Vec<String>,
    fonts_ok: bool,
}

impl Picker {
    fn new(review: Review) -> Self {
        let mut text = Text::new();
        text.set_ui_scale(UiScale::default());
        let fonts_ok = !text.missing_font;
        let directives = read_directives(Path::new(DECISIONS), &review.concept_set);
        Self {
            review,
            window: None,
            gpu: None,
            ui: UiScale::default(),
            text,
            icon_index: 0,
            checked: None,
            comment: String::new(),
            focus: Focus::None,
            cursor: (0.0, 0.0),
            hover: None,
            directives,
            started: Instant::now(),
            last: None,
            decided_here: Vec::new(),
            fonts_ok,
        }
    }

    fn icon(&self) -> &Icon {
        &self.review.icons[self.icon_index]
    }

    fn layout(&self) -> Layout {
        let width = match self.gpu.as_ref() {
            Some(gpu) => gpu.screen.w,
            None => 1400.0,
        };
        Layout::new(&self.review, width, self.ui)
    }

    /// Both sources, oldest first, deduplicated: what was asked for last time.
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

    /// Record what the tool currently holds: a target if something is checked, and a
    /// directive if anything was typed. Then move to the next icon.
    fn record_and_advance(&mut self) {
        let icon = self.icon().clone();
        let comment = self.comment.trim().to_string();
        let checked = self.checked;
        if checked.is_none() && comment.is_empty() {
            println!(            "nothing to record on {}: check a concept or type a comment", icon.id);
            return;
        }
        let position = checked.unwrap_or(0).min(icon.generations.len().saturating_sub(1));
        let line = record(
            &self.review,
            &icon,
            position,
            checked.is_some(),
            &comment,
            Path::new(DECISIONS),
        );
        let what = match checked {
            Some(index) => format!(
                "target = {} ({})",
                icon.generations[index].label, icon.generations[index].id
            ),
            None => "no target, comment only".to_string(),
        };
        println!(
            "\n✔ recorded {} → {what}{}\n    {}",
            icon.id,
            if comment.is_empty() {
                String::new()
            } else {
                format!(" · directive: “{comment}”")
            },
            line.trim()
        );
        self.last = Some(format!("{} → {what}", icon.id));
        self.decided_here.push(icon.id.clone());
        if !comment.is_empty() {
            self.directives
                .entry(icon.id.clone())
                .or_default()
                .push(comment);
        }
        self.comment.clear();
        self.checked = None;
        self.focus = Focus::None;
        self.hover = None;
        if self.icon_index + 1 < self.review.icons.len() {
            self.icon_index += 1;
        }
    }

    fn step(&mut self, delta: isize) {
        let count = self.review.icons.len();
        if count == 0 {
            return;
        }
        self.icon_index = (self.icon_index as isize + delta).rem_euclid(count as isize) as usize;
        self.checked = None;
        self.comment.clear();
        self.focus = Focus::None;
        self.hover = None;
    }

    /// The checkbox: checking one unchecks the others, and checking the checked one
    /// again unchecks it, so "none of these" is always reachable.
    fn toggle(&mut self, position: usize) {
        if position >= self.icon().generations.len() {
            return;
        }
        let generation = &self.icon().generations[position];
        if generation
            .gate
            .as_array()
            .map(|notes| !notes.is_empty())
            .unwrap_or(false)
        {
            self.last = Some(format!(
                "{} · {} is not selectable until its checks pass",
                self.icon().id,
                generation.label
            ));
            return;
        }
        self.focus = Focus::None;
        self.checked = if self.checked == Some(position) {
            None
        } else {
            Some(position)
        };
    }

    fn on_key(&mut self, code: KeyCode) {
        // While the comment box has focus, digits and letters are text, not
        // shortcuts. Focus is the only thing that decides that.
        if self.focus == Focus::Comment {
            match code {
                KeyCode::Backspace => {
                    self.comment.pop();
                }
                KeyCode::Tab | KeyCode::Escape => self.focus = Focus::None,
                _ => {}
            }
            return;
        }
        match code {
            KeyCode::Digit1 | KeyCode::KeyA => self.toggle(0),
            KeyCode::Digit2 | KeyCode::KeyB => self.toggle(1),
            KeyCode::Digit3 | KeyCode::KeyC => self.toggle(2),
            KeyCode::Digit4 => self.toggle(3),
            KeyCode::Digit5 => self.toggle(4),
            KeyCode::Digit6 => self.toggle(5),
            KeyCode::Tab => self.focus = Focus::Comment,
            KeyCode::Enter => self.record_and_advance(),
            KeyCode::KeyS | KeyCode::KeyN => self.step(1),
            KeyCode::KeyP | KeyCode::ArrowLeft => self.step(-1),
            KeyCode::ArrowRight => self.step(1),
            _ => {}
        }
    }

    fn on_text(&mut self, typed: &str) {
        if self.focus != Focus::Comment {
            return;
        }
        for character in typed.chars() {
            if !character.is_control() {
                self.comment.push(character);
            }
        }
    }

    fn report(&self) {
        if let Some(last) = &self.last {
            println!("\nlast decision: {last}");
        }
        if !self.decided_here.is_empty() {
            println!("decided in this session: {}", self.decided_here.join(", "));
        }
        println!("decisions are in {DECISIONS}");
    }

    /// The table, printed where a person can read it while looking at the window.
    /// The window carries the form; the terminal carries the numbers.
    fn print_icon(&self) {
        let icon = self.icon();
        println!(
            "\n─── [{}/{}] {} — {} ({})\n    locates: {} · sits on: {} · identity: {} ({})",
            self.icon_index + 1,
            self.review.icons.len(),
            icon.id,
            icon.meaning,
            icon.kind,
            icon.locates,
            icon.sits_on.join(", "),
            icon.identity.clone().unwrap_or_else(|| "—".into()),
            icon.identity_as,
        );
        println!("    lineage: {} · forbidden readings: {}", icon.lineage.join(" / "), icon.forbidden_readings.join(" / "));
        if let Some(cues) = icon.brief.get("semantic_cues").and_then(Value::as_array) {
            println!("    doctrine chips: Meaning=present · Silhouette={} cue(s) · Lineage={} · Material={} · Authority=guarded · Motion=metadata · Fallback=present",
                cues.len(), icon.lineage.len(), icon.brief.get("material_family").and_then(Value::as_str).unwrap_or("declared"));
        }
        for (position, generation) in icon.generations.iter().enumerate() {
            let blocked = icon.generations[position]
                .gate
                .as_array()
                .map(|notes| !notes.is_empty())
                .unwrap_or(false);
            let mark = if blocked { "⊘" } else if Some(position) == self.checked { "☑" } else { "☐" };
            let highlight = if Some(position) == self.hover { "▶" } else { " " };
            println!(
                " {highlight}{mark} {}. {} — {}\n      {}\n      {}",
                position + 1,
                generation.label,
                generation.id,
                generation.why,
                generation.facts(),
            );
            for note in generation.notes() {
                println!("      ! {note}");
            }
        }
        for directive in self.directives_for(icon) {
            println!("    ← asked for last time: {directive}");
        }
    }
}

impl ApplicationHandler for Picker {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attributes = Window::default_attributes()
            .with_title("ala-cities — icon generations")
            .with_inner_size(winit::dpi::LogicalSize::new(1400.0, 900.0));
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("a window for the picker"),
        );
        match Gpu::new(window.clone(), &self.review) {
            Ok(gpu) => self.gpu = Some(gpu),
            Err(err) => {
                eprintln!("could not start the picker's renderer: {err}");
                event_loop.exit();
                return;
            }
        }
        self.window = Some(window);
        if !self.fonts_ok {
            println!(
                "note: the text face did not load — the labels on screen will be missing, \
                 though the log format is unaffected"
            );
        }
        println!("{}", self.review.header());
        self.print_icon();
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => _event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(gpu) = self.gpu.as_mut() {
                    gpu.resize(size.width, size.height);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = (position.x as f32, position.y as f32);
                let layout = self.layout();
                self.hover = hit_checkbox(self.icon(), &layout, self.cursor)
                    .or_else(|| hit_tile(self.icon(), &layout, self.cursor));
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                if state == ElementState::Pressed {
                    let layout = self.layout();
                    let icon = self.icon().clone();
                    let cursor = self.cursor;
                    if let Some(position) = hit_checkbox(&icon, &layout, cursor) {
                        self.toggle(position);
                    } else if hit_comment(&layout, cursor) {
                        self.focus = Focus::Comment;
                    } else if let Some(position) = hit_tile(&icon, &layout, cursor) {
                        self.toggle(position);
                    }
                    self.print_icon();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state != ElementState::Pressed {
                    return;
                }
                if let PhysicalKey::Code(code) = event.physical_key {
                    if code == KeyCode::Escape && self.focus != Focus::Comment {
                        _event_loop.exit();
                        return;
                    }
                    if code == KeyCode::KeyQ && self.focus != Focus::Comment {
                        _event_loop.exit();
                        return;
                    }
                    if code == KeyCode::Enter {
                        // Enter records from either focus: the comment travels with it.
                        self.record_and_advance();
                        self.print_icon();
                        return;
                    }
                    self.on_key(code);
                    self.print_icon();
                }
                if let Some(typed) = event.text.as_ref() {
                    let typed = typed.to_string();
                    self.on_text(&typed);
                }
            }
            WindowEvent::RedrawRequested => {
                let (icon_index, checked, focus, hover, comment) = (
                    self.icon_index,
                    self.checked,
                    self.focus,
                    self.hover,
                    self.comment.clone(),
                );
                // The caret blinks because focus is a state: a still caret would
                // leave "is this box live?" as a question the screen cannot answer.
                let caret_on = (self.started.elapsed().as_millis() / 500).is_multiple_of(2);
                let layout = self.layout();
                if let Some(gpu) = self.gpu.as_mut() {
                    gpu.draw(
                        &self.review,
                        &mut self.text,
                        &layout,
                        self.ui,
                        icon_index,
                        checked,
                        focus == Focus::Comment,
                        hover,
                        &comment,
                        caret_on,
                    );
                }
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Hit testing
// ---------------------------------------------------------------------------

fn hit_checkbox(icon: &Icon, layout: &Layout, cursor: (f32, f32)) -> Option<usize> {
    for position in 0..icon.generations.len() {
        let x = layout.column_x(position);
        if cursor.0 >= x
            && cursor.0 <= x + layout.tile
            && cursor.1 >= layout.checkbox_y(position)
            && cursor.1 <= layout.checkbox_y(position) + layout.checkbox_h
        {
            return Some(position);
        }
    }
    None
}

fn hit_tile(icon: &Icon, layout: &Layout, cursor: (f32, f32)) -> Option<usize> {
    for position in 0..icon.generations.len() {
        let x = layout.column_x(position);
        if cursor.0 >= x
            && cursor.0 <= x + layout.tile
            && cursor.1 >= layout.tile_y(position)
            && cursor.1 <= layout.tile_y(position) + layout.tile
        {
            return Some(position);
        }
    }
    None
}

fn hit_comment(layout: &Layout, cursor: (f32, f32)) -> bool {
    let (x, y) = layout.comment();
    cursor.0 >= x
        && cursor.0 <= x + layout.comment_w
        && cursor.1 >= y
        && cursor.1 <= y + layout.comment_h
}

// ---------------------------------------------------------------------------
// Drawing
// ---------------------------------------------------------------------------

const IMAGE_SHADER: &str = r#"
struct Instance {
    @location(0) pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) colour: vec4<f32>,
    @location(3) uv: vec4<f32>,
};

struct Out {
    @builtin(position) clip: vec4<f32>,
    @location(0) colour: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) index: u32, instance: Instance) -> Out {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0)
    );
    let corner = corners[index];
    var out: Out;
    out.clip = vec4<f32>(instance.pos + corner * instance.size, 0.0, 1.0);
    out.uv = mix(instance.uv.xy, instance.uv.zw, corner);
    out.colour = instance.colour;
    return out;
}

@group(0) @binding(0) var image: texture_2d<f32>;
@group(0) @binding(1) var image_sampler: sampler;

@fragment
fn fs_main(in: Out) -> @location(0) vec4<f32> {
    let texel = textureSample(image, image_sampler, in.uv);
    return vec4<f32>(texel.rgb * in.colour.rgb, texel.a * in.colour.a);
}
"#;

const GLYPH_SHADER: &str = r#"
struct Instance {
    @location(0) pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) colour: vec4<f32>,
    @location(3) uv: vec4<f32>,
};

struct Out {
    @builtin(position) clip: vec4<f32>,
    @location(0) colour: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) index: u32, instance: Instance) -> Out {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0)
    );
    let corner = corners[index];
    var out: Out;
    out.clip = vec4<f32>(instance.pos + corner * instance.size, 0.0, 1.0);
    out.uv = mix(instance.uv.xy, instance.uv.zw, corner);
    out.colour = instance.colour;
    return out;
}

@group(0) @binding(0) var atlas: texture_2d<f32>;
@group(0) @binding(1) var atlas_sampler: sampler;

@fragment
fn fs_main(in: Out) -> @location(0) vec4<f32> {
    let coverage = textureSample(atlas, atlas_sampler, in.uv).r;
    return vec4<f32>(in.colour.rgb, in.colour.a * coverage);
}
"#;

struct Gpu {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    screen: Screen,
    glyph_pipeline: wgpu::RenderPipeline,
    image_pipeline: wgpu::RenderPipeline,
    bind_layout: wgpu::BindGroupLayout,
    vertex_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    /// The R8 coverage atlas the framework's text, panels and outlines live in.
    atlas_texture: wgpu::Texture,
    atlas_bind: wgpu::BindGroup,
    /// The atlas revision currently on the GPU.
    atlas_revision: u64,
    /// One RGBA texture per reviewed image, plus its bind group.
    bind_groups: Vec<wgpu::BindGroup>,
    image_bind: HashMap<(String, usize, u8), usize>,
    _textures: Vec<wgpu::Texture>,
    sampler: wgpu::Sampler,
    loaded_icon: Option<String>,
    /// Reported on the first frame and never again: the interface quads the batch
    /// actually holds and the icon quads actually bound to a texture. An interface
    /// that draws no images should say so rather than look fine.
    first_frame_reported: bool,
}

/// Which image of which candidate: 0 is the decision size, 1 the context size.
type ImageKey = (String, usize, u8);

impl Gpu {
    fn new(window: Arc<Window>, review: &Review) -> Result<Self, String> {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance
            .create_surface(window.clone())
            .map_err(|err| format!("no surface: {err}"))?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .map_err(|err| format!("no GPU adapter: {err}"))?;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("ala-cities pick"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::Off,
        }))
        .map_err(|err| format!("no device: {err}"))?;

        let caps = surface.get_capabilities(&adapter);
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .ok_or_else(|| "the surface has no default configuration".to_string())?;
        // A review tool has no reason to sit on a vsync queue, but it does have a
        // reason to present the same way the game does.
        config.present_mode = if caps.present_modes.contains(&wgpu::PresentMode::Mailbox) {
            wgpu::PresentMode::Mailbox
        } else if caps.present_modes.contains(&wgpu::PresentMode::AutoNoVsync) {
            wgpu::PresentMode::AutoNoVsync
        } else {
            wgpu::PresentMode::Fifo
        };
        surface.configure(&device, &config);
        let screen = Screen {
            w: config.width as f32,
            h: config.height as f32,
        };

        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("picker texture layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("picker sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            // Nearest, for the same reason the game's glyphs are nearest: these
            // pixels were rasterised at the size they are drawn at, and a filter
            // would invent the ones in between.
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("picker coverage atlas"),
            size: wgpu::Extent3d {
                width: render::ATLAS_SIZE,
                height: render::ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let atlas_bind = bind_group(&device, &bind_layout, &atlas_texture, &sampler);

        let glyph_pipeline = pipeline(&device, &bind_layout, &config, "picker glyphs", GLYPH_SHADER);
        let image_pipeline = pipeline(&device, &bind_layout, &config, "picker images", IMAGE_SHADER);

        let vertex_capacity = 4096;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("picker vertices"),
            size: (vertex_capacity * std::mem::size_of::<render::Instance>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let (width_px, height_px) = (config.width, config.height);
        let mut gpu = Self {
            surface,
            device,
            queue,
            config,
            screen,
            glyph_pipeline,
            image_pipeline,
            bind_layout,
            vertex_buffer,
            vertex_capacity,
            atlas_texture,
            atlas_bind,
            atlas_revision: u64::MAX,
            bind_groups: Vec::new(),
            image_bind: HashMap::new(),
            _textures: Vec::new(),
            sampler,
            loaded_icon: None,
            first_frame_reported: false,
        };
        if let Some(icon) = review.icons.first() {
            let id = icon.id.clone();
            gpu.ensure_icon(review, 0)?;
            gpu.loaded_icon = Some(id);
        }

        let info = adapter.get_info();
        println!(
            "picker renderer ready: {} · {:?} · {}x{}",
            info.name, info.backend, width_px, height_px
        );
        Ok(gpu)
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.screen = Screen {
            w: width as f32,
            h: height as f32,
        };
    }

    /// Bring the GPU's copy of the coverage atlas up to date.
    ///
    /// The first version of this compared `text.data.len()` against the last
    /// uploaded length — but the atlas is a fixed 1024 x 1024 buffer, so its length
    /// never changes and the texture was uploaded exactly once. Every glyph
    /// rasterised afterwards sampled empty texels: the text looked wrong, in this
    /// tool and in the game, for the same reason. `Text::revision` is the counter
    /// that cannot be forgotten.
    fn sync_atlas(&mut self, text: &Text) {
        if text.revision == self.atlas_revision {
            return;
        }
        self.upload_atlas(text);
    }

    fn upload_atlas(&mut self, text: &Text) {
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.atlas_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &text.data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(render::ATLAS_SIZE),
                rows_per_image: Some(render::ATLAS_SIZE),
            },
            wgpu::Extent3d {
                width: render::ATLAS_SIZE,
                height: render::ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
        );
        self.atlas_revision = text.revision;
    }

    /// Load both sizes of every generation of the icon now on screen. Textures for
    /// the icon that was already loaded are left alone.
    fn ensure_icon(&mut self, review: &Review, index: usize) -> Result<(), String> {
        let icon = &review.icons[index];
        if self.loaded_icon.as_deref() == Some(icon.id.as_str()) {
            return Ok(());
        }
        for (position, generation) in icon.generations.iter().enumerate() {
            for (kind, path, px) in [
                (0u8, &generation.sharp, review.decision_px),
                (1u8, &generation.recognition, review.recognition_px),
                (2u8, &generation.context, review.context_px),
            ] {
                let key: ImageKey = (icon.id.clone(), position, kind);
                if self.image_bind.contains_key(&key) {
                    continue;
                }
                let (bind, texture) =
                    load_image(&self.device, &self.queue, &self.bind_layout, &self.sampler, path, px)?;
                self.bind_groups.push(bind);
                self._textures.push(texture);
                self.image_bind.insert(key, self.bind_groups.len() - 1);
            }
        }
        self.loaded_icon = Some(icon.id.clone());
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn draw(
        &mut self,
        review: &Review,
        text: &mut Text,
        layout: &Layout,
        ui: UiScale,
        index: usize,
        checked: Option<usize>,
        comment_focused: bool,
        hover: Option<usize>,
        comment: &str,
        caret_on: bool,
    ) {
        if let Err(err) = self.ensure_icon(review, index) {
            eprintln!("{err}");
            return;
        }
        text.set_ui_scale(UiScale::default());
        let icon = &review.icons[index];
        let screen = self.screen;
        let mut batch = Batcher::default();

        let desk = review.fill("Desk");
        let ink = hud::style(Token::TextBody).text.unwrap_or([1.0; 4]);
        let muted = hud::style(Token::TextMuted).text.unwrap_or([0.8; 4]);
        let marker = hud::style(Token::Ink).fill.unwrap_or([0.2, 0.3, 0.6, 1.0]);
        let active = hud::style(Token::Nature).fill.unwrap_or([0.3, 0.7, 0.4, 1.0]);
        batch.screen_rect(&screen, 0.0, 0.0, screen.w, screen.h, desk, Text::solid_uv());

        // Header: what this is, and the facts a person needs before looking at it.
        hud::panel(
            &mut batch,
            &screen,
            layout.margin,
            layout.margin,
            screen.w - layout.margin * 2.0,
            layout.header,
            Token::Panel,
        );
        let text_x = layout.margin + Space::Sm.px(ui);
        let mut y = layout.margin + Space::Sm.px(ui);
        text.draw_step(
            Face::Body,
            &mut batch,
            &screen,
            text_x,
            y,
            Step::Title,
            ink,
            &format!(
                "[{}/{}] {} — {}",
                index + 1,
                review.icons.len(),
                icon.id,
                icon.meaning
            ),
        );
        y += layout.title_h + Space::Xs.px(ui);
        text.draw_step(
            Face::Body,
            &mut batch,
            &screen,
            text_x,
            y,
            Step::Small,
            muted,
            &format!(
                "locates {} · sits on {} · identity {} ({}) · host shown as {}",
                icon.locates,
                icon.sits_on.join(", "),
                icon.identity.clone().unwrap_or_else(|| "—".into()),
                icon.identity_as,
                icon.sits_on.first().cloned().unwrap_or_else(|| "PanelRaised".into()),
            ),
        );

        // The candidate plates, each on a surface the icon declares.
        let host = review.host_fill(icon);
        for position in 0..icon.generations.len() {
            let x = layout.column_x(position);
            batch.screen_rect(
                &screen,
                x,
                layout.tile_y(position),
                layout.tile,
                layout.tile,
                host,
                Text::solid_uv(),
            );
            if Some(position) == checked {
                batch.screen_outline(
                    &screen,
                    x - 2.0,
                    layout.tile_y(position) - 2.0,
                    layout.tile + 4.0,
                    layout.tile + 4.0,
                    active,
                );
                batch.screen_outline(
                    &screen,
                    x - 3.0,
                    layout.tile_y(position) - 3.0,
                    layout.tile + 6.0,
                    layout.tile + 6.0,
                    active,
                );
            } else if Some(position) == hover {
                batch.screen_outline(
                    &screen,
                    x - 2.0,
                    layout.tile_y(position) - 2.0,
                    layout.tile + 4.0,
                    layout.tile + 4.0,
                    marker,
                );
            }
        }

        // The checkboxes. A checkbox, not a radio button: it can be checked and
        // unchecked, because "none of these" has to be reachable.
        for position in 0..icon.generations.len() {
            let x = layout.column_x(position);
            let box_x = x + Space::Sm.px(ui);
            let box_y = layout.checkbox_y(position) + (layout.checkbox_h - layout.checkbox_box) / 2.0;
            let is_checked = Some(position) == checked;
            let blocked = icon.generations[position]
                .gate
                .as_array()
                .map(|notes| !notes.is_empty())
                .unwrap_or(false);
            batch.screen_outline(
                &screen,
                box_x,
                box_y,
                layout.checkbox_box,
                layout.checkbox_box,
                if blocked {
                    muted
                } else if is_checked {
                    active
                } else {
                    ink
                },
            );
            if is_checked {
                let inset = (layout.checkbox_box * 0.28).round();
                batch.screen_rect(
                    &screen,
                    box_x + inset,
                    box_y + inset,
                    layout.checkbox_box - inset * 2.0,
                    layout.checkbox_box - inset * 2.0,
                    active,
                    Text::solid_uv(),
                );
            }
            text.draw_step(
                Face::Body,
                &mut batch,
                &screen,
                box_x + layout.checkbox_box + Space::Sm.px(ui),
                layout.checkbox_y(position) + (layout.checkbox_h - layout.body_h) / 2.0,
                Step::Body,
                if blocked {
                    muted
                } else if is_checked {
                    ink
                } else {
                    muted
                },
                &format!(
                    "{}{} · {}",
                    if blocked { "blocked: " } else { "" },
                    icon.generations[position].label,
                    icon.generations[position].facts()
                ),
            );
        }

        // The smallest shipped size under each candidate: a choice that dies at
        // 24 px should be visible while the choice is being made.
        for position in 0..icon.generations.len() {
            batch.screen_rect(
                &screen,
                layout.context_x(position),
                layout.context_y(position),
                layout.context,
                layout.context,
                host,
                Text::solid_uv(),
            );
            text.draw_step(
                Face::Body,
                &mut batch,
                &screen,
                layout.column_x(position),
                layout.context_y(position) + layout.context + Space::Xs.px(ui),
                Step::Small,
                muted,
                &format!(
                    "{} px at {}x · caption line {} px tall",
                    review.context_px as u32,
                    CONTEXT_ZOOM as u32,
                    layout.small_h as u32
                ),
            );
        }

        // The comment box: the words that shape the next generation.
        let (comment_x, comment_y) = layout.comment();
        hud::panel(
            &mut batch,
            &screen,
            comment_x,
            comment_y,
            layout.comment_w,
            layout.comment_h,
            Token::PanelRaised,
        );
        batch.screen_outline(
            &screen,
            comment_x,
            comment_y,
            layout.comment_w,
            layout.comment_h,
            if comment_focused { active } else { muted },
        );
        let label = if comment.is_empty() && !comment_focused {
            "comment for the next generation — click here (or Tab) and type".to_string()
        } else {
            // Show the *end* of a long comment, so the caret and what was just typed
            // stay visible. Its own measurement, not a character-count guess.
            let available = layout.comment_w - Space::Md.px(ui) * 2.0;
            fit_from_end(text, comment, available)
        };
        let inner_x = comment_x + Space::Md.px(ui);
        let inner_y = comment_y + (layout.comment_h - layout.body_h) / 2.0;
        let advance = text.draw_step(
            Face::Body,
            &mut batch,
            &screen,
            inner_x,
            inner_y,
            Step::Body,
            if comment.is_empty() && !comment_focused { muted } else { ink },
            &label,
        );
        if comment_focused && caret_on {
            batch.screen_rect(
                &screen,
                inner_x + advance + 1.0,
                inner_y + 2.0,
                2.0,
                layout.body_h - 4.0,
                ink,
                Text::solid_uv(),
            );
        }

        // The hint: what Enter will do right now, in the same tokens as everything
        // else. It is a statement of state, not decoration.
        let hint = match checked {
            Some(position) => format!(
                "Enter records “{}” as the target{} · click a checked box to clear it",
                icon.generations[position].label,
                if comment.trim().is_empty() {
                    String::new()
                } else {
                    " · the comment travels with it".to_string()
                }
            ),
            None => "check a generation to mark a target — or leave every box clear and \
                     press Enter to record the comment alone"
                .to_string(),
        };
        text.draw_step(
            Face::Body,
            &mut batch,
            &screen,
            layout.margin,
            layout.hint_top,
            Step::Small,
            muted,
            &hint,
        );

        // The images, last, so nothing paints over them.
        let mut images: Vec<(usize, render::Instance)> = Vec::new();
        for position in 0..icon.generations.len() {
            for (kind, x, y, side) in [
                (0u8, layout.column_x(position), layout.tile_y(position), layout.tile),
                (
                    2u8,
                    layout.context_x(position),
                    layout.context_y(position),
                    layout.context,
                ),
            ] {
                if let Some(bind) = self.image_bind.get(&(icon.id.clone(), position, kind)).copied() {
                    let (cx, cy) = screen.to_clip(x, y);
                    let (cw, ch) = screen.size_to_clip(side, side);
                    images.push((
                        bind,
                        render::Instance {
                            pos: [cx, cy],
                            size: [cw, ch],
                            color: [1.0, 1.0, 1.0, 1.0],
                            uv: [0.0, 0.0, 1.0, 1.0],
                        },
                    ));
                }
            }
        }

        self.sync_atlas(text);
        if !self.first_frame_reported {
            self.first_frame_reported = true;
            let (slots, packed_to) = text.occupancy();
            println!(
                "picker frame: {} interface quads · {} icon quads · atlas {} bytes · 
                 {slots} glyph slots packed to row {packed_to}/{}",
                batch.instances.len(),
                images.len(),
                text.data.len(),
                render::ATLAS_SIZE,
            );
            if text.refused {
                println!(
                    "picker: the glyph atlas refused at least one glyph — text on screen is 
                     missing rather than misdrawn"
                );
            }
        }
        self.present(&batch, &images, desk);
    }

    fn present(&mut self, batch: &Batcher, images: &[(usize, render::Instance)], desk: [f32; 4]) {
        let needed = batch.instances.len() + images.len() + 8;
        if needed > self.vertex_capacity {
            self.vertex_capacity = needed + 1024;
            self.vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("picker vertices"),
                size: (self.vertex_capacity * std::mem::size_of::<render::Instance>())
                    as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        if !batch.instances.is_empty() {
            self.queue
                .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&batch.instances));
        }
        // The instance index the images start at: the buffer is read as a flat
        // instance array, so the byte offset has to land on an instance boundary.
        let base = batch.instances.len().next_multiple_of(4) as u32;
        if !images.is_empty() {
            let offset = base as u64 * std::mem::size_of::<render::Instance>() as u64;
            let instances: Vec<render::Instance> = images.iter().map(|(_, quad)| *quad).collect();
            self.queue
                .write_buffer(&self.vertex_buffer, offset, bytemuck::cast_slice(&instances));
        }

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            _ => return,
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("picker encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("picker pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: desk[0] as f64,
                            g: desk[1] as f64,
                            b: desk[2] as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if !batch.instances.is_empty() {
                pass.set_pipeline(&self.glyph_pipeline);
                pass.set_bind_group(0, &self.atlas_bind, &[]);
                pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                pass.draw(0..6, 0..batch.instances.len() as u32);
            }
            if !images.is_empty() {
                pass.set_pipeline(&self.image_pipeline);
                pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                for (position, (bind, _)) in images.iter().enumerate() {
                    pass.set_bind_group(0, &self.bind_groups[*bind], &[]);
                    let first = base + position as u32;
                    pass.draw(0..6, first..first + 1);
                }
            }
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

/// The tail of a string that fits in `width`, so a long comment shows what was just
/// typed rather than its beginning. Measured with the face that will draw it.
fn fit_from_end(text: &mut Text, value: &str, width: f32) -> String {
    if text.measure_step(Face::Body, value, Step::Body) <= width {
        return value.to_string();
    }
    let characters: Vec<char> = value.chars().collect();
    for start in 1..characters.len() {
        let candidate: String = characters[start..].iter().collect();
        if text.measure_step(Face::Body, &candidate, Step::Body) <= width {
            return format!("…{candidate}");
        }
    }
    "…".to_string()
}

fn pipeline(
    device: &wgpu::Device,
    bind_layout: &wgpu::BindGroupLayout,
    config: &wgpu::SurfaceConfiguration,
    label: &str,
    shader_source: &str,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[Some(bind_layout)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<render::Instance>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4, 3 => Float32x4],
            }],
            compilation_options: Default::default(),
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        multiview_mask: None,
        cache: None,
    })
}

fn bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    texture: &wgpu::Texture,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("picker texture bind group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

fn load_image(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    path: &Path,
    px: f32,
) -> Result<(wgpu::BindGroup, wgpu::Texture), String> {
    let side = px as u32;
    let bytes = std::fs::read(path).map_err(|err| format!("could not read {}: {err}", path.display()))?;
    let expected = (side * side * 4) as usize;
    if bytes.len() != expected {
        return Err(format!(
            "{} is {} bytes; {side}x{side} RGBA8 is {expected}. The runtime format is stated \
             in {REVIEW_JSON} — regenerate the review set rather than guessing.",
            path.display(),
            bytes.len(),
        ));
    }
    let texture = device.create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
            label: Some("picker icon"),
            size: wgpu::Extent3d {
                width: side,
                height: side,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        &bytes,
    );
    let bind = bind_group(device, layout, &texture, sampler);
    // The texture is returned with the bind group so its owner can hold it for as
    // long as the bind group lives, rather than relying on a reference inside wgpu
    // to keep it alive.
    Ok((bind, texture))
}

// ---------------------------------------------------------------------------
// The path that needs no GPU, so the log format is verifiable without a window
// ---------------------------------------------------------------------------

fn selftest(review: &Review) {
    println!("selftest: phase={AUTHORING_PHASE}; {} icon(s) awaiting a decision", review.icons.len());
    if review.icons.is_empty() {
        println!("selftest: nothing to decide, so nothing to check");
        return;
    }
    let icon = &review.icons[0];
    assert!(review_defects(review).is_empty(), "review assets must validate before authoring");

    let path = PathBuf::from("target/pick-selftest.jsonl");
    let _ = std::fs::remove_file(&path);

    // A mark: a checked generation, plus a comment for the next generation.
    let mark = record(review, icon, 1, true, "make the teeth longer", &path);
    // A note: no target at all, which is how "none of these" is recorded.
    let note = record(review, icon, 0, false, "none of these — try a squarer body", &path);

    let written = std::fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(written, format!("{mark}{note}"), "the returned lines and the log disagree");
    let lines: Vec<&str> = written.lines().collect();
    assert_eq!(lines.len(), 2, "two records were made, so two lines must exist");

    let marked: Value = serde_json::from_str(lines[0]).expect("a mark is valid JSON");
    assert_eq!(marked.get("target").and_then(Value::as_bool), Some(true));
    assert_eq!(
        marked.get("generation").and_then(Value::as_str),
        Some(icon.generations[1].id.as_str())
    );
    assert_eq!(
        marked.get("comment").and_then(Value::as_str),
        Some("make the teeth longer")
    );

    let noted: Value = serde_json::from_str(lines[1]).expect("a note is valid JSON");
    assert_eq!(noted.get("target").and_then(Value::as_bool), Some(false));
    assert_eq!(
        noted.get("comment").and_then(Value::as_str),
        Some("none of these — try a squarer body")
    );

    let directives = read_directives(&path, &review.concept_set);
    let for_icon = directives
        .get(&icon.id)
        .expect("both comments are directives for the icon");
    assert_eq!(for_icon.len(), 2, "both comments are read back, oldest first");

    for generation in &icon.generations {
        for (path, px) in [
            (&generation.sharp, review.decision_px),
            (&generation.recognition, review.recognition_px),
            (&generation.context, review.context_px),
        ] {
            let bytes = std::fs::read(path).unwrap_or_default();
            let expected = (px as usize) * (px as usize) * 4;
            assert_eq!(
                bytes.len(),
                expected,
                "{} is {} bytes, expected {expected} for {px}x{px} RGBA8",
                path.display(),
                bytes.len()
            );
        }
    }
    println!(
        "selftest: a mark and a note are both written and read back, {} directive(s) \
         recovered, {} image(s) at the declared sizes, layout targets verified at startup",
        for_icon.len(),            icon.generations.len() * 3
    );
}
