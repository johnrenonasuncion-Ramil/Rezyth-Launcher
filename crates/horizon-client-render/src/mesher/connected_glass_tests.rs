//! The Visuals tab's Connected Glass through the real mesher: with the
//! option on, a glass face's edge that borders more glass insets its texture
//! one texel, so a wall of stained glass shows no doubled frame between two
//! panes. Needs prepared resources, like the other mesher geometry tests.
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

/// Every quad of the section's two buffers, as (u span, v span, first
/// corner).
fn quads(f: &Fixture, chunk: &Chunk, connected: bool) -> Vec<([u16; 2], [f32; 3])> {
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
        connected_glass: connected,
        better_grass_sides: false,
        translucent_layers: &[],
        leaf_culling: LeafCullingMode::Off,
    };
    let mesh = mesher.mesh(&region, 0);
    let solid = mesh.solid.as_chunks::<4>().0;
    let translucent = mesh.translucent.as_chunks::<4>().0;
    let mut out = Vec::new();
    for q in solid.iter().chain(translucent.iter()) {
        let us: Vec<u16> = q.iter().map(|v| v.uv[0]).collect();
        let vs: Vec<u16> = q.iter().map(|v| v.uv[1]).collect();
        let span = |axis: &[u16]| axis.iter().max().unwrap() - axis.iter().min().unwrap();
        out.push(([span(&us), span(&vs)], q[0].pos));
    }
    out
}

#[test]
#[ignore = "requires prepared 26.3 resources (HORIZON_MESH_TEST_RESOURCES or .local/horizon-client/26.3)"]
fn connected_glass_insets_the_frame_between_two_panes() {
    let f = fixture();
    // Two panes side by side along z: different colours, so neither face
    // between them is skipped.
    let mut chunk = Chunk::empty(0, 0);
    let base = MIN_Y + 4;
    chunk.set(8, base, 8, state(&f, "minecraft:white_stained_glass"));
    chunk.set(8, base, 9, state(&f, "minecraft:orange_stained_glass"));
    let full: u16 = round_to_u16(UV_STEPS);
    let one = full - full / 16;
    // Without the option every face keeps its whole sprite.
    let off = quads(&f, &chunk, false);
    assert!(!off.is_empty());
    assert!(
        off.iter().all(|&([du, dv], _)| du == full && dv == full),
        "{off:?}"
    );
    // With it, every face is either whole or exactly one texel short on an
    // axis, and the edges the two panes share inset: the ±x faces of both
    // and their up/down faces toward the other pane, eight in all.
    let on = quads(&f, &chunk, true);
    assert_eq!(on.len(), off.len(), "the option changes no face's presence");
    assert!(
        on.iter()
            .all(|&([du, dv], _)| (du == full || du == one) && (dv == full || dv == one)),
        "{on:?}"
    );
    let inset = on.iter().filter(|&&([du, dv], _)| du != dv).count();
    assert!(inset >= 8, "the panes' shared edges inset: {inset}");
    // The east faces lie on the shared plane x = 9 (the block at x = 8 plus
    // one): each insets the edge toward the other pane.
    let east: Vec<([u16; 2], [f32; 3])> = on
        .iter()
        .copied()
        .filter(|&(_, pos)| pos[0] == 9.0)
        .collect();
    assert!(!east.is_empty());
    assert!(
        east.iter().all(|&([du, dv], _)| du == one && dv == full),
        "{east:?}"
    );
}
