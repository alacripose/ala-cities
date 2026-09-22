//! The record of what was authorised, attempted, observed, and retired.
//!
//! Three ideas do the work:
//!
//! * **The governor decides before the world changes.** A verdict carries the
//!   reason it was reached, and a refusal names the operations that *would* have
//!   been accepted. A refusal you can act on is a different thing from a dead
//!   end.
//! * **Validation reads the world back.** A ticket is never closed as validated
//!   because the click that filed it returned. The road is checked for in the
//!   road graph, afterwards, from the world's own state.
//! * **Closing execution does not erase history.** A retired ticket keeps its
//!   record; a wrong record gets a correction appended beside it; a demolished
//!   building is retired, not deleted.
//!
//! The board is also required to **drain**. Every ticket reaches a terminal
//! state within a bounded number of sim-seconds, and when the world did not
//! show the expected result, it closes as *unverified* — never as validated.
//! "Cannot stall" must never be implemented as "auto-promote to success".

pub mod season;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::sim::citizen::CitizenState;
use crate::sim::{BuildingKind, World, Zone, SIM_HZ};

pub use season::Season;

// ---------------------------------------------------------------------------
// Retirement
// ---------------------------------------------------------------------------

/// Why a piece of work stopped being active. These are not interchangeable:
/// "we did it and checked" and "we stopped without checking" are different
/// facts, and collapsing them is how a record starts lying.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetirementReason {
    CompletedAndValidated,
    CompletedWithKnownRegression,
    CompletedButUnverified,
    CancelledByAuthority,
    Superseded,
    Invalidated,
    BlockedAndClosed,
    Abandoned,
    Duplicate,
    Obsolete,
    ObjectiveWithdrawn,
    AuthorizationExpired,
    SafetyStop,
    SystemShutdown,
}

impl RetirementReason {
    pub fn token(self) -> &'static str {
        match self {
            RetirementReason::CompletedAndValidated => "completed_and_validated",
            RetirementReason::CompletedWithKnownRegression => "completed_with_known_regression",
            RetirementReason::CompletedButUnverified => "completed_but_unverified",
            RetirementReason::CancelledByAuthority => "cancelled_by_authority",
            RetirementReason::Superseded => "superseded",
            RetirementReason::Invalidated => "invalidated",
            RetirementReason::BlockedAndClosed => "blocked_and_closed",
            RetirementReason::Abandoned => "abandoned",
            RetirementReason::Duplicate => "duplicate",
            RetirementReason::Obsolete => "obsolete",
            RetirementReason::ObjectiveWithdrawn => "objective_withdrawn",
            RetirementReason::AuthorizationExpired => "authorization_expired",
            RetirementReason::SafetyStop => "safety_stop",
            RetirementReason::SystemShutdown => "system_shutdown",
        }
    }

    pub fn is_validated(self) -> bool {
        matches!(self, RetirementReason::CompletedAndValidated)
    }
}

// ---------------------------------------------------------------------------
// Operations and the governor
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Op {
    BuildRoad { distance: u32 },
    Zone { tiles: u32 },
    PlaceService { kind: BuildingKind },
    Demolish { tiles: u32 },
    SetTax { zone: Zone, rate: f32 },
    TakeLoan { amount: i64 },
}

/// The complete operation vocabulary this build knows. A name outside it is
/// refused at the moment it is proposed, and the refusal quotes this list.
///
/// The two observing operations are part of the vocabulary on purpose: a
/// read-only profile that could not even look would be a profile with no way to
/// describe itself, and `look`/`inspect` are exactly what a narrow profile is
/// permitted to keep.
pub const ALL_OPS: [&str; 8] = [
    "build_road",
    "zone",
    "place_service",
    "demolish",
    "set_tax",
    "take_loan",
    "look",
    "inspect",
];

/// Profiles this build knows the meaning of. An unknown profile in the file is
/// refused rather than guessed at.
pub const KNOWN_PROFILES: [&str; 2] = ["default", "read_only"];

impl Op {
    pub fn name(&self) -> &'static str {
        match self {
            Op::BuildRoad { .. } => "build_road",
            Op::Zone { .. } => "zone",
            Op::PlaceService { .. } => "place_service",
            Op::Demolish { .. } => "demolish",
            Op::SetTax { .. } => "set_tax",
            Op::TakeLoan { .. } => "take_loan",
        }
    }

    pub fn describe(&self) -> String {
        match self {
            Op::BuildRoad { distance } => format!("lay {distance} tiles of road"),
            Op::Zone { tiles } => format!("zone {tiles} tiles"),
            Op::PlaceService { kind } => format!("place a {}", kind.name()),
            Op::Demolish { tiles } => format!("demolish {tiles} tiles"),
            Op::SetTax { zone, rate } => {
                format!("set {} tax to {:.1}%", zone.name(), rate * 100.0)
            }
            Op::TakeLoan { amount } => format!("take a loan of {amount} credits"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Verdict {
    pub allowed: bool,
    pub reason: String,
    pub governor_version: u32,
    pub profile: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadState {
    Parsed,
    FailedClosed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GovernorFile {
    version: u32,
    profile: String,
    allowed_ops: Vec<String>,
    #[serde(default)]
    max_road_leg: u32,
    #[serde(default)]
    max_loan: i64,
    #[serde(default)]
    notes: String,
}

#[derive(Clone, Debug)]
pub struct Governor {
    pub version: u32,
    pub profile: String,
    pub allowed_ops: Vec<String>,
    pub max_road_leg: u32,
    pub max_loan: i64,
    pub load_state: LoadState,
    /// Operations named in the file that this build does not know. Dropped, and
    /// counted, rather than silently honoured.
    pub dropped_ops: Vec<String>,
    pub path: PathBuf,
    pub reason: String,
}

impl Governor {
    /// Read the governor. **A governance layer must not fail open**: a file that
    /// will not parse is a refusal, never a quiet fall back to the permissive
    /// default, because a permissive default that nobody knows about is worse
    /// than no governor at all.
    pub fn load(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_path_buf();
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(err) => {
                return Self {
                    version: 0,
                    profile: "none".to_string(),
                    allowed_ops: Vec::new(),
                    max_road_leg: 0,
                    max_loan: 0,
                    load_state: LoadState::FailedClosed,
                    dropped_ops: Vec::new(),
                    path,
                    reason: format!("the governor file could not be read: {err}"),
                }
            }
        };

        let file: GovernorFile = match serde_json::from_str(&text) {
            Ok(file) => file,
            Err(err) => {
                return Self {
                    version: 0,
                    profile: "none".to_string(),
                    allowed_ops: Vec::new(),
                    max_road_leg: 0,
                    max_loan: 0,
                    load_state: LoadState::FailedClosed,
                    dropped_ops: Vec::new(),
                    path,
                    reason: format!("the governor file did not parse: {err}"),
                }
            }
        };

        if !KNOWN_PROFILES.contains(&file.profile.as_str()) {
            return Self {
                version: file.version,
                profile: file.profile.clone(),
                allowed_ops: Vec::new(),
                max_road_leg: 0,
                max_loan: 0,
                load_state: LoadState::FailedClosed,
                dropped_ops: Vec::new(),
                path,
                reason: format!(
                    "unknown profile `{}`; this build knows {}",
                    file.profile,
                    KNOWN_PROFILES.join(", ")
                ),
            };
        }

        let mut allowed_ops = Vec::new();
        let mut dropped_ops = Vec::new();
        for op in &file.allowed_ops {
            if ALL_OPS.contains(&op.as_str()) {
                allowed_ops.push(op.clone());
            } else {
                dropped_ops.push(op.clone());
            }
        }

        let notes = if file.notes.is_empty() {
            String::new()
        } else {
            format!(" — {}", file.notes)
        };
        let reason = format!(
            "permitted by governor v{} ({} profile){}",
            file.version, file.profile, notes
        );

        Self {
            version: file.version,
            profile: file.profile,
            allowed_ops,
            max_road_leg: file.max_road_leg,
            max_loan: file.max_loan,
            load_state: LoadState::Parsed,
            dropped_ops,
            path,
            reason,
        }
    }

    pub fn authorize(&self, op: &Op) -> Verdict {
        let deny = |reason: String| Verdict {
            allowed: false,
            reason,
            governor_version: self.version,
            profile: self.profile.clone(),
        };

        if self.load_state == LoadState::FailedClosed {
            return deny(format!(
                "{}: no operations are permitted while the governor is unreadable",
                self.reason
            ));
        }

        let name = op.name();
        if !self.allowed_ops.iter().any(|allowed| allowed == name) {
            let known = if self.allowed_ops.is_empty() {
                "nothing".to_string()
            } else {
                self.allowed_ops.join(", ")
            };
            return deny(format!(
                "`{name}` is not permitted by the {} profile; operations it does permit: {known}",
                self.profile
            ));
        }

        match op {
            Op::BuildRoad { distance } => {
                if *distance == 0 {
                    return deny(
                        "a road leg of zero tiles asks the city to build nothing".to_string(),
                    );
                }
                if *distance > self.max_road_leg {
                    return deny(format!(
                        "a leg of {distance} tiles exceeds the {}-tile limit this profile permits",
                        self.max_road_leg
                    ));
                }
            }
            Op::SetTax { rate, .. } => {
                if !(0.0..=0.20).contains(rate) {
                    return deny(format!(
                        "a tax rate of {:.1}% is outside the 0-20% this profile permits",
                        rate * 100.0
                    ));
                }
            }
            Op::TakeLoan { amount } => {
                if *amount <= 0 {
                    return deny("a loan of nothing is not a loan".to_string());
                }
                if *amount > self.max_loan {
                    return deny(format!(
                        "a loan of {amount} credits exceeds the {} this profile permits",
                        self.max_loan
                    ));
                }
            }
            Op::Demolish { tiles }
                if *tiles == 0 => {
                    return deny("demolishing nothing changes nothing".to_string());
                }
            _ => {}
        }

        Verdict {
            allowed: true,
            reason: self.reason.clone(),
            governor_version: self.version,
            profile: self.profile.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Tickets
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TicketKind {
    Build,
    Case,
    Intervention,
}

impl TicketKind {
    pub fn prefix(self) -> &'static str {
        match self {
            TicketKind::Build => "BLD",
            TicketKind::Case => "CSE",
            TicketKind::Intervention => "INT",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TicketStatus {
    Queued,
    Open,
    Active,
    Blocked,
    Review,
    Complete,
    Rejected,
    Handled,
    Dropped,
}

impl TicketStatus {
    pub fn name(self) -> &'static str {
        match self {
            TicketStatus::Queued => "queued",
            TicketStatus::Open => "open",
            TicketStatus::Active => "active",
            TicketStatus::Blocked => "blocked",
            TicketStatus::Review => "review",
            TicketStatus::Complete => "complete",
            TicketStatus::Rejected => "rejected",
            TicketStatus::Handled => "handled",
            TicketStatus::Dropped => "dropped",
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            TicketStatus::Complete
                | TicketStatus::Rejected
                | TicketStatus::Handled
                | TicketStatus::Dropped
                | TicketStatus::Blocked
        )
    }
}

/// What the world must show before this ticket may be called validated. Read
/// back from the world, never inferred from the request that filed it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Expectation {
    RoadTiles(Vec<u32>),
    ZonedTiles(Vec<(u32, Zone)>),
    BuildingAt(u32),
    Demolished(u32),
    None,
}

impl Expectation {
    /// The tiles this expectation names.
    ///
    /// Used by the verifier to answer the question a stale claim raises: did a
    /// later recorded action explain this change, or did the world move with
    /// nothing on the record to account for it? Those are different findings
    /// and only one of them is a defect.
    pub fn tiles(&self) -> Vec<u32> {
        match self {
            Expectation::RoadTiles(tiles) => tiles.clone(),
            Expectation::ZonedTiles(tiles) => tiles.iter().map(|(tile, _)| *tile).collect(),
            Expectation::BuildingAt(tile) | Expectation::Demolished(tile) => vec![*tile],
            Expectation::None => Vec::new(),
        }
    }

    pub fn describe(&self) -> String {
        match self {
            Expectation::RoadTiles(tiles) => format!("{} tiles are road", tiles.len()),
            Expectation::ZonedTiles(tiles) => format!("{} tiles carry the zone set", tiles.len()),
            Expectation::BuildingAt(tile) => format!("a structure stands on tile {tile}"),
            Expectation::Demolished(tile) => format!("tile {tile} is clear"),
            Expectation::None => "no world expectation (nothing to read back)".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ticket {
    pub id: String,
    pub contract: String,
    pub kind: TicketKind,
    pub status: TicketStatus,
    pub objective: String,
    /// The gate, stated up front from the objective's own acceptance text, so
    /// nobody has to guess afterwards what "done" was going to mean.
    pub gate: String,
    pub opened_tick: u64,
    pub closed_tick: Option<u64>,
    pub terminal: Option<RetirementReason>,
    pub supersedes: Option<String>,
    pub governor_version: u32,
    pub case_key: Option<String>,
    /// For a sampled case: how many citizens this single ticket represents.
    pub count: u32,
    pub expectation: Expectation,
    pub roads_before: Vec<u32>,
    pub evidence: Vec<String>,
    pub actor: String,
    /// Set when a correction has been appended beside this record. Never a
    /// mutation of `terminal`.
    #[serde(default)]
    pub corrected: Option<String>,
}

impl Ticket {
    pub fn is_closed(&self) -> bool {
        self.closed_tick.is_some() || self.status.is_terminal()
    }

    pub fn age_seconds(&self, tick: u64) -> f64 {
        (tick.saturating_sub(self.opened_tick)) as f64 / SIM_HZ as f64
    }

    /// What the closing line reads on the ledger.
    pub fn closing_line(&self) -> String {
        match (self.terminal, self.corrected.as_ref()) {
            (Some(reason), Some(correction)) => format!("{} (corrected: {correction})", reason.token()),
            (Some(reason), None) => reason.token().to_string(),
            (None, _) => self.status.name().to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    pub ticket: String,
    /// `A` external oracle · `B` read back from world state · `C` self-report.
    pub grade: String,
    pub kind: String,
    pub detail: String,
    pub tick: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Retirement {
    pub id: String,
    pub ticket: String,
    pub contract: String,
    pub terminal_state: String,
    pub retirement_reason: String,
    pub objective: String,
    pub completion_claim: String,
    pub grounding: Grounding,
    pub verification: Verification,
    pub evidence: Vec<String>,
    pub governor_version: u32,
    pub retired_at_tick: u64,
    pub retired_by: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Grounding {
    pub inputs_verified: bool,
    pub note: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Verification {
    pub acceptance_criteria: Vec<String>,
    pub checks_run: Vec<String>,
    pub passed: bool,
    pub regressions_detected: bool,
}

/// An append-only correction. The original survives beside it: immutability
/// means "no silent edits", and a correction is the opposite of silent.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Correction {
    pub id: String,
    pub target: String,
    pub reason: String,
    pub old_value: String,
    pub new_value: String,
    pub evidence: Vec<String>,
    pub authorized_by: String,
    pub appended_at_tick: u64,
}

// ---------------------------------------------------------------------------
// Case sampling
// ---------------------------------------------------------------------------

pub const DISTRICT_TILES: u32 = 32;

pub fn district_of(width: u32, tile: u32) -> u32 {
    let x = tile % width;
    let y = tile / width;
    let districts_across = width.div_ceil(DISTRICT_TILES);
    (x / DISTRICT_TILES) + (y / DISTRICT_TILES) * districts_across
}

#[derive(Clone, Debug, PartialEq)]
pub struct CaseFinding {
    pub key: String,
    pub district: u32,
    pub count: u32,
    pub objective: String,
}

/// Sample population behaviour into findings.
///
/// This is the engine that keeps the queue alive: it samples *behaviour*, but
/// it dedupes by case identity, so four hundred hungry citizens in one district
/// are **one** case reading `400` rather than four hundred tickets. The count
/// is real, not decorative.
pub fn sample_cases(world: &World) -> Vec<CaseFinding> {
    use std::collections::BTreeMap;

    let mut unpowered: BTreeMap<u32, u32> = BTreeMap::new();
    for tile in 0..world.tiles.len() as u32 {
        let t = world.tile(tile);
        if t.zone != Zone::None && !t.powered {
            *unpowered.entry(district_of(world.width, tile)).or_default() += 1;
        }
    }

    let mut jobless: BTreeMap<u32, u32> = BTreeMap::new();
    let mut long_commute: BTreeMap<u32, u32> = BTreeMap::new();
    for citizen in &world.citizens {
        let Some(home) = citizen.home else { continue };
        let Some(building) = world.buildings.get(home as usize) else {
            continue;
        };
        let district = district_of(world.width, building.tile);
        match citizen.state {
            CitizenState::Unemployed => *jobless.entry(district).or_default() += 1,
            state if state.is_commuting() && citizen.path.len() > 24 => {
                *long_commute.entry(district).or_default() += 1
            }
            _ => {}
        }
    }

    let mut findings = Vec::new();
    for (district, count) in unpowered {
        if count >= 4 {
            findings.push(CaseFinding {
                key: format!("d{district}:unpowered"),
                district,
                count,
                objective: format!(
                    "district {district}: {count} zoned tiles have no power; growth has stopped on them"
                ),
            });
        }
    }
    for (district, count) in jobless {
        if count >= 8 {
            findings.push(CaseFinding {
                key: format!("d{district}:jobless"),
                district,
                count,
                objective: format!(
                    "district {district}: {count} residents have no job within reach"
                ),
            });
        }
    }
    for (district, count) in long_commute {
        if count >= 4 {
            findings.push(CaseFinding {
                key: format!("d{district}:long_commute"),
                district,
                count,
                objective: format!(
                    "district {district}: {count} residents commute more than 24 tiles; the road network does not connect where people live to where they work"
                ),
            });
        }
    }
    findings
}

// ---------------------------------------------------------------------------
// The government: governor + season + ticket population
// ---------------------------------------------------------------------------

pub struct Government {
    pub governor: Governor,
    pub season: Season,
    pub contract: String,
    /// The stall bound, in sim-seconds. Expressed in sim time so it holds
    /// identically at 1x, 2x and 3x.
    pub bound_sim_seconds: f64,
    pub tickets: Vec<Ticket>,
    pub retirements: Vec<Retirement>,
    pub evidence: Vec<Evidence>,
    pub corrections: Vec<Correction>,
    pub denials: u64,
    pub refused_capture: bool,
}

impl Government {
    pub fn open(
        saves_root: impl AsRef<Path>,
        season_id: &str,
        governor_path: impl AsRef<Path>,
    ) -> std::io::Result<Self> {
        let governor = Governor::load(governor_path);
        let season = Season::open(saves_root, season_id, "the city's own record")?;
        let tickets = season.read_tickets();
        let mut government = Self {
            bound_sim_seconds: 30.0,
            retirements: season.read_retirements(),
            evidence: season.read_evidence(),
            corrections: season.read_corrections(),
            contract: "CTR-CITY-0001".to_string(),
            governor,
            season,
            tickets,
            denials: 0,
            refused_capture: false,
        };
        government.apply_retirements();
        Ok(government)
    }

    /// Open the record for reading only: nothing is created, nothing is
    /// appended, no id is handed out. Reading history confers nothing, which is
    /// exactly why the verifier uses this and the game uses [`Self::open`].
    pub fn open_read_only(
        saves_root: impl AsRef<Path>,
        season_id: &str,
        governor_path: impl AsRef<Path>,
    ) -> Self {
        let governor = Governor::load(governor_path);
        let season = Season::attach(saves_root, season_id);
        let tickets = season.read_tickets();
        let retirements = season.read_retirements();
        let evidence = season.read_evidence();
        let corrections = season.read_corrections();
        let mut government = Self {
            bound_sim_seconds: 30.0,
            contract: "CTR-CITY-0001".to_string(),
            governor,
            season,
            tickets,
            retirements,
            evidence,
            corrections,
            denials: 0,
            refused_capture: false,
        };
        government.apply_retirements();
        government
    }

    /// Terminal states live in the *retirement* records, not in a rewritten
    /// ticket file. This is what an append-only ledger looks like when it is
    /// read back: filing and closing are two different records.
    fn apply_retirements(&mut self) {
        let retirements = self.retirements.clone();
        for retirement in retirements {
            if let Some(ticket) = self
                .tickets
                .iter_mut()
                .find(|t| t.id == retirement.ticket)
            {
                ticket.closed_tick = Some(retirement.retired_at_tick);
                for reason in ALL_REASONS {
                    if reason.token() == retirement.retirement_reason {
                        ticket.terminal = Some(reason);
                    }
                }
                ticket.evidence = retirement.evidence.clone();
            }
        }
        for correction in self.corrections.clone() {
            if let Some(ticket) = self.tickets.iter_mut().find(|t| {
                t.terminal.is_some()
                    && self
                        .retirements
                        .iter()
                        .any(|r| r.id == correction.target && r.ticket == t.id)
            }) {
                ticket.corrected = Some(correction.new_value.clone());
            }
        }
    }

    pub fn governor_banner(&self) -> String {
        match self.governor.load_state {
            LoadState::Parsed => format!(
                "governor v{} · {} profile",
                self.governor.version, self.governor.profile
            ),
            LoadState::FailedClosed => "GOVERNOR FAILED CLOSED".to_string(),
        }
    }

    /// Ask the governor, and record the refusal when there is one. A denial that
    /// is not written down is a denial nobody can audit.
    pub fn authorize_or_record(&mut self, tick: u64, op: &Op) -> Verdict {
        let verdict = self.governor.authorize(op);
        if !verdict.allowed {
            self.denials += 1;
            let _ = self
                .season
                .append_event(tick, "authorization.denied", &format!("{} — {}", op.describe(), verdict.reason));
        }
        verdict
    }

    pub fn record_refusal(&mut self, tick: u64, op_name: &str, reason: &str) {
        self.denials += 1;
        let _ = self.season.append_event(
            tick,
            "plan.refused",
            &format!("`{op_name}` refused: {reason}; known operations: {}", ALL_OPS.join(", ")),
        );
    }

    /// File a ticket. The ticket is filed **before** the work runs, with its
    /// gate stated up front.
    #[allow(clippy::too_many_arguments)]
    pub fn file(
        &mut self,
        tick: u64,
        kind: TicketKind,
        objective: String,
        gate: String,
        expectation: Expectation,
        roads_before: Vec<u32>,
        governor_version: u32,
    ) -> std::io::Result<String> {
        let id = self.season.next_id("tickets", kind.prefix());
        let ticket = Ticket {
            id: id.clone(),
            contract: self.contract.clone(),
            kind,
            status: TicketStatus::Open,
            objective,
            gate,
            opened_tick: tick,
            closed_tick: None,
            terminal: None,
            supersedes: None,
            governor_version,
            case_key: None,
            count: 1,
            expectation,
            roads_before,
            evidence: Vec::new(),
            actor: "operator".to_string(),
            corrected: None,
        };
        self.season.write_ticket(&ticket)?;
        self.season
            .append_event(tick, "ticket.filed", &format!("{id} — {}", ticket.objective))?;
        self.tickets.push(ticket);
        Ok(id)
    }

    /// A case, deduped by key. An existing open case with the same key has its
    /// count updated; nothing new is filed. This is the whole reason four
    /// hundred citizens are one ticket.
    pub fn file_or_update_case(&mut self, tick: u64, finding: &CaseFinding) -> std::io::Result<()> {
        if let Some(existing) = self
            .tickets
            .iter_mut()
            .find(|t| t.case_key.as_deref() == Some(finding.key.as_str()) && !t.is_closed())
        {
            existing.count = finding.count;
            existing.objective = finding.objective.clone();
            self.season.append_event(
                tick,
                "case.updated",
                &format!("{} — {}", existing.id, finding.objective),
            )?;
            return Ok(());
        }

        let id = self.season.next_id("tickets", TicketKind::Case.prefix());
        let ticket = Ticket {
            id: id.clone(),
            contract: self.contract.clone(),
            kind: TicketKind::Case,
            status: TicketStatus::Open,
            objective: finding.objective.clone(),
            gate: "the residents' condition improves, read from the world".to_string(),
            opened_tick: tick,
            closed_tick: None,
            terminal: None,
            supersedes: None,
            governor_version: self.governor.version,
            case_key: Some(finding.key.clone()),
            count: finding.count,
            expectation: Expectation::None,
            roads_before: Vec::new(),
            evidence: Vec::new(),
            actor: "sim".to_string(),
            corrected: None,
        };
        self.season.write_ticket(&ticket)?;
        self.season.append_event(
            tick,
            "case.opened",
            &format!("{id} — {}", finding.objective),
        )?;
        self.tickets.push(ticket);
        Ok(())
    }

    /// Sample the population and reconcile the case population with it: new
    /// conditions file cases, resolved conditions close them, and a condition
    /// that has come back files a *new* case referencing the old one rather
    /// than silently reopening it.
    pub fn sample(&mut self, world: &World, tick: u64) -> std::io::Result<()> {
        let findings = sample_cases(world);
        let keys: Vec<String> = findings.iter().map(|f| f.key.clone()).collect();

        let resolved: Vec<(usize, String)> = self
            .tickets
            .iter()
            .enumerate()
            .filter(|(_, t)| t.kind == TicketKind::Case && !t.is_closed())
            .filter(|(_, t)| {
                t.case_key
                    .as_ref()
                    .map(|key| !keys.contains(key))
                    .unwrap_or(false)
            })
            .map(|(index, t)| (index, t.id.clone()))
            .collect();

        for (index, _id) in resolved {
            self.close_ticket(
                index,
                tick,
                RetirementReason::CompletedAndValidated,
                "the condition this case described is no longer present in the world",
                Some(("B", "read back: the sampled condition now reports zero".to_string())),
            )?;
        }

        // A closed case that is still true gets a successor, never a reopening.
        for finding in &findings {
            let superseded: Option<String> = self
                .tickets
                .iter()
                .filter(|t| t.case_key.as_deref() == Some(finding.key.as_str()))
                .filter(|t| t.is_closed())
                .map(|t| t.id.clone())
                .next();
            self.file_or_update_case(tick, finding)?;
            if let Some(previous) = superseded {
                if let Some(current) = self
                    .tickets
                    .iter_mut()
                    .find(|t| t.case_key.as_deref() == Some(finding.key.as_str()) && !t.is_closed())
                {
                    if current.supersedes.is_none() {
                        current.supersedes = Some(previous.clone());
                        let _ = self.season.append_event(
                            tick,
                            "case.supersedes",
                            &format!("{} supersedes {previous}", current.id),
                        );
                    }
                }
            }
        }
        Ok(())
    }

    /// Read the world back. A ticket closes validated only when the world shows
    /// what its gate said it would show.
    pub fn validate(&mut self, world: &World, tick: u64) -> std::io::Result<Vec<String>> {
        let mut work: Vec<(usize, RetirementReason, String, (String, String))> = Vec::new();

        for (index, ticket) in self.tickets.iter().enumerate() {
            if ticket.is_closed() || ticket.expectation == Expectation::None {
                continue;
            }
            // Give the world a tick to reflect the change before reading it
            // back, so a same-tick check cannot pass on a stale world.
            if tick.saturating_sub(ticket.opened_tick) < 2 {
                continue;
            }
            if let Ok(()) = check_expectation(world, &ticket.expectation) {
                let damaged = first_damaged_road(world, &ticket.roads_before);
                match damaged {
                    Some(tile) => work.push((
                        index,
                        RetirementReason::CompletedWithKnownRegression,
                        format!(
                            "the build landed, and the check also found that tile {tile} was a road before and is not one now"
                        ),
                        (
                            "B".to_string(),
                            format!(
                                "read back: {} — and a regression was found at tile {tile}",
                                ticket.expectation.describe()
                            ),
                        ),
                    )),
                    None => work.push((
                        index,
                        RetirementReason::CompletedAndValidated,
                        format!("the world shows {}", ticket.expectation.describe()),
                        (
                            "B".to_string(),
                            format!(
                                "read back from world state at tick {tick}: {}",
                                ticket.expectation.describe()
                            ),
                        ),
                    )),
                }
            }
        }

        let mut closed = Vec::new();
        for (index, reason, detail, (grade, evidence_detail)) in work {
            let id = self.tickets[index].id.clone();
            self.close_ticket(index, tick, reason, &detail, Some((grade.as_str(), evidence_detail)))?;
            closed.push(id);
        }
        Ok(closed)
    }

    /// The stall bound. Every ticket reaches a terminal state within it, and
    /// when the world never showed the expected result it closes *unverified* —
    /// never promoted to validated to make the board look better.
    pub fn enforce_bound(&mut self, tick: u64) -> std::io::Result<Vec<String>> {
        let bound = self.bound_sim_seconds;
        let overdue: Vec<(usize, RetirementReason, String)> = self
            .tickets
            .iter()
            .enumerate()
            .filter(|(_, t)| !t.is_closed())
            .filter(|(_, t)| t.age_seconds(tick) > bound)
            .map(|(index, t)| {
                let (reason, detail) = match t.kind {
                    TicketKind::Case => (
                        RetirementReason::BlockedAndClosed,
                        format!(
                            "unresolved after {:.0}s: the condition this case describes is still present. Closed rather than left open forever; if it still holds, a new case will be filed that references this one.",
                            t.age_seconds(tick)
                        ),
                    ),
                    _ => (
                        RetirementReason::CompletedButUnverified,
                        format!(
                            "the world did not show \"{}\" within {:.0}s, so this is not called validated. It is recorded as unverified, which is the honest name for it.",
                            t.expectation.describe(),
                            t.age_seconds(tick)
                        ),
                    ),
                };
                (index, reason, detail)
            })
            .collect();

        let mut closed = Vec::new();
        for (index, reason, detail) in overdue {
            let id = self.tickets[index].id.clone();
            self.close_ticket(index, tick, reason, &detail, None)?;
            closed.push(id);
        }
        Ok(closed)
    }

    /// Late evidence. A ticket already retired as unverified is *not* rewritten
    /// when the world later catches up: a correction is appended beside it, and
    /// the original stays readable.
    pub fn reconcile(&mut self, world: &World, tick: u64) -> std::io::Result<Vec<String>> {
        let candidates: Vec<(usize, String, String)> = self
            .tickets
            .iter()
            .enumerate()
            .filter(|(_, t)| {
                t.terminal == Some(RetirementReason::CompletedButUnverified) && t.corrected.is_none()
            })
            .filter(|(_, t)| t.expectation != Expectation::None)
            .filter_map(|(index, t)| {
                check_expectation(world, &t.expectation)
                    .ok()
                    .map(|_| (index, t.id.clone(), t.expectation.describe()))
            })
            .collect();

        let mut appended = Vec::new();
        for (index, ticket_id, describe) in candidates {
            let retirement_id = self
                .retirements
                .iter()
                .find(|r| r.ticket == ticket_id)
                .map(|r| r.id.clone())
                .unwrap_or_else(|| format!("RET-{ticket_id}"));
            let id = self.season.next_id("corrections", "COR");
            let correction = Correction {
                id: id.clone(),
                target: retirement_id,
                reason: "validation_result_arrived_late".to_string(),
                old_value: "completed_but_unverified".to_string(),
                new_value: format!("read back from world state at tick {tick}: {describe}"),
                evidence: Vec::new(),
                authorized_by: "reconcile".to_string(),
                appended_at_tick: tick,
            };
            self.season.write_correction(&correction)?;
            self.season.append_event(
                tick,
                "correction.appended",
                &format!("{id} — {ticket_id} was unverified and the world has since caught up"),
            )?;
            self.corrections.push(correction);
            self.tickets[index].corrected = Some(format!("the world caught up at tick {tick}"));
            appended.push(ticket_id);
        }
        Ok(appended)
    }

    fn close_ticket(
        &mut self,
        index: usize,
        tick: u64,
        reason: RetirementReason,
        detail: &str,
        evidence: Option<(&str, String)>,
    ) -> std::io::Result<()> {
        let mut evidence_ids: Vec<String> = self.tickets[index].evidence.clone();

        if let Some((grade, evidence_detail)) = evidence {
            let id = self.season.next_id("evidence", "EVD");
            let record = Evidence {
                id: id.clone(),
                ticket: self.tickets[index].id.clone(),
                grade: grade.to_string(),
                kind: "world_read_back".to_string(),
                detail: evidence_detail,
                tick,
            };
            self.season.write_evidence(&record)?;
            self.evidence.push(record);
            evidence_ids.push(id);
        }

        let ticket = &mut self.tickets[index];
        let status = match reason {
            RetirementReason::CompletedAndValidated => TicketStatus::Complete,
            RetirementReason::CompletedWithKnownRegression => TicketStatus::Complete,
            RetirementReason::CompletedButUnverified => TicketStatus::Rejected,
            RetirementReason::BlockedAndClosed => TicketStatus::Blocked,
            RetirementReason::CancelledByAuthority => TicketStatus::Rejected,
            RetirementReason::AuthorizationExpired => TicketStatus::Rejected,
            RetirementReason::ObjectiveWithdrawn => TicketStatus::Dropped,
            RetirementReason::Superseded | RetirementReason::Obsolete => TicketStatus::Dropped,
            _ => TicketStatus::Dropped,
        };

        let retirement_id = self.season.next_id("retirements", "RET");
        let retirement = Retirement {
            id: retirement_id.clone(),
            ticket: ticket.id.clone(),
            contract: ticket.contract.clone(),
            terminal_state: "retired".to_string(),
            retirement_reason: reason.token().to_string(),
            objective: ticket.objective.clone(),
            completion_claim: detail.to_string(),
            grounding: Grounding {
                inputs_verified: true,
                note: format!("the governor version stamped on this ticket is v{}", ticket.governor_version),
            },
            verification: Verification {
                acceptance_criteria: vec![ticket.gate.clone()],
                checks_run: vec![ticket.expectation.describe()],
                passed: reason.is_validated(),
                regressions_detected: reason
                    == RetirementReason::CompletedWithKnownRegression,
            },
            evidence: evidence_ids.clone(),
            governor_version: ticket.governor_version,
            retired_at_tick: tick,
            retired_by: "operator".to_string(),
        };
        self.season.write_retirement(&retirement)?;
        self.season.append_event(
            tick,
            "ticket.retired",
            &format!("{} — {}", ticket.id, reason.token()),
        )?;
        self.retirements.push(retirement);

        let ticket = &mut self.tickets[index];
        ticket.status = status;
        ticket.closed_tick = Some(tick);
        ticket.terminal = Some(reason);
        ticket.evidence = evidence_ids;
        Ok(())
    }

    pub fn counts(&self) -> (usize, usize) {
        let closed = self.tickets.iter().filter(|t| t.is_closed()).count();
        (self.tickets.len() - closed, closed)
    }

    pub fn ledger(&self, limit: usize) -> Vec<&Ticket> {
        let mut open: Vec<&Ticket> = self.tickets.iter().filter(|t| !t.is_closed()).collect();
        open.sort_by_key(|t| std::cmp::Reverse(t.opened_tick));
        let mut closed: Vec<&Ticket> = self.tickets.iter().filter(|t| t.is_closed()).collect();
        closed.sort_by_key(|t| std::cmp::Reverse(t.closed_tick));
        open.into_iter().chain(closed).take(limit).collect()
    }

    /// What the city has to offer you right now, derived from stored state and
    /// never invented to fill a lull. An empty list here would be a bug, not a
    /// quiet moment: there is always something true to do.
    pub fn suggestions(&self, world: &World) -> Vec<String> {
        let mut out = Vec::new();
        for ticket in self.tickets.iter().filter(|t| !t.is_closed()) {
            if ticket.kind == TicketKind::Case {
                out.push(format!("{}: {}", ticket.id, ticket.objective));
            }
        }
        if world.stats.power_plants == 0 {
            out.push("no power plant stands; nothing you zone can grow until one does".to_string());
        }
        if world.stats.road_tiles <= world.height {
            out.push("the city is one strip of road along the map edge; connect it to somewhere".to_string());
        }
        if world.stats.unpowered_zoned > 0 && world.stats.power_plants > 0 {
            out.push(format!(
                "{} zoned tiles are outside any plant's reach along the road network",
                world.stats.unpowered_zoned
            ));
        }
        if world.stats.unemployed > 0 {
            out.push(format!(
                "{} residents have no job; zone commercial or industrial land within walking reach",
                world.stats.unemployed
            ));
        }
        if world.economy.credits < 2_000 {
            out.push(format!(
                "city funds are {} — tax rates or a loan are the two levers",
                world.economy.credits
            ));
        }
        if out.is_empty() {
            out.push(
                "everything you filed is terminal and every condition has cleared; the next move is to build something larger than the last thing"
                    .to_string(),
            );
        }
        out
    }
}

/// Every terminal reason, so a retirement read back from disk can be resolved
/// to its typed form.
pub const ALL_REASONS: [RetirementReason; 14] = [
    RetirementReason::CompletedAndValidated,
    RetirementReason::CompletedWithKnownRegression,
    RetirementReason::CompletedButUnverified,
    RetirementReason::CancelledByAuthority,
    RetirementReason::Superseded,
    RetirementReason::Invalidated,
    RetirementReason::BlockedAndClosed,
    RetirementReason::Abandoned,
    RetirementReason::Duplicate,
    RetirementReason::Obsolete,
    RetirementReason::ObjectiveWithdrawn,
    RetirementReason::AuthorizationExpired,
    RetirementReason::SafetyStop,
    RetirementReason::SystemShutdown,
];

/// Read an expectation back out of the world. `Err` names what did not match,
/// because "it failed" is not a finding.
pub fn check_expectation(world: &World, expectation: &Expectation) -> Result<(), String> {
    match expectation {
        Expectation::None => Ok(()),
        Expectation::RoadTiles(tiles) => {
            for &tile in tiles {
                if !world.tile(tile).road {
                    return Err(format!("tile {tile} is not road"));
                }
            }
            Ok(())
        }
        Expectation::ZonedTiles(pairs) => {
            for &(tile, zone) in pairs {
                if world.tile(tile).zone != zone {
                    return Err(format!(
                        "tile {tile} carries {} rather than {}",
                        world.tile(tile).zone.name(),
                        zone.name()
                    ));
                }
            }
            Ok(())
        }
        Expectation::BuildingAt(tile) => {
            if world.tile(*tile).building.is_some() {
                Ok(())
            } else {
                Err(format!("no structure stands on tile {tile}"))
            }
        }
        Expectation::Demolished(tile) => {
            if world.tile(*tile).building.is_none() {
                Ok(())
            } else {
                Err(format!("something still stands on tile {tile}"))
            }
        }
    }
}

/// The first road that a change damaged. Present behaviour that broke is part
/// of the record, not a footnote to it.
pub fn first_damaged_road(world: &World, roads_before: &[u32]) -> Option<u32> {
    roads_before
        .iter()
        .copied()
        .find(|&tile| !world.tile(tile).road)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{BuildingKind, World};

    /// The verifier's whole supersession check rests on this: an expectation
    /// must name exactly the tiles it is about, no more and no fewer, or a
    /// later action on an unrelated tile would look like an explanation.
    #[test]
    fn named_tiles_are_exactly_the_tiled_ones() {
        assert_eq!(Expectation::RoadTiles(vec![3, 9]).tiles(), vec![3, 9]);
        assert_eq!(
            Expectation::ZonedTiles(vec![(4, Zone::Residential)]).tiles(),
            vec![4]
        );
        assert_eq!(Expectation::BuildingAt(7).tiles(), vec![7]);
        assert_eq!(Expectation::Demolished(8).tiles(), vec![8]);
        assert!(Expectation::None.tiles().is_empty());
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ala-cities-gov-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn write_governor(dir: &Path, body: &str) -> PathBuf {
        std::fs::create_dir_all(dir).expect("dir");
        let path = dir.join("governor.json");
        std::fs::write(&path, body).expect("write governor");
        path
    }

    fn default_governor(dir: &Path) -> PathBuf {
        write_governor(
            dir,
            r#"{"version":3,"profile":"default","allowed_ops":["build_road","zone","place_service","demolish","set_tax","take_loan"],"max_road_leg":64,"max_loan":50000}"#,
        )
    }

    fn city() -> World {
        let mut world = World::new(48, 48, 3);
        for tile in world.tiles.iter_mut() {
            tile.terrain = crate::sim::Terrain::Ground;
        }
        for y in 0..48u32 {
            world.tiles[(y * 48) as usize].road = true;
        }
        world.rebuild_derived();
        world
    }

    #[test]
    fn a_permitted_operation_says_which_governor_permitted_it() {
        let dir = scratch("allow");
        let governor = Governor::load(default_governor(&dir));
        let verdict = governor.authorize(&Op::BuildRoad { distance: 4 });
        assert!(verdict.allowed);
        assert_eq!(verdict.governor_version, 3);
        assert!(verdict.reason.contains("governor v3"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_refusal_names_the_vocabulary_that_would_have_been_accepted() {
        let dir = scratch("refuse-vocab");
        let governor = Governor::load(write_governor(
            &dir,
            r#"{"version":1,"profile":"read_only","allowed_ops":["look","inspect"],"max_road_leg":0,"max_loan":0}"#,
        ));
        let verdict = governor.authorize(&Op::BuildRoad { distance: 4 });
        assert!(!verdict.allowed);
        assert!(
            verdict.reason.contains("`build_road` is not permitted"),
            "the refusal must name the operation: {}",
            verdict.reason
        );
        assert!(
            verdict.reason.contains("look, inspect"),
            "the refusal must name what is permitted instead: {}",
            verdict.reason
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_governor_that_will_not_parse_fails_closed() {
        let dir = scratch("fail-closed");
        let governor = Governor::load(write_governor(&dir, "{ this is not json"));
        assert_eq!(governor.load_state, LoadState::FailedClosed);
        let verdict = governor.authorize(&Op::BuildRoad { distance: 1 });
        assert!(!verdict.allowed);
        assert!(verdict.reason.contains("unreadable"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_missing_governor_is_a_refusal_not_a_default() {
        let dir = scratch("missing");
        let governor = Governor::load(dir.join("nope.json"));
        assert_eq!(governor.load_state, LoadState::FailedClosed);
        assert!(!governor.authorize(&Op::Zone { tiles: 1 }).allowed);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_unknown_profile_is_refused_rather_than_guessed_at() {
        let dir = scratch("unknown-profile");
        let governor = Governor::load(write_governor(
            &dir,
            r#"{"version":1,"profile":"dystopia","allowed_ops":["build_road"],"max_road_leg":9,"max_loan":9}"#,
        ));
        assert_eq!(governor.load_state, LoadState::FailedClosed);
        assert!(governor.reason.contains("unknown profile"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_operation_this_build_does_not_know_is_dropped_not_honoured() {
        let dir = scratch("unknown-op");
        let governor = Governor::load(write_governor(
            &dir,
            r#"{"version":2,"profile":"default","allowed_ops":["build_road","bulldoze_the_moon"],"max_road_leg":8,"max_loan":0}"#,
        ));
        assert_eq!(governor.dropped_ops, vec!["bulldoze_the_moon".to_string()]);
        assert_eq!(governor.allowed_ops, vec!["build_road".to_string()]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_leg_beyond_the_limit_is_refused_by_name() {
        let dir = scratch("leg-limit");
        let governor = Governor::load(default_governor(&dir));
        let verdict = governor.authorize(&Op::BuildRoad { distance: 65 });
        assert!(!verdict.allowed);
        assert!(verdict.reason.contains("64-tile limit"));
        assert!(governor.authorize(&Op::BuildRoad { distance: 64 }).allowed);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_tax_rate_outside_the_permitted_band_is_refused() {
        let dir = scratch("tax");
        let governor = Governor::load(default_governor(&dir));
        assert!(!governor
            .authorize(&Op::SetTax {
                zone: Zone::Residential,
                rate: 0.5
            })
            .allowed);
        assert!(governor
            .authorize(&Op::SetTax {
                zone: Zone::Residential,
                rate: 0.12
            })
            .allowed);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn validation_reads_the_world_back_rather_than_the_request() {
        let dir = scratch("validate");
        let governor_path = default_governor(&dir);
        let mut gov = Government::open(dir.join("saves"), "season_t_s1", &governor_path).expect("open");
        let mut world = city();

        // File a ticket for a road that is never built: the honest outcome is
        // unverified, never validated.
        let to = world.index(20, 20);
        let id = gov
            .file(
                world.clock.tick,
                TicketKind::Build,
                "lay a road to the far corner".to_string(),
                "the tiles are road".to_string(),
                Expectation::RoadTiles(vec![to]),
                Vec::new(),
                3,
            )
            .expect("file");

        world.clock.tick = 10;
        gov.validate(&world, 10).expect("validate");
        let ticket = gov.tickets.iter().find(|t| t.id == id).expect("ticket");
        assert!(!ticket.is_closed(), "a road that is not there must not close");

        // Now build it, and the same check reads it back successfully.
        world.lay_road(world.index(0, 20), to);
        world.clock.tick = 20;
        let closed = gov.validate(&world, 20).expect("validate");
        assert_eq!(closed, vec![id.clone()]);
        let ticket = gov.tickets.iter().find(|t| t.id == id).expect("ticket");
        assert_eq!(ticket.terminal, Some(RetirementReason::CompletedAndValidated));
        let evidence = gov.evidence.iter().find(|e| e.ticket == id).expect("evidence");
        assert_eq!(evidence.grade, "B", "read-back evidence is grade B, not a self-report");
        assert!(evidence.detail.contains("read back from world state"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_build_that_breaks_an_existing_road_is_retired_with_the_regression_recorded() {
        let dir = scratch("regression");
        let governor_path = default_governor(&dir);
        let mut gov = Government::open(dir.join("saves"), "season_t_s1", &governor_path).expect("open");
        let mut world = city();

        let existing = world.index(0, 5);
        let target = world.index(10, 10);
        let id = gov
            .file(
                0,
                TicketKind::Build,
                "zone the far side".to_string(),
                "the tile carries residential zoning".to_string(),
                Expectation::ZonedTiles(vec![(target, Zone::Residential)]),
                vec![existing],
                3,
            )
            .expect("file");

        // The build lands, and the existing road is destroyed in the process.
        world.set_zone(target, Zone::Residential);
        world.tiles[existing as usize].road = false;
        world.rebuild_derived();
        world.clock.tick = 10;
        gov.validate(&world, 10).expect("validate");

        let ticket = gov.tickets.iter().find(|t| t.id == id).expect("ticket");
        assert_eq!(
            ticket.terminal,
            Some(RetirementReason::CompletedWithKnownRegression),
            "an achieved objective plus a broken road is not a clean success"
        );
        let retirement = gov
            .retirements
            .iter()
            .find(|r| r.ticket == id)
            .expect("retirement");
        assert!(retirement.verification.regressions_detected);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_board_drains_even_when_nothing_is_ever_verified() {
        let dir = scratch("no-stall");
        let governor_path = default_governor(&dir);
        let mut gov = Government::open(dir.join("saves"), "season_t_s1", &governor_path).expect("open");
        gov.bound_sim_seconds = 5.0;

        let mut ids = Vec::new();
        for i in 0..25 {
            ids.push(
                gov.file(
                    0,
                    TicketKind::Build,
                    format!("build {i}"),
                    "the tile is road".to_string(),
                    Expectation::RoadTiles(vec![i as u32]),
                    Vec::new(),
                    3,
                )
                .expect("file"),
            );
        }
        assert_eq!(gov.counts(), (25, 0));

        // Past the bound, with the world never once satisfying the gate.
        let tick = (6.0 * SIM_HZ as f64) as u64;
        gov.enforce_bound(tick).expect("enforce");
        assert_eq!(gov.counts(), (0, 25), "the board must drain");

        let unvalidated = gov
            .tickets
            .iter()
            .filter(|t| t.terminal == Some(RetirementReason::CompletedButUnverified))
            .count();
        assert_eq!(unvalidated, 25);
        assert_eq!(
            gov.tickets
                .iter()
                .filter(|t| t.terminal == Some(RetirementReason::CompletedAndValidated))
                .count(),
            0,
            "cannot-stall must never be implemented as auto-promotion to validated"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_case_that_closes_and_returns_files_a_successor_rather_than_reopening() {
        let dir = scratch("supersede");
        let governor_path = default_governor(&dir);
        let mut gov = Government::open(dir.join("saves"), "season_t_s1", &governor_path).expect("open");
        let mut world = city();
        gov.bound_sim_seconds = 5.0;

        // Make a district with unpowered zoned tiles.
        for x in 1..8u32 {
            world.set_zone(world.index(x, 20), Zone::Residential);
        }
        world.rebuild_derived();
        world.clock.tick = 1;
        gov.sample(&world, 1).expect("sample");
        let first = gov.tickets[0].id.clone();
        assert_eq!(gov.tickets.len(), 1);
        assert_eq!(gov.tickets[0].count, 7, "one case, carrying a count of seven");

        // Sampling again updates the count rather than filing a second ticket.
        for x in 1..10u32 {
            world.set_zone(world.index(x, 20), Zone::Residential);
        }
        gov.sample(&world, 2).expect("sample");
        assert_eq!(gov.tickets.len(), 1, "a case is deduped by key, not refiled");
        assert_eq!(gov.tickets[0].count, 9);

        // Force it past the bound, then let the condition still be true.
        gov.enforce_bound((7.0 * SIM_HZ as f64) as u64).expect("enforce");
        assert_eq!(
            gov.tickets[0].terminal,
            Some(RetirementReason::BlockedAndClosed)
        );
        gov.sample(&world, 400).expect("sample");
        assert_eq!(gov.tickets.len(), 2, "a returning condition files a new case");
        assert_eq!(gov.tickets[1].supersedes, Some(first));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn late_evidence_is_a_correction_not_a_rewritten_record() {
        let dir = scratch("correction");
        let governor_path = default_governor(&dir);
        let mut gov = Government::open(dir.join("saves"), "season_t_s1", &governor_path).expect("open");
        let mut world = city();
        gov.bound_sim_seconds = 5.0;

        let tile = world.index(30, 30);
        let id = gov
            .file(
                0,
                TicketKind::Build,
                "a road to the north".to_string(),
                "the tile is road".to_string(),
                Expectation::RoadTiles(vec![tile]),
                Vec::new(),
                3,
            )
            .expect("file");

        // Bound closes it unverified because the road is not there yet.
        gov.enforce_bound((6.0 * SIM_HZ as f64) as u64).expect("enforce");
        assert_eq!(
            gov.tickets[0].terminal,
            Some(RetirementReason::CompletedButUnverified)
        );

        // The road appears later. The record must not be rewritten.
        world.tiles[tile as usize].road = true;
        world.rebuild_derived();
        let appended = gov.reconcile(&world, 200).expect("reconcile");
        assert_eq!(appended, vec![id.clone()]);
        assert_eq!(
            gov.tickets[0].terminal,
            Some(RetirementReason::CompletedButUnverified),
            "the original retirement stands; a correction sits beside it"
        );
        assert!(gov.tickets[0].corrected.is_some());
        assert_eq!(gov.corrections.len(), 1);
        assert_eq!(gov.corrections[0].reason, "validation_result_arrived_late");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_record_survives_being_reopened() {
        let dir = scratch("reopen");
        let governor_path = default_governor(&dir);
        let mut world = city();
        let id = {
            let mut gov =
                Government::open(dir.join("saves"), "season_t_s1", &governor_path).expect("open");
            let tile = world.index(4, 4);
            let id = gov
                .file(
                    0,
                    TicketKind::Build,
                    "lay road".to_string(),
                    "the tile is road".to_string(),
                    Expectation::RoadTiles(vec![tile]),
                    Vec::new(),
                    3,
                )
                .expect("file");
            world.tiles[tile as usize].road = true;
            world.rebuild_derived();
            gov.validate(&world, 10).expect("validate");
            id
        };

        let reopened =
            Government::open(dir.join("saves"), "season_t_s1", &governor_path).expect("reopen");
        let ticket = reopened
            .tickets
            .iter()
            .find(|t| t.id == id)
            .expect("ticket survived");
        assert_eq!(
            ticket.terminal,
            Some(RetirementReason::CompletedAndValidated),
            "closure lives in the retirement record and is reapplied on load"
        );
        assert_eq!(reopened.counts(), (0, 1));
        fs_remove(&dir);
    }

    #[test]
    fn there_is_always_something_to_do() {
        let dir = scratch("suggest");
        let governor_path = default_governor(&dir);
        let gov = Government::open(dir.join("saves"), "season_t_s1", &governor_path).expect("open");
        let world = city();
        let suggestions = gov.suggestions(&world);
        assert!(
            !suggestions.is_empty(),
            "an empty suggestion list is a bug, not a quiet moment"
        );
        for line in &suggestions {
            assert!(
                !line.contains("placeholder") && !line.is_empty(),
                "suggestions must be derived from state, never filler"
            );
        }
        fs_remove(&dir);
    }

    fn fs_remove(path: &Path) {
        let _ = std::fs::remove_dir_all(path);
    }

    #[test]
    fn a_power_plant_operation_is_the_one_that_lights_the_map() {
        let dir = scratch("service");
        let governor = Governor::load(default_governor(&dir));
        let verdict = governor.authorize(&Op::PlaceService {
            kind: BuildingKind::PowerPlant,
        });
        assert!(verdict.allowed);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
