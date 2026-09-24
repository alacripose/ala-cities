use ala_cities::asset_controls::{
    nudge_control, orb_center, recipe_value_from_pointer, slider_hit_test, SliderRect,
};
use ala_cities::asset_generator::{control_bounds, AssetRecipe, ShapeControl};

fn rect() -> SliderRect {
    SliderRect {
        x: 100.0,
        y: 200.0,
        width: 300.0,
        height: 48.0,
    }
}

#[test]
fn every_semantic_control_has_its_own_quantized_range() {
    let recipe = AssetRecipe::default();

    assert_eq!(recipe.teeth, 9.0);
    assert!((recipe.opening - 0.30).abs() < 1e-6);
    assert!((recipe.accent - 0.50).abs() < 1e-6);

    for control in [
        ShapeControl::Teeth,
        ShapeControl::Opening,
        ShapeControl::Accent,
    ] {
        let (min, max) = control_bounds(control);
        assert!(min < max);
        assert!(recipe.value(control) >= min && recipe.value(control) <= max);
    }
}

#[test]
fn a_full_row_hit_selects_the_owning_slider() {
    let rect = rect();
    assert_eq!(
        slider_hit_test(&[rect], (rect.x + 20.0, rect.y + 24.0)),
        Some(ShapeControl::Teeth)
    );
}

#[test]
fn pointer_x_maps_to_the_exact_semantic_value_and_orb_position() {
    let rect = rect();
    let recipe = AssetRecipe::default();
    let (min, max) = control_bounds(ShapeControl::Opening);

    let pointer = (rect.x + rect.width * 0.25, rect.y + 24.0);
    let value = recipe_value_from_pointer(rect, pointer, recipe, ShapeControl::Opening)
        .expect("pointer maps to the opening recipe");
    let knob = orb_center(rect, recipe, ShapeControl::Opening);
    let round_trip = recipe_value_from_pointer(rect, knob, recipe, ShapeControl::Opening)
        .expect("knob maps back to a value");

    assert!((value - (min + (max - min) * 0.25)).abs() < 0.001);
    assert!((round_trip - recipe.opening).abs() < 0.001);
}

#[test]
fn keyboard_nudge_uses_control_specific_steps() {
    let mut recipe = AssetRecipe::default();

    assert!(nudge_control(&mut recipe, ShapeControl::Teeth, 1.0));
    assert!((recipe.teeth - 9.125).abs() < 0.001);
    assert!(nudge_control(&mut recipe, ShapeControl::Opening, -1.0));
    assert!((recipe.opening - 0.295).abs() < 0.001);
    assert!(nudge_control(&mut recipe, ShapeControl::Accent, 1.0));
    assert!((recipe.accent - 0.51).abs() < 0.001);
}
