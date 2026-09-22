//! The design scales, and the check that keeps them.
//!
//! `UNIFIED_DESIGN.md` §3.3 asks for a display face, a body face and a
//! monospace face for numbers; §3.4 asks for an 8dp/4-unit spacing scale and
//! 48×48 targets where the viewport allows. Those are only real if they are
//! *structured*: a scale whose values live at their call sites is a convention,
//! and a convention is what the last build had — **46 of 61 text call sites at
//! 12 px, with six off-scale literals** (11, 12, 16, 24, 32, 40) leaking in.
//!
//! So the steps are a closed enum and the draw calls take it. A literal cannot
//! reach a text call site because there is no parameter that accepts one, which
//! is a stronger guarantee than a check would be.
//!
//! [`verify`] is the mechanical half, and it **fails closed**: it returns every
//! defect rather than the first, and the client refuses to start on a `Err`. A
//! scale that quietly drifts is the defect class this exists to prevent.
//!
//! What this does **not** establish is legibility. A scale can be perfectly
//! consistent and still unreadable; that is a judgement item and the playtest
//! record is where it is answered.

use crate::hud;

// ---------------------------------------------------------------------------
// UI scale
// ---------------------------------------------------------------------------

/// The scales a player can pick. 100 % is the design size; the rest exist
/// because "legible at every supported size" (design doc §2) cannot be claimed
/// on one display with one pair of eyes.
pub const UI_SCALES: [f32; 4] = [1.0, 1.25, 1.5, 2.0];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiScale(pub f32);

impl Default for UiScale {
    fn default() -> Self {
        Self(1.0)
    }
}

impl UiScale {
    /// The next scale up, wrapping. Ordered by the array, not by float luck.
    pub fn next(self) -> Self {
        let index = UI_SCALES
            .iter()
            .position(|scale| (*scale - self.0).abs() < f32::EPSILON)
            .unwrap_or(0);
        Self(UI_SCALES[(index + 1) % UI_SCALES.len()])
    }

    pub fn label(self) -> String {
        format!("{:.0}%", self.0 * 100.0)
    }
}

// ---------------------------------------------------------------------------
// Type scale
// ---------------------------------------------------------------------------

/// Every text size the interface can draw. Closed on purpose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// Only for micro-labels that sit beside something they annotate.
    Micro,
    Small,
    Body,
    Title,
    Display,
}

impl Step {
    pub const ALL: [Step; 5] = [
        Step::Micro,
        Step::Small,
        Step::Body,
        Step::Title,
        Step::Display,
    ];

    /// Point size at 100 % UI scale. These are the design numbers, and the
    /// floors below are the reason they are these numbers: 12 px was doing 46
    /// of 61 jobs in the last build, which is what made the interface hard to
    /// read at any display scale.
    pub fn logical_px(self) -> f32 {
        match self {
            Step::Micro => 12.0,
            Step::Small => 14.0,
            Step::Body => 16.0,
            Step::Title => 20.0,
            Step::Display => 26.0,
        }
    }

    /// Device pixels at a UI scale, always a whole number.
    ///
    /// Glyphs are rasterised at this size and drawn at it, so a fractional
    /// point size would resample every glyph — which is the other half of why
    /// the last build's text looked soft.
    pub fn px(self, ui: UiScale) -> u32 {
        (self.logical_px() * ui.0).round() as u32
    }

    pub fn name(self) -> &'static str {
        match self {
            Step::Micro => "micro",
            Step::Small => "small",
            Step::Body => "body",
            Step::Title => "title",
            Step::Display => "display",
        }
    }
}

/// The smallest size allowed anywhere. Text below this is not a size, it is a
/// decision to make a claim unreadable.
pub const MIN_STEP_PX: f32 = 12.0;

/// Body text must be at least this. The design doc asks for legibility "at
/// every supported size"; the measured problem was that 12 px was carrying the
/// message text, so the floor for message text is stated here rather than left
/// to whoever picks a size next.
pub const MIN_BODY_PX: f32 = 16.0;

// ---------------------------------------------------------------------------
// Spacing scale
// ---------------------------------------------------------------------------

/// The design doc's 4-unit scale (§3.4). Every padding, gap and inset is one of
/// these; anything else is off-scale by construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Space {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    Xxl,
}

impl Space {
    pub const ALL: [Space; 6] = [
        Space::Xs,
        Space::Sm,
        Space::Md,
        Space::Lg,
        Space::Xl,
        Space::Xxl,
    ];

    /// Base value at 100 % UI scale. All multiples of four.
    pub fn logical_px(self) -> f32 {
        match self {
            Space::Xs => 4.0,
            Space::Sm => 8.0,
            Space::Md => 12.0,
            Space::Lg => 16.0,
            Space::Xl => 24.0,
            Space::Xxl => 32.0,
        }
    }

    pub fn px(self, ui: UiScale) -> f32 {
        (self.logical_px() * ui.0).round()
    }
}

/// The spacing unit itself. Every value on the scale is a whole multiple.
pub const UNIT: f32 = 4.0;

// ---------------------------------------------------------------------------
// Targets
// ---------------------------------------------------------------------------

/// Design doc §3.4: meet or exceed 48×48 where the logical viewport allows, and
/// never fall below WCAG 2.5.8's 24×24. A desktop window at 1600×900 has the
/// room, so the comfortable floor applies and a shortfall is a defect.
pub const MIN_TARGET_PX: f32 = 48.0;

/// WCAG 2.5.8, which applies even where comfort does not.
pub const HARD_FLOOR_PX: f32 = 24.0;

pub fn target(ui: UiScale) -> f32 {
    (MIN_TARGET_PX * ui.0).round()
}

/// One interactive rectangle the player can hit.
#[derive(Clone, Copy, Debug)]
pub struct Target {
    pub name: &'static str,
    pub w: f32,
    pub h: f32,
}

// ---------------------------------------------------------------------------
// The check
// ---------------------------------------------------------------------------

/// Every mechanical defect in the scales, or an empty vector.
///
/// Deliberately not "the first defect": a check that reports one problem per
/// run is a check somebody runs five times and then stops running.
pub fn verify(targets: &[Target], ui: UiScale) -> Vec<String> {
    let mut defects = Vec::new();

    // --- the type scale ---------------------------------------------------
    let mut previous = 0.0;
    for step in Step::ALL {
        let logical = step.logical_px();
        if logical <= previous {
            defects.push(format!(
                "type step `{}` is {logical} px, which is not larger than the step below it ({previous} px)",
                step.name()
            ));
        }
        if logical < MIN_STEP_PX {
            defects.push(format!(
                "type step `{}` is {logical} px, below the {MIN_STEP_PX} px floor",
                step.name()
            ));
        }
        if step.px(ui) == 0 {
            defects.push(format!(
                "type step `{}` rasterises to zero pixels at {}",
                step.name(),
                ui.label()
            ));
        }
        previous = logical;
    }
    let body = Step::Body.logical_px();
    if body < MIN_BODY_PX {
        defects.push(format!(
            "body text is {body} px, below the {MIN_BODY_PX} px floor for message text; the last build carried its body text at 12 px and it was unreadable"
        ));
    }

    // --- the spacing scale ------------------------------------------------
    for space in Space::ALL {
        let value = space.logical_px();
        if (value / UNIT).fract().abs() > f32::EPSILON {
            defects.push(format!(
                "spacing step {value} px is not a multiple of the {UNIT} px unit"
            ));
        }
        if value <= 0.0 {
            defects.push("a spacing step is zero or negative".to_string());
        }
    }

    // --- targets ----------------------------------------------------------
    for target in targets {
        let comfortable = MIN_TARGET_PX * ui.0;
        let hard = HARD_FLOOR_PX * ui.0;
        if target.w < hard || target.h < hard {
            defects.push(format!(
                "target `{}` is {:.0}×{:.0} px, below the WCAG 2.5.8 floor of {hard:.0}",
                target.name, target.w, target.h
            ));
        } else if target.w < comfortable || target.h < comfortable {
            // Not a refusal: the design doc allows a shortfall where the
            // viewport cannot fit it, but only if it is *stated*.
            defects.push(format!(
                "target `{}` is {:.0}×{:.0} px, below the comfortable {comfortable:.0} px floor and does not declare the shortfall",
                target.name, target.w, target.h
            ));
        }
    }

    // --- tokens and contrast ---------------------------------------------
    defects.extend(hud::style_defects());

    defects
}

/// The logged form of the same numbers, including what was measured and what
/// was not. A green line that quietly includes an unmeasured claim is worse
/// than no line at all.
pub fn audit(targets: &[Target], ui: UiScale) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!(
        "ui scale {} · {} type steps · {} spacing steps · {} targets",
        ui.label(),
        Step::ALL.len(),
        Space::ALL.len(),
        targets.len()
    ));
    for step in Step::ALL {
        lines.push(format!(
            "type {}: {:.0} px → {} px",
            step.name(),
            step.logical_px(),
            step.px(ui)
        ));
    }
    let smallest = targets
        .iter()
        .map(|t| t.w.min(t.h))
        .fold(f32::INFINITY, f32::min);
    if smallest.is_finite() {
        lines.push(format!(
            "smallest target: {smallest:.0} px (floor {:.0}, comfortable {:.0})",
            HARD_FLOOR_PX * ui.0,
            MIN_TARGET_PX * ui.0
        ));
    }
    lines.extend(hud::audit());
    lines.push(
        "open — judgement: whether this is legible to a person, layout at every window size, composite contrast on a rendered frame, and frame timing under load"
            .to_string(),
    );
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Targets built the way the client builds them: the comfortable floor
    /// scales with the interface, so a target has to grow with it too.
    fn scaled_targets(scale: f32) -> Vec<Target> {
        let ui = UiScale(scale);
        vec![
            Target {
                name: "tool button",
                w: 150.0 * scale,
                h: target(ui),
            },
            Target {
                name: "menu row",
                w: 220.0 * scale,
                h: target(ui),
            },
        ]
    }

    fn defects_at(scale: f32) -> Vec<String> {
        verify(&scaled_targets(scale), UiScale(scale))
    }

    #[test]
    fn the_shipped_scales_have_no_defects() {
        for scale in UI_SCALES {
            let defects = defects_at(scale);
            assert!(defects.is_empty(), "at {scale}: {defects:?}");
        }
    }

    #[test]
    fn the_check_fails_closed_on_an_undersized_target() {
        // The point of the check is that it can fail. A target below the hard
        // floor must be reported, not rounded away.
        let defects = verify(
            &[Target {
                name: "tiny",
                w: 12.0,
                h: 20.0,
            }],
            UiScale(1.0),
        );
        assert!(
            defects.iter().any(|d| d.contains("tiny")),
            "an undersized target was accepted: {defects:?}"
        );
    }

    #[test]
    fn a_shortfall_below_the_comfortable_floor_is_stated_not_hidden() {
        let defects = verify(
            &[Target {
                name: "slim",
                w: 30.0,
                h: 500.0,
            }],
            UiScale(1.0),
        );
        assert!(
            defects.iter().any(|d| d.contains("slim")),
            "a shortfall was silently allowed: {defects:?}"
        );
    }

    #[test]
    fn steps_are_strictly_increasing_and_whole() {
        let mut previous = 0.0;
        for step in Step::ALL {
            let px = step.logical_px();
            assert!(px > previous, "{} is not larger", step.name());
            assert_eq!(px.fract(), 0.0, "{} is fractional", step.name());
            previous = px;
        }
    }

    #[test]
    fn the_ui_scale_only_ever_enlarges_text() {
        let base = Step::Micro.px(UiScale(UI_SCALES[0]));
        for scale in UI_SCALES {
            assert!(
                Step::Micro.px(UiScale(scale)) >= base,
                "scaling to {scale} shrank the smallest text"
            );
        }
    }

    #[test]
    fn the_scale_cycles_and_returns_home() {
        let mut scale = UiScale::default();
        for _ in 0..UI_SCALES.len() {
            scale = scale.next();
        }
        assert_eq!(scale, UiScale::default(), "cycling did not return to 100%");
    }

    #[test]
    fn no_call_site_carries_its_own_size_constant() {
        // The type system already prevents a literal *reaching* a draw call.
        // What it cannot prevent is somebody reintroducing the old constants,
        // so the source is read back and checked for them.
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut checked = 0;
        let mut stack = vec![root.clone()];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).expect("a source directory") {
                let path = entry.expect("a directory entry").path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                    continue;
                }
                // This file names the forbidden constants in order to forbid
                // them, so it is the one file the scan cannot judge.
                if path.file_name().and_then(|n| n.to_str()) == Some("design.rs") {
                    continue;
                }
                let source = std::fs::read_to_string(&path).expect("a source file");
                checked += 1;
                for ad_hoc in ["SIZE_SMALL", "SIZE_BODY", "SIZE_DISPLAY"] {
                    assert!(
                        !source.contains(ad_hoc),
                        "{} still uses {ad_hoc}; sizes come from design::Step",
                        path.display()
                    );
                }
            }
        }
        assert!(checked > 4, "the scan found too few files to mean anything");
    }
}
