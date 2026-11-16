//! The Rezyth Settings "3D World Waypoints": the saved waypoints drawn in
//! the world (a diamond marker, its beam, its label and an edge arrow for
//! one off screen), their dots on the minimap, and the automatic death
//! marker, over the data in `Settings::volt_waypoints`.
use super::{Game, Screen};
use horizon_client::{
    chat::ChatSource,
    hud::{HudBatch, rgba},
    minimap::{self, View},
    text::Text,
    volt_features::{effective_waypoint_coords, project_waypoint},
};

/// One saved waypoint ready to draw: its name and colour, where it is in
/// this world, its dimension tag when synced, the distance and its own
/// saved coordinates.
struct Marker {
    name: String,
    rgb: [u8; 3],
    target: [f64; 3],
    tag: Option<&'static str>,
    distance: f64,
    coords: [i32; 3],
}

impl Game<'_> {
    /// `3D World Waypoints`: every saved waypoint of this world or server as
    /// an in-world marker — a coloured diamond, a translucent beam, a label
    /// with the distance and the coordinates, and an arrow at the screen's
    /// edge while one is off it.
    pub(super) fn draw_waypoint_markers(&self, batch: &mut HudBatch) {
        let e = &self.settings.extras;
        let (beams, labels, distances, coords, arrows) = (
            e.waypoint_beams,
            e.waypoint_labels,
            e.waypoint_distance,
            e.waypoint_coords,
            e.waypoint_edge_arrows,
        );
        if !e.waypoints
            || !matches!(self.screen, Screen::Playing | Screen::Dead)
            || !(beams || labels || arrows)
        {
            return;
        }
        let Some((cam, yaw, pitch, fov)) = self.locator_camera else {
            return;
        };
        let sync = e.waypoint_dimension_sync;
        let max_distance = f64::from(e.waypoint_max_distance);
        let scale = (f32::from(e.waypoint_scale) / 100.0).max(0.25);
        let markers = self.visible_waypoints(max_distance, sync);
        let (w, h) = (batch.width, batch.height);
        let atlas = &self.gui.atlas;
        for m in markers {
            let p = project_waypoint(cam, yaw, pitch, fov, w, h, m.target);
            let [r, g, b] = m.rgb;
            let (x, y) = (p.screen_x, p.screen_y);
            if p.on_screen {
                if beams {
                    let (top, bottom) = (
                        p.beam_top_y.min(p.beam_bottom_y),
                        p.beam_top_y.max(p.beam_bottom_y),
                    );
                    batch.fill(
                        atlas,
                        x - scale,
                        top,
                        scale * 2.0,
                        bottom - top,
                        rgba(r, g, b, 0x38),
                    );
                }
                let d = 4.0 * scale;
                batch.fill_quad(
                    atlas,
                    [[x, y - d], [x + d, y], [x, y + d], [x - d, y]],
                    rgba(r, g, b, 255),
                );
                if labels {
                    let mut label = m.name;
                    if distances {
                        label.push_str(&format!(" {}m", m.distance.round() as i64));
                    }
                    if coords {
                        label.push_str(&format!(
                            " ({}, {}, {})",
                            m.coords[0], m.coords[1], m.coords[2]
                        ));
                    }
                    if let Some(tag) = m.tag {
                        label.push_str(&format!(" [{tag}]"));
                    }
                    let tw = HudBatch::text_width(atlas, &label);
                    batch.text(
                        atlas,
                        &label,
                        x - tw / 2.0,
                        y + d + 2.0,
                        rgba(255, 255, 255, 255),
                        true,
                    );
                }
            } else if arrows {
                // At the rim, pointing the way.
                let (sin, cos) = p.edge_angle.sin_cos();
                let (ex, ey) = p.edge_pos;
                let (px, py) = (-sin, cos);
                batch.fill_quad(
                    atlas,
                    [
                        [ex + cos * 6.0, ey + sin * 6.0],
                        [ex - cos * 2.0 + px * 4.0, ey - sin * 2.0 + py * 4.0],
                        [ex - cos * 2.0 - px * 4.0, ey - sin * 2.0 - py * 4.0],
                        [ex - cos * 6.0, ey - sin * 6.0],
                    ],
                    rgba(r, g, b, 220),
                );
            }
        }
    }
    /// `3D World Waypoints` on the minimap (`waypoint_on_minimap`): a dot
    /// per waypoint over the map image, at the map's own scale.
    pub(super) fn draw_minimap_waypoints(
        &self,
        batch: &mut HudBatch,
        view: &View,
        x: f32,
        y: f32,
        size: f32,
    ) {
        let e = &self.settings.extras;
        if !e.waypoints || !e.waypoint_on_minimap {
            return;
        }
        let atlas = &self.gui.atlas;
        let scale = size / minimap::SIZE as f32;
        let (cx, cy) = (x + size / 2.0, y + size / 2.0);
        let dot = (3.0 * f32::from(e.waypoint_scale) / 100.0).max(2.0);
        let sync = e.waypoint_dimension_sync;
        let max_distance = f64::from(e.waypoint_max_distance);
        for m in self.visible_waypoints(max_distance, sync) {
            let (ox, oy) = view.map_offset(m.target[0] - view.x, m.target[2] - view.z);
            let (px, py) = (cx + ox * scale, cy + oy * scale);
            if px < x || px > x + size || py < y || py > y + size {
                continue;
            }
            let [r, g, b] = m.rgb;
            batch.fill(
                atlas,
                px - dot / 2.0,
                py - dot / 2.0,
                dot,
                dot,
                rgba(r, g, b, 255),
            );
        }
    }
    /// The Maps tab's Waypoint Button: where it sits `(x, y, side, side)`
    /// while it shows — the waypoint master and button switches on, playing,
    /// the HUD up — else None.
    pub(super) fn waypoint_button_rect(&self) -> Option<(f32, f32, f32, f32)> {
        let e = &self.settings.extras;
        if !e.waypoints
            || !e.waypoint_hud_button
            || self.screen != Screen::Playing
            || self.ui.hide_gui
        {
            return None;
        }
        let (w, h) = self.gui_size();
        Some((w - 28.0, h - 28.0, 20.0, 20.0))
    }
    /// The Maps tab's Waypoint Button: a small marker in the HUD's bottom
    /// right corner that files a waypoint where the player stands.
    pub(super) fn draw_waypoint_button(&self, batch: &mut HudBatch) {
        let Some((x, y, w, h)) = self.waypoint_button_rect() else {
            return;
        };
        let atlas = &self.gui.atlas;
        batch.fill(atlas, x, y, w, h, rgba(0, 0, 0, 0x78));
        let [r, g, b] = self.settings.extras.waypoint_new_color.rgb();
        let (cx, cy) = (x + w / 2.0, y + h / 2.0);
        let d = 5.0;
        batch.fill_quad(
            atlas,
            [[cx, cy - d], [cx + d, cy], [cx, cy + d], [cx - d, cy]],
            rgba(r, g, b, 255),
        );
    }
    /// The Maps tab's Automatic Death Waypoints (marks where the player
    /// fell; the store keeps the newest three deaths of a world or server)
    /// and the Death & Chest Coords Log (the coords to the chat). Both death
    /// paths (`remote.rs`) call this once per death.
    pub(super) fn record_death_waypoint(&mut self) {
        let Some(p) = self.player.as_ref() else {
            return;
        };
        let pos = [
            p.position.x.floor() as i32,
            p.position.y.floor() as i32,
            p.position.z.floor() as i32,
        ];
        // The Death & Chest Coords Log: the coords go to the player's own
        // chat, wherever the waypoint switches stand.
        if self.settings.extras.volt_mods.death_tracker {
            let at = |v: i32| Text::literal(v.to_string());
            let line = Text::translate(
                "horizon.death_tracker.line",
                &[at(pos[0]), at(pos[1]), at(pos[2])],
            );
            self.local_chat_line(line, None, ChatSource::Client);
        }
        let e = &self.settings.extras;
        if !e.waypoints || !e.waypoint_auto_death {
            return;
        }
        let (scope, dimension) = (self.waypoint_scope(), self.dim.id());
        self.settings
            .volt_waypoints
            .record_death(&scope, dimension, pos);
        self.settings.save(&Self::options_path());
    }
    /// The saved waypoints of this world or server that draw now: on, in
    /// range, and with their effective coordinates (the 8x dimension sync).
    fn visible_waypoints(&self, max_distance: f64, sync: bool) -> Vec<Marker> {
        let Some(p) = self.player.as_ref() else {
            return Vec::new();
        };
        let eye = [
            p.position.x,
            p.position.y + f64::from(p.dimensions.eye_height),
            p.position.z,
        ];
        let scope = self.waypoint_scope();
        let mut out = Vec::new();
        for wp in &self.settings.volt_waypoints.entries {
            if !wp.enabled || wp.scope != scope {
                continue;
            }
            let Some((target, tag)) = effective_waypoint_coords(wp, self.dim.id(), sync) else {
                continue;
            };
            let (dx, dy, dz) = (target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]);
            let distance = (dx * dx + dy * dy + dz * dz).sqrt();
            if max_distance > 0.0 && distance > max_distance {
                continue;
            }
            out.push(Marker {
                name: wp.name.clone(),
                rgb: wp.color.rgb(),
                target,
                tag,
                distance,
                coords: [wp.x, wp.y, wp.z],
            });
        }
        out
    }
}
