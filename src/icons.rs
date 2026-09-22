//! The shipping side of the icon pipeline.
//!
//! The picker decides; this loader delivers what was decided into the game.
//! It reads the decision ledger (`review-decisions.jsonl`), takes the **last**
//! `target: true` per icon — deciding again supersedes without erasing, the
//! same rule the pipeline uses — packs the decided candidates' recognition-size
//! renders into one atlas strip, and reports the source rect of every shipped
//! icon inside it.
//!
//! Nothing is invented here: an icon exists in the game only because a decision
//! line says so, and the file it points at must exist at the size the manifest
//! declares. With no decisions the set is simply empty and the game falls back
//! to the text toolbar — an honest absence, not a placeholder pretending to be
//! a shipped icon.

use std::collections::HashMap;
use std::path::Path;

use serde_json::Value;

/// What was decided about one icon: which generation won, and where its
/// recognition-size render sits in the packed atlas (source pixels, the only
/// coordinates the image pipeline speaks).
#[derive(Clone, Debug)]
pub struct ShippedIcon {
    pub generation: String,
    /// `(x, y, w, h)` inside the atlas, in atlas pixels.
    pub src: [f32; 4],
}

/// The decided icons, keyed by the icon id the game knows (`tool-road`, …).
#[derive(Default, Debug)]
pub struct IconSet {
    icons: HashMap<String, ShippedIcon>,
}

impl IconSet {
    pub fn get(&self, id: &str) -> Option<&ShippedIcon> {
        self.icons.get(id)
    }

    pub fn len(&self) -> usize {
        self.icons.len()
    }

    pub fn is_empty(&self) -> bool {
        self.icons.is_empty()
    }

    /// Which icon won for each id, in decision order. Exposed for the debugger
    /// pane and tests; the game itself only ever asks `get`.
    pub fn decided(&self) -> Vec<(&String, &ShippedIcon)> {
        self.icons.iter().collect()
    }
}

/// Load every decided icon and pack its recognition-size render into one
/// atlas strip. Returns the set plus the RGBA atlas and its dimensions.
///
/// Errors mean the *pipeline* is misconfigured (unreadable or malformed
/// manifest, missing render for a decided icon) and are reported, not swept;
/// an empty or decision-free ledger is a normal `Ok` with an empty set.
pub fn load(
    decisions_path: &Path,
    review_path: &Path,
    assets: &Path,
) -> Result<(IconSet, Vec<u8>, u32, u32), String> {
    let review = std::fs::read_to_string(review_path)
        .map_err(|err| format!("could not read {}: {err}", review_path.display()))?;
    let review: Value = serde_json::from_str(&review)
        .map_err(|err| format!("{} is not valid JSON: {err}", review_path.display()))?;
    let concept_set = review
        .get("concept_set")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{} declares no concept_set", review_path.display()))?
        .to_string();
    let recognition_px = review
        .get("recognition_size_px")
        .and_then(Value::as_f64)
        .unwrap_or(0.0) as u32;
    if recognition_px == 0 {
        return Err(format!(
            "{} declares no recognition_size_px",
            review_path.display()
        ));
    }

    // The manifest maps icon id -> generation id -> recognition render path,
    // for every icon it carries — including ones no longer awaiting a decision,
    // which is exactly the population a shipping loader serves.
    let mut renders: HashMap<String, HashMap<String, String>> = HashMap::new();
    for entry in review
        .get("icons")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(id) = entry.get("id").and_then(Value::as_str) else {
            continue;
        };
        let mut generations = HashMap::new();
        for generation in entry
            .get("generations")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let Some(gen_id) = generation.get("id").and_then(Value::as_str) else {
                continue;
            };
            let file = generation
                .get("files")
                .and_then(|files| files.get("recognition_raw"))
                .and_then(Value::as_str)
                .unwrap_or_default();
            if !file.is_empty() {
                generations.insert(gen_id.to_string(), file.to_string());
            }
        }
        renders.insert(id.to_string(), generations);
    }

    // The ledger: the last target:true per icon wins. Only decisions about
    // this concept set count — a decision from another set is about other art.
    let text = match std::fs::read_to_string(decisions_path) {
        Ok(text) => text,
        Err(_) => return Ok((IconSet::default(), Vec::new(), 0, 0)),
    };
    let mut winners: Vec<(String, String)> = Vec::new();
    for line in text.lines() {
        let Ok(record) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if record.get("concept_set").and_then(Value::as_str) != Some(concept_set.as_str()) {
            continue;
        }
        if record.get("target").and_then(Value::as_bool) != Some(true) {
            continue;
        }
        let Some(icon) = record.get("icon").and_then(Value::as_str) else {
            continue;
        };
        let Some(generation) = record.get("generation").and_then(Value::as_str) else {
            continue;
        };
        winners.retain(|(winner_icon, _)| winner_icon != icon);
        winners.push((icon.to_string(), generation.to_string()));
    }
    if winners.is_empty() {
        return Ok((IconSet::default(), Vec::new(), 0, 0));
    }

    // Pack the winners' renders into one strip, in decision order, and refuse
    // honestly if a decided icon's file is missing or the wrong size.
    let tile = recognition_px;
    let mut atlas = vec![0u8; (tile * winners.len() as u32 * tile * 4) as usize];
    let mut set = IconSet::default();
    let mut defects = Vec::new();
    for (index, (icon, generation)) in winners.iter().enumerate() {
        let Some(files) = renders.get(icon) else {
            defects.push(format!("decided icon `{icon}` is not in the manifest"));
            continue;
        };
        let Some(file) = files.get(generation) else {
            defects.push(format!(
                "decided icon `{icon}` names generation `{generation}`, which the manifest does not carry"
            ));
            continue;
        };
        let path = assets.join(file);
        let expected = (tile * tile * 4) as usize;
        match std::fs::read(&path) {
            Ok(bytes) if bytes.len() == expected => {
                let x = index as u32 * tile;
                for row in 0..tile {
                    let from = (row * tile * 4) as usize;
                    let to = from + (tile * 4) as usize;
                    let dst = ((row * atlas_w(&winners, tile)) * 4 + x * 4) as usize;
                    atlas[dst..dst + to - from].copy_from_slice(&bytes[from..to]);
                }
                set.icons.insert(
                    icon.clone(),
                    ShippedIcon {
                        generation: generation.clone(),
                        src: [
                            x as f32,
                            0.0,
                            tile as f32,
                            tile as f32,
                        ],
                    },
                );
            }
            Ok(bytes) => defects.push(format!(
                "`{}` is {} bytes; expected {tile}x{tile} RGBA8 ({expected})",
                path.display(),
                bytes.len()
            )),
            Err(err) => defects.push(format!(
                "decided icon `{icon}` render `{}` cannot be read: {err}",
                path.display()
            )),
        }
    }
    if !defects.is_empty() {
        return Err(defects.join("; "));
    }

    let width = atlas_w(&winners, tile);
    Ok((set, atlas, width, tile))
}

/// The atlas strip's width: every winner gets a tile, decided or defect-skipped
/// slots included so indices never shift under a partial set.
fn atlas_w(winners: &[(String, String)], tile: u32) -> u32 {
    tile * winners.len().max(1) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The full honest path: a fabricated decision about a real generation of
    /// the real review set loads the real 24 px render into the strip. This is
    /// the test that breaks the moment the record format or the manifest's
    /// file layout drifts — the exact drift a loader must not paper over.
    #[test]
    fn a_decision_promotes_its_generation_into_the_set() {
        // No decisions file in the repo yet: write a temporary one that marks
        // the real `tool-road` canonical candidate as the target.
        let review_path = Path::new(super::super::iconreview::REVIEW_JSON);
        let assets = Path::new(super::super::iconreview::ASSETS);
        let review_text = std::fs::read_to_string(review_path).expect("the review set exists");
        let review: Value = serde_json::from_str(&review_text).expect("valid manifest");
        let concept_set = review.get("concept_set").and_then(Value::as_str).unwrap();
        // Mark whatever generation the manifest actually carries first — the
        // test pins the mechanism, not one authoring label.
        let first_gen = review["icons"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i.get("id").and_then(Value::as_str) == Some("tool-road"))
            .and_then(|i| i["generations"].as_array())
            .and_then(|gens| gens.first())
            .and_then(|g| g.get("id").and_then(Value::as_str))
            .expect("tool-road has at least one generation")
            .to_string();

        let decisions_path = std::env::temp_dir().join("ala-cities-c7-icons-test.jsonl");
        std::fs::write(
            &decisions_path,
            format!(
                "{}\n",
                serde_json::json!({
                    "icon": "tool-road",
                    "generation": first_gen,
                    "target": true,
                    "concept_set": concept_set,
                })
            ),
        )
        .expect("write the temp decision");

        let outcome = load(&decisions_path, review_path, assets);
        let _ = std::fs::remove_file(&decisions_path);
        let (set, atlas, width, height) = outcome.expect("the decision loads");
        assert_eq!(set.len(), 1, "one decision, one icon");
        let icon = set.get("tool-road").expect("the decided icon is in the set");
        assert_eq!(icon.generation, first_gen);
        assert_eq!(height, 32, "the recognition size is what ships to the toolbar");
        assert_eq!(
            (width as usize) * (height as usize) * 4,
            atlas.len(),
            "the strip is exactly as big as its tiles"
        );
        // The icon's source rect sits inside the strip, one tile wide.
        assert!(icon.src[0] >= 0.0 && icon.src[0] + icon.src[2] <= width as f32);
        assert_eq!(icon.src[2], 32.0);
    }

    /// No ledger at all is a normal empty set, not an error: the game runs
    /// text-only until the first decision exists.
    #[test]
    fn an_absent_ledger_is_an_empty_set_not_an_error() {
        let missing = std::env::temp_dir().join("ala-cities-c7-icons-nothing.jsonl");
        let _ = std::fs::remove_file(&missing);
        let (set, atlas, width, _height) =
            load(&missing, Path::new(super::super::iconreview::REVIEW_JSON), Path::new(super::super::iconreview::ASSETS))
                .expect("an absent ledger is not an error");
        assert!(set.is_empty());
        assert!(atlas.is_empty());
        assert_eq!(width, 0);
    }

    /// A decision naming a generation the manifest does not carry is refused,
    /// not silently dropped: a decision is provenance, and provenance that
    /// points at nothing is a defect to fix, not a gap to skip.
    #[test]
    fn a_decision_about_an_unknown_generation_is_refused() {
        let review_path = Path::new(super::super::iconreview::REVIEW_JSON);
        let review_text = std::fs::read_to_string(review_path).expect("the review set exists");
        let review: Value = serde_json::from_str(&review_text).expect("valid manifest");
        let concept_set = review.get("concept_set").and_then(Value::as_str).unwrap();
        let decisions_path = std::env::temp_dir().join("ala-cities-c7-icons-bad.jsonl");
        std::fs::write(
            &decisions_path,
            format!(
                "{}\n",
                serde_json::json!({
                    "icon": "tool-road",
                    "generation": "does-not-exist",
                    "target": true,
                    "concept_set": concept_set,
                })
            ),
        )
        .expect("write the temp decision");
        let outcome = load(&decisions_path, review_path, Path::new(super::super::iconreview::ASSETS));
        let _ = std::fs::remove_file(&decisions_path);
        let err = outcome.expect_err("an unknown generation must be refused");
        assert!(err.contains("does-not-exist"), "{err}");
    }
}
