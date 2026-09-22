//! The agent's work ledger.
//!
//! C2 a73/a74, C6 Q127: the debugger pane **reads** this record; the agent's
//! own tool **writes** it; the two never share a process, which is what makes
//! "only the agent's ledger may set [`Action::InProgress`]" enforceable rather
//! than a convention. The writer lives in this module so both sides share one
//! record format — but the game binary never calls it.
//!
//! The honesty rule that shapes everything here (a73): the pane reports what
//! a record says, **with the age of that record**, and degrades to unknown
//! when the record goes quiet. A green lamp that says an agent is working
//! when nothing has happened for hours is the defect class this project
//! keeps catching. So [`Ledger::presence`] returns the claim *and* its age,
//! and lets the reader decide what "quiet" means — the threshold is a display
//! concern, not a storage one.

use std::path::Path;

use serde_json::Value;

/// Where a season's agent ledger lives, inside that season's directory.
pub const LEDGER_FILE: &str = "agent-ledger.jsonl";

/// What the agent did. Closed on purpose; a74's states, minus the ones the
/// governor already owns (`filed`, `closed`, `retired` are the season store's
/// words, not the ledger's).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// The agent has begun work on a ticket. **Only this ledger may say so.**
    InProgress,
    /// Work is stopped for a stated reason; the ticket is not closed.
    Blocked,
    /// A note with no state change — what was tried, what was seen.
    Note,
    /// The agent released its claim: work stopped, the claim is withdrawn.
    Released,
}

impl Action {
    pub fn token(self) -> &'static str {
        match self {
            Action::InProgress => "in_progress",
            Action::Blocked => "blocked",
            Action::Note => "note",
            Action::Released => "released",
        }
    }

    pub fn from_token(token: &str) -> Option<Action> {
        match token {
            "in_progress" => Some(Action::InProgress),
            "blocked" => Some(Action::Blocked),
            "note" => Some(Action::Note),
            "released" => Some(Action::Released),
            _ => None,
        }
    }
}

/// One line of the ledger.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub at_unix_ms: u128,
    pub agent: String,
    pub ticket: String,
    pub action: Action,
    pub detail: String,
}

/// Append one entry. The *only* way `in_progress` enters the world — and the
/// game binary never calls this.
pub fn append(path: &Path, agent: &str, ticket: &str, action: Action, detail: &str) -> std::io::Result<()> {
    let entry = serde_json::json!({
        "at_unix_ms": now_unix_ms(),
        "agent": agent,
        "ticket": ticket,
        "action": action.token(),
        "detail": detail,
    });
    let mut line = serde_json::to_string(&entry).unwrap_or_default();
    line.push('\n');
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
    use std::io::Write;
    file.write_all(line.as_bytes())
}

/// Read every entry, oldest first. A missing file is an empty ledger — before
/// the agent has ever worked, the honest record is no record, not an error.
pub fn read(path: &Path) -> Vec<Entry> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for line in text.lines() {
        let Ok(record) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let Some(action) = record
            .get("action")
            .and_then(Value::as_str)
            .and_then(Action::from_token)
        else {
            continue;
        };
        let string_at = |key: &str| {
            record
                .get(key)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string()
        };
        out.push(Entry {
            at_unix_ms: record
                .get("at_unix_ms")
                .and_then(Value::as_u64)
                .unwrap_or(0) as u128,
            agent: string_at("agent"),
            ticket: string_at("ticket"),
            action,
            detail: string_at("detail"),
        });
    }
    out
}

/// What the pane can honestly say about the agent's presence right now.
#[derive(Clone, Debug, PartialEq)]
pub enum Presence {
    /// No ledger exists: nothing is claimed, nothing is known.
    NoRecord,
    /// The last entry, and how long ago it was written.
    Last { entry: Entry, age_seconds: u64 },
}

/// The presence claim as of `now_unix_ms`. The staleness threshold is the
/// display's decision; this returns the facts.
pub fn presence(path: &Path, now_unix_ms: u128) -> Presence {
    let entries = read(path);
    let Some(last) = entries.last() else {
        return Presence::NoRecord;
    };
    let age = now_unix_ms.saturating_sub(last.at_unix_ms);
    Presence::Last {
        entry: last.clone(),
        age_seconds: (age / 1000) as u64,
    }
}

/// Is there an open agent claim (`in_progress` with no later `released` or
/// `blocked`) on this ticket?
pub fn claim_open(path: &Path, ticket: &str) -> bool {
    let mut open = false;
    for entry in read(path) {
        if entry.ticket != ticket {
            continue;
        }
        match entry.action {
            Action::InProgress => open = true,
            Action::Released | Action::Blocked => open = false,
            Action::Note => {}
        }
    }
    open
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

    fn temp(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "ala-cities-c7-agentledger-{name}.jsonl",
        ))
    }

    #[test]
    fn an_absent_ledger_is_no_record_not_an_error() {
        let path = temp("absent");
        let _ = std::fs::remove_file(&path);
        assert_eq!(presence(&path, now_unix_ms()), Presence::NoRecord);
        assert!(!claim_open(&path, "BLD-000001"));
    }

    #[test]
    fn a_claim_opens_and_releases_and_the_record_keeps_its_age() {
        let path = temp("claim");
        let _ = std::fs::remove_file(&path);
        append(&path, "codebuff", "BLD-000001", Action::InProgress, "starting the atlas fix")
            .expect("append works");
        assert!(claim_open(&path, "BLD-000001"));

        let now = now_unix_ms();
        match presence(&path, now) {
            Presence::Last { entry, age_seconds } => {
                assert_eq!(entry.action, Action::InProgress);
                assert_eq!(entry.ticket, "BLD-000001");
                assert!(age_seconds <= 5, "a fresh record is not old ({age_seconds}s)");
            }
            Presence::NoRecord => panic!("a written record must read back"),
        }

        append(&path, "codebuff", "BLD-000001", Action::Released, "done and verified")
            .expect("append works");
        assert!(!claim_open(&path, "BLD-000001"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_claim_on_one_ticket_does_not_leak_to_another() {
        let path = temp("leak");
        let _ = std::fs::remove_file(&path);
        append(&path, "codebuff", "BLD-000001", Action::InProgress, "working").unwrap();
        assert!(!claim_open(&path, "BLD-000002"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_quiet_record_carries_its_age_forward() {
        // The staleness display rule: age grows with the clock. A record
        // written "hours ago" reads as hours old, whatever its action says.
        let path = temp("quiet");
        let _ = std::fs::remove_file(&path);
        append(&path, "codebuff", "CSE-000001", Action::Note, "observing").unwrap();
        let hours_later = now_unix_ms() + 3 * 3600 * 1000;
        match presence(&path, hours_later) {
            Presence::Last { age_seconds, .. } => {
                assert!(age_seconds >= 3 * 3600, "age must reflect the clock");
            }
            Presence::NoRecord => panic!("the record exists"),
        }
        let _ = std::fs::remove_file(&path);
    }
}
