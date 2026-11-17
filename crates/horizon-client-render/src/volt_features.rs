//! Volt Settings feature engines:
//! 1. **Volt Waypoints** (`WaypointStore`, 3D world-to-screen projection, 8x
//!    Overworld <-> Nether coordinate sync, automatic death waypoints).
//! 2. **Volt Minimap** (`MinimapFrame`, `MinimapTerrainMode`, `MinimapEntities`,
//!    radar entity classification, world-to-minimap projection, HUD readouts).
//! 3. **Volt Boost** (Tier 1, Tier 2 & Tier 3 performance & optimization types).
//!
//! Every feature is **off by default** in `extras::Extras`.

use crate::{camera::view_matrix, minimap::View};
use glam::Vec3 as GlamVec3;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

pub const WAYPOINTS_FILE: &str = "volt-waypoints.json";

/// Vivid preset colours for Volt Waypoints.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaypointColor {
    #[default]
    Crimson,
    Gold,
    Emerald,
    Aqua,
    Sky,
    Violet,
    Coral,
    Snow,
}

impl WaypointColor {
    pub const ALL: [Self; 8] = [
        Self::Crimson,
        Self::Gold,
        Self::Emerald,
        Self::Aqua,
        Self::Sky,
        Self::Violet,
        Self::Coral,
        Self::Snow,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Crimson => "Crimson",
            Self::Gold => "Gold",
            Self::Emerald => "Emerald",
            Self::Aqua => "Aqua",
            Self::Sky => "Sky",
            Self::Violet => "Violet",
            Self::Coral => "Coral",
            Self::Snow => "White",
        }
    }

    pub fn rgb(self) -> [u8; 3] {
        match self {
            Self::Crimson => [239, 68, 68],
            Self::Gold => [245, 158, 11],
            Self::Emerald => [16, 185, 129],
            Self::Aqua => [6, 182, 212],
            Self::Sky => [59, 130, 246],
            Self::Violet => [168, 85, 247],
            Self::Coral => [244, 63, 94],
            Self::Snow => [248, 250, 252],
        }
    }

    pub fn cycle(self, back: bool) -> Self {
        let idx = Self::ALL.iter().position(|&c| c == self).unwrap_or(0);
        let next = if back {
            (idx + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (idx + 1) % Self::ALL.len()
        };
        Self::ALL[next]
    }
}

/// One saved Volt Waypoint, scoped to a singleplayer world or multiplayer server.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Waypoint {
    pub id: u64,
    pub scope: String,
    pub name: String,
    pub dimension: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    #[serde(default)]
    pub color: WaypointColor,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub is_death: bool,
}

fn default_true() -> bool {
    true
}

/// Persistent collection of Volt Waypoints (`volt-waypoints.json`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaypointStore {
    #[serde(default)]
    pub next_id: u64,
    #[serde(default)]
    pub entries: Vec<Waypoint>,
}

impl WaypointStore {
    pub fn load(dir: &Path) -> Self {
        fs::read_to_string(dir.join(WAYPOINTS_FILE))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, dir: &Path) {
        let path = dir.join(WAYPOINTS_FILE);
        if self.entries.is_empty() && !path.exists() {
            return;
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }

    fn alloc_id(&mut self) -> u64 {
        self.next_id = self
            .next_id
            .max(self.entries.iter().map(|w| w.id).max().unwrap_or(0))
            + 1;
        self.next_id
    }

    pub fn add(
        &mut self,
        scope: &str,
        name: &str,
        dimension: &str,
        pos: [i32; 3],
        color: WaypointColor,
    ) -> u64 {
        let id = self.alloc_id();
        let trimmed = name.trim();
        let final_name = if trimmed.is_empty() {
            let count = self
                .entries
                .iter()
                .filter(|w| w.scope == scope && !w.is_death)
                .count()
                + 1;
            format!("Waypoint {count}")
        } else {
            trimmed.chars().take(28).collect()
        };
        self.entries.push(Waypoint {
            id,
            scope: scope.to_owned(),
            name: final_name,
            dimension: dimension.to_owned(),
            x: pos[0],
            y: pos[1],
            z: pos[2],
            color,
            enabled: true,
            is_death: false,
        });
        id
    }

    /// Records or updates the player's latest death waypoint in `scope`,
    /// keeping at most 3 death markers per world/server.
    pub fn record_death(&mut self, scope: &str, dimension: &str, pos: [i32; 3]) -> u64 {
        for wp in &mut self.entries {
            if wp.scope == scope && wp.is_death && wp.name == "Last Death" {
                wp.name = format!("Death ({}, {}, {})", wp.x, wp.y, wp.z);
            }
        }
        let death_indices: Vec<usize> = self
            .entries
            .iter()
            .enumerate()
            .filter_map(|(i, w)| (w.scope == scope && w.is_death).then_some(i))
            .collect();
        if death_indices.len() >= 3 {
            let remove_count = death_indices.len() - 2;
            for &idx in death_indices.iter().take(remove_count).rev() {
                self.entries.remove(idx);
            }
        }
        let id = self.alloc_id();
        self.entries.push(Waypoint {
            id,
            scope: scope.to_owned(),
            name: "Last Death".to_owned(),
            dimension: dimension.to_owned(),
            x: pos[0],
            y: pos[1],
            z: pos[2],
            color: WaypointColor::Crimson,
            enabled: true,
            is_death: true,
        });
        id
    }

    pub fn scoped_indices(&self, scope: &str) -> Vec<usize> {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(i, w)| (w.scope == scope).then_some(i))
            .collect()
    }

    pub fn toggle(&mut self, index: usize) -> bool {
        if let Some(wp) = self.entries.get_mut(index) {
            wp.enabled = !wp.enabled;
            true
        } else {
            false
        }
    }

    pub fn cycle_color(&mut self, index: usize, back: bool) -> bool {
        if let Some(wp) = self.entries.get_mut(index) {
            wp.color = wp.color.cycle(back);
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, index: usize) -> bool {
        if index < self.entries.len() {
            self.entries.remove(index);
            true
        } else {
            false
        }
    }

    pub fn clear_deaths(&mut self, scope: &str) -> usize {
        let before = self.entries.len();
        self.entries.retain(|w| !(w.scope == scope && w.is_death));
        before - self.entries.len()
    }
}

/// Effective target coordinates of a waypoint in `current_dim`, applying 8x
/// Overworld <-> Nether scaling when `nether_sync` is enabled.
/// Returns `(world_x, world_y, world_z, cross_dimension_tag)`.
pub fn effective_waypoint_coords(
    wp: &Waypoint,
    current_dim: &str,
    nether_sync: bool,
) -> Option<([f64; 3], Option<&'static str>)> {
    if wp.dimension == current_dim {
        return Some((
            [
                f64::from(wp.x) + 0.5,
                f64::from(wp.y) + 0.5,
                f64::from(wp.z) + 0.5,
            ],
            None,
        ));
    }
    if !nether_sync {
        return None;
    }
    let overworld = "minecraft:overworld";
    let nether = "minecraft:the_nether";
    if wp.dimension == overworld && current_dim == nether {
        Some((
            [
                (f64::from(wp.x) + 0.5) / 8.0,
                f64::from(wp.y).clamp(16.0, 120.0) + 0.5,
                (f64::from(wp.z) + 0.5) / 8.0,
            ],
            Some("OW"),
        ))
    } else if wp.dimension == nether && current_dim == overworld {
        Some((
            [
                (f64::from(wp.x) + 0.5) * 8.0,
                f64::from(wp.y) + 0.5,
                (f64::from(wp.z) + 0.5) * 8.0,
            ],
            Some("Nether"),
        ))
    } else {
        None
    }
}

pub fn short_dimension_label(dim: &str) -> &'static str {
    match dim {
        "minecraft:overworld" => "OW",
        "minecraft:the_nether" => "Nether",
        "minecraft:the_end" => "End",
        _ => "Dim",
    }
}

/// The four corners of a HUD arrow whose base sits `radius` px from
/// `center`, pointing along `angle` (screen radians, 0 = right, positive
/// clockwise — `ProjectedWaypoint::edge_angle`'s convention).
pub fn hud_arrow(center: (f32, f32), angle: f32, radius: f32) -> [[f32; 2]; 4] {
    let (sin, cos) = angle.sin_cos();
    let (cx, cy) = center;
    let (ex, ey) = (cx + cos * radius, cy + sin * radius);
    let (px, py) = (-sin, cos);
    [
        [ex + cos * 6.0, ey + sin * 6.0],
        [ex - cos * 2.0 + px * 4.0, ey - sin * 2.0 + py * 4.0],
        [ex - cos * 2.0 - px * 4.0, ey - sin * 2.0 - py * 4.0],
        [ex - cos * 6.0, ey - sin * 6.0],
    ]
}

/// A damage popup's rise (in blocks) and label alpha `age_ticks` after the
/// hit landed: it climbs and fades for one second, and is gone at 20 ticks.
pub fn damage_popup_style(age_ticks: i64) -> Option<(f32, u8)> {
    if !(0..20).contains(&age_ticks) {
        return None;
    }
    let t = age_ticks as f32;
    Some(((t * 0.045).min(0.9), (255.0 - t * 13.0) as u8))
}

/// The direction a player throw starts in (`Projectile.shootFromRotation`
/// with offset 0): yaw 0 looks along +Z, positive pitch looks down.
pub fn throw_direction(yaw: f32, pitch: f32) -> [f64; 3] {
    let (p, y) = (pitch.to_radians(), yaw.to_radians());
    [
        f64::from(-y.sin() * p.cos()),
        f64::from(-p.sin()),
        f64::from(y.cos() * p.cos()),
    ]
}

/// The predicted flight of a thrown item (`ThrowableProjectile.tick` from
/// `origin` at `velocity`): drag 0.99 and `gravity` off the Y each tick,
/// until the point enters a solid block or three seconds (60 ticks) pass.
pub fn trajectory_points(
    origin: [f64; 3],
    velocity: [f64; 3],
    gravity: f64,
    solid: impl Fn([i32; 3]) -> bool,
) -> Vec<[f64; 3]> {
    let mut points = Vec::with_capacity(60);
    let (mut p, mut v) = (origin, velocity);
    for _ in 0..60 {
        v = [v[0] * 0.99, v[1] * 0.99 - gravity, v[2] * 0.99];
        p = [p[0] + v[0], p[1] + v[1], p[2] + v[2]];
        let hit = solid([
            p[0].floor() as i32,
            p[1].floor() as i32,
            p[2].floor() as i32,
        ]);
        points.push(p);
        if hit {
            break;
        }
    }
    points
}

/// The Spawn Light overlay's box colour for a position's block light: red
/// where monsters spawn (the overworld's limit, 0), amber up to 7, nothing
/// above that.
pub fn spawn_light_color(block_light: i32) -> Option<u32> {
    if block_light <= 0 {
        Some(0xFFFF_3030)
    } else if block_light <= 7 {
        Some(0xFFFF_A500)
    } else {
        None
    }
}

/// Whether slimes spawn in the chunk at (`chunk_x`, `chunk_z`, `y`): the
/// slime-chunk 1-in-10 roll, and only below y 40 (`Slime` spawn rules).
pub fn slime_spawns_in_chunk(chunk_x: i32, chunk_z: i32, y: i32, seed: i64) -> bool {
    use horizon_core::random::{RandomSource, slime_chunk_random};
    if y >= 40 {
        return false;
    }
    slime_chunk_random(chunk_x, chunk_z, seed, 987_234_911)
        .next_int_bound(10)
        .unwrap_or(1)
        == 0
}

/// The Auto Tool hint's pick: the hotbar slot with the highest mining
/// speed on the targeted block, when any held item actually speeds it up
/// (above `ItemStack.getDestroySpeed`'s 1.0 floor).
pub fn best_tool_slot(speeds: &[f32]) -> Option<usize> {
    let mut best: Option<(usize, f32)> = None;
    for (i, &s) in speeds.iter().enumerate() {
        if s > 1.0 && best.is_none_or(|(_, b)| s > b) {
            best = Some((i, s));
        }
    }
    best.map(|(i, _)| i)
}

/// The Tool Break Shield's warning: the uses left once the held tool is
/// at or under a tenth of its durability (or at five uses), else None.
pub fn tool_break_warning(left: u32, max: u32) -> Option<u32> {
    if max == 0 || (left * 10 > max && left > 5) {
        return None;
    }
    Some(left)
}

/// The Volt Cam Framing Grid's four lines as fill rects (x0, y0, x1, y1):
/// thirds of the screen both ways, one pixel wide.
pub fn framing_grid(w: f32, h: f32) -> [(f32, f32, f32, f32); 4] {
    [
        (w / 3.0, 0.0, w / 3.0 + 1.0, h),
        (2.0 * w / 3.0, 0.0, 2.0 * w / 3.0 + 1.0, h),
        (0.0, h / 3.0, w, h / 3.0 + 1.0),
        (0.0, 2.0 * h / 3.0, w, 2.0 * h / 3.0 + 1.0),
    ]
}

/// The Shulker & Bundle Peek grid's cell for a container slot: `(col, row)`
/// of the 9-wide, 3-tall grid, or None past the 27 slots a shulker box holds.
pub fn shulker_cell(slot: u8) -> Option<(u8, u8)> {
    (slot < 27).then_some((slot % 9, slot / 9))
}

/// The Volt Atlas grid's line spacing in blocks for a map of
/// `blocks_per_pixel`: the smallest power-of-two multiple of 16 that stays
/// at least 24 image pixels apart.
pub fn atlas_grid_spacing(blocks_per_pixel: f32) -> i32 {
    let mut spacing = 16;
    while spacing as f32 * 24.0 < blocks_per_pixel * 128.0 && spacing < 4096 {
        spacing *= 2;
    }
    spacing
}

/// The Volt Atlas biome tint's colour for a biome id: a stable hue from the
/// id, full saturation, so each biome keeps its own tint on the map.
pub fn biome_tint(id: u16) -> (u8, u8, u8) {
    let h = (id as u32).wrapping_mul(2_654_435_761) % 360;
    let (r, g, b) = match h {
        0..=59 => (255u32, 255 * h / 60, 0),
        60..=119 => (255 * (120 - h) / 60, 255, 0),
        120..=179 => (0, 255, 255 * (h - 120) / 60),
        180..=239 => (0, 255 * (240 - h) / 60, 255),
        240..=299 => (255 * (h - 240) / 60, 0, 255),
        _ => (255, 0, 255 * (360 - h) / 60),
    };
    (r as u8, g as u8, b as u8)
}

/// Screen-space projection of a 3D world waypoint for HUD rendering.
#[derive(Clone, Copy, Debug)]
pub struct ProjectedWaypoint {
    pub screen_x: f32,
    pub screen_y: f32,
    pub beam_top_y: f32,
    pub beam_bottom_y: f32,
    pub distance: f64,
    pub in_front: bool,
    pub on_screen: bool,
    /// Cosine similarity (`0..=1`) between camera centre and marker ray.
    pub center_focus: f32,
    /// Clamped rim position `(x, y)` for off-screen edge arrows.
    pub edge_pos: (f32, f32),
    /// Angle in radians pointing from screen centre toward the waypoint.
    pub edge_angle: f32,
}

pub fn project_waypoint(
    cam_pos: [f64; 3],
    yaw: f32,
    pitch: f32,
    fov_degrees: f32,
    screen_w: f32,
    screen_h: f32,
    target: [f64; 3],
) -> ProjectedWaypoint {
    let dx = target[0] - cam_pos[0];
    let dy = target[1] - cam_pos[1];
    let dz = target[2] - cam_pos[2];
    let distance = (dx * dx + dy * dy + dz * dz).sqrt();
    let view = view_matrix(yaw, pitch);
    let aspect = (screen_w / screen_h.max(1.0)).max(0.1);
    let tan_half = (fov_degrees.clamp(10.0, 150.0).to_radians() * 0.5)
        .tan()
        .max(0.05);

    let project_pt = |wy: f64| -> (f32, f32, f32) {
        let rel = GlamVec3::new(dx as f32, (wy - cam_pos[1]) as f32, dz as f32);
        let eye = view.transform_point3(rel);
        let depth = -eye.z;
        let z_abs = depth.abs().max(0.05);
        let ndc_x = eye.x / (z_abs * tan_half * aspect);
        let ndc_y = eye.y / (z_abs * tan_half);
        (
            (ndc_x * 0.5 + 0.5) * screen_w,
            (0.5 - ndc_y * 0.5) * screen_h,
            depth,
        )
    };

    let (sx, sy, depth) = project_pt(target[1] + 1.35);
    let (_, top_y, _) = project_pt(target[1] + 180.0);
    let (_, bot_y, _) = project_pt(target[1] - 48.0);
    let in_front = depth > 0.1;
    let margin = 18.0;
    let on_screen = in_front
        && sx >= margin
        && sx <= screen_w - margin
        && sy >= margin
        && sy <= screen_h - margin;

    let rel_main = GlamVec3::new(dx as f32, (target[1] + 1.35 - cam_pos[1]) as f32, dz as f32);
    let eye_main = view.transform_point3(rel_main);
    let len = eye_main.length().max(0.001);
    let center_focus = (-eye_main.z / len).clamp(-1.0, 1.0);

    // Direction on the 2D screen plane from screen centre toward the waypoint.
    let mut dir_x = eye_main.x;
    let mut dir_y = -eye_main.y;
    if !in_front {
        // Behind camera: push cleanly toward the horizontal side so turning toward
        // the arrow always brings the waypoint into view.
        if dir_x.abs() < 0.001 {
            dir_x = 1.0;
        }
        dir_y = dir_y.clamp(-0.35 * dir_x.abs(), 0.35 * dir_x.abs());
    }
    let edge_angle = dir_y.atan2(dir_x);
    let (cx, cy) = (screen_w * 0.5, screen_h * 0.5);
    let half_w = (screen_w * 0.5 - 22.0).max(16.0);
    let half_h = (screen_h * 0.5 - 26.0).max(16.0);
    let scale_x = if dir_x.abs() > 1e-4 {
        half_w / dir_x.abs()
    } else {
        f32::INFINITY
    };
    let scale_y = if dir_y.abs() > 1e-4 {
        half_h / dir_y.abs()
    } else {
        f32::INFINITY
    };
    let t = scale_x.min(scale_y);
    let edge_pos = if t.is_finite() {
        (cx + dir_x * t, cy + dir_y * t)
    } else {
        (cx + half_w, cy)
    };

    ProjectedWaypoint {
        screen_x: sx,
        screen_y: sy,
        beam_top_y: top_y.min(bot_y).clamp(-32.0, screen_h + 32.0),
        beam_bottom_y: top_y.max(bot_y).clamp(-32.0, screen_h + 32.0),
        distance,
        in_front,
        on_screen,
        center_focus,
        edge_pos,
        edge_angle,
    }
}

// ============================================================================
// Volt Minimap Features
// ============================================================================

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MinimapFrame {
    #[default]
    VoltCrimson,
    ClassicDark,
    CleanMinimal,
}

impl MinimapFrame {
    pub const ALL: [Self; 3] = [Self::VoltCrimson, Self::ClassicDark, Self::CleanMinimal];
    pub fn name(self) -> &'static str {
        match self {
            Self::VoltCrimson => "crimson",
            Self::ClassicDark => "dark",
            Self::CleanMinimal => "minimal",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::VoltCrimson => "Volt Crimson",
            Self::ClassicDark => "Classic Dark",
            Self::CleanMinimal => "Minimal Clean",
        }
    }
    pub fn parse(s: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|m| m.name() == s)
            .unwrap_or_default()
    }
    pub fn cycle(self, back: bool) -> Self {
        let i = Self::ALL.iter().position(|&m| m == self).unwrap_or(0);
        Self::ALL[if back {
            (i + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (i + 1) % Self::ALL.len()
        }]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MinimapTerrainMode {
    #[default]
    Auto,
    Surface,
    Cave,
}

impl MinimapTerrainMode {
    pub const ALL: [Self; 3] = [Self::Auto, Self::Surface, Self::Cave];
    pub fn name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Surface => "surface",
            Self::Cave => "cave",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto (Cave/Surface)",
            Self::Surface => "Surface Only",
            Self::Cave => "Cave Slice",
        }
    }
    pub fn parse(s: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|m| m.name() == s)
            .unwrap_or_default()
    }
    pub fn cycle(self, back: bool) -> Self {
        let i = Self::ALL.iter().position(|&m| m == self).unwrap_or(0);
        Self::ALL[if back {
            (i + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (i + 1) % Self::ALL.len()
        }]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MinimapEntities {
    Off,
    Players,
    Hostiles,
    #[default]
    All,
}

impl MinimapEntities {
    pub const ALL: [Self; 4] = [Self::Off, Self::Players, Self::Hostiles, Self::All];
    pub fn name(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Players => "players",
            Self::Hostiles => "hostiles",
            Self::All => "all",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Players => "Players Only",
            Self::Hostiles => "Players & Hostiles",
            Self::All => "All Entities",
        }
    }
    pub fn parse(s: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|m| m.name() == s)
            .unwrap_or_default()
    }
    pub fn cycle(self, back: bool) -> Self {
        let i = Self::ALL.iter().position(|&m| m == self).unwrap_or(0);
        Self::ALL[if back {
            (i + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (i + 1) % Self::ALL.len()
        }]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadarKind {
    Player,
    Hostile,
    Passive,
    Item,
}

impl RadarKind {
    pub fn rgb(self) -> [u8; 3] {
        match self {
            Self::Player => [56, 189, 248],
            Self::Hostile => [239, 68, 68],
            Self::Passive => [34, 197, 94],
            Self::Item => [250, 204, 21],
        }
    }

    pub fn shown_by(self, filter: MinimapEntities) -> bool {
        match filter {
            MinimapEntities::Off => false,
            MinimapEntities::Players => self == Self::Player,
            MinimapEntities::Hostiles => matches!(self, Self::Player | Self::Hostile),
            MinimapEntities::All => true,
        }
    }
}

pub fn classify_mob_id(id: &str) -> RadarKind {
    let bare = id.strip_prefix("minecraft:").unwrap_or(id);
    if bare == "player" {
        return RadarKind::Player;
    }
    const HOSTILES: &[&str] = &[
        "zombie",
        "zombie_villager",
        "husk",
        "drowned",
        "zombified_piglin",
        "zoglin",
        "skeleton",
        "stray",
        "bogged",
        "wither_skeleton",
        "creeper",
        "spider",
        "cave_spider",
        "enderman",
        "endermite",
        "witch",
        "slime",
        "magma_cube",
        "blaze",
        "ghast",
        "phantom",
        "pillager",
        "vindicator",
        "evoker",
        "illusioner",
        "ravager",
        "vex",
        "guardian",
        "elder_guardian",
        "silverfish",
        "shulker",
        "piglin_brute",
        "hoglin",
        "warden",
        "breeze",
        "creaking",
        "ender_dragon",
        "wither",
    ];
    if HOSTILES.contains(&bare) {
        RadarKind::Hostile
    } else {
        RadarKind::Passive
    }
}

/// Projects world `(wx, wz)` into local minimap coordinates `(-half..=half, -half..=half)`
/// relative to the minimap centre `(cx, cy)` for a given `map_size` in GUI pixels.
/// When `clamp_rim` is true, out-of-range points are clamped onto the map rim and
/// returned with `clamped = true`; otherwise returns `None` when outside the map.
pub fn world_to_minimap_offset(
    wx: f64,
    wz: f64,
    view: &View,
    map_size: f32,
    clamp_rim: bool,
) -> Option<(f32, f32, bool)> {
    let scale = map_size / crate::minimap::SIZE as f32;
    let bpp = view.blocks_per_pixel.max(0.05);
    let dx = (wx - view.x) as f32 / bpp * scale;
    let dz = (wz - view.z) as f32 / bpp * scale;
    let (rx, ry) = if view.rotate {
        // Match `minimap::compose`: forward is up (-Y on screen).
        let (sin, cos) = (view.yaw + 180.0).to_radians().sin_cos();
        // Inverse of compose's `(u * c - v * s, u * s + v * c)`:
        (dx * cos + dz * sin, -dx * sin + dz * cos)
    } else {
        (dx, dz)
    };
    let half = (map_size * 0.5 - 3.0).max(4.0);
    if view.round {
        let dist = (rx * rx + ry * ry).sqrt();
        if dist <= half {
            Some((rx, ry, false))
        } else if clamp_rim && dist > 1e-4 {
            Some((rx / dist * half, ry / dist * half, true))
        } else {
            None
        }
    } else {
        let max_axis = rx.abs().max(ry.abs());
        if max_axis <= half {
            Some((rx, ry, false))
        } else if clamp_rim && max_axis > 1e-4 {
            Some((rx / max_axis * half, ry / max_axis * half, true))
        } else {
            None
        }
    }
}

pub fn format_coords(pos: [f64; 3]) -> String {
    format!(
        "{}, {}, {}",
        pos[0].floor() as i32,
        pos[1].floor() as i32,
        pos[2].floor() as i32
    )
}

pub fn format_heading_and_biome(yaw: f32, biome: Option<&str>) -> String {
    let mut wrapped = yaw % 360.0;
    if wrapped >= 180.0 {
        wrapped -= 360.0;
    }
    if wrapped < -180.0 {
        wrapped += 360.0;
    }
    // Vanilla yaw: 0 = South (+Z), -90 = East (+X), +/-180 = North (-Z), 90 = West (-X).
    let norm = (wrapped + 360.0) % 360.0;
    let dirs = ["S", "SW", "W", "NW", "N", "NE", "E", "SE"];
    let idx = (((norm + 22.5) / 45.0).floor() as usize) % 8;
    let cardinal = dirs[idx];
    if let Some(raw) = biome {
        let pretty = prettify_identifier(raw);
        format!("{cardinal} · {pretty}")
    } else {
        format!("{cardinal} ({wrapped:.0}°)")
    }
}

fn prettify_identifier(id: &str) -> String {
    let bare = id.rsplit(':').next().unwrap_or(id);
    bare.split('_')
        .filter(|s| !s.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn format_world_clock(day_time: i64) -> String {
    let day = day_time.div_euclid(24_000) + 1;
    let ticks = day_time.rem_euclid(24_000);
    // Tick 0 in Minecraft is 06:00 AM.
    let total_minutes = ((ticks + 6_000) % 24_000) * 60 / 1_000;
    let hour = total_minutes / 60;
    let minute = total_minutes % 60;
    let phase = if (0..12_500).contains(&ticks) {
        "Day"
    } else {
        "Night"
    };
    format!("D{day} {hour:02}:{minute:02} ({phase})")
}

/// The blocks of a blueprint template, as (offset from the anchor, block).
/// The anchor is where the player stands; the offsets grow up and outward.
pub fn blueprint_layout(preset: crate::volt_qol::BlueprintPreset) -> Vec<([i16; 3], &'static str)> {
    use crate::volt_qol::BlueprintPreset as P;
    let mut out = Vec::new();
    match preset {
        P::StarterCabin => {
            blueprint_ring(&mut out, 7, 4, 7, "minecraft:oak_planks");
            for x in 0..7 {
                for z in 0..7 {
                    if x == 0 || x == 6 || z == 0 || z == 6 {
                        out.push(([x, 4, z], "minecraft:oak_slab"));
                    }
                }
            }
        }
        P::Watchtower => blueprint_ring(&mut out, 5, 8, 5, "minecraft:stone_bricks"),
        // The classic 9x9: a water channel down the middle, farmland around.
        P::CropFarm9x9 => {
            for x in 0..9 {
                for z in 0..9 {
                    let block = if x == 4 {
                        "minecraft:water"
                    } else {
                        "minecraft:farmland"
                    };
                    out.push(([x, 0, z], block));
                }
            }
        }
        // The 4x5 obsidian frame with its 2x3 opening.
        P::PortalShrine => {
            for x in 0..4 {
                for y in 0..5 {
                    if x == 0 || x == 3 || y == 0 || y == 4 {
                        out.push(([x, y, 0], "minecraft:obsidian"));
                    }
                }
            }
        }
        // A 9x9 spawning deck with a three high glass rim.
        P::MobFarmDeck => {
            for x in 0..9 {
                for z in 0..9 {
                    out.push(([x, 0, z], "minecraft:stone"));
                }
            }
            for y in 1..4 {
                for x in 0..9 {
                    for z in 0..9 {
                        if x == 0 || x == 8 || z == 0 || z == 8 {
                            out.push(([x, y, z], "minecraft:glass"));
                        }
                    }
                }
            }
        }
        // What the player copied: nothing to project.
        P::WorldClipboard => {}
    }
    out
}

/// Hollow walls for a template: the perimeter of every layer, and a floor.
fn blueprint_ring(
    out: &mut Vec<([i16; 3], &'static str)>,
    w: i16,
    h: i16,
    d: i16,
    block: &'static str,
) {
    for y in 0..h {
        for x in 0..w {
            for z in 0..d {
                if x == 0 || x == w - 1 || z == 0 || z == d - 1 || y == 0 {
                    out.push(([x, y, z], block));
                }
            }
        }
    }
}

/// The hologram's ghost boxes: the template's blocks, sliced by `layer`
/// (-1 shows every layer), each a unit cube at `anchor + offset`.
pub fn blueprint_boxes(
    preset: crate::volt_qol::BlueprintPreset,
    layer: i16,
    anchor: [i32; 3],
) -> Vec<([f64; 3], [f64; 3])> {
    blueprint_layout(preset)
        .into_iter()
        .filter(|(at, _)| layer < 0 || at[1] == layer)
        .map(|(at, _)| {
            let min = [
                f64::from(anchor[0] + i32::from(at[0])),
                f64::from(anchor[1] + i32::from(at[1])),
                f64::from(anchor[2] + i32::from(at[2])),
            ];
            (min, [min[0] + 1.0, min[1] + 1.0, min[2] + 1.0])
        })
        .collect()
}

/// The template's materials for the checklist: a `layer` of -1 counts every
/// layer, else only blocks whose Y offset equals it. Sorted by count
/// (largest first), then by name.
pub fn blueprint_materials(
    preset: crate::volt_qol::BlueprintPreset,
    layer: i16,
) -> Vec<(&'static str, usize)> {
    let mut counts: Vec<(&'static str, usize)> = Vec::new();
    for (at, block) in blueprint_layout(preset) {
        if layer >= 0 && at[1] != layer {
            continue;
        }
        match counts.iter_mut().find(|(name, _)| *name == block) {
            Some((_, count)) => *count += 1,
            None => counts.push((block, 1)),
        }
    }
    counts.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    counts
}

/// Rezyth Settings' replay recorder (`volt_qol.replay_buffer`): a session's
/// camera track, one sample per world tick while recording. Written as JSON
/// to the player's `replays/` folder when the button ends the take.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayTrack {
    /// The dimension id at the start (`minecraft:overworld`).
    pub dimension: String,
    /// The world's game time at the first sample.
    pub start_game_time: i64,
    /// The camera track: the eye's position and view per world tick.
    pub samples: Vec<ReplaySample>,
}

/// One world tick's camera pose.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReplaySample {
    pub tick: i64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

/// The recorder's state: the button (or the key) cycles idle → recording ⇄
/// paused → idle, saving the track on the way out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReplayState {
    #[default]
    Idle,
    Recording,
    Paused,
}

impl ReplayTrack {
    /// One world tick's camera sample.
    pub fn record(&mut self, tick: i64, at: [f64; 3], yaw: f32, pitch: f32) {
        self.samples.push(ReplaySample {
            tick,
            x: at[0],
            y: at[1],
            z: at[2],
            yaw,
            pitch,
        });
    }

    /// The take's length in world ticks (first to last sample).
    pub fn length_ticks(&self) -> i64 {
        match (self.samples.first(), self.samples.last()) {
            (Some(first), Some(last)) => last.tick - first.tick,
            _ => 0,
        }
    }
}

/// The replay file's name: `replay-<unix>-<dimension>.voltreplay.json`, the
/// dimension short of its namespace.
pub fn replay_file_name(unix: u64, dimension: &str) -> String {
    let dim = dimension.rsplit(':').next().unwrap_or(dimension);
    format!("replay-{unix}-{dim}.voltreplay.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The replay track: samples land in order, the length is first to last,
    /// and the JSON round-trips.
    #[test]
    fn replay_track_records_and_round_trips() {
        let mut track = ReplayTrack {
            dimension: "minecraft:overworld".into(),
            start_game_time: 100,
            samples: Vec::new(),
        };
        track.record(100, [0.5, 64.0, 0.5], 0.0, 0.0);
        track.record(101, [1.5, 64.0, 0.5], 90.0, 10.0);
        assert_eq!(track.samples.len(), 2);
        assert_eq!(track.length_ticks(), 1);
        assert_eq!(ReplayTrack::default().length_ticks(), 0);
        let json = serde_json::to_string(&track).unwrap();
        let back: ReplayTrack = serde_json::from_str(&json).unwrap();
        assert_eq!(back, track);
    }

    /// The replay file's name carries the stamp and the short dimension.
    #[test]
    fn replay_file_name_is_stamped_and_short() {
        assert_eq!(
            replay_file_name(1_700_000_000, "minecraft:the_nether"),
            "replay-1700000000-the_nether.voltreplay.json"
        );
        assert_eq!(
            replay_file_name(1, "overworld"),
            "replay-1-overworld.voltreplay.json"
        );
    }

    /// The damage-direction arrow's tip sits `radius` out along its angle.
    #[test]
    fn hud_arrow_points_along_its_angle() {
        let c = (100.0, 100.0);
        let right = hud_arrow(c, 0.0, 40.0);
        assert!((right[0][0] - 146.0).abs() < 0.001);
        assert!((right[0][1] - 100.0).abs() < 0.001);
        let up = hud_arrow(c, -std::f32::consts::FRAC_PI_2, 40.0);
        assert!((up[0][0] - 100.0).abs() < 0.001);
        assert!((up[0][1] - 54.0).abs() < 0.001);
    }

    /// A damage popup rises and fades over its twenty ticks of life.
    #[test]
    fn damage_popup_rises_then_fades_out() {
        assert_eq!(damage_popup_style(0), Some((0.0, 255)));
        let (rise10, alpha10) = damage_popup_style(10).unwrap();
        assert!(rise10 > 0.0 && alpha10 < 255);
        assert!(damage_popup_style(19).is_some());
        assert_eq!(damage_popup_style(20), None);
        assert_eq!(damage_popup_style(-1), None);
    }

    /// A throw starts along the look: yaw 0 is +Z, pitch 90 straight down.
    #[test]
    fn throw_direction_matches_the_look() {
        let d = throw_direction(0.0, 0.0);
        assert!(d[0].abs() < 1e-9 && d[1].abs() < 1e-9 && (d[2] - 1.0).abs() < 1e-9);
        // Pitch is in degrees: 90 straight down.
        let down = throw_direction(0.0, 90.0);
        assert!((down[1] + 1.0).abs() < 1e-6 && down[2].abs() < 1e-6);
    }

    /// The predicted arc rises, then falls, and stops when it lands.
    #[test]
    fn trajectory_arc_rises_then_falls_and_stops_at_solid() {
        let pts = trajectory_points([0.0, 64.0, 0.0], [0.0, 0.75, 0.0], 0.03, |_| false);
        assert_eq!(pts.len(), 60);
        let apex = pts.iter().map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max);
        assert!(apex > 64.5);
        assert!(pts.last().unwrap()[1] < apex);
        let landed = trajectory_points([0.0, 64.0, 0.0], [0.0, 0.75, 0.0], 0.03, |b| b[1] < 63);
        assert!(landed.len() < 60);
        let last = landed.last().unwrap();
        assert!(last[1] < 63.0 && last[1] > 62.0);
    }

    /// The spawn-light overlay paints 0 red, 1..=7 amber, nothing above.
    #[test]
    fn spawn_light_color_marks_the_dark_spots() {
        assert_eq!(spawn_light_color(0), Some(0xFFFF_3030));
        assert_eq!(spawn_light_color(7), Some(0xFFFF_A500));
        assert_eq!(spawn_light_color(8), None);
        assert_eq!(spawn_light_color(15), None);
    }

    /// The Auto Tool hint picks the fastest slot, only when one speeds up.
    #[test]
    fn best_tool_slot_picks_the_fastest_real_tool() {
        assert_eq!(best_tool_slot(&[1.0, 8.0, 4.0]), Some(1));
        assert_eq!(best_tool_slot(&[2.0, 8.0, 8.0]), Some(1));
        assert_eq!(best_tool_slot(&[1.0, 1.0]), None);
        assert_eq!(best_tool_slot(&[]), None);
    }

    /// The peek grid maps the 27 container slots onto 9 columns and 3 rows.
    #[test]
    fn shulker_cell_maps_slots_onto_a_9_by_3_grid() {
        assert_eq!(shulker_cell(0), Some((0, 0)));
        assert_eq!(shulker_cell(8), Some((8, 0)));
        assert_eq!(shulker_cell(9), Some((0, 1)));
        assert_eq!(shulker_cell(26), Some((8, 2)));
        assert_eq!(shulker_cell(27), None);
        assert_eq!(shulker_cell(200), None);
    }

    /// The atlas grid's spacing grows with the map's zoom, in 16-block steps.
    #[test]
    fn atlas_grid_spacing_stays_readable_at_any_zoom() {
        assert_eq!(atlas_grid_spacing(0.5), 16);
        assert_eq!(atlas_grid_spacing(1.0), 16);
        assert_eq!(atlas_grid_spacing(2.0), 16);
        assert_eq!(atlas_grid_spacing(4.0), 32);
        assert_eq!(atlas_grid_spacing(8.0), 64);
        assert_eq!(atlas_grid_spacing(32.0), 256);
        assert_eq!(atlas_grid_spacing(64.0), 512);
        for bpp in [0.5, 1.0, 2.0, 3.7, 8.0, 32.0] {
            let s = atlas_grid_spacing(bpp);
            assert!(s >= 16 && s & (s - 1) == 0);
            assert!(s as f32 * 24.0 >= bpp * 128.0 || s >= 4096);
        }
    }

    /// The biome tint is stable per id and differs between biomes.
    #[test]
    fn biome_tint_is_stable_and_distinct() {
        assert_eq!(biome_tint(3), biome_tint(3));
        assert_ne!(biome_tint(1), biome_tint(2));
        assert_ne!(biome_tint(1), biome_tint(200));
        for id in 0..300 {
            let (r, g, b) = biome_tint(id);
            assert!(r.max(g).max(b) == 255);
        }
    }

    /// The framing grid cuts the screen in thirds both ways.
    #[test]
    fn framing_grid_cuts_the_screen_in_thirds() {
        let g = framing_grid(900.0, 600.0);
        assert_eq!(g[0], (300.0, 0.0, 301.0, 600.0));
        assert_eq!(g[1], (600.0, 0.0, 601.0, 600.0));
        assert_eq!(g[2], (0.0, 200.0, 900.0, 201.0));
        assert_eq!(g[3], (0.0, 400.0, 900.0, 401.0));
    }

    /// The break warning fires at a tenth of the durability or five left.
    #[test]
    fn tool_break_warning_fires_at_a_tenth_or_five_left() {
        assert_eq!(tool_break_warning(10, 100), Some(10));
        assert_eq!(tool_break_warning(11, 100), None);
        assert_eq!(tool_break_warning(3, 20), Some(3));
        assert_eq!(tool_break_warning(6, 20), None);
        assert_eq!(tool_break_warning(0, 100), Some(0));
        assert_eq!(tool_break_warning(50, 0), None);
    }

    /// A slime chunk's roll decides, and slimes only spawn below y 40.
    #[test]
    fn slime_chunks_roll_and_only_spawn_below_40() {
        let seed = 42;
        let mut found = None;
        for cx in 0..64 {
            for cz in 0..64 {
                if slime_spawns_in_chunk(cx, cz, 0, seed) {
                    found = Some((cx, cz));
                }
            }
        }
        let (cx, cz) = found.expect("some chunk in 0..64 x 0..64 is a slime chunk");
        assert!(slime_spawns_in_chunk(cx, cz, 39, seed));
        assert!(!slime_spawns_in_chunk(cx, cz, 40, seed));
    }

    /// The blueprint templates list their materials, sliced by the Y layer.
    #[test]
    fn blueprint_templates_list_their_materials() {
        use crate::volt_qol::BlueprintPreset as P;
        assert_eq!(
            blueprint_materials(P::PortalShrine, -1),
            [("minecraft:obsidian", 14)]
        );
        assert_eq!(
            blueprint_materials(P::PortalShrine, 0),
            [("minecraft:obsidian", 4)]
        );
        assert_eq!(
            blueprint_materials(P::PortalShrine, 2),
            [("minecraft:obsidian", 2)]
        );
        // The classic 9x9 farm counts every cell: 9 water, 72 farmland.
        let farm = blueprint_materials(P::CropFarm9x9, -1);
        assert_eq!(farm, [("minecraft:farmland", 72), ("minecraft:water", 9)]);
        let total: usize = farm.iter().map(|(_, count)| count).sum();
        assert_eq!(total, 81);
        // Every template but the clipboard (nothing copied) has blocks.
        for preset in P::ALL {
            let empty = blueprint_layout(preset).is_empty();
            assert_eq!(empty, preset == P::WorldClipboard, "{preset:?}");
        }
    }
    /// The hologram's boxes sit at the anchor, sliced by the Y layer.
    #[test]
    fn blueprint_hologram_boxes_sit_at_the_anchor() {
        use crate::volt_qol::BlueprintPreset as P;
        let boxes = blueprint_boxes(P::PortalShrine, -1, [10, 64, -3]);
        assert_eq!(boxes.len(), 14);
        assert_eq!(boxes[0], ([10.0, 64.0, -3.0], [11.0, 65.0, -2.0]));
        assert_eq!(blueprint_boxes(P::PortalShrine, 2, [0, 0, 0]).len(), 2);
        assert!(blueprint_boxes(P::WorldClipboard, -1, [0, 0, 0]).is_empty());
    }
    #[test]
    fn waypoint_store_handles_death_limit_and_nether_sync() {
        let mut store = WaypointStore::default();
        let id = store.add(
            "sp:Test",
            "  Base  ",
            "minecraft:overworld",
            [80, 64, -160],
            WaypointColor::Emerald,
        );
        assert_eq!(id, 1);
        assert_eq!(store.entries[0].name, "Base");

        let wp = &store.entries[0];
        let (nether_pos, tag) =
            effective_waypoint_coords(wp, "minecraft:the_nether", true).unwrap();
        assert_eq!(tag, Some("OW"));
        assert!((nether_pos[0] - 10.0625).abs() < 0.01);
        assert!((nether_pos[2] - -19.9375).abs() < 0.01);
        assert!(effective_waypoint_coords(wp, "minecraft:the_nether", false).is_none());

        for x in [10, 20, 30, 40] {
            store.record_death("sp:Test", "minecraft:overworld", [x, 70, 0]);
        }
        let deaths: Vec<&Waypoint> = store.entries.iter().filter(|w| w.is_death).collect();
        assert_eq!(deaths.len(), 3);
        assert_eq!(deaths.last().unwrap().name, "Last Death");
        assert_eq!(deaths.last().unwrap().x, 40);
        assert_eq!(store.clear_deaths("sp:Test"), 3);
    }

    #[test]
    fn projection_places_forward_waypoint_at_screen_center() {
        // Yaw 0 in Minecraft faces South (+Z).
        let proj = project_waypoint(
            [0.0, 65.62, 0.0],
            0.0,
            0.0,
            70.0,
            480.0,
            270.0,
            [0.0, 64.27, 25.0],
        );
        assert!(proj.in_front && proj.on_screen);
        assert!((proj.screen_x - 240.0).abs() < 1.0, "{}", proj.screen_x);
        assert!((proj.screen_y - 135.0).abs() < 2.0, "{}", proj.screen_y);
        assert!(proj.center_focus > 0.99);
    }

    #[test]
    fn volt_boost_presets_round_trip_and_default_off() {
        assert_eq!(VoltBoostPreset::default(), VoltBoostPreset::Off);
        for p in VoltBoostPreset::ALL {
            assert_eq!(VoltBoostPreset::parse(p.as_str()), Some(p));
        }
        for m in PerfHudMode::ALL {
            assert_eq!(PerfHudMode::parse(m.as_str()), Some(m));
        }
        for c in EntityCap::ALL {
            assert_eq!(EntityCap::parse(c.as_str()), Some(c));
        }
        for b in ParticleBudget::ALL {
            assert_eq!(ParticleBudget::parse(b.as_str()), Some(b));
        }
    }
}

// ============================================================================
// 4. Volt Boost — Tier 1, Tier 2 & Tier 3 Performance & Optimization Types
// ============================================================================

/// Master one-tap profile for **Volt Boost**. `Off` preserves 100% vanilla behaviour.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoltBoostPreset {
    #[default]
    Off,
    Balanced,
    Competitive,
    UltraFps,
    BatterySaver,
    LowRam,
}

impl VoltBoostPreset {
    pub const ALL: [Self; 6] = [
        Self::Off,
        Self::Balanced,
        Self::Competitive,
        Self::UltraFps,
        Self::BatterySaver,
        Self::LowRam,
    ];

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Balanced => "balanced",
            Self::Competitive => "competitive",
            Self::UltraFps => "ultra_fps",
            Self::BatterySaver => "battery_saver",
            Self::LowRam => "low_ram",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "off" => Some(Self::Off),
            "balanced" => Some(Self::Balanced),
            "competitive" => Some(Self::Competitive),
            "ultra_fps" => Some(Self::UltraFps),
            "battery_saver" => Some(Self::BatterySaver),
            "low_ram" => Some(Self::LowRam),
            _ => None,
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Off => "options.off",
            Self::Balanced => "volt.boost.preset.balanced",
            Self::Competitive => "volt.boost.preset.competitive",
            Self::UltraFps => "volt.boost.preset.ultra_fps",
            Self::BatterySaver => "volt.boost.preset.battery_saver",
            Self::LowRam => "volt.boost.preset.low_ram",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::Balanced => "Balanced",
            Self::Competitive => "Competitive PvP",
            Self::UltraFps => "Ultra FPS",
            Self::BatterySaver => "Battery Saver",
            Self::LowRam => "Low-RAM Rescue",
        }
    }
}

/// Live performance pill overlay (`Off`, `Compact` FPS/1%-low/RAM, `Full` telemetry + graph).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerfHudMode {
    #[default]
    Off,
    Compact,
    Full,
}

impl PerfHudMode {
    pub const ALL: [Self; 3] = [Self::Off, Self::Compact, Self::Full];

    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::Compact,
            Self::Compact => Self::Full,
            Self::Full => Self::Off,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Compact => "compact",
            Self::Full => "full",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "off" | "false" => Some(Self::Off),
            "compact" | "true" => Some(Self::Compact),
            "full" => Some(Self::Full),
            _ => None,
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Off => "options.off",
            Self::Compact => "volt.boost.perf_hud.compact",
            Self::Full => "volt.boost.perf_hud.full",
        }
    }
}

/// Per-frame chunk GPU upload limiter to prevent camera-turn frame spikes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChunkUploadCap {
    #[default]
    Auto,
    Smooth4,
    Smooth8,
    Balanced16,
    Burst32,
}

impl ChunkUploadCap {
    pub const ALL: [Self; 5] = [
        Self::Auto,
        Self::Smooth4,
        Self::Smooth8,
        Self::Balanced16,
        Self::Burst32,
    ];

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn max_uploads(self) -> Option<usize> {
        match self {
            Self::Auto => None,
            Self::Smooth4 => Some(4),
            Self::Smooth8 => Some(8),
            Self::Balanced16 => Some(16),
            Self::Burst32 => Some(32),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Smooth4 => "4",
            Self::Smooth8 => "8",
            Self::Balanced16 => "16",
            Self::Burst32 => "32",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "auto" => Some(Self::Auto),
            "4" | "smooth4" => Some(Self::Smooth4),
            "8" | "smooth8" => Some(Self::Smooth8),
            "16" | "balanced16" => Some(Self::Balanced16),
            "32" | "burst32" => Some(Self::Burst32),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Smooth4 => "4 / frame",
            Self::Smooth8 => "8 / frame",
            Self::Balanced16 => "16 / frame",
            Self::Burst32 => "32 / frame",
        }
    }
}

/// Maximum rendered entities (nearest-first priority so nearby combat targets never vanish).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityCap {
    #[default]
    Unlimited,
    Cap200,
    Cap128,
    Cap96,
    Cap64,
    Cap32,
}

impl EntityCap {
    pub const ALL: [Self; 6] = [
        Self::Unlimited,
        Self::Cap200,
        Self::Cap128,
        Self::Cap96,
        Self::Cap64,
        Self::Cap32,
    ];

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn limit(self) -> Option<usize> {
        match self {
            Self::Unlimited => None,
            Self::Cap200 => Some(200),
            Self::Cap128 => Some(128),
            Self::Cap96 => Some(96),
            Self::Cap64 => Some(64),
            Self::Cap32 => Some(32),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unlimited => "unlimited",
            Self::Cap200 => "200",
            Self::Cap128 => "128",
            Self::Cap96 => "96",
            Self::Cap64 => "64",
            Self::Cap32 => "32",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "unlimited" | "off" => Some(Self::Unlimited),
            "200" => Some(Self::Cap200),
            "128" => Some(Self::Cap128),
            "96" => Some(Self::Cap96),
            "64" => Some(Self::Cap64),
            "32" => Some(Self::Cap32),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Unlimited => "Unlimited",
            Self::Cap200 => "200 Max",
            Self::Cap128 => "128 Max",
            Self::Cap96 => "96 Max",
            Self::Cap64 => "64 Max",
            Self::Cap32 => "32 Max",
        }
    }
}

/// Dropped ground-item rendering cap & distance cutoffs for farm/explosion drops.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemRenderCap {
    #[default]
    Unlimited,
    Cap128,
    Cap64,
    Cap32,
}

impl ItemRenderCap {
    pub const ALL: [Self; 4] = [Self::Unlimited, Self::Cap128, Self::Cap64, Self::Cap32];

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn limit_and_dist_sq(self) -> Option<(usize, f64)> {
        match self {
            Self::Unlimited => None,
            Self::Cap128 => Some((128, 40.0 * 40.0)),
            Self::Cap64 => Some((64, 28.0 * 28.0)),
            Self::Cap32 => Some((32, 20.0 * 20.0)),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unlimited => "unlimited",
            Self::Cap128 => "128",
            Self::Cap64 => "64",
            Self::Cap32 => "32",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "unlimited" | "off" => Some(Self::Unlimited),
            "128" => Some(Self::Cap128),
            "64" => Some(Self::Cap64),
            "32" => Some(Self::Cap32),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Unlimited => "Unlimited",
            Self::Cap128 => "128 (40m)",
            Self::Cap64 => "64 (28m)",
            Self::Cap32 => "32 (20m)",
        }
    }
}

/// Chest / sign / banner / skull / shulker block-entity render cap + wall occlusion.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockEntityCap {
    #[default]
    Unlimited,
    Cap64,
    Cap32,
    Cap16,
}

impl BlockEntityCap {
    pub const ALL: [Self; 4] = [Self::Unlimited, Self::Cap64, Self::Cap32, Self::Cap16];

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn limit_and_dist_sq(self) -> Option<(usize, f64)> {
        match self {
            Self::Unlimited => None,
            Self::Cap64 => Some((64, 48.0 * 48.0)),
            Self::Cap32 => Some((32, 32.0 * 32.0)),
            Self::Cap16 => Some((16, 24.0 * 24.0)),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unlimited => "unlimited",
            Self::Cap64 => "64",
            Self::Cap32 => "32",
            Self::Cap16 => "16",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "unlimited" | "off" => Some(Self::Unlimited),
            "64" => Some(Self::Cap64),
            "32" => Some(Self::Cap32),
            "16" => Some(Self::Cap16),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Unlimited => "Unlimited",
            Self::Cap64 => "64 + Occlude",
            Self::Cap32 => "32 + Occlude",
            Self::Cap16 => "16 + Occlude",
        }
    }
}

/// Entity contact-shadow simplification mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FastShadowMode {
    #[default]
    Vanilla,
    Nearby16,
    Off,
}

impl FastShadowMode {
    pub const ALL: [Self; 3] = [Self::Vanilla, Self::Nearby16, Self::Off];

    pub fn next(self) -> Self {
        match self {
            Self::Vanilla => Self::Nearby16,
            Self::Nearby16 => Self::Off,
            Self::Off => Self::Vanilla,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vanilla => "vanilla",
            Self::Nearby16 => "nearby_16",
            Self::Off => "off",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "vanilla" => Some(Self::Vanilla),
            "nearby_16" | "nearby" => Some(Self::Nearby16),
            "off" => Some(Self::Off),
            _ => None,
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Vanilla => "volt.boost.shadows.vanilla",
            Self::Nearby16 => "volt.boost.shadows.nearby",
            Self::Off => "options.off",
        }
    }
}

/// Active particle engine cap & distance cutoff.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticleBudget {
    #[default]
    Vanilla16k,
    Cap4096,
    Cap2048,
    Cap512,
    Off,
}

impl ParticleBudget {
    pub const ALL: [Self; 5] = [
        Self::Vanilla16k,
        Self::Cap4096,
        Self::Cap2048,
        Self::Cap512,
        Self::Off,
    ];

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn max_particles(self) -> usize {
        match self {
            Self::Vanilla16k => 16_384,
            Self::Cap4096 => 4_096,
            Self::Cap2048 => 2_048,
            Self::Cap512 => 512,
            Self::Off => 0,
        }
    }

    pub fn max_distance_sq(self) -> Option<f64> {
        match self {
            Self::Vanilla16k => None,
            Self::Cap4096 => Some(36.0 * 36.0),
            Self::Cap2048 => Some(24.0 * 24.0),
            Self::Cap512 => Some(16.0 * 16.0),
            Self::Off => Some(0.0),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vanilla16k => "16384",
            Self::Cap4096 => "4096",
            Self::Cap2048 => "2048",
            Self::Cap512 => "512",
            Self::Off => "0",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "16384" | "vanilla" => Some(Self::Vanilla16k),
            "4096" => Some(Self::Cap4096),
            "2048" => Some(Self::Cap2048),
            "512" => Some(Self::Cap512),
            "0" | "off" => Some(Self::Off),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Vanilla16k => "16K (Vanilla)",
            Self::Cap4096 => "4,096 (36m)",
            Self::Cap2048 => "2,048 (24m)",
            Self::Cap512 => "512 (16m)",
            Self::Off => "OFF",
        }
    }
}

/// Enchantment glint & armor-trim shader simplification.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GlintTrimMode {
    #[default]
    Full,
    NearbyOnly,
    Off,
}

impl GlintTrimMode {
    pub const ALL: [Self; 3] = [Self::Full, Self::NearbyOnly, Self::Off];

    pub fn next(self) -> Self {
        match self {
            Self::Full => Self::NearbyOnly,
            Self::NearbyOnly => Self::Off,
            Self::Off => Self::Full,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::NearbyOnly => "nearby",
            Self::Off => "off",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "full" => Some(Self::Full),
            "nearby" => Some(Self::NearbyOnly),
            "off" => Some(Self::Off),
            _ => None,
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Full => "volt.boost.glint.full",
            Self::NearbyOnly => "volt.boost.glint.nearby",
            Self::Off => "options.off",
        }
    }
}

/// Automatic Dynamic Resolution Scaling (scales 3D scene while keeping HUD 100% native crisp).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoDrsMode {
    #[default]
    Off,
    Target60,
    Target120,
    Aggressive50,
}

impl AutoDrsMode {
    pub const ALL: [Self; 4] = [
        Self::Off,
        Self::Target60,
        Self::Target120,
        Self::Aggressive50,
    ];

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn target_and_floor(self) -> Option<(f32, f32)> {
        match self {
            Self::Off => None,
            Self::Target60 => Some((60.0, 0.65)),
            Self::Target120 => Some((120.0, 0.60)),
            Self::Aggressive50 => Some((60.0, 0.50)),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Target60 => "target_60",
            Self::Target120 => "target_120",
            Self::Aggressive50 => "aggressive_50",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "off" => Some(Self::Off),
            "target_60" => Some(Self::Target60),
            "target_120" => Some(Self::Target120),
            "aggressive_50" => Some(Self::Aggressive50),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::Target60 => "60 FPS (65-100%)",
            Self::Target120 => "120 FPS (60-100%)",
            Self::Aggressive50 => "Ultra (50-100%)",
        }
    }
}

/// Dynamic FPS cap when paused, in menus, or AFK-idle.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DynamicIdleFps {
    #[default]
    Off,
    Fps30,
    Fps15,
}

impl DynamicIdleFps {
    pub const ALL: [Self; 3] = [Self::Off, Self::Fps30, Self::Fps15];

    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::Fps30,
            Self::Fps30 => Self::Fps15,
            Self::Fps15 => Self::Off,
        }
    }

    pub fn cap_fps(self) -> Option<u32> {
        match self {
            Self::Off => None,
            Self::Fps30 => Some(30),
            Self::Fps15 => Some(15),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Fps30 => "30",
            Self::Fps15 => "15",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "off" => Some(Self::Off),
            "30" => Some(Self::Fps30),
            "15" => Some(Self::Fps15),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::Fps30 => "30 FPS Idle",
            Self::Fps15 => "15 FPS Eco",
        }
    }
}

/// Pause / menu background blur saver (skips multi-pass Kawase blur on mobile tile GPUs).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MenuBgMode {
    #[default]
    PanoramaBlur,
    FastDark,
}

impl MenuBgMode {
    pub const ALL: [Self; 2] = [Self::PanoramaBlur, Self::FastDark];

    pub fn next(self) -> Self {
        match self {
            Self::PanoramaBlur => Self::FastDark,
            Self::FastDark => Self::PanoramaBlur,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::PanoramaBlur => "blur",
            Self::FastDark => "fast_dark",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "blur" | "panorama" => Some(Self::PanoramaBlur),
            "fast_dark" | "fast" => Some(Self::FastDark),
            _ => None,
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::PanoramaBlur => "volt.boost.menu_bg.blur",
            Self::FastDark => "volt.boost.menu_bg.fast_dark",
        }
    }
}

/// Active simultaneous audio mixer voice cap to avoid CPU mixer spikes in mob farms/TNT.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioVoiceCap {
    #[default]
    Vanilla,
    Cap32,
    Cap16,
}

impl AudioVoiceCap {
    pub const ALL: [Self; 3] = [Self::Vanilla, Self::Cap32, Self::Cap16];

    pub fn next(self) -> Self {
        match self {
            Self::Vanilla => Self::Cap32,
            Self::Cap32 => Self::Cap16,
            Self::Cap16 => Self::Vanilla,
        }
    }

    pub fn max_per_tick(self) -> Option<usize> {
        match self {
            Self::Vanilla => None,
            Self::Cap32 => Some(8),
            Self::Cap16 => Some(4),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vanilla => "vanilla",
            Self::Cap32 => "32",
            Self::Cap16 => "16",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "vanilla" | "off" => Some(Self::Vanilla),
            "32" => Some(Self::Cap32),
            "16" => Some(Self::Cap16),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Vanilla => "Vanilla",
            Self::Cap32 => "32 Voices",
            Self::Cap16 => "16 Voices",
        }
    }
}
