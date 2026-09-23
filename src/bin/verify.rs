//! Headless verifier.
//!
//! No window, no GPU. Re-reads a season record and re-checks its claims —
//! against the record's own structure, and where a world save exists, against
//! the world itself.
//!
//! Exit code 0 means every claim reproduced. A non-zero exit is usable as a CI
//! gate, which is the point: a verification step that cannot fail is decoration.
//!
//! What this does **not** establish, stated here rather than implied: it says
//! nothing about how the interface looks, nothing about frame timing, and
//! nothing about a build it has no record for. Those are judgement items and
//! they are reported as open, not as passed.

use std::path::PathBuf;
use std::process::ExitCode;

use ala_cities::gov::{Government, LoadState, RetirementReason, ALL_REASONS};
use ala_cities::sim::World;

struct Args {
    saves: PathBuf,
    season: String,
    governor: PathBuf,
    world: Option<PathBuf>,
    bound_seconds: f64,
}

impl Args {
    fn parse() -> Self {
        let mut args = std::env::args().skip(1);
        let mut out = Self {
            saves: PathBuf::from("saves"),
            season: "season_2026_s1".to_string(),
            governor: PathBuf::from("config/governor.json"),
            world: Some(PathBuf::from("saves/world.ron")),
            bound_seconds: 30.0,
        };
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--saves" => {
                    if let Some(value) = args.next() {
                        out.saves = PathBuf::from(value);
                    }
                }
                "--season" => {
                    if let Some(value) = args.next() {
                        out.season = value;
                    }
                }
                "--governor" => {
                    if let Some(value) = args.next() {
                        out.governor = PathBuf::from(value);
                    }
                }
                "--world" => out.world = args.next().map(PathBuf::from),
                "--no-world" => out.world = None,
                "--bound" => {
                    if let Some(value) = args.next() {
                        out.bound_seconds = value.parse().unwrap_or(30.0);
                    }
                }
                "--help" | "-h" => {
                    println!(
                        "verify [--saves DIR] [--season ID] [--governor FILE] [--world FILE|--no-world] [--bound SECONDS]"
                    );
                    std::process::exit(0);
                }
                _ => {}
            }
        }
        out
    }
}

fn main() -> ExitCode {
    let args = Args::parse();
    let mut findings: Vec<String> = Vec::new();

    // Read-only on purpose: verification must not create the record it is
    // checking, or a missing record would come back looking like an empty valid
    // one. This cannot fail on a missing directory for the same reason it
    // cannot help create one.
    let government = Government::open_read_only(&args.saves, &args.season, &args.governor);

    println!("record — {}", government.season.dir().display());
    println!(
        "governor — v{} profile `{}` from {}",
        government.governor.version,
        government.governor.profile,
        government.governor.path.display()
    );

    // -----------------------------------------------------------------
    // 1. The governor. A governance layer that fails open is worse than none,
    //    because it is believed.
    // -----------------------------------------------------------------
    if government.governor.load_state == LoadState::FailedClosed {
        findings.push(format!(
            "the governor failed closed: {}. Every operation in this record was evaluated under a governor that could not be read.",
            government.governor.reason
        ));
    }
    if !government.governor.dropped_ops.is_empty() {
        findings.push(format!(
            "the governor names operations this build does not know, which were dropped rather than honoured: {}",
            government.governor.dropped_ops.join(", ")
        ));
    }

    // -----------------------------------------------------------------
    // 2. Structure of the record itself.
    // -----------------------------------------------------------------
    let (open, terminal) = government.counts();
    println!(
        "tickets — {} filed, {open} open, {terminal} terminal",
        government.tickets.len()
    );
    println!(
        "evidence — {} · retirements {} · corrections {} · recorded refusals {}",
        government.evidence.len(),
        government.retirements.len(),
        government.corrections.len(),
        government.denials
    );

    for retirement in &government.retirements {
        if !government.tickets.iter().any(|t| t.id == retirement.ticket) {
            findings.push(format!(
                "{} retires {}, which is not a ticket in this record",
                retirement.id, retirement.ticket
            ));
        }
        if !ALL_REASONS
            .iter()
            .any(|r| r.token() == retirement.retirement_reason)
        {
            findings.push(format!(
                "{} carries the reason `{}`, which is not in the taxonomy",
                retirement.id, retirement.retirement_reason
            ));
        }
    }

    for ticket in &government.tickets {
        if ticket.is_closed() && ticket.terminal.is_none() {
            findings.push(format!(
                "{} is closed with no terminal reason recorded; a closure that cannot name itself is not a closure",
                ticket.id
            ));
        }
        // The claim this exists to catch: a ticket marked validated with
        // nothing behind it.
        if ticket.terminal == Some(RetirementReason::CompletedAndValidated) {
            let strong: Vec<&String> = government
                .evidence
                .iter()
                .filter(|e| e.ticket == ticket.id)
                .filter(|e| e.grade == "A" || e.grade == "B")
                .map(|e| &e.id)
                .collect();
            if strong.is_empty() {
                findings.push(format!(
                    "{} is recorded as completed_and_validated with no grade-A or grade-B evidence attached",
                    ticket.id
                ));
            }
        }
        if ticket.is_closed() && ticket.terminal == Some(RetirementReason::CompletedButUnverified)
            && ticket.evidence.is_empty()
            && ticket.expectation != ala_cities::gov::Expectation::None
        {
            // Not a defect: unverified work legitimately has no read-back
            // evidence. Reported so a reader can tell the two apart.
            println!(
                "  note — {} closed unverified with no evidence, which is the honest shape for work the world never showed",
                ticket.id
            );
        }
    }

    // -----------------------------------------------------------------
    // 3. The strongest check: read the verdict back out of the world.
    // -----------------------------------------------------------------
    let mut world_checked = false;
    if let Some(path) = &args.world {
        match World::load(path) {
            Ok(world) => {
                world_checked = true;
                println!("world — {}", path.display());
                // A save that had to be migrated is reported here too, because this is
                // the program whose job is to say what the record does and does not
                // support: a derived material claim is a claim this run cannot corroborate
                // from anything a person recorded.
                if let Some(migration) = &world.migration {
                    println!("  {}", migration.describe());
                }
                let mut reproduced = 0;
                let mut contradicted = 0;
                let mut late = 0;
                let mut superseded = 0;
                let mut unchecked = 0;

                println!("  world snapshot at tick {}", world.clock.tick);

                // The mass audit (C9 phase 2). Read back out of the world, not out of the
                // record: this is the one finding that is about the world's own arithmetic
                // rather than about a claim somebody made, and it is the campaign's central
                // assertion — the city is made of what it dug up, or this says by how much it
                // is not.
                let audit = world.mass_audit();
                let kg = |grams: i64| format!("{:.3} t", grams as f64 / 1_000_000.0);
                println!(
                    "  mass — ground {} taken, {} standing, {}",
                    kg(audit.extracted_g),
                    kg(audit.standing_g),
                    if audit.conserves() {
                        format!("{} held loose or spent: the city is made of what it dug", kg(audit.loose_g()))
                    } else {
                        format!("{} THAT WAS NEVER DUG UP", kg(-audit.loose_g()))
                    }
                );
                for finding in audit.findings() {
                    println!("  material finding — {finding}");
                    findings.push(finding);
                }

                // A claim is about the world at the tick it was made. This is a
                // final-state check, so a claim that no longer holds may still
                // be true when it was written — the honest reading is that some
                // later recorded action changed the tile. The distinction that
                // matters: explained by a record, or not.
                enum Explanation {
                    /// A later terminal ticket names one of these tiles.
                    Recorded(String),
                    /// The city grew over the zone. Expected, and not something a
                    /// ticket would record: growth is simulation, not paper.
                    Grown,
                    /// The world changed under a ticket that never reached a
                    /// terminal state. This is the stall, visible on the record.
                    Open(String),
                }

                let explain = |ticket: &ala_cities::gov::Ticket| -> Option<Explanation> {
                    let tiles = ticket.expectation.tiles();
                    if tiles.is_empty() {
                        return None;
                    }
                    if let Some(later) = government.tickets.iter().find(|later| {
                        later.opened_tick > ticket.opened_tick
                            && later.terminal.is_some()
                            && later.expectation.tiles().iter().any(|t| tiles.contains(t))
                    }) {
                        return Some(Explanation::Recorded(later.id.clone()));
                    }
                    if let Some(later) = government.tickets.iter().find(|later| {
                        later.opened_tick > ticket.opened_tick
                            && later.terminal.is_none()
                            && later.expectation.tiles().iter().any(|t| tiles.contains(t))
                    }) {
                        return Some(Explanation::Open(later.id.clone()));
                    }
                    // A zone claim whose tile now carries a structure was not
                    // contradicted: the zone was set, and the city then built on
                    // it, which is the zone doing its job.
                    if matches!(ticket.expectation, ala_cities::gov::Expectation::ZonedTiles(_))
                        && tiles.iter().any(|t| {
                            world
                                .tiles
                                .get(*t as usize)
                                .is_some_and(|tile| tile.building.is_some())
                        })
                    {
                        return Some(Explanation::Grown);
                    }
                    None
                };

                for ticket in &government.tickets {
                    if ticket.expectation == ala_cities::gov::Expectation::None {
                        continue;
                    }
                    // A claim is about the world at the tick it was closed. A
                    // snapshot older than the claim cannot speak to it, and
                    // saying "the world does not show this" about a world from
                    // before the work happened is a false finding — the kind
                    // that teaches a reader to ignore the verifier.
                    if ticket
                        .closed_tick
                        .is_some_and(|closed| closed > world.clock.tick)
                    {
                        unchecked += 1;
                        continue;
                    }
                    let result = ala_cities::gov::check_expectation(&world, &ticket.expectation);
                    match ticket.terminal {
                        Some(RetirementReason::CompletedAndValidated) => match result {
                            Ok(()) => reproduced += 1,
                            Err(why) => match explain(ticket) {
                                Some(Explanation::Recorded(by)) => {
                                    superseded += 1;
                                    println!(
                                        "  superseded — {} was validated as \"{}\" and {by} later changed that tile; the claim was true when it was written",
                                        ticket.id,
                                        ticket.expectation.describe()
                                    );
                                }
                                Some(Explanation::Grown) => {
                                    superseded += 1;
                                    println!(
                                        "  grown over — {} was validated as \"{}\" and the city has since built on it, which is the zone working rather than a claim failing",
                                        ticket.id,
                                        ticket.expectation.describe()
                                    );
                                }
                                Some(Explanation::Open(by)) => {
                                    contradicted += 1;
                                    findings.push(format!(
                                        "{} was validated as \"{}\" and the world has since changed under {by}, which is still open — the record has not caught up with the city"
                                        , ticket.id, ticket.expectation.describe()
                                    ));
                                }
                                None => {
                                    contradicted += 1;
                                    findings.push(format!(
                                        "{} was validated as \"{}\" and the world does not show it now, with nothing later on the record to account for the change: {why}",
                                        ticket.id,
                                        ticket.expectation.describe()
                                    ));
                                }
                            },
                        },
                        Some(RetirementReason::CompletedButUnverified) if result.is_ok() => {
                            late += 1;
                            println!(
                                "  late evidence — {} was unverified and the world has since caught up; a correction belongs beside it",
                                ticket.id
                            );
                        }
                        _ => {}
                    }
                }
                println!(
                    "world read-back — {reproduced} reproduced, {contradicted} contradicted, {superseded} superseded by later recorded work, {late} late"
                );
                if unchecked > 0 {
                    println!(
                        "  not checkable — {unchecked} claim(s) were closed after this snapshot; check them against a later world save or a replay"
                    );
                }
            }
            Err(err) => {
                findings.push(format!(
                    "a world save was expected at {} and could not be read ({err}); only the record's internal consistency was checked",
                    path.display()
                ));
            }
        }
    } else {
        println!("world — not supplied; record consistency checked only");
    }

    // -----------------------------------------------------------------
    // 4. Report. Mechanical results and open judgement items are kept apart,
    //    because a green line that quietly includes an unmeasured claim is
    //    worse than no report at all.
    // -----------------------------------------------------------------
    println!();
    println!("mechanical:");
    println!(
        "  {}",
        if findings.is_empty() {
            "every check reproduced".to_string()
        } else {
            format!("{} finding(s)", findings.len())
        }
    );
    if !world_checked {
        println!("  note: without a world save, no expectation was re-read from world state");
    }
    println!("judgement — NOT established by this run:");
    println!("  composite contrast on a rendered frame, layout at every window size,");
    println!("  frame timing under load, and whether the city is any fun.");
    if !findings.is_empty() {
        println!();
        println!("findings:");
        for finding in &findings {
            println!("  - {finding}");
        }
    }

    if findings.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
