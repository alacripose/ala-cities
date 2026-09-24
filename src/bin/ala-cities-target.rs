//! Native target game client for the real founding-day world and the runtime
//! Dynamic Asset Generator chunk path. Asset authoring controls deliberately do
//! not appear in this player surface.

use std::io;
use std::sync::Arc;
use std::time::Instant;

use ala_cities::asset_generator::{
    chunk_builder_hash, ChunkAsset, DynamicAssetGenerator, MaterialKey,
};
use ala_cities::design::{Space, Step, UiScale};
use ala_cities::founding_day::{ChunkCoord, FoundingWorld, GeneratorRevision, WorldSeed};
use ala_cities::hud::{self, Token};
use ala_cities::raycast::VoxelHit;
use ala_cities::render::{
    Batcher, Camera, Gpu, ImageBatcher, Layer, Screen, Text, WorldBatch, LIGHT,
};
use ala_cities::ui::LINE_ADVANCE_FACTOR;

use glam::Vec3;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

const CHUNK: ChunkCoord = ChunkCoord::new(0, 0, 0);

struct TargetApp {
    gpu: Option<Gpu>,
    text: Text,
    hud_batch: Batcher,
    image_batch: ImageBatcher,
    world_batch: WorldBatch,
    camera: Camera,
    world: FoundingWorld,
    generator: DynamicAssetGenerator,
    asset: Option<ChunkAsset>,
    asset_error: Option<String>,
    world_digest: u64,
    cursor: (f32, f32),
    last_cursor: (f32, f32),
    orbiting: bool,
    panning: bool,
    hover_hit: Option<VoxelHit>,
    last_frame: Instant,
}

impl TargetApp {
    fn new() -> io::Result<Self> {
        let mut world = FoundingWorld::new(WorldSeed(7), GeneratorRevision(1));
        world.load_chunk(CHUNK);
        let world_digest = world.state_digest();
        let mut generator = DynamicAssetGenerator::default();
        let (asset, asset_error) = match generator.build_chunk(&world, CHUNK) {
            Ok(asset) => (Some(asset), None),
            Err(error) => (None, Some(error.to_string())),
        };
        let screen = Screen {
            w: 1280.0,
            h: 800.0,
        };
        let mut camera = Camera::new(screen, 32, 32);
        camera.focus = Vec3::new(16.0, 16.0, 1.5);
        camera.zoom = 24.0;
        let cursor = (screen.w / 2.0, screen.h / 2.0);
        let hover_hit = self_raycast(&camera, &asset, cursor);
        Ok(Self {
            gpu: None,
            text: Text::new(),
            hud_batch: Batcher::default(),
            image_batch: ImageBatcher::default(),
            world_batch: WorldBatch::default(),
            camera,
            world,
            generator,
            asset,
            asset_error,
            world_digest,
            cursor,
            last_cursor: cursor,
            orbiting: false,
            panning: false,
            hover_hit,
            last_frame: Instant::now(),
        })
    }

    fn update_cursor(&mut self, position: (f32, f32)) {
        self.cursor = position;
        let delta = (
            self.cursor.0 - self.last_cursor.0,
            self.cursor.1 - self.last_cursor.1,
        );
        if self.orbiting {
            self.camera.orbit(delta.0, delta.1);
        } else if self.panning {
            self.camera.pan(delta.0, delta.1);
        }
        self.last_cursor = self.cursor;
        self.hover_hit = self_raycast(&self.camera, &self.asset, self.cursor);
    }

    fn draw_asset(&mut self) {
        let Some(asset) = &self.asset else {
            return;
        };
        for quad in &asset.quads {
            let center = quad
                .vertices
                .map(|vertex| vertex.position)
                .into_iter()
                .sum::<Vec3>()
                / 4.0;
            let right = (quad.vertices[1].position - quad.vertices[0].position) * 0.5;
            let up = (quad.vertices[3].position - quad.vertices[0].position) * 0.5;
            let base = asset
                .material_manifest
                .get(MaterialKey::from(quad.material))
                .and_then(|material| material.opaque_rgba())
                .expect("generated chunk materials are renderable opaque materials");
            let lambert = quad.vertices[0].normal.dot(LIGHT.normalize()).max(0.0);
            let shade = 0.58 + 0.42 * lambert;
            self.world_batch.push(
                Layer::Opaque,
                center,
                right,
                up,
                [base[0] * shade, base[1] * shade, base[2] * shade, base[3]],
                Text::solid_uv(),
            );
        }
    }

    fn draw_panel(&mut self) {
        let screen = self.camera.screen;
        let ui = UiScale::default();
        let x = hud::space(Space::Md, ui);
        let y = hud::space(Space::Md, ui);
        let w = 430.0_f32.max(screen.w * 0.32);
        let h = 304.0_f32;
        hud::panel(&mut self.hud_batch, &screen, x, y, w, h, Token::PanelRaised);

        let content_x = x + hud::space(Space::Md, ui);
        let mut cursor_y = y + hud::space(Space::Md, ui);
        hud::label(
            &mut self.text,
            &mut self.hud_batch,
            &screen,
            content_x,
            cursor_y,
            Step::Title,
            Token::TextBody,
            "TARGET WORLD",
        );
        cursor_y += line_height(Step::Title) + hud::space(Space::Sm, ui);
        hud::label(
            &mut self.text,
            &mut self.hud_batch,
            &screen,
            content_x,
            cursor_y,
            Step::Small,
            Token::TextMuted,
            "dynamic asset generator · runtime chunk output",
        );
        cursor_y += line_height(Step::Small) + hud::space(Space::Sm, ui);
        hud::rule(
            &mut self.hud_batch,
            &screen,
            content_x,
            cursor_y,
            w - hud::space(Space::Xl, ui),
            Token::Grid,
        );
        cursor_y += hud::space(Space::Sm, ui);

        let builder_hash = chunk_builder_hash().to_string();
        hud::label_mono(
            &mut self.text,
            &mut self.hud_batch,
            &screen,
            content_x,
            cursor_y,
            Step::Small,
            Token::TextMuted,
            &format!("chunk builder  {builder_hash}"),
        );
        cursor_y += line_height(Step::Small) + hud::space(Space::Xs, ui);

        let material_status = self
            .asset
            .as_ref()
            .map(|asset| {
                let materials = asset.material_manifest.materials();
                let first = materials.first().expect("chunk manifest is non-empty");
                let classification = if materials
                    .iter()
                    .all(|material| material.opaque_rgba().is_some())
                {
                    "opaque"
                } else {
                    "mixed"
                };
                format!(
                    "{} materials · {classification} · alpha {:.3} · transmission {:.3}",
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
        cursor_y += line_height(Step::Small) + hud::space(Space::Sm, ui);

        let source_resident = self.world.chunk(CHUNK).is_some();
        let status = self
            .asset
            .as_ref()
            .map(|asset| {
                format!(
                    "{} visible quads · {} cached chunks · source {} · digest {:016x}",
                    asset.quads.len(),
                    self.generator.cache_len(),
                    if source_resident {
                        "resident"
                    } else {
                        "missing"
                    },
                    asset.digest
                )
            })
            .or_else(|| self.asset_error.clone())
            .unwrap_or_else(|| "chunk unavailable".to_string());
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
        cursor_y += line_height(Step::Small) + hud::space(Space::Sm, ui);

        for help in [
            "right-drag orbit · middle-drag pan · wheel zoom",
            "asset authoring controls live in the external builder",
        ] {
            hud::label(
                &mut self.text,
                &mut self.hud_batch,
                &screen,
                content_x,
                cursor_y,
                Step::Small,
                Token::TextMuted,
                help,
            );
            cursor_y += line_height(Step::Small) + hud::space(Space::Xs, ui);
        }

        let hit = self
            .hover_hit
            .map(|hit| {
                format!(
                    "ray hit ({},{},{}) face {:?} · {:.3}m",
                    hit.coord.x, hit.coord.y, hit.coord.z, hit.normal, hit.distance
                )
            })
            .unwrap_or_else(|| "ray hit: none".to_string());
        hud::label_mono(
            &mut self.text,
            &mut self.hud_batch,
            &screen,
            content_x,
            cursor_y,
            Step::Small,
            Token::TextMuted,
            &hit,
        );
        cursor_y += line_height(Step::Small) + hud::space(Space::Xs, ui);
        hud::label(
            &mut self.text,
            &mut self.hud_batch,
            &screen,
            content_x,
            cursor_y,
            Step::Small,
            Token::TextOnInk,
            &format!("WORLD TRUTH {:016x} · unchanged", self.world_digest),
        );
    }

    fn render(&mut self) {
        if self.gpu.is_none() {
            return;
        }
        self.world_batch.clear(self.camera.toward_camera());
        self.hud_batch.clear();
        self.image_batch.clear();
        self.draw_asset();
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
            [0.035, 0.047, 0.065, 1.0],
            seconds,
            &mut self.text,
        );
    }
}

fn self_raycast(
    camera: &Camera,
    asset: &Option<ChunkAsset>,
    cursor: (f32, f32),
) -> Option<VoxelHit> {
    let (near, far) = camera.ray(cursor.0, cursor.1);
    let ray = far - near;
    asset
        .as_ref()
        .and_then(|asset| asset.raycast(near, ray, ray.length() + 1.0))
}

fn line_height(step: Step) -> f32 {
    step.px(UiScale::default()) as f32 * LINE_ADVANCE_FACTOR
}

impl ApplicationHandler for TargetApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("ala-cities — target world")
                        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 800.0)),
                )
                .expect("target client window"),
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
                    .zoom_by_with_limits(factor, self.cursor, 4.0, 96.0);
            }
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
    let mut app = TargetApp::new()?;
    Ok(event_loop.run_app(&mut app)?)
}
