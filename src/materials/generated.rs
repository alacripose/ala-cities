//! GENERATED FILE — derive it with `python tools/materials/emit.py`; never hand-edit it.
//!
//! One home for the material truth: the icon pipeline, this crate's renderer and
//! the world's sim all read these values, and `tests` in `mod.rs` refuses a build
//! whose generated source no longer matches the declaration it came from.

pub const SOURCE_DIGEST: u64 = 0x4218A3E2DCE1FCC2;
pub const GENERATED_BY: &str = "tools/materials/emit.py";

/// One declared material family: its own lightness, the chroma a hue variation
/// may carry, and the chroma and hue its untinted body carries.
#[derive(Clone, Copy, Debug)]
pub struct Family {
    pub name: &'static str,
    pub lightness: f32,
    pub variant_chroma: f32,
    pub natural_chroma: f32,
    pub natural_hue: f32,
    pub note: &'static str,
}

/// The icon set's families: exactly seven. A family the world needs does not
/// appear here, so an icon cannot be made of one.
pub const ICON_FAMILIES: &[Family] = &[
    Family { name: "metal", lightness: 0.78, variant_chroma: 0.06, natural_chroma: 0.018, natural_hue: 250.0, note: "the light cool object the study's chrome reads as" },
    Family { name: "paper", lightness: 0.88, variant_chroma: 0.06, natural_chroma: 0.035, natural_hue: 82.0, note: "matte warm sheet" },
    Family { name: "ceramic", lightness: 0.9, variant_chroma: 0.05, natural_chroma: 0.025, natural_hue: 72.0, note: "glazed insulator" },
    Family { name: "glass", lightness: 0.66, variant_chroma: 0.1, natural_chroma: 0.09, natural_hue: 220.0, note: "lens and pane" },
    Family { name: "polymer", lightness: 0.34, variant_chroma: 0.05, natural_chroma: 0.018, natural_hue: 250.0, note: "dark grip" },
    Family { name: "road", lightness: 0.38, variant_chroma: 0.04, natural_chroma: 0.018, natural_hue: 250.0, note: "matte surface" },
    Family { name: "enamel", lightness: 0.62, variant_chroma: 0.19, natural_chroma: 0.19, natural_hue: 245.0, note: "the study's blue control face; coated colour" },
];

/// The world's own families, resolvable and never available to an icon.
pub const WORLD_FAMILIES: &[Family] = &[
    Family { name: "water", lightness: 0.42, variant_chroma: 0.1, natural_chroma: 0.07, natural_hue: 240.0, note: "body volume: colour held in the water itself, with depth absorption" },
    Family { name: "organic", lightness: 0.3, variant_chroma: 0.12, natural_chroma: 0.03, natural_hue: 140.0, note: "pigment in living tissue: never uniform, always slightly varied" },
    Family { name: "soil", lightness: 0.52, variant_chroma: 0.06, natural_chroma: 0.005, natural_hue: 260.0, note: "aggregate and mineral: near-neutral, matte, granular" },
];

/// How colour physically arrives, per family, and the most chroma that mechanism
/// can carry. Asking for more is asking for a different material.
pub const MECHANISMS: &[(&str, &str, f32)] = &[
    ("metal", "anodised film: an oxide layer on the substrate, so a metal body stays a metal", 0.06),
    ("glass", "body-tinted: colour held in the glass itself, which also tints what shows through it", 0.1),
    ("ceramic", "fired glaze: a mineral glaze over the body, opaque rather than transparent", 0.05),
    ("polymer", "pigmented resin: colour compounded into the plastic itself", 0.05),
    ("paper", "dyed stock: pigment in the sheet, which is why it stays matte and pale", 0.06),
    ("road", "aggregate: asphalt and stone, near-neutral because that is what it is made of", 0.04),
    ("enamel", "painted colour: a pigmented coating over a substrate -- the family paint belongs to", 0.19),
    ("water", "body volume: light travels through the material, so its colour is a property of depth", 0.1),
    ("organic", "pigment in living tissue: grown rather than applied, and never uniform across a body", 0.12),
    ("soil", "aggregate and mineral grain: a mixture, so it reads near-neutral and matte", 0.06),
];

/// Declared lightness offsets on a family's own lightness — one material read
/// twice, rather than two materials.
pub const LEVELS: &[(&str, f32)] = &[
    ("body", 0.0),
    ("edge", 0.1),
    ("deep", -0.18),
];
pub const LIGHTNESS_BOUNDS: (f32, f32) = (0.05, 0.97);

/// The hue ring, and what backs each anchor. A slot says so rather than
/// borrowing a measurement's authority.
pub const ANCHORS: &[(&str, f32, &str)] = &[
    ("red", 25.0, "backed by Token::Refused (25) — destructive"),
    ("amber", 62.0, "backed by ZoneIndustrial (60) / Warning (70) / Scaffold (80) — caution, power"),
    ("yellow", 115.0, "declared slot — no reference and no game meaning occupies it"),
    ("green", 150.0, "backed by Nature / ZoneResidential (150) — nature, residential"),
    ("cyan", 200.0, "partially backed — the study's people (162.6) and store (201.8) classes; no game token"),
    ("blue", 250.0, "backed by Ink / ZoneCommercial (250) — data, commercial"),
    ("violet", 300.0, "declared slot — no reference and no game meaning occupies it"),
];
pub const NATURAL: &str = "natural";

/// One resolved entry: family x anchor x level, in both vocabularies — OKLCH,
/// which is what a designer reads, and linear sRGB, which is what the frame takes.
#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub name: &'static str,
    pub family: &'static str,
    pub hue: &'static str,
    pub level: &'static str,
    pub lightness: f32,
    pub chroma: f32,
    pub hue_degrees: f32,
    pub rgb: [f32; 3],
}

pub const MATERIALS: &[Material] = &[
    Material { name: "metal:red", family: "metal", hue: "red", level: "body", lightness: 0.78, chroma: 0.06, hue_degrees: 25.0, rgb: [0.7115925, 0.39738902, 0.37202606] },
    Material { name: "metal:red:edge", family: "metal", hue: "red", level: "edge", lightness: 0.88, chroma: 0.06, hue_degrees: 25.0, rgb: [0.98211163, 0.58368665, 0.55019313] },
    Material { name: "metal:red:deep", family: "metal", hue: "red", level: "deep", lightness: 0.6, chroma: 0.06, hue_degrees: 25.0, rgb: [0.35758707, 0.16981152, 0.1562832] },
    Material { name: "metal:amber", family: "metal", hue: "amber", level: "body", lightness: 0.78, chroma: 0.06, hue_degrees: 62.0, rgb: [0.6584038, 0.43097198, 0.2808675] },
    Material { name: "metal:amber:edge", family: "metal", hue: "amber", level: "edge", lightness: 0.88, chroma: 0.06, hue_degrees: 62.0, rgb: [0.9144439, 0.6267371, 0.4322094] },
    Material { name: "metal:amber:deep", family: "metal", hue: "amber", level: "deep", lightness: 0.6, chroma: 0.06, hue_degrees: 62.0, rgb: [0.32606593, 0.18931778, 0.10470182] },
    Material { name: "metal:yellow", family: "metal", hue: "yellow", level: "body", lightness: 0.78, chroma: 0.06, hue_degrees: 115.0, rgb: [0.46568993, 0.50561243, 0.281665] },
    Material { name: "metal:yellow:edge", family: "metal", hue: "yellow", level: "edge", lightness: 0.88, chroma: 0.06, hue_degrees: 115.0, rgb: [0.6698474, 0.72151214, 0.43343344] },
    Material { name: "metal:yellow:deep", family: "metal", hue: "yellow", level: "deep", lightness: 0.6, chroma: 0.06, hue_degrees: 115.0, rgb: [0.21117362, 0.23376705, 0.104922675] },
    Material { name: "metal:green", family: "metal", hue: "green", level: "body", lightness: 0.78, chroma: 0.06, hue_degrees: 150.0, rgb: [0.33597314, 0.54406023, 0.36836702] },
    Material { name: "metal:green:edge", family: "metal", hue: "green", level: "edge", lightness: 0.88, chroma: 0.06, hue_degrees: 150.0, rgb: [0.5047363, 0.77014923, 0.5456647] },
    Material { name: "metal:green:deep", family: "metal", hue: "green", level: "deep", lightness: 0.6, chroma: 0.06, hue_degrees: 150.0, rgb: [0.13442385, 0.2568801, 0.15396148] },
    Material { name: "metal:cyan", family: "metal", hue: "cyan", level: "body", lightness: 0.78, chroma: 0.06, hue_degrees: 200.0, rgb: [0.251678, 0.5488914, 0.56769717] },
    Material { name: "metal:cyan:edge", family: "metal", hue: "cyan", level: "edge", lightness: 0.88, chroma: 0.06, hue_degrees: 200.0, rgb: [0.39679933, 0.77646685, 0.7994553] },
    Material { name: "metal:cyan:deep", family: "metal", hue: "cyan", level: "deep", lightness: 0.6, chroma: 0.06, hue_degrees: 200.0, rgb: [0.0853294, 0.25953215, 0.2718266] },
    Material { name: "metal:blue", family: "metal", hue: "blue", level: "body", lightness: 0.78, chroma: 0.06, hue_degrees: 250.0, rgb: [0.32690224, 0.49690995, 0.7228958] },
    Material { name: "metal:blue:edge", family: "metal", hue: "blue", level: "edge", lightness: 0.88, chroma: 0.06, hue_degrees: 250.0, rgb: [0.49255264, 0.7107573, 0.99416935] },
    Material { name: "metal:blue:deep", family: "metal", hue: "blue", level: "deep", lightness: 0.6, chroma: 0.06, hue_degrees: 250.0, rgb: [0.1298452, 0.22820489, 0.36718157] },
    Material { name: "metal:violet", family: "metal", hue: "violet", level: "body", lightness: 0.78, chroma: 0.06, hue_degrees: 300.0, rgb: [0.5081421, 0.430164, 0.692183] },
    Material { name: "metal:violet:edge", family: "metal", hue: "violet", level: "edge", lightness: 0.88, chroma: 0.06, hue_degrees: 300.0, rgb: [0.7238852, 0.62547743, 0.9559596] },
    Material { name: "metal:violet:deep", family: "metal", hue: "violet", level: "deep", lightness: 0.6, chroma: 0.06, hue_degrees: 300.0, rgb: [0.23629996, 0.189109, 0.34790605] },
    Material { name: "metal:natural", family: "metal", hue: "natural", level: "body", lightness: 0.78, chroma: 0.018, hue_degrees: 250.0, rgb: [0.42883626, 0.48244408, 0.5441972] },
    Material { name: "metal:natural:edge", family: "metal", hue: "natural", level: "edge", lightness: 0.88, chroma: 0.018, hue_degrees: 250.0, rgb: [0.62319535, 0.6915896, 0.7698253] },
    Material { name: "metal:natural:deep", family: "metal", hue: "natural", level: "deep", lightness: 0.6, chroma: 0.018, hue_degrees: 250.0, rgb: [0.18905675, 0.22058101, 0.25757283] },
    Material { name: "paper:red", family: "paper", hue: "red", level: "body", lightness: 0.88, chroma: 0.06, hue_degrees: 25.0, rgb: [0.98211163, 0.58368665, 0.55019313] },
    Material { name: "paper:red:edge", family: "paper", hue: "red", level: "edge", lightness: 0.97, chroma: 0.0146, hue_degrees: 25.0, rgb: [0.9996373, 0.884536, 0.87244135] },
    Material { name: "paper:red:deep", family: "paper", hue: "red", level: "deep", lightness: 0.7, chroma: 0.06, hue_degrees: 25.0, rgb: [0.5345969, 0.28057897, 0.26092035] },
    Material { name: "paper:amber", family: "paper", hue: "amber", level: "body", lightness: 0.88, chroma: 0.06, hue_degrees: 62.0, rgb: [0.9144439, 0.6267371, 0.4322094] },
    Material { name: "paper:amber:edge", family: "paper", hue: "amber", level: "edge", lightness: 0.97, chroma: 0.0189, hue_degrees: 62.0, rgb: [0.9996051, 0.89331794, 0.8113084] },
    Material { name: "paper:amber:deep", family: "paper", hue: "amber", level: "deep", lightness: 0.7, chroma: 0.06, hue_degrees: 62.0, rgb: [0.49173585, 0.307435, 0.1887339] },
    Material { name: "paper:yellow", family: "paper", hue: "yellow", level: "body", lightness: 0.88, chroma: 0.06, hue_degrees: 115.0, rgb: [0.6698474, 0.72151214, 0.43343344] },
    Material { name: "paper:yellow:edge", family: "paper", hue: "yellow", level: "edge", lightness: 0.97, chroma: 0.06, hue_degrees: 115.0, rgb: [0.8982444, 0.9617687, 0.60907304] },
    Material { name: "paper:yellow:deep", family: "paper", hue: "yellow", level: "deep", lightness: 0.7, chroma: 0.06, hue_degrees: 115.0, rgb: [0.33608025, 0.36769667, 0.18924478] },
    Material { name: "paper:green", family: "paper", hue: "green", level: "body", lightness: 0.88, chroma: 0.06, hue_degrees: 150.0, rgb: [0.5047363, 0.77014923, 0.5456647] },
    Material { name: "paper:green:edge", family: "paper", hue: "green", level: "edge", lightness: 0.97, chroma: 0.0483, hue_degrees: 150.0, rgb: [0.73909736, 0.99982774, 0.7784981] },
    Material { name: "paper:green:deep", family: "paper", hue: "green", level: "deep", lightness: 0.7, chroma: 0.06, hue_degrees: 150.0, rgb: [0.23160958, 0.3988519, 0.2578918] },
    Material { name: "paper:cyan", family: "paper", hue: "cyan", level: "body", lightness: 0.88, chroma: 0.06, hue_degrees: 200.0, rgb: [0.39679933, 0.77646685, 0.7994553] },
    Material { name: "paper:cyan:edge", family: "paper", hue: "cyan", level: "edge", lightness: 0.97, chroma: 0.0371, hue_degrees: 200.0, rgb: [0.69624865, 0.98500085, 0.99983877] },
    Material { name: "paper:cyan:deep", family: "paper", hue: "cyan", level: "deep", lightness: 0.7, chroma: 0.06, hue_degrees: 200.0, rgb: [0.16412659, 0.40263578, 0.4183861] },
    Material { name: "paper:blue", family: "paper", hue: "blue", level: "body", lightness: 0.88, chroma: 0.06, hue_degrees: 250.0, rgb: [0.49255264, 0.7107573, 0.99416935] },
    Material { name: "paper:blue:edge", family: "paper", hue: "blue", level: "edge", lightness: 0.97, chroma: 0.0147, hue_degrees: 250.0, rgb: [0.8546725, 0.9228566, 0.99975353] },
    Material { name: "paper:blue:deep", family: "paper", hue: "blue", level: "deep", lightness: 0.7, chroma: 0.06, hue_degrees: 250.0, rgb: [0.22471154, 0.36047783, 0.5451964] },
    Material { name: "paper:violet", family: "paper", hue: "violet", level: "body", lightness: 0.88, chroma: 0.06, hue_degrees: 300.0, rgb: [0.7238852, 0.62547743, 0.9559596] },
    Material { name: "paper:violet:edge", family: "paper", hue: "violet", level: "edge", lightness: 0.97, chroma: 0.0165, hue_degrees: 300.0, rgb: [0.92618924, 0.8949305, 0.99955165] },
    Material { name: "paper:violet:deep", family: "paper", hue: "violet", level: "deep", lightness: 0.7, chroma: 0.06, hue_degrees: 300.0, rgb: [0.37027207, 0.30692732, 0.5198934] },
    Material { name: "paper:natural", family: "paper", hue: "natural", level: "body", lightness: 0.88, chroma: 0.035, hue_degrees: 82.0, rgb: [0.76941824, 0.67099005, 0.51556796] },
    Material { name: "paper:natural:edge", family: "paper", hue: "natural", level: "edge", lightness: 0.97, chroma: 0.0287, hue_degrees: 82.0, rgb: [0.99975777, 0.90274847, 0.74510634] },
    Material { name: "paper:natural:deep", family: "paper", hue: "natural", level: "deep", lightness: 0.7, chroma: 0.035, hue_degrees: 82.0, rgb: [0.39898843, 0.3360393, 0.23946267] },
    Material { name: "ceramic:red", family: "ceramic", hue: "red", level: "body", lightness: 0.9, chroma: 0.05, hue_degrees: 25.0, rgb: [0.9896983, 0.64430875, 0.6135863] },
    Material { name: "ceramic:red:edge", family: "ceramic", hue: "red", level: "edge", lightness: 0.97, chroma: 0.0146, hue_degrees: 25.0, rgb: [0.9996373, 0.884536, 0.87244135] },
    Material { name: "ceramic:red:deep", family: "ceramic", hue: "red", level: "deep", lightness: 0.72, chroma: 0.05, hue_degrees: 25.0, rgb: [0.54104686, 0.31866357, 0.30007678] },
    Material { name: "ceramic:amber", family: "ceramic", hue: "amber", level: "body", lightness: 0.9, chroma: 0.05, hue_degrees: 62.0, rgb: [0.93075293, 0.68222255, 0.5082617] },
    Material { name: "ceramic:amber:edge", family: "ceramic", hue: "amber", level: "edge", lightness: 0.97, chroma: 0.0189, hue_degrees: 62.0, rgb: [0.9996051, 0.89331794, 0.8113084] },
    Material { name: "ceramic:amber:deep", family: "ceramic", hue: "amber", level: "deep", lightness: 0.72, chroma: 0.05, hue_degrees: 62.0, rgb: [0.5032958, 0.34265468, 0.23441736] },
    Material { name: "ceramic:yellow", family: "ceramic", hue: "yellow", level: "body", lightness: 0.9, chroma: 0.05, hue_degrees: 115.0, rgb: [0.71843016, 0.76454186, 0.50959545] },
    Material { name: "ceramic:yellow:edge", family: "ceramic", hue: "yellow", level: "edge", lightness: 0.97, chroma: 0.05, hue_degrees: 115.0, rgb: [0.9002326, 0.9541967, 0.6566222] },
    Material { name: "ceramic:yellow:deep", family: "ceramic", hue: "yellow", level: "deep", lightness: 0.72, chroma: 0.05, hue_degrees: 115.0, rgb: [0.36679107, 0.39554363, 0.23508328] },
    Material { name: "ceramic:green", family: "ceramic", hue: "green", level: "body", lightness: 0.9, chroma: 0.05, hue_degrees: 150.0, rgb: [0.5745129, 0.80655247, 0.6098027] },
    Material { name: "ceramic:green:edge", family: "ceramic", hue: "green", level: "edge", lightness: 0.97, chroma: 0.0483, hue_degrees: 150.0, rgb: [0.73909736, 0.99982774, 0.7784981] },
    Material { name: "ceramic:green:deep", family: "ceramic", hue: "green", level: "deep", lightness: 0.72, chroma: 0.05, hue_degrees: 150.0, rgb: [0.27468368, 0.42270026, 0.2975402] },
    Material { name: "ceramic:cyan", family: "ceramic", hue: "cyan", level: "body", lightness: 0.9, chroma: 0.05, hue_degrees: 200.0, rgb: [0.47961932, 0.8122709, 0.83111644] },
    Material { name: "ceramic:cyan:edge", family: "ceramic", hue: "cyan", level: "edge", lightness: 0.97, chroma: 0.0371, hue_degrees: 200.0, rgb: [0.69624865, 0.98500085, 0.99983877] },
    Material { name: "ceramic:cyan:deep", family: "ceramic", hue: "cyan", level: "deep", lightness: 0.72, chroma: 0.05, hue_degrees: 200.0, rgb: [0.2145227, 0.426211, 0.4391115] },
    Material { name: "ceramic:blue", family: "ceramic", hue: "blue", level: "body", lightness: 0.9, chroma: 0.05, hue_degrees: 250.0, rgb: [0.56309426, 0.75556105, 0.99730766] },
    Material { name: "ceramic:blue:edge", family: "ceramic", hue: "blue", level: "edge", lightness: 0.97, chroma: 0.0147, hue_degrees: 250.0, rgb: [0.8546725, 0.9228566, 0.99975353] },
    Material { name: "ceramic:blue:deep", family: "ceramic", hue: "blue", level: "deep", lightness: 0.72, chroma: 0.05, hue_degrees: 250.0, rgb: [0.26793832, 0.38951835, 0.5479603] },
    Material { name: "ceramic:violet", family: "ceramic", hue: "violet", level: "body", lightness: 0.9, chroma: 0.05, hue_degrees: 300.0, rgb: [0.7655418, 0.6808233, 0.9651013] },
    Material { name: "ceramic:violet:edge", family: "ceramic", hue: "violet", level: "edge", lightness: 0.97, chroma: 0.0165, hue_degrees: 300.0, rgb: [0.92618924, 0.8949305, 0.99955165] },
    Material { name: "ceramic:violet:deep", family: "ceramic", hue: "violet", level: "deep", lightness: 0.72, chroma: 0.05, hue_degrees: 300.0, rgb: [0.39693573, 0.3419708, 0.5265741] },
    Material { name: "ceramic:natural", family: "ceramic", hue: "natural", level: "body", lightness: 0.9, chroma: 0.025, hue_degrees: 72.0, rgb: [0.8125986, 0.7138988, 0.60709995] },
    Material { name: "ceramic:natural:edge", family: "ceramic", hue: "natural", level: "edge", lightness: 0.97, chroma: 0.0225, hue_degrees: 72.0, rgb: [0.9998472, 0.8970721, 0.7844681] },
    Material { name: "ceramic:natural:deep", family: "ceramic", hue: "natural", level: "deep", lightness: 0.72, chroma: 0.025, hue_degrees: 72.0, rgb: [0.42695814, 0.36341074, 0.2959384] },
    Material { name: "glass:red", family: "glass", hue: "red", level: "body", lightness: 0.66, chroma: 0.1, hue_degrees: 25.0, rgb: [0.5790552, 0.19195753, 0.17127113] },
    Material { name: "glass:red:edge", family: "glass", hue: "red", level: "edge", lightness: 0.76, chroma: 0.1, hue_degrees: 25.0, rgb: [0.82247615, 0.31352186, 0.28272128] },
    Material { name: "glass:red:deep", family: "glass", hue: "red", level: "deep", lightness: 0.48, chroma: 0.1, hue_degrees: 25.0, rgb: [0.2683657, 0.058658816, 0.051510654] },
    Material { name: "glass:amber", family: "glass", hue: "amber", level: "body", lightness: 0.66, chroma: 0.1, hue_degrees: 62.0, rgb: [0.51518834, 0.22973922, 0.07746465] },
    Material { name: "glass:amber:edge", family: "glass", hue: "amber", level: "edge", lightness: 0.76, chroma: 0.1, hue_degrees: 62.0, rgb: [0.7379606, 0.36439905, 0.15318008] },
    Material { name: "glass:amber:deep", family: "glass", hue: "amber", level: "deep", lightness: 0.48, chroma: 0.1, hue_degrees: 62.0, rgb: [0.23434201, 0.077809975, 0.007513112] },
    Material { name: "glass:yellow", family: "glass", hue: "yellow", level: "body", lightness: 0.66, chroma: 0.1, hue_degrees: 115.0, rgb: [0.279586, 0.32065463, 0.076836854] },
    Material { name: "glass:yellow:edge", family: "glass", hue: "yellow", level: "edge", lightness: 0.76, chroma: 0.1, hue_degrees: 115.0, rgb: [0.42756405, 0.4842968, 0.15288597] },
    Material { name: "glass:yellow:deep", family: "glass", hue: "yellow", level: "deep", lightness: 0.48, chroma: 0.1, hue_degrees: 115.0, rgb: [0.10742419, 0.12664223, 0.0066032484] },
    Material { name: "glass:green", family: "glass", hue: "green", level: "body", lightness: 0.66, chroma: 0.1, hue_degrees: 150.0, rgb: [0.124873035, 0.36882982, 0.16590178] },
    Material { name: "glass:green:edge", family: "glass", hue: "green", level: "edge", lightness: 0.76, chroma: 0.1, hue_degrees: 150.0, rgb: [0.22237422, 0.54738915, 0.27595037] },
    Material { name: "glass:green:deep", family: "glass", hue: "green", level: "deep", lightness: 0.48, chroma: 0.1, hue_degrees: 150.0, rgb: [0.02566486, 0.15297663, 0.048284847] },
    Material { name: "glass:cyan", family: "glass", hue: "cyan", level: "body", lightness: 0.66, chroma: 0.1, hue_degrees: 200.0, rgb: [0.02935696, 0.37324572, 0.40332243] },
    Material { name: "glass:cyan:edge", family: "glass", hue: "cyan", level: "edge", lightness: 0.76, chroma: 0.1, hue_degrees: 200.0, rgb: [0.09393834, 0.55372345, 0.5908873] },
    Material { name: "glass:cyan:deep", family: "glass", hue: "cyan", level: "deep", lightness: 0.48, chroma: 0.0816, hue_degrees: 200.0, rgb: [0.000003397872, 0.14728756, 0.1610913] },
    Material { name: "glass:blue", family: "glass", hue: "blue", level: "body", lightness: 0.66, chroma: 0.1, hue_degrees: 250.0, rgb: [0.119253926, 0.30739492, 0.6119837] },
    Material { name: "glass:blue:edge", family: "glass", hue: "blue", level: "edge", lightness: 0.76, chroma: 0.1, hue_degrees: 250.0, rgb: [0.21305473, 0.46780288, 0.8590627] },
    Material { name: "glass:blue:deep", family: "glass", hue: "blue", level: "deep", lightness: 0.48, chroma: 0.1, hue_degrees: 250.0, rgb: [0.024855306, 0.1182809, 0.29412523] },
    Material { name: "glass:violet", family: "glass", hue: "violet", level: "body", lightness: 0.66, chroma: 0.1, hue_degrees: 300.0, rgb: [0.33039156, 0.23037738, 0.5679525] },
    Material { name: "glass:violet:edge", family: "glass", hue: "violet", level: "edge", lightness: 0.76, chroma: 0.1, hue_degrees: 300.0, rgb: [0.49484015, 0.364735, 0.80336785] },
    Material { name: "glass:violet:deep", family: "glass", hue: "violet", level: "deep", lightness: 0.48, chroma: 0.1, hue_degrees: 300.0, rgb: [0.13446294, 0.07863553, 0.2676598] },
    Material { name: "glass:natural", family: "glass", hue: "natural", level: "body", lightness: 0.66, chroma: 0.09, hue_degrees: 220.0, rgb: [0.067254715, 0.34694955, 0.4809612] },
    Material { name: "glass:natural:edge", family: "glass", hue: "natural", level: "edge", lightness: 0.76, chroma: 0.09, hue_degrees: 220.0, rgb: [0.14436796, 0.5191996, 0.6913864] },
    Material { name: "glass:natural:deep", family: "glass", hue: "natural", level: "deep", lightness: 0.48, chroma: 0.0873, hue_degrees: 220.0, rgb: [0.00010757121, 0.13972421, 0.21396822] },
    Material { name: "polymer:red", family: "polymer", hue: "red", level: "body", lightness: 0.34, chroma: 0.05, hue_degrees: 25.0, rgb: [0.07792159, 0.026654446, 0.023834312] },
    Material { name: "polymer:red:edge", family: "polymer", hue: "red", level: "edge", lightness: 0.44, chroma: 0.05, hue_degrees: 25.0, rgb: [0.14898881, 0.06434421, 0.05867142] },
    Material { name: "polymer:red:deep", family: "polymer", hue: "red", level: "deep", lightness: 0.16, chroma: 0.05, hue_degrees: 25.0, rgb: [0.013230763, 0.001067399, 0.0010484203] },
    Material { name: "polymer:amber", family: "polymer", hue: "amber", level: "body", lightness: 0.34, chroma: 0.05, hue_degrees: 62.0, rgb: [0.069451086, 0.031684935, 0.0112727275] },
    Material { name: "polymer:amber:edge", family: "polymer", hue: "amber", level: "edge", lightness: 0.44, chroma: 0.05, hue_degrees: 62.0, rgb: [0.13484748, 0.07299173, 0.03616827] },
    Material { name: "polymer:amber:deep", family: "polymer", hue: "amber", level: "deep", lightness: 0.16, chroma: 0.0368, hue_degrees: 62.0, rgb: [0.009207474, 0.0027215523, 0.0000016251171] },
    Material { name: "polymer:yellow", family: "polymer", hue: "yellow", level: "body", lightness: 0.34, chroma: 0.05, hue_degrees: 115.0, rgb: [0.038233895, 0.0437339, 0.011201331] },
    Material { name: "polymer:yellow:edge", family: "polymer", hue: "yellow", level: "edge", lightness: 0.44, chroma: 0.05, hue_degrees: 115.0, rgb: [0.083129995, 0.092986636, 0.036202416] },
    Material { name: "polymer:yellow:deep", family: "polymer", hue: "yellow", level: "deep", lightness: 0.16, chroma: 0.0362, hue_degrees: 115.0, rgb: [0.003981054, 0.004723316, 0.00000455405] },
    Material { name: "polymer:green", family: "polymer", hue: "green", level: "body", lightness: 0.34, chroma: 0.05, hue_degrees: 150.0, rgb: [0.017703906, 0.050108846, 0.023129608] },
    Material { name: "polymer:green:edge", family: "polymer", hue: "green", level: "edge", lightness: 0.44, chroma: 0.05, hue_degrees: 150.0, rgb: [0.04873739, 0.10343878, 0.05758999] },
    Material { name: "polymer:green:deep", family: "polymer", hue: "green", level: "deep", lightness: 0.16, chroma: 0.044, hue_degrees: 150.0, rgb: [0.0000055433575, 0.0061324565, 0.0011621024] },
    Material { name: "polymer:cyan", family: "polymer", hue: "cyan", level: "body", lightness: 0.34, chroma: 0.05, hue_degrees: 200.0, rgb: [0.004990042, 0.05070551, 0.05463555] },
    Material { name: "polymer:cyan:edge", family: "polymer", hue: "cyan", level: "edge", lightness: 0.44, chroma: 0.05, hue_degrees: 200.0, rgb: [0.026942354, 0.104572505, 0.1103921] },
    Material { name: "polymer:cyan:deep", family: "polymer", hue: "cyan", level: "deep", lightness: 0.16, chroma: 0.0272, hue_degrees: 200.0, rgb: [0.00000012584711, 0.005455095, 0.0059663444] },
    Material { name: "polymer:blue", family: "polymer", hue: "blue", level: "body", lightness: 0.34, chroma: 0.05, hue_degrees: 250.0, rgb: [0.016916353, 0.041999243, 0.08213144] },
    Material { name: "polymer:blue:edge", family: "polymer", hue: "blue", level: "edge", lightness: 0.44, chroma: 0.05, hue_degrees: 250.0, rgb: [0.046896197, 0.09037932, 0.15407252] },
    Material { name: "polymer:blue:deep", family: "polymer", hue: "blue", level: "deep", lightness: 0.16, chroma: 0.0452, hue_degrees: 250.0, rgb: [0.000006420557, 0.0042894143, 0.014130759] },
    Material { name: "polymer:violet", family: "polymer", hue: "violet", level: "body", lightness: 0.34, chroma: 0.05, hue_degrees: 300.0, rgb: [0.04497302, 0.031758565, 0.07634955] },
    Material { name: "polymer:violet:edge", family: "polymer", hue: "violet", level: "edge", lightness: 0.44, chroma: 0.05, hue_degrees: 300.0, rgb: [0.09439509, 0.072965056, 0.14513664] },
    Material { name: "polymer:violet:deep", family: "polymer", hue: "violet", level: "deep", lightness: 0.16, chroma: 0.05, hue_degrees: 300.0, rgb: [0.005548728, 0.0021338433, 0.01375624] },
    Material { name: "polymer:natural", family: "polymer", hue: "natural", level: "body", lightness: 0.34, chroma: 0.018, hue_degrees: 250.0, rgb: [0.030767012, 0.040679604, 0.05304408] },
    Material { name: "polymer:natural:edge", family: "polymer", hue: "natural", level: "edge", lightness: 0.44, chroma: 0.018, hue_degrees: 250.0, rgb: [0.070785806, 0.08757186, 0.10785049] },
    Material { name: "polymer:natural:deep", family: "polymer", hue: "natural", level: "deep", lightness: 0.16, chroma: 0.018, hue_degrees: 250.0, rgb: [0.0022720934, 0.0043444065, 0.0073707905] },
    Material { name: "road:red", family: "road", hue: "red", level: "body", lightness: 0.38, chroma: 0.04, hue_degrees: 25.0, rgb: [0.09281461, 0.042488508, 0.03896083] },
    Material { name: "road:red:edge", family: "road", hue: "red", level: "edge", lightness: 0.48, chroma: 0.04, hue_degrees: 25.0, rgb: [0.17059344, 0.09104824, 0.08482067] },
    Material { name: "road:red:deep", family: "road", hue: "red", level: "deep", lightness: 0.2, chroma: 0.04, hue_degrees: 25.0, rgb: [0.018920034, 0.004407816, 0.0038730274] },
    Material { name: "road:amber", family: "road", hue: "amber", level: "body", lightness: 0.38, chroma: 0.04, hue_degrees: 62.0, rgb: [0.08438224, 0.047682803, 0.02530936] },
    Material { name: "road:amber:edge", family: "road", hue: "amber", level: "edge", lightness: 0.48, chroma: 0.04, hue_degrees: 62.0, rgb: [0.15715985, 0.09948296, 0.06208819] },
    Material { name: "road:amber:deep", family: "road", hue: "amber", level: "deep", lightness: 0.2, chroma: 0.04, hue_degrees: 62.0, rgb: [0.016560063, 0.005745895, 0.00076196354] },
    Material { name: "road:yellow", family: "road", hue: "yellow", level: "body", lightness: 0.38, chroma: 0.04, hue_degrees: 115.0, rgb: [0.053606562, 0.059586152, 0.025353402] },
    Material { name: "road:yellow:edge", family: "road", hue: "yellow", level: "edge", lightness: 0.48, chroma: 0.04, hue_degrees: 115.0, rgb: [0.108404435, 0.118360676, 0.062259443] },
    Material { name: "road:yellow:deep", family: "road", hue: "yellow", level: "deep", lightness: 0.2, chroma: 0.04, hue_degrees: 115.0, rgb: [0.007769882, 0.009129437, 0.00070444366] },
    Material { name: "road:green", family: "road", hue: "green", level: "body", lightness: 0.38, chroma: 0.04, hue_degrees: 150.0, rgb: [0.03308366, 0.06578852, 0.038330525] },
    Material { name: "road:green:edge", family: "road", hue: "green", level: "edge", lightness: 0.48, chroma: 0.04, hue_degrees: 150.0, rgb: [0.07565587, 0.12811083, 0.08387823] },
    Material { name: "road:green:deep", family: "road", hue: "green", level: "deep", lightness: 0.2, chroma: 0.04, hue_degrees: 150.0, rgb: [0.0020912595, 0.010949792, 0.003652874] },
    Material { name: "road:cyan", family: "road", hue: "cyan", level: "body", lightness: 0.38, chroma: 0.04, hue_degrees: 200.0, rgb: [0.020003067, 0.066485085, 0.06984403] },
    Material { name: "road:cyan:edge", family: "road", hue: "cyan", level: "edge", lightness: 0.48, chroma: 0.04, hue_degrees: 200.0, rgb: [0.054467447, 0.12930606, 0.13419197] },
    Material { name: "road:cyan:deep", family: "road", hue: "cyan", level: "deep", lightness: 0.2, chroma: 0.034, hue_degrees: 200.0, rgb: [0.00000024579515, 0.010654482, 0.011653016] },
    Material { name: "road:blue", family: "road", hue: "blue", level: "body", lightness: 0.38, chroma: 0.04, hue_degrees: 250.0, rgb: [0.031907726, 0.058072668, 0.095560044] },
    Material { name: "road:blue:edge", family: "road", hue: "blue", level: "edge", lightness: 0.48, chroma: 0.04, hue_degrees: 250.0, rgb: [0.07345889, 0.116115764, 0.17378864] },
    Material { name: "road:blue:deep", family: "road", hue: "blue", level: "deep", lightness: 0.2, chroma: 0.04, hue_degrees: 250.0, rgb: [0.0020127688, 0.00856323, 0.020622] },
    Material { name: "road:violet", family: "road", hue: "violet", level: "body", lightness: 0.38, chroma: 0.04, hue_degrees: 300.0, rgb: [0.060326472, 0.04764261, 0.09033739] },
    Material { name: "road:violet:edge", family: "road", hue: "violet", level: "edge", lightness: 0.48, chroma: 0.04, hue_degrees: 300.0, rgb: [0.119122334, 0.0993118, 0.16590524] },
    Material { name: "road:violet:deep", family: "road", hue: "violet", level: "deep", lightness: 0.2, chroma: 0.04, hue_degrees: 300.0, rgb: [0.009645476, 0.0057988465, 0.018817233] },
    Material { name: "road:natural", family: "road", hue: "natural", level: "body", lightness: 0.38, chroma: 0.018, hue_degrees: 250.0, rgb: [0.044173248, 0.05661942, 0.07191587] },
    Material { name: "road:natural:edge", family: "road", hue: "natural", level: "edge", lightness: 0.48, chroma: 0.018, hue_degrees: 250.0, rgb: [0.09342298, 0.11346197, 0.13745153] },
    Material { name: "road:natural:deep", family: "road", hue: "natural", level: "deep", lightness: 0.2, chroma: 0.018, hue_degrees: 250.0, rgb: [0.005110641, 0.008421687, 0.012977939] },
    Material { name: "enamel:red", family: "enamel", hue: "red", level: "body", lightness: 0.62, chroma: 0.19, hue_degrees: 25.0, rgb: [0.75831354, 0.065994695, 0.06367555] },
    Material { name: "enamel:red:edge", family: "enamel", hue: "red", level: "edge", lightness: 0.72, chroma: 0.1742, hue_degrees: 25.0, rgb: [0.99998593, 0.16643876, 0.14711177] },
    Material { name: "enamel:red:deep", family: "enamel", hue: "red", level: "deep", lightness: 0.44, chroma: 0.1783, hue_degrees: 25.0, rgb: [0.3406036, 0.000042907373, 0.008367374] },
    Material { name: "enamel:amber", family: "enamel", hue: "amber", level: "body", lightness: 0.62, chroma: 0.1426, hue_degrees: 62.0, rgb: [0.5357419, 0.15835501, 0.00009455832] },
    Material { name: "enamel:amber:edge", family: "enamel", hue: "amber", level: "edge", lightness: 0.72, chroma: 0.1656, hue_degrees: 62.0, rgb: [0.8390311, 0.24800146, 0.00014808879] },
    Material { name: "enamel:amber:deep", family: "enamel", hue: "amber", level: "deep", lightness: 0.44, chroma: 0.1012, hue_degrees: 62.0, rgb: [0.19148669, 0.056599785, 0.000033797358] },
    Material { name: "enamel:yellow", family: "enamel", hue: "yellow", level: "body", lightness: 0.62, chroma: 0.1404, hue_degrees: 115.0, rgb: [0.23164195, 0.27484939, 0.000112125985] },
    Material { name: "enamel:yellow:edge", family: "enamel", hue: "yellow", level: "edge", lightness: 0.72, chroma: 0.1631, hue_degrees: 115.0, rgb: [0.36277816, 0.43045673, 0.00008520552] },
    Material { name: "enamel:yellow:deep", family: "enamel", hue: "yellow", level: "deep", lightness: 0.44, chroma: 0.0997, hue_degrees: 115.0, rgb: [0.08279478, 0.09824269, 0.0000023493901] },
    Material { name: "enamel:green", family: "enamel", hue: "green", level: "body", lightness: 0.62, chroma: 0.1707, hue_degrees: 150.0, rgb: [0.000060567152, 0.3569494, 0.067448504] },
    Material { name: "enamel:green:edge", family: "enamel", hue: "green", level: "edge", lightness: 0.72, chroma: 0.19, hue_degrees: 150.0, rgb: [0.014674476, 0.5518374, 0.11508209] },
    Material { name: "enamel:green:deep", family: "enamel", hue: "green", level: "deep", lightness: 0.44, chroma: 0.1211, hue_degrees: 150.0, rgb: [0.000049311962, 0.12756844, 0.024125524] },
    Material { name: "enamel:cyan", family: "enamel", hue: "cyan", level: "body", lightness: 0.62, chroma: 0.1054, hue_degrees: 200.0, rgb: [0.000007322483, 0.31740767, 0.347155] },
    Material { name: "enamel:cyan:edge", family: "enamel", hue: "cyan", level: "edge", lightness: 0.72, chroma: 0.1224, hue_degrees: 200.0, rgb: [0.000011467818, 0.49709553, 0.5436831] },
    Material { name: "enamel:cyan:deep", family: "enamel", hue: "cyan", level: "deep", lightness: 0.44, chroma: 0.0748, hue_degrees: 200.0, rgb: [0.0000026172265, 0.113448925, 0.12408132] },
    Material { name: "enamel:blue", family: "enamel", hue: "blue", level: "body", lightness: 0.62, chroma: 0.1754, hue_degrees: 250.0, rgb: [0.00010304026, 0.24953568, 0.82330495] },
    Material { name: "enamel:blue:edge", family: "enamel", hue: "blue", level: "edge", lightness: 0.72, chroma: 0.1514, hue_degrees: 250.0, rgb: [0.08156747, 0.39910418, 0.99987113] },
    Material { name: "enamel:blue:deep", family: "enamel", hue: "blue", level: "deep", lightness: 0.44, chroma: 0.1245, hue_degrees: 250.0, rgb: [0.000024527124, 0.08918778, 0.29431844] },
    Material { name: "enamel:violet", family: "enamel", hue: "violet", level: "body", lightness: 0.62, chroma: 0.19, hue_degrees: 300.0, rgb: [0.32078806, 0.12700576, 0.78634834] },
    Material { name: "enamel:violet:edge", family: "enamel", hue: "violet", level: "edge", lightness: 0.72, chroma: 0.172, hue_degrees: 300.0, rgb: [0.46815735, 0.24584424, 0.9997666] },
    Material { name: "enamel:violet:deep", family: "enamel", hue: "violet", level: "deep", lightness: 0.44, chroma: 0.19, hue_degrees: 300.0, rgb: [0.13137844, 0.0222332, 0.39562166] },
    Material { name: "enamel:natural", family: "enamel", hue: "natural", level: "body", lightness: 0.62, chroma: 0.156, hue_degrees: 245.0, rgb: [0.00007528055, 0.26417276, 0.72074664] },
    Material { name: "enamel:natural:edge", family: "enamel", hue: "natural", level: "edge", lightness: 0.72, chroma: 0.1562, hue_degrees: 245.0, rgb: [0.04418149, 0.41422945, 0.9997785] },
    Material { name: "enamel:natural:deep", family: "enamel", hue: "natural", level: "deep", lightness: 0.44, chroma: 0.1107, hue_degrees: 245.0, rgb: [0.000033113593, 0.09442175, 0.2575924] },
    Material { name: "water:red", family: "water", hue: "red", level: "body", lightness: 0.42, chroma: 0.1, hue_degrees: 25.0, rgb: [0.19632606, 0.033763353, 0.029794442] },
    Material { name: "water:red:edge", family: "water", hue: "red", level: "edge", lightness: 0.52, chroma: 0.1, hue_degrees: 25.0, rgb: [0.32458866, 0.08012286, 0.07048478] },
    Material { name: "water:red:deep", family: "water", hue: "red", level: "deep", lightness: 0.24, chroma: 0.0972, hue_degrees: 25.0, rgb: [0.05524762, 0.000016094718, 0.0013627971] },
    Material { name: "water:amber", family: "water", hue: "amber", level: "body", lightness: 0.42, chroma: 0.0966, hue_degrees: 62.0, rgb: [0.16654378, 0.04922714, 0.00002939494] },
    Material { name: "water:amber:edge", family: "water", hue: "amber", level: "edge", lightness: 0.52, chroma: 0.1, hue_degrees: 62.0, rgb: [0.28474697, 0.10286509, 0.017031204] },
    Material { name: "water:amber:deep", family: "water", hue: "amber", level: "deep", lightness: 0.24, chroma: 0.0552, hue_degrees: 62.0, rgb: [0.031075226, 0.00918524, 0.00000548477] },
    Material { name: "water:yellow", family: "water", hue: "yellow", level: "body", lightness: 0.42, chroma: 0.0951, hue_degrees: 115.0, rgb: [0.07200946, 0.08544051, 0.00004028515] },
    Material { name: "water:yellow:edge", family: "water", hue: "yellow", level: "edge", lightness: 0.52, chroma: 0.1, hue_degrees: 115.0, rgb: [0.13655984, 0.15992859, 0.016148629] },
    Material { name: "water:yellow:deep", family: "water", hue: "yellow", level: "deep", lightness: 0.24, chroma: 0.0543, hue_degrees: 115.0, rgb: [0.013436057, 0.015941191, 0.000015369918] },
    Material { name: "water:green", family: "water", hue: "green", level: "body", lightness: 0.42, chroma: 0.1, hue_degrees: 150.0, rgb: [0.009497746, 0.10628773, 0.027174791] },
    Material { name: "water:green:edge", family: "water", hue: "green", level: "edge", lightness: 0.52, chroma: 0.1, hue_degrees: 150.0, rgb: [0.040578015, 0.19055983, 0.06682477] },
    Material { name: "water:green:deep", family: "water", hue: "green", level: "deep", lightness: 0.24, chroma: 0.066, hue_degrees: 150.0, rgb: [0.000018708832, 0.02069704, 0.0039220955] },
    Material { name: "water:cyan", family: "water", hue: "cyan", level: "body", lightness: 0.42, chroma: 0.0714, hue_degrees: 200.0, rgb: [0.0000022763088, 0.09867116, 0.10791858] },
    Material { name: "water:cyan:edge", family: "water", hue: "cyan", level: "edge", lightness: 0.52, chroma: 0.0884, hue_degrees: 200.0, rgb: [0.0000043200953, 0.18726318, 0.20481342] },
    Material { name: "water:cyan:deep", family: "water", hue: "cyan", level: "deep", lightness: 0.24, chroma: 0.0408, hue_degrees: 200.0, rgb: [0.000000424734, 0.018410945, 0.020136412] },
    Material { name: "water:blue", family: "water", hue: "blue", level: "body", lightness: 0.42, chroma: 0.1, hue_degrees: 250.0, rgb: [0.00976248, 0.078809865, 0.21952885] },
    Material { name: "water:blue:edge", family: "water", hue: "blue", level: "edge", lightness: 0.52, chroma: 0.1, hue_degrees: 250.0, rgb: [0.038905352, 0.15058132, 0.3520063] },
    Material { name: "water:blue:deep", family: "water", hue: "blue", level: "deep", lightness: 0.24, chroma: 0.0679, hue_degrees: 250.0, rgb: [0.0000054538727, 0.014474002, 0.047757182] },
    Material { name: "water:violet", family: "water", hue: "violet", level: "body", lightness: 0.42, chroma: 0.1, hue_degrees: 300.0, rgb: [0.09285137, 0.048902314, 0.19793908] },
    Material { name: "water:violet:edge", family: "water", hue: "violet", level: "edge", lightness: 0.52, chroma: 0.1, hue_degrees: 300.0, rgb: [0.16822675, 0.10368861, 0.3220183] },
    Material { name: "water:violet:deep", family: "water", hue: "violet", level: "deep", lightness: 0.24, chroma: 0.1, hue_degrees: 300.0, rgb: [0.02096656, 0.004100894, 0.061763424] },
    Material { name: "water:natural", family: "water", hue: "natural", level: "body", lightness: 0.42, chroma: 0.07, hue_degrees: 240.0, rgb: [0.018112175, 0.083940126, 0.16004391] },
    Material { name: "water:natural:edge", family: "water", hue: "natural", level: "edge", lightness: 0.52, chroma: 0.07, hue_degrees: 240.0, rgb: [0.053182077, 0.15692179, 0.267688] },
    Material { name: "water:natural:deep", family: "water", hue: "natural", level: "deep", lightness: 0.24, chroma: 0.0549, hue_degrees: 240.0, rgb: [0.000016104697, 0.015946431, 0.037400894] },
    Material { name: "organic:red", family: "organic", hue: "red", level: "body", lightness: 0.3, chroma: 0.12, hue_degrees: 25.0, rgb: [0.10675207, 0.00042302892, 0.0028735672] },
    Material { name: "organic:red:edge", family: "organic", hue: "red", level: "edge", lightness: 0.4, chroma: 0.12, hue_degrees: 25.0, rgb: [0.20034917, 0.018829955, 0.017873049] },
    Material { name: "organic:red:deep", family: "organic", hue: "red", level: "deep", lightness: 0.12, chroma: 0.0486, hue_degrees: 25.0, rgb: [0.0069059525, 0.0000020118398, 0.00017034964] },
    Material { name: "organic:amber", family: "organic", hue: "amber", level: "body", lightness: 0.3, chroma: 0.069, hue_degrees: 62.0, rgb: [0.0606938, 0.01793992, 0.0000107124415] },
    Material { name: "organic:amber:edge", family: "organic", hue: "amber", level: "edge", lightness: 0.4, chroma: 0.092, hue_degrees: 62.0, rgb: [0.14386679, 0.042524256, 0.000025392454] },
    Material { name: "organic:amber:deep", family: "organic", hue: "amber", level: "deep", lightness: 0.12, chroma: 0.0276, hue_degrees: 62.0, rgb: [0.0038844033, 0.001148155, 0.00000068559626] },
    Material { name: "organic:yellow", family: "organic", hue: "yellow", level: "body", lightness: 0.3, chroma: 0.0679, hue_degrees: 115.0, rgb: [0.0262424, 0.031136107, 0.0000228605] },
    Material { name: "organic:yellow:edge", family: "organic", hue: "yellow", level: "edge", lightness: 0.4, chroma: 0.0906, hue_degrees: 115.0, rgb: [0.06220468, 0.07380869, 0.00002026253] },
    Material { name: "organic:yellow:deep", family: "organic", hue: "yellow", level: "deep", lightness: 0.12, chroma: 0.0271, hue_degrees: 115.0, rgb: [0.0016794752, 0.001992339, 0.0000042138386] },
    Material { name: "organic:green", family: "organic", hue: "green", level: "body", lightness: 0.3, chroma: 0.0826, hue_degrees: 150.0, rgb: [0.000005872381, 0.040439017, 0.0076405522] },
    Material { name: "organic:green:edge", family: "organic", hue: "green", level: "edge", lightness: 0.4, chroma: 0.1101, hue_degrees: 150.0, rgb: [0.000032092477, 0.0958465, 0.018122666] },
    Material { name: "organic:green:deep", family: "organic", hue: "green", level: "deep", lightness: 0.12, chroma: 0.033, hue_degrees: 150.0, rgb: [0.000002338604, 0.00258713, 0.00049026194] },
    Material { name: "organic:cyan", family: "organic", hue: "cyan", level: "body", lightness: 0.3, chroma: 0.051, hue_degrees: 200.0, rgb: [0.0000008295586, 0.03595888, 0.03932893] },
    Material { name: "organic:cyan:edge", family: "organic", hue: "cyan", level: "edge", lightness: 0.4, chroma: 0.068, hue_degrees: 200.0, rgb: [0.0000019663612, 0.08523586, 0.09322413] },
    Material { name: "organic:cyan:deep", family: "organic", hue: "cyan", level: "deep", lightness: 0.12, chroma: 0.0204, hue_degrees: 200.0, rgb: [0.00000005309175, 0.0023013682, 0.0025170515] },
    Material { name: "organic:blue", family: "organic", hue: "blue", level: "body", lightness: 0.3, chroma: 0.0849, hue_degrees: 250.0, rgb: [0.0000043208624, 0.02826845, 0.09330149] },
    Material { name: "organic:blue:edge", family: "organic", hue: "blue", level: "edge", lightness: 0.4, chroma: 0.1132, hue_degrees: 250.0, rgb: [0.0000102420445, 0.0670067, 0.22115909] },
    Material { name: "organic:blue:deep", family: "organic", hue: "blue", level: "deep", lightness: 0.12, chroma: 0.0339, hue_degrees: 250.0, rgb: [0.0000027086724, 0.0018095967, 0.005961414] },
    Material { name: "organic:violet", family: "organic", hue: "violet", level: "body", lightness: 0.3, chroma: 0.12, hue_degrees: 300.0, rgb: [0.040206213, 0.009043851, 0.115512624] },
    Material { name: "organic:violet:edge", family: "organic", hue: "violet", level: "edge", lightness: 0.4, chroma: 0.12, hue_degrees: 300.0, rgb: [0.08555682, 0.03491318, 0.20717543] },
    Material { name: "organic:violet:deep", family: "organic", hue: "violet", level: "deep", lightness: 0.12, chroma: 0.064, hue_degrees: 300.0, rgb: [0.0029860649, 0.0000021737515, 0.010249225] },
    Material { name: "organic:natural", family: "organic", hue: "natural", level: "body", lightness: 0.3, chroma: 0.03, hue_degrees: 140.0, rgb: [0.019253936, 0.031460833, 0.017025413] },
    Material { name: "organic:natural:edge", family: "organic", hue: "natural", level: "edge", lightness: 0.4, chroma: 0.03, hue_degrees: 140.0, rgb: [0.050145503, 0.07200928, 0.045927115] },
    Material { name: "organic:natural:deep", family: "organic", hue: "natural", level: "deep", lightness: 0.12, chroma: 0.03, hue_degrees: 140.0, rgb: [0.0005324699, 0.0024008984, 0.00030726727] },
    Material { name: "soil:red", family: "soil", hue: "red", level: "body", lightness: 0.52, chroma: 0.06, hue_degrees: 25.0, rgb: [0.24762292, 0.10564962, 0.09622545] },
    Material { name: "soil:red:edge", family: "soil", hue: "red", level: "edge", lightness: 0.62, chroma: 0.06, hue_degrees: 25.0, rgb: [0.38931307, 0.18908812, 0.17442264] },
    Material { name: "soil:red:deep", family: "soil", hue: "red", level: "deep", lightness: 0.34, chroma: 0.06, hue_degrees: 25.0, rgb: [0.086197585, 0.023906572, 0.021116389] },
    Material { name: "soil:amber", family: "soil", hue: "amber", level: "body", lightness: 0.52, chroma: 0.06, hue_degrees: 62.0, rgb: [0.2239181, 0.12012314, 0.058640078] },
    Material { name: "soil:amber:edge", family: "soil", hue: "amber", level: "edge", lightness: 0.62, chroma: 0.06, hue_degrees: 62.0, rgb: [0.35566348, 0.2099702, 0.11899669] },
    Material { name: "soil:amber:deep", family: "soil", hue: "amber", level: "deep", lightness: 0.34, chroma: 0.06, hue_degrees: 62.0, rgb: [0.07599869, 0.029809495, 0.0069359276] },
    Material { name: "soil:yellow", family: "soil", hue: "yellow", level: "body", lightness: 0.52, chroma: 0.06, hue_degrees: 115.0, rgb: [0.13718867, 0.1536514, 0.05868353] },
    Material { name: "soil:yellow:edge", family: "soil", hue: "yellow", level: "edge", lightness: 0.62, chroma: 0.06, hue_degrees: 115.0, rgb: [0.2331127, 0.25738987, 0.11926948] },
    Material { name: "soil:yellow:deep", family: "soil", hue: "yellow", level: "deep", lightness: 0.34, chroma: 0.06, hue_degrees: 115.0, rgb: [0.038180687, 0.044384293, 0.0067576896] },
    Material { name: "soil:green", family: "soil", hue: "green", level: "body", lightness: 0.52, chroma: 0.06, hue_degrees: 150.0, rgb: [0.07954622, 0.17118956, 0.094404176] },
    Material { name: "soil:green:edge", family: "soil", hue: "green", level: "edge", lightness: 0.62, chroma: 0.06, hue_degrees: 150.0, rgb: [0.15115978, 0.28201592, 0.17196678] },
    Material { name: "soil:green:deep", family: "soil", hue: "green", level: "deep", lightness: 0.34, chroma: 0.06, hue_degrees: 150.0, rgb: [0.013554194, 0.052170284, 0.020209834] },
    Material { name: "soil:cyan", family: "soil", hue: "cyan", level: "body", lightness: 0.52, chroma: 0.06, hue_degrees: 200.0, rgb: [0.043061294, 0.17307794, 0.18289845] },
    Material { name: "soil:cyan:edge", family: "soil", hue: "cyan", level: "edge", lightness: 0.62, chroma: 0.06, hue_degrees: 200.0, rgb: [0.09862109, 0.28487855, 0.2978319] },
    Material { name: "soil:cyan:deep", family: "soil", hue: "cyan", level: "deep", lightness: 0.34, chroma: 0.0578, hue_degrees: 200.0, rgb: [0.0000012075915, 0.052345473, 0.05725127] },
    Material { name: "soil:blue", family: "soil", hue: "blue", level: "body", lightness: 0.52, chroma: 0.06, hue_degrees: 250.0, rgb: [0.07650576, 0.14925633, 0.25631315] },
    Material { name: "soil:blue:edge", family: "soil", hue: "blue", level: "edge", lightness: 0.62, chroma: 0.06, hue_degrees: 250.0, rgb: [0.14615251, 0.25151396, 0.3991197] },
    Material { name: "soil:blue:deep", family: "soil", hue: "blue", level: "deep", lightness: 0.34, chroma: 0.06, hue_degrees: 250.0, rgb: [0.01294331, 0.042100817, 0.092527494] },
    Material { name: "soil:violet", family: "soil", hue: "violet", level: "body", lightness: 0.52, chroma: 0.06, hue_degrees: 300.0, rgb: [0.15607065, 0.1200924, 0.24127187] },
    Material { name: "soil:violet:edge", family: "soil", hue: "violet", level: "edge", lightness: 0.62, chroma: 0.06, hue_degrees: 300.0, rgb: [0.25994012, 0.20970844, 0.378704] },
    Material { name: "soil:violet:deep", family: "soil", hue: "violet", level: "deep", lightness: 0.34, chroma: 0.06, hue_degrees: 300.0, rgb: [0.04628896, 0.029980684, 0.08510298] },
    Material { name: "soil:natural", family: "soil", hue: "natural", level: "body", lightness: 0.52, chroma: 0.005, hue_degrees: 260.0, rgb: [0.13609903, 0.14110748, 0.14940839] },
    Material { name: "soil:natural:edge", family: "soil", hue: "natural", level: "edge", lightness: 0.62, chroma: 0.005, hue_degrees: 260.0, rgb: [0.23191191, 0.2390439, 0.25081316] },
    Material { name: "soil:natural:deep", family: "soil", hue: "natural", level: "deep", lightness: 0.34, chroma: 0.005, hue_degrees: 260.0, rgb: [0.03738241, 0.03951176, 0.043091495] },
];

/// The colours this table owns. `hud.rs` resolves these rather than carrying
/// literals, so an icon is never measured against a copy of a colour.
pub const TOKEN_COLOURS: &[(&str, [f32; 4])] = &[
    ("Agent", [0.7226559, 0.78453785, 0.890638, 1.0]),
    ("CaseOpen", [0.81974816, 0.30368575, 0.00053456455, 1.0]),
    ("Desk", [0.0038004562, 0.0049987733, 0.0073557785, 1.0]),
    ("Grid", [0.038837552, 0.04328174, 0.05104877, 1.0]),
    ("Ground", [0.019253936, 0.031460833, 0.017025413, 1.0]),
    ("Ink", [0.000043074193, 0.095410906, 0.3147767, 1.0]),
    ("Nature", [0.07243466, 0.4780014, 0.14489588, 1.0]),
    ("NotObtained", [0.14653288, 0.16828988, 0.20715414, 1.0]),
    ("Panel", [0.012791147, 0.01586739, 0.02167422, 1.0]),
    ("PanelRaised", [0.024797067, 0.030230582, 0.040361214, 1.0]),
    ("Plaque", [0.092606135, 0.07131653, 0.043180045, 1.0]),
    ("Refused", [0.7278254, 0.07626843, 0.07093878, 1.0]),
    ("Retired", [0.24681339, 0.27739376, 0.33116874, 1.0]),
    ("Road", [0.044173248, 0.05661942, 0.07191587, 1.0]),
    ("RoadEdge", [0.13609903, 0.14110748, 0.14940839, 1.0]),
    ("Scaffold", [0.3343722, 0.19446994, 0.046290666, 1.0]),
    ("TextBody", [0.86932635, 0.8864781, 0.914557, 1.0]),
    ("TextMuted", [0.49069375, 0.5143282, 0.5538166, 1.0]),
    ("Warning", [0.81974816, 0.30368575, 0.00053456455, 1.0]),
    ("Water", [0.018112175, 0.083940126, 0.16004391, 1.0]),
    ("ZoneCommercial", [0.022802262, 0.17705375, 0.48996267, 1.0]),
    ("ZoneIndustrial", [0.38420415, 0.10669558, 0.00007315424, 1.0]),
    ("ZoneResidential", [0.022327544, 0.23819055, 0.06168903, 1.0]),
    ("IconBlack", [0.0022720934, 0.0043444065, 0.0073707905, 1.0]),
];

/// Tokens that **are** a surface's material read out, so the two cannot drift.
pub const SURFACE_TOKENS: &[(&str, &str)] = &[
    ("Ground", "terrain.ground"),
    ("Water", "terrain.water"),
    ("Road", "road.surface"),
    ("RoadEdge", "road.edge"),
];

/// Where the table's reading of a surface differs from the literal it replaced.
/// Superseded, never erased: the shipping value is still in the declaration.
pub const DIVERGENCES: &[(&str, &str, &str)] = &[
    ("Ink", "Ink declared oklch(0.45, 0.13, 250.0), which linear sRGB cannot hold — the frame silently clipped it", "now oklch(0.45, 0.1273, 250.0), resolved in gamut"),
    ("Road", "Road was oklch(0.42, 0.005, 260.0)", "now the reading of road:natural:body — oklch(0.38, 0.018, 250.0)"),
    ("ZoneIndustrial", "ZoneIndustrial declared oklch(0.55, 0.13, 60.0), which linear sRGB cannot hold — the frame silently clipped it", "now oklch(0.55, 0.1289, 60.0), resolved in gamut"),
];

/// part, family, hue, level, note — every part of the world, as a claim.
pub const WORLD_SURFACES: &[(&str, &str, &str, &str, &str)] = &[
    ("home.walls", "ceramic", "natural", "body", "the home's body"),
    ("home.roof", "polymer", "amber", "edge", "a different material from the walls, read once"),
    ("home.window", "glass", "cyan", "body", "the one transparent part"),
    ("shop.walls", "ceramic", "natural", "body", "shares the residential body"),
    ("shop.frontage", "enamel", "blue", "body", "the commercial face paint belongs to"),
    ("shop.window", "glass", "cyan", "body", ""),
    ("factory.frame", "metal", "natural", "body", "structural metal"),
    ("factory.cladding", "enamel", "amber", "body", "coated industrial panel"),
    ("factory.vent", "polymer", "natural", "deep", "the dark recessed part"),
    ("power.frame", "metal", "natural", "body", ""),
    ("power.stack", "ceramic", "natural", "edge", "a fired stack, not a painted one"),
    ("power.insulator", "ceramic", "amber", "body", "the insulator is the part that must not conduct"),
    ("power.core", "enamel", "amber", "body", "painted caution face"),
    ("road.surface", "road", "natural", "body", "aggregate"),
    ("road.edge", "soil", "natural", "body", "the shoulder: mineral, not asphalt"),
    ("terrain.ground", "organic", "natural", "body", "the grass is a living surface, not a mineral one"),
    ("terrain.water", "water", "natural", "body", "a body of water is a volume, not a pane"),
    ("powerline.conductor", "metal", "natural", "body", "what a conductor must be made of"),
    ("powerline.pylon", "metal", "natural", "deep", "the same metal read darker"),
    ("scaffold.frame", "metal", "amber", "edge", "under construction, and painted to say so"),
    ("scaffold.deck", "paper", "natural", "body", "a board, not a beam"),
    ("zone.paint", "enamel", "__anchor__", "body", "a tint over whatever it paints on; the anchor is the zone's"),
    ("ruin.parts", "__inherits__", "__inherits__", "deep", "the retired structure's own parts, darker"),
];

/// Surfaces the world does not have yet: a declared absence, never an invention.
pub const DEFERRED_SURFACES: &[(&str, &str)] = &[
    ("terrain.vegetation", "the world draws one green ground; a second vegetation surface does not exist, so `organic` has one consumer and not two"),
    ("soil.paving", "declared slot: `soil` is the road shoulder today; unpaved ground and yards would be its second consumer"),
];

/// A structure's build cost is the sum of its parts' prices, per a173.
pub const PART_PRICE: &[(&str, f32)] = &[
    ("home.walls", 60.0),
    ("home.roof", 40.0),
    ("home.window", 40.0),
    ("shop.walls", 70.0),
    ("shop.frontage", 90.0),
    ("shop.window", 80.0),
    ("factory.frame", 180.0),
    ("factory.cladding", 120.0),
    ("factory.vent", 60.0),
    ("power.frame", 700.0),
    ("power.stack", 400.0),
    ("power.insulator", 250.0),
    ("power.core", 700.0),
    ("road.surface", 3.0),
    ("road.edge", 1.0),
    ("powerline.conductor", 12.0),
    ("powerline.pylon", 30.0),
    ("scaffold.frame", 8.0),
    ("scaffold.deck", 4.0),
];

/// Credits per month, per part, by family.
pub const UPKEEP_PER_MONTH: &[(&str, f32)] = &[
    ("metal", 2.0),
    ("enamel", 1.6),
    ("glass", 1.5),
    ("ceramic", 1.2),
    ("polymer", 0.8),
    ("paper", 0.3),
    ("road", 0.4),
    ("water", 0.0),
    ("organic", 0.05),
    ("soil", 0.1),
];

/// Condition lost per sim-day, by family.
pub const DECAY_PER_DAY: &[(&str, f32)] = &[
    ("metal", 0.0008),
    ("enamel", 0.0006),
    ("glass", 0.0005),
    ("ceramic", 0.0004),
    ("polymer", 0.0012),
    ("paper", 0.0018),
    ("road", 0.0006),
    ("water", 0.0),
    ("organic", 0.0015),
    ("soil", 0.0009),
];

pub const REPAIR_FLOOR: f32 = 0.35;
pub const REPAIR_CEILING: f32 = 1.0;
pub const REPAIR_SHARE: f32 = 0.4;

/// Whether a family carries power, and how much.
pub const CONDUCTION: &[(&str, f32)] = &[
    ("metal", 1.0),
    ("enamel", 0.0),
    ("glass", 0.0),
    ("ceramic", 0.0),
    ("polymer", 0.0),
    ("paper", 0.0),
    ("road", 0.0),
    ("water", 0.2),
    ("organic", 0.0),
    ("soil", 0.0),
];

/// Travel speed over a surface; 0.0 means unbuildable.
pub const SURFACE_SPEED: &[(&str, f32)] = &[
    ("road", 1.0),
    ("soil", 0.6),
    ("organic", 0.8),
    ("water", 0.0),
    ("metal", 0.0),
    ("enamel", 0.0),
    ("glass", 0.0),
    ("ceramic", 0.0),
    ("polymer", 0.0),
    ("paper", 0.0),
];

/// How a structure moves demand around it.
pub const DESIRABILITY: &[(&str, f32)] = &[
    ("metal", -0.02),
    ("enamel", 0.03),
    ("glass", 0.04),
    ("ceramic", 0.02),
    ("polymer", -0.01),
    ("paper", 0.0),
    ("road", -0.04),
    ("water", 0.03),
    ("organic", 0.05),
    ("soil", -0.01),
];

/// Noise and pollution per structure, sampled at read points.
pub const NUISANCE: &[(&str, f32)] = &[
    ("metal", 0.3),
    ("enamel", 0.1),
    ("glass", 0.0),
    ("ceramic", 0.05),
    ("polymer", 0.1),
    ("paper", 0.0),
    ("road", 0.25),
    ("water", 0.0),
    ("organic", 0.0),
    ("soil", 0.1),
];

/// One declared rational: a numerator over a denominator, in integers.
///
/// A float is what the schema refuses, because a ledger in mixed units balances
/// only if every conversion is exact — so the type holding a density, a unit mass,
/// a rot rate and a quantity has no float in it at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rational {
    pub num: i64,
    pub den: i64,
}

/// One substance: what it is measured in, what it presents as, and where it
/// comes from.
#[derive(Clone, Copy, Debug)]
pub struct Substance {
    pub name: &'static str,
    pub family: &'static str,
    pub hue: &'static str,
    /// Mass | Volume | Count | Gas — the canonical unit the ledger counts in.
    pub unit: &'static str,
    /// g/mL, required for Volume and Gas; zero where the unit does not need it.
    pub density: Rational,
    /// Grams per unit, required for Count; zero where the unit does not need it.
    pub unit_mass: Rational,
    /// Condition lost per sim-day. All zero today: rot's mechanism arrives with
    /// phase 7's couplings, and a rate nothing reads would be a placeholder.
    pub rot_per_day: Rational,
    pub tags: &'static [&'static str],
    pub source: &'static str,
    /// The natural unit a person reads it in, and its exact grams per unit. Never
    /// used in arithmetic — Q67's per-substance units are a reading, not a conversion.
    pub display_unit: &'static str,
    pub display_grams: Rational,
    /// Why nothing consumes it, or empty when something does.
    pub no_consumer: &'static str,
    pub note: &'static str,
}

/// What a process costs to run, as numbers rather than as prose (Q37's rule).
#[derive(Clone, Copy, Debug)]
pub struct Mechanism {
    pub heat_c: i64,
    /// `material` or `flame`, empty when nothing is heated. The two readings of
    /// "kiln temperature" differ by ~600 °C and both are correct.
    pub heat_kind: &'static str,
    pub hours: Rational,
    pub labour_hours: Rational,
    pub power_kw: i64,
}

/// One process: what it takes, what it gives, and the number behind it.
#[derive(Clone, Copy, Debug)]
pub struct Process {
    pub name: &'static str,
    pub tier: &'static str,
    pub inputs: &'static [(&'static str, Rational)],
    pub outputs: &'static [(&'static str, Rational)],
    pub mechanism: Mechanism,
    /// What must exist for the work to be possible: (kind, name) pairs, each a
    /// declared entry in the vocabulary that kind owns.
    pub requires: &'static [(&'static str, &'static str)],
    pub note: &'static str,
}

/// The ages, as rows. An age is *reached* rather than assumed, so the ordinal is
/// what a progression compares against.
pub const TIERS: &[(&str, i64, &str)] = &[
    ("hands & stone", 0, "no structure and no tool: what a person does with hands and with stone picked up and used"),
    ("bound & composite", 1, "things joined to other things — cord, haft, assembly — which is the first real manufacturing"),
];

/// Every declared substance. The type has no float in it, which is the point.
pub const SUBSTANCES: &[Substance] = &[
    Substance {
        name: "stone",
        family: "ceramic", hue: "natural", unit: "Mass",
        density: Rational { num: 13, den: 5 }, unit_mass: Rational { num: 0, den: 1 }, rot_per_day: Rational { num: 0, den: 1 },
        tags: &["structure"] as &[&str],
        source: "mined", display_unit: "t", display_grams: Rational { num: 1000000, den: 1 },
        no_consumer: "", note: "the deposit kind `stone`; consumed by knapping, and the bulk of every masonry age",
    },
    Substance {
        name: "sand",
        family: "ceramic", hue: "natural", unit: "Mass",
        density: Rational { num: 8, den: 5 }, unit_mass: Rational { num: 0, den: 1 }, rot_per_day: Rational { num: 0, den: 1 },
        tags: &["structure"] as &[&str],
        source: "mined", display_unit: "t", display_grams: Rational { num: 1000000, den: 1 },
        no_consumer: "the aggregate half of glass and mortar: glass waits for the kiln and mortar for the lime process, both of which need SOURCES [NS] rows", note: "the deposit kind `sand`; the aggregate half of glass and mortar",
    },
    Substance {
        name: "clay",
        family: "ceramic", hue: "natural", unit: "Mass",
        density: Rational { num: 19, den: 10 }, unit_mass: Rational { num: 0, den: 1 }, rot_per_day: Rational { num: 0, den: 1 },
        tags: &["structure"] as &[&str],
        source: "mined", display_unit: "t", display_grams: Rational { num: 1000000, den: 1 },
        no_consumer: "unfired it is mud, which is why brick waits for the kiln — and the kiln is a process structure this table has not declared yet", note: "the deposit kind `clay`; unfired it is mud, which is why brick waits for the kiln",
    },
    Substance {
        name: "coal",
        family: "soil", hue: "natural", unit: "Mass",
        density: Rational { num: 13, den: 10 }, unit_mass: Rational { num: 0, den: 1 }, rot_per_day: Rational { num: 0, den: 1 },
        tags: &["fuel"] as &[&str],
        source: "mined", display_unit: "t", display_grams: Rational { num: 1000000, den: 1 },
        no_consumer: "the fuel of the coal ages: nothing burns anything until the smelt lands, and the smelt needs SOURCES [NS] rows (coke per tonne, blast temperature)", note: "",
    },
    Substance {
        name: "iron_ore",
        family: "metal", hue: "natural", unit: "Mass",
        density: Rational { num: 27, den: 10 }, unit_mass: Rational { num: 0, den: 1 }, rot_per_day: Rational { num: 0, den: 1 },
        tags: &[] as &[&str],
        source: "mined", display_unit: "t", display_grams: Rational { num: 1000000, den: 1 },
        no_consumer: "the metal ages' input: the ore-to-blade rung is blocked on the iron ore grade row in `tools/materials/SOURCES.md`, which is [NS] — and an invented grade would make the whole ledger look sourced while being declared", note: "",
    },
    Substance {
        name: "timber",
        family: "organic", hue: "natural", unit: "Mass",
        density: Rational { num: 7, den: 10 }, unit_mass: Rational { num: 0, den: 1 }, rot_per_day: Rational { num: 0, den: 1 },
        tags: &["fuel", "structure"] as &[&str],
        source: "gathered", display_unit: "kg", display_grams: Rational { num: 1000, den: 1 },
        no_consumer: "", note: "taken from a standing surface deposit (Q102); the first material a founder touches",
    },
    Substance {
        name: "plant_fibre",
        family: "organic", hue: "natural", unit: "Mass",
        density: Rational { num: 3, den: 10 }, unit_mass: Rational { num: 0, den: 1 }, rot_per_day: Rational { num: 0, den: 1 },
        tags: &[] as &[&str],
        source: "gathered", display_unit: "kg", display_grams: Rational { num: 1000, den: 1 },
        no_consumer: "", note: "brush and cordage stock; the only thing available to bind with before metal",
    },
    Substance {
        name: "haft_blank",
        family: "organic", hue: "natural", unit: "Mass",
        density: Rational { num: 7, den: 10 }, unit_mass: Rational { num: 0, den: 1 }, rot_per_day: Rational { num: 0, den: 1 },
        tags: &[] as &[&str],
        source: "made", display_unit: "kg", display_grams: Rational { num: 1000, den: 1 },
        no_consumer: "", note: "a riven haft, before assembly: the same timber, one process later",
    },
    Substance {
        name: "knapped_edge",
        family: "ceramic", hue: "natural", unit: "Mass",
        density: Rational { num: 13, den: 5 }, unit_mass: Rational { num: 0, den: 1 }, rot_per_day: Rational { num: 0, den: 1 },
        tags: &[] as &[&str],
        source: "made", display_unit: "kg", display_grams: Rational { num: 1000, den: 1 },
        no_consumer: "", note: "a worked stone edge: flaked, not ground, which is what the stone age actually did",
    },
    Substance {
        name: "cord",
        family: "organic", hue: "natural", unit: "Mass",
        density: Rational { num: 1, den: 2 }, unit_mass: Rational { num: 0, den: 1 }, rot_per_day: Rational { num: 0, den: 1 },
        tags: &[] as &[&str],
        source: "made", display_unit: "kg", display_grams: Rational { num: 1000, den: 1 },
        no_consumer: "", note: "twisted fibre; the first thing in the world that binds two other things together",
    },
    Substance {
        name: "hatchet",
        family: "ceramic", hue: "natural", unit: "Count",
        density: Rational { num: 0, den: 1 }, unit_mass: Rational { num: 2900, den: 1 }, rot_per_day: Rational { num: 0, den: 1 },
        tags: &["tool"] as &[&str],
        source: "made", display_unit: "hatchet", display_grams: Rational { num: 0, den: 1 },
        no_consumer: "a tool is held and used, not consumed by a process: what consumes it is wear, which is the MAINTAIN half of phase 3 and not a row in this rung", note: "the edge is the part that identifies it, so the table reads it as the stone it is made of — a tool of two materials presents as the one that says what it does",
    },
    Substance {
        name: "timber_offcuts",
        family: "organic", hue: "natural", unit: "Mass",
        density: Rational { num: 7, den: 10 }, unit_mass: Rational { num: 0, den: 1 }, rot_per_day: Rational { num: 0, den: 1 },
        tags: &[] as &[&str],
        source: "made", display_unit: "kg", display_grams: Rational { num: 1000, den: 1 },
        no_consumer: "an end product: offcuts leave the process and accumulate. They are mass the ledger can still point at, which is why a loss is declared as an output rather than subtracted from the total", note: "declared, not hoped for: a process that loses mass silently is a process that creates it",
    },
    Substance {
        name: "stone_flakes",
        family: "ceramic", hue: "natural", unit: "Mass",
        density: Rational { num: 13, den: 5 }, unit_mass: Rational { num: 0, den: 1 }, rot_per_day: Rational { num: 0, den: 1 },
        tags: &[] as &[&str],
        source: "made", display_unit: "kg", display_grams: Rational { num: 1000, den: 1 },
        no_consumer: "an end product: debitage. Salvage (Q71) is where it earns a consumer", note: "knapping loses more than half its stone as flakes, and the ratio is declared here",
    },
    Substance {
        name: "fibre_dust",
        family: "organic", hue: "natural", unit: "Mass",
        density: Rational { num: 3, den: 10 }, unit_mass: Rational { num: 0, den: 1 }, rot_per_day: Rational { num: 0, den: 1 },
        tags: &[] as &[&str],
        source: "made", display_unit: "kg", display_grams: Rational { num: 1000, den: 1 },
        no_consumer: "an end product: short fibres too fine to twist", note: "",
    },
    Substance {
        name: "trim_waste",
        family: "soil", hue: "natural", unit: "Mass",
        density: Rational { num: 1, den: 1 }, unit_mass: Rational { num: 0, den: 1 }, rot_per_day: Rational { num: 0, den: 1 },
        tags: &[] as &[&str],
        source: "made", display_unit: "kg", display_grams: Rational { num: 1000, den: 1 },
        no_consumer: "an end product: mixed-material trimmings, and the one declared destination for a process whose losses are not one material", note: "declared as `soil` because a mixture of stone, timber and fibre reads as aggregate",
    },
    Substance {
        name: "water",
        family: "water", hue: "natural", unit: "Volume",
        density: Rational { num: 1, den: 1 }, unit_mass: Rational { num: 0, den: 1 }, rot_per_day: Rational { num: 0, den: 1 },
        tags: &[] as &[&str],
        source: "gathered", display_unit: "L", display_grams: Rational { num: 1000, den: 1 },
        no_consumer: "drinking, washing and irrigation arrive with phase 7's couplings; the world already draws it, so the substance exists and the consumer does not", note: "declared in millilitres with a density of 1 g/mL, so the conversion is exact by construction rather than by a constant that happens to be 1",
    },
];

/// Every declared process. `SCHEMA.md` is the contract these rows must satisfy,
/// and `schema.py` is the gate that refuses a row which does not.
pub const PROCESSES: &[Process] = &[
    Process {
        name: "gather timber", tier: "hands & stone",
        inputs: &[],
        outputs: &[("timber", Rational { num: 3000, den: 1 })],
        mechanism: Mechanism { heat_c: 0, heat_kind: "", hours: Rational { num: 1, den: 4 }, labour_hours: Rational { num: 1, den: 4 }, power_kw: 0 },
        requires: &[] as &[(&str, &str)],
        note: "3000 g is one armful; the rate is declared (SOURCES: durations and labour costs are `[D]`, because no source states what a game gather costs)",
    },
    Process {
        name: "riven haft", tier: "hands & stone",
        inputs: &[("timber", Rational { num: 3000, den: 1 })],
        outputs: &[("haft_blank", Rational { num: 2400, den: 1 }), ("timber_offcuts", Rational { num: 600, den: 1 })],
        mechanism: Mechanism { heat_c: 0, heat_kind: "", hours: Rational { num: 1, den: 3 }, labour_hours: Rational { num: 1, den: 3 }, power_kw: 0 },
        requires: &[] as &[(&str, &str)],
        note: "riving, not sawing: splitting along the grain is what the stone age can do, and it is why a haft is a blank rather than a cut board",
    },
    Process {
        name: "gather stone", tier: "hands & stone",
        inputs: &[],
        outputs: &[("stone", Rational { num: 1000, den: 1 })],
        mechanism: Mechanism { heat_c: 0, heat_kind: "", hours: Rational { num: 1, den: 4 }, labour_hours: Rational { num: 1, den: 4 }, power_kw: 0 },
        requires: &[] as &[(&str, &str)],
        note: "1000 g is one carried load",
    },
    Process {
        name: "knap a core", tier: "hands & stone",
        inputs: &[("stone", Rational { num: 1000, den: 1 })],
        outputs: &[("knapped_edge", Rational { num: 800, den: 1 }), ("stone_flakes", Rational { num: 200, den: 1 })],
        mechanism: Mechanism { heat_c: 0, heat_kind: "", hours: Rational { num: 1, den: 2 }, labour_hours: Rational { num: 1, den: 2 }, power_kw: 0 },
        requires: &[] as &[(&str, &str)],
        note: "the 800/200 split is declared `[D]`: real knapping loses more, but a usable edge is what the process is for and the flakes are the loss",
    },
    Process {
        name: "gather plant fibre", tier: "hands & stone",
        inputs: &[],
        outputs: &[("plant_fibre", Rational { num: 100, den: 1 })],
        mechanism: Mechanism { heat_c: 0, heat_kind: "", hours: Rational { num: 1, den: 6 }, labour_hours: Rational { num: 1, den: 6 }, power_kw: 0 },
        requires: &[] as &[(&str, &str)],
        note: "",
    },
    Process {
        name: "twist cord", tier: "bound & composite",
        inputs: &[("plant_fibre", Rational { num: 100, den: 1 })],
        outputs: &[("cord", Rational { num: 95, den: 1 }), ("fibre_dust", Rational { num: 5, den: 1 })],
        mechanism: Mechanism { heat_c: 0, heat_kind: "", hours: Rational { num: 1, den: 4 }, labour_hours: Rational { num: 1, den: 4 }, power_kw: 0 },
        requires: &[] as &[(&str, &str)],
        note: "the first process in `bound & composite`, and the reason that tier exists: binding is what makes a composite tool possible before metal is",
    },
    Process {
        name: "assemble the hatchet", tier: "bound & composite",
        inputs: &[("knapped_edge", Rational { num: 800, den: 1 }), ("haft_blank", Rational { num: 2400, den: 1 }), ("cord", Rational { num: 95, den: 1 })],
        outputs: &[("hatchet", Rational { num: 1, den: 1 }), ("trim_waste", Rational { num: 395, den: 1 })],
        mechanism: Mechanism { heat_c: 0, heat_kind: "", hours: Rational { num: 1, den: 2 }, labour_hours: Rational { num: 1, den: 2 }, power_kw: 0 },
        requires: &[] as &[(&str, &str)],
        note: "the chain's end, and Q7's own example: a hatchet fashioned from natural materials, which is the tool that multiplies work before anything is mined",
    },
];

/// The closed vocabularies a process may require from: what exists, as rows.
/// Empty today because the first rung is hand work — and an entry no process
/// requires is refused by the gate, so this cannot fill with intentions.
pub const STRUCTURES: &[(&str, &str)] = &[
];

pub const TOOLS: &[(&str, &str)] = &[
];

pub const SKILLS: &[(&str, &str)] = &[
];

/// What the substance and process tables deliberately do not decide.
pub const SCHEMA_OPEN: &[&str] = &[
    "`sand` is declared and nothing consumes it: the aggregate half of glass and mortar: glass waits for the kiln and mortar for the lime process, both of which need SOURCES [NS] rows",
    "`clay` is declared and nothing consumes it: unfired it is mud, which is why brick waits for the kiln — and the kiln is a process structure this table has not declared yet",
    "`coal` is declared and nothing consumes it: the fuel of the coal ages: nothing burns anything until the smelt lands, and the smelt needs SOURCES [NS] rows (coke per tonne, blast temperature)",
    "`iron_ore` is declared and nothing consumes it: the metal ages' input: the ore-to-blade rung is blocked on the iron ore grade row in `tools/materials/SOURCES.md`, which is [NS] — and an invented grade would make the whole ledger look sourced while being declared",
    "`hatchet` is declared and nothing consumes it: a tool is held and used, not consumed by a process: what consumes it is wear, which is the MAINTAIN half of phase 3 and not a row in this rung",
    "`timber_offcuts` is declared and nothing consumes it: an end product: offcuts leave the process and accumulate. They are mass the ledger can still point at, which is why a loss is declared as an output rather than subtracted from the total",
    "`stone_flakes` is declared and nothing consumes it: an end product: debitage. Salvage (Q71) is where it earns a consumer",
    "`fibre_dust` is declared and nothing consumes it: an end product: short fibres too fine to twist",
    "`trim_waste` is declared and nothing consumes it: an end product: mixed-material trimmings, and the one declared destination for a process whose losses are not one material",
    "`water` is declared and nothing consumes it: drinking, washing and irrigation arrive with phase 7's couplings; the world already draws it, so the substance exists and the consumer does not",
    "`gather timber` draws 3000 g of `timber` out of the world's own ground",
    "`gather stone` draws 1000 g of `stone` out of the world's own ground",
    "`gather plant fibre` draws 100 g of `plant_fibre` out of the world's own ground",
    "the ages: two tiers are declared and Q69 asked for as many as possible — the content is unbounded and an age is rows, so adding one touches no schema",
    "the numbers in these tables are declared, not measured: every duration, labour cost, power figure and loss ratio is `[D]`, and the rung above the stone one is blocked on `[NS]` source rows rather than on this schema",
];

/// What this table cannot check, printed every run rather than implied away.
pub const OPEN: &[&str] = &[
    "whether the world reads as these materials to a person",
    "frame timing with the material shade term",
    "whether the four sim effects are balanced",
    "composite contrast on a rendered frame",
];

