//! What you did, and what you thought of it.
//!
//! Two artefacts, both tracked in the repository rather than sent anywhere:
//!
//! * an **interaction capture** — every click, its screen position, the tile it
//!   landed on, the tool that was active, and the ticket it filed;
//! * a **feedback record**, available from the pause menu at any time, shaped
//!   as a lesson with its confidence stated rather than implied.
//!
//! Nothing here is networked. Recording is a state, so the chrome says so
//! truthfully rather than recording silently.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::gov::season::now_unix_ms;

/// One recorded interaction. Wall-clock time is recorded for the human reading
/// it back; it is never an input to the simulation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Interaction {
    pub tick: u64,
    pub wall_ms: u128,
    pub kind: String,
    pub tool: Option<String>,
    pub screen: Option<(f32, f32)>,
    pub tile: Option<(i32, i32)>,
    pub ticket: Option<String>,
    pub detail: String,
}

#[derive(Clone, Debug)]
pub struct Session {
    pub stage: String,
    pub id: String,
    pub build: String,
    pub recording: bool,
    /// Set when the capture hit its cap. The cap is a refusal, not a silent
    /// truncation: past it, events stop and the file says why.
    pub refused: bool,
    path: PathBuf,
    events: Vec<Interaction>,
    cap: usize,
}

impl Session {
    pub fn open(stage: &str, build: &str) -> std::io::Result<Self> {
        let id = format!("session-{}", now_unix_ms());
        let dir = Path::new("playtest").join(stage).join("sessions");
        fs::create_dir_all(&dir)?;
        Ok(Self {
            stage: stage.to_string(),
            id: id.clone(),
            build: build.to_string(),
            recording: true,
            refused: false,
            path: dir.join(format!("{id}.jsonl")),
            events: Vec::new(),
            cap: 20_000,
        })
    }

    pub fn count(&self) -> usize {
        self.events.len()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn record(&mut self, interaction: Interaction) {
        if !self.recording {
            return;
        }
        if self.events.len() >= self.cap {
            if !self.refused {
                self.refused = true;
                let note = Interaction {
                    tick: interaction.tick,
                    wall_ms: interaction.wall_ms,
                    kind: "capture.refused".to_string(),
                    tool: None,
                    screen: None,
                    tile: None,
                    ticket: None,
                    detail: format!(
                        "capture reached its cap of {} events and refused further recording rather than dropping events silently",
                        self.cap
                    ),
                };
                self.events.push(note);
            }
            return;
        }
        self.events.push(interaction);
    }

    pub fn flush(&self) -> std::io::Result<usize> {
        let mut file = fs::File::create(&self.path)?;
        writeln!(
            file,
            "{}",
            serde_json::to_string(&serde_json::json!({
                "kind": "session.header",
                "stage": self.stage,
                "session": self.id,
                "build": self.build,
                "opened_wall_ms": now_unix_ms(),
                "recording": self.recording,
            }))?
        )?;
        for event in &self.events {
            writeln!(file, "{}", serde_json::to_string(event)?)?;
        }
        Ok(self.events.len())
    }
}

/// Write a feedback record shaped as a lesson: observation, evidence,
/// confidence, applicability, limitations. One playtest is one playtest, and
/// the record has to say so rather than reading like a finding.
pub fn write_feedback(
    stage: &str,
    session: &Session,
    text: &str,
    extra: &[String],
) -> std::io::Result<PathBuf> {
    let dir = Path::new("playtest").join(stage).join("feedback");
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("feedback-{}.md", now_unix_ms()));

    let mut body = String::new();
    body.push_str(&format!("# Playtest feedback — {stage}\n\n"));
    body.push_str(&format!("- **session**: `{}`\n", session.id));
    body.push_str(&format!("- **build**: `{}`\n", session.build));
    body.push_str(&format!("- **captured**: {} (unix ms)\n", now_unix_ms()));
    body.push_str(&format!(
        "- **interactions recorded**: {} in this session\n",
        session.count()
    ));
    body.push_str(&format!(
        "- **capture file**: `playtest/{stage}/sessions/{}.jsonl`\n",
        session.id
    ));
    body.push_str("- **confidence**: `tentative`\n\n");
    body.push_str("## Observation\n\n");
    body.push_str(text.trim());
    body.push_str("\n\n## Supporting evidence\n\n");
    if extra.is_empty() {
        body.push_str("- None offered with this record.\n");
    } else {
        for line in extra {
            body.push_str(&format!("- {line}\n"));
        }
    }
    body.push_str("\n## Applicability and limitations\n\n");
    body.push_str("- Conditions: this playtest only, on this build.\n");
    body.push_str("- Limitations: a single session, with no baseline to compare against.\n");
    body.push_str("- A claim with one playtest behind it must not read like one with ten.\n");

    fs::write(&path, body)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_capture_refuses_rather_than_truncating() {
        let mut session = Session::open("TEST", "test-build").expect("open");
        session.cap = 3;
        for i in 0..10u64 {
            session.record(Interaction {
                tick: i,
                wall_ms: 0,
                kind: "click".to_string(),
                tool: None,
                screen: None,
                tile: None,
                ticket: None,
                detail: String::new(),
            });
        }
        assert!(session.refused, "the cap must be recorded as a refusal");
        assert_eq!(session.count(), 4, "three events plus the refusal notice");
        assert_eq!(
            session.events.last().expect("a last event").kind,
            "capture.refused"
        );
        let _ = fs::remove_file(session.path());
    }

    #[test]
    fn feedback_is_shaped_as_a_lesson_at_tentative_confidence() {
        let session = Session::open("TEST", "test-build").expect("open");
        let path = write_feedback("TEST", &session, "Roads feel slow.", &[]).expect("write");
        let text = fs::read_to_string(&path).expect("read back");
        assert!(text.contains("confidence**: `tentative`"));
        assert!(text.contains("## Observation"));
        assert!(text.contains("## Applicability and limitations"));
        assert!(text.contains("Roads feel slow."));
        let _ = fs::remove_file(&path);
    }
}
