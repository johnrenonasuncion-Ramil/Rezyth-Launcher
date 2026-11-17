//! The Rezyth Settings world overlays: the Trajectory mod's predicted arc
//! gizmo, the Spawn Light overlay's boxes at dark spawnable spots, and the
//! Slime Tech HUD panel. Each draws nothing while its switch is off.
use super::{Game, Screen};
use horizon_client::{
    blocks::Fluid,
    extras::Corner,
    extras_hud::{self, Corners},
    gizmos::Gizmos,
    hud::HudBatch,
    volt_features::{slime_spawns_in_chunk, spawn_light_color, throw_direction, trajectory_points},
};
use horizon_core::position::BlockPos;

impl Game<'_> {
    /// The Trajectory mod: the predicted arc of the held throwable, as line
    /// segments from the eye along the throw (speed 1.5, gravity 0.03, as
    /// `throw_item` shoots it, without the random spread).
    pub(super) fn trajectory_gizmo(&self, g: &mut Gizmos) {
        if !self.settings.extras.volt_mods.trajectory_arc {
            return;
        }
        let Some(stack) = self.held() else {
            return;
        };
        let item = &*stack.item;
        let throwable = matches!(
            item,
            "minecraft:snowball"
                | "minecraft:egg"
                | "minecraft:blue_egg"
                | "minecraft:brown_egg"
                | "minecraft:ender_pearl"
        ) || horizon_client::projectiles::thrown_by_item(item).is_some();
        if !throwable {
            return;
        }
        let Some(eye) = self.eye_position() else {
            return;
        };
        let dir = throw_direction(self.yaw, self.pitch);
        let mv = self.known_player_movement();
        let on_ground = self.player.as_ref().map_or(true, |p| p.ground.on_ground);
        let v = [
            dir[0] * 1.5 + mv.x,
            dir[1] * 1.5 + if on_ground { 0.0 } else { mv.y },
            dir[2] * 1.5 + mv.z,
        ];
        let start = [eye[0], eye[1] - 0.1, eye[2]];
        let facts = self.res.facts();
        let points = trajectory_points(start, v, 0.03, |b| {
            let f = facts.get(self.block_state(BlockPos::new(b[0], b[1], b[2])));
            !f.is_air && f.fluid == Fluid::None
        });
        for w in points.windows(2) {
            g.line(w[0], w[1], 0xFF66_FF66, 2.0);
        }
    }

    /// The Spawn Light overlay: a box at every spawnable position near the
    /// player whose block light is 7 or less — red at 0, where monsters
    /// spawn in the overworld, amber above it.
    pub(super) fn spawn_light_gizmo(&self, g: &mut Gizmos) {
        if !self.settings.extras.volt_qol.spawn_light_overlay {
            return;
        }
        let Some(p) = self.player.as_ref() else {
            return;
        };
        let (px, py, pz) = (
            p.position.x.floor() as i32,
            p.position.y.floor() as i32,
            p.position.z.floor() as i32,
        );
        let facts = self.res.facts();
        let at = |x: i32, y: i32, z: i32| facts.get(self.block_state(BlockPos::new(x, y, z)));
        for dx in -8..=8 {
            for dz in -8..=8 {
                for dy in -4..=4 {
                    let (x, y, z) = (px + dx, py + dy, pz + dz);
                    let below = at(x, y - 1, z);
                    if below.is_air || below.fluid != Fluid::None {
                        continue;
                    }
                    let here = at(x, y, z);
                    let above = at(x, y + 1, z);
                    if here.full_collision
                        || here.fluid != Fluid::None
                        || above.full_collision
                        || above.fluid != Fluid::None
                    {
                        continue;
                    }
                    let Some(color) = spawn_light_color(self.block_light(BlockPos::new(x, y, z)))
                    else {
                        continue;
                    };
                    g.cuboid(
                        [f64::from(x), f64::from(y), f64::from(z)],
                        [
                            f64::from(x + 1),
                            f64::from(y + 1),
                            f64::from(z + 1),
                        ],
                        color,
                        1.5,
                    );
                }
            }
        }
    }

    /// The Slime Tech mod: a panel telling whether the chunk the player
    /// stands in is a slime chunk (slimes spawn in it below y 40).
    pub(super) fn draw_slime_hud(&self, batch: &mut HudBatch, corners: &mut Corners) {
        if self.screen != Screen::Playing || !self.settings.extras.volt_mods.slime_tech_hud {
            return;
        }
        let Some(p) = self.player.as_ref() else {
            return;
        };
        let (cx, cz) = (
            (p.position.x.floor() as i32) >> 4,
            (p.position.z.floor() as i32) >> 4,
        );
        let y = p.position.y.floor() as i32;
        let slime = slime_spawns_in_chunk(cx, cz, y, self.world_seed());
        let rows = [
            (
                format!("Slime chunk: {}", if slime { "YES" } else { "no" }),
                if slime { 0xFF55_FF55 } else { u32::MAX },
            ),
            (format!("chunk {} {} y {}", cx, cz, y), u32::MAX),
        ];
        extras_hud::draw_panel(batch, &self.gui.atlas, corners, Corner::TopRight, &rows);
    }
}
