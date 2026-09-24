//! Deterministic presentation asset generation for the target client and the
//! external asset-builder tool.
//!
//! One generator owns two deliberately separate outputs: sparse world-chunk
//! assets derived from authoritative chunk truth, and asset-scoped shape meshes
//! derived from named presentation recipes. Shape controls never mutate or
//! independently rescale chunk voxels.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use glam::Vec3;
use serde::Serialize;

use crate::founding_day::{Chunk, ChunkCoord, Coord, GeneratorRevision, VoxelKind, WorldSeed};
use crate::octree::SparseVoxelOctree;
use crate::raycast::VoxelHit;
use crate::render::WorldTriangleCuller;

pub const TEETH_MIN: f32 = 6.0;
pub const TEETH_MAX: f32 = 12.0;
pub const OPENING_MIN: f32 = 0.22;
pub const OPENING_MAX: f32 = 0.39;
pub const ACCENT_MIN: f32 = 0.0;
pub const ACCENT_MAX: f32 = 1.0;
pub const RECIPE_QUANTUM: f32 = 1.0 / 1024.0;

const SHAPE_RECIPE_SCHEMA: &[u8] = b"ala-cities/semantic-shape-recipe/v2\0";
const CHUNK_RECIPE_SCHEMA: &[u8] = b"ala-cities/chunk-mesh-recipe/v3\0";
const MATERIAL_MANIFEST_SCHEMA: &str = "ala-cities/material-manifest/v1";

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
    pub material: VoxelKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeshTriangle {
    pub vertices: [MeshVertex; 3],
    pub material: VoxelKind,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChunkMesh {
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,
}

impl ChunkMesh {
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    pub fn triangles(&self) -> impl Iterator<Item = MeshTriangle> + '_ {
        self.indices.as_chunks::<3>().0.iter().map(|indices| {
            let vertices = indices.map(|index| self.vertices[index as usize]);
            MeshTriangle {
                vertices,
                material: vertices[0].material,
            }
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapeMaterial {
    Body,
    Accent,
}

/// Stable presentation identity used by generated manifests and renderers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialKey {
    Soil,
    Forage,
    Wood,
    SettingsGearBody,
    SettingsGearAccent,
}

impl MaterialKey {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Soil => "soil",
            Self::Forage => "forage",
            Self::Wood => "wood",
            Self::SettingsGearBody => "settings-gear:body",
            Self::SettingsGearAccent => "settings-gear:accent",
        }
    }
}

impl From<VoxelKind> for MaterialKey {
    fn from(kind: VoxelKind) -> Self {
        match kind {
            VoxelKind::Soil => Self::Soil,
            VoxelKind::Forage => Self::Forage,
            VoxelKind::Wood => Self::Wood,
        }
    }
}

impl From<ShapeMaterial> for MaterialKey {
    fn from(material: ShapeMaterial) -> Self {
        match material {
            ShapeMaterial::Body => Self::SettingsGearBody,
            ShapeMaterial::Accent => Self::SettingsGearAccent,
        }
    }
}

/// Optical behaviour is explicit. A viewport tint may change colour, but it never
/// changes this classification or any of the alpha/transmission invariants.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OpticalClass {
    Opaque,
    Translucent,
    Transmissive,
}

impl OpticalClass {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Opaque => "opaque",
            Self::Translucent => "translucent",
            Self::Transmissive => "transmissive",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BlendMode {
    Opaque,
    AlphaBlend,
    Transmission,
}

impl BlendMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Opaque => "opaque",
            Self::AlphaBlend => "alpha_blend",
            Self::Transmission => "transmission",
        }
    }
}

/// OpenPBR-ready material data plus the runtime optical classification needed to
/// keep opacity and transmission from contradicting one another.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenPbrMaterial {
    pub key: MaterialKey,
    pub base_color: [f32; 3],
    pub base_weight: f32,
    pub base_metalness: f32,
    pub specular_roughness: f32,
    pub specular_ior: f32,
    pub geometry_coat_weight: f32,
    pub transmission_weight: f32,
    pub optical_class: OpticalClass,
    pub blend_mode: BlendMode,
    pub alpha: f32,
}

impl OpenPbrMaterial {
    pub fn opaque_rgba(self) -> Option<[f32; 4]> {
        (self.optical_class == OpticalClass::Opaque && self.blend_mode == BlendMode::Opaque)
            .then_some([
                self.base_color[0],
                self.base_color[1],
                self.base_color[2],
                self.alpha,
            ])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaterialManifestError {
    DuplicateKey(MaterialKey),
    InvalidField {
        key: MaterialKey,
        field: &'static str,
        problem: &'static str,
    },
    OpticalClassMismatch(MaterialKey),
}

impl fmt::Display for MaterialManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateKey(key) => {
                write!(formatter, "duplicate material key `{}`", key.as_str())
            }
            Self::InvalidField {
                key,
                field,
                problem,
            } => write!(
                formatter,
                "material `{}` has invalid {field}: {problem}",
                key.as_str()
            ),
            Self::OpticalClassMismatch(key) => write!(
                formatter,
                "material `{}` contradicts its optical class, alpha, transmission, or blend mode",
                key.as_str()
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialManifest {
    schema: &'static str,
    materials: Vec<OpenPbrMaterial>,
}

impl MaterialManifest {
    pub fn new(materials: Vec<OpenPbrMaterial>) -> Result<Self, MaterialManifestError> {
        let mut keys = BTreeSet::new();
        for material in &materials {
            validate_material(material)?;
            if !keys.insert(material.key) {
                return Err(MaterialManifestError::DuplicateKey(material.key));
            }
        }
        Ok(Self {
            schema: MATERIAL_MANIFEST_SCHEMA,
            materials,
        })
    }

    pub fn schema(&self) -> &'static str {
        self.schema
    }

    pub fn materials(&self) -> &[OpenPbrMaterial] {
        &self.materials
    }

    pub fn get(&self, key: MaterialKey) -> Option<&OpenPbrMaterial> {
        self.materials.iter().find(|material| material.key == key)
    }

    pub fn digest(&self) -> u64 {
        let mut hash = fnv1a_seed();
        feed(&mut hash, self.schema.as_bytes());
        for material in &self.materials {
            feed(&mut hash, material.key.as_str().as_bytes());
            for channel in material.base_color {
                feed_f32(&mut hash, channel);
            }
            for value in [
                material.base_weight,
                material.base_metalness,
                material.specular_roughness,
                material.specular_ior,
                material.geometry_coat_weight,
                material.transmission_weight,
                material.alpha,
            ] {
                feed_f32(&mut hash, value);
            }
            feed(&mut hash, material.optical_class.as_str().as_bytes());
            feed(&mut hash, material.blend_mode.as_str().as_bytes());
        }
        hash
    }
}

fn validate_material(material: &OpenPbrMaterial) -> Result<(), MaterialManifestError> {
    let invalid = |field, problem| MaterialManifestError::InvalidField {
        key: material.key,
        field,
        problem,
    };
    for (field, value) in [
        ("baseWeight", material.base_weight),
        ("baseMetalness", material.base_metalness),
        ("specularRoughness", material.specular_roughness),
        ("specularIor", material.specular_ior),
        ("geometryCoatWeight", material.geometry_coat_weight),
        ("transmissionWeight", material.transmission_weight),
        ("alpha", material.alpha),
    ] {
        if !value.is_finite() {
            return Err(invalid(field, "value must be finite"));
        }
    }
    for (index, channel) in material.base_color.into_iter().enumerate() {
        if !channel.is_finite() || !(0.0..=1.0).contains(&channel) {
            return Err(invalid(
                match index {
                    0 => "baseColorR",
                    1 => "baseColorG",
                    _ => "baseColorB",
                },
                "linear RGB channel must be within 0..=1",
            ));
        }
    }
    for (field, value) in [
        ("baseWeight", material.base_weight),
        ("baseMetalness", material.base_metalness),
        ("specularRoughness", material.specular_roughness),
        ("geometryCoatWeight", material.geometry_coat_weight),
        ("transmissionWeight", material.transmission_weight),
        ("alpha", material.alpha),
    ] {
        if !(0.0..=1.0).contains(&value) {
            return Err(invalid(field, "value must be within 0..=1"));
        }
    }
    if material.specular_ior <= 0.0 {
        return Err(invalid("specularIor", "IOR must be greater than zero"));
    }

    let matches = match material.optical_class {
        OpticalClass::Opaque => {
            material.alpha == 1.0
                && material.base_weight == 1.0
                && material.transmission_weight == 0.0
                && material.blend_mode == BlendMode::Opaque
        }
        OpticalClass::Translucent => {
            material.alpha > 0.0
                && material.alpha < 1.0
                && material.base_weight == material.alpha
                && material.transmission_weight == 0.0
                && material.blend_mode == BlendMode::AlphaBlend
        }
        OpticalClass::Transmissive => {
            material.alpha == 1.0
                && material.base_weight == 1.0
                && material.transmission_weight > 0.0
                && material.blend_mode == BlendMode::Transmission
        }
    };
    if matches {
        Ok(())
    } else {
        Err(MaterialManifestError::OpticalClassMismatch(material.key))
    }
}

fn chunk_material_manifest() -> MaterialManifest {
    MaterialManifest::new(vec![
        OpenPbrMaterial {
            key: MaterialKey::Soil,
            base_color: [0.40, 0.31, 0.22],
            base_weight: 1.0,
            base_metalness: 0.0,
            specular_roughness: 0.92,
            specular_ior: 1.45,
            geometry_coat_weight: 0.0,
            transmission_weight: 0.0,
            optical_class: OpticalClass::Opaque,
            blend_mode: BlendMode::Opaque,
            alpha: 1.0,
        },
        OpenPbrMaterial {
            key: MaterialKey::Forage,
            base_color: [0.34, 0.72, 0.31],
            base_weight: 1.0,
            base_metalness: 0.0,
            specular_roughness: 0.88,
            specular_ior: 1.40,
            geometry_coat_weight: 0.0,
            transmission_weight: 0.0,
            optical_class: OpticalClass::Opaque,
            blend_mode: BlendMode::Opaque,
            alpha: 1.0,
        },
        OpenPbrMaterial {
            key: MaterialKey::Wood,
            base_color: [0.55, 0.31, 0.16],
            base_weight: 1.0,
            base_metalness: 0.0,
            specular_roughness: 0.72,
            specular_ior: 1.47,
            geometry_coat_weight: 0.0,
            transmission_weight: 0.0,
            optical_class: OpticalClass::Opaque,
            blend_mode: BlendMode::Opaque,
            alpha: 1.0,
        },
    ])
    .expect("built-in chunk material manifest is valid")
}

fn shape_material_manifest() -> MaterialManifest {
    MaterialManifest::new(vec![
        OpenPbrMaterial {
            key: MaterialKey::SettingsGearBody,
            base_color: [0.66, 0.72, 0.76],
            base_weight: 1.0,
            base_metalness: 0.90,
            specular_roughness: 0.28,
            specular_ior: 1.50,
            geometry_coat_weight: 0.35,
            transmission_weight: 0.0,
            optical_class: OpticalClass::Opaque,
            blend_mode: BlendMode::Opaque,
            alpha: 1.0,
        },
        OpenPbrMaterial {
            key: MaterialKey::SettingsGearAccent,
            base_color: [0.95, 0.46, 0.16],
            base_weight: 1.0,
            base_metalness: 0.0,
            specular_roughness: 0.24,
            specular_ior: 1.50,
            geometry_coat_weight: 0.45,
            transmission_weight: 0.0,
            optical_class: OpticalClass::Opaque,
            blend_mode: BlendMode::Opaque,
            alpha: 1.0,
        },
    ])
    .expect("built-in shape material manifest is valid")
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShapeVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub material: ShapeMaterial,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShapeTriangle {
    pub vertices: [ShapeVertex; 3],
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShapeMesh {
    pub vertices: Vec<ShapeVertex>,
    pub indices: Vec<u32>,
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

    pub fn triangles(&self) -> impl Iterator<Item = ShapeTriangle> + '_ {
        self.indices
            .as_chunks::<3>()
            .0
            .iter()
            .map(|indices| ShapeTriangle {
                vertices: indices.map(|index| self.vertices[index as usize]),
            })
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

            push_surface(
                &mut mesh,
                outer0 + front,
                outer1 + front,
                inner1 + front,
                inner0 + front,
                Vec3::Z,
                ShapeMaterial::Body,
            );
            push_surface(
                &mut mesh,
                outer1 + back,
                outer0 + back,
                inner0 + back,
                inner1 + back,
                Vec3::NEG_Z,
                ShapeMaterial::Body,
            );

            let wall_normal = (direction0 + direction1).normalize();
            push_surface(
                &mut mesh,
                outer0 + front,
                outer0 + back,
                outer1 + back,
                outer1 + front,
                wall_normal,
                ShapeMaterial::Body,
            );
            push_surface(
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
    pub material_manifest: MaterialManifest,
    pub digest: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChunkAsset {
    pub key: ChunkAssetKey,
    pub octree: SparseVoxelOctree,
    pub mesh: ChunkMesh,
    pub material_manifest: MaterialManifest,
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

    /// Triangles retained by conservative backface and camera-volume culling.
    pub fn visible_triangles(
        &self,
        culler: WorldTriangleCuller,
    ) -> impl Iterator<Item = MeshTriangle> + '_ {
        self.mesh.triangles().filter(move |triangle| {
            culler.accepts_triangle(
                triangle.vertices.map(|vertex| vertex.position),
                triangle.vertices[0].normal,
            )
        })
    }

    /// Retained indices into the original indexed mesh, preserving shared edges.
    pub fn visible_triangle_indices(&self, culler: WorldTriangleCuller) -> Vec<u32> {
        let mut indices = Vec::new();
        for triangle in self.mesh.indices.as_chunks::<3>().0 {
            let vertices = triangle.map(|index| self.mesh.vertices[index as usize]);
            let mesh_triangle = MeshTriangle {
                vertices,
                material: vertices[0].material,
            };
            if culler.accepts_triangle(
                mesh_triangle.vertices.map(|vertex| vertex.position),
                mesh_triangle.vertices[0].normal,
            ) {
                indices.extend_from_slice(triangle);
            }
        }
        indices
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
            material_manifest: shape_material_manifest(),
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
        for (local, voxel) in &chunk.voxels {
            octree.insert(*local, voxel.kind);
        }
        let mesh = greedy_visible_mesh(world, chunk_coord, chunk);

        let mut asset = ChunkAsset {
            key,
            octree,
            mesh,
            material_manifest: chunk_material_manifest(),
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

fn greedy_visible_mesh(
    world: &crate::founding_day::FoundingWorld,
    chunk_coord: ChunkCoord,
    chunk: &Chunk,
) -> ChunkMesh {
    const N: usize = crate::founding_day::CHUNK_SIZE as usize;
    let mut mesh = ChunkMesh::default();

    for normal_axis in 0..3 {
        let u_axis = (normal_axis + 1) % 3;
        let v_axis = (normal_axis + 2) % 3;
        for sign in [-1, 1] {
            let normal = normal_coord(normal_axis, sign);
            for slice in 0..N {
                let mut mask = vec![None::<VoxelKind>; N * N];
                for v in 0..N {
                    for u in 0..N {
                        let local = local_coord(normal_axis, slice, u_axis, u, v_axis, v);
                        let Some(voxel) = chunk.voxels.get(&local) else {
                            continue;
                        };
                        let coord = chunk_world_coord(chunk_coord, local);
                        let neighbour =
                            Coord::new(coord.x + normal.x, coord.y + normal.y, coord.z + normal.z);
                        if world.voxel_at(neighbour).is_none() {
                            mask[mask_index(u, v)] = Some(voxel.kind);
                        }
                    }
                }

                let mut v = 0;
                while v < N {
                    let mut u = 0;
                    while u < N {
                        let Some(material) = mask[mask_index(u, v)] else {
                            u += 1;
                            continue;
                        };
                        let mut width = 1;
                        while u + width < N && mask[mask_index(u + width, v)] == Some(material) {
                            width += 1;
                        }
                        let mut height = 1;
                        while v + height < N
                            && (0..width).all(|column| {
                                mask[mask_index(u + column, v + height)] == Some(material)
                            })
                        {
                            height += 1;
                        }

                        append_face_rectangle(
                            &mut mesh,
                            FaceRectangle {
                                chunk: chunk_coord,
                                axes: [normal_axis, u_axis, v_axis],
                                slice,
                                sign,
                                origin: [u, v],
                                size: [width, height],
                                material,
                            },
                        );
                        for row in v..v + height {
                            for column in u..u + width {
                                mask[mask_index(column, row)] = None;
                            }
                        }
                        u += width;
                    }
                    v += 1;
                }
            }
        }
    }
    mesh
}

fn mask_index(u: usize, v: usize) -> usize {
    u + v * crate::founding_day::CHUNK_SIZE as usize
}

fn local_coord(
    normal_axis: usize,
    slice: usize,
    u_axis: usize,
    u: usize,
    v_axis: usize,
    v: usize,
) -> Coord {
    let mut values = [0; 3];
    values[normal_axis] = slice as i32;
    values[u_axis] = u as i32;
    values[v_axis] = v as i32;
    Coord::new(values[0], values[1], values[2])
}

fn normal_coord(axis: usize, sign: i32) -> Coord {
    match axis {
        0 => Coord::new(sign, 0, 0),
        1 => Coord::new(0, sign, 0),
        _ => Coord::new(0, 0, sign),
    }
}

struct FaceRectangle {
    chunk: ChunkCoord,
    axes: [usize; 3],
    slice: usize,
    sign: i32,
    origin: [usize; 2],
    size: [usize; 2],
    material: VoxelKind,
}

fn append_face_rectangle(mesh: &mut ChunkMesh, rectangle: FaceRectangle) {
    let FaceRectangle {
        chunk,
        axes,
        slice,
        sign,
        origin: face_origin,
        size: face_size,
        material,
    } = rectangle;
    let [normal_axis, u_axis, v_axis] = axes;
    let [u, v] = face_origin;
    let [width, height] = face_size;
    let mut origin = [0.0_f32; 3];
    origin[0] = (chunk.x * crate::founding_day::CHUNK_SIZE) as f32;
    origin[1] = (chunk.y * crate::founding_day::CHUNK_SIZE) as f32;
    origin[2] = (chunk.z * crate::founding_day::CHUNK_SIZE) as f32;
    origin[normal_axis] += slice as f32 + if sign > 0 { 1.0 } else { 0.0 };
    origin[u_axis] += u as f32;
    origin[v_axis] += v as f32;
    let origin = Vec3::new(origin[0], origin[1], origin[2]);
    let across = axis_vector(u_axis) * width as f32;
    let up = axis_vector(v_axis) * height as f32;
    let normal = axis_vector(normal_axis) * sign as f32;
    let positions = if sign > 0 {
        [origin, origin + across, origin + across + up, origin + up]
    } else {
        [origin, origin + up, origin + across + up, origin + across]
    };
    let base = mesh.vertices.len() as u32;
    mesh.vertices
        .extend(positions.into_iter().map(|position| MeshVertex {
            position,
            normal,
            material,
        }));
    mesh.indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

fn axis_vector(axis: usize) -> Vec3 {
    match axis {
        0 => Vec3::X,
        1 => Vec3::Y,
        _ => Vec3::Z,
    }
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
    push_surface(
        mesh,
        points[0],
        points[3],
        points[2],
        points[1],
        Vec3::NEG_Z,
        ShapeMaterial::Accent,
    );
    push_surface(
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
        push_surface(
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

fn push_surface(
    mesh: &mut ShapeMesh,
    a: Vec3,
    b: Vec3,
    c: Vec3,
    d: Vec3,
    normal: Vec3,
    material: ShapeMaterial,
) {
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
    feed(&mut hash, &asset.material_manifest.digest().to_le_bytes());
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
    feed(&mut hash, &asset.material_manifest.digest().to_le_bytes());
    feed(&mut hash, &asset.octree.digest().to_le_bytes());
    for vertex in &asset.mesh.vertices {
        feed_f32(&mut hash, vertex.position.x);
        feed_f32(&mut hash, vertex.position.y);
        feed_f32(&mut hash, vertex.position.z);
        feed_f32(&mut hash, vertex.normal.x);
        feed_f32(&mut hash, vertex.normal.y);
        feed_f32(&mut hash, vertex.normal.z);
        feed(&mut hash, &[vertex.material as u8]);
    }
    for index in &asset.mesh.indices {
        feed(&mut hash, &index.to_le_bytes());
    }
    hash
}

pub fn chunk_builder_hash() -> BuilderHash {
    let mut hash = fnv1a_seed();
    feed(&mut hash, CHUNK_RECIPE_SCHEMA);
    feed(&mut hash, &chunk_material_manifest().digest().to_le_bytes());
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
