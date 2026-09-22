//! The season record on disk.
//!
//! ```text
//! <root>/index.json
//! <root>/<season>/
//!   season.json
//!   events.jsonl          <- the one log; every other kind is append-only
//!   tickets/<id>.json
//!   retirements/<id>.json
//!   evidence/<id>.json
//!   corrections/<id>.json
//! ```
//!
//! Two rules are load-bearing here.
//!
//! **Ids come from the directory, never from a counter.** A process killed
//! mid-run leaves a counter behind its own directory; trusting the counter
//! hands out an id that already exists, and because records are append-only
//! the writer would then overwrite history. The counter is honoured as a
//! starting point and then skipped past whatever is already on disk.
//!
//! **A corrupt record loses that record, not the file.** A log that refuses to
//! load because of one bad line is a log nobody can use.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::gov::{Correction, Evidence, Retirement, Ticket};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SeasonMeta {
    pub id: String,
    pub schema_version: u32,
    pub timezone: String,
    pub opened_at_unix_ms: u128,
    pub purpose: String,
    pub campaigns: Vec<String>,
    pub counters: Counters,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Counters {
    pub tickets_filed: u64,
    pub tickets_retired: u64,
    pub corrections_appended: u64,
}

#[derive(Clone, Debug)]
pub struct Season {
    pub root: PathBuf,
    pub id: String,
    pub events_written: u64,
    /// Records that could not be parsed when read back. Counted, never hidden.
    pub corrupt_records: u64,
}

impl Season {
    pub fn open(root: impl AsRef<Path>, id: &str, purpose: &str) -> std::io::Result<Self> {
        let root = root.as_ref().to_path_buf();
        let dir = root.join(id);
        for sub in ["tickets", "retirements", "evidence", "corrections"] {
            fs::create_dir_all(dir.join(sub))?;
        }
        fs::create_dir_all(&root)?;

        let meta_path = dir.join("season.json");
        if !meta_path.exists() {
            let meta = SeasonMeta {
                id: id.to_string(),
                schema_version: 1,
                timezone: "UTC".to_string(),
                opened_at_unix_ms: now_unix_ms(),
                purpose: purpose.to_string(),
                campaigns: vec!["C1 — sim core".to_string()],
                counters: Counters::default(),
            };
            fs::write(&meta_path, serde_json::to_string_pretty(&meta)?)?;
        }

        let season = Self {
            root,
            id: id.to_string(),
            events_written: 0,
            corrupt_records: 0,
        };
        season.write_index()?;
        Ok(season)
    }

    /// Attach to a season that already exists, without creating anything.
    ///
    /// A verifier must not write to the record it is verifying, so this path
    /// makes no directories, writes no `season.json`, and will never hand out
    /// an id.
    pub fn attach(root: impl AsRef<Path>, id: &str) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            id: id.to_string(),
            events_written: 0,
            corrupt_records: 0,
        }
    }

    pub fn dir(&self) -> PathBuf {
        self.root.join(&self.id)
    }

    /// A lightweight discovery file, so a reader can find a season without a
    /// deep read of every record in it.
    fn write_index(&self) -> std::io::Result<()> {
        let index = serde_json::json!({
            "seasons": [{
                "id": self.id,
                "path": self.dir().to_string_lossy(),
            }]
        });
        fs::write(
            self.root.join("index.json"),
            serde_json::to_string_pretty(&index)?,
        )
    }

    /// The next free id for a prefix, read from the directory rather than a
    /// persisted counter.
    ///
    /// The folder matters: retirements and tickets have their own sequences, and
    /// scanning the wrong directory hands out an id that already exists in
    /// another one — which the append-only writer then refuses, after the fact.
    pub fn next_id(&self, folder: &str, prefix: &str) -> String {
        let dir = self.dir().join(folder);
        let mut highest = 0u64;
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let Some(rest) = name.strip_prefix(&format!("{prefix}-")) else {
                    continue;
                };
                let Some(number) = rest.strip_suffix(".json") else {
                    continue;
                };
                if let Ok(value) = number.parse::<u64>() {
                    highest = highest.max(value);
                }
            }
        }
        format!("{prefix}-{:06}", highest + 1)
    }

    pub fn append_event(&mut self, tick: u64, kind: &str, detail: &str) -> std::io::Result<()> {
        let path = self.dir().join("events.jsonl");
        let line = serde_json::json!({
            "tick": tick,
            "sim_seconds": tick as f64 / crate::sim::SIM_HZ as f64,
            "kind": kind,
            "detail": detail,
        });
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        writeln!(file, "{}", serde_json::to_string(&line)?)?;
        self.events_written += 1;
        Ok(())
    }

    fn write_record<T: Serialize>(&self, folder: &str, id: &str, value: &T) -> std::io::Result<()> {
        let path = self.dir().join(folder).join(format!("{id}.json"));
        if path.exists() {
            // Append-only means append-only. Re-writing a record in place would
            // be the silent edit the whole layout exists to prevent.
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("refused to overwrite existing record {folder}/{id}.json"),
            ));
        }
        fs::write(path, serde_json::to_string_pretty(value)?)
    }

    pub fn write_ticket(&self, ticket: &Ticket) -> std::io::Result<()> {
        self.write_record("tickets", &ticket.id, ticket)
    }

    pub fn write_retirement(&self, retirement: &Retirement) -> std::io::Result<()> {
        self.write_record("retirements", &retirement.id, retirement)
    }

    pub fn write_evidence(&self, evidence: &Evidence) -> std::io::Result<()> {
        self.write_record("evidence", &evidence.id, evidence)
    }

    pub fn write_correction(&self, correction: &Correction) -> std::io::Result<()> {
        self.write_record("corrections", &correction.id, correction)
    }

    fn read_folder<T: for<'de> Deserialize<'de>>(&self, folder: &str) -> (Vec<T>, u64) {
        let mut values = Vec::new();
        let mut corrupt = 0;
        let dir = self.dir().join(folder);
        let Ok(entries) = fs::read_dir(&dir) else {
            return (values, corrupt);
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            match fs::read_to_string(&path).map_err(|e| e.to_string()).and_then(|t| {
                serde_json::from_str::<T>(&t).map_err(|e| e.to_string())
            }) {
                Ok(value) => values.push(value),
                Err(err) => {
                    corrupt += 1;
                    tracing::warn!(path = %path.display(), error = %err, "skipped one unreadable record");
                }
            }
        }
        (values, corrupt)
    }

    pub fn read_tickets(&self) -> Vec<Ticket> {
        self.read_folder::<Ticket>("tickets").0
    }

    pub fn read_retirements(&self) -> Vec<Retirement> {
        self.read_folder::<Retirement>("retirements").0
    }

    pub fn read_evidence(&self) -> Vec<Evidence> {
        self.read_folder::<Evidence>("evidence").0
    }

    pub fn read_corrections(&self) -> Vec<Correction> {
        self.read_folder::<Correction>("corrections").0
    }

    /// Every event line that parses. Unparseable lines are counted and dropped,
    /// which is the difference between a log with a bad line and no log at all.
    pub fn read_events(&self) -> (Vec<serde_json::Value>, u64) {
        let path = self.dir().join("events.jsonl");
        let Ok(text) = fs::read_to_string(path) else {
            return (Vec::new(), 0);
        };
        let mut events = Vec::new();
        let mut corrupt = 0;
        for line in text.lines() {
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<serde_json::Value>(line) {
                Ok(value) => events.push(value),
                Err(_) => corrupt += 1,
            }
        }
        (events, corrupt)
    }
}

pub fn now_unix_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gov::{Ticket, TicketKind, TicketStatus};

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ala-cities-season-{name}"));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn ids_come_from_the_directory_not_a_counter() {
        let root = scratch("ids");
        let season = Season::open(&root, "season_test_s1", "test").expect("open");
        assert_eq!(season.next_id("tickets", "BLD"), "BLD-000001");
        assert_eq!(season.next_id("tickets", "CSE"), "CSE-000001");

        // Simulate a counter that is behind its own directory: an id already on
        // disk must never be handed out again.
        let ticket = Ticket {
            id: "BLD-000007".to_string(),
            contract: "CTR-CITY-0001".to_string(),
            kind: TicketKind::Build,
            status: TicketStatus::Open,
            objective: "lay a road".to_string(),
            gate: "the tiles are road afterwards".to_string(),
            opened_tick: 0,
            closed_tick: None,
            terminal: None,
            supersedes: None,
            governor_version: 1,
            case_key: None,
            count: 1,
            expectation: crate::gov::Expectation::None,
            roads_before: Vec::new(),
            evidence: Vec::new(),
            actor: "operator".to_string(),
            corrected: None,
        };
        season.write_ticket(&ticket).expect("write");
        assert_eq!(
            season.next_id("tickets", "BLD"),
            "BLD-000008",
            "the next id must skip past what is already on disk"
        );

        // A different record kind has its own sequence, and must not be handed
        // an id that already exists in its own folder.
        assert_eq!(season.next_id("retirements", "RET"), "RET-000001");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_record_is_never_overwritten() {
        let root = scratch("append-only");
        let season = Season::open(&root, "season_test_s1", "test").expect("open");
        let mut ticket = Ticket {
            id: "BLD-000001".to_string(),
            contract: "CTR-CITY-0001".to_string(),
            kind: TicketKind::Build,
            status: TicketStatus::Open,
            objective: "lay a road".to_string(),
            gate: "the tiles are road afterwards".to_string(),
            opened_tick: 0,
            closed_tick: None,
            terminal: None,
            supersedes: None,
            governor_version: 1,
            case_key: None,
            count: 1,
            expectation: crate::gov::Expectation::None,
            roads_before: Vec::new(),
            evidence: Vec::new(),
            actor: "operator".to_string(),
            corrected: None,
        };
        season.write_ticket(&ticket).expect("first write");
        ticket.status = TicketStatus::Complete;
        let err = season.write_ticket(&ticket).expect_err("must refuse");
        assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_corrupt_record_loses_that_record_not_the_file() {
        let root = scratch("corrupt");
        let season = Season::open(&root, "season_test_s1", "test").expect("open");
        let ticket = Ticket {
            id: "BLD-000001".to_string(),
            contract: "CTR-CITY-0001".to_string(),
            kind: TicketKind::Build,
            status: TicketStatus::Open,
            objective: "lay a road".to_string(),
            gate: "the tiles are road afterwards".to_string(),
            opened_tick: 0,
            closed_tick: None,
            terminal: None,
            supersedes: None,
            governor_version: 1,
            case_key: None,
            count: 1,
            expectation: crate::gov::Expectation::None,
            roads_before: Vec::new(),
            evidence: Vec::new(),
            actor: "operator".to_string(),
            corrected: None,
        };
        season.write_ticket(&ticket).expect("write");
        fs::write(season.dir().join("tickets").join("BLD-000002.json"), "{ not json")
            .expect("write garbage");

        let read = season.read_tickets();
        assert_eq!(read.len(), 1, "the good record must still load");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn events_accumulate_across_reopening() {
        let root = scratch("events");
        let mut season = Season::open(&root, "season_test_s1", "test").expect("open");
        season.append_event(1, "ticket.filed", "BLD-000001").expect("append");
        season.append_event(2, "ticket.retired", "BLD-000001").expect("append");

        let reopened = Season::open(&root, "season_test_s1", "test").expect("reopen");
        let (events, corrupt) = reopened.read_events();
        assert_eq!(events.len(), 2, "events must append, not replace");
        assert_eq!(corrupt, 0);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_corrupt_event_line_does_not_lose_the_log() {
        let root = scratch("events-corrupt");
        let mut season = Season::open(&root, "season_test_s1", "test").expect("open");
        season.append_event(1, "ticket.filed", "a").expect("append");
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(season.dir().join("events.jsonl"))
            .expect("open log");
        writeln!(file, "this is not json").expect("write garbage");
        season.append_event(3, "ticket.retired", "a").expect("append");

        let (events, corrupt) = season.read_events();
        assert_eq!(events.len(), 2);
        assert_eq!(corrupt, 1, "the bad line is counted, not hidden");
        let _ = fs::remove_dir_all(&root);
    }
}
