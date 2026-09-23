//! ala-cities — the client.
//!
//! The window, the input, and the interface. Every action the player takes goes
//! through the governor before it touches the world, and every action that lands
//! files a ticket first and is validated afterwards by reading the world back.
//!
//! The simulation runs at a fixed 20 Hz regardless of the display, and the
//! renderer runs uncapped so a 240 Hz panel is actually fed. Agents are drawn by
//! the *work they have completed*, never by a frame clock.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use glam::Vec3;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use ala_cities::gov::{Expectation, Government, Op, TicketKind, ALL_OPS};
use ala_cities::session::{write_feedback, Interaction, Session};
use ala_cities::sim::citizen::CitizenState;
use ala_cities::sim::{BuildingKind, Terrain, World, Zone, DAYS_PER_MONTH, SIM_HZ, TICKS_PER_DAY};

use ala_cities::audio::{self, Sound};
use ala_cities::design::{self, Space, Step, Target, UiScale};
use ala_cities::hud::{self, Token};
use ala_cities::icons::{self, IconSet};
use ala_cities::materials;
use ala_cities::render::{
    self, Batcher, Camera, Face, Gpu, ImageBatcher, Layer, Screen, Text, WorldBatch,
    LEVEL_HEIGHT, TILE,
};
use ala_cities::ui::{self, Block, Frame};

const STAGE: &str = "C1";
const MAP: u32 = 256;
const SEED: u64 = 0xC117_2026;

/// Where a session's world snapshot lives. Local and disposable, unlike the
/// season record beside it, which is the city's history and is tracked.
const WORLD_SAVE: &str = "saves/world.ron";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tool {
    Road,
    Zone,
    Power,
    Demolish,
    Inspect,
}

impl Tool {
    fn name(self) -> &'static str {
        match self {
            Tool::Road => "Road",
            Tool::Zone => "Zone",
            Tool::Power => "Power",
            Tool::Demolish => "Demolish",
            Tool::Inspect => "Inspect",
        }
    }

    fn hint(self) -> &'static str {
        match self {
            Tool::Road => "1 · drag",
            Tool::Zone => "2 · drag",
            Tool::Power => "3 · click",
            Tool::Demolish => "4 · click",
            Tool::Inspect => "5 · click",
        }
    }

    fn all() -> [Tool; 5] {
        [Tool::Road, Tool::Zone, Tool::Power, Tool::Demolish, Tool::Inspect]
    }
}

struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

struct App {
    gpu: Option<Gpu>,
    text: Text,
    batch: Batcher,
    image_batch: ImageBatcher,
    /// The city's own quads: world space, depth-tested.
    world_batch: WorldBatch,
    camera: Camera,

    /// The interface scale, cycled from the pause menu.
    ui: UiScale,
    /// Whether the first frame's geometry has been reported yet.
    logged_first_frame: bool,

    world: World,
    gov: Government,
    session: Session,

    tool: Tool,
    zone: Zone,
    cursor: (f32, f32),
    press_tile: Option<(i32, i32)>,
    panning: bool,
    orbiting: bool,
    last_cursor: (f32, f32),

    last_frame: Instant,
    accumulator: f32,
    speed: u8,
    ticks_run: u64,

    show_ledger: bool,
    show_help: bool,
    menu_open: bool,
    feedback: String,
    feedback_focus: bool,

    /// The session console (a78): a pane beside the narrowed world view that
    /// reads the agent's ledger and the season store. Read-only by design —
    /// the agent's tool is the only writer (Q127).
    console_open: bool,
    console_scroll: ui::Scroll,
    /// The build identity, read once at startup (a76: a reading of the tree
    /// at launch, not a live VCS poll).
    build_info: ala_cities::buildinfo::BuildInfo,
    /// Fault annotations against the debugger itself (a71 item 4): what the
    /// player reported about this pane, shown back in the pane.
    console_faults: Vec<String>,

    /// The ledger panel's scroll offset. The ledger keeps a private offset
    /// because it is not the tool's one action — its wheel events are routed
    /// only while it is open, so it can never steal the wheel from the camera.
    ledger_scroll: ui::Scroll,

    /// Icons that a decision promoted into the shipping set. Empty until the
    /// first decision exists — the toolbar then draws text-only, honestly.
    icon_set: IconSet,
    /// UI feedback sounds. `None` when no output device exists: the game runs
    /// silent rather than failing over feedback.
    audio: Option<audio::Player>,

    /// The most recent refusal, shown as its own tier rather than as an error.
    refusal: Option<String>,
    /// The most recent governed action, so the session chrome can name it.
    last_ticket: Option<String>,
    last_verdict: Option<String>,
    toast: Option<(String, u64)>,
}

impl App {
    fn new() -> std::io::Result<Self> {
        let world = World::new(MAP, MAP, SEED);
        let governor_path: PathBuf = std::env::var("ALA_GOVERNOR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("config/governor.json"));
        let gov = Government::open("saves", "season_2026_s1", &governor_path)?;
        let session = Session::open(STAGE, &format!("{}-dev", env!("CARGO_PKG_VERSION")))?;
        // Read once at launch (a76): the pane shows the tree the build ran
        // from, not a live poll of the repository.
        let build_info = ala_cities::buildinfo::query();
        let screen = Screen { w: 1600.0, h: 900.0 };

        Ok(Self {
            gpu: None,
            text: Text::new(),
            batch: Batcher::default(),
            image_batch: ImageBatcher::default(),
            world_batch: WorldBatch::default(),
            camera: Camera::new(screen, MAP, MAP),

            ui: UiScale::default(),
            logged_first_frame: false,
            world,
            gov,
            session,
            tool: Tool::Road,
            zone: Zone::Residential,
            cursor: (0.0, 0.0),
            press_tile: None,
            panning: false,
            orbiting: false,
            last_cursor: (0.0, 0.0),
            last_frame: Instant::now(),
            accumulator: 0.0,
            speed: 1,
            ticks_run: 0,
            show_ledger: false,
            show_help: false,
            menu_open: false,
            feedback: String::new(),
            feedback_focus: false,
            console_open: false,
            console_scroll: ui::Scroll::new(),
            build_info,
            console_faults: Vec::new(),
            ledger_scroll: ui::Scroll::new(),
            icon_set: IconSet::default(),
            audio: None,
            refusal: None,
            last_ticket: None,
            last_verdict: None,
            toast: None,
        })
    }

    fn screen(&self) -> Screen {
        match self.gpu.as_ref() {
            Some(gpu) => gpu.screen(),
            None => self.camera.screen,
        }
    }

    // -----------------------------------------------------------------
    // Simulation
    // -----------------------------------------------------------------

    fn advance(&mut self, real_seconds: f32) {
        let sim_dt = 1.0 / SIM_HZ as f32;
        if self.speed == 0 || self.menu_open {
            return;
        }
        self.accumulator += real_seconds.min(0.25) * self.speed as f32;

        // A fixed number of steps per frame, so a stalled frame catches up
        // rather than spiralling into an ever-growing backlog.
        let mut steps = 0;
        while self.accumulator >= sim_dt && steps < 8 {
            self.world.tick();
            self.accumulator -= sim_dt;
            steps += 1;
            self.ticks_run += 1;
            self.govern();
        }
        if steps == 8 {
            // Dropping the remainder is honest: the city falls behind in real
            // time rather than the record gaining ticks that never simulated.
            self.accumulator = 0.0;
        }
    }

    /// Governance work, on cadences that do not need to run every tick.
    fn govern(&mut self) {
        let tick = self.world.clock.tick;

        if tick.is_multiple_of(4) {
            if let Ok(closed) = self.gov.validate(&self.world, tick) {
                if let Some(id) = closed.last() {
                    self.toast = Some((format!("{id} validated by reading the world back"), tick));
                    self.play(Sound::Notify);
                }
            }
        }
        if tick.is_multiple_of(40) {
            let _ = self.gov.sample(&self.world, tick);
        }
        if tick.is_multiple_of(20) {
            let _ = self.gov.enforce_bound(tick);
        }
        if tick.is_multiple_of(200) {
            let _ = self.gov.reconcile(&self.world, tick);
        }

        // The capture is flushed as the session runs, not only on a clean exit.
        // A playtest lost to a crash is a playtest somebody has to sit through
        // again, and the entire point of capturing it is that it survives.
        if tick.is_multiple_of(600) {
            if let Ok(events) = self.session.flush() {
                tracing::debug!(events, "capture flushed mid-session");
            }
        }

        // Autosave the world once a sim-month, on the same boundary the
        // economy already posts on. The record is the part that must survive a
        // crash; the snapshot is the part that lets the verifier re-read a
        // verdict out of the world rather than only out of the paperwork, and
        // without a periodic snapshot there is no world left to check it
        // against. Measured at the shipped map size: 4.5 MB, ~31 ms — a
        // once-a-sim-month cost, not a per-frame one.
        if tick > 0 && tick.is_multiple_of(TICKS_PER_DAY * DAYS_PER_MONTH) {
            self.autosave();
        }
    }

    fn autosave(&mut self) {
        let path = PathBuf::from(WORLD_SAVE);
        match self.world.save(&path) {
            Ok(()) => {
                tracing::debug!(tick = self.world.clock.tick, "world autosaved");
            }
            Err(err) => {
                // A failed save is surfaced, not swallowed: silently continuing
                // would let the session end believing its world was on disk.
                tracing::warn!(%err, "autosave failed");
                self.toast = Some((format!("autosave failed: {err}"), self.world.clock.tick));
            }
        }
    }

    // -----------------------------------------------------------------
    // Actions. The governor decides first; the world changes second; the
    // ticket is filed before either.
    // -----------------------------------------------------------------

    fn act_road(&mut self, from: (i32, i32), to: (i32, i32)) {
        if !self.world.in_bounds(from.0, from.1) || !self.world.in_bounds(to.0, to.1) {
            return;
        }
        let from_tile = self.world.index(from.0 as u32, from.1 as u32);
        let to_tile = self.world.index(to.0 as u32, to.1 as u32);
        let legs = self.world.leg_tiles(from_tile, to_tile);
        let distance = legs.len() as u32;

        let op = Op::BuildRoad { distance };
        let verdict = self.gov.authorize_or_record(self.world.clock.tick, &op);
        if !verdict.allowed {
            self.refuse(&op, &verdict.reason);
            return;
        }

        // What existed before, so a change that breaks something already there
        // is recorded as a regression rather than quietly absorbed.
        let roads_before: Vec<u32> = legs
            .iter()
            .copied()
            .filter(|&tile| self.world.tile(tile).road)
            .collect();

        let laid = self.world.lay_road(from_tile, to_tile);
        if laid.is_empty() {
            self.refusal = Some("that leg is already built; nothing changed".to_string());
            return;
        }

        let id = self
            .gov
            .file(
                self.world.clock.tick,
                TicketKind::Build,
                format!("lay {distance} tiles of road"),
                format!("all {distance} tiles are road", ),
                Expectation::RoadTiles(laid.clone()),
                roads_before,
                verdict.governor_version,
            )
            .unwrap_or_else(|err| format!("unfiled ({err})"));

        self.last_ticket = Some(id.clone());
        self.last_verdict = Some(verdict.reason.clone());
        self.refusal = None;
        self.play(Sound::Place);
        self.capture(
            "build_road",
            Some((from.0, from.1)),
            format!("{distance} tiles, filed as {id}"),
        );
    }

    fn act_zone(&mut self, tiles: &[u32]) {
        let targets: Vec<u32> = tiles
            .iter()
            .copied()
            .filter(|&tile| {
                let t = self.world.tile(tile);
                t.terrain == Terrain::Ground && !t.road && t.building.is_none() && t.zone != self.zone
            })
            .collect();
        if targets.is_empty() {
            return;
        }

        let op = Op::Zone {
            tiles: targets.len() as u32,
        };
        let verdict = self.gov.authorize_or_record(self.world.clock.tick, &op);
        if !verdict.allowed {
            self.refuse(&op, &verdict.reason);
            return;
        }

        let mut applied = Vec::new();
        for tile in &targets {
            if self.world.set_zone(*tile, self.zone) {
                applied.push((*tile, self.zone));
            }
        }
        if applied.is_empty() {
            return;
        }

        let id = self
            .gov
            .file(
                self.world.clock.tick,
                TicketKind::Build,
                format!("zone {} tiles {}", applied.len(), self.zone.name()),
                format!("all {} tiles carry the zone set", applied.len()),
                Expectation::ZonedTiles(applied),
                Vec::new(),
                verdict.governor_version,
            )
            .unwrap_or_else(|err| format!("unfiled ({err})"));

        self.last_ticket = Some(id.clone());
        self.last_verdict = Some(verdict.reason.clone());
        self.refusal = None;
        self.play(Sound::Place);
        let (cx, cy) = self.world.coords(targets[0]);
        self.capture(
            "zone",
            Some((cx as i32, cy as i32)),
            format!("{} tiles as {}, filed as {id}", targets.len(), self.zone.name()),
        );
    }

    fn act_power(&mut self, tile: u32) {
        let op = Op::PlaceService {
            kind: BuildingKind::PowerPlant,
        };
        let verdict = self.gov.authorize_or_record(self.world.clock.tick, &op);
        if !verdict.allowed {
            self.refuse(&op, &verdict.reason);
            return;
        }
        if !self.world.has_road_access(tile) {
            self.refusal =
                Some("a power plant needs a road within two tiles to connect to".to_string());
            return;
        }
        if self.world.economy.credits < BuildingKind::PowerPlant.build_cost() {
            self.refusal = Some(format!(
                "the city has {} credits and a plant costs {}",
                self.world.economy.credits,
                BuildingKind::PowerPlant.build_cost()
            ));
            return;
        }

        match self.world.place_building(tile, BuildingKind::PowerPlant) {
            Some(id) => {
                self.world.economy.credits -= BuildingKind::PowerPlant.build_cost();
                let ticket = self
                    .gov
                    .file(
                        self.world.clock.tick,
                        TicketKind::Build,
                        format!("place power plant {id}"),
                        "a structure stands on the tile".to_string(),
                        Expectation::BuildingAt(tile),
                        Vec::new(),
                        verdict.governor_version,
                    )
                    .unwrap_or_else(|err| format!("unfiled ({err})"));
                // The material claim, filed as its own kind (a175) and closed only by
                // reading the structure back. The claim is the world's own derivation
                // from the declared mapping, so the ticket and the world cannot disagree
                // about what was built — only about whether it is still there.
                if let Some(claim) = self
                    .world
                    .building_on(tile)
                    .and_then(|building| building.material_as_built.clone())
                {
                    let material_ticket = self
                        .gov
                        .file(
                            self.world.clock.tick,
                            TicketKind::Material,
                            format!("material of power plant {id}"),
                            format!("built as {}", claim.describe()),
                            Expectation::MaterialOf {
                                tile,
                                part: claim.part.clone(),
                                family: claim.family.clone(),
                                anchor: claim.anchor.clone(),
                                level: claim.level.clone(),
                            },
                            Vec::new(),
                            verdict.governor_version,
                        )
                        .unwrap_or_else(|err| format!("unfiled ({err})"));
                    self.last_ticket = Some(material_ticket);
                } else {
                    self.last_ticket = Some(ticket.clone());
                }
                self.last_verdict = Some(verdict.reason.clone());
                self.refusal = None;
                self.play(Sound::Place);
                let (x, y) = self.world.coords(tile);
                self.capture(
                    "place_service",
                    Some((x as i32, y as i32)),
                    format!("power plant {id}, filed as {ticket}"),
                );
            }
            None => {
                self.refusal = Some("something already stands there".to_string());
            }
        }
    }

    fn act_demolish(&mut self, tile: u32) {
        let op = Op::Demolish { tiles: 1 };
        let verdict = self.gov.authorize_or_record(self.world.clock.tick, &op);
        if !verdict.allowed {
            self.refuse(&op, &verdict.reason);
            return;
        }

        let had = self.world.tile(tile).building.is_some() || self.world.tile(tile).road;
        if !had {
            return;
        }
        // Demolition retires; it never deletes. The reason is recorded, and the
        // building stays in the city's history.
        let retired = self
            .world
            .demolish(tile, ala_cities::gov::RetirementReason::Superseded);
        let id = self
            .gov
            .file(
                self.world.clock.tick,
                TicketKind::Intervention,
                format!(
                    "clear the tile{}",
                    retired
                        .map(|id| format!(" (retiring structure {id})"))
                        .unwrap_or_default()
                ),
                "nothing stands on the tile".to_string(),
                Expectation::Demolished(tile),
                Vec::new(),
                verdict.governor_version,
            )
            .unwrap_or_else(|err| format!("unfiled ({err})"));

        self.last_ticket = Some(id.clone());
        self.last_verdict = Some(verdict.reason.clone());
        self.refusal = None;
        self.play(Sound::Place);
        let (x, y) = self.world.coords(tile);
        self.capture(
            "demolish",
            Some((x as i32, y as i32)),
            format!("retired rather than deleted, filed as {id}"),
        );
    }

    fn refuse(&mut self, op: &Op, reason: &str) {
        // A refusal is a distinct state from an error, and it names the
        // operation and the vocabulary that would have been accepted.
        self.refusal = Some(format!(
            "refused: {} — {reason}. Known operations: {}",
            op.name(),
            ALL_OPS.join(", ")
        ));
        self.gov.record_refusal(self.world.clock.tick, op.name(), reason);
        self.last_verdict = Some(reason.to_string());
        self.capture("refused", None, format!("{} — {reason}", op.name()));
        self.play(Sound::Refuse);
    }

    /// One UI feedback sound, if there is a device to play it on.
    fn play(&self, sound: Sound) {
        if let Some(player) = self.audio.as_ref() {
            player.play(sound);
        }
    }

    fn capture(&mut self, kind: &str, tile: Option<(i32, i32)>, detail: String) {
        let interaction = Interaction {
            tick: self.world.clock.tick,
            wall_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0),
            kind: kind.to_string(),
            tool: Some(self.tool.name().to_string()),
            screen: Some(self.cursor),
            tile,
            ticket: self.last_ticket.clone(),
            detail,
        };
        self.session.record(interaction);
    }

    // -----------------------------------------------------------------
    // Input
    // -----------------------------------------------------------------

    fn on_key(&mut self, code: KeyCode, pressed: bool, text: Option<String>) {
        if !pressed {
            return;
        }

        if self.feedback_focus {
            match code {
                KeyCode::Enter => {
                    let body = self.feedback.trim().to_string();
                    if !body.is_empty() {
                        match write_feedback(
                            STAGE,
                            &self.session,
                            &body,
                            &[format!(
                                "at {} with {} tickets filed, {} retired",
                                self.world.clock.label(),
                                self.gov.tickets.len(),
                                self.gov.retirements.len()
                            )],
                        ) {
                            Ok(path) => {
                                self.toast = Some((
                                    format!("feedback written to {}", path.display()),
                                    self.world.clock.tick,
                                ));
                            }
                            Err(err) => {
                                self.toast =
                                    Some((format!("feedback not written: {err}"), self.world.clock.tick));
                            }
                        }
                        self.feedback.clear();
                    }
                    self.feedback_focus = false;
                }
                KeyCode::Escape => self.feedback_focus = false,
                KeyCode::Backspace => {
                    self.feedback.pop();
                }
                KeyCode::Space => self.feedback.push(' '),
                _ => {
                    if let Some(ch) = text {
                        if ch.chars().all(|c| !c.is_control()) {
                            self.feedback.push_str(&ch);
                        }
                    }
                }
            }
            return;
        }

        match code {
            KeyCode::Escape => {
                if self.menu_open {
                    self.menu_open = false;
                } else {
                    self.menu_open = true;
                    self.speed = 0;
                }
            }
            KeyCode::Space => {
                self.speed = if self.speed == 0 { 1 } else { 0 };
            }
            KeyCode::Tab => {
                self.speed = match self.speed {
                    0 => 1,
                    1 => 2,
                    2 => 3,
                    _ => 1,
                };
            }
            // Home returns the view to the configured angle without touching
            // the city, so getting lost in a free camera is never destructive.
            KeyCode::Home => {
                self.camera.pitch = render::DEFAULT_PITCH;
                self.camera.yaw = 0.0;
            }
            // The interface scale lives behind the pause menu rather than on a
            // bare key, because a stray press should not re-lay-out everything.
            KeyCode::KeyU if self.menu_open => {
                self.ui = self.ui.next();
                self.toast = Some((
                    format!("interface scale {}", self.ui.label()),
                    self.world.clock.tick,
                ));
            }
            KeyCode::Digit1 => self.select_tool(Tool::Road),
            KeyCode::Digit2 => self.select_tool(Tool::Zone),
            KeyCode::Digit3 => self.select_tool(Tool::Power),
            KeyCode::Digit4 => self.select_tool(Tool::Demolish),
            KeyCode::Digit5 => self.select_tool(Tool::Inspect),
            KeyCode::KeyR if self.tool == Tool::Zone => self.zone = Zone::Residential,
            KeyCode::KeyC if self.tool == Tool::Zone => self.zone = Zone::Commercial,
            KeyCode::KeyI if self.tool == Tool::Zone => self.zone = Zone::Industrial,
            KeyCode::KeyL => {
                self.show_ledger = !self.show_ledger;
                // One right-rail surface at a time, both directions.
                if self.show_ledger {
                    self.console_open = false;
                }
            }
            KeyCode::KeyH => self.show_help = !self.show_help,
            // The session console: pause menu (below) or F9 directly, per a78.
            KeyCode::F9 => {
                self.console_open = !self.console_open;
                if self.console_open {
                    self.speed = 0;
                    // Both are right-side surfaces; the pane covers the ledger
                    // and an honest interface does not hide one surface behind
                    // another.
                    self.show_ledger = false;
                }
            }
            KeyCode::KeyD if self.console_open => {
                // The fault annotation path (a71 item 4): report the debugger
                // itself. The report lands in the session capture with the
                // full context, and the pane shows what was reported.
                let note = format!(
                    "console fault reported at {}: build {}",
                    self.world.clock.label(),
                    self.build_info.summary_line()
                );
                self.console_faults.push(note.clone());
                self.capture("console_fault", None, note);
            }
            KeyCode::KeyF if self.menu_open => self.feedback_focus = true,
            KeyCode::KeyS if self.menu_open => {
                let path = PathBuf::from(WORLD_SAVE);
                match self.world.save(&path) {
                    Ok(()) => {
                        self.toast = Some((
                            format!("saved to {}", path.display()),
                            self.world.clock.tick,
                        ))
                    }
                    Err(err) => {
                        self.toast = Some((format!("save failed: {err}"), self.world.clock.tick))
                    }
                }
            }
            KeyCode::KeyO if self.menu_open => {
                let path = PathBuf::from(WORLD_SAVE);
                match World::load(&path) {
                    Ok(world) => {
                        // A migration that ran is said out loud rather than left in the
                        // file: deriving what an old building is made of is a decision
                        // about the record, and a decision nobody is told about is one
                        // nobody can disagree with (a152).
                        let note = match &world.migration {
                            Some(migration) => format!(" · {}", migration.describe()),
                            None => String::new(),
                        };
                        self.world = world;
                        self.toast = Some((
                            format!("loaded {}{note}", path.display()),
                            self.world.clock.tick,
                        ));
                    }
                    Err(err) => {
                        self.toast = Some((format!("load failed: {err}"), self.world.clock.tick))
                    }
                }
            }
            KeyCode::KeyQ if self.menu_open => {
                match self.session.flush() {
                    Ok(events) => {
                        self.toast = Some((
                            format!("{events} interactions written to {}", self.session.path().display()),
                            self.world.clock.tick,
                        ));
                    }
                    Err(err) => {
                        self.toast = Some((
                            format!("capture not written: {err}"),
                            self.world.clock.tick,
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    fn on_click(&mut self, pressed: bool, button: MouseButton) {
        if self.menu_open {
            return;
        }
        match button {
            MouseButton::Left if pressed => {
                // An exact ray, not a screen rectangle: with a free camera this
                // is the only version that stays right when the view tilts.
                let Some((tx, ty)) = self.camera.screen_to_tile(self.cursor.0, self.cursor.1)
                else {
                    return;
                };
                if !self.world.in_bounds(tx, ty) {
                    return;
                }
                let screen = self.screen();
                if toolbar_hit(&screen, self.ui, self.cursor.0, self.cursor.1).is_some() {
                    return;
                }
                self.press_tile = Some((tx, ty));
                if self.tool == Tool::Power {
                    let tile = self.world.index(tx as u32, ty as u32);
                    self.act_power(tile);
                    self.press_tile = None;
                } else if self.tool == Tool::Demolish {
                    let tile = self.world.index(tx as u32, ty as u32);
                    self.act_demolish(tile);
                    self.press_tile = None;
                }
            }
            MouseButton::Left => {
                if let Some(from) = self.press_tile.take() {
                    let Some((tx, ty)) = self.camera.screen_to_tile(self.cursor.0, self.cursor.1)
                    else {
                        return;
                    };
                    match self.tool {
                        Tool::Road => self.act_road(from, (tx, ty)),
                        Tool::Zone => {
                            // Zoning paints over the tiles the drag covered: a
                            // brush, not a line, because a district is a shape.
                            let tiles = self.tiles_between(from, (tx, ty));
                            self.act_zone(&tiles);
                        }
                        _ => {}
                    }
                }
            }
            // Right-drag orbits: free rotation and free tilt, so the camera is
            // a thing the player points rather than a series of preset angles.
            MouseButton::Right if pressed => {
                self.orbiting = true;
                self.last_cursor = self.cursor;
            }
            MouseButton::Right => self.orbiting = false,
            // Middle-drag pans, and it is separate from orbiting so a player can
            // move the city without changing the angle they are reading it at.
            MouseButton::Middle if pressed => {
                self.panning = true;
                self.last_cursor = self.cursor;
            }
            MouseButton::Middle => self.panning = false,
            _ => {}
        }
    }

    /// Every tile inside the rectangle a drag covered.
    fn tiles_between(&self, from: (i32, i32), to: (i32, i32)) -> Vec<u32> {
        let (x0, x1) = (from.0.min(to.0), from.0.max(to.0));
        let (y0, y1) = (from.1.min(to.1), from.1.max(to.1));
        let mut tiles = Vec::new();
        for y in y0..=y1 {
            for x in x0..=x1 {
                if self.world.in_bounds(x, y) {
                    tiles.push(self.world.index(x as u32, y as u32));
                }
            }
        }
        tiles
    }

    // -----------------------------------------------------------------
    // Drawing
    // -----------------------------------------------------------------

    fn draw_world(&mut self) {
        let (left, top, right, bottom) = self.camera.visible_tiles(MAP, MAP);
        let tick = self.world.clock.tick;
        // Face culling is decided against the camera that is about to draw.
        self.world_batch.clear(self.camera.toward_camera());

        for y in top..bottom {
            for x in left..right {
                let tile = self.world.index(x, y);
                let t = self.world.tile(tile);
                let (wx, wy) = (x as f32 * TILE, y as f32 * TILE);

                let ground = match t.terrain {
                    Terrain::Water => Token::Water,
                    Terrain::Ground => Token::Ground,
                };
                let ground_color = hud::style(ground).fill.unwrap_or([0.2, 0.3, 0.2, 1.0]);
                self.world_batch
                    .ground_rect(Layer::Opaque, wx, wy, TILE, TILE, 0.0, ground_color);

                if t.zone != Zone::None && t.building.is_none() {
                    let token = match t.zone {
                        Zone::Residential => Token::ZoneResidential,
                        Zone::Commercial => Token::ZoneCommercial,
                        Zone::Industrial => Token::ZoneIndustrial,
                        Zone::None => Token::Ground,
                    };
                    let color = hud::style(token).fill.unwrap_or([0.5, 0.5, 0.5, 1.0]);
                    // Zones are painted *on* the ground, so they are an overlay
                    // slightly above it rather than a replacement for it.
                    self.world_batch.ground_rect(
                        Layer::Overlay,
                        wx + 1.0,
                        wy + 1.0,
                        TILE - 2.0,
                        TILE - 2.0,
                        0.05,
                        hud::with_alpha(color, 0.45),
                    );
                    // A border, so a zoned district reads as a shape at a
                    // glancing angle where the fill alone would not.
                    self.world_batch.ground_border(
                        Layer::Overlay,
                        wx + 1.0,
                        wy + 1.0,
                        TILE - 2.0,
                        TILE - 2.0,
                        0.06,
                        1.0,
                        hud::with_alpha(color, 0.7),
                    );
                    if !t.powered {
                        // Unpowered and zoned is the condition the cases are
                        // sampled from, so it is visible rather than reported.
                        let brown = hud::style(Token::Brownout).text.unwrap_or([1.0, 0.6, 0.0, 1.0]);
                        self.world_batch.ground_rect(
                            Layer::Overlay,
                            wx + 4.0,
                            wy + 4.0,
                            TILE - 8.0,
                            TILE - 8.0,
                            0.08,
                            brown,
                        );
                    }
                }

                if t.road {
                    let color = hud::style(Token::Road).fill.unwrap_or([0.3, 0.3, 0.3, 1.0]);
                    let edge = hud::style(Token::RoadEdge).fill.unwrap_or([0.5, 0.5, 0.5, 1.0]);
                    // Roads sit just above the ground so their edges stay
                    // readable at a glancing angle without z-fighting.
                    self.world_batch
                        .ground_rect(Layer::Opaque, wx, wy, TILE, TILE, 0.4, color);
                    self.world_batch.ground_border(
                        Layer::Opaque,
                        wx,
                        wy,
                        TILE,
                        TILE,
                        0.42,
                        1.0,
                        hud::with_alpha(edge, 0.5),
                    );
                }

                if let Some(index) = t.building {
                    let b = self.world.building(index);
                    let color = if tick < b.ready_tick {
                        hud::style(Token::Scaffold).fill.unwrap_or([0.6, 0.5, 0.2, 1.0])
                    } else {
                        match b.kind {
                            BuildingKind::Home => hud::style(Token::ZoneResidential).fill.unwrap_or([0.4, 0.7, 0.4, 1.0]),
                            BuildingKind::Shop => hud::style(Token::ZoneCommercial).fill.unwrap_or([0.4, 0.5, 0.8, 1.0]),
                            BuildingKind::Factory => hud::style(Token::ZoneIndustrial).fill.unwrap_or([0.8, 0.6, 0.3, 1.0]),
                            BuildingKind::PowerPlant => hud::style(Token::Powered).text.unwrap_or([0.5, 0.9, 0.6, 1.0]),
                        }
                    };
                    let height = building_height(b, tick);
                    // A contact shadow, offset along the same light that shades
                    // the faces, so a shadow cannot point away from its own
                    // building.
                    let shadow = hud::style(Token::Desk).fill.unwrap_or([0.06, 0.06, 0.07, 1.0]);
                    let reach = shadow_reach(height);
                    self.world_batch.ground_rect(
                        Layer::Overlay,
                        wx + 1.5 + reach.0,
                        wy + 1.5 + reach.1,
                        TILE - 3.0,
                        TILE - 3.0,
                        0.1,
                        hud::with_alpha(shadow, 0.35),
                    );
                    // The building is a box, extruded by level. This is the
                    // whole of the 3D: a footprint, a height, and five quads.
                    self.world_batch.building(
                        wx + TILE / 2.0,
                        wy + TILE / 2.0,
                        (TILE - 3.0) / 2.0,
                        height,
                        color,
                        1.0,
                    );
                }
            }
        }

        // Retired structures are drawn, faintly. The city shows what it used to
        // be rather than pretending those buildings never existed.
        let retired_color = hud::style(Token::Retired).text.unwrap_or([0.6, 0.6, 0.6, 0.5]);
        for building in &self.world.buildings {
            if building.retired.is_none() {
                continue;
            }
            let (x, y) = self.world.coords(building.tile);
            if x < left || x >= right || y < top || y >= bottom {
                continue;
            }
            // A ghost of its own volume: still on the map, visibly not a
            // building, and never a fill that could be mistaken for one.
            self.world_batch.building(
                x as f32 * TILE + TILE / 2.0,
                y as f32 * TILE + TILE / 2.0,
                (TILE - 4.0) / 2.0,
                building.level as f32 * LEVEL_HEIGHT * 0.6,
                retired_color,
                0.25,
            );
        }

        // Agents. Position comes from the work an agent has completed, so a
        // congested citizen stands still rather than gliding forward.
        let agent = hud::style(Token::Agent).fill.unwrap_or([1.0, 1.0, 1.0, 1.0]);
        let stuck = hud::style(Token::AgentStuck).fill.unwrap_or([1.0, 0.3, 0.3, 1.0]);
        for citizen in &self.world.citizens {
            let Some(tile) = citizen.current_tile() else {
                continue;
            };
            let (x, y) = self.world.coords(tile);
            if x < left || x >= right || y < top || y >= bottom {
                continue;
            }
            // Only full-detail agents are drawn; offscreen cohorts are counted,
            // not rendered, which is what keeps the frame budget honest. The
            // distance is measured from the tile the camera is looking at, so
            // the split is a function of view rather than of frame timing.
            let focus = (
                (self.camera.focus.x / TILE).floor() as i32,
                (self.camera.focus.y / TILE).floor() as i32,
            );
            let distance = (x as i32 - focus.0).abs() + (y as i32 - focus.1).abs();
            if distance > ala_cities::sim::LOD_RADIUS {
                continue;
            }

            let (fx, fy) = match citizen.interpolated_tiles() {
                Some((from, to, work)) => {
                    let (ax, ay) = self.world.coords(from);
                    let (bx, by) = self.world.coords(to);
                    (
                        (ax as f32 + (bx as f32 - ax as f32) * work) * TILE + TILE * 0.3,
                        (ay as f32 + (by as f32 - ay as f32) * work) * TILE + TILE * 0.3,
                    )
                }
                None => (x as f32 * TILE + TILE * 0.3, y as f32 * TILE + TILE * 0.3),
            };
            let color = if citizen.state == CitizenState::Unemployed {
                stuck
            } else {
                agent
            };
            // A small body standing on the street rather than a flat dot, so a
            // crowded junction is readable from an angle.
            self.world_batch.building(
                fx + TILE * 0.2,
                fy + TILE * 0.2,
                TILE * 0.12,
                TILE * 0.45,
                color,
                1.0,
            );
        }

        // A preview of the leg the drag would build, so the action is visible
        // before it is taken rather than only afterwards.
        if let Some(from) = self.press_tile {
            if self.tool == Tool::Road {
                let target = self.camera.screen_to_tile(self.cursor.0, self.cursor.1);
                if let Some((tx, ty)) = target.filter(|(tx, ty)| self.world.in_bounds(*tx, *ty)) {
                    let from_tile = self.world.index(from.0 as u32, from.1 as u32);
                    let to_tile = self.world.index(tx as u32, ty as u32);
                    let legs = self.world.leg_tiles(from_tile, to_tile);
                    let allowed = self
                        .gov
                        .governor
                        .authorize(&Op::BuildRoad {
                            distance: legs.len() as u32,
                        })
                        .allowed;
                    let color = if allowed {
                        hud::style(Token::Nature).fill.unwrap_or([0.4, 0.9, 0.5, 1.0])
                    } else {
                        hud::style(Token::Refused).text.unwrap_or([0.9, 0.3, 0.3, 1.0])
                    };
                    for tile in legs {
                        let (x, y) = self.world.coords(tile);
                        self.world_batch.ground_rect(
                            Layer::Overlay,
                            x as f32 * TILE,
                            y as f32 * TILE,
                            TILE,
                            TILE,
                            0.3,
                            hud::with_alpha(color, 0.5),
                        );
                    }
                }
            }
        }

        self.draw_grid_sense(left, top, right, bottom);
    }

    /// Knowing where you are on the grid, at any angle.
    ///
    /// Three indicators, because they answer three different questions:
    ///
    /// * the **tile under the cursor**, highlighted in every mode, always —
    ///   which tile;
    /// * a **local grid patch** that fades with distance, drawn only while a
    ///   build tool is active — which way the lattice runs;
    /// * the **bearing and coordinates** in the corner of the interface, drawn
    ///   by `draw_hud` — where in the city you are.
    ///
    /// A whole-map grid drawn at all times would answer none of them well once
    /// the camera tilts, which is why it is not the design.
    fn draw_grid_sense(&mut self, left: u32, top: u32, right: u32, bottom: u32) {
        let ink = hud::style(Token::TextBody).text.unwrap_or([1.0, 1.0, 1.0, 1.0]);
        let grid = hud::style(Token::Grid).text.unwrap_or([0.35, 0.35, 0.4, 0.35]);

        // A local patch, faded by distance from the cursor, and only while a
        // placement tool is in hand. Zones and roads are placed by the tile, so
        // this is the mode that needs the lattice.
        if self.tool != Tool::Inspect {
            if let Some((hx, hy)) = self.camera.screen_to_tile(self.cursor.0, self.cursor.1) {
                let radius = 14i32;
                let thickness = 0.35;
                for step in -radius..=radius {
                    let fade = 1.0 - (step.abs() as f32 / radius as f32);
                    let alpha = fade * 0.5;
                    if alpha <= 0.02 {
                        continue;
                    }
                    let colour = hud::with_alpha(grid, alpha);
                    let (gx, gy) = (hx + step, hy + step);
                    // North-south line at this x, clipped to the visible tiles.
                    let x = gx as f32 * TILE;
                    if gx >= 0 && (gx as u32) <= right {
                        let y0 = top.max(hy.saturating_sub(radius) as u32) as f32 * TILE;
                        let y1 = (bottom.min((hy + radius).max(0) as u32)) as f32 * TILE;
                        self.world_batch.ground_rect(
                            Layer::Overlay,
                            x,
                            y0,
                            thickness,
                            (y1 - y0).max(0.0),
                            0.12,
                            colour,
                        );
                    }
                    let y = gy as f32 * TILE;
                    if gy >= 0 && (gy as u32) <= bottom {
                        let x0 = left.max(hx.saturating_sub(radius) as u32) as f32 * TILE;
                        let x1 = (right.min((hx + radius).max(0) as u32)) as f32 * TILE;
                        self.world_batch.ground_rect(
                            Layer::Overlay,
                            x0,
                            y,
                            (x1 - x0).max(0.0),
                            thickness,
                            0.12,
                            colour,
                        );
                    }
                }
            }
        }

        // The tile under the cursor: an outline, not a fill, so whatever is on
        // the tile stays readable through it, plus a faint bed so the outline
        // is visible over water and roads alike.
        if let Some((hx, hy)) = self.camera.screen_to_tile(self.cursor.0, self.cursor.1) {
            if self.world.in_bounds(hx, hy) {
                let (wx, wy) = (hx as f32 * TILE, hy as f32 * TILE);
                self.world_batch.ground_rect(
                    Layer::Overlay,
                    wx,
                    wy,
                    TILE,
                    TILE,
                    0.18,
                    hud::with_alpha(ink, 0.14),
                );
                self.world_batch.ground_border(
                    Layer::Overlay,
                    wx,
                    wy,
                    TILE,
                    TILE,
                    0.22,
                    1.2,
                    hud::with_alpha(ink, 0.95),
                );
                // A short axis arm on two edges, so the tile's own orientation
                // is legible even when the lattice is not drawn.
                self.world_batch.ground_rect(
                    Layer::Overlay,
                    wx - 2.0,
                    wy,
                    2.0,
                    TILE,
                    0.2,
                    hud::with_alpha(ink, 0.5),
                );
                self.world_batch.ground_rect(
                    Layer::Overlay,
                    wx,
                    wy - 2.0,
                    TILE,
                    2.0,
                    0.2,
                    hud::with_alpha(ink, 0.5),
                );
            }
        }
    }

    fn draw_hud(&mut self) {
        let screen = self.screen();
        let tick = self.world.clock.tick;
        // Every inset and gap in the chrome comes from the design's 4-unit
        // scale, so the interface re-lays out at a new UI scale instead of
        // being stretched.
        let ui = self.text.ui_scale();
        let pad = |space: Space| hud::space(space, ui);
        // One line of text, and one comfortable target. Row advances and panel
        // insets are built from these rather than from their own numbers, so
        // the interface re-lays out at a new scale instead of drifting off it.
        let small_line = Step::Small.px(ui) as f32 * ui::LINE_ADVANCE_FACTOR;
        let target_px = design::target(ui);

        // ---- governed-session chrome -------------------------------------
        // One bar tall in the small step's own metrics — never a literal 30
        // that clips its text at 200% scale. The run of labels is already
        // measured flow (each width is measured before the next x lands);
        // what the literals got wrong was the bar's height and baseline, and
        // both now come from the font.
        let bar_line = small_line;
        let bar_h = pad(Space::Sm) * 2.0 + bar_line;
        // `draw`'s y is an ascent line — ink begins about one ascent below it
        // — so the bar's text y is simply the top pad; the font does the rest.
        let bar_text_y = pad(Space::Sm);

        hud::panel(&mut self.batch, &screen, 0.0, 0.0, screen.w, bar_h, Token::Panel);
        let mut x = pad(Space::Md);
        x += hud::label_mono(
            &mut self.text,
            &mut self.batch,
            &screen,
            x,
            bar_text_y,
            Step::Small,
            Token::TextMuted,
            &self.gov.contract,
        ) + pad(Space::Lg);
        x += hud::label_mono(
            &mut self.text,
            &mut self.batch,
            &screen,
            x,
            bar_text_y,
            Step::Small,
            Token::TextMuted,
            &self.gov.governor_banner(),
        ) + pad(Space::Lg);
        x += hud::label(
            &mut self.text,
            &mut self.batch,
            &screen,
            x,
            bar_text_y,
            Step::Small,
            Token::Procedural,
            "identity: PROCEDURAL",
        ) + pad(Space::Lg);
        if let Some(ticket) = &self.last_ticket {
            hud::label_mono(
                &mut self.text,
                &mut self.batch,
                &screen,
                x,
                bar_text_y,
                Step::Small,
                Token::TextMuted,
                ticket,
            );
        }
        // A missing font is stated rather than discovered: the interface looks
        // broken when it cannot draw a string, and that is not something to
        // leave the player to work out.
        if self.text.missing_font {
            hud::label(
                &mut self.text,
                &mut self.batch,
                &screen,
                screen.w / 2.0 - 150.0,
                screen.h - 90.0,
                Step::Small,
                Token::Warning,
                "no system font found: text is not being drawn",
            );
        }

        let clock = self.world.clock.label();
        let speed = match self.speed {
            0 => "paused",
            1 => "1x",
            2 => "2x",
            _ => "3x",
        };
        let centre = format!("{clock}   {speed}");
        let width = self.text.measure_step(Face::Mono, &centre, Step::Small);
        hud::label_mono(
            &mut self.text,
            &mut self.batch,
            &screen,
            screen.w / 2.0 - width / 2.0,
            bar_text_y,
            Step::Small,
            Token::TextBody,
            &centre,
        );

        let mut right = screen.w - pad(Space::Md);
        let fps = format!("{:.0} fps", self.gpu.as_ref().map(|g| g.stats.reported_fps).unwrap_or(0.0));
        let fps_width = self.text.measure_step(Face::Mono, &fps, Step::Small);
        hud::label_mono(
            &mut self.text,
            &mut self.batch,
            &screen,
            right - fps_width,
            bar_text_y,
            Step::Small,
            Token::TextMuted,
            &fps,
        );
        right -= fps_width + pad(Space::Lg);

        let rec = "● RECORDING";
        let rec_width = self.text.measure_step(Face::Body, rec, Step::Small);
        hud::label(
            &mut self.text,
            &mut self.batch,
            &screen,
            right - rec_width,
            bar_text_y,
            Step::Small,
            if self.session.recording {
                Token::Recording
            } else {
                Token::NotObtained
            },
            rec,
        );

        // ---- left column: demand, state, what to do next ------------------
        // The panel's height **is** the measurement: the blocks are built,
        // measured, and only then does a panel get drawn around them. The
        // old code reserved a literal 226.0 and advanced its own cursor —
        // two truths that happened to agree at 100% scale and silently
        // clipped the last stat row at every other scale.
        let panel_x = pad(Space::Md);
        let panel_y = bar_h + pad(Space::Sm) + pad(Space::Xs);
        let panel_w = 250.0 * ui.0;
        let muted = hud::style(Token::TextMuted).text.unwrap_or([0.8; 4]);
        let body_ink = hud::style(Token::TextBody).text.unwrap_or([1.0; 4]);

        let mut demand_blocks = vec![Block::Line {
            face: Face::Body,
            step: Step::Body,
            text: "Demand".into(),
            color: body_ink,
        }];
        for (token, name, value) in [
            (Token::ZoneResidential, "residential", self.world.demand.residential),
            (Token::ZoneCommercial, "commercial", self.world.demand.commercial),
            (Token::ZoneIndustrial, "industrial", self.world.demand.industrial),
        ] {
            demand_blocks.push(Block::Bar {
                label: name.into(),
                step: Step::Small,
                label_color: muted,
                color: hud::style(token).fill.unwrap_or([0.5; 4]),
                fraction: value,
            });
        }
        demand_blocks.push(Block::Gap(Space::Sm));
        demand_blocks.push(Block::Rule);
        demand_blocks.push(Block::Gap(Space::Sm));
        let brownout = self.world.stats.brownout;
        for (name, value, is_power) in [
            ("credits", format!("{}", self.world.economy.credits), false),
            ("population", format!("{}", self.world.stats.population), false),
            ("jobs", format!("{}", self.world.stats.jobs), false),
            ("out of work", format!("{}", self.world.stats.unemployed), false),
            ("road tiles", format!("{}", self.world.stats.road_tiles), false),
            (
                "power",
                if brownout {
                    "BROWNOUT".to_string()
                } else {
                    format!("{} plants", self.world.stats.power_plants)
                },
                brownout,
            ),
            ("retired", format!("{}", self.world.stats.retired_buildings), false),
            ("no route", format!("{}", self.world.stats.commute_failures), false),
        ] {
            demand_blocks.push(Block::Row {
                label: name.into(),
                value,
                step: Step::Small,
                color: muted,
                value_color: if is_power && brownout {
                    hud::style(Token::Brownout).text.unwrap_or([1.0; 4])
                } else {
                    body_ink
                },
            });
        }
        let demand_inner_w = panel_w - 2.0 * pad(Space::Md);
        let demand_frame = Frame::new(
            panel_x + pad(Space::Md),
            panel_y + pad(Space::Sm),
            demand_inner_w,
            screen.h,
        );
        let demand_measured = ui::measure(&mut self.text, ui, &demand_frame, &demand_blocks);
        let demand_h = demand_measured.content_height + 2.0 * pad(Space::Sm);
        hud::panel(&mut self.batch, &screen, panel_x, panel_y, panel_w, demand_h, Token::Panel);
        let demand_scroll = ui::Scroll::new();
        ui::paint(&demand_measured, &demand_scroll, &mut self.batch, &screen, &mut self.text);

        // What to do next, derived from stored state. An empty list would be a
        // bug, not a quiet moment. Measured like the demand panel: the height
        // comes from the blocks, and the panel sits on the demand panel's
        // *measured* bottom edge rather than on a remembered coordinate.
        let suggestions = self.gov.suggestions(&self.world);
        let mut next_blocks = vec![Block::Line {
            face: Face::Body,
            step: Step::Body,
            text: "What to do next".into(),
            color: body_ink,
        }];
        for line in suggestions.iter().take(4) {
            let is_case = line.starts_with("CSE-");
            let clipped =
                hud::truncate(&mut self.text, Face::Body, line, Step::Small, panel_w - 2.0 * pad(Space::Md));
            next_blocks.push(Block::Line {
                face: Face::Body,
                step: Step::Small,
                text: clipped,
                color: if is_case {
                    hud::style(Token::CaseOpen).text.unwrap_or([1.0; 4])
                } else {
                    muted
                },
            });
        }
        let next_frame = Frame::new(
            panel_x + pad(Space::Md),
            panel_y + demand_h + pad(Space::Sm) + pad(Space::Sm),
            demand_inner_w,
            screen.h,
        );
        let next_measured = ui::measure(&mut self.text, ui, &next_frame, &next_blocks);
        let next_h = next_measured.content_height + 2.0 * pad(Space::Sm);
        hud::panel(
            &mut self.batch,
            &screen,
            panel_x,
            next_frame.y - pad(Space::Sm),
            panel_w,
            next_h,
            Token::Panel,
        );
        let next_scroll = ui::Scroll::new();
        ui::paint(&next_measured, &next_scroll, &mut self.batch, &screen, &mut self.text);

        // ---- ledger -------------------------------------------------------
        if self.show_ledger {
            let (ledger_frame, ledger_blocks) = self.ledger_layout(&screen);
            let ly = ledger_frame.y - pad(Space::Sm);
            let lx = screen.w - 470.0 - pad(Space::Md);
            let lh = screen.bottom_anchor(160.0, 12.0) - ly;
            hud::panel(&mut self.batch, &screen, lx, ly, 470.0, lh, Token::Panel);
            let ledger_measured = ui::measure(&mut self.text, ui, &ledger_frame, &ledger_blocks);
            // Re-clamp the held offset against this frame's measurement
            // (closing tickets only shorten the list), then paint.
            self.ledger_scroll.scroll_by(0.0, &ledger_measured);
            ui::paint(
                &ledger_measured,
                &self.ledger_scroll,
                &mut self.batch,
                &screen,
                &mut self.text,
            );
        }

        // ---- debug console (F9) -------------------------------------------
        // Same right-rail geometry as the ledger — the two surfaces are one
        // family — and the same measure/re-clamp/paint discipline.
        if self.console_open {
            let (console_frame, console_blocks) = self.console_layout(&screen);
            let cy = console_frame.y - pad(Space::Sm);
            let cx = screen.w - 470.0 - pad(Space::Md);
            let ch = screen.bottom_anchor(160.0, 12.0) - cy;
            hud::panel(&mut self.batch, &screen, cx, cy, 470.0, ch, Token::Panel);
            let console_measured = ui::measure(&mut self.text, ui, &console_frame, &console_blocks);
            self.console_scroll.scroll_by(0.0, &console_measured);
            ui::paint(
                &console_measured,
                &self.console_scroll,
                &mut self.batch,
                &screen,
                &mut self.text,
            );
        }

        // ---- per-object page ----------------------------------------------
        if self.tool == Tool::Inspect {
            let (hx, hy) = self
                .camera
                .screen_to_tile(self.cursor.0, self.cursor.1)
                .unwrap_or((0, 0));
            if self.world.in_bounds(hx, hy) {
                let tile = self.world.index(hx as u32, hy as u32);
                if let Some(index) = self.world.tile(tile).building {
                    let b = self.world.building(index);
                    let width = 240.0;
                    let px = (self.cursor.0 + pad(Space::Lg)).min(screen.w - width - 8.0);
                    let mut inspect_blocks = vec![Block::Line {
                        face: Face::Body,
                        step: Step::Body,
                        text: format!("{} #{}", b.kind.name(), b.id),
                        color: body_ink,
                    }];
                    // Only fields the store already admits. Nothing is invented
                    // for the panel to have something to show.
                    for (name, value) in [
                        ("tile", format!("{tile}")),
                        ("level", format!("{}", b.level)),
                        ("powered", if b.powered { "yes".into() } else { "no".into() }),
                        ("occupants", format!("{}", b.occupants)),
                        ("built at tick", format!("{}", b.built_tick)),
                        (
                            "state",
                            if b.retired.is_some() {
                                "retired".to_string()
                            } else if tick < b.ready_tick {
                                "under construction".to_string()
                            } else {
                                "in use".to_string()
                            },
                        ),
                    ] {
                        inspect_blocks.push(Block::Row {
                            label: name.into(),
                            value,
                            step: Step::Small,
                            color: muted,
                            value_color: body_ink,
                        });
                    }
                    // Heights depend on the frame's width, not its origin, so
                    // measure once, clamp the popup's y against the *measured*
                    // height, then point the frame at the final position.
                    let frame = Frame::new(
                        px + pad(Space::Sm),
                        0.0,
                        width - 2.0 * pad(Space::Sm),
                        screen.h,
                    );
                    let mut measured = ui::measure(&mut self.text, ui, &frame, &inspect_blocks);
                    let height = measured.content_height + 2.0 * pad(Space::Sm);
                    let py = (self.cursor.1 + pad(Space::Sm)).min(screen.h - height - 8.0);
                    hud::panel(&mut self.batch, &screen, px, py, width, height, Token::PanelRaised);
                    measured.frame.y = py + pad(Space::Sm);
                    let scroll = ui::Scroll::new();
                    ui::paint(&measured, &scroll, &mut self.batch, &screen, &mut self.text);
                }
            }
        }        // ---- refusal banner ------------------------------------------------
        // Measured: two wrapped lines and an accent bar that follows the
        // content's height, instead of a 46 px box that clips a long refusal
        // at 200% scale.
        if let Some(refusal) = self.refusal.clone() {
            let width = (screen.w - 80.0).min(900.0);
            let x = (screen.w - width) / 2.0;
            let inner_w = width - 3.0 - 2.0 * pad(Space::Md);
            let clipped =
                hud::truncate(&mut self.text, Face::Body, &refusal, Step::Small, inner_w);
            let warn_ink = hud::style(Token::Warning).text.unwrap_or([1.0; 4]);
            let blocks = [
                Block::Line {
                    face: Face::Body,
                    step: Step::Small,
                    text: clipped,
                    color: warn_ink,
                },
                Block::Line {
                    face: Face::Body,
                    step: Step::Small,
                    text: "this is a refusal, not an error: the city is unchanged".into(),
                    color: muted,
                },
            ];
            let frame = Frame::new(x + 3.0 + pad(Space::Md), 0.0, inner_w, screen.h);
            let mut measured = ui::measure(&mut self.text, ui, &frame, &blocks);
            let height = measured.content_height + 2.0 * pad(Space::Sm);
            let y = screen.bottom_anchor(height + 150.0, 12.0);
            hud::panel(&mut self.batch, &screen, x, y, width, height, Token::Panel);
            hud::panel(&mut self.batch, &screen, x, y, 3.0, height, Token::Warning);
            measured.frame.y = y + pad(Space::Sm);
            let scroll = ui::Scroll::new();
            ui::paint(&measured, &scroll, &mut self.batch, &screen, &mut self.text);
        }

        // ---- toast ---------------------------------------------------------
        // Measured the same way: the pill is one small line plus its padding,
        // whatever the scale.
        if let Some((message, at)) = self.toast.clone() {
            if tick.saturating_sub(at) < 120 {
                let max_w = screen.w * 0.7;
                let clipped =
                    hud::truncate(&mut self.text, Face::Body, &message, Step::Small, max_w);
                let text_w = self.text.measure_step(Face::Body, &clipped, Step::Small);
                let width = text_w + 2.0 * pad(Space::Md);
                let blocks = [Block::Line {
                    face: Face::Body,
                    step: Step::Small,
                    text: clipped,
                    color: body_ink,
                }];
                let frame = Frame::new(0.0, 0.0, text_w, screen.h);
                let mut measured = ui::measure(&mut self.text, ui, &frame, &blocks);
                let height = measured.content_height + 2.0 * pad(Space::Sm);
                let x = (screen.w - width) / 2.0;
                let y = bar_h + pad(Space::Xs);
                hud::panel(&mut self.batch, &screen, x, y, width, height, Token::PanelRaised);
                measured.frame = Frame::new(x + pad(Space::Md), y + pad(Space::Sm), text_w, height);
                let scroll = ui::Scroll::new();
                ui::paint(&measured, &scroll, &mut self.batch, &screen, &mut self.text);
            } else {
                self.toast = None;
            }
        }

        self.draw_compass(&screen);

        // ---- toolbar --------------------------------------------------------
        // A decided tool icon replaces the text label at recognition size;
        // an undecided one keeps its text. The two never mix inside one
        // button — half-icon, half-text is neither honest nor legible.
        for (tool, rect) in toolbar_layout(&screen, self.ui) {
            let active = tool == self.tool;
            hud::panel(
                &mut self.batch,
                &screen,
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                if active { Token::Ink } else { Token::PanelRaised },
            );
            let icon_id = match tool {
                Tool::Road => Some("tool-road"),
                Tool::Zone => Some("tool-zone"),
                Tool::Power => Some("tool-power"),
                Tool::Demolish => Some("tool-demolish"),
                Tool::Inspect => Some("tool-inspect"),
            };
            let shipped = icon_id.and_then(|id| self.icon_set.get(id));
            if let Some(icon) = shipped {
                // The icon sits centred on the button's left edge, at the
                // recognition size the manifest declares — the size it was
                // checked to be legible at.
                let size = icon.src[2];
                let icon_y = rect.y + (rect.h - size) * 0.5;
                self.image_batch.image(
                    &screen,
                    rect.x + pad(Space::Sm),
                    icon_y,
                    size,
                    size,
                    icon.src,
                    [1.0; 4],
                );
                let text_x = rect.x + pad(Space::Sm) + size + pad(Space::Sm);
                let label_y = rect.y
                    + (rect.h - Step::Body.px(ui) as f32) / 2.0
                    - Step::Body.px(ui) as f32 * 0.5;
                hud::label(
                    &mut self.text,
                    &mut self.batch,
                    &screen,
                    text_x,
                    label_y,
                    Step::Body,
                    if active { Token::TextOnInk } else { Token::TextBody },
                    tool.name(),
                );
                hud::label_mono(
                    &mut self.text,
                    &mut self.batch,
                    &screen,
                    text_x,
                    label_y + Step::Body.px(ui) as f32 + pad(Space::Xs),
                    Step::Small,
                    if active { Token::TextOnInk } else { Token::TextMuted },
                    tool.hint(),
                );
            } else {
                // Text is centred against the control's own target height rather
                // than against a number chosen when the button was 56 px tall.
                let label_y = rect.y
                    + (rect.h - Step::Body.px(ui) as f32) / 2.0
                    - Step::Body.px(ui) as f32 * 0.5;
                hud::label(
                    &mut self.text,
                    &mut self.batch,
                    &screen,
                    rect.x + pad(Space::Sm),
                    label_y,
                    Step::Body,
                    if active { Token::TextOnInk } else { Token::TextBody },
                    tool.name(),
                );
                hud::label_mono(
                    &mut self.text,
                    &mut self.batch,
                    &screen,
                    rect.x + pad(Space::Sm),
                    label_y + Step::Body.px(ui) as f32 + pad(Space::Xs),
                    Step::Small,
                    if active { Token::TextOnInk } else { Token::TextMuted },
                    tool.hint(),
                );
            }
        }

        // The zone sub-selector appears only under the tool that uses it, which
        // is mode scope rather than a control that is always there but disabled.
        if self.tool == Tool::Zone {
            let base = screen.bottom_anchor(
                target_px + pad(Space::Sm),
                pad(Space::Md),
            );
            let button_w = 142.0 * ui.0;
            for (index, (zone, key)) in [
                (Zone::Residential, "R"),
                (Zone::Commercial, "C"),
                (Zone::Industrial, "I"),
            ]
            .into_iter()
            .enumerate()
            {
                let x = pad(Space::Md) + index as f32 * button_w;
                let active = zone == self.zone;
                // A zone row is a target too, so it is built from the target
                // height rather than from whatever looked right once.
                hud::panel(
                    &mut self.batch,
                    &screen,
                    x,
                    base - target_px - pad(Space::Xs),
                    button_w - pad(Space::Sm),
                    target_px,
                    if active { Token::PanelRaised } else { Token::Panel },
                );
                hud::label(
                    &mut self.text,
                    &mut self.batch,
                    &screen,
                    x + pad(Space::Sm),
                    base - target_px + pad(Space::Xs),
                    Step::Body,
                    if active { Token::TextBody } else { Token::TextMuted },
                    &format!("{}  ({key})", zone.name()),
                );
            }
        }

        // ---- pause menu -----------------------------------------------------
        // Measured: the dialog's height is its content's height, so the menu
        // stops clipping its own feedback field at 200% scale. The field is a
        // `Fixed` region the caller draws, at the place the measurer gave it.
        if self.menu_open {
            let width = 620.0 * ui.0;
            let x = (screen.w - width) / 2.0;
            hud::panel(&mut self.batch, &screen, 0.0, 0.0, screen.w, screen.h, Token::Desk);

            let field_h = pad(Space::Sm) * 2.0 + Step::Body.px(ui) as f32;
            let mut menu_blocks = vec![
                Block::Line {
                    face: Face::Body,
                    step: Step::Display,
                    text: "Paused".into(),
                    color: body_ink,
                },
                Block::Line {
                    face: Face::Mono,
                    step: Step::Small,
                    text: format!(
                        "{} · {} · {} interactions recorded · {} ticks simulated",
                        self.gov.contract,
                        self.world.clock.label(),
                        self.session.count(),
                        self.ticks_run
                    ),
                    color: muted,
                },
                Block::Gap(Space::Md),
            ];
            for item in [
                "Esc   resume",
                "F     leave feedback",
                "S     save the city",
                "O     load the last save",
                "Q     flush the capture",
                "U     interface scale",
                "F9    session console (the agent's ledger, beside the world)",
            ] {
                menu_blocks.push(Block::Line {
                    face: Face::Body,
                    step: Step::Body,
                    text: item.into(),
                    color: body_ink,
                });
            }
            menu_blocks.push(Block::Gap(Space::Md));
            let field_index = menu_blocks.len();
            menu_blocks.push(Block::Fixed { height: field_h });
            if self.feedback_focus {
                menu_blocks.push(Block::Gap(Space::Xs));
                menu_blocks.push(Block::Line {
                    face: Face::Body,
                    step: Step::Small,
                    text: "Enter writes the record · Esc backs out without writing".into(),
                    color: muted,
                });
            }
            let menu_frame = Frame::new(x + pad(Space::Lg), 0.0, width - 2.0 * pad(Space::Lg), screen.h);
            let mut measured = ui::measure(&mut self.text, ui, &menu_frame, &menu_blocks);
            let height = measured.content_height + 2.0 * pad(Space::Lg);
            let y = ((screen.h - height) / 2.0).max(pad(Space::Md));
            hud::panel(&mut self.batch, &screen, x, y, width, height, Token::PanelRaised);
            measured.frame.y = y + pad(Space::Lg);
            ui::paint(&measured, &ui::Scroll::new(), &mut self.batch, &screen, &mut self.text);

            // The feedback field, at its measured place. Feedback is its own
            // tier, always available, and its record says what it is: one
            // playtest, at tentative confidence.
            if let Some((field_top, field_hm)) = measured.region(field_index) {
                let fy = y + pad(Space::Lg) + field_top;
                hud::panel(
                    &mut self.batch,
                    &screen,
                    x + pad(Space::Lg),
                    fy,
                    width - 2.0 * pad(Space::Lg),
                    field_hm,
                    if self.feedback_focus { Token::Ink } else { Token::Panel },
                );
                let inner_w = width - 2.0 * pad(Space::Lg) - 2.0 * pad(Space::Sm);
                if self.feedback.is_empty() && !self.feedback_focus {
                    self.text.draw_step(
                        Face::Body,
                        &mut self.batch,
                        &screen,
                        x + pad(Space::Lg) + pad(Space::Sm),
                        fy + pad(Space::Sm),
                        Step::Small,
                        muted,
                        "press F to type feedback; Enter saves it into playtest/ with this session attached",
                    );
                } else {
                    let shown = hud::truncate(
                        &mut self.text,
                        Face::Body,
                        &self.feedback,
                        Step::Body,
                        inner_w,
                    );
                    self.text.draw_step(
                        Face::Body,
                        &mut self.batch,
                        &screen,
                        x + pad(Space::Lg) + pad(Space::Sm),
                        fy + pad(Space::Sm),
                        Step::Body,
                        hud::style(if self.feedback_focus { Token::TextOnInk } else { Token::TextBody })
                            .text
                            .unwrap_or(body_ink),
                        &shown,
                    );
                }
            }
        }

        // ---- help / the stage's own test script -----------------------------
        // Measured: the dialog's height is its content's height. The old 300
        // px panel ran its last lines below its own border at 1x.
        if self.show_help {
            let width = 700.0 * ui.0;
            let x = (screen.w - width) / 2.0;

            let lines = [
                ("H", "close this panel"),
                ("1 / 2 / 3 / 4 / 5", "road · zone · power · demolish · inspect"),
                ("R / C / I", "zone type while the zone tool is active"),
                ("drag", "lay a road leg, or paint a zone block"),
                ("right-drag", "orbit: free rotation and tilt"),
                ("middle-drag", "pan   ·   wheel: zoom   ·   Home: reset the view"),
                ("space / tab", "pause   ·   cycle 1x 2x 3x"),
                ("L / H", "ledger   ·   this panel"),
                ("Esc then U", "interface scale 100 / 125 / 150 / 200%"),
                ("F9", "session console: the agent's ledger, beside the world"),
                ("Esc then F", "leave feedback at any time; Enter writes it to playtest/C1/"),
            ];
            let key_col = 148.0 * ui.0;
            let mut help_blocks = vec![
                Block::Line {
                    face: Face::Body,
                    step: Step::Display,
                    text: "C1 — sim core".into(),
                    color: body_ink,
                },
                Block::Gap(Space::Sm),
            ];
            for (keys, what) in lines {
                help_blocks.push(Block::Keyed {
                    key: keys.into(),
                    text: what.into(),
                    step: Step::Small,
                    key_color: muted,
                    text_color: body_ink,
                    key_col,
                });
            }
            help_blocks.push(Block::Gap(Space::Sm));
            help_blocks.push(Block::Rule);
            help_blocks.push(Block::Gap(Space::Xs));
            help_blocks.push(Block::Line {
                face: Face::Body,
                step: Step::Small,
                text: "Everything you place files a ticket. The ticket closes only when the world".into(),
                color: muted,
            });
            help_blocks.push(Block::Line {
                face: Face::Body,
                step: Step::Small,
                text: "shows what it promised, or honestly as unverified when it does not.".into(),
                color: muted,
            });
            help_blocks.push(Block::Gap(Space::Sm));
            let adapter = self
                .gpu
                .as_ref()
                .map(|gpu| gpu.adapter_name.clone())
                .unwrap_or_else(|| "no device".to_string());
            help_blocks.push(Block::Line {
                face: Face::Mono,
                step: Step::Small,
                text: format!(
                    "{} city quads · body text on panels measures {:.2}:1 — measured, not asserted",
                    self.world_batch.count(),
                    hud::measured_body_on_panel()
                ),
                color: muted,
            });
            help_blocks.push(Block::Line {
                face: Face::Mono,
                step: Step::Small,
                text: format!("rendering on {adapter}"),
                color: muted,
            });

            let help_frame = Frame::new(x + pad(Space::Lg), 0.0, width - 2.0 * pad(Space::Lg), screen.h);
            let mut measured = ui::measure(&mut self.text, ui, &help_frame, &help_blocks);
            let height = measured.content_height + 2.0 * pad(Space::Lg);
            let y = ((screen.h - height) / 2.0).max(pad(Space::Md));
            hud::panel(&mut self.batch, &screen, x, y, width, height, Token::PanelRaised);
            measured.frame.y = y + pad(Space::Lg);
            ui::paint(&measured, &ui::Scroll::new(), &mut self.batch, &screen, &mut self.text);
        }
    }

    /// Where you are, and which way the world runs.
    ///
    /// With a free camera there is no fixed "up" on screen, so the axis
    /// directions are projected from world space into the interface and drawn
    /// as they actually fall. A north arrow bolted to the corner would be a
    /// decoration; this is a reading.
    fn draw_compass(&mut self, screen: &Screen) {
        // The scale the text was actually laid out at this frame, rather than a
        // second copy that could drift from it.
        let ui = self.text.ui_scale();
        let (ax, ay) = (screen.w - hud::space(Space::Xxl, ui) * 2.0, screen.h - hud::space(Space::Xxl, ui) * 2.0 - design::target(ui));
        let origin = Vec3::new(self.camera.focus.x, self.camera.focus.y, 0.0);
        let (ox, oy) = self.camera.world_to_screen(origin);
        let _ = (ox, oy);

        let arm = |direction: Vec3, colour: [f32; 4], screen: &Screen, batch: &mut Batcher| {
            let (ex, ey) = self.camera.world_to_screen(origin + direction);
            let (dx, dy) = (ex - ox, ey - oy);
            let length = (dx * dx + dy * dy).sqrt();
            if length < 1.0 {
                return;
            }
            let (ux, uy) = (dx / length, dy / length);
            let reach = hud::space(Space::Xxl, ui) * 1.6;
            let segments = 12;
            for step in 0..segments {
                let t = step as f32 / segments as f32;
                let alpha = (1.0 - t) * 0.9;
                let px = ax + ux * reach * t;
                let py = ay + uy * reach * t;
                let thickness = 3.0 * ui.0;
                // A segment is emitted as a small square, which at this length
                // is indistinguishable from a rotated line.
                batch.screen_rect(
                    screen,
                    px - thickness / 2.0,
                    py - thickness / 2.0,
                    thickness,
                    thickness,
                    hud::with_alpha(colour, alpha),
                    Text::solid_uv(),
                );
            }
        };

        let north = hud::style(Token::TextBody).text.unwrap_or([1.0; 4]);
        let east = hud::style(Token::TextMuted).text.unwrap_or([0.8, 0.8, 0.8, 1.0]);
        arm(Vec3::new(0.0, 1.0, 0.0), north, screen, &mut self.batch);
        arm(Vec3::new(1.0, 0.0, 0.0), east, screen, &mut self.batch);

        // The reading itself: where the cursor is, which way the city runs, and
        // the scale the interface is laid out at.
        let tile = self
            .camera
            .screen_to_tile(self.cursor.0, self.cursor.1)
            .map(|(x, y)| format!("{x}, {y}"))
            .unwrap_or_else(|| "off the map".to_string());
        let reading = format!(
            "N {:>3.0}°  ·  cursor {tile}  ·  {}",
            self.camera.bearing_degrees(),
            ui.label()
        );
        let width = self.text.measure_step(Face::Mono, &reading, Step::Small);
        hud::label_mono(
            &mut self.text,
            &mut self.batch,
            screen,
            ax - width + hud::space(Space::Xxl, ui) * 0.4,
            ay + hud::space(Space::Xxl, ui),
            Step::Small,
            Token::TextMuted,
            &reading,
        );
        hud::label(
            &mut self.text,
            &mut self.batch,
            screen,
            ax - hud::space(Space::Sm, ui),
            ay + hud::space(Space::Xxl, ui) - hud::space(Space::Xl, ui) - hud::space(Space::Md, ui),
            Step::Small,
            Token::TextBody,
            "N",
        );
        hud::label(
            &mut self.text,
            &mut self.batch,
            screen,
            ax + hud::space(Space::Md, ui),
            ay + hud::space(Space::Md, ui),
            Step::Small,
            Token::TextMuted,
            "E",
        );
    }

    fn draw(&mut self) {
        self.batch.clear();
        self.image_batch.clear();
        // The scale is applied once per frame, here, rather than carried
        // through every call site that draws text.
        self.text.set_ui_scale(self.ui);
        // Level-of-detail is driven by where the camera is looking, so a replay
        // with the same camera splits the same agents into the same cohorts.
        self.world.focus = (
            (self.camera.focus.x / TILE).floor() as i32,
            (self.camera.focus.y / TILE).floor() as i32,
        );
        self.draw_world();
        self.draw_hud();

        // Once, on the first frame: what the passes were actually asked to
        // draw. "It renders" is otherwise a claim with nothing behind it.
        if !self.logged_first_frame {
            self.logged_first_frame = true;
            tracing::info!(
                world_opaque = self.world_batch.opaque.len(),
                world_overlay = self.world_batch.overlay.len(),
                interface = self.batch.instances.len(),
                text_areas = self.text.pending_area_count(),
                yaw_degrees = self.camera.yaw.to_degrees(),
                pitch_degrees = self.camera.pitch.to_degrees(),
                zoom = self.camera.zoom,
                "first frame"
            );
        }

        let clear = hud::style(Token::Desk).fill.unwrap_or([0.1, 0.1, 0.1, 1.0]);
        let now = Instant::now();
        let seconds = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;
        let camera = self.camera;
        if let Some(gpu) = self.gpu.as_mut() {
            gpu.render(&self.world_batch, &self.batch, &self.image_batch, &camera, clear, seconds, &mut self.text);
        }
    }

    /// Every rectangle the player can hit, handed to the design check so a
    /// target that shrinks below the floor is a refusal rather than a surprise.
    fn interactive_targets(&self) -> Vec<Target> {
        let screen = self.screen();
        let mut targets = Vec::new();
        for (tool, rect) in toolbar_layout(&screen, self.ui) {
            let _ = tool;
            targets.push(Target {
                name: "tool button",
                w: rect.w,
                h: rect.h,
            });
        }
        for rect in menu_rows(&screen, self.ui) {
            targets.push(Target {
                name: "menu row",
                w: rect.w,
                h: rect.h,
            });
        }
        targets
    }

    /// Select a tool and say so. A re-select of the active tool is silent:
    /// a sound must indicate something of use (a62), and "you pressed 1 again"
    /// is not information.
    fn select_tool(&mut self, tool: Tool) {
        if self.tool != tool {
            self.tool = tool;
            self.play(Sound::Select);
        }
    }

    /// Scroll the ledger's list by a wheel delta. Clamped against the list's
    /// own measurement — the same blocks, frame and measure pass the draw
    /// uses, so scrolling and painting can never disagree about the content.
    fn scroll_ledger(&mut self, pixels: f32) {
        if !self.show_ledger || pixels == 0.0 {
            return;
        }
        let screen = self.screen();
        let (frame, blocks) = self.ledger_layout(&screen);
        let ui = self.text.ui_scale();
        let measured = ui::measure(&mut self.text, ui, &frame, &blocks);
        self.ledger_scroll.scroll_by(pixels, &measured);
    }

    /// The ledger panel's content: frame and blocks. One copy, shared by the
    /// draw pass and the wheel handler.
    fn ledger_layout(&mut self, screen: &Screen) -> (Frame, Vec<Block>) {
        let ui = self.text.ui_scale();
        let pad = |space: Space| hud::space(space, ui);
        let width = 470.0;
        let lx = screen.w - width - pad(Space::Md);
        let ly = pad(Space::Sm) * 2.0
            + Step::Small.px(ui) as f32 * ui::LINE_ADVANCE_FACTOR
            + pad(Space::Sm)
            + pad(Space::Xs);
        let lh = screen.bottom_anchor(160.0, 12.0) - ly;
        let frame = Frame::new(
            lx + pad(Space::Md),
            ly + pad(Space::Sm),
            width - 2.0 * pad(Space::Md),
            lh - 2.0 * pad(Space::Sm),
        );

        let muted = hud::style(Token::TextMuted).text.unwrap_or([0.8; 4]);
        let body_ink = hud::style(Token::TextBody).text.unwrap_or([1.0; 4]);
        let (open, closed) = self.gov.counts();
        let counts = format!(
            "open {open}   terminal {closed}   evidence {}   corrections {}   refusals {}",
            self.gov.evidence.len(),
            self.gov.corrections.len(),
            self.gov.denials
        );
        let mut blocks = vec![
            Block::Line {
                face: Face::Body,
                step: Step::Body,
                text: "Ticket ledger".into(),
                color: body_ink,
            },
            Block::Line {
                face: Face::Body,
                step: Step::Small,
                text: counts,
                color: muted,
            },
            Block::Rule,
            Block::Gap(Space::Xs),
        ];
        // Every ticket becomes a row. The old panel drew 22 and then
        // silently stopped — a ledger whose older entries simply did not
        // exist. The viewport, not a cap, decides what is visible.
        for ticket in self.gov.ledger(usize::MAX) {
            let line = format!("{}  {}", ticket.id, ticket.objective);
            let clipped =
                hud::truncate(&mut self.text, Face::Body, &line, Step::Small, width - 130.0);
            let token = if ticket.is_closed() {
                match ticket.terminal {
                    Some(reason) if reason.is_validated() => Token::Verified,
                    Some(ala_cities::gov::RetirementReason::CompletedButUnverified) => Token::Warning,
                    Some(ala_cities::gov::RetirementReason::CompletedWithKnownRegression) => Token::Refused,
                    Some(ala_cities::gov::RetirementReason::BlockedAndClosed) => Token::CaseOpen,
                    _ => Token::NotObtained,
                }
            } else {
                Token::TextBody
            };
            let closing = ticket.closing_line();
            // Truncated before the Row block is built: the text atlas is
            // borrowed mutably to build the string, and again to draw it,
            // which cannot overlap.
            let clipped_closing =
                hud::truncate(&mut self.text, Face::Mono, &closing, Step::Small, 160.0);
            blocks.push(Block::Row {
                label: clipped,
                value: clipped_closing,
                step: Step::Small,
                color: hud::style(token).text.unwrap_or(body_ink),
                value_color: muted,
            });
        }
        (frame, blocks)
    }

    /// Scroll the console's report by a wheel delta, clamped against the
    /// same measurement the draw pass uses — the ledger's discipline, kept.
    fn scroll_console(&mut self, pixels: f32) {
        if !self.console_open || pixels == 0.0 {
            return;
        }
        let screen = self.screen();
        let (frame, blocks) = self.console_layout(&screen);
        let ui = self.text.ui_scale();
        let measured = ui::measure(&mut self.text, ui, &frame, &blocks);
        self.console_scroll.scroll_by(pixels, &measured);
    }

    /// The console pane's content: frame and blocks. One copy, shared by the
    /// draw pass and the wheel handler. Read-only by construction (Q127):
    /// every fact here comes from a record or a query, and nothing in this
    /// panel writes anything — the agent process is the ledger's only author.
    fn console_layout(&mut self, screen: &Screen) -> (Frame, Vec<Block>) {
        let ui = self.text.ui_scale();
        let pad = |space: Space| hud::space(space, ui);
        let width = 470.0;
        let lx = screen.w - width - pad(Space::Md);
        let ly = pad(Space::Sm) * 2.0
            + Step::Small.px(ui) as f32 * ui::LINE_ADVANCE_FACTOR
            + pad(Space::Sm)
            + pad(Space::Xs);
        let lh = screen.bottom_anchor(160.0, 12.0) - ly;
        let frame = Frame::new(
            lx + pad(Space::Md),
            ly + pad(Space::Sm),
            width - 2.0 * pad(Space::Md),
            lh - 2.0 * pad(Space::Sm),
        );

        /// Age of the last agent record, in the pane's words. A display
        /// concern, kept out of the ledger store (a73's split of labour).
        fn age_line(seconds: u64) -> String {
            if seconds < 90 {
                format!("{seconds}s ago")
            } else if seconds < 3600 {
                format!("{}m ago", seconds / 60)
            } else {
                format!("{}h ago", seconds / 3600)
            }
        }

        let muted = hud::style(Token::TextMuted).text.unwrap_or([0.8; 4]);
        let body_ink = hud::style(Token::TextBody).text.unwrap_or([1.0; 4]);
        let row = |label: &str, value: String| Block::Row {
            label: label.into(),
            value,
            step: Step::Small,
            color: body_ink,
            value_color: muted,
        };
        let small = |text: String| Block::Line {
            face: Face::Body,
            step: Step::Small,
            text,
            color: muted,
        };

        let build = &self.build_info;
        let (open, closed) = self.gov.counts();
        let mut blocks = vec![
            Block::Line {
                face: Face::Body,
                step: Step::Body,
                text: "Debug console".into(),
                color: body_ink,
            },
            // Each fact its own row, each absence named — "unknown" is a
            // fact too (a76), never a blank and never a guess.
            row("Build", build.commit.clone().unwrap_or_else(|| "unknown".into())),
            row("Branch", build.branch.clone().unwrap_or_else(|| "unknown".into())),
            row(
                "Tree",
                match build.dirty_files {
                    Some(0) => "clean".into(),
                    Some(n) => format!("{n} uncommitted"),
                    None => "unknown".into(),
                },
            ),
            small(build.release_label()),
            row("Tickets", format!("open {open} · terminal {closed}")),
            Block::Rule,
            Block::Gap(Space::Xs),
        ];

        blocks.push(Block::Line {
            face: Face::Body,
            step: Step::Body,
            text: "Session faults".into(),
            color: body_ink,
        });
        if self.console_faults.is_empty() {
            blocks.push(small(
                "none reported — KeyD reports the debugger itself".into(),
            ));
        } else {
            for note in self.console_faults.iter().cloned() {
                blocks.push(small(note));
            }
        }
        blocks.push(Block::Gap(Space::Xs));

        blocks.push(Block::Line {
            face: Face::Body,
            step: Step::Body,
            text: "Agent ledger".into(),
            color: body_ink,
        });
        let path = std::path::PathBuf::from("saves")
            .join("season_2026_s1")
            .join(ala_cities::agentledger::LEDGER_FILE);
        match ala_cities::agentledger::presence(&path, ala_cities::agentledger::now_unix_ms()) {
            ala_cities::agentledger::Presence::NoRecord => {
                blocks.push(small("no agent activity recorded".into()));
            }
            ala_cities::agentledger::Presence::Last {
                entry,
                age_seconds,
            } => {
                blocks.push(row(
                    "Agent",
                    format!("{} · {}", entry.agent, entry.action.token()),
                ));
                blocks.push(small(format!("{} — {}", entry.ticket, entry.detail)));
                // The claim and its age, together (a73): the pane reports
                // what a record says and how long ago it said it.
                blocks.push(small(age_line(age_seconds)));
            }
        }
        (frame, blocks)
    }
}

/// Where the tool buttons live. One function, used by both the drawing code and
/// the hit test, so a button cannot be drawn in one place and clickable in
/// another.
fn toolbar_layout(screen: &Screen, ui: UiScale) -> Vec<(Tool, Rect)> {
    let height = design::target(ui) + hud::space(Space::Sm, ui);
    let width = 150.0 * ui.0;
    let gap = hud::space(Space::Xs, ui);
    let tools = Tool::all();
    let total = tools.len() as f32 * width + (tools.len() as f32 - 1.0) * gap;
    let start = (screen.w - total) / 2.0;
    let y = screen.bottom_anchor(height, hud::space(Space::Md, ui));
    tools
        .iter()
        .enumerate()
        .map(|(index, tool)| {
            (
                *tool,
                Rect {
                    x: start + index as f32 * (width + gap),
                    y,
                    w: width,
                    h: height,
                },
            )
        })
        .collect()
}

fn toolbar_hit(screen: &Screen, ui: UiScale, px: f32, py: f32) -> Option<Tool> {
    toolbar_layout(screen, ui)
        .into_iter()
        .find(|(_, rect)| hud::hit(rect.x, rect.y, rect.w, rect.h, px, py))
        .map(|(tool, _)| tool)
}

/// The rows of the pause menu, which is the only other surface with targets on
/// it. Sized from the scale for the same reason the toolbar is.
fn menu_rows(screen: &Screen, ui: UiScale) -> Vec<Rect> {
    let width = 520.0 * ui.0;
    let height = 300.0 * ui.0;
    let x = (screen.w - width) / 2.0;
    let y = (screen.h - height) / 2.0;
    let row = design::target(ui);
    (0..6)
        .map(|index| Rect {
            x: x + hud::space(Space::Xl, ui),
            y: y + hud::space(Space::Xl, ui) + index as f32 * (row + hud::space(Space::Xs, ui)),
            w: width - hud::space(Space::Xl, ui) * 2.0,
            h: row,
        })
        .collect()
}

/// How tall a building stands, in world units.
///
/// Level gives the storeys; the kind gives the shape a city actually has — a
/// plant is tall and thin, a shop is low and wide — and under construction a
/// building is a stub, so scaffolding is legible as *unfinished* rather than as
/// a smaller building.
fn building_height(b: &ala_cities::sim::Building, tick: u64) -> f32 {
    let kind = match b.kind {
        BuildingKind::Home => 1.0,
        BuildingKind::Shop => 0.9,
        BuildingKind::Factory => 1.3,
        BuildingKind::PowerPlant => 2.1,
    };
    let storeys = if tick < b.ready_tick {
        0.45
    } else {
        b.level.max(1) as f32
    };
    storeys * kind * LEVEL_HEIGHT
}

/// Where a building's shadow lands, from the same light that shades its faces.
fn shadow_reach(height: f32) -> (f32, f32) {
    let light = render::LIGHT.normalize();
    if light.z.abs() < 1e-3 {
        return (0.0, 0.0);
    }
    ((-light.x / light.z) * height, (-light.y / light.z) * height)
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }
        let attributes = Window::default_attributes()
            .with_title("ala-cities — C1")
            .with_inner_size(winit::dpi::LogicalSize::new(1600.0, 900.0));
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("a window"),
        );
        // Logged, not assumed: every UI coordinate in this build is a physical
        // pixel, so on a display with a scale factor above 1 the whole chrome
        // renders smaller than it was designed. Stating the number here is what
        // makes that visible instead of arguable.
        let scale = window.scale_factor();
        tracing::info!(
            scale_factor = scale,
            physical = ?window.inner_size(),
            logical_body_px = Step::Body.logical_px() / scale as f32,
            logical_small_px = Step::Small.logical_px() / scale as f32,
            "interface scale"
        );
        let gpu = Gpu::new(window, &self.text);
        self.camera.screen = gpu.screen();
        self.camera.zoom = (gpu.screen().h / (MAP as f32 * TILE) * 2.4).max(0.2);
        self.gpu = Some(gpu);

        // Icons a decision promoted. A load failure is the pipeline's defect
        // (a decided icon whose render is missing), stated and carried as an
        // empty set — text toolbar, no invented placeholder art.
        let (set, atlas, width, height) =
            match icons::load(
                std::path::Path::new(ala_cities::iconreview::DECISIONS),
                std::path::Path::new(ala_cities::iconreview::REVIEW_JSON),
                std::path::Path::new(ala_cities::iconreview::ASSETS),
            ) {
                Ok(loaded) => loaded,
                Err(defects) => {
                    tracing::error!(%defects, "icon set failed to load; the toolbar stays text-only");
                    (IconSet::default(), Vec::new(), 0, 0)
                }
            };
        if set.is_empty() {
            tracing::info!(
                "no icons decided yet; the toolbar draws text until the picker promotes one"
            );
        } else {
            tracing::info!(count = set.len(), "decided icons loaded");
        }
        if let Some(gpu) = self.gpu.as_mut() {
            if !atlas.is_empty() {
                gpu.bind_image(&atlas, width, height);
            }
        }
        self.icon_set = set;

        // Audio output. None when there is no device: silent is honest.
        self.audio = audio::Player::new();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if self.gpu.is_none() {
            return;
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(gpu) = self.gpu.as_mut() {
                    gpu.resize(size.width, size.height);
                    self.camera.screen = gpu.screen();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = (position.x as f32, position.y as f32);
                let (dx, dy) = (
                    self.cursor.0 - self.last_cursor.0,
                    self.cursor.1 - self.last_cursor.1,
                );
                if self.orbiting {
                    self.camera.orbit(dx, dy);
                    self.last_cursor = self.cursor;
                } else if self.panning {
                    self.camera.pan(dx, dy);
                    self.last_cursor = self.cursor;
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                // Toolbar clicks are handled before anything reaches the world.
                if button == MouseButton::Left
                    && state == ElementState::Pressed
                    && !self.menu_open
                {
                    if let Some(tool) =
                        toolbar_hit(&self.screen(), self.ui, self.cursor.0, self.cursor.1)
                    {
                        self.tool = tool;
                        return;
                    }
                }
                self.on_click(state == ElementState::Pressed, button);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                // The open console owns the wheel first — it is the topmost
                // right-rail surface — then the ledger, and the camera never
                // sees the event while either is visible. Closed, the wheel
                // zooms as before.
                if self.console_open {
                    let pixels = match delta {
                        MouseScrollDelta::LineDelta(_, y) => y * 40.0,
                        MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                    };
                    self.scroll_console(pixels);
                } else if self.show_ledger {
                    let pixels = match delta {
                        MouseScrollDelta::LineDelta(_, y) => y * 40.0,
                        MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                    };
                    self.scroll_ledger(pixels);
                } else {
                    let factor = match delta {
                        MouseScrollDelta::LineDelta(_, y) => 1.0 + y * 0.1,
                        MouseScrollDelta::PixelDelta(pos) => 1.0 + pos.y as f32 * 0.01,
                    };
                    self.camera.zoom_by(factor, self.cursor);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let text = event.text.as_ref().map(|t| t.to_string());
                if let PhysicalKey::Code(code) = event.physical_key {
                    self.on_key(code, event.state == ElementState::Pressed, text);
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let seconds = now.duration_since(self.last_frame).as_secs_f32();
                self.advance(seconds);
                self.camera.clamp_to(MAP, MAP);
                self.draw();
            }
            _ => {}
        }
    }



    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(gpu) = self.gpu.as_ref() {
            gpu.window.request_redraw();
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        // A clean exit saves the world; a crash does not need to, because the
        // autosave has already been running.
        self.autosave();
        if let Ok(count) = self.session.flush() {
            tracing::info!(events = count, path = %self.session.path().display(), "capture written");
        }
        let (open, closed) = self.gov.counts();
        tracing::info!(
            tickets = self.gov.tickets.len(),
            open,
            closed,
            evidence = self.gov.evidence.len(),
            corrections = self.gov.corrections.len(),
            refusals = self.gov.denials,
            "ledger on exit"
        );
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let mut app = match App::new() {
        Ok(app) => app,
        Err(err) => {
            eprintln!("could not start: {err}");
            std::process::exit(1);
        }
    };

    // The mechanical half of the design check, logged at startup. Judgement
    // items are named as open rather than folded in with it.
    let targets = app.interactive_targets();
    let ui = app.ui;
    for line in design::audit(&targets, ui) {
        tracing::info!("design: {line}");
    }

    // And then it **fails closed**, like the style table does. A scale that is
    // allowed to drift is the defect this check exists to prevent, and a check
    // that only prints is a check nobody reads.
    let mut defects = design::verify(&targets, ui);
    // The check the scale gate cannot make on its own, because it holds no font: that
    // each step *renders* at the size it declares. The build that shipped the
    // unreadable text passed every declared-size check it was ever given — the scale
    // said 16 px and the screen drew 12, and every measurement agreed with the scale.
    defects.extend(app.text.scale_defects());
    if !defects.is_empty() {
        eprintln!("the design check failed closed:");
        for defect in &defects {
            eprintln!("  - {defect}");
        }
        eprintln!("fix these and restart; nothing will be drawn until the scales hold.");
        std::process::exit(1);
    }

    // The material gate, in the same shape: every line printed, every defect named,
    // and a defect refuses the start. The world's materials are claims — this part
    // is this material, and its colour arrives by this mechanism — so a table that
    // does not hold is not a table to draw the city from.
    let materials = materials::verify();
    for line in &materials.lines {
        tracing::info!("{line}");
    }
    if !materials.ok() {
        eprintln!("the material check failed closed:");
        for defect in &materials.defects {
            eprintln!("  - {defect}");
        }
        eprintln!("fix the declaration and re-run `python tools/materials/emit.py`.");
        std::process::exit(1);
    }

    if app.gov.governor.load_state == ala_cities::gov::LoadState::FailedClosed {
        eprintln!(
            "the governor failed closed: {}\nno build operations will be permitted. Fix {} and restart.",
            app.gov.governor.reason,
            app.gov.governor.path.display()
        );
    }

    let event_loop = EventLoop::new().expect("an event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).expect("the event loop ran");
}
