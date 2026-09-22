//! The agent's ledger tool — the **only** process that writes the work
//! ledger, and the only author of `in_progress` (C6 Q127: the pane reads, the
//! agent's tool writes, and the two never share a process).
//!
//! This is the tool I (the coding agent) run while working on a playtest's
//! ticketed items; a73 makes that work real, so this tool is real. One
//! command appends one record and exits; there is no interactive session to
//! leave dangling, and no state to forget to save.

use std::path::PathBuf;

use ala_cities::agentledger::{self, Action};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut season = "season_2026_s1".to_string();
    let mut ticket = String::new();
    let mut action = String::new();
    let mut detail = String::new();
    let mut agent = "codebuff".to_string();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--season" => {
                i += 1;
                season = args.get(i).cloned().unwrap_or(season);
            }
            "--agent" => {
                i += 1;
                agent = args.get(i).cloned().unwrap_or(agent);
            }
            "--ticket" => {
                i += 1;
                ticket = args.get(i).cloned().unwrap_or_default();
            }
            "--action" => {
                i += 1;
                action = args.get(i).cloned().unwrap_or_default();
            }
            "--detail" => {
                i += 1;
                detail = args.get(i).cloned().unwrap_or_default();
            }
            other => {
                eprintln!("unknown argument {other}");
                std::process::exit(2);
            }
        }
        i += 1;
    }

    let Some(action) = Action::from_token(&action) else {
        eprintln!(
            "--action must be one of in_progress | blocked | note | released \
             (only this tool may set in_progress)"
        );
        std::process::exit(2);
    };
    if ticket.is_empty() {
        eprintln!("--ticket is required: the ledger records work on a named ticket");
        std::process::exit(2);
    }
    if action == Action::InProgress && detail.is_empty() {
        eprintln!("--detail is required with in_progress: a claim without a stated task is not a record");
        std::process::exit(2);
    }

    let path = ledger_path(&season);
    if let Err(err) = agentledger::append(&path, &agent, &ticket, action, &detail) {
        eprintln!("could not append to {}: {err}", path.display());
        std::process::exit(1);
    }
    println!("{:?} {} {}", action, ticket, path.display());
}

fn ledger_path(season: &str) -> PathBuf {
    PathBuf::from("saves")
        .join(season)
        .join(agentledger::LEDGER_FILE)
}
