//! The interface.
//!
//! Colours are defined in **OKLCH** and converted to linear sRGB, so the values
//! the shader receives and the values the contrast test measures are the same
//! numbers. Contrast is *measured* rather than asserted: [`tests`] computes
//! every pair the interface actually draws.
//!
//! The token set is closed. [`style`] fails closed on a token with no record,
//! and [`style_by_name`] refuses a name it does not have, so a typo cannot
//! silently paint a default colour that nobody notices until a screenshot.
//!
//! No raster textures are used: fills are solid and everything else is
//! geometry, which is why there is no provenance file for this file to carry.
//!
//! Sizes and spacing do **not** come from here: they come from [`crate::design`],
//! whose steps are a closed enum so a call site cannot carry its own number.

use std::sync::LazyLock;

use crate::design::{Space, Step, UiScale};
use crate::render::{Batcher, Face, Screen, Text};

// ---------------------------------------------------------------------------
// Colour
// ---------------------------------------------------------------------------

/// OKLCH to linear sRGB. No gamma step: the surface is an sRGB target, so it
/// does the encoding, and the linear values are what a contrast ratio wants.
pub fn oklch(l: f32, c: f32, h_deg: f32) -> [f32; 4] {
    let h = h_deg.to_radians();
    let a = c * h.cos();
    let b = c * h.sin();

    let l_ = l + 0.396_337_78 * a + 0.215_803_76 * b;
    let m_ = l - 0.105_561_35 * a - 0.063_854_17 * b;
    let s_ = l - 0.089_484_18 * a - 1.291_485_5 * b;

    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;

    let r = 4.076_741_7 * l3 - 3.307_711_6 * m3 + 0.230_969_93 * s3;
    let g = -1.268_438 * l3 + 2.609_757_4 * m3 - 0.341_319_4 * s3;
    let b = -0.004_196_09 * l3 - 0.703_418_6 * m3 + 1.707_614_7 * s3;

    [
        r.clamp(0.0, 1.0),
        g.clamp(0.0, 1.0),
        b.clamp(0.0, 1.0),
        1.0,
    ]
}

pub fn with_alpha(color: [f32; 4], alpha: f32) -> [f32; 4] {
    [color[0], color[1], color[2], alpha]
}

/// WCAG relative luminance, from linear values.
pub fn relative_luminance(color: [f32; 4]) -> f32 {
    0.2126 * color[0] + 0.7152 * color[1] + 0.0722 * color[2]
}

/// WCAG contrast ratio between two opaque colours.
pub fn contrast_ratio(a: [f32; 4], b: [f32; 4]) -> f32 {
    let (la, lb) = (relative_luminance(a), relative_luminance(b));
    let (lighter, darker) = if la > lb { (la, lb) } else { (lb, la) };
    (lighter + 0.05) / (darker + 0.05)
}

// ---------------------------------------------------------------------------
// Tokens
// ---------------------------------------------------------------------------

/// Every colour role the interface can paint. Closed on purpose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Token {
    Desk,
    Panel,
    PanelRaised,
    Plaque,
    Ink,
    Nature,
    Warning,
    Refused,
    NotObtained,
    TextBody,
    TextMuted,
    TextOnInk,
    TextPlaque,
    Grid,
    Ground,
    Water,
    Road,
    RoadEdge,
    Scaffold,
    Retired,
    ZoneResidential,
    ZoneCommercial,
    ZoneIndustrial,
    Powered,
    Brownout,
    Agent,
    AgentStuck,
    CaseOpen,
    Verified,
    Correction,
    Recording,
    Procedural,
}

pub const ALL_TOKENS: [Token; 32] = [
    Token::Desk,
    Token::Panel,
    Token::PanelRaised,
    Token::Plaque,
    Token::Ink,
    Token::Nature,
    Token::Warning,
    Token::Refused,
    Token::NotObtained,
    Token::TextBody,
    Token::TextMuted,
    Token::TextOnInk,
    Token::TextPlaque,
    Token::Grid,
    Token::Ground,
    Token::Water,
    Token::Road,
    Token::RoadEdge,
    Token::Scaffold,
    Token::Retired,
    Token::ZoneResidential,
    Token::ZoneCommercial,
    Token::ZoneIndustrial,
    Token::Powered,
    Token::Brownout,
    Token::Agent,
    Token::AgentStuck,
    Token::CaseOpen,
    Token::Verified,
    Token::Correction,
    Token::Recording,
    Token::Procedural,
];

#[derive(Clone, Copy, Debug)]
pub struct Style {
    pub fill: Option<[f32; 4]>,
    pub text: Option<[f32; 4]>,
    pub border: Option<[f32; 4]>,
}

fn solid_fill(color: [f32; 4]) -> Style {
    Style {
        fill: Some(color),
        text: None,
        border: None,
    }
}

fn ink(color: [f32; 4]) -> Style {
    Style {
        fill: None,
        text: Some(color),
        border: None,
    }
}

/// A colour that is both a fill and an ink.
///
/// These started as ink-only, which meant `Token::Warning` drew a refusal's
/// accent bar with a token that had no fill — invisible, and green across every
/// test that only looked at colours rather than at what was painted. The
/// measurement test is what caught it, which is the argument for measuring.
fn tint(color: [f32; 4]) -> Style {
    Style {
        fill: Some(color),
        text: Some(color),
        border: None,
    }
}

fn boxed(fill: [f32; 4], border: [f32; 4]) -> Style {
    Style {
        fill: Some(fill),
        text: None,
        border: Some(border),
    }
}

//: A colour this table does **not** own is a literal here, and the two reasons are
//: both deliberate: it is either pure interface chrome that no icon is ever measured
//: against (text pairs, borders, the plaque's own ink), or it is *derived* from an
//: owned colour at the point of use (`Powered` is Nature at 55 % alpha). Everything
//: else — the host surfaces, the world's surfaces, and the state colours the world
//: draws — is resolved from [`crate::materials`], which is the one home for it.
use crate::materials::token_or_defect as material_token;

static STYLES: LazyLock<Vec<(Token, Style)>> = LazyLock::new(|| {
    // Desk and paper are warm neutrals that do not scroll away; text sits on a
    // scrim rather than on a texture, because there are no textures.
    let desk = material_token("Desk");
    let panel = material_token("Panel");
    let panel_raised = material_token("PanelRaised");
    let plaque = material_token("Plaque");
    let ink_blue = material_token("Ink");
    let nature = material_token("Nature");
    let warning = material_token("Warning");
    let refused = material_token("Refused");
    let not_obtained = material_token("NotObtained");
    let text_body = material_token("TextBody");
    let text_muted = material_token("TextMuted");
    let on_ink = oklch(0.98, 0.005, 250.0);

    vec![
        (Token::Desk, solid_fill(desk)),
        (Token::Panel, boxed(panel, oklch(0.38, 0.020, 260.0))),
        (
            Token::PanelRaised,
            boxed(panel_raised, oklch(0.46, 0.025, 260.0)),
        ),
        (Token::Plaque, solid_fill(plaque)),
        // A primary constructive action: solid, high contrast, one job.
        (Token::Ink, solid_fill(ink_blue)),
        // Measured active or verified *state*. Never a grant of authority.
        (Token::Nature, tint(nature)),
        // Escalation and never-granted: its own row and its own tier.
        (Token::Warning, tint(warning)),
        (Token::Refused, tint(refused)),
        (Token::NotObtained, ink(not_obtained)),
        (Token::TextBody, ink(text_body)),
        (Token::TextMuted, ink(text_muted)),
        (Token::TextOnInk, ink(on_ink)),
        (Token::TextPlaque, ink(oklch(0.97, 0.01, 80.0))),
        (Token::Grid, ink(with_alpha(material_token("Grid"), 0.35))),
        // The world's surfaces are the *readings* of their materials, and the gate
        // checks that to the last float: a road is aggregate before it is a colour.
        (Token::Ground, solid_fill(material_token("Ground"))),
        (Token::Water, solid_fill(material_token("Water"))),
        (Token::Road, solid_fill(material_token("Road"))),
        (Token::RoadEdge, solid_fill(material_token("RoadEdge"))),
        // Under construction: a named solid, not a half-drawn building.
        (Token::Scaffold, solid_fill(material_token("Scaffold"))),
        // Retired structures stay visible. The city shows its own history.
        (Token::Retired, ink(with_alpha(material_token("Retired"), 0.55))),
        (Token::ZoneResidential, solid_fill(material_token("ZoneResidential"))),
        (Token::ZoneCommercial, solid_fill(material_token("ZoneCommercial"))),
        (Token::ZoneIndustrial, solid_fill(material_token("ZoneIndustrial"))),
        (Token::Powered, ink(with_alpha(nature, 0.55))),
        (Token::Brownout, ink(warning)),
        (Token::Agent, solid_fill(material_token("Agent"))),
        (Token::AgentStuck, solid_fill(refused)),
        (Token::CaseOpen, ink(warning)),
        (Token::Verified, ink(nature)),
        (Token::Correction, ink(oklch(0.72, 0.090, 300.0))),
        (Token::Recording, ink(refused)),
        (Token::Procedural, ink(text_muted)),
    ]
});

/// Look up a token's style. **Fails closed**: a token with no record is a
/// defect, and it stops here rather than painting something plausible.
pub fn style(token: Token) -> Style {
    STYLES
        .iter()
        .find(|(t, _)| *t == token)
        .map(|(_, s)| *s)
        .unwrap_or_else(|| panic!("no style record for token {token:?}; the table failed closed"))
}

/// Look up a style by name. A name that is not in the table is refused, never
/// defaulted.
pub fn style_by_name(name: &str) -> Option<Style> {
    ALL_TOKENS
        .iter()
        .find(|token| {
            format!("{token:?}").eq_ignore_ascii_case(name)
        })
        .map(|token| style(*token))
}

/// The mechanical half of the design check, run at startup and logged.
///
/// It reports what it *measured*. A mechanical pass means the requirement is
/// present in the source; it does not mean the surface looks right, and the
/// report says so rather than letting a green line imply more than it checked.
///
/// Judgement items — sampled composite contrast on a rendered frame, whether a
/// layout reads well, whether the frame budget holds under load — are **not**
/// established here and are named as open.
pub fn audit() -> Vec<String> {
    let mut lines = Vec::new();

    let missing: Vec<Token> = ALL_TOKENS
        .iter()
        .copied()
        .filter(|token| !STYLES.iter().any(|(t, _)| t == token))
        .collect();
    lines.push(format!(
        "style table: {} tokens, {} records, {} with no record",
        ALL_TOKENS.len(),
        STYLES.len(),
        missing.len()
    ));

    for (ink, surface) in [
        (Token::TextBody, Token::Panel),
        (Token::TextBody, Token::Desk),
        (Token::TextMuted, Token::Panel),
        (Token::TextOnInk, Token::Ink),
    ] {
        let ratio = contrast_ratio(
            style(ink).text.unwrap_or([0.0, 0.0, 0.0, 1.0]),
            style(surface).fill.unwrap_or([0.0, 0.0, 0.0, 1.0]),
        );
        lines.push(format!(
            "{ink:?} on {surface:?}: {ratio:.2}:1 (AA floor 4.5)",
        ));
    }

    lines.push(match style_by_name("DefinitelyNotAToken") {
        None => "unknown style names are refused".to_string(),
        Some(_) => "DEFECT: an unknown style name was accepted".to_string(),
    });
    lines.push(
        "open: composite contrast on a rendered frame, layout judgement, frame budget under load"
            .to_string(),
    );
    lines
}

/// The one measured number the interface shows back to the player, so a claim
/// about legibility is a reading rather than a promise.
pub fn measured_body_on_panel() -> f32 {
    contrast_ratio(
        style(Token::TextBody).text.unwrap_or([1.0, 1.0, 1.0, 1.0]),
        style(Token::Panel).fill.unwrap_or([0.0, 0.0, 0.0, 1.0]),
    )
}

// ---------------------------------------------------------------------------
// Primitives
// ---------------------------------------------------------------------------

/// A padding or inset from the design's spacing scale.
pub fn space(space: Space, ui: UiScale) -> f32 {
    space.px(ui)
}

pub fn panel(
    batch: &mut Batcher,
    screen: &Screen,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    token: Token,
) {
    let style = style(token);
    if let Some(fill) = style.fill {
        let uv = Text::solid_uv();
        batch.screen_rect(screen, x, y, w, h, fill, uv);
    }
    if let Some(border) = style.border {
        batch.screen_outline(screen, x, y, w, h, border);
    }
}

pub fn fill(
    batch: &mut Batcher,
    screen: &Screen,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    color: [f32; 4],
) {
    batch.screen_rect(screen, x, y, w, h, color, Text::solid_uv());
}

#[allow(clippy::too_many_arguments)]
pub fn label(
    text: &mut Text,
    batch: &mut Batcher,
    screen: &Screen,
    x: f32,
    y: f32,
    step: Step,
    token: Token,
    body: &str,
) -> f32 {
    let color = style(token).text.unwrap_or([1.0, 1.0, 1.0, 1.0]);
    text.draw_step(Face::Body, batch, screen, x, y, step, color, body)
}

/// Numbers, identifiers and versions: monospace, so a digit can be checked by
/// eye and a column of them lines up.
#[allow(clippy::too_many_arguments)]
pub fn label_mono(
    text: &mut Text,
    batch: &mut Batcher,
    screen: &Screen,
    x: f32,
    y: f32,
    step: Step,
    token: Token,
    body: &str,
) -> f32 {
    let color = style(token).text.unwrap_or([1.0, 1.0, 1.0, 1.0]);
    text.draw_step(Face::Mono, batch, screen, x, y, step, color, body)
}

#[allow(clippy::too_many_arguments)]
pub fn bar(
    batch: &mut Batcher,
    screen: &Screen,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    token: Token,
    fraction: f32,
) {
    let color = match style(token).fill {
        Some(fill) => fill,
        None => style(token).text.unwrap_or([1.0, 1.0, 1.0, 1.0]),
    };
    let track = with_alpha(color, 0.18);
    fill(batch, screen, x, y, w, h, track);
    let filled = (w * fraction.clamp(0.0, 1.0)).max(0.0);
    if filled > 0.5 {
        fill(batch, screen, x, y, filled, h, color);
    }
}

pub fn rule(batch: &mut Batcher, screen: &Screen, x: f32, y: f32, w: f32, token: Token) {
    if let Some(color) = style(token).text {
        fill(batch, screen, x, y, w, 1.0, with_alpha(color, 0.5));
    }
}

pub fn hit(x: f32, y: f32, w: f32, h: f32, px: f32, py: f32) -> bool {
    px >= x && py >= y && px <= x + w && py <= y + h
}

/// Clip a line to a width, with an ellipsis. Long objectives must not spill
/// across the surface and cover a control.
pub fn truncate(text: &mut Text, face: Face, body: &str, step: Step, max_width: f32) -> String {
    if text.measure_step(face, body, step) <= max_width {
        return body.to_string();
    }
    let mut out = String::new();
    for ch in body.chars() {
        let mut candidate = out.clone();
        candidate.push(ch);
        candidate.push('…');
        if text.measure_step(face, &candidate, step) > max_width {
            break;
        }
        out.push(ch);
    }
    out.push('…');
    out
}

/// Every mechanical defect in the token table, for the design check to fail
/// closed on. Returns all of them rather than the first: a check that reports
/// one problem per run is one somebody stops running.
pub fn style_defects() -> Vec<String> {
    let mut defects = Vec::new();
    for token in ALL_TOKENS {
        if !STYLES.iter().any(|(t, _)| *t == token) {
            defects.push(format!(
                "token {token:?} has no style record; the table failed closed"
            ));
        }
    }
    for (ink_token, surface) in [
        (Token::TextBody, Token::Panel),
        (Token::TextBody, Token::Desk),
        (Token::TextMuted, Token::Panel),
        (Token::TextOnInk, Token::Ink),
    ] {
        let text = style(ink_token).text.unwrap_or([0.0, 0.0, 0.0, 1.0]);
        let background = style(surface).fill.unwrap_or([0.0, 0.0, 0.0, 1.0]);
        let ratio = contrast_ratio(text, background);
        if ratio < 4.5 {
            defects.push(format!(
                "{ink_token:?} on {surface:?} measures {ratio:.2}:1, below the 4.5:1 floor"
            ));
        }
    }
    if style_by_name("DefinitelyNotAToken").is_some() {
        defects.push("an unknown style name was accepted instead of refused".to_string());
    }
    defects
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_token_has_a_style_record() {
        for token in ALL_TOKENS {
            let style = style(token);
            assert!(
                style.fill.is_some() || style.text.is_some(),
                "{token:?} has neither a fill nor a text colour, so drawing it does anything at all"
            );
        }
        assert_eq!(
            STYLES.len(),
            ALL_TOKENS.len(),
            "the table and the token list disagree, which means one of them is missing a row"
        );
    }

    #[test]
    fn an_unknown_style_name_is_refused_not_defaulted() {
        assert!(style_by_name("DefinitelyNotAToken").is_none());
        assert!(style_by_name("Ink").is_some());
        assert!(style_by_name("ink").is_some());
    }

    #[test]
    fn body_text_meets_aa_on_the_surfaces_it_is_drawn_on() {
        // Measured, not asserted: the same numbers the shader receives.
        let pairs = [
            (Token::TextBody, Token::Panel),
            (Token::TextBody, Token::Desk),
            (Token::TextBody, Token::PanelRaised),
            (Token::TextMuted, Token::Panel),
            (Token::TextOnInk, Token::Ink),
        ];
        for (ink_token, surface) in pairs {
            let text = style(ink_token).text.expect("a text colour");
            let background = style(surface)
                .fill
                .expect("a fill colour for a surface");
            let ratio = contrast_ratio(text, background);
            assert!(
                ratio >= 4.5,
                "{ink_token:?} on {surface:?} measures {ratio:.2}:1, below the 4.5:1 floor"
            );
        }
    }

    #[test]
    fn accents_meet_the_non_text_contrast_floor() {
        // 1.4.11 asks 3:1 for non-text UI, and large text counts as large text
        // rather than as body text. These are the accent pairs, scored honestly
        // at their own threshold rather than folded in with body copy.
        let accents = [
            Token::Nature,
            Token::Warning,
            Token::Refused,
            Token::Correction,
            Token::CaseOpen,
            Token::Verified,
        ];
        for token in accents {
            let color = style(token).text.expect("an accent colour");
            let background = style(Token::Panel).fill.expect("panel fill");
            let ratio = contrast_ratio(color, background);
            assert!(
                ratio >= 3.0,
                "{token:?} on Panel measures {ratio:.2}:1, below the 3:1 non-text floor"
            );
        }
    }

    #[test]
    fn a_primary_action_is_not_the_same_colour_as_an_escalation() {
        let ink = style(Token::Ink).fill.expect("ink fill");
        let warning = style(Token::Warning).text.expect("warning colour");
        let refused = style(Token::Refused).text.expect("refused colour");
        assert!(
            contrast_ratio(ink, warning) > 1.5,
            "a primary action and an escalation must not read as the same tier"
        );
        assert!(
            contrast_ratio(ink, refused) > 1.5,
            "a primary action and a refusal must not read as the same tier"
        );
    }

    #[test]
    fn characterisation_of_oklch_is_stable_and_in_range() {
        let dark = oklch(0.17, 0.012, 260.0);
        let light = oklch(0.96, 0.005, 260.0);
        for value in [dark, light] {
            for (channel, component) in value.iter().enumerate().take(3) {
                assert!(
                    (0.0..=1.0).contains(component),
                    "channel {channel} came out as {component}"
                );
            }
        }
        assert!(
            relative_luminance(light) > relative_luminance(dark),
            "a lighter OKLCH value must be a lighter linear colour"
        );
    }

    #[test]
    fn contrast_is_symmetric() {
        let a = oklch(0.9, 0.01, 260.0);
        let b = oklch(0.2, 0.01, 260.0);
        assert!((contrast_ratio(a, b) - contrast_ratio(b, a)).abs() < 1e-6);
    }

    #[test]
    fn truncation_never_exceeds_its_budget() {
        let mut text = Text::new();
        let long = "district 3: 412 residents have no job within reach of the roads they can walk";
        let clipped = truncate(&mut text, Face::Body, long, Step::Small, 120.0);
        assert!(
            text.measure_step(Face::Body, &clipped, Step::Small) <= 120.0 + 1.0,
            "the clipped string overran its width"
        );
        assert!(clipped.ends_with('…'));
        assert_eq!(
            truncate(&mut text, Face::Body, "short", Step::Small, 400.0),
            "short"
        );
    }

    #[test]
    fn the_mechanical_defect_list_is_empty() {
        assert_eq!(style_defects(), Vec::<String>::new());
    }
}
