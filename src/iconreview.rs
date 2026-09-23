//! The review hand-off between the icon pipeline and the picker.
//!
//! Extracted from the old picker (`src/bin/pick.rs`), because two pickers —
//! the old one still runnable per a135(c), the rebuilt one on the widget
//! layer — must read the same file and write the same record format. One
//! copy, or the two tools begin to disagree about what a decision is.
//!
//! A **mark** carries a target and the checked generation; a **note** carries
//! only a comment, which is how "none of these, change this" gets recorded.
//! The pipeline reads the last `target: true` per icon, so deciding again
//! supersedes without erasing — and every comment becomes a directive the
//! next generation is authored against.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde_json::Value;

pub const REVIEW_JSON: &str = "assets/icons/review.json";
pub const DECISIONS: &str = "assets/icons/review-decisions.jsonl";
/// What the external validator recorded about each icon's object (Q206). The
/// picker reads it and the pipeline writes it; neither owns it alone.
pub const VALIDATION: &str = "assets/icons/validation.jsonl";
pub const ASSETS: &str = "assets/icons";
pub const AUTHORING_PHASE: &str = "icon-authoring-review";
pub const CANDIDATE_COUNT: usize = 6;

#[derive(Clone, Debug)]
pub struct Review {
    pub concept_set: String,
    pub decision_px: f32,
    pub recognition_px: f32,
    pub context_px: f32,
    pub fills: HashMap<String, [f32; 4]>,
    pub host_fills: HashMap<String, [f32; 4]>,
    pub blender: String,
    pub palette_hash: String,
    pub icons: Vec<Icon>,
}

#[derive(Clone, Debug)]
pub struct Icon {
    pub id: String,
    pub kind: String,
    pub meaning: String,
    pub locates: String,
    pub sits_on: Vec<String>,
    pub identity: Option<String>,
    pub identity_as: String,
    /// Comments the pipeline already carries forward for this icon.
    pub directives: Vec<String>,
    pub generations: Vec<Generation>,
    pub brief: Value,
    pub lineage: Vec<String>,
    pub forbidden_readings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Generation {
    pub id: String,
    pub label: String,
    pub why: String,
    pub sharp: PathBuf,
    pub recognition: PathBuf,
    pub context: PathBuf,
    pub measurements: Value,
    pub checks: Value,
    pub gate: Value,
    pub brief: Value,
}

fn string(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn strings(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn rgba(value: &Value) -> [f32; 4] {
    let channel =
        |index: usize| value.get(index).and_then(Value::as_f64).unwrap_or(0.0) as f32;
    [channel(0), channel(1), channel(2), 1.0]
}

pub fn load_review() -> Result<Review, String> {
    let text = std::fs::read_to_string(REVIEW_JSON)
        .map_err(|err| format!("could not read {REVIEW_JSON}: {err}"))?;
    let root: Value = serde_json::from_str(&text)
        .map_err(|err| format!("{REVIEW_JSON} is not valid JSON: {err}"))?;

    let number = |value: &Value, key: &str| {
        value.get(key).and_then(Value::as_f64).unwrap_or(0.0) as f32
    };
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
        let directives = strings(entry.get("directives"));
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
                gate: generation
                    .get("selection_notes")
                    .cloned()
                    .unwrap_or(Value::Array(Vec::new())),
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
            sits_on: strings(entry.get("sits_on")),
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
        palette_hash: string(&root, "palette_hash"),
        icons,
    })
}

/// Validate the authoring hand-off before opening a window. A malformed manifest
/// must refuse here instead of turning into a missing texture, an out-of-bounds
/// generation, or a misleading blank candidate in the review phase.
pub fn review_defects(review: &Review) -> Vec<String> {
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
                icon.id,
                icon.generations.len(),
                CANDIDATE_COUNT
            ));
        }
        if icon.brief.is_null() || icon.lineage.is_empty() || icon.forbidden_readings.is_empty() {
            defects.push(format!(
                "icon `{}` is missing its full brief, lineage, or forbidden readings",
                icon.id
            ));
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
    pub fn fill(&self, name: &str) -> [f32; 4] {
        self.fills.get(name).copied().unwrap_or([0.1, 0.1, 0.12, 1.0])
    }

    /// The surface the icon declares first. Painting every candidate on the same
    /// surface is what makes the three comparable; painting it on *its own* declared
    /// surface is what makes the contrast claim checkable by eye.
    pub fn host_fill(&self, icon: &Icon) -> [f32; 4] {
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

    pub fn header(&self) -> String {
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
    pub fn facts(&self) -> String {
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
            self.measurements.get("96").map(|_| "96").unwrap_or("96"),
            coverage("96")
        ));
        if let Some(internal) = self.checks.get("internal_contrast").and_then(Value::as_f64) {
            parts.push(format!("accent {internal:.2}:1"));
        }
        parts.join(" · ")
    }

    pub fn notes(&self) -> Vec<String> {
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

/// What an instrument outside this pipeline recorded about one icon's object.
#[derive(Clone, Debug, PartialEq)]
pub struct Reading {
    /// The family the icon declares as its own reading. `tool-power`'s is the bolt.
    pub expected: String,
    /// Every λ sample of the ladder read as that family. This is Q206's gate.
    pub reads_as_own_object: bool,
    /// The five samples were read as one object rather than as several.
    pub ladder_is_one_object: bool,
    /// Whatever families the ladder *was* read as — the refusal's evidence.
    pub distinct_reads: Vec<String>,
    /// Whether the run that produced this reading passed its own calibration.
    pub calibrated: bool,
    /// The corpus marks that run identified, against which the reading is read.
    pub ceiling: String,
    pub model: String,
}

/// The last reading recorded per icon, which is the one that stands.
///
/// Deciding again supersedes without erasing, exactly as a `target: true` line
/// does: the lens's geometry was corrected after its first reading, and a gate
/// that averaged the two would gate the object that no longer exists.
///
/// A ledger that is absent or unreadable yields no readings rather than an
/// error — the caller's refusal says what that means.
pub fn readings(path: &Path) -> HashMap<String, Reading> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return HashMap::new();
    };
    let mut out: HashMap<String, Reading> = HashMap::new();
    for line in text.lines() {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if value.get("kind").and_then(Value::as_str) != Some("reading") {
            continue;
        }
        let Some(icon) = value.get("icon").and_then(Value::as_str) else {
            continue;
        };
        let calibration = value.get("calibration");
        out.insert(
            icon.to_string(),
            Reading {
                expected: text_of(&value, "expected"),
                reads_as_own_object: flag(&value, "reads_as_own_object"),
                ladder_is_one_object: flag(&value, "ladder_is_one_object"),
                distinct_reads: value
                    .get("distinct_reads")
                    .and_then(Value::as_array)
                    .map(|readings| readings.iter().filter_map(string_of).collect())
                    .unwrap_or_default(),
                calibrated: calibration
                    .and_then(|c| c.get("calibrated"))
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                ceiling: calibration
                    .and_then(|c| c.get("ceiling"))
                    .and_then(Value::as_str)
                    .unwrap_or("none")
                    .to_string(),
                model: text_of(&value, "model"),
            },
        );
    }
    out
}

fn string_of(value: &Value) -> Option<String> {
    value.as_str().map(str::to_string)
}

fn flag(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn text_of(value: &Value, key: &str) -> String {
    value.get(key).and_then(Value::as_str).unwrap_or("").to_string()
}

/// Why a promotion may not be recorded, or `None` when it may (Q206).
///
/// Q206 answered that the validator's verdict **gates** promotion rather than
/// merely being filed, and Q211 added the escape: a candidate that does not read
/// as its own object may still be promoted by a person who **says why in the same
/// act**. So the gate has exactly one door, and it is a reason on the record.
///
/// `reason` is the mark's comment — already recorded with the mark, already the
/// field a directive is read back from, so an override needs no new file, no new
/// UI and no separate act to be honest about:
///
/// * a reason typed with the mark is the override, and it is written down;
/// * no reason, a missing reading, or an uncalibrated run refuses the mark.
///
/// Failing closed matters more here than anywhere else in the pipeline: the
/// refusal is the only thing standing between a person's click and an icon that
/// nothing outside the pipeline has ever confirmed a player will recognise.
pub fn promotion_refusal(icon: &Icon, reason: &str, path: &Path) -> Option<String> {
    if !reason.trim().is_empty() {
        return None;
    }
    let readings = readings(path);
    let Some(reading) = readings.get(&icon.id) else {
        return Some(format!(
            "`{}` has no reading in {}: nothing outside this pipeline has said its \
             object reads as itself, so there is no evidence to promote. Mark it again \
             with the reason typed into the comment box — the reason is recorded with \
             the mark, which is what an override is here.",
            icon.id,
            path.display()
        ));
    };
    if !reading.calibrated {
        return Some(format!(
            "the run that read `{}` did not pass its own calibration (ceiling {}), so \
             its verdict is not evidence either way. Re-run the validator, or mark with \
             a reason to promote on a person's judgement.",
            icon.id, reading.ceiling
        ));
    }
    if reading.reads_as_own_object {
        return None;
    }
    let read_as = if reading.distinct_reads.is_empty() {
        "something its declaration does not name".to_string()
    } else {
        reading.distinct_reads.join(", ")
    };
    Some(format!(
        "{} read `{}` as {} where it declares {} — ceiling {} on the corpus's own \
         marks, ladder one object: {} — so it is not confirmed that a player sees the \
         object it means. Mark with the reason typed into the comment box to promote it \
         anyway.",
        reading.model,
        icon.id,
        read_as,
        reading.expected,
        reading.ceiling,
        reading.ladder_is_one_object
    ))
}

/// One line, appended. A **mark** carries a target and the checked generation; a
/// **note** carries only a comment. Returns the JSON line it wrote.
///
/// A mark is a promotion, so it passes the gate first (Q206): `Err` carries the
/// refusal and nothing is written, which is how the refusal stays a refusal
/// rather than becoming a note nobody reads.
///
/// `validation` is the ledger the gate reads. It is a parameter rather than the
/// constant so the gate can be tested against a fixture — and a caller that
/// passes a path holding no readings gets a refusal, not a free pass.
pub fn record(
    review: &Review,
    icon: &Icon,
    position: usize,
    target: bool,
    comment: &str,
    path: &Path,
    validation: &Path,
) -> Result<String, String> {
    if target {
        if let Some(refusal) = promotion_refusal(icon, comment, validation) {
            return Err(refusal);
        }
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
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
        "at_unix_seconds": now,
        "by": "ala-cities pick",
        "phase": AUTHORING_PHASE,
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
    Ok(line)
}

/// Every comment recorded for an icon, oldest first, from the picker's own log — so
/// the comments shown next to an icon include the newest ones even before the
/// pipeline has re-read them.
pub fn read_directives(path: &Path, concept_set: &str) -> HashMap<String, Vec<String>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The recorded line carries exactly one `concept_set` key. The extraction
    /// found the old builder writing it three times in one object literal —
    /// serde_json silently keeps the last, so the record was never wrong, but
    /// the file it came from was nonsense waiting to be edited.
    #[test]
    fn the_recorded_line_carries_one_concept_set() {
        let review = Review {
            concept_set: "test-set".into(),
            decision_px: 96.0,
            recognition_px: 24.0,
            context_px: 24.0,
            fills: HashMap::new(),
            host_fills: HashMap::new(),
            blender: "Blender test".into(),
            palette_hash: "abc".into(),
            icons: Vec::new(),
        };
        let icon = Icon {
            id: "tool-road".into(),
            kind: "tool".into(),
            meaning: "the road tool".into(),
            locates: "Tool::Road".into(),
            sits_on: vec!["PanelRaised".into()],
            identity: None,
            identity_as: String::new(),
            directives: Vec::new(),
            generations: vec![Generation {
                id: "a-canonical".into(),
                label: "canonical".into(),
                why: "the clearest authored reading".into(),
                sharp: PathBuf::new(),
                recognition: PathBuf::new(),
                context: PathBuf::new(),
                measurements: Value::Null,
                checks: Value::Null,
                gate: Value::Array(Vec::new()),
                brief: Value::Null,
            }],
            brief: Value::Null,
            lineage: vec!["MD1".into()],
            forbidden_readings: vec!["capability grant".into()],
        };
        let path = std::env::temp_dir().join("ala-cities-c7-record-test.jsonl");
        let line = record(
            &review,
            &icon,
            0,
            true,
            "sharpen the tab",
            &path,
            Path::new(VALIDATION),
        )
        .expect("a reason with the mark is an override, so this records");
        let parsed: Value = serde_json::from_str(&line).expect("the line is valid JSON");
        assert_eq!(parsed.get("concept_set").and_then(Value::as_str), Some("test-set"));
        assert_eq!(parsed.get("target").and_then(Value::as_bool), Some(true));
        assert_eq!(
            parsed.get("comment").and_then(Value::as_str),
            Some("sharpen the tab")
        );
        let _ = std::fs::remove_file(&path);
    }

    /// Q206's gate, tested the only way a gate can be evidence: it has to refuse.
    ///
    /// The three things it must not let through are a mark on an object nothing has
    /// confirmed reads as itself, a mark whose only support is a run that failed its
    /// own calibration, and a mark on an icon no run has read at all. The one thing
    /// it must let through is a person's override — the mark with a reason typed
    /// into the comment box, which is recorded with the mark.
    #[test]
    fn a_promotion_without_a_reason_fails_closed() {
        let review = Review {
            concept_set: "test-set".into(),
            decision_px: 96.0,
            recognition_px: 24.0,
            context_px: 24.0,
            fills: HashMap::new(),
            host_fills: HashMap::new(),
            blender: "Blender test".into(),
            palette_hash: "abc".into(),
            icons: Vec::new(),
        };
        let icon = Icon {
            id: "tool-inspect".into(),
            kind: "tool".into(),
            meaning: "the inspect tool".into(),
            locates: "Tool::Inspect".into(),
            sits_on: vec!["PanelRaised".into()],
            identity: None,
            identity_as: String::new(),
            directives: Vec::new(),
            generations: vec![Generation {
                id: "a-canonical".into(),
                label: "canonical".into(),
                why: "the clearest authored reading".into(),
                sharp: PathBuf::new(),
                recognition: PathBuf::new(),
                context: PathBuf::new(),
                measurements: Value::Null,
                checks: Value::Null,
                gate: Value::Array(Vec::new()),
                brief: Value::Null,
            }],
            brief: Value::Null,
            lineage: vec!["MD1".into()],
            forbidden_readings: vec!["capability grant".into()],
        };

        let ledger = std::env::temp_dir().join("ala-cities-c8-gate-ledger.jsonl");
        let decisions = std::env::temp_dir().join("ala-cities-c8-gate-decisions.jsonl");
        let reading = |reads_as_own_object: bool, calibrated: bool| {
            serde_json::json!({
                "kind": "reading",
                "icon": "tool-inspect",
                "expected": "lens",
                "reads_as_own_object": reads_as_own_object,
                "ladder_is_one_object": true,
                "distinct_reads": ["gear"],
                "model": "a test instrument",
                "calibration": { "calibrated": calibrated, "ceiling": "5/6" },
            })
            .to_string()
        };
        let write_ledger = |line: Option<String>| {
            let text = line.map(|line| format!("{line}\n")).unwrap_or_default();
            std::fs::write(&ledger, text).expect("the fixture ledger is written");
        };

        // Uncalibrated: the run's verdict is not evidence either way, so it refuses.
        write_ledger(Some(reading(false, false)));
        let refusal = promotion_refusal(&icon, "", &ledger).expect("an uncalibrated run refuses");
        assert!(refusal.contains("did not pass its own calibration"), "{refusal}");

        // Calibrated, and it does not read as the object it declares: refuse, naming
        // both what it was read as and what it declares.
        write_ledger(Some(reading(false, true)));
        let refusal = promotion_refusal(&icon, "", &ledger).expect("a misreading refuses");
        assert!(refusal.contains("as gear"), "{refusal}");
        assert!(refusal.contains("declares lens"), "{refusal}");
        assert!(
            record(&review, &icon, 0, true, "", &decisions, &ledger).is_err(),
            "a mark with no reason does not reach the decisions file"
        );
        assert!(!decisions.exists(), "nothing was written for the refused mark");

        // The override: the same mark, with the reason typed in, is recorded.
        let line = record(&review, &icon, 0, true, "the handle reads at 24px, ship it",
                          &decisions, &ledger)
            .expect("a reason with the mark is the override");
        assert!(line.contains("the handle reads at 24px"), "{line}");

        // A reading that holds: no refusal, and no reason needed.
        write_ledger(Some(reading(true, true)));
        assert_eq!(promotion_refusal(&icon, "", &ledger), None);

        // No reading at all is not a free pass; it is the absence of evidence.
        write_ledger(None);
        let refusal = promotion_refusal(&icon, "", &ledger).expect("no reading refuses");
        assert!(refusal.contains("has no reading in"), "{refusal}");

        let _ = std::fs::remove_file(&ledger);
        let _ = std::fs::remove_file(&decisions);
    }
}
