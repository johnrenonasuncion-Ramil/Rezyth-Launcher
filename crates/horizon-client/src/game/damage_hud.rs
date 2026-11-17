//! The Rezyth Settings rows that draw over the world or around the
//! crosshair: the Damage Direction arrow, the Damage Popups floating over
//! hit mobs, and the World Inspector panel for the block under the
//! crosshair. Each draws nothing while its switch is off.
use super::{Game, Screen};
use horizon_client::{
    extras::Corner,
    extras_hud::{self, Corners},
    hud::{HudBatch, rgba},
    mobs::Who,
    volt_features::{damage_popup_style, hud_arrow, project_waypoint},
};
use horizon_core::position::BlockPos;

impl Game<'_> {
    /// The Damage Popups mod: remember a landed melee hit on a mob — its
    /// id, the damage and when — for the floating label.
    pub(super) fn record_damage_popup(&mut self, mob_id: u32, amount: f32) {
        let now = self.world.game_time;
        self.combat.popups.push((mob_id, amount, now));
        if self.combat.popups.len() > 16 {
            self.combat.popups.remove(0);
        }
    }

    /// The Damage Direction arrow, the Damage Popups and the World
    /// Inspector, drawn after the other Rezyth Settings HUD panels.
    pub(super) fn draw_damage_hud(&self, batch: &mut HudBatch, corners: &mut Corners) {
        if self.screen != Screen::Playing {
            return;
        }
        let v = &self.settings.extras.volt_mods;
        let q = &self.settings.extras.volt_qol;
        let now = self.world.game_time;
        let Some((cam, yaw, pitch, fov)) = self.locator_camera else {
            return;
        };
        let atlas = &self.gui.atlas;
        let (w, h) = (batch.width, batch.height);

        // Damage Direction: an arrow around the crosshair pointing at the
        // last mob that hurt the player, fading over two seconds.
        if v.damage_direction_hud
            && let Some((Who::Mob(id), at)) = self.player_hurt_by()
            && now - at <= 40
            && let Some(m) = self.mobs.iter().find(|m| m.id == id)
        {
            let target = [m.position.x, m.position.y + 1.2, m.position.z];
            let p = project_waypoint(cam, yaw, pitch, fov, w, h, target);
            let alpha = (255.0 - (now - at) as f32 * 6.0).max(0.0) as u8;
            let arrow = hud_arrow((w * 0.5, h * 0.5), p.edge_angle, (h * 0.12).min(60.0));
            batch.fill_quad(atlas, arrow, rgba(255, 60, 60, alpha));
        }

        // Damage Popups: the damage of each recent landed hit, floating up
        // over the mob it hit.
        if v.damage_popups {
            for &(id, amount, at) in &self.combat.popups {
                let Some((rise, alpha)) = damage_popup_style(now - at) else {
                    continue;
                };
                let Some(m) = self.mobs.iter().find(|m| m.id == id) else {
                    continue;
                };
                let target = [
                    m.position.x,
                    m.position.y + 1.6 + f64::from(rise),
                    m.position.z,
                ];
                let p = project_waypoint(cam, yaw, pitch, fov, w, h, target);
                if p.on_screen {
                    let label = format!("-{amount:.1}");
                    let tw = HudBatch::text_width(atlas, &label);
                    batch.text(
                        atlas,
                        &label,
                        p.screen_x - tw / 2.0,
                        p.screen_y,
                        rgba(255, 85, 85, alpha),
                        true,
                    );
                }
            }
        }

        // World Inspector: the block under the crosshair — its id, its
        // position, the light and the biome there.
        if q.world_inspector
            && let Some(Some(hit)) = self.tick_hit
        {
            let pos = BlockPos::new(hit.pos[0], hit.pos[1], hit.pos[2]);
            let state = self.world.block(pos.x, pos.y, pos.z);
            let name = self.res.facts().get(state).name.clone();
            let biome = self
                .world
                .chunks
                .get(&(pos.x >> 4, pos.z >> 4))
                .map(|chunk| {
                    let at = &world_biomes!(self).list[chunk.biome(pos.x, pos.y, pos.z) as usize];
                    at.name.clone()
                });
            let mut rows = vec![
                (name.to_string(), u32::MAX),
                (format!("{} {} {}", pos.x, pos.y, pos.z), u32::MAX),
                (
                    format!(
                        "light {} sky {}",
                        i32::from(self.world.light(pos.x, pos.y, pos.z) & 15),
                        i32::from(self.world.light(pos.x, pos.y, pos.z) >> 4)
                    ),
                    u32::MAX,
                ),
            ];
            if let Some(biome) = biome {
                rows.push((biome.to_string(), u32::MAX));
            }
            extras_hud::draw_panel(batch, atlas, corners, Corner::BottomRight, &rows);
        }
    }
}
