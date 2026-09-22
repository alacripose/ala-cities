//! Rendering.
//!
//! One pipeline draws everything: tiles, roads, buildings, agents, panels and
//! glyphs are all instanced quads sampling a single-channel coverage atlas. A
//! solid fill samples a texel that is already 1.0, a glyph samples its own
//! coverage, and the fragment shader multiplies the colour's alpha by whatever
//! it sampled. That is the whole renderer.
//!
//! There is no uniform buffer and no matrix maths on the GPU: the camera is
//! flattened on the CPU into clip-space rectangles, which is both simpler and
//! faster for a 2D top-down view.

use std::collections::HashMap;

use ab_glyph::{Font, FontVec, PxScale, ScaleFont};
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

pub const ATLAS_SIZE: u32 = 1024;
pub const TILE: f32 = 12.0;

/// One instanced quad, already in clip space.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Instance {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub uv: [f32; 4],
}

/// A glyph's place in the atlas and its metrics. `x_offset`/`y_offset` are from
/// the text box's top-left to the bitmap's top-left, so layout never has to
/// reason about baselines.
#[derive(Clone, Copy, Debug)]
pub struct Slot {
    pub uv: [f32; 4],
    pub w: f32,
    pub h: f32,
    pub x_offset: f32,
    pub y_offset: f32,
    pub advance: f32,
}

/// Which face to draw with. Numbers, ticket ids, governor versions and check
/// digits use the monospace face, so a digit can be read by eye without the
/// surrounding prose shifting under it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Face {
    Body,
    Mono,
}

pub struct Text {
    fonts: [Option<FontVec>; 2],
    /// R8 coverage. The first two texels are solid white, so a "solid" draw is
    /// the same sampling path as a glyph with no special case.
    pub data: Vec<u8>,
    pen_x: u32,
    pen_y: u32,
    row_height: u32,
    cache: HashMap<(u8, char, u32), Slot>,
    pub refused: bool,
    pub missing_font: bool,
}

impl Text {
    /// Find an installed font and read it. The font is never copied into the
    /// build and never redistributed: it is referenced from the operating
    /// system at runtime, which is why there is nothing here to license.
    pub fn new() -> Self {
        let body_paths = [
            "C:/Windows/Fonts/segoeui.ttf",
            "C:/Windows/Fonts/tahoma.ttf",
            "C:/Windows/Fonts/arial.ttf",
            "C:/Windows/Fonts/calibri.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/System/Library/Fonts/Helvetica.ttc",
        ];
        let mono_paths = [
            "C:/Windows/Fonts/consola.ttf",
            "C:/Windows/Fonts/cour.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
        ];
        let load = |paths: &[&str]| -> Option<(String, FontVec)> {
            for path in paths {
                if let Ok(bytes) = std::fs::read(path) {
                    if let Ok(parsed) = FontVec::try_from_vec(bytes) {
                        tracing::info!(font = path, "using a system font");
                        return Some((path.to_string(), parsed));
                    }
                }
            }
            None
        };
        let body = load(&body_paths);
        // A missing monospace face falls back to the body face rather than to
        // nothing: legibility degrades, correctness does not.
        let mono = match load(&mono_paths) {
            Some(font) => Some(font),
            None => body
                .as_ref()
                .and_then(|(path, _)| load(&[path.as_str()])),
        };
        let fonts = [body.map(|(_, f)| f), mono.map(|(_, f)| f)];
        if fonts[0].is_none() {
            tracing::warn!("no system font found; the interface will draw without text");
        }

        let mut data = vec![0u8; (ATLAS_SIZE * ATLAS_SIZE) as usize];
        // Solid region at the origin.
        for y in 0..2u32 {
            for x in 0..2u32 {
                data[(y * ATLAS_SIZE + x) as usize] = 255;
            }
        }

        Self {
            missing_font: fonts[0].is_none(),
            fonts,
            data,
            pen_x: 4,
            pen_y: 0,
            row_height: 0,
            cache: HashMap::new(),
            refused: false,
        }
    }

    /// UVs of the solid white texel. Every non-text quad uses this.
    pub fn solid_uv() -> [f32; 4] {
        [0.5 / ATLAS_SIZE as f32, 0.5 / ATLAS_SIZE as f32, 0.5 / ATLAS_SIZE as f32, 0.5 / ATLAS_SIZE as f32]
    }

    fn rasterise(&mut self, face: Face, ch: char, size: u32) -> Option<Slot> {
        let font = self.fonts[face as usize].as_ref()?;
        let scale = PxScale::from(size as f32);
        let scaled = font.as_scaled(scale);
        let id = font.glyph_id(ch);
        let advance = scaled.h_advance(id);
        let ascent = scaled.ascent();
        if ch == ' ' || ch == '\t' {
            return Some(Slot {
                uv: Self::solid_uv(),
                w: 0.0,
                h: 0.0,
                x_offset: 0.0,
                y_offset: 0.0,
                advance,
            });
        }

        let glyph = id.with_scale_and_position(scale, ab_glyph::point(0.0, ascent));
        let outline = font.outline_glyph(glyph)?;
        let bounds = outline.px_bounds();
        let w = bounds.width().ceil().max(1.0) as u32;
        let h = bounds.height().ceil().max(1.0) as u32;

        let mut coverage = vec![0.0f32; (w * h) as usize];
        outline.draw(|x, y, value| {
            let index = (y * w + x) as usize;
            if index < coverage.len() {
                coverage[index] = value;
            }
        });

        // A one-pixel gutter, so linear filtering cannot bleed one glyph into
        // its neighbour. Getting this wrong looks like a font bug and is not.
        let pad = 1;
        if self.pen_x + w + pad >= ATLAS_SIZE {
            self.pen_x = 4;
            self.pen_y += self.row_height + pad;
            self.row_height = 0;
        }
        if self.pen_y + h + pad >= ATLAS_SIZE {
            // The atlas is full. Refusing to draw is honest; drawing a
            // wrong glyph because the coordinates wrapped would not be.
            if !self.refused {
                self.refused = true;
                tracing::error!("glyph atlas is full; further glyphs will not be drawn");
            }
            return None;
        }

        let at_x = self.pen_x;
        let at_y = self.pen_y;
        for y in 0..h {
            for x in 0..w {
                let value = coverage[(y * w + x) as usize];
                let px = at_x + x;
                let py = at_y + y;
                self.data[(py * ATLAS_SIZE + px) as usize] = (value * 255.0) as u8;
            }
        }
        self.pen_x += w + pad;
        self.row_height = self.row_height.max(h + pad);

        Some(Slot {
            uv: [
                at_x as f32 / ATLAS_SIZE as f32,
                at_y as f32 / ATLAS_SIZE as f32,
                (at_x + w) as f32 / ATLAS_SIZE as f32,
                (at_y + h) as f32 / ATLAS_SIZE as f32,
            ],
            w: w as f32,
            h: h as f32,
            x_offset: bounds.min.x.floor(),
            y_offset: ascent - bounds.max.y,
            advance,
        })
    }

    fn slot(&mut self, face: Face, ch: char, size: u32) -> Option<Slot> {
        let key = (face as u8, ch, size);
        if let Some(slot) = self.cache.get(&key) {
            return Some(*slot);
        }
        let slot = self.rasterise(face, ch, size)?;
        self.cache.insert(key, slot);
        Some(slot)
    }

    pub fn measure(&mut self, face: Face, text: &str, size: u32) -> f32 {
        let mut width = 0.0;
        for ch in text.chars() {
            if let Some(slot) = self.slot(face, ch, size) {
                width += slot.advance;
            }
        }
        width
    }

    /// Draw text with its top-left at `(x, y)` in screen pixels, pushing quads
    /// into the batcher through `screen`.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        face: Face,
        batch: &mut Batcher,
        screen: &Screen,
        x: f32,
        y: f32,
        size: u32,
        color: [f32; 4],
        text: &str,
    ) -> f32 {
        let mut pen = x;
        for ch in text.chars() {
            let Some(slot) = self.slot(face, ch, size) else {
                continue;
            };
            if slot.w > 0.0 {
                batch.screen_rect(
                    screen,
                    pen + slot.x_offset,
                    y + slot.y_offset,
                    slot.w,
                    slot.h,
                    color,
                    slot.uv,
                );
            }
            pen += slot.advance;
        }
        pen - x
    }
}

/// The CPU-side batcher. Quads are already in clip space by the time they get
/// here, which is what removes the need for a GPU-side camera.
#[derive(Default)]
pub struct Batcher {
    pub instances: Vec<Instance>,
}

impl Batcher {
    pub fn clear(&mut self) {
        self.instances.clear();
    }

    /// A rectangle in clip space.
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

    /// A rectangle in screen pixels, top-left origin.
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
        let (cx, cy) = screen.to_clip(x, y);
        let (cw, ch) = screen.size_to_clip(w, h);
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
}

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    /// Centre of the view in world pixels.
    pub x: f32,
    pub y: f32,
    /// Screen pixels per world pixel.
    pub zoom: f32,
    pub screen: Screen,
}

impl Camera {
    pub fn new(screen: Screen, map_width: u32, map_height: u32) -> Self {
        let scale = screen.h / (map_height as f32 * TILE) * 1.6;
        Self {
            x: (map_width as f32 * TILE) / 2.0,
            y: (map_height as f32 * TILE) / 2.0,
            zoom: scale.max(0.2),
            screen,
        }
    }

    pub fn clamp_to(&mut self, map_width: u32, map_height: u32) {
        self.zoom = self.zoom.clamp(0.15, 6.0);
        let margin = 200.0;
        self.x = self
            .x
            .clamp(-margin, map_width as f32 * TILE + margin);
        self.y = self
            .y
            .clamp(-margin, map_height as f32 * TILE + margin);
    }

    pub fn pan(&mut self, dx_screen: f32, dy_screen: f32) {
        self.x -= dx_screen / self.zoom;
        self.y -= dy_screen / self.zoom;
    }

    pub fn zoom_by(&mut self, factor: f32, at_screen: (f32, f32)) {
        let before = self.screen_to_world(at_screen.0, at_screen.1);
        self.zoom = (self.zoom * factor).clamp(0.15, 6.0);
        let after = self.screen_to_world(at_screen.0, at_screen.1);
        self.x += before.0 - after.0;
        self.y += before.1 - after.1;
    }

    pub fn world_to_screen(&self, x: f32, y: f32) -> (f32, f32) {
        (
            (x - self.x) * self.zoom + self.screen.w / 2.0,
            (y - self.y) * self.zoom + self.screen.h / 2.0,
        )
    }

    pub fn screen_to_world(&self, sx: f32, sy: f32) -> (f32, f32) {
        (
            (sx - self.screen.w / 2.0) / self.zoom + self.x,
            (sy - self.screen.h / 2.0) / self.zoom + self.y,
        )
    }

    pub fn screen_to_tile(&self, sx: f32, sy: f32) -> (i32, i32) {
        let (wx, wy) = self.screen_to_world(sx, sy);
        ((wx / TILE).floor() as i32, (wy / TILE).floor() as i32)
    }

    /// The clip-space rectangle for a world-space rectangle.
    pub fn world_rect(&self, batch: &mut Batcher, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        let (sx, sy) = self.world_to_screen(x, y);
        batch.screen_rect(
            &self.screen,
            sx,
            sy,
            w * self.zoom,
            h * self.zoom,
            color,
            Text::solid_uv(),
        );
    }

    /// Which tiles are on screen, so the client can skip the other 60,000.
    pub fn visible_tiles(&self, map_width: u32, map_height: u32) -> (u32, u32, u32, u32) {
        let (left, top) = self.screen_to_tile(0.0, 0.0);
        let (right, bottom) = self.screen_to_tile(self.screen.w, self.screen.h);
        (
            left.clamp(0, map_width as i32) as u32,
            top.clamp(0, map_height as i32) as u32,
            right.clamp(0, map_width as i32) as u32,
            bottom.clamp(0, map_height as i32) as u32,
        )
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

pub struct Gpu {
    pub window: std::sync::Arc<winit::window::Window>,
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub atlas_texture: wgpu::Texture,
    instances: wgpu::Buffer,
    instance_capacity: usize,
    pub present_modes: Vec<wgpu::PresentMode>,
    pub adapter_name: String,
    pub stats: FrameStats,
    pub atlas_uploaded: u64,
}

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

impl Gpu {
    pub fn new(window: std::sync::Arc<winit::window::Window>, text: &Text) -> Self {
        let size = window.inner_size();
        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance
            .create_surface(window.clone())
            .expect("a surface for the window");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
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

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let texture = device.create_texture_with_data(
            &queue,
            &wgpu::TextureDescriptor {
                label: Some("coverage atlas"),
                size: wgpu::Extent3d {
                    width: ATLAS_SIZE,
                    height: ATLAS_SIZE,
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
            &text.data,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("atlas sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("atlas bind group"),
            layout: &bind_group_layout,
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("quad layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let attributes = [
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
        ];

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("quad pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Instance>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &attributes,
                }],
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
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
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        // Start with room for a full-screen-worth of quads; it grows on demand.
        let instance_capacity = 4096;
        let instances = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instances"),
            contents: bytemuck::cast_slice(&vec![Instance::zeroed(); instance_capacity]),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let info = adapter.get_info();
        tracing::info!(
            adapter = %info.name,
            backend = ?info.backend,
            present_modes = ?present_modes,
            format = ?config.format,
            "renderer ready"
        );

        Self {
            window,
            surface,
            device,
            queue,
            config,
            pipeline,
            bind_group,
            atlas_texture: texture,
            instances,
            instance_capacity,
            present_modes,
            adapter_name: info.name,
            stats: FrameStats::default(),
            atlas_uploaded: 0,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn screen(&self) -> Screen {
        Screen {
            w: self.config.width as f32,
            h: self.config.height as f32,
        }
    }

    /// Upload the coverage atlas. Called only when new glyphs have been
    /// rasterised, not every frame: re-uploading a megabyte per frame would
    /// make the frame budget a lie.
    pub fn upload_atlas(&mut self, text: &Text) {
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.atlas_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &text.data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(ATLAS_SIZE),
                rows_per_image: Some(ATLAS_SIZE),
            },
            wgpu::Extent3d {
                width: ATLAS_SIZE,
                height: ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
        );
        self.atlas_uploaded += 1;
    }

    fn ensure_capacity(&mut self, needed: usize) {
        if needed <= self.instance_capacity {
            return;
        }
        let capacity = needed.next_power_of_two();
        self.instances = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instances"),
            size: (capacity * std::mem::size_of::<Instance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.instance_capacity = capacity;
    }

    pub fn render(&mut self, batch: &Batcher, clear: [f32; 4], frame_seconds: f32) -> bool {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return false;
            }
            _ => return false,
        };

        self.ensure_capacity(batch.instances.len());
        if !batch.instances.is_empty() {
            self.queue
                .write_buffer(&self.instances, 0, bytemuck::cast_slice(&batch.instances));
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
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            if !batch.instances.is_empty() {
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.set_vertex_buffer(0, self.instances.slice(..));
                pass.draw(0..6, 0..batch.instances.len() as u32);
            }
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
        let refresh = if self
            .present_modes
            .contains(&wgpu::PresentMode::AutoNoVsync) { 240.0 } else { 60.0 };
        self.stats.observe(frame_seconds, refresh);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_camera_round_trips_a_point() {
        let screen = Screen { w: 1600.0, h: 900.0 };
        let camera = Camera {
            x: 500.0,
            y: 400.0,
            zoom: 2.0,
            screen,
        };
        let (sx, sy) = camera.world_to_screen(600.0, 500.0);
        let (wx, wy) = camera.screen_to_world(sx, sy);
        assert!((wx - 600.0).abs() < 0.01);
        assert!((wy - 500.0).abs() < 0.01);
    }

    #[test]
    fn a_tile_picks_back_the_tile_it_was_drawn_at() {
        let screen = Screen { w: 1600.0, h: 900.0 };
        let camera = Camera {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
            screen,
        };
        let (sx, sy) = camera.world_to_screen(7.0 * TILE + 3.0, 9.0 * TILE + 3.0);
        assert_eq!(camera.screen_to_tile(sx, sy), (7, 9));
    }

    #[test]
    fn zooming_keeps_the_point_under_the_cursor() {
        let screen = Screen { w: 1600.0, h: 900.0 };
        let mut camera = Camera {
            x: 100.0,
            y: 100.0,
            zoom: 1.0,
            screen,
        };
        let cursor = (900.0, 300.0);
        let before = camera.screen_to_world(cursor.0, cursor.1);
        camera.zoom_by(1.5, cursor);
        let after = camera.screen_to_world(cursor.0, cursor.1);
        assert!((before.0 - after.0).abs() < 0.5, "x drifted");
        assert!((before.1 - after.1).abs() < 0.5, "y drifted");
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

    #[test]
    fn the_solid_texel_is_inside_the_atlas() {
        let uv = Text::solid_uv();
        assert!(uv[0] > 0.0 && uv[0] < 1.0);
        assert_eq!(uv[0], uv[2], "a solid draw samples one texel");
    }
}
