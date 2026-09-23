//! The world's surfaces, declared in Rust and checked against the emitted copy.
//!
//! C8 a157 settled this shape deliberately: the mapping is **declared here**,
//! typed, so the compiler knows a family from a hue and the sim can match on it —
//! and the generator emits the same rows as strings, which `verify` compares row
//! for row in both directions. A mapping that exists twice without a check is the
//! defect the one-home rule exists to remove; a mapping that exists twice *with* a
//! check is a typed view and a serializable view of one fact.
//!
//! The two special forms are rules rather than materials, and are named as such:
//! zone paint takes the anchor of the zone that painted it, and a ruin is the
//! retired structure's own parts read at `deep`.

/// A material family the world may use. The icon set's seven, plus the world's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    Metal,
    Paper,
    Ceramic,
    Glass,
    Polymer,
    Road,
    Enamel,
    // World-only: declared so a lake is not a pane and grass is not a sheet.
    Water,
    Organic,
    Soil,
}

impl Family {
    pub fn as_str(self) -> &'static str {
        match self {
            Family::Metal => "metal",
            Family::Paper => "paper",
            Family::Ceramic => "ceramic",
            Family::Glass => "glass",
            Family::Polymer => "polymer",
            Family::Road => "road",
            Family::Enamel => "enamel",
            Family::Water => "water",
            Family::Organic => "organic",
            Family::Soil => "soil",
        }
    }

    /// Whether an icon may be made of this family. Exactly the seven.
    pub fn is_icon(self) -> bool {
        !matches!(self, Family::Water | Family::Organic | Family::Soil)
    }
}

/// A declared hue anchor, or the family's own untinted hue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hue {
    Natural,
    Red,
    Amber,
    Yellow,
    Green,
    Cyan,
    Blue,
    Violet,
}

impl Hue {
    pub fn as_str(self) -> &'static str {
        match self {
            Hue::Natural => "natural",
            Hue::Red => "red",
            Hue::Amber => "amber",
            Hue::Yellow => "yellow",
            Hue::Green => "green",
            Hue::Cyan => "cyan",
            Hue::Blue => "blue",
            Hue::Violet => "violet",
        }
    }
}

/// A declared lightness offset on the family's own lightness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Body,
    Edge,
    Deep,
}

impl Level {
    pub fn as_str(self) -> &'static str {
        match self {
            Level::Body => "body",
            Level::Edge => "edge",
            Level::Deep => "deep",
        }
    }
}

/// One part of one world surface, as a claim: this part is this material.
#[derive(Clone, Copy, Debug)]
pub struct Part {
    /// The part's name, which is also the row the generated copy carries.
    pub part: &'static str,
    pub family: Family,
    pub hue: Hue,
    pub level: Level,
    /// Why this part is this material, so the claim can be argued with.
    pub note: &'static str,
}

/// Every part of the world. Grass and water included: they are materials too, and
/// saying so is what stops "the ground" being an unexamined colour.
pub const PARTS: &[Part] = &[
    Part { part: "home.walls", family: Family::Ceramic, hue: Hue::Natural, level: Level::Body, note: "the home's body" },
    Part { part: "home.roof", family: Family::Polymer, hue: Hue::Amber, level: Level::Edge, note: "a different material from the walls, read once" },
    Part { part: "home.window", family: Family::Glass, hue: Hue::Cyan, level: Level::Body, note: "the one transparent part" },
    Part { part: "shop.walls", family: Family::Ceramic, hue: Hue::Natural, level: Level::Body, note: "shares the residential body" },
    Part { part: "shop.frontage", family: Family::Enamel, hue: Hue::Blue, level: Level::Body, note: "the commercial face paint belongs to" },
    Part { part: "shop.window", family: Family::Glass, hue: Hue::Cyan, level: Level::Body, note: "" },
    Part { part: "factory.frame", family: Family::Metal, hue: Hue::Natural, level: Level::Body, note: "structural metal" },
    Part { part: "factory.cladding", family: Family::Enamel, hue: Hue::Amber, level: Level::Body, note: "coated industrial panel" },
    Part { part: "factory.vent", family: Family::Polymer, hue: Hue::Natural, level: Level::Deep, note: "the dark recessed part" },
    Part { part: "power.frame", family: Family::Metal, hue: Hue::Natural, level: Level::Body, note: "" },
    Part { part: "power.stack", family: Family::Ceramic, hue: Hue::Natural, level: Level::Edge, note: "a fired stack, not a painted one" },
    Part { part: "power.insulator", family: Family::Ceramic, hue: Hue::Amber, level: Level::Body, note: "the insulator is the part that must not conduct" },
    Part { part: "power.core", family: Family::Enamel, hue: Hue::Amber, level: Level::Body, note: "painted caution face" },
    Part { part: "road.surface", family: Family::Road, hue: Hue::Natural, level: Level::Body, note: "aggregate" },
    Part { part: "road.edge", family: Family::Soil, hue: Hue::Natural, level: Level::Body, note: "the shoulder: mineral, not asphalt" },
    Part { part: "terrain.ground", family: Family::Organic, hue: Hue::Natural, level: Level::Body, note: "the grass is a living surface, not a mineral one" },
    Part { part: "terrain.water", family: Family::Water, hue: Hue::Natural, level: Level::Body, note: "a body of water is a volume, not a pane" },
    Part { part: "powerline.conductor", family: Family::Metal, hue: Hue::Natural, level: Level::Body, note: "what a conductor must be made of" },
    Part { part: "powerline.pylon", family: Family::Metal, hue: Hue::Natural, level: Level::Deep, note: "the same metal read darker" },
    Part { part: "scaffold.frame", family: Family::Metal, hue: Hue::Amber, level: Level::Edge, note: "under construction, and painted to say so" },
    Part { part: "scaffold.deck", family: Family::Paper, hue: Hue::Natural, level: Level::Body, note: "a board, not a beam" },
];

/// Surfaces whose material is a **rule**, not a row: named so the absence of a row
/// is a declaration rather than an oversight.
pub const RULES: &[(&str, &str)] = &[
    (
        "zone.paint",
        "a tint over whatever it paints on; the anchor is the zone's own",
    ),
    (
        "ruin.parts",
        "the retired structure's own parts, read at `deep`",
    ),
];

/// The row for a part, if it has one.
pub fn part(name: &str) -> Option<&'static Part> {
    PARTS.iter().find(|part| part.part == name)
}

/// The part of a structure of a given kind, in the order the frame draws them.
pub fn parts_of(kind: &str) -> Vec<&'static Part> {
    let prefix = format!("{kind}.");
    PARTS
        .iter()
        .filter(|part| part.part.starts_with(&prefix))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materials::generated;

    /// Both directions, so an extra row on either side is a failure: a mapping that
    /// exists twice is only safe with the check that makes drift impossible.
    #[test]
    fn the_declared_mapping_and_the_emitted_copy_agree() {
        assert_eq!(
            PARTS.len() + RULES.len(),
            generated::WORLD_SURFACES.len(),
            "the declared mapping and the emitted copy describe different worlds"
        );
        for part in PARTS {
            let row = generated::WORLD_SURFACES
                .iter()
                .find(|(name, ..)| *name == part.part)
                .unwrap_or_else(|| panic!("`{}` is declared in Rust and missing from the emitted copy", part.part));
            assert_eq!(row.1, part.family.as_str(), "{} family", part.part);
            assert_eq!(row.2, part.hue.as_str(), "{} hue", part.part);
            assert_eq!(row.3, part.level.as_str(), "{} level", part.part);
        }
        for (name, ..) in generated::WORLD_SURFACES {
            assert!(
                part(name).is_some() || RULES.iter().any(|(rule, _)| rule == name),
                "`{name}` is in the emitted copy and not declared in Rust"
            );
        }
    }

    #[test]
    fn an_icon_may_only_be_made_of_the_seven() {
        let icon: Vec<&str> = generated::ICON_FAMILIES.iter().map(|f| f.name).collect();
        assert_eq!(icon.len(), 7, "the icon set is seven families, not {}", icon.len());
        for family in [Family::Metal, Family::Paper, Family::Ceramic, Family::Glass, Family::Polymer, Family::Road, Family::Enamel] {
            assert!(family.is_icon(), "{} must be available to an icon", family.as_str());
            assert!(icon.contains(&family.as_str()), "{} is missing from the icon set", family.as_str());
        }
        for family in [Family::Water, Family::Organic, Family::Soil] {
            assert!(!family.is_icon(), "{} is a world material, not an icon material", family.as_str());
            assert!(!icon.contains(&family.as_str()), "{} leaked into the icon set", family.as_str());
        }
    }

    #[test]
    fn every_declared_world_part_resolves_to_a_declared_material() {
        let report = crate::materials::verify();
        assert!(
            report.ok(),
            "the gate found defects in the declared world: {:#?}",
            report.defects
        );
    }
}
