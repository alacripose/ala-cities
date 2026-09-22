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
                let mut reproduced = 0;
                let mut contradicted = 0;
                let mut late = 0;

                for ticket in &government.tickets {
                    if ticket.expectation == ala_cities::gov::Expectation::None {
                        continue;
                    }
                    let result = ala_cities::gov::check_expectation(&world, &ticket.expectation);
                    match ticket.terminal {
                        Some(RetirementReason::CompletedAndValidated) => match result {
                            Ok(()) => reproduced += 1,
                            Err(why) => {
                                contradicted += 1;
                                findings.push(format!(
                                    "{} was validated as \"{}\" and the world does not show it now: {why}",
                                    ticket.id,
                                    ticket.expectation.describe()
                                ));
                            }
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
                    "world read-back — {reproduced} reproduced, {contradicted} contradicted, {late} late"
                );
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
