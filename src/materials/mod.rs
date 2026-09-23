//! The material truth: one table, read by the icon pipeline and by the game.
//!
//! Before C8 there were **two** answers to "what colour is a panel": `src/hud.rs`
//! held the interface's tokens and `tools/icons/palette.py` transcribed them, so the
//! icon checks measured contrast against a copy, and the world had no material
//! vocabulary at all — a road was a colour, not a material.
//!
//! Now the declaration lives in `tools/materials/declare.py`, the generator emits
//! [`generated`], and both sides read it. Three properties are load-bearing:
//!
//! * **The generated source is the artifact.** The game compiles the table in, so
//!   there is no runtime parse and no "file is missing" case; what remains possible
//!   is a *stale* generated file, and [`verify`] and the `digest` test both catch it.
//! * **Failing closed is the house rule.** `governor.json` refuses to start when it
//!   does not parse, `design::verify` refuses to start on a design defect, and this
//!   gate reports **every** defect it found rather than the first.
//! * **A mechanism is a claim with a number.** A family's colour arrives by a named
//!   physical mechanism, and asking for more chroma than that mechanism carries is
//!   asking for a different material — checked, not asserted.
//!
//! What it cannot check is printed as open, every run, in the same spirit as
//! `design::verify`: whether the world *reads* as these materials to a person,
//! frame timing with the new shade term, whether the sim effects are balanced, and
//! composite contrast on a rendered frame.

pub mod effects;
pub mod generated;
pub mod geology;
pub mod world;

pub use generated::*;

/// One resolved entry, by its declared name.
pub fn material(name: &str) -> Option<&'static Material> {
    MATERIALS.iter().find(|entry| entry.name == name)
}

/// The entry for a family, anchor and level, whichever vocabulary the caller has.
pub fn material_of(family: &str, hue: &str, level: &str) -> Option<&'static Material> {
    MATERIALS
        .iter()
        .find(|entry| entry.family == family && entry.hue == hue && entry.level == level)
}

/// A token's colour, as the frame takes it. `None` means the token is not owned by
/// this table, which is a question a caller is allowed to ask.
pub fn token(name: &str) -> Option<[f32; 4]> {
    TOKEN_COLOURS
        .iter()
        .find(|(token, _)| *token == name)
        .map(|(_, colour)| *colour)
}

/// The same lookup where a missing token is a **defect** rather than a question.
pub fn token_or_defect(name: &str) -> [f32; 4] {
    token(name).unwrap_or_else(|| {
        panic!("`{name}` is not in the material table; the table is the one home for it")
    })
}

pub fn family(name: &str) -> Option<&'static Family> {
    ICON_FAMILIES
        .iter()
        .chain(WORLD_FAMILIES.iter())
        .find(|family| family.name == name)
}

/// The declared mechanism behind a family's colour, and the chroma ceiling it sets.
pub fn mechanism(name: &str) -> Option<(&'static str, f32)> {
    MECHANISMS
        .iter()
        .find(|(family, _, _)| *family == name)
        .map(|(_, mechanism, ceiling)| (*mechanism, *ceiling))
}

/// Whether a chroma is still the family's own colour mechanism.
pub fn within_ceiling(name: &str, chroma: f32) -> bool {
    mechanism(name).is_some_and(|(_, ceiling)| chroma <= ceiling + 1e-9)
}

/// The material a surface's token **is**: `Ground` is the grass's own reading, so a
/// token cannot quietly drift away from the material it claims to be.
pub fn surface_of_token(name: &str) -> Option<&'static str> {
    SURFACE_TOKENS
        .iter()
        .find(|(token, _)| *token == name)
        .map(|(_, surface)| *surface)
}

/// A lookup, a ceiling and a token all in one, for a caller that has the name of a
/// world part rather than of a material.
pub fn material_of_part(part: &str) -> Option<&'static Material> {
    let part = world::part(part)?;
    material_of(part.family.as_str(), part.hue.as_str(), part.level.as_str())
}

/// The colour the frame takes for a material, at a given alpha.
///
/// The renderer asks by the vocabulary a claim is recorded in (`family`, `anchor`,
/// `level`) so a structure's colour comes from its own `MAT-*` claim rather than from a
/// token chosen at the call site. `None` means the claim names a material the table does
/// not resolve, which a caller has to handle rather than default — a structure drawn in a
/// colour nobody declared is exactly the silent drift the one-home rule removes.
pub fn colour_of(family: &str, anchor: &str, level: &str, alpha: f32) -> Option<[f32; 4]> {
    material_of(family, anchor, level).map(|entry| {
        [entry.rgb[0], entry.rgb[1], entry.rgb[2], alpha]
    })
}

/// The same, by the declared name of a world part (`scaffold.frame`, `home.roof`).
pub fn part_colour(part: &str, alpha: f32) -> Option<[f32; 4]> {
    material_of_part(part).map(|entry| [entry.rgb[0], entry.rgb[1], entry.rgb[2], alpha])
}

/// What a gate run found.
#[derive(Clone, Debug, Default)]
pub struct Report {
    /// Every line the gate prints, defects included.
    pub lines: Vec<String>,
    /// The subset that is a defect: a note that refuses promotion, here or in the
    /// world. Empty means the requirement is present in the source — and nothing
    /// more than that.
    pub defects: Vec<String>,
}

impl Report {
    pub fn ok(&self) -> bool {
        self.defects.is_empty()
    }
}

/// The material gate. Run at startup and in tests; it reports **every** defect, so
/// a run is one reading rather than a queue.
pub fn verify() -> Report {
    let mut report = Report::default();
    let defect = |report: &mut Report, line: String| {
        report.lines.push(format!("DEFECT: {line}"));
        report.defects.push(line);
    };

    // Families: exactly seven an icon may use, and a world that may name more.
    let expected = ICON_FAMILIES.len() * world_anchors() * LEVELS.len();
    let icon_entries = MATERIALS
        .iter()
        .filter(|entry| is_icon_family(entry.family))
        .count();
    report.lines.push(format!(
        "materials: {} icon families (+{} world-only), {} anchors, {} levels, {} entries",
        ICON_FAMILIES.len(),
        WORLD_FAMILIES.len(),
        world_anchors(),
        LEVELS.len(),
        MATERIALS.len()
    ));
    if icon_entries != expected {
        defect(
            &mut report,
            format!("the icon matrix resolves {icon_entries} entries, not the declared {expected}"),
        );
    }

    // Every family has a mechanism, and the ceiling matches the chroma in use.
    for family in ICON_FAMILIES.iter().chain(WORLD_FAMILIES.iter()) {
        match mechanism(family.name) {
            None => defect(
                &mut report,
                format!("`{}` has no declared colour mechanism", family.name),
            ),
            Some((mechanism, ceiling)) => {
                if (ceiling - family.variant_chroma).abs() > 1e-9 {
                    defect(
                        &mut report,
                        format!(
                            "`{}` claims a {ceiling} ceiling but varies chroma to {} — the \
                             mechanism and the number disagree",
                            family.name, family.variant_chroma
                        ),
                    );
                }
                if mechanism.trim().is_empty() {
                    defect(&mut report, format!("`{}`'s mechanism is unnamed", family.name));
                }
            }
        }
    }

    // Every entry is inside its family's ceiling and inside the declared bounds.
    let (low, high) = LIGHTNESS_BOUNDS;
    for entry in MATERIALS {
        if !within_ceiling(entry.family, entry.chroma) {
            defect(
                &mut report,
                format!(
                    "`{}` asks for chroma {} on `{}`, past that mechanism's ceiling — that is a \
                     different material, not a stronger colour",
                    entry.name, entry.chroma, entry.family
                ),
            );
        }
        if entry.lightness < low - 1e-9 || entry.lightness > high + 1e-9 {
            defect(
                &mut report,
                format!(
                    "`{}` sits at lightness {:.4}, outside the declared bounds ({low}, {high})",
                    entry.name, entry.lightness
                ),
            );
        }
    }

    // Every world part resolves, unless it is one of the two declared rules.
    for part in world::PARTS {
        if family(part.family.as_str()).is_none() {
            defect(
                &mut report,
                format!("`{}` names a family the table does not declare", part.part),
            );
        }
        if material_of(part.family.as_str(), part.hue.as_str(), part.level.as_str()).is_none() {
            defect(
                &mut report,
                format!("`{}` does not resolve to a declared entry", part.part),
            );
        }
    }
    for (rule, why) in world::RULES {
        report.lines.push(format!("materials: {rule} is a rule, not a row — {why}"));
    }

    // The mapping exists twice, so the two copies are compared here as well as in
    // `world`'s own test: a difference is a build failure, not a drift.
    if world::PARTS.len() + world::RULES.len() != WORLD_SURFACES.len() {
        defect(
            &mut report,
            "the Rust mapping and the emitted copy describe different numbers of surfaces"
                .to_string(),
        );
    }

    // A surface token must be its material's reading, to the last float.
    for (name, surface) in SURFACE_TOKENS {
        let Some(_part) = world::part(surface) else {
            defect(
                &mut report,
                format!("`{name}` claims to be `{surface}`, which is not a declared row"),
            );
            continue;
        };
        let Some(entry) = material_of_part(surface) else {
            defect(&mut report, format!("`{surface}` does not resolve"));
            continue;
        };
        match token(name) {
            None => defect(&mut report, format!("`{name}` has no colour in the table")),
            Some(colour) => {
                for (channel, (token_channel, material_channel)) in
                    colour.iter().zip(entry.rgb.iter()).enumerate()
                {
                    if (token_channel - material_channel).abs() > 1e-6 {
                        defect(
                            &mut report,
                            format!(
                                "`{name}` is not its material's reading: token {token_channel:.6}, \
                                 `{}` {material_channel:.6} on channel {channel}",
                                entry.name
                            ),
                        );
                    }
                }
            }
        }
    }

    // The declared divergence is reported, not hidden: it is a supersession, and a
    // reader who wants the value it replaced finds it named.
    for (token, was, now) in DIVERGENCES {
        report
            .lines
            .push(format!("materials: {token} moved — {was}; {now}"));
    }

    for item in OPEN {
        report.lines.push(format!("materials: open — {item}"));
    }
    report
}

fn world_anchors() -> usize {
    ANCHORS.len() + 1
}

/// Whether a family name belongs to the icon set's seven. The icon set is a
/// property of the *set*, so this is asked of the emitted list rather than inferred
/// from the world's families being "the rest".
fn is_icon_family(name: &str) -> bool {
    ICON_FAMILIES.iter().any(|family| family.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The generator's own digest, recomputed here from the files it read. FNV-1a is
    /// mirrored in two languages on purpose: this has to be runnable by `cargo test`,
    /// which the project keeps free of Python and Blender.
    #[test]
    fn the_generated_table_is_not_stale() {
        fn fnv1a64(digest: u64, data: &[u8]) -> u64 {
            let mut digest = digest;
            for byte in data {
                digest ^= *byte as u64;
                digest = digest.wrapping_mul(0x0000_0100_0000_01B3);
            }
            digest
        }
        let mut digest = 0xCBF2_9CE4_8422_2325;
        for path in [
            "tools/materials/declare.py",
            "tools/icons/palette.py",
        ] {
            let bytes = std::fs::read(path)
                .unwrap_or_else(|error| panic!("the declaration `{path}` is unreadable: {error}"));
            digest = fnv1a64(digest, &bytes);
        }
        assert_eq!(
            digest, SOURCE_DIGEST,
            "src/materials/generated.rs is stale: re-run `python tools/materials/emit.py`"
        );
        assert_eq!(GENERATED_BY, "tools/materials/emit.py");
    }

    #[test]
    fn every_defect_reported_is_a_real_one() {
        let report = verify();
        assert!(report.ok(), "the material gate found defects: {:#?}", report.defects);
    }

    #[test]
    fn the_gate_reports_every_open_item_it_has() {
        let report = verify();
        for item in OPEN {
            assert!(
                report.lines.iter().any(|line| line.contains(item)),
                "the open list is part of the report: `{item}` was not printed"
            );
        }
    }

    #[test]
    fn a_ceiling_refuses_a_chroma_the_mechanism_cannot_carry() {
        // A metal's colour arrives by an anodised film, which carries no more than
        // 0.060. Asking for enamel's 0.190 is asking for a different material.
        assert!(within_ceiling("metal", 0.060));
        assert!(!within_ceiling("metal", 0.190));
        assert!(within_ceiling("enamel", 0.190));
        // The world families are bounded by the same rule as the icon ones.
        assert!(within_ceiling("water", 0.100));
        assert!(!within_ceiling("water", 0.200));
    }

    #[test]
    fn an_undeclared_family_is_refused_rather_than_defaulted() {
        assert!(family("plastic").is_none());
        assert!(material("metal:chartreuse").is_none());
        assert!(mechanism("plastic").is_none());
        assert!(
            !within_ceiling("plastic", 0.0),
            "an unknown family must fail closed, not pass as declared"
        );
    }

    #[test]
    fn a_structure_resolves_to_the_parts_it_is_made_of() {
        let home = world::parts_of("home");
        assert_eq!(home.len(), 3, "a home is three parts: {home:#?}");
        let materials: Vec<&str> = home
            .iter()
            .map(|part| material_of_part(part.part).expect("resolves").name)
            .collect();
        assert_eq!(materials, vec!["ceramic:natural", "polymer:amber:edge", "glass:cyan"]);
    }

    #[test]
    fn the_price_of_a_home_is_the_sum_of_its_parts() {
        let total: f32 = world::parts_of("home")
            .iter()
            .map(|part| {
                PART_PRICE
                    .iter()
                    .find(|(name, _)| *name == part.part)
                    .map(|(_, price)| *price)
                    .unwrap_or_else(|| panic!("`{}` has no declared price", part.part))
            })
            .sum();
        assert_eq!(total, 140.0, "the re-tuned home price, per a173");
    }

    #[test]
    fn the_same_family_is_never_both_icon_and_world() {
        for icon in ICON_FAMILIES {
            for world in WORLD_FAMILIES {
                assert_ne!(icon.name, world.name, "`{}` is declared twice", icon.name);
            }
            assert!(is_icon_family(icon.name));
            assert!(family(icon.name).is_some());
        }
    }
}
