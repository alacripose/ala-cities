//! Deterministic presentation asset generation for the target client and the
//! external asset-builder tool.
//!
//! One generator owns two deliberately separate outputs: sparse world-chunk
//! assets derived from authoritative chunk truth, and asset-scoped shape meshes
//! derived from named presentation recipes. Shape controls never mutate or
//! independently rescale chunk voxels.

use std::collections::BTreeMap;
use std::fmt;

use glam::Vec3;

use crate::founding_day::{ChunkCoord, Coord, GeneratorRevision, VoxelKind, WorldSeed};
use crate::octree::SparseVoxelOctree;
use crate::raycast::VoxelHit;

pub const TEETH_MIN: f32 = 6.0;
pub const TEETH_MAX: f32 = 12.0;
pub const OPENING_MIN: f32 = 0.22;
pub const OPENING_MAX: f32 = 0.39;
pub const ACCENT_MIN: f32 = 0.0;
pub const ACCENT_MAX: f32 = 1.0;
pub const RECIPE_QUANTUM: f32 = 1.0 / 1024.0;

const SHAPE_RECIPE_SCHEMA: &[u8] = b"ala-cities/semantic-shape-recipe/v2\0";
const CHUNK_RECIPE_SCHEMA: &[u8] = b"ala-cities/chunk-mesh-recipe/v2\0";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShapeControl {
    Teeth,
    Opening,
    Accent,
}

impl ShapeControl {
    pub const ALL: [Self; 3] = [Self::Teeth, Self::Opening, Self::Accent];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Teeth => "Teeth",
            Self::Opening => "Opening",
            Self::Accent => "Accent",
        }
    }
}

pub const fn control_bounds(control: ShapeControl) -> (f32, f32) {
    match control {
        ShapeControl::Teeth => (TEETH_MIN, TEETH_MAX),
        ShapeControl::Opening => (OPENING_MIN, OPENING_MAX),
        ShapeControl::Accent => (ACCENT_MIN, ACCENT_MAX),
    }
}

pub const fn control_nudge_step(control: ShapeControl) -> f32 {
    match control {
        ShapeControl::Teeth => 0.125,
        ShapeControl::Opening => 0.005,
        ShapeControl::Accent => 0.01,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AssetRecipe {
    pub teeth: f32,
    pub opening: f32,
    pub accent: f32,
}

impl Default for AssetRecipe {
    fn default() -> Self {
        Self {
            teeth: 9.0,
            opening: 0.30,
            accent: 0.50,
        }
    }
}

impl AssetRecipe {
    fn canonical(control: ShapeControl, value: f32) -> f32 {
        let (min, max) = control_bounds(control);
        let clamped = if value.is_finite() {
            value.clamp(min, max)
        } else {
            (min + max) * 0.5
        };
        (clamped / RECIPE_QUANTUM).round() * RECIPE_QUANTUM
    }

    pub fn set(&mut self, control: ShapeControl, value: f32) -> bool {
        let value = Self::canonical(control, value);
        let slot = match control {
            ShapeControl::Teeth => &mut self.teeth,
            ShapeControl::Opening => &mut self.opening,
            ShapeControl::Accent => &mut self.accent,
        };
        if *slot == value {
            return false;
        }
        *slot = value;
        true
    }

    pub fn value(self, control: ShapeControl) -> f32 {
        match control {
            ShapeControl::Teeth => self.teeth,
            ShapeControl::Opening => self.opening,
            ShapeControl::Accent => self.accent,
        }
    }

    pub fn hash(self) -> BuilderHash {
        let mut hash = fnv1a_seed();
        feed(&mut hash, SHAPE_RECIPE_SCHEMA);
        for control in ShapeControl::ALL {
            feed(
                &mut hash,
                &Self::canonical(control, self.value(control))
                    .to_bits()
                    .to_le_bytes(),
            );
        }
        BuilderHash(hash)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuilderHash(pub u64);

impl fmt::Display for BuilderHash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:016x}", self.0)
    }
}

/// Digest of the authoritative chunk facts that can affect one chunk asset.
/// It includes the chunk itself, the residency and boundary voxels of its six
/// cardinal neighbours, and enough voxel state to invalidate future material
/// presentation without depending on presentation write-back.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChunkInputDigest(pub u64);

impl fmt::Display for ChunkInputDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:016x}", self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkAssetKey {
    pub seed: WorldSeed,
    pub generator_revision: GeneratorRevision,
    pub chunk: ChunkCoord,
    pub builder_hash: BuilderHash,
    pub input_digest: ChunkInputDigest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AssetKind {
    SettingsGear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShapeAssetKey {
    pub asset: AssetKind,
    pub builder_hash: BuilderHash,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeshVertex {
    pub position: Vec3,
    pub normal: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeshQuad {
    pub vertices: [MeshVertex; 4],
    pub material: VoxelKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapeMaterial {
    Body,
    Accent,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShapeVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub material: ShapeMaterial,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShapeFace {
    pub positions: [Vec3; 4],
    pub normal: Vec3,
    pub material: ShapeMaterial,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShapeMesh {
    pub vertices: Vec<ShapeVertex>,
    pub indices: Vec<u32>,
    pub faces: Vec<ShapeFace>,
    inner_radius: Option<f32>,
    outer_radius: Option<f32>,
}

impl ShapeMesh {
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Shape recipes always produce one addressable mesh, even when the mesh
    /// contains multiple material regions or disconnected construction islands.
    pub fn mesh_count(&self) -> usize {
        1
    }

    pub fn inner_radius(&self) -> Option<f32> {
        self.inner_radius
    }

    pub fn outer_radius(&self) -> Option<f32> {
        self.outer_radius
    }

    fn settings_gear(recipe: AssetRecipe) -> Self {
        const SEGMENTS: usize = 240;
        const ROOT_RADIUS: f32 = 1.0;
        const TIP_RADIUS: f32 = 1.22;
        const HALF_DEPTH: f32 = 0.16;

        let inner_radius = ROOT_RADIUS * recipe.opening;
        let mut mesh = Self {
            inner_radius: Some(inner_radius),
            outer_radius: Some(TIP_RADIUS),
            ..Self::default()
        };

        for segment in 0..SEGMENTS {
            let a0 = segment as f32 * std::f32::consts::TAU / SEGMENTS as f32;
            let a1 = (segment + 1) as f32 * std::f32::consts::TAU / SEGMENTS as f32;
            let r0 = gear_radius(a0, recipe.teeth, ROOT_RADIUS, TIP_RADIUS);
            let r1 = gear_radius(a1, recipe.teeth, ROOT_RADIUS, TIP_RADIUS);
            let direction0 = Vec3::new(a0.cos(), a0.sin(), 0.0);
            let direction1 = Vec3::new(a1.cos(), a1.sin(), 0.0);
            let outer0 = direction0 * r0;
            let outer1 = direction1 * r1;
            let inner0 = direction0 * inner_radius;
            let inner1 = direction1 * inner_radius;
            let front = Vec3::Z * HALF_DEPTH;
            let back = Vec3::NEG_Z * HALF_DEPTH;

            push_quad(
                &mut mesh,
                outer0 + front,
                outer1 + front,
                inner1 + front,
                inner0 + front,
                Vec3::Z,
                ShapeMaterial::Body,
            );
            push_quad(
                &mut mesh,
                outer1 + back,
                outer0 + back,
                inner0 + back,
                inner1 + back,
                Vec3::NEG_Z,
                ShapeMaterial::Body,
            );

            let wall_normal = (direction0 + direction1).normalize();
            push_quad(
                &mut mesh,
                outer0 + front,
                outer0 + back,
                outer1 + back,
                outer1 + front,
                wall_normal,
                ShapeMaterial::Body,
            );
            push_quad(
                &mut mesh,
                inner0 + front,
                inner1 + front,
                inner1 + back,
                inner0 + back,
                -wall_normal,
                ShapeMaterial::Body,
            );
        }

        push_accent_prism(&mut mesh, recipe.accent, HALF_DEPTH);
        mesh
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShapeAsset {
    pub key: ShapeAssetKey,
    pub recipe: AssetRecipe,
    pub mesh: ShapeMesh,
    pub digest: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChunkAsset {
    pub key: ChunkAssetKey,
    pub octree: SparseVoxelOctree,
    pub quads: Vec<MeshQuad>,
    pub digest: u64,
}

impl ChunkAsset {
    /// Exact world-space ray query through this chunk's sparse occupancy.
    pub fn raycast(&self, origin: Vec3, direction: Vec3, max_distance: f32) -> Option<VoxelHit> {
        let chunk_origin = Vec3::new(
            (self.key.chunk.x * crate::founding_day::CHUNK_SIZE) as f32,
            (self.key.chunk.y * crate::founding_day::CHUNK_SIZE) as f32,
            (self.key.chunk.z * crate::founding_day::CHUNK_SIZE) as f32,
        );
        let mut hit = self
            .octree
            .raycast_local(origin - chunk_origin, direction, max_distance)?;
        hit.coord = Coord::new(
            hit.coord.x + self.key.chunk.x * crate::founding_day::CHUNK_SIZE,
            hit.coord.y + self.key.chunk.y * crate::founding_day::CHUNK_SIZE,
            hit.coord.z + self.key.chunk.z * crate::founding_day::CHUNK_SIZE,
        );
        if let Some(previous) = hit.previous {
            hit.previous = Some(Coord::new(
                previous.x + self.key.chunk.x * crate::founding_day::CHUNK_SIZE,
                previous.y + self.key.chunk.y * crate::founding_day::CHUNK_SIZE,
                previous.z + self.key.chunk.z * crate::founding_day::CHUNK_SIZE,
            ));
        }
        Some(hit)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetBuildError {
    ChunkNotLoaded(ChunkCoord),
}

impl fmt::Display for AssetBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChunkNotLoaded(chunk) => write!(
                formatter,
                "chunk {},{},{} is not resident",
                chunk.x, chunk.y, chunk.z
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DynamicAssetGenerator {
    recipe: AssetRecipe,
    shape_cache: BTreeMap<ShapeAssetKey, ShapeAsset>,
    chunk_cache: BTreeMap<ChunkAssetKey, ChunkAsset>,
}

impl Default for DynamicAssetGenerator {
    fn default() -> Self {
        Self::new(AssetRecipe::default())
    }
}

impl DynamicAssetGenerator {
    pub fn new(recipe: AssetRecipe) -> Self {
        Self {
            recipe,
            shape_cache: BTreeMap::new(),
            chunk_cache: BTreeMap::new(),
        }
    }

    pub fn recipe(&self) -> AssetRecipe {
        self.recipe
    }

    pub fn builder_hash(&self) -> BuilderHash {
        self.recipe.hash()
    }

    pub fn set_control(&mut self, control: ShapeControl, value: f32) -> bool {
        if !self.recipe.set(control, value) {
            return false;
        }
        self.shape_cache.clear();
        true
    }

    pub fn cache_len(&self) -> usize {
        self.chunk_cache.len()
    }

    pub fn shape_cache_len(&self) -> usize {
        self.shape_cache.len()
    }

    pub fn build_shape(&mut self, asset: AssetKind) -> Result<ShapeAsset, AssetBuildError> {
        let key = ShapeAssetKey {
            asset,
            builder_hash: self.builder_hash(),
        };
        if let Some(cached) = self.shape_cache.get(&key) {
            return Ok(cached.clone());
        }

        let mesh = match asset {
            AssetKind::SettingsGear => ShapeMesh::settings_gear(self.recipe),
        };
        let mut built = ShapeAsset {
            key,
            recipe: self.recipe,
            mesh,
            digest: 0,
        };
        built.digest = shape_digest(&built);
        self.shape_cache.insert(key, built.clone());
        Ok(built)
    }

    pub fn build_chunk(
        &mut self,
        world: &crate::founding_day::FoundingWorld,
        chunk_coord: ChunkCoord,
    ) -> Result<ChunkAsset, AssetBuildError> {
        let key = ChunkAssetKey {
            seed: world.seed,
            generator_revision: world.revision,
            chunk: chunk_coord,
            builder_hash: chunk_builder_hash(),
            input_digest: chunk_input_digest(world, chunk_coord),
        };
        if let Some(cached) = self.chunk_cache.get(&key) {
            return Ok(cached.clone());
        }
        let chunk = world
            .chunk(chunk_coord)
            .ok_or(AssetBuildError::ChunkNotLoaded(chunk_coord))?;

        let mut octree = SparseVoxelOctree::default();
        let mut quads = Vec::new();
        for (local, voxel) in &chunk.voxels {
            octree.insert(*local, voxel.kind);
            let coord = chunk_world_coord(chunk_coord, *local);
            for normal in [
                Coord::new(1, 0, 0),
                Coord::new(-1, 0, 0),
                Coord::new(0, 1, 0),
                Coord::new(0, -1, 0),
                Coord::new(0, 0, 1),
                Coord::new(0, 0, -1),
            ] {
                let neighbour =
                    Coord::new(coord.x + normal.x, coord.y + normal.y, coord.z + normal.z);
                if world.voxel_at(neighbour).is_some() {
                    continue;
                }
                quads.push(MeshQuad {
                    vertices: face_vertices(coord, normal),
                    material: voxel.kind,
                });
            }
        }

        let mut asset = ChunkAsset {
            key,
            octree,
            quads,
            digest: 0,
        };
        asset.digest = asset_digest(&asset);
        self.chunk_cache.insert(key, asset.clone());
        Ok(asset)
    }
}

pub fn chunk_input_digest(
    world: &crate::founding_day::FoundingWorld,
    center: ChunkCoord,
) -> ChunkInputDigest {
    const NEIGHBOUR_OFFSETS: [(i32, i32, i32); 7] = [
        (0, 0, 0),
        (1, 0, 0),
        (-1, 0, 0),
        (0, 1, 0),
        (0, -1, 0),
        (0, 0, 1),
        (0, 0, -1),
    ];

    let mut hash = fnv1a_seed();
    feed(&mut hash, b"ala-cities/chunk-input/v1\0");
    for (dx, dy, dz) in NEIGHBOUR_OFFSETS {
        let neighbour = ChunkCoord::new(center.x + dx, center.y + dy, center.z + dz);
        feed(&mut hash, &neighbour.x.to_le_bytes());
        feed(&mut hash, &neighbour.y.to_le_bytes());
        feed(&mut hash, &neighbour.z.to_le_bytes());
        let Some(chunk) = world.chunk(neighbour) else {
            feed(&mut hash, &[0]);
            continue;
        };
        feed(&mut hash, &[1]);

        let selected = chunk.voxels.iter().filter(|(local, _)| {
            (dx, dy, dz) == (0, 0, 0)
                || (dx == 1 && local.x == 0)
                || (dx == -1 && local.x == crate::founding_day::CHUNK_SIZE - 1)
                || (dy == 1 && local.y == 0)
                || (dy == -1 && local.y == crate::founding_day::CHUNK_SIZE - 1)
                || (dz == 1 && local.z == 0)
                || (dz == -1 && local.z == crate::founding_day::CHUNK_SIZE - 1)
        });
        let selected_count = selected.clone().count() as u64;
        feed(&mut hash, &selected_count.to_le_bytes());
        for (local, voxel) in selected {
            feed(&mut hash, &local.x.to_le_bytes());
            feed(&mut hash, &local.y.to_le_bytes());
            feed(&mut hash, &local.z.to_le_bytes());
            feed(&mut hash, voxel.kind.as_str().as_bytes());
            feed(&mut hash, &voxel.mass_grams.to_le_bytes());
        }
    }
    ChunkInputDigest(hash)
}

fn chunk_world_coord(chunk: ChunkCoord, local: Coord) -> Coord {
    Coord::new(
        chunk.x * crate::founding_day::CHUNK_SIZE + local.x,
        chunk.y * crate::founding_day::CHUNK_SIZE + local.y,
        chunk.z * crate::founding_day::CHUNK_SIZE + local.z,
    )
}

fn face_vertices(coord: Coord, normal: Coord) -> [MeshVertex; 4] {
    let center = Vec3::new(
        coord.x as f32 + 0.5,
        coord.y as f32 + 0.5,
        coord.z as f32 + 0.5,
    );
    let (hx, hy, hz) = (0.5, 0.5, 0.5);
    let offsets = if normal.x > 0 {
        [
            Vec3::new(hx, -hy, -hz),
            Vec3::new(hx, hy, -hz),
            Vec3::new(hx, hy, hz),
            Vec3::new(hx, -hy, hz),
        ]
    } else if normal.x < 0 {
        [
            Vec3::new(-hx, hy, -hz),
            Vec3::new(-hx, -hy, -hz),
            Vec3::new(-hx, -hy, hz),
            Vec3::new(-hx, hy, hz),
        ]
    } else if normal.y > 0 {
        [
            Vec3::new(-hx, hy, hz),
            Vec3::new(hx, hy, hz),
            Vec3::new(hx, hy, -hz),
            Vec3::new(-hx, hy, -hz),
        ]
    } else if normal.y < 0 {
        [
            Vec3::new(-hx, -hy, -hz),
            Vec3::new(hx, -hy, -hz),
            Vec3::new(hx, -hy, hz),
            Vec3::new(-hx, -hy, hz),
        ]
    } else if normal.z > 0 {
        [
            Vec3::new(-hx, -hy, hz),
            Vec3::new(hx, -hy, hz),
            Vec3::new(hx, hy, hz),
            Vec3::new(-hx, hy, hz),
        ]
    } else {
        [
            Vec3::new(-hx, -hy, -hz),
            Vec3::new(hx, -hy, -hz),
            Vec3::new(hx, hy, -hz),
            Vec3::new(-hx, hy, -hz),
        ]
    };
    let normal = Vec3::new(normal.x as f32, normal.y as f32, normal.z as f32);
    offsets.map(|offset| MeshVertex {
        position: center + offset,
        normal,
    })
}

fn gear_radius(angle: f32, teeth: f32, root: f32, tip: f32) -> f32 {
    let phase = 0.5 + 0.5 * (teeth * angle).cos();
    let profile = phase.powf(0.72);
    root + (tip - root) * profile
}

#[derive(Clone, Copy)]
struct AccentFrame {
    radius: f32,
    length: f32,
    width: f32,
    height: f32,
    angle: f32,
}

fn accent_frame(value: f32) -> AccentFrame {
    let inner = AccentFrame {
        radius: 0.50,
        length: 0.16,
        width: 0.15,
        height: 0.08,
        angle: -0.62,
    };
    let middle = AccentFrame {
        radius: 0.63,
        length: 0.68,
        width: 0.19,
        height: 0.13,
        angle: 0.0,
    };
    let outer = AccentFrame {
        radius: 0.97,
        length: 0.24,
        width: 0.46,
        height: 0.09,
        angle: 0.62,
    };
    let value = value.clamp(ACCENT_MIN, ACCENT_MAX);
    if value <= 0.5 {
        lerp_frame(inner, middle, value * 2.0)
    } else {
        lerp_frame(middle, outer, (value - 0.5) * 2.0)
    }
}

fn lerp_frame(a: AccentFrame, b: AccentFrame, t: f32) -> AccentFrame {
    AccentFrame {
        radius: a.radius + (b.radius - a.radius) * t,
        length: a.length + (b.length - a.length) * t,
        width: a.width + (b.width - a.width) * t,
        height: a.height + (b.height - a.height) * t,
        angle: a.angle + (b.angle - a.angle) * t,
    }
}

fn push_accent_prism(mesh: &mut ShapeMesh, value: f32, body_half_depth: f32) {
    let frame = accent_frame(value);
    let radial = Vec3::new(frame.angle.cos(), frame.angle.sin(), 0.0);
    let tangent = Vec3::new(-radial.y, radial.x, 0.0);
    let center = radial * frame.radius;
    let bottom = Vec3::Z * (body_half_depth - 0.006);
    let top = Vec3::Z * (body_half_depth + frame.height);
    let point = |radial_offset: f32, tangent_offset: f32, z: Vec3| {
        center + radial * radial_offset + tangent * tangent_offset + z
    };
    let points = [
        point(-frame.length * 0.5, -frame.width * 0.5, bottom),
        point(frame.length * 0.5, -frame.width * 0.5, bottom),
        point(frame.length * 0.5, frame.width * 0.5, bottom),
        point(-frame.length * 0.5, frame.width * 0.5, bottom),
        point(-frame.length * 0.5, -frame.width * 0.5, top),
        point(frame.length * 0.5, -frame.width * 0.5, top),
        point(frame.length * 0.5, frame.width * 0.5, top),
        point(-frame.length * 0.5, frame.width * 0.5, top),
    ];
    push_quad(
        mesh,
        points[0],
        points[3],
        points[2],
        points[1],
        Vec3::NEG_Z,
        ShapeMaterial::Accent,
    );
    push_quad(
        mesh,
        points[4],
        points[5],
        points[6],
        points[7],
        Vec3::Z,
        ShapeMaterial::Accent,
    );
    for (a, b, c, d) in [(0, 1, 5, 4), (1, 2, 6, 5), (2, 3, 7, 6), (3, 0, 4, 7)] {
        let normal = triangle_normal(points[a], points[b], points[c]);
        push_quad(
            mesh,
            points[a],
            points[b],
            points[c],
            points[d],
            normal,
            ShapeMaterial::Accent,
        );
    }
}

fn push_quad(
    mesh: &mut ShapeMesh,
    a: Vec3,
    b: Vec3,
    c: Vec3,
    d: Vec3,
    normal: Vec3,
    material: ShapeMaterial,
) {
    mesh.faces.push(ShapeFace {
        positions: [a, b, c, d],
        normal,
        material,
    });
    push_triangle(mesh, a, b, c, Some(normal), material);
    push_triangle(mesh, a, c, d, Some(normal), material);
}

fn triangle_normal(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    let candidate = (b - a).cross(c - a);
    if candidate.length_squared() > 1e-10 {
        candidate.normalize()
    } else {
        Vec3::Z
    }
}

fn push_triangle(
    mesh: &mut ShapeMesh,
    a: Vec3,
    b: Vec3,
    c: Vec3,
    normal: Option<Vec3>,
    material: ShapeMaterial,
) {
    let normal = normal.unwrap_or_else(|| triangle_normal(a, b, c));
    let base = mesh.vertices.len() as u32;
    mesh.vertices.push(ShapeVertex {
        position: a,
        normal,
        material,
    });
    mesh.vertices.push(ShapeVertex {
        position: b,
        normal,
        material,
    });
    mesh.vertices.push(ShapeVertex {
        position: c,
        normal,
        material,
    });
    mesh.indices.extend_from_slice(&[base, base + 1, base + 2]);
}

fn shape_digest(asset: &ShapeAsset) -> u64 {
    let mut hash = fnv1a_seed();
    feed(&mut hash, &[asset.key.asset as u8]);
    feed(&mut hash, &asset.key.builder_hash.0.to_le_bytes());
    for vertex in &asset.mesh.vertices {
        feed_f32(&mut hash, vertex.position.x);
        feed_f32(&mut hash, vertex.position.y);
        feed_f32(&mut hash, vertex.position.z);
        feed_f32(&mut hash, vertex.normal.x);
        feed_f32(&mut hash, vertex.normal.y);
        feed_f32(&mut hash, vertex.normal.z);
        feed(&mut hash, &[vertex.material as u8]);
    }
    hash
}

fn asset_digest(asset: &ChunkAsset) -> u64 {
    let mut hash = fnv1a_seed();
    feed(&mut hash, &asset.key.seed.0.to_le_bytes());
    feed(&mut hash, &asset.key.generator_revision.0.to_le_bytes());
    feed(&mut hash, &asset.key.chunk.x.to_le_bytes());
    feed(&mut hash, &asset.key.chunk.y.to_le_bytes());
    feed(&mut hash, &asset.key.chunk.z.to_le_bytes());
    feed(&mut hash, &asset.key.builder_hash.0.to_le_bytes());
    feed(&mut hash, &asset.key.input_digest.0.to_le_bytes());
    feed(&mut hash, &asset.octree.digest().to_le_bytes());
    for quad in &asset.quads {
        for vertex in &quad.vertices {
            feed_f32(&mut hash, vertex.position.x);
            feed_f32(&mut hash, vertex.position.y);
            feed_f32(&mut hash, vertex.position.z);
            feed_f32(&mut hash, vertex.normal.x);
            feed_f32(&mut hash, vertex.normal.y);
            feed_f32(&mut hash, vertex.normal.z);
        }
    }
    hash
}

pub fn chunk_builder_hash() -> BuilderHash {
    let mut hash = fnv1a_seed();
    feed(&mut hash, CHUNK_RECIPE_SCHEMA);
    BuilderHash(hash)
}

fn fnv1a_seed() -> u64 {
    0xcbf2_9ce4_8422_2325
}

fn feed(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
}

fn feed_f32(hash: &mut u64, value: f32) {
    feed(hash, &value.to_bits().to_le_bytes());
}
