//! Screen-space controls for the external Dynamic Asset Builder tool.
//!
//! Each semantic recipe field owns one conventional slider. The visible orb is
//! its tactile handle; pointer and keyboard input update the same quantized
//! `AssetRecipe` value.

use crate::asset_generator::{control_bounds, control_nudge_step, AssetRecipe, ShapeControl};

pub const SLIDER_HIT_TARGET_PX: f32 = 48.0;
pub const ORB_RADIUS_PX: f32 = 10.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl SliderRect {
    fn contains(self, pointer: (f32, f32)) -> bool {
        let half_height = self.height.max(SLIDER_HIT_TARGET_PX) * 0.5;
        let center_y = self.y + self.height * 0.5;
        pointer.0 >= self.x
            && pointer.0 <= self.x + self.width
            && (pointer.1 - center_y).abs() <= half_height
    }
}

pub fn slider_hit_test(rects: &[SliderRect], pointer: (f32, f32)) -> Option<ShapeControl> {
    ShapeControl::ALL
        .into_iter()
        .zip(rects.iter().copied())
        .find_map(|(control, rect)| rect.contains(pointer).then_some(control))
}

pub fn orb_center(rect: SliderRect, recipe: AssetRecipe, control: ShapeControl) -> (f32, f32) {
    let (min, max) = control_bounds(control);
    let fraction = ((recipe.value(control) - min) / (max - min)).clamp(0.0, 1.0);
    (rect.x + rect.width * fraction, rect.y + rect.height * 0.5)
}

fn slider_fraction(rect: SliderRect, pointer: (f32, f32)) -> Option<f32> {
    if rect.width <= 0.0 || !rect.contains(pointer) {
        return None;
    }
    Some(((pointer.0 - rect.x) / rect.width).clamp(0.0, 1.0))
}

pub fn recipe_value_from_pointer(
    rect: SliderRect,
    pointer: (f32, f32),
    _recipe: AssetRecipe,
    control: ShapeControl,
) -> Option<f32> {
    let fraction = slider_fraction(rect, pointer)?;
    let (min, max) = control_bounds(control);
    Some(min + fraction * (max - min))
}

pub fn nudge_control(recipe: &mut AssetRecipe, control: ShapeControl, direction: f32) -> bool {
    if !direction.is_finite() || direction == 0.0 {
        return false;
    }
    let direction = if direction > 0.0 { 1.0 } else { -1.0 };
    recipe.set(
        control,
        recipe.value(control) + control_nudge_step(control) * direction,
    )
}
