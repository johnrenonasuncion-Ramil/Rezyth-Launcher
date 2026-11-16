//! The Visuals tab's Better Grass Sides through the real mesher: with the
//! option on, a grass block's side faces wear the tinted top texture, so a
//! slope reads green from the side. Needs prepared resources, like the other
//! mesher geometry tests.
use super::*;
use crate::{
    assets::{Assets, RenderAssets},
    biome::Biomes,
    blocks::BlockFacts,
    chunk::Chunk,
    graphics::LeafCullingMode,
    models,
};

struct Fixture {
    assets: Assets,
    biomes: Biomes,
    render: RenderAssets,
    visibility: horizon_world::block_render::FaceVisibility,
    tables: MeshTables,
}

fn fixture() -> Fixture {
    let root = std::env::var_os("HORIZON_MESH_TEST_RESOURCES")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../.local/horizon-client/26.3")
        });
    let assets = Assets::load(&root).unwrap();
    let facts = BlockFacts::load(&root, assets.registry).unwrap();
    let biomes = Biomes::load(&root).unwrap();
    let render = assets.reload(&root).unwrap();
    let visibility = models::load_visibility(&root, assets.registry).unwrap();
    let tables = MeshTables::new(&assets, &render, &facts, visibility.clone()).unwrap();
    Fixture {
        assets,
        biomes,
        render,
        visibility,
        tables,
    }
}

fn state(f: &Fixture, name: &str) -> u32 {
    f.assets.registry.default_state(name).unwrap().0
}

/// The meshed section's bare sprite layers (without the opaque-layer flag)
/// of the up face and of the north face.
fn face_layers(f: &Fixture, chunk: &Chunk, better: bool) -> (Vec<u16>, Vec<u16>) {
    let region = [[Some(chunk); 3]; 3];
    let mut mesher = Mesher {
        nether_shading: false,
        shader_pack_lighting: false,
        separate_ao: false,
        assets: &f.assets,
        render: &f.render,
        tables: &f.tables,
        biomes: &f.biomes,
        visibility: f.visibility.clone(),
        zoom: BiomeZoom::new(0),
        blend_radius: 0,
        smooth_lighting: false,
        cutout_leaves: true,
        connected_glass: false,
        better_grass_sides: better,
        translucent_layers: &[],
        leaf_culling: LeafCullingMode::Off,
    };
    let mesh = mesher.mesh(&region, 0);
    let solid = mesh.solid.as_chunks::<4>().0;
    let translucent = mesh.translucent.as_chunks::<4>().0;
    let mut up = Vec::new();
    let mut north = Vec::new();
    let base = (MIN_Y + 4) as f32;
    for q in solid.iter().chain(translucent.iter()) {
        let layer = q[0].layer & !OPAQUE_LAYER;
        if q.iter().all(|v| v.pos[1] == base + 1.0) {
            up.push(layer);
        }
        if q.iter().all(|v| v.pos[2] == 8.0) {
            north.push(layer);
        }
    }
    (up, north)
}

#[test]
#[ignore = "requires prepared 26.3 resources (HORIZON_MESH_TEST_RESOURCES or .local/horizon-client/26.3)"]
fn better_grass_sides_wear_the_tinted_top_texture() {
    let f = fixture();
    let mut chunk = Chunk::empty(0, 0);
    chunk.set(8, MIN_Y + 4, 8, state(&f, "minecraft:grass_block"));
    let (up_off, north_off) = face_layers(&f, &chunk, false);
    assert!(!up_off.is_empty(), "the block meshed");
    assert!(!north_off.is_empty(), "the side meshed");
    let grass_top = up_off[0];
    // Without the option no side quad wears the top's sprite (the sides are
    // dirt with the grass fringe overlay).
    assert!(!north_off.contains(&grass_top), "{north_off:?}");
    // With it, the side's base quad wears the top's sprite, so the slope
    // reads green from the side; the overlay quad stays the fringe.
    let (up_on, north_on) = face_layers(&f, &chunk, true);
    assert_eq!(up_on[0], grass_top);
    assert!(north_on.contains(&grass_top), "{north_on:?}");
    assert_eq!(north_on.len(), north_off.len(), "no quad appears or goes");
}
