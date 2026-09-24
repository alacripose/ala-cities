//! External authoring/review tool for the Dynamic Asset Generator.
//!
//! This process is deliberately not player UI. It edits a semantic asset recipe,
//! derives a candidate hash, and previews the real generated mesh. The target
//! game consumes promoted/runtime presentation assets without these controls.

use std::io;
use std::sync::Arc;
use std::time::Instant;

use ala_cities::asset_controls::{
    nudge_control, orb_center, recipe_value_from_pointer, slider_hit_test, SliderRect,
    ORB_RADIUS_PX,
};
use ala_cities::asset_generator::{
    control_bounds, AssetKind, AssetRecipe, DynamicAssetGenerator, MaterialKey, ShapeAsset,
    ShapeControl,
};
use ala_cities::design::{target, Space, Step, UiScale};
use ala_cities::hud::{self, Token};
use ala_cities::render::{
    Batcher, Camera, Gpu, ImageBatcher, Screen, Text, WorldBatch, WorldVertexData, LIGHT,
};
use ala_cities::ui::LINE_ADVANCE_FACTOR;

use glam::Vec3;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

struct BuilderApp {
    gpu: Option<Gpu>,
    text: Text,
    hud_batch: Batcher,
    image_batch: ImageBatcher,
    world_batch: WorldBatch,
    camera: Camera,
    generator: DynamicAssetGenerator,
    asset: Option<ShapeAsset>,
    asset_error: Option<String>,
    cursor: (f32, f32),
    last_cursor: (f32, f32),
    selected: Option<ShapeControl>,
    dragging: Option<ShapeControl>,
    hovering: Option<ShapeControl>,
    orbiting: bool,
    panning: bool,
    last_frame: Instant,
}

impl BuilderApp {
    fn new() -> io::Result<Self> {
        let screen = Screen {
            w: 1280.0,
            h: 800.0,
        };
        let mut camera = Camera::new(screen, 32, 32);
        camera.focus = Vec3::ZERO;
        camera.yaw = 0.58;
        camera.pitch = 0.92;
        camera.zoom = 220.0;
        let cursor = (screen.w * 0.62, screen.h * 0.5);
        let mut generator = DynamicAssetGenerator::default();
        let (asset, asset_error) = match generator.build_shape(AssetKind::SettingsGear) {
            Ok(asset) => (Some(asset), None),
            Err(error) => (None, Some(error.to_string())),
        };
        Ok(Self {
            gpu: None,
            text: Text::new(),
            hud_batch: Batcher::default(),
            image_batch: ImageBatcher::default(),
            world_batch: WorldBatch::default(),
            camera,
            generator,
            asset,
            asset_error,
            cursor,
            last_cursor: cursor,
            selected: Some(ShapeControl::Teeth),
            dragging: None,
            hovering: None,
            orbiting: false,
            panning: false,
            last_frame: Instant::now(),
        })
    }

    fn rebuild(&mut self) {
        match self.generator.build_shape(AssetKind::SettingsGear) {
            Ok(asset) => {
                self.asset = Some(asset);
                self.asset_error = None;
            }
            Err(error) => self.asset_error = Some(error.to_string()),
        }
    }

    fn set_control(&mut self, control: ShapeControl, value: f32) {
        if self.generator.set_control(control, value) {
            self.rebuild();
        }
    }

    fn handle_key(&mut self, event: KeyEvent) {
        if event.state != ElementState::Pressed {
            return;
        }
        match event.physical_key {
            PhysicalKey::Code(KeyCode::Digit1) => self.selected = Some(ShapeControl::Teeth),
            PhysicalKey::Code(KeyCode::Digit2) => self.selected = Some(ShapeControl::Opening),
            PhysicalKey::Code(KeyCode::Digit3) => self.selected = Some(ShapeControl::Accent),
            PhysicalKey::Code(KeyCode::Escape) => self.selected = None,
            PhysicalKey::Code(KeyCode::KeyR) => {
                self.generator = DynamicAssetGenerator::new(AssetRecipe::default());
                self.rebuild();
            }
            PhysicalKey::Code(code) => {
                let Some(control) = self.selected else {
                    return;
                };
                let direction = match code {
                    KeyCode::ArrowLeft | KeyCode::ArrowDown => -1.0,
                    KeyCode::ArrowRight | KeyCode::ArrowUp => 1.0,
                    _ => return,
                };
                let mut recipe = self.generator.recipe();
                if nudge_control(&mut recipe, control, direction) {
                    self.set_control(control, recipe.value(control));
                }
            }
            _ => {}
        }
    }

    fn update_cursor(&mut self, position: (f32, f32)) {
        self.cursor = position;
        let delta = (
            self.cursor.0 - self.last_cursor.0,
            self.cursor.1 - self.last_cursor.1,
        );
        if let Some(control) = self.dragging {
            let rect = control_rect(self.camera.screen, control);
            if let Some(value) =
                recipe_value_from_pointer(rect, self.cursor, self.generator.recipe(), control)
            {
                self.set_control(control, value);
            }
        } else if self.orbiting {
            self.camera.orbit(delta.0, delta.1);
        } else if self.panning {
            self.camera.pan(delta.0, delta.1);
        } else {
            self.hovering = slider_hit_test(&control_rects(self.camera.screen), self.cursor);
        }
        self.last_cursor = self.cursor;
    }

    fn draw_shape(&mut self) {
        let Some(asset) = &self.asset else {
            return;
        };
        let vertices = asset
            .mesh
            .vertices
            .iter()
            .map(|vertex| {
                let material = vertex.material;
                let base = asset
                    .material_manifest
                    .get(MaterialKey::from(material))
                    .and_then(|material| material.opaque_rgba())
                    .expect("generated shape materials are renderable opaque materials");
                let lambert = vertex.normal.dot(LIGHT.normalize()).abs().max(0.0);
                let shade = 0.48 + 0.52 * lambert;
                WorldVertexData {
                    position: vertex.position.to_array(),
                    normal: vertex.normal.to_array(),
                    color: [base[0] * shade, base[1] * shade, base[2] * shade, base[3]],
                }
            })
            .collect::<Vec<_>>();
        self.world_batch
            .push_indexed_triangles(&vertices, &asset.mesh.indices);
    }

    fn draw_panel(&mut self) {
        let screen = self.camera.screen;
        let ui = UiScale::default();
        let x = hud::space(Space::Xl, ui);
        let y = hud::space(Space::Xl, ui);
        let w = 384.0_f32;
        let h = screen.h - y * 2.0;
        hud::panel(&mut self.hud_batch, &screen, x, y, w, h, Token::PanelRaised);

        let content_x = x + hud::space(Space::Lg, ui);
        let content_w = w - hud::space(Space::Lg, ui) * 2.0;
        let mut cursor_y = y + hud::space(Space::Lg, ui);
        hud::label(
            &mut self.text,
            &mut self.hud_batch,
            &screen,
            content_x,
            cursor_y,
            Step::Title,
            Token::TextBody,
            "ASSET BUILDER",
        );
        cursor_y += line_height(Step::Title) + hud::space(Space::Xs, ui);
        hud::label(
            &mut self.text,
            &mut self.hud_batch,
            &screen,
            content_x,
            cursor_y,
            Step::Small,
            Token::TextMuted,
            "external authoring · not player UI",
        );
        cursor_y += line_height(Step::Small) + hud::space(Space::Sm, ui);
        hud::rule(
            &mut self.hud_batch,
            &screen,
            content_x,
            cursor_y,
            content_w,
            Token::Grid,
        );
        cursor_y += hud::space(Space::Lg, ui);

        let recipe = self.generator.recipe();
        hud::label_mono(
            &mut self.text,
            &mut self.hud_batch,
            &screen,
            content_x,
            cursor_y,
            Step::Small,
            Token::Procedural,
            &format!("candidate  {}", self.generator.builder_hash()),
        );
        cursor_y += line_height(Step::Small) + hud::space(Space::Xs, ui);
        let material_status = self
            .asset
            .as_ref()
            .map(|asset| {
                let materials = asset.material_manifest.materials();
                let first = materials.first().expect("shape manifest is non-empty");
                let classification = if materials
                    .iter()
                    .all(|material| material.opaque_rgba().is_some())
                {
                    "opaque"
                } else {
                    "mixed"
                };
                format!(
                    "{} {classification} · alpha {:.3} · transmission {:.3}",
                    materials.len(),
                    first.alpha,
                    first.transmission_weight
                )
            })
            .unwrap_or_else(|| "materials unavailable".to_string());
        hud::label_mono(
            &mut self.text,
            &mut self.hud_batch,
            &screen,
            content_x,
            cursor_y,
            Step::Small,
            Token::TextMuted,
            &material_status,
        );
        cursor_y += line_height(Step::Small) + hud::space(Space::Xs, ui);
        hud::label(
            &mut self.text,
            &mut self.hud_batch,
            &screen,
            content_x,
            cursor_y,
            Step::Small,
            Token::TextMuted,
            "MD1 · TouchWiz · iOS 6 shape corpus",
        );
        cursor_y += line_height(Step::Small) + hud::space(Space::Lg, ui);

        for control in ShapeControl::ALL {
            let rect = control_rect(screen, control);
            let selected = self.selected == Some(control);
            let active = self.dragging == Some(control);
            let hovered = self.hovering == Some(control);
            if selected || active {
                self.hud_batch.screen_outline(
                    &screen,
                    rect.x - hud::space(Space::Sm, ui),
                    rect.y - hud::space(Space::Xs, ui),
                    rect.width + hud::space(Space::Md, ui),
                    rect.height + hud::space(Space::Sm, ui),
                    hud::style(Token::Procedural).text.expect("procedural ink"),
                );
            }

            hud::label(
                &mut self.text,
                &mut self.hud_batch,
                &screen,
                rect.x,
                rect.y - hud::space(Space::Sm, ui),
                Step::Small,
                if selected || hovered {
                    Token::TextBody
                } else {
                    Token::TextMuted
                },
                &format!("{}  ·  {}", control_index(control) + 1, control.name()),
            );
            let (min, max) = control_bounds(control);
            hud::label_mono(
                &mut self.text,
                &mut self.hud_batch,
                &screen,
                rect.x + rect.width - 188.0,
                rect.y - hud::space(Space::Sm, ui),
                Step::Small,
                Token::TextMuted,
                &format!("{:>7.3}  [{:.3}, {:.3}]", recipe.value(control), min, max),
            );
            self.draw_slider(rect, control, selected || active);
            cursor_y = rect.y + rect.height + hud::space(Space::Lg, ui);
        }

        cursor_y += hud::space(Space::Sm, ui);
        hud::rule(
            &mut self.hud_batch,
            &screen,
            content_x,
            cursor_y,
            content_w,
            Token::Grid,
        );
        cursor_y += hud::space(Space::Sm, ui);
        let description = match self.selected {
            Some(ShapeControl::Teeth) => "profile morph · continuous tooth count",
            Some(ShapeControl::Opening) => "center opening · ratio to root radius",
            Some(ShapeControl::Accent) => "construction · authored corpus states",
            None => "select a control",
        };
        hud::label(
            &mut self.text,
            &mut self.hud_batch,
            &screen,
            content_x,
            cursor_y,
            Step::Small,
            Token::TextMuted,
            description,
        );
        cursor_y += line_height(Step::Small) + hud::space(Space::Sm, ui);

        if let Some(asset) = &self.asset {
            let mesh = &asset.mesh;
            let status = format!(
                "one mesh · {} triangles · {} verts",
                mesh.triangle_count(),
                mesh.vertex_count()
            );
            hud::label_mono(
                &mut self.text,
                &mut self.hud_batch,
                &screen,
                content_x,
                cursor_y,
                Step::Small,
                Token::TextBody,
                &status,
            );
            cursor_y += line_height(Step::Small) + hud::space(Space::Xs, ui);
            hud::label_mono(
                &mut self.text,
                &mut self.hud_batch,
                &screen,
                content_x,
                cursor_y,
                Step::Small,
                Token::TextMuted,
                &format!("digest  {:016x}", asset.digest),
            );
            cursor_y += line_height(Step::Small) + hud::space(Space::Xs, ui);
        } else if let Some(error) = &self.asset_error {
            hud::label(
                &mut self.text,
                &mut self.hud_batch,
                &screen,
                content_x,
                cursor_y,
                Step::Small,
                Token::Warning,
                error,
            );
            cursor_y += line_height(Step::Small) + hud::space(Space::Xs, ui);
        }

        hud::label(
            &mut self.text,
            &mut self.hud_batch,
            &screen,
            content_x,
            cursor_y,
            Step::Small,
            Token::TextOnInk,
            "CANDIDATE · HUMAN PROMOTION REQUIRED",
        );
    }

    fn draw_slider(&mut self, rect: SliderRect, control: ShapeControl, active: bool) {
        let screen = self.camera.screen;
        let center_y = rect.y + rect.height * 0.5;
        let track = hud::style(Token::Grid)
            .fill
            .or(hud::style(Token::Grid).text)
            .expect("grid colour");
        let accent = hud::style(Token::Procedural)
            .fill
            .or(hud::style(Token::Procedural).text)
            .expect("procedural colour");
        self.hud_batch.screen_rect(
            &screen,
            rect.x,
            center_y - 2.0,
            rect.width,
            4.0,
            track,
            Text::solid_uv(),
        );
        let knob = orb_center(rect, self.generator.recipe(), control);
        if knob.0 > rect.x {
            self.hud_batch.screen_rect(
                &screen,
                rect.x,
                center_y - 2.0,
                knob.0 - rect.x,
                4.0,
                accent,
                Text::solid_uv(),
            );
        }
        let radius = if active {
            ORB_RADIUS_PX + 2.0
        } else {
            ORB_RADIUS_PX
        };
        draw_screen_orb(
            &mut self.hud_batch,
            &screen,
            knob,
            radius,
            if active { accent } else { track },
        );
        draw_screen_orb(
            &mut self.hud_batch,
            &screen,
            knob,
            radius * 0.38,
            hud::style(Token::PanelRaised).fill.expect("panel fill"),
        );
    }

    fn render(&mut self) {
        if self.gpu.is_none() {
            return;
        }
        self.world_batch.clear(self.camera.toward_camera());
        self.hud_batch.clear();
        self.image_batch.clear();
        self.draw_shape();
        self.draw_panel();
        let now = Instant::now();
        let seconds = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;
        let gpu = self.gpu.as_mut().expect("GPU was checked before drawing");
        gpu.render(
            &self.world_batch,
            &self.hud_batch,
            &self.image_batch,
            &self.camera,
            [0.025, 0.034, 0.048, 1.0],
            seconds,
            &mut self.text,
        );
    }
}

fn control_rects(screen: Screen) -> [SliderRect; 3] {
    [
        control_rect(screen, ShapeControl::Teeth),
        control_rect(screen, ShapeControl::Opening),
        control_rect(screen, ShapeControl::Accent),
    ]
}

fn control_rect(_screen: Screen, control: ShapeControl) -> SliderRect {
    let ui = UiScale::default();
    let x = hud::space(Space::Xl, ui) + hud::space(Space::Lg, ui);
    let width = 384.0 - hud::space(Space::Lg, ui) * 2.0;
    let first_y = 236.0;
    let row_gap = target(ui) + hud::space(Space::Lg, ui);
    SliderRect {
        x,
        y: first_y + control_index(control) as f32 * row_gap,
        width,
        height: target(ui),
    }
}

fn control_index(control: ShapeControl) -> usize {
    ShapeControl::ALL
        .iter()
        .position(|candidate| *candidate == control)
        .expect("shape control is in the closed list")
}

fn draw_screen_orb(
    batch: &mut Batcher,
    screen: &Screen,
    center: (f32, f32),
    radius: f32,
    color: [f32; 4],
) {
    let integer_radius = radius.round() as i32;
    for y in -integer_radius..=integer_radius {
        let half_width = ((radius * radius - (y * y) as f32).max(0.0).sqrt()).round();
        if half_width <= 0.0 {
            continue;
        }
        batch.screen_rect(
            screen,
            center.0 - half_width,
            center.1 + y as f32 - 0.5,
            half_width * 2.0,
            1.0,
            color,
            Text::solid_uv(),
        );
    }
}

fn line_height(step: Step) -> f32 {
    step.px(UiScale::default()) as f32 * LINE_ADVANCE_FACTOR
}

impl ApplicationHandler for BuilderApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("ala-cities — Dynamic Asset Builder")
                        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 800.0)),
                )
                .expect("asset-builder window"),
        );
        let gpu = Gpu::new(window, &self.text);
        self.camera.screen = gpu.screen();
        self.gpu = Some(gpu);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(gpu) = self.gpu.as_mut() {
                    gpu.resize(size.width, size.height);
                    self.camera.screen = gpu.screen();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.update_cursor((position.x as f32, position.y as f32));
            }
            WindowEvent::MouseInput { state, button, .. } => match (button, state) {
                (MouseButton::Left, ElementState::Pressed) => {
                    let hit = slider_hit_test(&control_rects(self.camera.screen), self.cursor);
                    self.selected = hit;
                    self.dragging = hit;
                }
                (MouseButton::Left, ElementState::Released) => {
                    self.dragging = None;
                    self.last_cursor = self.cursor;
                }
                (MouseButton::Right, ElementState::Pressed) => {
                    self.orbiting = true;
                    self.last_cursor = self.cursor;
                }
                (MouseButton::Right, ElementState::Released) => self.orbiting = false,
                (MouseButton::Middle, ElementState::Pressed) => {
                    self.panning = true;
                    self.last_cursor = self.cursor;
                }
                (MouseButton::Middle, ElementState::Released) => self.panning = false,
                _ => {}
            },
            WindowEvent::MouseWheel { delta, .. } => {
                let factor = match delta {
                    MouseScrollDelta::LineDelta(_, lines) => 1.0_f32.powf(lines * 0.12),
                    MouseScrollDelta::PixelDelta(position) => {
                        1.0_f32.powf(-position.y as f32 * 0.002)
                    }
                };
                self.camera
                    .zoom_by_with_limits(factor, self.cursor, 120.0, 520.0);
            }
            WindowEvent::KeyboardInput { event, .. } => self.handle_key(event),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.render();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = BuilderApp::new()?;
    Ok(event_loop.run_app(&mut app)?)
}
