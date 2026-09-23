//! Rendering.
//!
//! Two passes, one shader family. The **world** is instanced quads in world
//! space with a real depth buffer and a free orbit camera; the **interface** is
//! instanced quads already flattened to clip space, so the HUD is exact at any
//! camera angle and never moves with the camera.
//!
//! Solid geometry uses a dedicated opaque-white texel; screen-space text is
//! shaped and rendered by Glyphon. World quads still carry a position, a right
//! vector and an up vector rather than a screen rectangle, which lets a building
//! be a box and a camera be free.
//!
//! The camera is **orthographic**, deliberately: a city builder is read at a
//! consistent tile size, and orthographic projection keeps two tiles the same
//! size wherever they are on screen.

pub use crate::text::Text;
use glyphon::{Cache as GlyphonCache, TextAtlas, TextRenderer, Viewport};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3, Vec4};
use wgpu::util::DeviceExt;


pub const TILE: f32 = 12.0;
/// Image-atlas sizing constant used by the picker.
pub const ATLAS_SIZE: u32 = 1024;

/// Height of one building level, in world units. One level is a visible step at
/// the default zoom rather than a token amount of extrusion.
pub const LEVEL_HEIGHT: f32 = 7.0;

/// The light, fixed in world space. One direction used for both face shading
/// and the offset of the contact shadows, so a shadow cannot disagree with the
/// face it belongs to.
pub const LIGHT: Vec3 = Vec3::new(0.42, 0.50, 0.76);

/// Which pass a world quad belongs to. Opaque geometry writes depth; overlays
/// test against it and do not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    Opaque,
    Overlay,
}

// ---------------------------------------------------------------------------
// Text
// ---------------------------------------------------------------------------

/// Which face to shape with. Monospace keeps numeric columns stable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Face {
    Body,
    Mono,
}

/// One solid UI quad in clip space. Text is no longer represented here;
/// Glyphon owns glyph vertices and atlas packing.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Instance {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub uv: [f32; 4],
}

/// One world-space quad instance.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct WorldInstance {
    pub center: [f32; 3],
    pub right: [f32; 3],
    pub up: [f32; 3],
    pub color: [f32; 4],
    pub uv: [f32; 4],
}



// ---------------------------------------------------------------------------
// Batchers
// ---------------------------------------------------------------------------

/// The CPU-side interface batcher. Quads are already in clip space by the time
/// they get here, which is what keeps the HUD independent of the camera.
#[derive(Default)]
pub struct Batcher {
    pub instances: Vec<Instance>,
}

impl Batcher {
    pub fn clear(&mut self) {
        self.instances.clear();
    }

    pub fn clip_rect(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: [f32; 4],
        uv: [f32; 4],
    ) {
        self.instances.push(Instance {
            pos: [x, y],
            size: [w, h],
            color,
            uv,
        });
    }

    /// A rectangle in screen pixels, top-left origin, snapped to whole pixels
    /// so panel edges do not sit half a pixel off the grid.
    #[allow(clippy::too_many_arguments)]
    pub fn screen_rect(
        &mut self,
        screen: &Screen,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: [f32; 4],
        uv: [f32; 4],
    ) {
        let (cx, cy) = screen.to_clip(x.round(), y.round());
        let (cw, ch) = screen.size_to_clip(w.round().max(1.0), h.round().max(1.0));
        self.clip_rect(cx, cy, cw, ch, color, uv);
    }

    /// A 1px outline, drawn as four thin rectangles. Cheaper and simpler than a
    /// second pipeline, and at 240 Hz the extra quads are free.
    pub fn screen_outline(
        &mut self,
        screen: &Screen,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: [f32; 4],
    ) {
        let uv = Text::solid_uv();
        self.screen_rect(screen, x, y, w, 1.0, color, uv);
        self.screen_rect(screen, x, y + h - 1.0, w, 1.0, color, uv);
        self.screen_rect(screen, x, y, 1.0, h, color, uv);
        self.screen_rect(screen, x + w - 1.0, y, 1.0, h, color, uv);
    }
}

/// The world batcher. Two lists, because the passes differ in depth behaviour
/// rather than in geometry.
#[derive(Default)]
pub struct WorldBatch {
    pub opaque: Vec<WorldInstance>,
    pub overlay: Vec<WorldInstance>,
    /// Direction from the scene toward the camera, so a box can skip the faces
    /// pointing away from it.
    cull: Vec3,
}

impl WorldBatch {
    pub fn clear(&mut self, toward_camera: Vec3) {
        self.opaque.clear();
        self.overlay.clear();
        self.cull = toward_camera;
    }

    pub fn count(&self) -> usize {
        self.opaque.len() + self.overlay.len()
    }

    pub fn push(
        &mut self,
        layer: Layer,
        center: Vec3,
        right: Vec3,
        up: Vec3,
        color: [f32; 4],
        uv: [f32; 4],
    ) {
        let instance = WorldInstance {
            center: center.to_array(),
            right: right.to_array(),
            up: up.to_array(),
            color,
            uv,
        };
        match layer {
            Layer::Opaque => self.opaque.push(instance),
            Layer::Overlay => self.overlay.push(instance),
        }
    }

    /// A quad lying flat on the ground plane at height `z`.
    #[allow(clippy::too_many_arguments)]
    pub fn ground_rect(
        &mut self,
        layer: Layer,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        z: f32,
        color: [f32; 4],
    ) {
        self.push(
            layer,
            Vec3::new(x + w / 2.0, y + h / 2.0, z),
            Vec3::new(w / 2.0, 0.0, 0.0),
            Vec3::new(0.0, h / 2.0, 0.0),
            color,
            SOLID_UV,
        );
    }

    /// A flat border of the given thickness, drawn inside the rectangle's
    /// edges. Four thin quads, because there is no line primitive.
    #[allow(clippy::too_many_arguments)]
    pub fn ground_border(
        &mut self,
        layer: Layer,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        z: f32,
        thickness: f32,
        color: [f32; 4],
    ) {
        self.ground_rect(layer, x, y, w, thickness, z, color);
        self.ground_rect(layer, x, y + h - thickness, w, thickness, z, color);
        self.ground_rect(layer, x, y, thickness, h, z, color);
        self.ground_rect(layer, x + w - thickness, y, thickness, h, z, color);
    }

    /// An axis-aligned box: four walls and a roof, with the faces pointing away
    /// from the camera skipped and every face shaded by the fixed light. The
    /// floor is not drawn because nothing can see it.
    #[allow(clippy::too_many_arguments)]
    pub fn building(
        &mut self,
        center_x: f32,
        center_y: f32,
        half: f32,
        height: f32,
        base: [f32; 4],
        alpha: f32,
    ) {
        let z = height / 2.0;
        let roof = shade(base, Vec3::Z, alpha);
        let walls = [
            // normal, centre offset, right extent, up extent
            (Vec3::Y, Vec3::new(0.0, half, 0.0), Vec3::X * half, Vec3::Z * z),
            (Vec3::NEG_Y, Vec3::new(0.0, -half, 0.0), Vec3::X * half, Vec3::Z * z),
            (Vec3::X, Vec3::new(half, 0.0, 0.0), Vec3::Y * half, Vec3::Z * z),
            (Vec3::NEG_X, Vec3::new(-half, 0.0, 0.0), Vec3::Y * half, Vec3::Z * z),
        ];

        // The roof is always visible from above, and that is the only
        // direction this camera can be.
        self.push(
            Layer::Opaque,
            Vec3::new(center_x, center_y, height),
            Vec3::X * half,
            Vec3::Y * half,
            roof,
            SOLID_UV,
        );

        for (normal, offset, right, up) in walls {
            if normal.dot(self.cull) <= 0.01 {
                continue;
            }
            self.push(
                Layer::Opaque,
                Vec3::new(center_x, center_y, z) + offset,
                right,
                up,
                shade(base, normal, alpha),
                SOLID_UV,
            );
        }
    }
}

/// Flat shading from the fixed light. A face turned away from the light keeps a
/// floor of its own colour so nothing goes black.
fn shade(base: [f32; 4], normal: Vec3, alpha: f32) -> [f32; 4] {
    let lambert = normal.dot(LIGHT.normalize()).max(0.0);
    let amount = 0.55 + 0.45 * lambert;
    [
        base[0] * amount,
        base[1] * amount,
        base[2] * amount,
        base[3] * alpha,
    ]
}

// ---------------------------------------------------------------------------
// Screen
// ---------------------------------------------------------------------------

/// Screen geometry and the identity transform. The HUD is in pixels and never
/// moves with the camera.
#[derive(Clone, Copy, Debug)]
pub struct Screen {
    pub w: f32,
    pub h: f32,
}

impl Screen {
    pub fn to_clip(self, x: f32, y: f32) -> (f32, f32) {
        ((x / self.w) * 2.0 - 1.0, 1.0 - (y / self.h) * 2.0)
    }

    pub fn size_to_clip(&self, w: f32, h: f32) -> (f32, f32) {
        ((w / self.w) * 2.0, -(h / self.h) * 2.0)
    }

    pub fn bottom_anchor(&self, height: f32, margin: f32) -> f32 {
        self.h - height - margin
    }

    /// Screen pixels to normalised device coordinates, depth ignored.
    pub fn to_ndc(self, x: f32, y: f32) -> Vec2 {
        Vec2::new((x / self.w) * 2.0 - 1.0, 1.0 - (y / self.h) * 2.0)
    }
}

// ---------------------------------------------------------------------------
// Camera
// ---------------------------------------------------------------------------

/// How far the camera may tilt. Below the minimum the ground is nearly edge-on
/// and clicks stop resolving to tiles; at the maximum the yaw stops meaning
/// anything.
pub const MIN_PITCH: f32 = 20.0_f32.to_radians();
pub const MAX_PITCH: f32 = 88.0_f32.to_radians();
pub const DEFAULT_PITCH: f32 = 52.0_f32.to_radians();

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    /// The point on the ground the camera orbits.
    pub focus: Vec3,
    /// Free rotation about the vertical axis. Wraps.
    pub yaw: f32,
    /// Free tilt above the horizon, clamped to keep the ground readable.
    pub pitch: f32,
    /// Screen pixels per world unit at the focus plane.
    pub zoom: f32,
    pub screen: Screen,
}

impl Camera {
    pub fn new(screen: Screen, map_width: u32, map_height: u32) -> Self {
        let fit = screen.h / (map_height as f32 * TILE);
        Self {
            focus: Vec3::new(
                (map_width as f32 * TILE) / 2.0,
                (map_height as f32 * TILE) / 2.0,
                0.0,
            ),
            yaw: 0.0,
            pitch: DEFAULT_PITCH,
            zoom: (fit * 2.4).max(0.2),
            screen,
        }
    }

    /// The direction the camera looks, from `yaw` and `pitch`.
    pub fn forward(&self) -> Vec3 {
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        Vec3::new(
            -sin_yaw * cos_pitch,
            -cos_yaw * cos_pitch,
            -sin_pitch,
        )
    }

    /// The camera's up, in world space.
    pub fn up(&self) -> Vec3 {
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        Vec3::new(sin_yaw * sin_pitch, cos_yaw * sin_pitch, -cos_pitch)
    }

    pub fn right(&self) -> Vec3 {
        self.forward().cross(self.up()).normalize()
    }

    /// The vector pointing from the scene back toward the camera, used for
    /// face culling.
    pub fn toward_camera(&self) -> Vec3 {
        -self.forward()
    }

    /// World → clip. Orthographic, so this is a rotation and a scale.
    pub fn view_proj(&self) -> Mat4 {
        let half_h = self.screen.h / 2.0 / self.zoom;
        let half_w = self.screen.w / 2.0 / self.zoom;
        // A symmetric depth range: the city is shallow next to the depth
        // precision of a 32-bit buffer, and a symmetric range cannot clip a
        // building that leans toward the camera.
        let projection = Mat4::orthographic_rh(-half_w, half_w, -half_h, half_h, -4096.0, 4096.0);
        projection * self.view()
    }

    pub fn view(&self) -> Mat4 {
        let r = self.right();
        let u = self.up();
        let f = self.forward();
        // Rows of the rotation are (right, up, -forward): the classic look-at
        // basis, built as columns because that is how a matrix is assembled.
        let rotation = Mat4::from_cols(
            Vec4::new(r.x, u.x, -f.x, 0.0),
            Vec4::new(r.y, u.y, -f.y, 0.0),
            Vec4::new(r.z, u.z, -f.z, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        );
        rotation * Mat4::from_translation(-self.focus)
    }

    /// World point → screen pixels.
    pub fn world_to_screen(&self, p: Vec3) -> (f32, f32) {
        let clip = self.view_proj() * p.extend(1.0);
        let w = if clip.w.abs() < f32::EPSILON {
            1.0
        } else {
            clip.w
        };
        (
            (clip.x / w * 0.5 + 0.5) * self.screen.w,
            (0.5 - clip.y / w * 0.5) * self.screen.h,
        )
    }

    /// The exact cursor ray in world space.
    ///
    /// Two points are unprojected and the ray between them returned, so this
    /// holds at any yaw, any pitch and any zoom. Deriving a tile from a screen
    /// rectangle instead would be a tile or two wrong the moment the camera
    /// tilts, and would read as "the game is broken" rather than "the picking
    /// is approximate".
    pub fn ray(&self, sx: f32, sy: f32) -> (Vec3, Vec3) {
        let inverse = self.view_proj().inverse();
        let ndc = self.screen.to_ndc(sx, sy);
        let near = inverse * Vec4::new(ndc.x, ndc.y, 0.0, 1.0);
        let far = inverse * Vec4::new(ndc.x, ndc.y, 1.0, 1.0);
        let near = near.truncate() / near.w;
        let far = far.truncate() / far.w;
        (near, far)
    }

    /// Where a screen point lands on the ground plane, if it lands on it at all.
    ///
    /// A ray pointing above the horizon never meets the ground, and returning
    /// `None` there is the honest answer — the cursor is off the world.
    pub fn pick_ground(&self, sx: f32, sy: f32) -> Option<Vec2> {
        let (near, far) = self.ray(sx, sy);
        let direction = far - near;
        if direction.z.abs() < 1e-6 {
            return None;
        }
        let t = -near.z / direction.z;
        if t < 0.0 {
            return None;
        }
        let hit = near + direction * t;
        Some(Vec2::new(hit.x, hit.y))
    }

    /// The tile a screen point is over.
    pub fn screen_to_tile(&self, sx: f32, sy: f32) -> Option<(i32, i32)> {
        let ground = self.pick_ground(sx, sy)?;
        Some(((ground.x / TILE).floor() as i32, (ground.y / TILE).floor() as i32))
    }

    /// Which tiles are on screen, so the client can skip the other 60,000.
    ///
    /// Computed from the four screen corners projected onto the ground. When a
    /// corner misses the ground (the view is tilted enough to see the horizon)
    /// the widest reading is used, because drawing too much is a performance
    /// question and drawing too little is a hole in the city.
    pub fn visible_tiles(&self, map_width: u32, map_height: u32) -> (u32, u32, u32, u32) {
        let corners = [
            (0.0, 0.0),
            (self.screen.w, 0.0),
            (0.0, self.screen.h),
            (self.screen.w, self.screen.h),
            (self.screen.w / 2.0, self.screen.h / 2.0),
        ];
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);
        let mut hits = 0;
        for (sx, sy) in corners {
            if let Some(ground) = self.pick_ground(sx, sy) {
                min = min.min(ground);
                max = max.max(ground);
                hits += 1;
            }
        }
        if hits == 0 {
            // Nothing on screen is ground: fall back to a window around the
            // focus rather than to nothing.
            let centre = Vec2::new(self.focus.x, self.focus.y);
            let span = 64.0 * TILE;
            min = centre - span;
            max = centre + span;
        }
        let clamp = |v: f32, limit: u32| v.clamp(0.0, limit as f32) as u32;
        (
            clamp(min.x / TILE - 1.0, map_width),
            clamp(min.y / TILE - 1.0, map_height),
            clamp(max.x / TILE + 1.0, map_width),
            clamp(max.y / TILE + 1.0, map_height),
        )
    }

    pub fn clamp_to(&mut self, map_width: u32, map_height: u32) {
        self.zoom = self.zoom.clamp(0.15, 8.0);
        self.pitch = self.pitch.clamp(MIN_PITCH, MAX_PITCH);
        let margin = 200.0;
        self.focus.x = self
            .focus
            .x
            .clamp(-margin, map_width as f32 * TILE + margin);
        self.focus.y = self
            .focus
            .y
            .clamp(-margin, map_height as f32 * TILE + margin);
        self.focus.z = 0.0;
        // Keep the angle in one turn so it never grows without bound.
        let turn = std::f32::consts::TAU;
        self.yaw = self.yaw.rem_euclid(turn);
    }

    /// Pan across the ground plane, in screen pixels of drag. The pan follows
    /// the screen axes, so a drag moves the city with the cursor rather than
    /// with the world's own axes.
    pub fn pan(&mut self, dx_screen: f32, dy_screen: f32) {
        let u = self.up();
        let r = self.right();
        // Only the ground-plane component moves the focus; otherwise panning
        // would lift or sink the camera.
        let rx = Vec2::new(r.x, r.y);
        let ux = Vec2::new(u.x, u.y);
        let lift = Vec2::new(u.z, u.z);
        // Screen-up on the ground is shorter than screen-up in the plane by the
        // tilt, and dividing by it keeps the drag tracking the cursor.
        let ground_up = if lift.x.abs() < 1e-3 { 1.0 } else { 1.0 - lift.x.abs() };
        self.focus.x -= (dx_screen * rx.x + dy_screen * ux.x / ground_up) / self.zoom;
        self.focus.y -= (dx_screen * rx.y + dy_screen * ux.y / ground_up) / self.zoom;
    }

    /// Orbit: free rotation and free tilt, both driven by a drag.
    pub fn orbit(&mut self, dx_screen: f32, dy_screen: f32) {
        self.yaw -= dx_screen * 0.006;
        self.pitch = (self.pitch + dy_screen * 0.005).clamp(MIN_PITCH, MAX_PITCH);
    }

    pub fn zoom_by(&mut self, factor: f32, at_screen: (f32, f32)) {
        // Zoom toward the cursor: the ground point under it stays under it.
        let before = self.pick_ground(at_screen.0, at_screen.1);
        self.zoom = (self.zoom * factor).clamp(0.15, 8.0);
        if let (Some(before), Some(after)) = (before, self.pick_ground(at_screen.0, at_screen.1)) {
            self.focus.x += before.x - after.x;
            self.focus.y += before.y - after.y;
        }
    }

    /// A compass bearing: the world direction the **top of the screen** faces,
    /// in degrees clockwise from north, where north is +Y.
    ///
    /// This is deliberately the screen's up rather than the camera's view
    /// direction, because that is the question the corner indicator answers —
    /// panning the city north on screen is a different act from looking along
    /// north, and only one of them is what a player checks.
    pub fn bearing_degrees(&self) -> f32 {
        self.yaw.to_degrees().rem_euclid(360.0)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FrameStats {
    pub frames: u64,
    pub last_ms: f32,
    pub average_ms: f32,
    pub worst_ms: f32,
    pub reported_fps: f32,
}

impl FrameStats {
    pub fn observe(&mut self, seconds: f32, refresh_hz: f32) {
        self.frames += 1;
        self.last_ms = seconds * 1000.0;
        // A rolling average, deliberately including the worst frame: a mean
        // that hides a stall is the wrong statistic for a real-time budget.
        let alpha = 0.02;
        self.average_ms = if self.frames == 1 {
            self.last_ms
        } else {
            self.average_ms * (1.0 - alpha) + self.last_ms * alpha
        };
        self.worst_ms = self.worst_ms.max(self.last_ms);
        self.reported_fps = self
            .average_ms
            .clamp(0.0, 1_000.0)
            .recip()
            .min(refresh_hz * 4.0);
    }
}

// ---------------------------------------------------------------------------
// GPU
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [f32; 16],
}

pub struct Gpu {
    pub window: std::sync::Arc<winit::window::Window>,
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    /// Interface pipeline: clip-space quads, depth always passes.
    pub pipeline: wgpu::RenderPipeline,
    /// World pipelines: world-space quads, depth-tested.
    pub world_opaque: wgpu::RenderPipeline,
    pub world_overlay: wgpu::RenderPipeline,
    /// Image pipeline: textured RGBA quads, screen space, depth always passes.
    pub image_pipeline: wgpu::RenderPipeline,
    pub image_bind_group: wgpu::BindGroup,
    image_texture: wgpu::Texture,
    image_sampler: wgpu::Sampler,
    image_size_buffer: wgpu::Buffer,
    images: wgpu::Buffer,
    image_capacity: usize,
    pub bind_group: wgpu::BindGroup,
    pub world_bind_group: wgpu::BindGroup,
    pub camera_buffer: wgpu::Buffer,
    depth: wgpu::Texture,
    depth_view: wgpu::TextureView,
    /// A tiny opaque-white texture used only by solid geometry. Glyphon owns
    /// the glyph atlas now.
    pub atlas_texture: wgpu::Texture,
    pub text_atlas: TextAtlas,
    pub text_renderer: TextRenderer,
    pub text_viewport: Viewport,
    instances: wgpu::Buffer,
    instance_capacity: usize,
    world_opaque_buffer: wgpu::Buffer,
    world_opaque_capacity: usize,
    world_overlay_buffer: wgpu::Buffer,
    world_overlay_capacity: usize,
    pub present_modes: Vec<wgpu::PresentMode>,
    pub adapter_name: String,
    pub stats: FrameStats,
    /// The last camera the frame was drawn with, for the picking check the
    /// client runs at startup.
    pub last_camera: Option<Camera>,
}

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
const SOLID_UV: [f32; 4] = [0.5, 0.5, 0.5, 0.5];

const SHADER: &str = r#"
struct Instance {
    @location(0) pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv: vec4<f32>,
};

struct VertexOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) index: u32, instance: Instance) -> VertexOut {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0)
    );
    let corner = corners[index];
    var out: VertexOut;
    out.clip = vec4<f32>(instance.pos + corner * instance.size, 0.0, 1.0);
    out.uv = mix(instance.uv.xy, instance.uv.zw, corner);
    out.color = instance.color;
    return out;
}

@group(0) @binding(0) var atlas_texture: texture_2d<f32>;
@group(0) @binding(1) var atlas_sampler: sampler;

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let coverage = textureSample(atlas_texture, atlas_sampler, in.uv).r;
    return vec4<f32>(in.color.rgb, in.color.a * coverage);
}
"#;

/// The world shader. A quad is a centre plus two half-extents, so a wall, a roof
/// and a ground tile are the same primitive and there are no vertices to upload.
const WORLD_SHADER: &str = r#"
struct Camera {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var atlas_texture: texture_2d<f32>;
@group(0) @binding(2) var atlas_sampler: sampler;

struct Instance {
    @location(0) center: vec3<f32>,
    @location(1) right: vec3<f32>,
    @location(2) up: vec3<f32>,
    @location(3) color: vec4<f32>,
    @location(4) uv: vec4<f32>,
};

struct VertexOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_world(@builtin(vertex_index) index: u32, instance: Instance) -> VertexOut {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0)
    );
    let corner = corners[index];
    let world = instance.center + instance.right * corner.x + instance.up * corner.y;
    var out: VertexOut;
    out.clip = camera.view_proj * vec4<f32>(world, 1.0);
    out.uv = mix(instance.uv.xy, instance.uv.zw, corner * 0.5 + vec2<f32>(0.5, 0.5));
    out.color = instance.color;
    return out;
}

@fragment
fn fs_world(in: VertexOut) -> @location(0) vec4<f32> {
    let coverage = textureSample(atlas_texture, atlas_sampler, in.uv).r;
    return vec4<f32>(in.color.rgb, in.color.a * coverage);
}
"#;

/// The image shader. An instance is a screen rect and a source rect in *image
/// pixels*, with the source image's size in a uniform; the fragment samples
/// `src / image_size`. The atlas stays GPU-side: no CPU-side packing, no CPU
/// coordinates to disagree with the shader's.
const IMAGE_SHADER: &str = r#"
struct ImageSize {
    size: vec2<f32>,
};

@group(0) @binding(0) var image_texture: texture_2d<f32>;
@group(0) @binding(1) var image_sampler: sampler;
@group(0) @binding(2) var<uniform> image_size: ImageSize;

struct ImageInstance {
    @location(0) pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) src: vec4<f32>,
    @location(3) tint: vec4<f32>,
};

struct ImageOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) tint: vec4<f32>,
};

@vertex
fn vs_image(@builtin(vertex_index) index: u32, instance: ImageInstance) -> ImageOut {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0)
    );
    let corner = corners[index];
    var out: ImageOut;
    out.clip = vec4<f32>(instance.pos + corner * instance.size, 0.0, 1.0);
    out.uv = mix(instance.src.xy, instance.src.zw, corner);
    out.tint = instance.tint;
    return out;
}

@fragment
fn fs_image(in: ImageOut) -> @location(0) vec4<f32> {
    let uv = in.uv / image_size.size;
    let texel = textureSample(image_texture, image_sampler, uv);
    return vec4<f32>(texel.rgb * in.tint.rgb, texel.a * in.tint.a);
}
"#;

/// One textured image quad: a screen-space destination rect and a source rect
/// in the source image's own pixels. The shader divides by the image size.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ImageInstanceData {
    pos: [f32; 2],
    size: [f32; 2],
    src: [f32; 4],
    tint: [f32; 4],
}

/// The CPU side of the image pipeline: a growable instance list the GPU draws
/// with the image bind group.
#[derive(Default)]
pub struct ImageBatcher {
    pub instances: Vec<ImageInstanceData>,
}

impl ImageBatcher {
    pub fn clear(&mut self) {
        self.instances.clear();
    }

    pub fn count(&self) -> usize {
        self.instances.len()
    }

    /// Draw part (or all) of a source image at a screen rect. Coordinates are
    /// top-left-origin device pixels; the source rect is in the image's own
    /// pixels, exactly as `review.json` declares it.
    #[allow(clippy::too_many_arguments)]
    pub fn image(
        &mut self,
        screen: &Screen,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        src: [f32; 4],
        tint: [f32; 4],
    ) {
        let (cx, cy) = screen.to_clip(x.round(), y.round());
        let (cw, ch) = screen.size_to_clip(w.round().max(1.0), h.round().max(1.0));
        self.instances.push(ImageInstanceData {
            pos: [cx, cy],
            size: [cw, ch],
            src,
            tint,
        });
    }
}

fn world_attributes() -> [wgpu::VertexAttribute; 5] {
    [
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x3,
            offset: 0,
            shader_location: 0,
        },
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x3,
            offset: 12,
            shader_location: 1,
        },
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x3,
            offset: 24,
            shader_location: 2,
        },
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x4,
            offset: 36,
            shader_location: 3,
        },
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x4,
            offset: 52,
            shader_location: 4,
        },
    ]
}

impl Gpu {
    pub fn new(window: std::sync::Arc<winit::window::Window>, _text: &Text) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance
            .create_surface(window.clone())
            .expect("a surface for the window");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        }))
        .expect("a GPU adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("ala-cities"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::Off,
        }))
        .expect("a device");

        let caps = surface.get_capabilities(&adapter);
        let present_modes = caps.present_modes.clone();
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .expect("a surface configuration");

        // Uncapped, so a 240 Hz panel is actually fed.
        //
        // Mailbox is preferred because it does not tear, and it is chosen only
        // when the surface actually reports it: unlike the Auto modes, Mailbox
        // does not fall back when unsupported — it crashes. This machine's own
        // capabilities were read as `[Fifo, FifoRelaxed, Mailbox, Immediate]`,
        // which is why the branch exists rather than an assumption.
        config.present_mode = if present_modes.contains(&wgpu::PresentMode::Mailbox) {
            wgpu::PresentMode::Mailbox
        } else if present_modes.contains(&wgpu::PresentMode::AutoNoVsync) {
            wgpu::PresentMode::AutoNoVsync
        } else {
            wgpu::PresentMode::Fifo
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("quad shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let world_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("world shader"),
            source: wgpu::ShaderSource::Wgsl(WORLD_SHADER.into()),
        });

        let atlas_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("atlas layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let world_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("world layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Solid geometry samples this one opaque texel. Glyphs are prepared
        // and packed by Glyphon in its own atlas.
        let texture = device.create_texture_with_data(
            &queue,
            &wgpu::TextureDescriptor {
                label: Some("solid white texel"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &[255u8],
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        // Solid geometry samples a single texel, so nearest filtering prevents
        // interpolation with an unintended neighbouring value.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("atlas sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("atlas bind group"),
            layout: &atlas_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let glyphon_cache = GlyphonCache::new(&device);
        let mut text_atlas = TextAtlas::new(&device, &queue, &glyphon_cache, config.format);
        let text_viewport = Viewport::new(&device, &glyphon_cache);
        let text_renderer = TextRenderer::new(
            &mut text_atlas,
            &device,
            wgpu::MultisampleState::default(),
            Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: Default::default(),
                bias: Default::default(),
            }),
        );

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera"),
            contents: bytemuck::bytes_of(&CameraUniform {
                view_proj: Mat4::IDENTITY.to_cols_array(),
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let world_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("world bind group"),
            layout: &world_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let interface_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("interface layout"),
                bind_group_layouts: &[Some(&atlas_layout)],
                immediate_size: 0,
            });
        let world_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("world layout"),
                bind_group_layouts: &[Some(&world_layout)],
                immediate_size: 0,
            });

        let blend = Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("interface pipeline"),
            layout: Some(&interface_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Instance>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 16,
                            shader_location: 2,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 32,
                            shader_location: 3,
                        },
                    ],
                })],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            // The pass carries a depth attachment, so even the interface
            // pipeline has to declare a depth state. It always passes: the HUD
            // is in front of the world by construction.
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        let world_pipeline = |label: &str,
                              entry: &str,
                              depth_write: bool,
                              compare: wgpu::CompareFunction| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&world_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &world_shader,
                    entry_point: Some(entry),
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<WorldInstance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &world_attributes(),
                    })],
                    compilation_options: Default::default(),
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    // No back-face culling: a quad drawn with a basis pointing
                    // away is skipped on the CPU, where a box already knows
                    // which of its faces the camera can see.
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: Some(depth_write),
                    depth_compare: Some(compare),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &world_shader,
                    entry_point: Some("fs_world"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                multiview_mask: None,
                cache: None,
            })
        };

        let world_opaque = world_pipeline(
            "world opaque",
            "vs_world",
            true,
            wgpu::CompareFunction::LessEqual,
        );
        let world_overlay = world_pipeline(
            "world overlay",
            "vs_world",
            false,
            wgpu::CompareFunction::LessEqual,
        );

        // The image pipeline. One 4×4 opaque-white RGBA texture stands in until
        // a real image is bound with `bind_image`; the shader divides source
        // pixels by the uniform image size, so the atlas never touches the CPU.
        let image_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("image atlas"),
            size: wgpu::Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let image_view = image_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let image_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("image sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let image_size_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("image size"),
            size: 8,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let image_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("image layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let image_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("image bind group"),
            layout: &image_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&image_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&image_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: image_size_buffer.as_entire_binding(),
                },
            ],
        });
        let image_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("image pipeline layout"),
                bind_group_layouts: &[Some(&image_bind_group_layout)],
                immediate_size: 0,
            });
        let image_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("image shader"),
            source: wgpu::ShaderSource::Wgsl(IMAGE_SHADER.into()),
        });
        let image_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("image pipeline"),
            layout: Some(&image_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &image_shader,
                entry_point: Some("vs_image"),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<ImageInstanceData>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 16,
                            shader_location: 2,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 32,
                            shader_location: 3,
                        },
                    ],
                })],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &image_shader,
                entry_point: Some("fs_image"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        let image_capacity = 1024usize;
        let images = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("image instances"),
            contents: bytemuck::cast_slice(&vec![
                ImageInstanceData {
                    pos: [0.0; 2],
                    size: [0.0; 2],
                    src: [0.0; 4],
                    tint: [0.0; 4],
                };
                image_capacity
            ]),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let (depth, depth_view) = create_depth(&device, &config);

        // Start with room for a full-screen-worth of quads; it grows on demand.
        let instance_capacity = 4096;
        let instances = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("interface instances"),
            contents: bytemuck::cast_slice(&vec![Instance::zeroed(); instance_capacity]),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let world_capacity = 8192;
        let world_opaque_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("world instances (opaque)"),
            contents: bytemuck::cast_slice(&vec![WorldInstance::zeroed(); world_capacity]),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let world_overlay_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("world instances (overlay)"),
            contents: bytemuck::cast_slice(&vec![WorldInstance::zeroed(); world_capacity]),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let info = adapter.get_info();
        tracing::info!(
            adapter = %info.name,
            backend = ?info.backend,
            present_modes = ?present_modes,
            format = ?config.format,
            depth = ?DEPTH_FORMAT,
            "renderer ready"
        );

        Self {
            window,
            surface,
            device,
            queue,
            config,
            pipeline,
            world_opaque,
            world_overlay,
            image_pipeline,
            image_bind_group,
            image_texture,
            image_sampler,
            image_size_buffer,
            images,
            image_capacity,
            bind_group,
            world_bind_group,
            camera_buffer,
            depth,
            depth_view,
            atlas_texture: texture,
            text_atlas,
            text_renderer,
            text_viewport,
            instances,
            instance_capacity,
            world_opaque_buffer,
            world_opaque_capacity: world_capacity,
            world_overlay_buffer,
            world_overlay_capacity: world_capacity,
            present_modes,
            adapter_name: info.name,
            stats: FrameStats::default(),
            last_camera: None,
        }
    }

    /// Bind an RGBA8 image as the image pipeline's source atlas and record its
    /// size for the shader's division. The texture is (re)created at the bound
    /// size — wgpu textures do not resize — and the bind group is rebuilt
    /// around the new view. The image stays GPU-side: the CPU only ever speaks
    /// source pixels, which is what keeps `review.json`'s coordinates
    /// authoritative end to end.
    pub fn bind_image(&mut self, rgba: &[u8], width: u32, height: u32) {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("image atlas"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.image_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("image bind group"),
            layout: &self.image_bind_group_layout(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.image_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.image_size_buffer.as_entire_binding(),
                },
            ],
        });
        self.queue.write_buffer(
            &self.image_size_buffer,
            0,
            bytemuck::bytes_of(&[width as f32, height as f32]),
        );
        self.image_texture = texture;
    }

    /// The image bind group layout, rebuilt on demand by `bind_image`. It is
    /// deterministic, so a field would only be a second place to keep it.
    fn image_bind_group_layout(&self) -> wgpu::BindGroupLayout {
        self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("image layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }

    fn ensure_image_capacity(&mut self, needed: usize) {
        if needed <= self.image_capacity {
            return;
        }
        let capacity = needed.next_power_of_two();
        self.images = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("image instances"),
            size: (capacity * std::mem::size_of::<ImageInstanceData>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.image_capacity = capacity;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        let (depth, depth_view) = create_depth(&self.device, &self.config);
        self.depth = depth;
        self.depth_view = depth_view;
    }

    pub fn screen(&self) -> Screen {
        Screen {
            w: self.config.width as f32,
            h: self.config.height as f32,
        }
    }

    pub fn prepare_text(&mut self, text: &mut Text) -> Result<(), glyphon::PrepareError> {
        let screen = self.screen();
        text.prepare(
            &self.device,
            &self.queue,
            &mut self.text_atlas,
            &mut self.text_renderer,
            &mut self.text_viewport,
            screen,
        )
    }

    fn ensure_instance_capacity(&mut self, needed: usize) {
        if needed <= self.instance_capacity {
            return;
        }
        let capacity = needed.next_power_of_two();
        self.instances = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("interface instances"),
            size: (capacity * std::mem::size_of::<Instance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.instance_capacity = capacity;
    }

    fn ensure_world_capacity(&mut self, opaque: usize, overlay: usize) {
        if opaque > self.world_opaque_capacity {
            let capacity = opaque.next_power_of_two();
            self.world_opaque_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("world instances (opaque)"),
                size: (capacity * std::mem::size_of::<WorldInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.world_opaque_capacity = capacity;
        }
        if overlay > self.world_overlay_capacity {
            let capacity = overlay.next_power_of_two();
            self.world_overlay_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("world instances (overlay)"),
                size: (capacity * std::mem::size_of::<WorldInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.world_overlay_capacity = capacity;
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        world: &WorldBatch,
        hud: &Batcher,
        images: &ImageBatcher,
        camera: &Camera,
        clear: [f32; 4],
        frame_seconds: f32,
        text: &mut Text,
    ) -> bool {
        if let Err(error) = self.prepare_text(text) {
            tracing::error!(%error, "Glyphon text preparation failed");
            return false;
        }
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return false;
            }
            _ => return false,
        };

        self.last_camera = Some(*camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::bytes_of(&CameraUniform {
                view_proj: camera.view_proj().to_cols_array(),
            }),
        );

        self.ensure_instance_capacity(hud.instances.len());
        self.ensure_image_capacity(images.instances.len());
        self.ensure_world_capacity(world.opaque.len(), world.overlay.len());
        if !images.instances.is_empty() {
            self.queue
                .write_buffer(&self.images, 0, bytemuck::cast_slice(&images.instances));
        }
        if !hud.instances.is_empty() {
            self.queue
                .write_buffer(&self.instances, 0, bytemuck::cast_slice(&hud.instances));
        }
        if !world.opaque.is_empty() {
            self.queue.write_buffer(
                &self.world_opaque_buffer,
                0,
                bytemuck::cast_slice(&world.opaque),
            );
        }
        if !world.overlay.is_empty() {
            self.queue.write_buffer(
                &self.world_overlay_buffer,
                0,
                bytemuck::cast_slice(&world.overlay),
            );
        }

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: clear[0] as f64,
                            g: clear[1] as f64,
                            b: clear[2] as f64,
                            a: clear[3] as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            // The city, with depth written.
            if !world.opaque.is_empty() {
                pass.set_pipeline(&self.world_opaque);
                pass.set_bind_group(0, &self.world_bind_group, &[]);
                pass.set_vertex_buffer(0, self.world_opaque_buffer.slice(..));
                pass.draw(0..6, 0..world.opaque.len() as u32);
            }

            // Overlays: shadows, ghosts, the grid, the cursor. Depth-tested
            // against the city, not written, so they cannot hide each other.
            if !world.overlay.is_empty() {
                pass.set_pipeline(&self.world_overlay);
                pass.set_bind_group(0, &self.world_bind_group, &[]);
                pass.set_vertex_buffer(0, self.world_overlay_buffer.slice(..));
                pass.draw(0..6, 0..world.overlay.len() as u32);
            }

            // The interface, always in front.
            if !hud.instances.is_empty() {
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.set_vertex_buffer(0, self.instances.slice(..));
                pass.draw(0..6, 0..hud.instances.len() as u32);
            }

            // Images — candidate plates, and later every icon the game draws.
            // After the interface so a plate's tint can sit on a panel.
            if !images.instances.is_empty() {
                pass.set_pipeline(&self.image_pipeline);
                pass.set_bind_group(0, &self.image_bind_group, &[]);
                pass.set_vertex_buffer(0, self.images.slice(..));
                pass.draw(0..6, 0..images.instances.len() as u32);
            }

            // Glyphon renders shaped screen-space text after solid and image UI
            // geometry, using its dynamic etagere-packed atlas.
            if let Err(error) = self.text_renderer.render(
                &self.text_atlas,
                &self.text_viewport,
                &mut pass,
            ) {
                tracing::error!(%error, "Glyphon text rendering failed");
            }
        }

        self.queue.submit(Some(encoder.finish()));
        self.queue.present(frame);
        self.text_atlas.trim();
        text.clear();
        let refresh = if self
            .present_modes
            .contains(&wgpu::PresentMode::AutoNoVsync)
        {
            240.0
        } else {
            60.0
        };
        self.stats.observe(frame_seconds, refresh);
        true
    }
}

fn create_depth(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth"),
        size: wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn camera(yaw: f32, pitch: f32) -> Camera {
        Camera {
            focus: Vec3::new(500.0, 400.0, 0.0),
            yaw,
            pitch,
            zoom: 2.0,
            screen: Screen {
                w: 1600.0,
                h: 900.0,
            },
        }
    }

    #[test]
    fn the_camera_round_trips_a_ground_point_at_any_angle() {
        // The whole point of an inverse projection: the answers have to hold
        // for every yaw and every tilt, not just the one the tests were written
        // against.
        let mut checked = 0;
        for yaw_step in 0..12 {
            for pitch_step in 0..7 {
                let yaw = yaw_step as f32 * 30.0_f32.to_radians();
                let pitch = MIN_PITCH
                    + (MAX_PITCH - MIN_PITCH) * (pitch_step as f32 / 6.0);
                let camera = camera(yaw, pitch);
                let world = Vec3::new(560.0, 372.0, 0.0);
                let (sx, sy) = camera.world_to_screen(world);
                let picked = camera
                    .pick_ground(sx, sy)
                    .expect("a point on the ground must pick back");
                assert!(
                    (picked.x - world.x).abs() < 0.05 && (picked.y - world.y).abs() < 0.05,
                    "yaw {:.0}° pitch {:.0}°: ({}, {}) picked back as ({}, {})",
                    yaw.to_degrees(),
                    pitch.to_degrees(),
                    world.x,
                    world.y,
                    picked.x,
                    picked.y
                );
                checked += 1;
            }
        }
        assert!(checked >= 80, "the sweep barely ran: {checked} cases");
    }

    #[test]
    fn a_tile_picks_back_the_tile_it_was_drawn_at_at_any_angle() {
        for yaw_step in 0..8 {
            for pitch_step in 0..4 {
                let yaw = yaw_step as f32 * 45.0_f32.to_radians();
                let pitch =
                    MIN_PITCH + (MAX_PITCH - MIN_PITCH) * (pitch_step as f32 / 3.0);
                let camera = camera(yaw, pitch);
                let (tx, ty) = (7i32, 9i32);
                let centre = Vec3::new(
                    tx as f32 * TILE + TILE * 0.5,
                    ty as f32 * TILE + TILE * 0.5,
                    0.0,
                );
                let (sx, sy) = camera.world_to_screen(centre);
                assert_eq!(
                    camera.screen_to_tile(sx, sy),
                    Some((tx, ty)),
                    "yaw {:.0}° pitch {:.0}°",
                    yaw.to_degrees(),
                    pitch.to_degrees()
                );
            }
        }
    }

    #[test]
    fn an_orthographic_view_reports_distance_rather_than_faking_a_horizon() {
        // Under an orthographic projection every screen point meets the ground
        // plane, so there is no "sky" to report. What matters is that the far
        // edge of the screen reports a *distant* tile instead of being clamped
        // into the city, and that the visible set stays bounded.
        let camera = camera(0.0, MIN_PITCH);
        let near = camera
            .pick_ground(camera.screen.w / 2.0, camera.screen.h)
            .expect("the near edge is ground");
        let far = camera
            .pick_ground(camera.screen.w / 2.0, 0.0)
            .expect("the far edge is ground");
        assert!(
            far.y > near.y,
            "looking north, the top of the screen must be further north"
        );
        let (left, top, right, bottom) = camera.visible_tiles(256, 256);
        assert!(right > left && bottom > top, "the visible set collapsed");
        assert!(
            right - left <= 256 && bottom - top <= 256,
            "the visible set escaped the map: {left},{top},{right},{bottom}"
        );
    }

    #[test]
    fn zooming_keeps_the_point_under_the_cursor() {
        let mut camera = camera(0.7, 0.9);
        let cursor = (900.0, 300.0);
        let before = camera.pick_ground(cursor.0, cursor.1).expect("ground");
        camera.zoom_by(1.5, cursor);
        let after = camera.pick_ground(cursor.0, cursor.1).expect("ground");
        assert!((before.x - after.x).abs() < 0.5, "x drifted");
        assert!((before.y - after.y).abs() < 0.5, "y drifted");
    }

    #[test]
    fn tilting_and_rotating_stay_inside_their_limits() {
        let mut camera = camera(0.0, DEFAULT_PITCH);
        camera.orbit(100_000.0, -100_000.0);
        camera.clamp_to(256, 256);
        assert!((MIN_PITCH..=MAX_PITCH).contains(&camera.pitch));
        assert!((0.0..std::f32::consts::TAU).contains(&camera.yaw));
        camera.orbit(-250_000.0, 900_000.0);
        camera.clamp_to(256, 256);
        assert!((MIN_PITCH..=MAX_PITCH).contains(&camera.pitch));
        assert!((0.0..std::f32::consts::TAU).contains(&camera.yaw));
    }

    #[test]
    fn a_box_draws_the_faces_the_camera_can_see_and_no_more() {
        let mut batch = WorldBatch::default();
        // Looking straight down: the roof is visible and all four walls are
        // edge-on, so only the roof is worth drawing.
        batch.clear(Vec3::Z);
        batch.building(0.0, 0.0, 5.0, 10.0, [1.0, 1.0, 1.0, 1.0], 1.0);
        let straight_down = batch.opaque.len();

        batch.clear(Vec3::new(0.0, -0.7, 0.7));
        batch.building(0.0, 0.0, 5.0, 10.0, [1.0, 1.0, 1.0, 1.0], 1.0);
        let tilted = batch.opaque.len();

        assert_eq!(straight_down, 1, "a top-down box is one roof");
        assert!(
            tilted > straight_down,
            "a tilted camera must see walls as well as a roof"
        );
        assert!(tilted <= 5, "a box is at most five faces");
    }

    #[test]
    fn a_visible_set_always_contains_the_focus() {
        let camera = camera(1.1, 0.8);
        let (left, top, right, bottom) = camera.visible_tiles(256, 256);
        let (tx, ty) = (camera.focus.x / TILE, camera.focus.y / TILE);
        assert!(
            left as f32 <= tx && tx <= right as f32 && top as f32 <= ty && ty <= bottom as f32,
            "the tile under the camera was not in the visible set"
        );
    }

    #[test]
    fn the_bearing_says_where_the_top_of_the_screen_points() {
        let mut camera = camera(0.0, DEFAULT_PITCH);
        assert!(
            camera.bearing_degrees() < 1.0,
            "at rest the top of the screen is north"
        );
        // A quarter turn clockwise puts the top of the screen on east.
        camera.yaw = std::f32::consts::FRAC_PI_2;
        assert!((camera.bearing_degrees() - 90.0).abs() < 1.0);
        camera.yaw = std::f32::consts::PI;
        assert!((camera.bearing_degrees() - 180.0).abs() < 1.0);
        // And it is a bearing, not a running total.
        camera.yaw = -std::f32::consts::FRAC_PI_2;
        assert!(
            (camera.bearing_degrees() - 270.0).abs() < 1.0,
            "a negative rotation must read as a bearing"
        );
    }

    #[test]
    fn the_top_of_the_screen_really_is_north_at_rest() {
        // The bearing is a claim about the projection, so it is checked
        // against the projection rather than trusted.
        let camera = camera(0.0, DEFAULT_PITCH);
        let centre = camera.focus;
        let (sx, sy) = camera.world_to_screen(centre);
        let ground = camera
            .pick_ground(sx, sy)
            .expect("the centre of the screen is ground");
        assert!((ground.x - centre.x).abs() < 0.5);
        assert!((ground.y - centre.y).abs() < 0.5);
        // A point north of the focus (larger y) must be higher on screen.
        let (_, north_y) = camera.world_to_screen(centre + Vec3::new(0.0, 100.0, 0.0));
        assert!(north_y < sy, "north pointed down the screen");
    }

    #[test]
    fn a_frame_worst_case_is_not_hidden_by_the_average() {
        let mut stats = FrameStats::default();
        for _ in 0..100 {
            stats.observe(1.0 / 240.0, 240.0);
        }
        stats.observe(0.05, 240.0);
        assert!(stats.average_ms < 6.0, "the mean stays low");
        assert!(stats.worst_ms >= 49.0, "the worst frame is still visible");
    }

}
