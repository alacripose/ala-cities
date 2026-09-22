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

/// One line, appended. A **mark** carries a target and the checked generation; a
/// **note** carries only a comment. Returns the JSON line it wrote.
pub fn record(
    review: &Review,
    icon: &Icon,
    position: usize,
    target: bool,
    comment: &str,
    path: &Path,
) -> String {
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
    line
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
        let line = record(&review, &icon, 0, true, "sharpen the tab", &path);
        let parsed: Value = serde_json::from_str(&line).expect("the line is valid JSON");
        assert_eq!(parsed.get("concept_set").and_then(Value::as_str), Some("test-set"));
        assert_eq!(parsed.get("target").and_then(Value::as_bool), Some(true));
        assert_eq!(
            parsed.get("comment").and_then(Value::as_str),
            Some("sharpen the tab")
        );
        let _ = std::fs::remove_file(&path);
    }
}
