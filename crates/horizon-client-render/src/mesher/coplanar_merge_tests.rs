//! The Boost tab's Coplanar Quad Merge through the real mesher: two
//! same-block glass blocks each draw a face at their shared plane — with the
//! option on, vanilla's `SkipRendering::SameBlock` hides the pair. Needs
//! prepared resources, like the other mesher geometry tests.
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

/// The section's quad count, with the merge on or off.
fn quad_count(f: &Fixture, chunk: &Chunk, merge: bool) -> usize {
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
        better_grass_sides: false,
        greedy_quad_merge: merge,
        translucent_layers: &[],
        leaf_culling: LeafCullingMode::Off,
    };
    let mesh = mesher.mesh(&region, 0);
    mesh.solid.as_chunks::<4>().0.len() + mesh.translucent.as_chunks::<4>().0.len()
}

#[test]
#[ignore = "requires prepared 26.3 resources (HORIZON_MESH_TEST_RESOURCES or .local/horizon-client/26.3)"]
fn the_coplanar_merge_hides_same_block_glass_faces() {
    let f = fixture();
    let mut chunk = Chunk::empty(0, 0);
    let base = MIN_Y + 4;
    chunk.set(8, base, 8, state(&f, "minecraft:glass"));
    chunk.set(9, base, 8, state(&f, "minecraft:glass"));
    assert_eq!(
        quad_count(&f, &chunk, false),
        12,
        "two cubes, six quads each"
    );
    assert_eq!(quad_count(&f, &chunk, true), 10, "the shared pair hides");
}
