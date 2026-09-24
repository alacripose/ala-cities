use ala_cities::asset_generator::{
    AssetKind, BlendMode, DynamicAssetGenerator, MaterialKey, MaterialManifest, OpenPbrMaterial,
    OpticalClass,
};
use ala_cities::founding_day::{ChunkCoord, FoundingWorld, GeneratorRevision, WorldSeed};

fn opaque(key: MaterialKey) -> OpenPbrMaterial {
    OpenPbrMaterial {
        key,
        base_color: [0.5, 0.5, 0.5],
        base_weight: 1.0,
        base_metalness: 0.0,
        specular_roughness: 0.5,
        specular_ior: 1.5,
        geometry_coat_weight: 0.0,
        transmission_weight: 0.0,
        optical_class: OpticalClass::Opaque,
        blend_mode: BlendMode::Opaque,
        alpha: 1.0,
    }
}

fn assert_manifest_is_explicitly_opaque(manifest: &MaterialManifest) {
    assert_eq!(manifest.schema(), "ala-cities/material-manifest/v1");
    assert!(!manifest.materials().is_empty());

    let json = serde_json::to_value(manifest).expect("material manifest serializes");
    assert_eq!(json["schema"], "ala-cities/material-manifest/v1");

    for (index, material) in manifest.materials().iter().enumerate() {
        assert_eq!(material.optical_class, OpticalClass::Opaque);
        assert_eq!(material.blend_mode, BlendMode::Opaque);
        assert_eq!(material.alpha, 1.0);
        assert_eq!(material.base_weight, 1.0);
        assert_eq!(material.transmission_weight, 0.0);
        assert_eq!(
            material.opaque_rgba().expect("opaque color"),
            [
                material.base_color[0],
                material.base_color[1],
                material.base_color[2],
                1.0,
            ]
        );

        let entry = &json["materials"][index];
        assert_eq!(entry["opticalClass"], "opaque");
        assert_eq!(entry["blendMode"], "opaque");
        assert_eq!(entry["alpha"], 1.0);
        assert_eq!(entry["transmissionWeight"], 0.0);
    }
}

#[test]
fn generated_chunk_and_shape_assets_export_opaque_material_manifests() {
    let mut generator = DynamicAssetGenerator::default();
    let shape = generator
        .build_shape(AssetKind::SettingsGear)
        .expect("settings gear builds");
    assert_manifest_is_explicitly_opaque(&shape.material_manifest);
    assert!(shape
        .material_manifest
        .get(MaterialKey::SettingsGearBody)
        .is_some());
    assert!(shape
        .material_manifest
        .get(MaterialKey::SettingsGearAccent)
        .is_some());

    let world = FoundingWorld::new(WorldSeed(7), GeneratorRevision(1));
    let chunk = generator
        .build_chunk(&world, ChunkCoord::new(0, 0, 0))
        .expect("origin chunk builds");
    assert_manifest_is_explicitly_opaque(&chunk.material_manifest);
    for key in [MaterialKey::Soil, MaterialKey::Forage, MaterialKey::Wood] {
        assert!(chunk.material_manifest.get(key).is_some());
    }
}

#[test]
fn semantic_shape_tweaks_cannot_change_material_optics() {
    let mut generator = DynamicAssetGenerator::default();
    let before = generator
        .build_shape(AssetKind::SettingsGear)
        .expect("initial gear builds");
    let before_json =
        serde_json::to_string(&before.material_manifest).expect("manifest serializes");

    assert!(generator.set_control(ala_cities::asset_generator::ShapeControl::Teeth, 11.5,));
    assert!(generator.set_control(ala_cities::asset_generator::ShapeControl::Opening, 0.37,));
    assert!(generator.set_control(ala_cities::asset_generator::ShapeControl::Accent, 0.82,));
    let after = generator
        .build_shape(AssetKind::SettingsGear)
        .expect("tweaked gear builds");

    assert_ne!(before.digest, after.digest);
    assert_eq!(before.material_manifest, after.material_manifest);
    assert_eq!(
        before_json,
        serde_json::to_string(&after.material_manifest).expect("manifest serializes")
    );
}

#[test]
fn a_material_cannot_claim_opaque_while_being_translucent_or_transmissive() {
    let mut translucent = opaque(MaterialKey::Soil);
    translucent.alpha = 0.35;
    assert!(MaterialManifest::new(vec![translucent]).is_err());

    let mut transmissive = opaque(MaterialKey::Soil);
    transmissive.transmission_weight = 0.4;
    assert!(MaterialManifest::new(vec![transmissive]).is_err());

    let mut mislabeled_blend = opaque(MaterialKey::Soil);
    mislabeled_blend.blend_mode = BlendMode::AlphaBlend;
    assert!(MaterialManifest::new(vec![mislabeled_blend]).is_err());
}
