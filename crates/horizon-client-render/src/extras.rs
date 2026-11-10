//! Horizon's optional extras (minimap, lighting, inventory tools): their
//! settings, kept in `horizon-extras.txt` next to `options.txt` so that file
//! stays exactly vanilla's, and the logic behind them. Every extra is off by
//! default; see `docs/extras.md`. Horizon's own switches outside the Extras page
//! (No AI, Agent Control, Discord Rich Presence) are kept here too.
use crate::volt_features::{
    ChunkUploadCap, FastShadowMode, GlintTrimMode, MinimapEntities, MinimapFrame,
    MinimapTerrainMode, PerfHudMode, WaypointColor,
};
use std::{collections::VecDeque, fs, path::Path};

pub const FILE: &str = "horizon-extras.txt";
/// The name the options screen uses while the sort key waits for a key.
pub const SORT_KEY: &str = "horizon.sort";
/// The names the options screen uses while a ported Volt keybind waits for a key.
pub const WAYPOINT_KEY: &str = "volt.waypoint_add";
pub const MINIMAP_ZOOM_KEY: &str = "volt.minimap_zoom";

/// The minimap's sizes in GUI pixels, and its zooms in blocks per pixel.
pub const MINIMAP_SIZES: [(u8, &str); 4] = [
    (64, "Small"),
    (96, "Medium"),
    (128, "Large"),
    (160, "XL"),
];
pub const MINIMAP_ZOOMS: [f32; 4] = [0.5, 1.0, 2.0, 4.0];
/// The minimap's opacities in percent (`minimapOpacity`).
pub const MINIMAP_OPACITIES: [u8; 4] = [50, 75, 90, 100];
/// The waypoint marker scales in percent.
pub const WAYPOINT_SCALES: [u8; 3] = [75, 100, 125];
/// The waypoint render distances in blocks (0 unlimited).
pub const WAYPOINT_DISTANCES: [u16; 5] = [0, 256, 512, 1024, 2048];

/// A screen corner for an overlay.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}
impl Corner {
    pub const ALL: [Corner; 4] = [
        Corner::TopLeft,
        Corner::TopRight,
        Corner::BottomLeft,
        Corner::BottomRight,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Corner::TopLeft => "top_left",
            Corner::TopRight => "top_right",
            Corner::BottomLeft => "bottom_left",
            Corner::BottomRight => "bottom_right",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Corner::TopLeft => "Top left",
            Corner::TopRight => "Top right",
            Corner::BottomLeft => "Bottom left",
            Corner::BottomRight => "Bottom right",
        }
    }
    fn parse(s: &str, default: Corner) -> Corner {
        Self::ALL
            .into_iter()
            .find(|c| c.name() == s)
            .unwrap_or(default)
    }
    pub fn top(self) -> bool {
        matches!(self, Corner::TopLeft | Corner::TopRight)
    }
    pub fn left(self) -> bool {
        matches!(self, Corner::TopLeft | Corner::BottomLeft)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Extras {
    pub fullbright: bool,
    /// Held, dropped and burning light sources light their surroundings (`dynamic_light`).
    pub dynamic_lights: bool,
    pub zoom: bool,
    /// A vanilla key name (`key.keyboard.z`).
    pub zoom_key: String,
    /// The zoom the key starts at.
    pub zoom_level: u8,
    /// Ease through vanilla's FOV easing (as the spyglass does).
    pub zoom_smooth: bool,
    /// Divide mouse turning by the zoom.
    pub zoom_sensitivity: bool,
    /// The FPS counter and keystrokes extras are official mods now: their
    /// old settings stay here, kept as they were.
    pub fps: bool,
    pub fps_corner: Corner,
    /// Also show the minimum and 1% low.
    pub fps_lows: bool,
    pub keystrokes: bool,
    pub keystrokes_corner: Corner,
    /// Clicks per second under the mouse buttons.
    pub keystrokes_cps: bool,
    /// The official mods once installed for the extras they replace, by mod
    /// id (`movedToMods`).
    pub moved: Vec<String>,
    pub minimap: bool,
    pub minimap_corner: Corner,
    /// Side in GUI pixels (`MINIMAP_SIZES`).
    pub minimap_size: u8,
    pub minimap_round: bool,
    /// Blocks per pixel (`MINIMAP_ZOOMS`).
    pub minimap_zoom: f32,
    /// Turn with the player (forward up) instead of north up.
    pub minimap_rotate: bool,
    /// Retry lost multiplayer connections (`reconnect`).
    pub auto_reconnect: bool,
    /// Positional sounds leave a few static sound channels free for UI sounds
    /// and others played to the listener (`sound_engine::HEARD_RESERVE`).
    pub ui_sound_reserve: bool,
    /// The player's own name tag over its model in third person.
    pub own_name_tag: bool,
    /// No particles from breaking or mining a block (other particles stay).
    pub hide_block_particles: bool,
    /// Mouse Tweaks-style drags in the inventory and storage containers.
    pub mouse_tweaks: bool,
    /// With them, the wheel over a slot moves its items one at a time.
    pub mouse_tweaks_wheel: bool,
    /// Sort buttons on the inventory and chest-like containers.
    pub sort: bool,
    /// A vanilla key name: sorts the part under the cursor.
    pub sort_key: String,
    /// A new chat line slides in from the left while it fades in (`Chat::layout`).
    pub chat_animation: bool,
    /// The indicator bar at the left of a chat line (on by default, as vanilla).
    pub chat_indicator: bool,
    /// No AI: every AI feature hidden and off (`horizon_core::no_ai`; the launcher writes it too).
    pub no_ai: bool,
    /// Agents may control the game the player starts: it opens its control socket for
    /// horizon-ctl and MCP clients (off by default; an agent's own launch always opens it).
    pub agent_control: bool,
    /// F3+R and F3+E start/stop a repro recording and write its bundle (off by
    /// default: a recording copies the world).
    pub repro_keys: bool,
    /// "Playing Horizon" on the player's Discord profile (`discord_presence`; on by default).
    pub discord_presence: bool,
    /// Another controller can join P1's world (`splitscreen`, off by default).
    pub splitscreen: bool,
    /// Two players in one window: 0 auto (side by side when wider than tall), 1 side by side,
    /// 2 top and bottom (`splitscreenTwo`).
    pub splitscreen_two: u8,
    /// Three players on a screen: 0 one wide on top, 1 one tall on the left, 2 side by side
    /// (`splitscreenThree`).
    pub splitscreen_three: u8,
    /// Splitscreen: each player's name in the corner of their view (`splitscreenNames`).
    pub splitscreen_names: bool,
    /// The wheel over a cycle button in the settings changes it (`CycleButton.mouseScrolled`,
    /// on by default as vanilla); off, the wheel only scrolls the list.
    pub wheel_cycles_settings: bool,
    /// The built-in music tracks the Music page turned off (sound ids): they
    /// leave every pool they play in (`musicOff`, empty by default: vanilla's
    /// random selection).
    pub music_off: Vec<String>,
    /// The Music page: the Music... button on Music & Sounds, playing a
    /// built-in track and leaving the off ones out (`musicPage`, off by
    /// default: vanilla plays exactly as before).
    pub music_page: bool,
    /// Cosmetics (`horizon_cosmetics`, off unless enabled): the player's own, and the other
    /// players' (`cosmeticsMine`, `cosmeticsOthers`; both on by default).
    pub cosmetics_mine: bool,
    pub cosmetics_others: bool,
    /// Volt Boost's big.LITTLE thread tuning on Android (horizon-threads'
    /// 50 us timer slack): off, as vanilla's scheduling.
    pub thread_boost: bool,
    /// Volt Boost's display refresh rate request on Android, in Hz (0: the
    /// system's own choice; else one of `DISPLAY_RATES`). The frame cap's
    /// own vote stays as it is when this is 0.
    pub display_hz: u16,
    /// Volt Boost's ADPF performance hints on Android (`frame_hint`'s
    /// session): on, as the game has always asked for the boost.
    pub performance_hints: bool,
    /// The selected Volt Boost profile, as the switches below hold it
    /// (`apply_volt_preset` writes it; the profiles themselves grow with the
    /// ported switches).
    pub volt_preset: crate::volt_features::VoltBoostPreset,
    /// Volt Boost's dynamic idle FPS: a 30 or 15 FPS cap while the game is
    /// paused, in a menu, or shows no world (`framerate_limit`).
    pub dynamic_fps: crate::volt_features::DynamicIdleFps,
    /// Volt Boost's particle budget: the active cap (16,384 as vanilla) and
    /// the distance past which nothing renders (`particles.rs`).
    pub particle_budget: crate::volt_features::ParticleBudget,
    /// Volt Boost's audio voice cap: how many of a tick's particle sounds
    /// play (all of them as vanilla).
    pub audio_voice_cap: crate::volt_features::AudioVoiceCap,
    /// Volt Boost's Auto-DRS: the world's resolution eases towards a frame
    /// target while the HUD stays native (`extras_frame`).
    pub auto_drs: crate::volt_features::AutoDrsMode,
    /// Volt Boost's thermal guard: under load, the resolution and the
    /// particle budget ease down until the phone cools.
    pub thermal_guard: bool,
    /// Volt Boost's menu backdrop saver: FastDark skips the multi-pass menu
    /// blur for a plain dark backdrop (PanoramaBlur is the vanilla blur).
    pub menu_bg_mode: crate::volt_features::MenuBgMode,
    /// Volt Boost's animation throttle: animated sprites upload every second
    /// tick (10 Hz instead of vanilla's 20 Hz).
    pub anim_throttle: bool,
    /// Volt Boost's active-animation cull off switch: on, the engine skips
    /// animated sprites no resident section uses (this engine's default).
    pub active_anim_cull: bool,
    /// Volt Boost's fast translucency: distant water and glass re-sort on a
    /// coarser grid with a smaller rotating budget.
    pub fast_translucency: bool,
    /// Volt Boost's fast weather: rain and snow draw within four blocks.
    pub fast_weather: bool,
    /// Volt Boost's sign-text cull: signs' standing text is skipped past
    /// sixteen blocks (the block itself stays).
    pub cull_sign_text: bool,
    /// Volt Boost's distant mob throttle: far mobs' bodies and player layers
    /// draw without their animation alpha.
    pub distant_mob_throttle: bool,
    /// Volt Boost's hub lag shield: in a world of more than 24 mobs, players
    /// beyond nametag range give up their render slot first.
    pub hub_lag_shield: bool,
    /// Volt Boost's chaos shield: explosions throw at most 24 debris
    /// particles, and a far one starts no firework show.
    pub chaos_particle_shield: bool,
    /// Volt Boost's particle frustum cull: particles behind the camera skip
    /// their sprite and light lookups.
    pub particle_frustum_cull: bool,
    /// Volt Boost's memory saver: on a world or dimension change, the
    /// emptied queues and the allocator's freed pages go back to the OS.
    pub memory_saver: bool,
    /// Volt Boost's streaming lookahead: chunks behind the player's
    /// movement and look generate later.
    pub lookahead_stream: bool,
    /// Volt Boost's cave sky cull: sealed underground in the Overworld, the
    /// weather draw is skipped.
    pub cave_sky_cull: bool,
    /// Volt Boost's frustum-priority meshing: sections behind the view wait
    /// behind every visible one in the upload order.
    pub frustum_priority_mesh: bool,
    /// Volt Boost's entity render caps: past the cap only the nearest mobs,
    /// dropped items or block entities draw (rendering only), and past the
    /// mode's distance nothing does.
    pub entity_cap: crate::volt_features::EntityCap,
    pub item_render_cap: crate::volt_features::ItemRenderCap,
    pub block_entity_cap: crate::volt_features::BlockEntityCap,
    /// Volt Reflex: the GPU's queue is worked before the frame's sleep, with
    /// precise pacing while playing.
    pub volt_reflex: bool,
    // --- The ported Volt minimap extras (off by default) ---
    /// The minimap bezel style (Volt Crimson, Classic Dark or Clean Minimal).
    pub minimap_frame: crate::volt_features::MinimapFrame,
    /// Map opacity percentage (`50`, `75`, `90` or `100`).
    pub minimap_opacity: u8,
    /// Auto (cave slice underground), Surface or Cave.
    pub minimap_terrain_mode: crate::volt_features::MinimapTerrainMode,
    pub minimap_chunk_grid: bool,
    /// The entity dots the radar plots.
    pub minimap_entities: crate::volt_features::MinimapEntities,
    pub minimap_coords: bool,
    pub minimap_biome: bool,
    pub minimap_time: bool,
    /// A vanilla key name that cycles the minimap's zoom (`key.keyboard.unknown` unbound).
    pub minimap_zoom_key: String,
    // --- The ported Volt waypoints (off by default) ---
    pub waypoints: bool,
    pub waypoint_beams: bool,
    pub waypoint_labels: bool,
    pub waypoint_distance: bool,
    pub waypoint_coords: bool,
    /// Marker and nameplate size percentage (`75`, `100` or `125`).
    pub waypoint_scale: u8,
    /// Blocks past which markers hide (0 unlimited; `WAYPOINT_DISTANCES`).
    pub waypoint_max_distance: u16,
    pub waypoint_edge_arrows: bool,
    pub waypoint_auto_death: bool,
    pub waypoint_dimension_sync: bool,
    pub waypoint_on_minimap: bool,
    pub waypoint_hud_button: bool,
    pub waypoint_new_color: crate::volt_features::WaypointColor,
    /// A vanilla key name that drops a waypoint at the player (`key.keyboard.b`).
    pub waypoint_key: String,
    // --- The ported Volt Boost telemetry and Tier 1 switches ---
    /// The live performance HUD (Off, Compact or Full graph).
    pub perf_hud: crate::volt_features::PerfHudMode,
    pub perf_hud_corner: Corner,
    /// Smart leaf cull: hidden interior leaf-to-leaf faces are skipped.
    pub cull_leaves: bool,
    /// Chunk mesh uploads per frame (Auto, 4, 8, 16 or 32).
    pub chunk_upload_cap: crate::volt_features::ChunkUploadCap,
    /// Distant LOD boost: outer render rings use the low-cost LOD band.
    pub distant_lod_boost: bool,
    /// Entity contact shadows (Vanilla, Nearby 16m or Off).
    pub fast_shadows: crate::volt_features::FastShadowMode,
    /// Enchantment glint and armor trim detail (Full, Nearby Only or Off).
    pub glint_mode: crate::volt_features::GlintTrimMode,
    /// Pre-warm caches and pipelines during startup.
    pub prewarm_pipelines: bool,
    /// Keep the display awake during long sessions (Android).
    pub keep_screen_awake: bool,
    /// Transient one-shot flag triggered when the player asks to purge unused RAM now.
    pub purge_ram_requested: bool,
    // --- The ported Volt Shaders, QoL, Chat, Visuals, Boost+ and Mod catalog ---
    /// Built-in native WGSL Volt Shaders (`enabled: false` by default).
    pub volt_shaders: horizon_shader_api::volt_shaders::VoltShadersConfig,
    /// Built-in Volt QoL suite (zoom, freelook, blueprint, viewmodel...).
    pub volt_qol: crate::volt_qol::VoltQolConfig,
    /// Built-in Volt Chat and 3D proximity voice chat.
    pub volt_chat: crate::volt_qol::VoltChatConfig,
    /// Built-in Volt Visuals (connected glass, better grass, 3D items).
    pub volt_visuals: crate::volt_qol::VoltVisualsConfig,
    /// Built-in Volt Boost+ render passes (greedy merge, cave occlusion, mesh pool).
    pub volt_boost_plus: crate::volt_qol::VoltBoostPlusConfig,
    /// Built-in native Volt mods (all off by default).
    pub volt_mods: crate::volt_qol::VoltModsConfig,
    /// The native Volt mod catalog state (synced between launcher and Mod Menu).
    pub mod_catalog: crate::volt_mods::VoltModCatalogState,
}
impl Default for Extras {
    fn default() -> Self {
        Self {
            fullbright: false,
            dynamic_lights: false,
            zoom: false,
            zoom_key: "key.keyboard.z".into(),
            zoom_level: 4,
            zoom_smooth: true,
            zoom_sensitivity: true,
            fps: false,
            fps_corner: Corner::TopLeft,
            fps_lows: false,
            keystrokes: false,
            keystrokes_corner: Corner::TopLeft,
            keystrokes_cps: true,
            moved: Vec::new(),
            minimap: false,
            minimap_corner: Corner::TopRight,
            minimap_size: 96,
            minimap_round: false,
            minimap_zoom: 1.0,
            minimap_rotate: false,
            auto_reconnect: false,
            ui_sound_reserve: false,
            own_name_tag: false,
            hide_block_particles: false,
            mouse_tweaks: false,
            mouse_tweaks_wheel: true,
            sort: false,
            sort_key: "key.keyboard.unknown".into(),
            chat_animation: false,
            chat_indicator: true,
            no_ai: false,
            agent_control: false,
            repro_keys: false,
            discord_presence: true,
            splitscreen: false,
            splitscreen_two: 0,
            splitscreen_three: 0,
            splitscreen_names: true,
            wheel_cycles_settings: true,
            music_off: Vec::new(),
            music_page: false,
            cosmetics_mine: true,
            cosmetics_others: true,
            thread_boost: false,
            display_hz: 0,
            performance_hints: true,
            volt_preset: crate::volt_features::VoltBoostPreset::Off,
            dynamic_fps: crate::volt_features::DynamicIdleFps::Off,
            particle_budget: crate::volt_features::ParticleBudget::Vanilla16k,
            audio_voice_cap: crate::volt_features::AudioVoiceCap::Vanilla,
            auto_drs: crate::volt_features::AutoDrsMode::Off,
            thermal_guard: false,
            volt_reflex: false,
            entity_cap: crate::volt_features::EntityCap::Unlimited,
            item_render_cap: crate::volt_features::ItemRenderCap::Unlimited,
            block_entity_cap: crate::volt_features::BlockEntityCap::Unlimited,
            menu_bg_mode: crate::volt_features::MenuBgMode::PanoramaBlur,
            anim_throttle: false,
            active_anim_cull: true,
            fast_translucency: false,
            fast_weather: false,
            cull_sign_text: false,
            distant_mob_throttle: false,
            hub_lag_shield: false,
            chaos_particle_shield: false,
            particle_frustum_cull: false,
            memory_saver: false,
            lookahead_stream: false,
            cave_sky_cull: false,
            frustum_priority_mesh: false,
            minimap_frame: crate::volt_features::MinimapFrame::VoltCrimson,
            minimap_opacity: 100,
            minimap_terrain_mode: crate::volt_features::MinimapTerrainMode::Auto,
            minimap_chunk_grid: false,
            minimap_entities: crate::volt_features::MinimapEntities::All,
            minimap_coords: true,
            minimap_biome: true,
            minimap_time: false,
            minimap_zoom_key: "key.keyboard.unknown".into(),
            waypoints: false,
            waypoint_beams: true,
            waypoint_labels: true,
            waypoint_distance: true,
            waypoint_coords: false,
            waypoint_scale: 100,
            waypoint_max_distance: 0,
            waypoint_edge_arrows: true,
            waypoint_auto_death: true,
            waypoint_dimension_sync: true,
            waypoint_on_minimap: true,
            waypoint_hud_button: false,
            waypoint_new_color: crate::volt_features::WaypointColor::Crimson,
            waypoint_key: "key.keyboard.b".into(),
            perf_hud: crate::volt_features::PerfHudMode::Off,
            perf_hud_corner: Corner::TopLeft,
            cull_leaves: false,
            chunk_upload_cap: crate::volt_features::ChunkUploadCap::Auto,
            distant_lod_boost: false,
            fast_shadows: crate::volt_features::FastShadowMode::Vanilla,
            glint_mode: crate::volt_features::GlintTrimMode::Full,
            prewarm_pipelines: false,
            keep_screen_awake: false,
            purge_ram_requested: false,
            volt_shaders: horizon_shader_api::volt_shaders::VoltShadersConfig::default(),
            volt_qol: crate::volt_qol::VoltQolConfig::default(),
            volt_chat: crate::volt_qol::VoltChatConfig::default(),
            volt_visuals: crate::volt_qol::VoltVisualsConfig::default(),
            volt_boost_plus: crate::volt_qol::VoltBoostPlusConfig::default(),
            volt_mods: crate::volt_qol::VoltModsConfig::default(),
            mod_catalog: crate::volt_mods::VoltModCatalogState::default(),
        }
    }
}

/// The display rates the Display refresh rate switch cycles through (0: the
/// system's own choice).
pub const DISPLAY_RATES: [u16; 7] = [0, 30, 60, 90, 120, 144, 165];
fn bool_text(v: bool) -> &'static str {
    if v { "true" } else { "false" }
}
impl Extras {
    pub fn load(dir: &Path) -> Self {
        fs::read_to_string(dir.join(FILE))
            .map(|t| Self::parse(&t))
            .unwrap_or_default()
    }
    pub fn parse(text: &str) -> Self {
        let mut e = Self::default();
        for line in text.lines() {
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            let value = value.trim();
            let flag = |d: bool| match value {
                "true" => true,
                "false" => false,
                _ => d,
            };
            match key.trim() {
                "fullbright" => e.fullbright = flag(e.fullbright),
                "dynamicLights" => e.dynamic_lights = flag(e.dynamic_lights),
                "zoom" => e.zoom = flag(e.zoom),
                "zoomKey" if value.starts_with("key.") => e.zoom_key = value.to_owned(),
                "zoomLevel" => {
                    if let Ok(v) = value.parse::<u8>() {
                        e.zoom_level = v.clamp(2, 10);
                    }
                }
                "zoomSmooth" => e.zoom_smooth = flag(e.zoom_smooth),
                "zoomSensitivity" => e.zoom_sensitivity = flag(e.zoom_sensitivity),
                "fps" => e.fps = flag(e.fps),
                "fpsCorner" => e.fps_corner = Corner::parse(value, e.fps_corner),
                "fpsLows" => e.fps_lows = flag(e.fps_lows),
                "keystrokes" => e.keystrokes = flag(e.keystrokes),
                "keystrokesCorner" => {
                    e.keystrokes_corner = Corner::parse(value, e.keystrokes_corner)
                }
                "keystrokesCps" => e.keystrokes_cps = flag(e.keystrokes_cps),
                "movedToMods" => {
                    e.moved = value
                        .split(',')
                        .map(str::trim)
                        .filter(|id| !id.is_empty())
                        .map(str::to_owned)
                        .collect()
                }
                "minimap" => e.minimap = flag(e.minimap),
                "minimapCorner" => e.minimap_corner = Corner::parse(value, e.minimap_corner),
                "minimapSize" => {
                    if let Some(&(v, _)) = MINIMAP_SIZES.iter().find(|s| s.0.to_string() == value) {
                        e.minimap_size = v;
                    }
                }
                "minimapShape" => e.minimap_round = value == "round",
                "minimapZoom" => {
                    if let Some(&z) = MINIMAP_ZOOMS.iter().find(|z| z.to_string() == value) {
                        e.minimap_zoom = z;
                    }
                }
                "minimapRotate" => e.minimap_rotate = flag(e.minimap_rotate),
                "autoReconnect" => e.auto_reconnect = flag(e.auto_reconnect),
                "uiSoundReserve" => e.ui_sound_reserve = flag(e.ui_sound_reserve),
                "ownNameTag" => e.own_name_tag = flag(e.own_name_tag),
                "hideBlockParticles" => e.hide_block_particles = flag(e.hide_block_particles),
                "mouseTweaks" => e.mouse_tweaks = flag(e.mouse_tweaks),
                "mouseTweaksWheel" => e.mouse_tweaks_wheel = flag(e.mouse_tweaks_wheel),
                "sort" => e.sort = flag(e.sort),
                "sortKey" if value.starts_with("key.") => e.sort_key = value.to_owned(),
                "chatAnimation" => e.chat_animation = flag(e.chat_animation),
                "chatIndicator" => e.chat_indicator = flag(e.chat_indicator),
                horizon_core::no_ai::KEY => e.no_ai = flag(e.no_ai),
                "agentControl" => e.agent_control = flag(e.agent_control),
                "reproKeys" => e.repro_keys = flag(e.repro_keys),
                "discordPresence" => e.discord_presence = flag(e.discord_presence),
                "splitscreen" => e.splitscreen = flag(e.splitscreen),
                "splitscreenTwo" => e.splitscreen_two = value.parse().unwrap_or(0).min(2),
                "splitscreenThree" => e.splitscreen_three = value.parse().unwrap_or(0).min(2),
                "splitscreenNames" => e.splitscreen_names = flag(e.splitscreen_names),
                "wheelCyclesSettings" => e.wheel_cycles_settings = flag(e.wheel_cycles_settings),
                "musicOff" => {
                    e.music_off = value
                        .split(',')
                        .map(str::trim)
                        .filter(|id| !id.is_empty())
                        .map(str::to_owned)
                        .collect()
                }
                "musicPage" => e.music_page = flag(e.music_page),
                "cosmeticsMine" => e.cosmetics_mine = flag(e.cosmetics_mine),
                "cosmeticsOthers" => e.cosmetics_others = flag(e.cosmetics_others),
                "threadBoost" => e.thread_boost = flag(e.thread_boost),
                "displayHz" => {
                    e.display_hz = value
                        .parse::<u16>()
                        .ok()
                        .filter(|hz| DISPLAY_RATES.contains(hz))
                        .unwrap_or(0);
                }
                "performanceHints" => e.performance_hints = flag(e.performance_hints),
                "voltPreset" => {
                    if let Some(p) = crate::volt_features::VoltBoostPreset::parse(value) {
                        e.volt_preset = p;
                    }
                }
                "dynamicFps" => {
                    if let Some(m) = crate::volt_features::DynamicIdleFps::parse(value) {
                        e.dynamic_fps = m;
                    }
                }
                "particleBudget" => {
                    if let Some(b) = crate::volt_features::ParticleBudget::parse(value) {
                        e.particle_budget = b;
                    }
                }
                "audioVoiceCap" => {
                    if let Some(c) = crate::volt_features::AudioVoiceCap::parse(value) {
                        e.audio_voice_cap = c;
                    }
                }
                "autoDrs" => {
                    if let Some(m) = crate::volt_features::AutoDrsMode::parse(value) {
                        e.auto_drs = m;
                    }
                }
                "thermalGuard" => e.thermal_guard = flag(e.thermal_guard),
                "voltReflex" => e.volt_reflex = flag(e.volt_reflex),
                "entityCap" => {
                    if let Some(c) = crate::volt_features::EntityCap::parse(value) {
                        e.entity_cap = c;
                    }
                }
                "itemRenderCap" => {
                    if let Some(c) = crate::volt_features::ItemRenderCap::parse(value) {
                        e.item_render_cap = c;
                    }
                }
                "blockEntityCap" => {
                    if let Some(c) = crate::volt_features::BlockEntityCap::parse(value) {
                        e.block_entity_cap = c;
                    }
                }
                "menuBgMode" => {
                    if let Some(m) = crate::volt_features::MenuBgMode::parse(value) {
                        e.menu_bg_mode = m;
                    }
                }
                "animThrottle" => e.anim_throttle = flag(e.anim_throttle),
                "activeAnimCull" => e.active_anim_cull = flag(e.active_anim_cull),
                "fastTranslucency" => e.fast_translucency = flag(e.fast_translucency),
                "fastWeather" => e.fast_weather = flag(e.fast_weather),
                "cullSignText" => e.cull_sign_text = flag(e.cull_sign_text),
                "distantMobThrottle" => e.distant_mob_throttle = flag(e.distant_mob_throttle),
                "hubLagShield" => e.hub_lag_shield = flag(e.hub_lag_shield),
                "chaosShield" => e.chaos_particle_shield = flag(e.chaos_particle_shield),
                "particleFrustum" => e.particle_frustum_cull = flag(e.particle_frustum_cull),
                "memorySaver" => e.memory_saver = flag(e.memory_saver),
                "lookaheadStream" => e.lookahead_stream = flag(e.lookahead_stream),
                "caveSkyCull" => e.cave_sky_cull = flag(e.cave_sky_cull),
                "frustumPriorityMesh" => e.frustum_priority_mesh = flag(e.frustum_priority_mesh),
                // The ported Volt minimap extras:
                "minimapFrame" => e.minimap_frame = MinimapFrame::parse(value),
                "minimapOpacity" => {
                    if let Some(&op) = MINIMAP_OPACITIES.iter().find(|o| o.to_string() == value) {
                        e.minimap_opacity = op;
                    }
                }
                "minimapTerrain" => e.minimap_terrain_mode = MinimapTerrainMode::parse(value),
                "minimapChunkGrid" => e.minimap_chunk_grid = flag(e.minimap_chunk_grid),
                "minimapEntities" => e.minimap_entities = MinimapEntities::parse(value),
                "minimapCoords" => e.minimap_coords = flag(e.minimap_coords),
                "minimapBiome" => e.minimap_biome = flag(e.minimap_biome),
                "minimapTime" => e.minimap_time = flag(e.minimap_time),
                "minimapZoomKey" if value.starts_with("key.") => {
                    e.minimap_zoom_key = value.to_owned()
                }
                // The ported Volt waypoints:
                "waypoints" => e.waypoints = flag(e.waypoints),
                "waypointBeams" => e.waypoint_beams = flag(e.waypoint_beams),
                "waypointLabels" => e.waypoint_labels = flag(e.waypoint_labels),
                "waypointDistance" => e.waypoint_distance = flag(e.waypoint_distance),
                "waypointCoords" => e.waypoint_coords = flag(e.waypoint_coords),
                "waypointScale" => {
                    if let Some(&sc) = WAYPOINT_SCALES.iter().find(|s| s.to_string() == value) {
                        e.waypoint_scale = sc;
                    }
                }
                "waypointMaxDistance" => {
                    if let Some(&d) = WAYPOINT_DISTANCES.iter().find(|d| d.to_string() == value) {
                        e.waypoint_max_distance = d;
                    }
                }
                "waypointEdgeArrows" => e.waypoint_edge_arrows = flag(e.waypoint_edge_arrows),
                "waypointAutoDeath" => e.waypoint_auto_death = flag(e.waypoint_auto_death),
                "waypointDimensionSync" => {
                    e.waypoint_dimension_sync = flag(e.waypoint_dimension_sync)
                }
                "waypointOnMinimap" => e.waypoint_on_minimap = flag(e.waypoint_on_minimap),
                "waypointHudButton" => e.waypoint_hud_button = flag(e.waypoint_hud_button),
                "waypointNewColor" => {
                    if let Some(&c) = WaypointColor::ALL
                        .iter()
                        .find(|c| c.label().eq_ignore_ascii_case(value))
                    {
                        e.waypoint_new_color = c;
                    }
                }
                "waypointKey" if value.starts_with("key.") => e.waypoint_key = value.to_owned(),
                // The ported Volt Boost telemetry and Tier 1 switches:
                "voltPerfHud" => {
                    if let Some(v) = PerfHudMode::parse(value) {
                        e.perf_hud = v;
                    }
                }
                "voltPerfHudCorner" => {
                    e.perf_hud_corner = Corner::parse(value, e.perf_hud_corner)
                }
                "voltCullLeaves" => e.cull_leaves = flag(e.cull_leaves),
                "voltChunkUploadCap" => {
                    if let Some(v) = ChunkUploadCap::parse(value) {
                        e.chunk_upload_cap = v;
                    }
                }
                "voltDistantLodManual" => e.distant_lod_boost = flag(e.distant_lod_boost),
                "voltFastShadows" => {
                    if let Some(v) = FastShadowMode::parse(value) {
                        e.fast_shadows = v;
                    }
                }
                "voltGlintMode" => {
                    if let Some(v) = GlintTrimMode::parse(value) {
                        e.glint_mode = v;
                    }
                }
                "voltPrewarm" => e.prewarm_pipelines = flag(e.prewarm_pipelines),
                "voltKeepAwake" => e.keep_screen_awake = flag(e.keep_screen_awake),
                // The ported Volt Shaders:
                "voltShadersEnabled" => e.volt_shaders.enabled = flag(e.volt_shaders.enabled),
                "voltShadersProfile" => {
                    if let Some(p) =
                        horizon_shader_api::volt_shaders::VoltShaderProfile::parse(value)
                    {
                        e.volt_shaders.profile = p;
                    }
                }
                "voltShaderWavePlants" => {
                    e.volt_shaders.waving_plants = flag(e.volt_shaders.waving_plants)
                }
                "voltShaderWaveLeaves" => {
                    e.volt_shaders.waving_leaves = flag(e.volt_shaders.waving_leaves)
                }
                "voltShaderWaveWater" => {
                    e.volt_shaders.waving_water = flag(e.volt_shaders.waving_water)
                }
                "voltShaderWindSpeed" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_shaders.wind_speed = v.clamp(25, 200);
                    }
                }
                "voltShaderWindStrength" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_shaders.wind_strength = v.clamp(25, 200);
                    }
                }
                "voltShaderStormWind" => {
                    e.volt_shaders.storm_wind = flag(e.volt_shaders.storm_wind)
                }
                "voltShaderSunShading" => {
                    e.volt_shaders.sun_shading = flag(e.volt_shaders.sun_shading)
                }
                "voltShaderShadowRichness" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_shaders.shadow_richness = v.clamp(0, 100);
                    }
                }
                "voltShaderFoliageGlow" => {
                    e.volt_shaders.foliage_subsurface = flag(e.volt_shaders.foliage_subsurface)
                }
                "voltShaderTorchStyle" => {
                    if let Some(v) =
                        horizon_shader_api::volt_shaders::TorchLightStyle::parse(value)
                    {
                        e.volt_shaders.torch_style = v;
                    }
                }
                "voltShaderTorchIntensity" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_shaders.torch_intensity = v.clamp(50, 200);
                    }
                }
                "voltShaderEmissiveGlow" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_shaders.emissive_glow = v.clamp(0, 200);
                    }
                }
                "voltShaderRainWetness" => {
                    e.volt_shaders.rain_wetness = flag(e.volt_shaders.rain_wetness)
                }
                "voltShaderWaterRipples" => {
                    e.volt_shaders.water_ripples = flag(e.volt_shaders.water_ripples)
                }
                "voltShaderWaterReflection" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_shaders.water_reflection = v.clamp(0, 150);
                    }
                }
                "voltShaderWaterGlint" => {
                    e.volt_shaders.water_glint = flag(e.volt_shaders.water_glint)
                }
                "voltShaderClearWater" => {
                    e.volt_shaders.clear_water = flag(e.volt_shaders.clear_water)
                }
                "voltShaderUnderwaterRefract" => {
                    e.volt_shaders.underwater_refraction =
                        flag(e.volt_shaders.underwater_refraction)
                }
                "voltShaderCustomSky" => {
                    e.volt_shaders.custom_sky = flag(e.volt_shaders.custom_sky)
                }
                "voltShaderSunsetGlow" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_shaders.sunset_glow = v.clamp(0, 200);
                    }
                }
                "voltShaderSunHalo" => {
                    e.volt_shaders.sun_halo = flag(e.volt_shaders.sun_halo)
                }
                "voltShaderAurora" => {
                    if let Some(v) = horizon_shader_api::volt_shaders::AuroraMode::parse(value) {
                        e.volt_shaders.aurora = v;
                    }
                }
                "voltShaderDepthHaze" => {
                    if let Some(v) = horizon_shader_api::volt_shaders::DepthHazeMode::parse(value)
                    {
                        e.volt_shaders.depth_haze = v;
                    }
                }
                "voltShaderTonemap" => {
                    if let Some(v) = horizon_shader_api::volt_shaders::TonemapMode::parse(value) {
                        e.volt_shaders.tonemap = v;
                    }
                }
                "voltShaderBloom" => {
                    if let Some(v) = horizon_shader_api::volt_shaders::BloomMode::parse(value) {
                        e.volt_shaders.bloom = v;
                    }
                }
                "voltShaderSunRays" => e.volt_shaders.sun_rays = flag(e.volt_shaders.sun_rays),
                "voltShaderVibrance" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_shaders.vibrance = v.clamp(50, 180);
                    }
                }
                "voltShaderSaturation" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_shaders.saturation = v.clamp(50, 180);
                    }
                }
                "voltShaderContrast" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_shaders.contrast = v.clamp(70, 150);
                    }
                }
                "voltShaderExposure" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_shaders.exposure = v.clamp(70, 150);
                    }
                }
                "voltShaderWarmth" => {
                    if let Ok(v) = value.parse::<i16>() {
                        e.volt_shaders.warmth = v.clamp(-50, 50);
                    }
                }
                "voltShaderVignette" => {
                    if let Some(v) = horizon_shader_api::volt_shaders::VignetteMode::parse(value) {
                        e.volt_shaders.vignette = v;
                    }
                }
                "voltShaderStyle" => {
                    if let Some(v) =
                        horizon_shader_api::volt_shaders::ShaderVisualStyle::parse(value)
                    {
                        e.volt_shaders.style = v;
                    }
                }
                "voltShaderCloudShadows" => {
                    e.volt_shaders.cloud_shadows = flag(e.volt_shaders.cloud_shadows)
                }
                "voltShaderGlowingOres" => {
                    e.volt_shaders.glowing_ores = flag(e.volt_shaders.glowing_ores)
                }
                "voltShaderOreGlow" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_shaders.ore_glow = v.clamp(25, 200);
                    }
                }
                "voltShaderAutoPbr" => e.volt_shaders.auto_pbr = flag(e.volt_shaders.auto_pbr),
                "voltShaderWaterFoam" => {
                    e.volt_shaders.water_foam = flag(e.volt_shaders.water_foam)
                }
                "voltShaderCloudsMode" => {
                    if let Some(v) = horizon_shader_api::volt_shaders::CloudMode::parse(value) {
                        e.volt_shaders.clouds_mode = v;
                    }
                }
                "voltShaderCloudCoverage" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_shaders.cloud_coverage = v.clamp(30, 160);
                    }
                }
                "voltShaderNightSkyPlus" => {
                    e.volt_shaders.night_sky_plus = flag(e.volt_shaders.night_sky_plus)
                }
                "voltShaderRainbow" => {
                    e.volt_shaders.post_rain_rainbow = flag(e.volt_shaders.post_rain_rainbow)
                }
                "voltShaderEyeComfort" => {
                    e.volt_shaders.eye_comfort = flag(e.volt_shaders.eye_comfort)
                }
                // The ported Volt Visuals and Volt Boost+ passes:
                "voltCtmGlass" => {
                    e.volt_visuals.connected_glass = flag(e.volt_visuals.connected_glass)
                }
                "voltBetterGrass" => {
                    e.volt_visuals.better_grass_sides = flag(e.volt_visuals.better_grass_sides)
                }
                "voltDroppedItems3d" => {
                    e.volt_visuals.dropped_items_3d = flag(e.volt_visuals.dropped_items_3d)
                }
                "voltGreedyQuadMerge" => {
                    e.volt_boost_plus.greedy_quad_merge = flag(e.volt_boost_plus.greedy_quad_merge)
                }
                "voltCaveOcclusion" => {
                    e.volt_boost_plus.cave_chunk_occlusion =
                        flag(e.volt_boost_plus.cave_chunk_occlusion)
                }
                "voltZeroStutterPool" => {
                    e.volt_boost_plus.zero_stutter_mesh_pool =
                        flag(e.volt_boost_plus.zero_stutter_mesh_pool)
                }
                "voltOffscreenEntitySkip" => {
                    e.volt_boost_plus.offscreen_entity_skip =
                        flag(e.volt_boost_plus.offscreen_entity_skip)
                }
                "voltSmartFramePacer" => {
                    e.volt_boost_plus.smart_frame_pacer = flag(e.volt_boost_plus.smart_frame_pacer)
                }
                // The ported Volt QoL suite:
                "voltQolZoom" => e.volt_qol.zoom_enabled = flag(e.volt_qol.zoom_enabled),
                "voltQolZoomFov" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_qol.zoom_fov_pct = v.clamp(10, 60);
                    }
                }
                "voltQolZoomSmooth" => e.volt_qol.zoom_smooth = flag(e.volt_qol.zoom_smooth),
                "voltQolFreelook" => {
                    e.volt_qol.freelook_enabled = flag(e.volt_qol.freelook_enabled)
                }
                "voltQolInspector" => {
                    e.volt_qol.world_inspector = flag(e.volt_qol.world_inspector)
                }
                "voltQolSpawnLight" => {
                    e.volt_qol.spawn_light_overlay = flag(e.volt_qol.spawn_light_overlay)
                }
                "voltQolBlueprint" => {
                    e.volt_qol.blueprint_enabled = flag(e.volt_qol.blueprint_enabled)
                }
                "voltQolBlueprintPreset" => {
                    if let Some(p) = crate::volt_qol::BlueprintPreset::parse(value) {
                        e.volt_qol.blueprint_preset = p;
                    }
                }
                "voltQolBlueprintLayer" => {
                    if let Ok(v) = value.parse::<i16>() {
                        e.volt_qol.blueprint_layer = v.clamp(-1, 15);
                    }
                }
                "voltQolBlueprintChecklist" => {
                    e.volt_qol.blueprint_checklist = flag(e.volt_qol.blueprint_checklist)
                }
                "voltQolPhotoOverlays" => {
                    e.volt_qol.photo_mode_overlays = flag(e.volt_qol.photo_mode_overlays)
                }
                "voltQolCamRoll" => {
                    if let Ok(v) = value.parse::<i16>() {
                        e.volt_qol.cam_roll_deg = v.clamp(-45, 45);
                    }
                }
                "voltQolLetterbox" => e.volt_qol.cam_letterbox = flag(e.volt_qol.cam_letterbox),
                "voltQolReplay" => e.volt_qol.replay_buffer = flag(e.volt_qol.replay_buffer),
                "voltQolReplayButton" => {
                    e.volt_qol.replay_button = flag(e.volt_qol.replay_button)
                }
                "voltQolReplayBtnX" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_qol.replay_btn_x_pct = v.clamp(0, 100);
                    }
                }
                "voltQolReplayBtnY" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_qol.replay_btn_y_pct = v.clamp(0, 100);
                    }
                }
                "voltQolReplayBtnSize" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_qol.replay_btn_size = v.clamp(22, 72);
                    }
                }
                "voltQolReplayBtnOpacity" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_qol.replay_btn_opacity_pct = v.clamp(25, 100);
                    }
                }
                "voltQolCaveReverb" => {
                    e.volt_qol.sound_cave_reverb = flag(e.volt_qol.sound_cave_reverb)
                }
                "voltQolUnderwaterLpf" => {
                    e.volt_qol.sound_underwater_lpf = flag(e.volt_qol.sound_underwater_lpf)
                }
                "voltQolWallOcclusion" => {
                    e.volt_qol.sound_wall_occlusion = flag(e.volt_qol.sound_wall_occlusion)
                }
                "voltQolMufflePortal" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_qol.muffle_portal_pct = v.clamp(0, 100);
                    }
                }
                "voltQolMufflePiston" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_qol.muffle_piston_pct = v.clamp(0, 100);
                    }
                }
                "voltQolMuffleMinecart" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_qol.muffle_minecart_pct = v.clamp(0, 100);
                    }
                }
                "voltQolMuffleExplosion" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_qol.muffle_explosion_pct = v.clamp(0, 100);
                    }
                }
                "voltQolMuffleRain" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_qol.muffle_rain_pct = v.clamp(0, 100);
                    }
                }
                "voltQolAtlas" => e.volt_qol.atlas_enabled = flag(e.volt_qol.atlas_enabled),
                "voltQolAtlasMode" => {
                    if let Some(m) = crate::volt_qol::AtlasViewMode::parse(value) {
                        e.volt_qol.atlas_mode = m;
                    }
                }
                "voltQolAtlasGrid" => e.volt_qol.atlas_grid = flag(e.volt_qol.atlas_grid),
                "voltQolVmCustom" => {
                    e.volt_qol.viewmodel_custom = flag(e.volt_qol.viewmodel_custom)
                }
                "voltQolVmX" => {
                    if let Ok(v) = value.parse::<i16>() {
                        e.volt_qol.vm_offset_x = v.clamp(-100, 100);
                    }
                }
                "voltQolVmY" => {
                    if let Ok(v) = value.parse::<i16>() {
                        e.volt_qol.vm_offset_y = v.clamp(-100, 100);
                    }
                }
                "voltQolVmZ" => {
                    if let Ok(v) = value.parse::<i16>() {
                        e.volt_qol.vm_offset_z = v.clamp(-100, 100);
                    }
                }
                "voltQolVmScale" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_qol.vm_scale_pct = v.clamp(40, 180);
                    }
                }
                "voltQolVmRoll" => {
                    if let Ok(v) = value.parse::<i16>() {
                        e.volt_qol.vm_roll_deg = v.clamp(-45, 45);
                    }
                }
                "voltQolVmSwing" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_qol.vm_swing_speed_pct = v.clamp(40, 200);
                    }
                }
                "voltQolVmClassic" => {
                    e.volt_qol.vm_classic_animations = flag(e.volt_qol.vm_classic_animations)
                }
                "voltQolVmTotem" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_qol.vm_totem_scale_pct = v.clamp(25, 120);
                    }
                }
                // The ported Volt Chat and 3D proximity voice chat:
                "voltVoiceEnabled" => e.volt_chat.voice_enabled = flag(e.volt_chat.voice_enabled),
                "voltVoicePtt" => e.volt_chat.voice_ptt_mode = flag(e.volt_chat.voice_ptt_mode),
                "voltVoiceVad" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_chat.voice_vad_threshold = v.clamp(5, 80);
                    }
                }
                "voltVoiceMicGain" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_chat.voice_mic_gain_pct = v.clamp(0, 200);
                    }
                }
                "voltVoiceOutGain" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_chat.voice_output_gain_pct = v.clamp(0, 200);
                    }
                }
                "voltVoiceDist" => {
                    if let Ok(v) = value.parse::<u16>() {
                        e.volt_chat.voice_distance_blocks = v.clamp(12, 96);
                    }
                }
                "voltVoiceCaveMuffle" => {
                    e.volt_chat.voice_cave_muffle = flag(e.volt_chat.voice_cave_muffle)
                }
                "voltVoiceHudBtn" => {
                    e.volt_chat.voice_hud_button = flag(e.volt_chat.voice_hud_button)
                }
                "voltVoiceLoopback" => {
                    e.volt_chat.voice_loopback_test = flag(e.volt_chat.voice_loopback_test)
                }
                "voltChatTimestamps" => {
                    e.volt_chat.chat_timestamps = flag(e.volt_chat.chat_timestamps)
                }
                "voltChatCompact" => {
                    e.volt_chat.chat_compact_spam = flag(e.volt_chat.chat_compact_spam)
                }
                "voltChatMention" => {
                    e.volt_chat.chat_mention_ping = flag(e.volt_chat.chat_mention_ping)
                }
                "voltChatQuickBar" => e.volt_chat.chat_quick_bar = flag(e.volt_chat.chat_quick_bar),
                // The ported Volt native mods:
                "voltModArmorHud" => e.volt_mods.armor_hud = flag(e.volt_mods.armor_hud),
                "voltModPotionHud" => e.volt_mods.potion_hud = flag(e.volt_mods.potion_hud),
                "voltModCpsHud" => e.volt_mods.cps_hud = flag(e.volt_mods.cps_hud),
                "voltModReachHud" => e.volt_mods.reach_hud = flag(e.volt_mods.reach_hud),
                "voltModFpsPill" => e.volt_mods.fps_ping_pill = flag(e.volt_mods.fps_ping_pill),
                "voltModKeystrokes" => {
                    e.volt_mods.keystrokes_hud = flag(e.volt_mods.keystrokes_hud)
                }
                "voltModCrosshair" => {
                    if let Some(c) = crate::volt_qol::CrosshairStyle::parse(value) {
                        e.volt_mods.custom_crosshair = c;
                    }
                }
                "voltModLowShieldFire" => {
                    e.volt_mods.low_shield_fire = flag(e.volt_mods.low_shield_fire)
                }
                "voltModNoHurtcam" => e.volt_mods.no_hurtcam = flag(e.volt_mods.no_hurtcam),
                "voltModToggleSprint" => {
                    e.volt_mods.toggle_sprint = flag(e.volt_mods.toggle_sprint)
                }
                "voltModAppleSkin" => e.volt_mods.appleskin_hud = flag(e.volt_mods.appleskin_hud),
                "voltModShulkerPreview" => {
                    e.volt_mods.shulker_preview = flag(e.volt_mods.shulker_preview)
                }
                "voltModToolShield" => {
                    e.volt_mods.tool_break_shield = flag(e.volt_mods.tool_break_shield)
                }
                "voltModAutoTool" => e.volt_mods.auto_tool_hint = flag(e.volt_mods.auto_tool_hint),
                "voltModTrajectory" => {
                    e.volt_mods.trajectory_arc = flag(e.volt_mods.trajectory_arc)
                }
                "voltModDamagePopups" => {
                    e.volt_mods.damage_popups = flag(e.volt_mods.damage_popups)
                }
                "voltModHitboxPlus" => e.volt_mods.hitbox_plus = flag(e.volt_mods.hitbox_plus),
                "voltModDamageDir" => {
                    e.volt_mods.damage_direction_hud = flag(e.volt_mods.damage_direction_hud)
                }
                "voltModChatCalc" => {
                    e.volt_mods.chat_calculator = flag(e.volt_mods.chat_calculator)
                }
                "voltModEnchantGuide" => {
                    e.volt_mods.enchant_guide = flag(e.volt_mods.enchant_guide)
                }
                "voltModSlimeTech" => e.volt_mods.slime_tech_hud = flag(e.volt_mods.slime_tech_hud),
                "voltModDeathTracker" => {
                    e.volt_mods.death_tracker = flag(e.volt_mods.death_tracker)
                }
                "voltModTimeWeather" => {
                    if let Some(tw) = crate::volt_qol::ClientTimeWeather::parse(value) {
                        e.volt_mods.time_weather = tw;
                    }
                }
                "voltModOledAfk" => e.volt_mods.oled_afk_saver = flag(e.volt_mods.oled_afk_saver),
                // The catalog keys (`voltMod.<slug>`, `voltModCfg.*`) name the mods.
                _ => {
                    let _ = e.mod_catalog.parse_entry(key, value);
                }
            }
        }
        e
    }
    pub fn serialize(&self) -> String {
        let mut s = format!(
            "fullbright:{}\ndynamicLights:{}\nzoom:{}\nzoomKey:{}\nzoomLevel:{}\nzoomSmooth:{}\nzoomSensitivity:{}\nfps:{}\nfpsCorner:{}\nfpsLows:{}\nkeystrokes:{}\nkeystrokesCorner:{}\nkeystrokesCps:{}\nmovedToMods:{}\nminimap:{}\nminimapCorner:{}\nminimapSize:{}\nminimapShape:{}\nminimapZoom:{}\nminimapRotate:{}\nautoReconnect:{}\nuiSoundReserve:{}\nownNameTag:{}\nhideBlockParticles:{}\nmouseTweaks:{}\nmouseTweaksWheel:{}\nsort:{}\nsortKey:{}\nchatAnimation:{}\nchatIndicator:{}\n{}:{}\nagentControl:{}\nreproKeys:{}\ndiscordPresence:{}\nsplitscreen:{}\nsplitscreenTwo:{}\nsplitscreenThree:{}\nsplitscreenNames:{}\nwheelCyclesSettings:{}\nmusicOff:{}\nmusicPage:{}\ncosmeticsMine:{}\ncosmeticsOthers:{}\nthreadBoost:{}\ndisplayHz:{}\nperformanceHints:{}\nvoltPreset:{}\ndynamicFps:{}\nparticleBudget:{}\naudioVoiceCap:{}\nautoDrs:{}\nthermalGuard:{}\nvoltReflex:{}\nentityCap:{}\nitemRenderCap:{}\nblockEntityCap:{}\nmenuBgMode:{}\nanimThrottle:{}\nactiveAnimCull:{}\nfastTranslucency:{}\nfastWeather:{}\ncullSignText:{}\ndistantMobThrottle:{}\nhubLagShield:{}\nchaosShield:{}\nparticleFrustum:{}\nmemorySaver:{}\nlookaheadStream:{}\ncaveSkyCull:{}\nfrustumPriorityMesh:{}\n",
            bool_text(self.fullbright),
            bool_text(self.dynamic_lights),
            bool_text(self.zoom),
            self.zoom_key,
            self.zoom_level,
            bool_text(self.zoom_smooth),
            bool_text(self.zoom_sensitivity),
            bool_text(self.fps),
            self.fps_corner.name(),
            bool_text(self.fps_lows),
            bool_text(self.keystrokes),
            self.keystrokes_corner.name(),
            bool_text(self.keystrokes_cps),
            self.moved.join(","),
            bool_text(self.minimap),
            self.minimap_corner.name(),
            self.minimap_size,
            if self.minimap_round {
                "round"
            } else {
                "square"
            },
            self.minimap_zoom,
            bool_text(self.minimap_rotate),
            bool_text(self.auto_reconnect),
            bool_text(self.ui_sound_reserve),
            bool_text(self.own_name_tag),
            bool_text(self.hide_block_particles),
            bool_text(self.mouse_tweaks),
            bool_text(self.mouse_tweaks_wheel),
            bool_text(self.sort),
            self.sort_key,
            bool_text(self.chat_animation),
            bool_text(self.chat_indicator),
            horizon_core::no_ai::KEY,
            bool_text(self.no_ai),
            bool_text(self.agent_control),
            bool_text(self.repro_keys),
            bool_text(self.discord_presence),
            bool_text(self.splitscreen),
            self.splitscreen_two,
            self.splitscreen_three,
            bool_text(self.splitscreen_names),
            bool_text(self.wheel_cycles_settings),
            self.music_off.join(","),
            bool_text(self.music_page),
            bool_text(self.cosmetics_mine),
            bool_text(self.cosmetics_others),
            bool_text(self.thread_boost),
            self.display_hz,
            bool_text(self.performance_hints),
            self.volt_preset.as_str(),
            self.dynamic_fps.as_str(),
            self.particle_budget.as_str(),
            self.audio_voice_cap.as_str(),
            self.auto_drs.as_str(),
            bool_text(self.thermal_guard),
            bool_text(self.volt_reflex),
            self.entity_cap.as_str(),
            self.item_render_cap.as_str(),
            self.block_entity_cap.as_str(),
            self.menu_bg_mode.as_str(),
            bool_text(self.anim_throttle),
            bool_text(self.active_anim_cull),
            bool_text(self.fast_translucency),
            bool_text(self.fast_weather),
            bool_text(self.cull_sign_text),
            bool_text(self.distant_mob_throttle),
            bool_text(self.hub_lag_shield),
            bool_text(self.chaos_particle_shield),
            bool_text(self.particle_frustum_cull),
            bool_text(self.memory_saver),
            bool_text(self.lookahead_stream),
            bool_text(self.cave_sky_cull),
            bool_text(self.frustum_priority_mesh),
        );
        let mut push = |k: &str, v: &str| {
            s.push_str(k);
            s.push(':');
            s.push_str(v);
            s.push('\n');
        };
        // The ported Volt minimap extras:
        push("minimapFrame", self.minimap_frame.name());
        push("minimapOpacity", &self.minimap_opacity.to_string());
        push("minimapTerrain", self.minimap_terrain_mode.name());
        push("minimapChunkGrid", bool_text(self.minimap_chunk_grid));
        push("minimapEntities", self.minimap_entities.name());
        push("minimapCoords", bool_text(self.minimap_coords));
        push("minimapBiome", bool_text(self.minimap_biome));
        push("minimapTime", bool_text(self.minimap_time));
        push("minimapZoomKey", &self.minimap_zoom_key);
        // The ported Volt waypoints:
        push("waypoints", bool_text(self.waypoints));
        push("waypointBeams", bool_text(self.waypoint_beams));
        push("waypointLabels", bool_text(self.waypoint_labels));
        push("waypointDistance", bool_text(self.waypoint_distance));
        push("waypointCoords", bool_text(self.waypoint_coords));
        push("waypointScale", &self.waypoint_scale.to_string());
        push(
            "waypointMaxDistance",
            &self.waypoint_max_distance.to_string(),
        );
        push("waypointEdgeArrows", bool_text(self.waypoint_edge_arrows));
        push("waypointAutoDeath", bool_text(self.waypoint_auto_death));
        push(
            "waypointDimensionSync",
            bool_text(self.waypoint_dimension_sync),
        );
        push("waypointOnMinimap", bool_text(self.waypoint_on_minimap));
        push("waypointHudButton", bool_text(self.waypoint_hud_button));
        push("waypointNewColor", self.waypoint_new_color.label());
        push("waypointKey", &self.waypoint_key);
        // The ported Volt Boost telemetry and Tier 1 switches:
        push("voltPerfHud", self.perf_hud.as_str());
        push("voltPerfHudCorner", self.perf_hud_corner.name());
        push("voltCullLeaves", bool_text(self.cull_leaves));
        push("voltChunkUploadCap", self.chunk_upload_cap.as_str());
        push("voltDistantLodManual", bool_text(self.distant_lod_boost));
        push("voltFastShadows", self.fast_shadows.as_str());
        push("voltGlintMode", self.glint_mode.as_str());
        push("voltPrewarm", bool_text(self.prewarm_pipelines));
        push("voltKeepAwake", bool_text(self.keep_screen_awake));
        // The ported Volt Shaders:
        push("voltShadersEnabled", bool_text(self.volt_shaders.enabled));
        push("voltShadersProfile", self.volt_shaders.profile.as_str());
        push(
            "voltShaderWavePlants",
            bool_text(self.volt_shaders.waving_plants),
        );
        push(
            "voltShaderWaveLeaves",
            bool_text(self.volt_shaders.waving_leaves),
        );
        push(
            "voltShaderWaveWater",
            bool_text(self.volt_shaders.waving_water),
        );
        push(
            "voltShaderWindSpeed",
            &self.volt_shaders.wind_speed.to_string(),
        );
        push(
            "voltShaderWindStrength",
            &self.volt_shaders.wind_strength.to_string(),
        );
        push(
            "voltShaderStormWind",
            bool_text(self.volt_shaders.storm_wind),
        );
        push(
            "voltShaderSunShading",
            bool_text(self.volt_shaders.sun_shading),
        );
        push(
            "voltShaderShadowRichness",
            &self.volt_shaders.shadow_richness.to_string(),
        );
        push(
            "voltShaderFoliageGlow",
            bool_text(self.volt_shaders.foliage_subsurface),
        );
        push(
            "voltShaderTorchStyle",
            self.volt_shaders.torch_style.as_str(),
        );
        push(
            "voltShaderTorchIntensity",
            &self.volt_shaders.torch_intensity.to_string(),
        );
        push(
            "voltShaderEmissiveGlow",
            &self.volt_shaders.emissive_glow.to_string(),
        );
        push(
            "voltShaderRainWetness",
            bool_text(self.volt_shaders.rain_wetness),
        );
        push(
            "voltShaderWaterRipples",
            bool_text(self.volt_shaders.water_ripples),
        );
        push(
            "voltShaderWaterReflection",
            &self.volt_shaders.water_reflection.to_string(),
        );
        push(
            "voltShaderWaterGlint",
            bool_text(self.volt_shaders.water_glint),
        );
        push(
            "voltShaderClearWater",
            bool_text(self.volt_shaders.clear_water),
        );
        push(
            "voltShaderUnderwaterRefract",
            bool_text(self.volt_shaders.underwater_refraction),
        );
        push(
            "voltShaderCustomSky",
            bool_text(self.volt_shaders.custom_sky),
        );
        push(
            "voltShaderSunsetGlow",
            &self.volt_shaders.sunset_glow.to_string(),
        );
        push(
            "voltShaderSunHalo",
            bool_text(self.volt_shaders.sun_halo),
        );
        push("voltShaderAurora", self.volt_shaders.aurora.as_str());
        push("voltShaderDepthHaze", self.volt_shaders.depth_haze.as_str());
        push("voltShaderTonemap", self.volt_shaders.tonemap.as_str());
        push("voltShaderBloom", self.volt_shaders.bloom.as_str());
        push("voltShaderSunRays", bool_text(self.volt_shaders.sun_rays));
        push(
            "voltShaderVibrance",
            &self.volt_shaders.vibrance.to_string(),
        );
        push(
            "voltShaderSaturation",
            &self.volt_shaders.saturation.to_string(),
        );
        push(
            "voltShaderContrast",
            &self.volt_shaders.contrast.to_string(),
        );
        push(
            "voltShaderExposure",
            &self.volt_shaders.exposure.to_string(),
        );
        push("voltShaderWarmth", &self.volt_shaders.warmth.to_string());
        push("voltShaderVignette", self.volt_shaders.vignette.as_str());
        push("voltShaderStyle", self.volt_shaders.style.as_str());
        push(
            "voltShaderCloudShadows",
            bool_text(self.volt_shaders.cloud_shadows),
        );
        push(
            "voltShaderGlowingOres",
            bool_text(self.volt_shaders.glowing_ores),
        );
        push("voltShaderOreGlow", &self.volt_shaders.ore_glow.to_string());
        push("voltShaderAutoPbr", bool_text(self.volt_shaders.auto_pbr));
        push(
            "voltShaderWaterFoam",
            bool_text(self.volt_shaders.water_foam),
        );
        push(
            "voltShaderCloudsMode",
            self.volt_shaders.clouds_mode.as_str(),
        );
        push(
            "voltShaderCloudCoverage",
            &self.volt_shaders.cloud_coverage.to_string(),
        );
        push(
            "voltShaderNightSkyPlus",
            bool_text(self.volt_shaders.night_sky_plus),
        );
        push(
            "voltShaderRainbow",
            bool_text(self.volt_shaders.post_rain_rainbow),
        );
        push(
            "voltShaderEyeComfort",
            bool_text(self.volt_shaders.eye_comfort),
        );
        // The ported Volt Visuals and Volt Boost+ passes:
        push("voltCtmGlass", bool_text(self.volt_visuals.connected_glass));
        push(
            "voltBetterGrass",
            bool_text(self.volt_visuals.better_grass_sides),
        );
        push(
            "voltDroppedItems3d",
            bool_text(self.volt_visuals.dropped_items_3d),
        );
        push(
            "voltGreedyQuadMerge",
            bool_text(self.volt_boost_plus.greedy_quad_merge),
        );
        push(
            "voltCaveOcclusion",
            bool_text(self.volt_boost_plus.cave_chunk_occlusion),
        );
        push(
            "voltZeroStutterPool",
            bool_text(self.volt_boost_plus.zero_stutter_mesh_pool),
        );
        push(
            "voltOffscreenEntitySkip",
            bool_text(self.volt_boost_plus.offscreen_entity_skip),
        );
        push(
            "voltSmartFramePacer",
            bool_text(self.volt_boost_plus.smart_frame_pacer),
        );
        // The ported Volt QoL suite:
        push("voltQolZoom", bool_text(self.volt_qol.zoom_enabled));
        push("voltQolZoomFov", &self.volt_qol.zoom_fov_pct.to_string());
        push("voltQolZoomSmooth", bool_text(self.volt_qol.zoom_smooth));
        push("voltQolFreelook", bool_text(self.volt_qol.freelook_enabled));
        push(
            "voltQolInspector",
            bool_text(self.volt_qol.world_inspector),
        );
        push(
            "voltQolSpawnLight",
            bool_text(self.volt_qol.spawn_light_overlay),
        );
        push(
            "voltQolBlueprint",
            bool_text(self.volt_qol.blueprint_enabled),
        );
        push(
            "voltQolBlueprintPreset",
            self.volt_qol.blueprint_preset.as_str(),
        );
        push(
            "voltQolBlueprintLayer",
            &self.volt_qol.blueprint_layer.to_string(),
        );
        push(
            "voltQolBlueprintChecklist",
            bool_text(self.volt_qol.blueprint_checklist),
        );
        push(
            "voltQolPhotoOverlays",
            bool_text(self.volt_qol.photo_mode_overlays),
        );
        push("voltQolCamRoll", &self.volt_qol.cam_roll_deg.to_string());
        push("voltQolLetterbox", bool_text(self.volt_qol.cam_letterbox));
        push("voltQolReplay", bool_text(self.volt_qol.replay_buffer));
        push(
            "voltQolReplayButton",
            bool_text(self.volt_qol.replay_button),
        );
        push(
            "voltQolReplayBtnX",
            &self.volt_qol.replay_btn_x_pct.to_string(),
        );
        push(
            "voltQolReplayBtnY",
            &self.volt_qol.replay_btn_y_pct.to_string(),
        );
        push(
            "voltQolReplayBtnSize",
            &self.volt_qol.replay_btn_size.to_string(),
        );
        push(
            "voltQolReplayBtnOpacity",
            &self.volt_qol.replay_btn_opacity_pct.to_string(),
        );
        push(
            "voltQolCaveReverb",
            bool_text(self.volt_qol.sound_cave_reverb),
        );
        push(
            "voltQolUnderwaterLpf",
            bool_text(self.volt_qol.sound_underwater_lpf),
        );
        push(
            "voltQolWallOcclusion",
            bool_text(self.volt_qol.sound_wall_occlusion),
        );
        push(
            "voltQolMufflePortal",
            &self.volt_qol.muffle_portal_pct.to_string(),
        );
        push(
            "voltQolMufflePiston",
            &self.volt_qol.muffle_piston_pct.to_string(),
        );
        push(
            "voltQolMuffleMinecart",
            &self.volt_qol.muffle_minecart_pct.to_string(),
        );
        push(
            "voltQolMuffleExplosion",
            &self.volt_qol.muffle_explosion_pct.to_string(),
        );
        push(
            "voltQolMuffleRain",
            &self.volt_qol.muffle_rain_pct.to_string(),
        );
        push("voltQolAtlas", bool_text(self.volt_qol.atlas_enabled));
        push("voltQolAtlasMode", self.volt_qol.atlas_mode.as_str());
        push("voltQolAtlasGrid", bool_text(self.volt_qol.atlas_grid));
        push("voltQolVmCustom", bool_text(self.volt_qol.viewmodel_custom));
        push("voltQolVmX", &self.volt_qol.vm_offset_x.to_string());
        push("voltQolVmY", &self.volt_qol.vm_offset_y.to_string());
        push("voltQolVmZ", &self.volt_qol.vm_offset_z.to_string());
        push("voltQolVmScale", &self.volt_qol.vm_scale_pct.to_string());
        push("voltQolVmRoll", &self.volt_qol.vm_roll_deg.to_string());
        push(
            "voltQolVmSwing",
            &self.volt_qol.vm_swing_speed_pct.to_string(),
        );
        push(
            "voltQolVmClassic",
            bool_text(self.volt_qol.vm_classic_animations),
        );
        push(
            "voltQolVmTotem",
            &self.volt_qol.vm_totem_scale_pct.to_string(),
        );
        // The ported Volt Chat and 3D proximity voice chat:
        push("voltVoiceEnabled", bool_text(self.volt_chat.voice_enabled));
        push("voltVoicePtt", bool_text(self.volt_chat.voice_ptt_mode));
        push(
            "voltVoiceVad",
            &self.volt_chat.voice_vad_threshold.to_string(),
        );
        push(
            "voltVoiceMicGain",
            &self.volt_chat.voice_mic_gain_pct.to_string(),
        );
        push(
            "voltVoiceOutGain",
            &self.volt_chat.voice_output_gain_pct.to_string(),
        );
        push(
            "voltVoiceDist",
            &self.volt_chat.voice_distance_blocks.to_string(),
        );
        push(
            "voltVoiceCaveMuffle",
            bool_text(self.volt_chat.voice_cave_muffle),
        );
        push(
            "voltVoiceHudBtn",
            bool_text(self.volt_chat.voice_hud_button),
        );
        push(
            "voltVoiceLoopback",
            bool_text(self.volt_chat.voice_loopback_test),
        );
        push(
            "voltChatTimestamps",
            bool_text(self.volt_chat.chat_timestamps),
        );
        push(
            "voltChatCompact",
            bool_text(self.volt_chat.chat_compact_spam),
        );
        push(
            "voltChatMention",
            bool_text(self.volt_chat.chat_mention_ping),
        );
        push("voltChatQuickBar", bool_text(self.volt_chat.chat_quick_bar));
        // The ported Volt native mods:
        push("voltModArmorHud", bool_text(self.volt_mods.armor_hud));
        push("voltModPotionHud", bool_text(self.volt_mods.potion_hud));
        push("voltModCpsHud", bool_text(self.volt_mods.cps_hud));
        push("voltModReachHud", bool_text(self.volt_mods.reach_hud));
        push("voltModFpsPill", bool_text(self.volt_mods.fps_ping_pill));
        push("voltModKeystrokes", bool_text(self.volt_mods.keystrokes_hud));
        push("voltModCrosshair", self.volt_mods.custom_crosshair.as_str());
        push(
            "voltModLowShieldFire",
            bool_text(self.volt_mods.low_shield_fire),
        );
        push("voltModNoHurtcam", bool_text(self.volt_mods.no_hurtcam));
        push(
            "voltModToggleSprint",
            bool_text(self.volt_mods.toggle_sprint),
        );
        push("voltModAppleSkin", bool_text(self.volt_mods.appleskin_hud));
        push(
            "voltModShulkerPreview",
            bool_text(self.volt_mods.shulker_preview),
        );
        push(
            "voltModToolShield",
            bool_text(self.volt_mods.tool_break_shield),
        );
        push("voltModAutoTool", bool_text(self.volt_mods.auto_tool_hint));
        push(
            "voltModTrajectory",
            bool_text(self.volt_mods.trajectory_arc),
        );
        push(
            "voltModDamagePopups",
            bool_text(self.volt_mods.damage_popups),
        );
        push("voltModHitboxPlus", bool_text(self.volt_mods.hitbox_plus));
        push(
            "voltModDamageDir",
            bool_text(self.volt_mods.damage_direction_hud),
        );
        push(
            "voltModChatCalc",
            bool_text(self.volt_mods.chat_calculator),
        );
        push(
            "voltModEnchantGuide",
            bool_text(self.volt_mods.enchant_guide),
        );
        push(
            "voltModSlimeTech",
            bool_text(self.volt_mods.slime_tech_hud),
        );
        push(
            "voltModDeathTracker",
            bool_text(self.volt_mods.death_tracker),
        );
        push("voltModTimeWeather", self.volt_mods.time_weather.as_str());
        push("voltModOledAfk", bool_text(self.volt_mods.oled_afk_saver));
        self.mod_catalog.serialize_into(&mut push);
        s
    }
    /// Applies a Volt Boost profile to the switches this port carries so far
    /// (the thread tuning, the dynamic idle FPS and the particle budget; the
    /// method grows with each ported switch, as the head's does across its
    /// Tier 1-3 fields). `Off` puts every one of them back to vanilla's
    /// behaviour.
    pub fn apply_volt_preset(&mut self, preset: crate::volt_features::VoltBoostPreset) {
        use crate::volt_features::{
            AudioVoiceCap, AutoDrsMode, BlockEntityCap, DynamicIdleFps, EntityCap, ItemRenderCap,
            ParticleBudget, VoltBoostPreset,
        };
        self.volt_preset = preset;
        let (thread_boost, dynamic_fps, particle_budget, audio_voice_cap) = match preset {
            VoltBoostPreset::Off => {
                (false, DynamicIdleFps::Off, ParticleBudget::Vanilla16k, AudioVoiceCap::Vanilla)
            }
            VoltBoostPreset::Balanced => {
                (true, DynamicIdleFps::Fps30, ParticleBudget::Cap4096, AudioVoiceCap::Cap32)
            }
            VoltBoostPreset::Competitive => {
                (true, DynamicIdleFps::Fps30, ParticleBudget::Cap2048, AudioVoiceCap::Cap32)
            }
            VoltBoostPreset::UltraFps => {
                (true, DynamicIdleFps::Fps15, ParticleBudget::Cap512, AudioVoiceCap::Cap16)
            }
            VoltBoostPreset::BatterySaver => {
                (false, DynamicIdleFps::Fps15, ParticleBudget::Cap2048, AudioVoiceCap::Cap16)
            }
            VoltBoostPreset::LowRam => {
                (true, DynamicIdleFps::Fps15, ParticleBudget::Cap512, AudioVoiceCap::Cap16)
            }
        };
        self.thread_boost = thread_boost;
        self.dynamic_fps = dynamic_fps;
        self.particle_budget = particle_budget;
        self.audio_voice_cap = audio_voice_cap;
        let (auto_drs, thermal_guard, volt_reflex, entity_cap, item_render_cap, block_entity_cap) =
            match preset {
                VoltBoostPreset::Off => (
                    AutoDrsMode::Off,
                    false,
                    false,
                    EntityCap::Unlimited,
                    ItemRenderCap::Unlimited,
                    BlockEntityCap::Unlimited,
                ),
                VoltBoostPreset::Balanced => (
                    AutoDrsMode::Off,
                    true,
                    true,
                    EntityCap::Cap128,
                    ItemRenderCap::Cap128,
                    BlockEntityCap::Cap64,
                ),
                VoltBoostPreset::Competitive => (
                    AutoDrsMode::Target120,
                    true,
                    true,
                    EntityCap::Cap96,
                    ItemRenderCap::Cap64,
                    BlockEntityCap::Cap32,
                ),
                VoltBoostPreset::UltraFps => (
                    AutoDrsMode::Aggressive50,
                    true,
                    true,
                    EntityCap::Cap64,
                    ItemRenderCap::Cap32,
                    BlockEntityCap::Cap16,
                ),
                VoltBoostPreset::BatterySaver => (
                    AutoDrsMode::Target60,
                    true,
                    false,
                    EntityCap::Cap96,
                    ItemRenderCap::Cap64,
                    BlockEntityCap::Cap32,
                ),
                VoltBoostPreset::LowRam => (
                    AutoDrsMode::Target60,
                    true,
                    true,
                    EntityCap::Cap64,
                    ItemRenderCap::Cap32,
                    BlockEntityCap::Cap16,
                ),
            };
        self.auto_drs = auto_drs;
        self.thermal_guard = thermal_guard;
        self.volt_reflex = volt_reflex;
        self.entity_cap = entity_cap;
        self.item_render_cap = item_render_cap;
        self.block_entity_cap = block_entity_cap;
        // Every profile that is not Off puts the menu backdrop on FastDark.
        self.menu_bg_mode = match preset {
            VoltBoostPreset::Off => crate::volt_features::MenuBgMode::PanoramaBlur,
            _ => crate::volt_features::MenuBgMode::FastDark,
        };
        // The Tier 2 switches, per the head's profiles: the throttle, the
        // sign-text cull and the mob throttle are on in every profile that
        // is not Off; the weather and the hub shield skip Balanced.
        let (anim_throttle, fast_translucency) = match preset {
            VoltBoostPreset::Off => (false, false),
            _ => (true, true),
        };
        self.anim_throttle = anim_throttle;
        self.fast_translucency = fast_translucency;
        let (fast_weather, hub_lag_shield) = match preset {
            VoltBoostPreset::Off | VoltBoostPreset::Balanced => (false, false),
            _ => (true, true),
        };
        self.fast_weather = fast_weather;
        self.hub_lag_shield = hub_lag_shield;
        self.cull_sign_text = preset != VoltBoostPreset::Off;
        self.distant_mob_throttle = preset != VoltBoostPreset::Off;
        self.chaos_particle_shield = preset != VoltBoostPreset::Off;
        self.particle_frustum_cull = preset != VoltBoostPreset::Off;
        // The memory saver is Ultra FPS's, Low-RAM Rescue's and Battery
        // Saver's; Balanced keeps the caches warm.
        self.memory_saver = matches!(
            preset,
            VoltBoostPreset::UltraFps
                | VoltBoostPreset::LowRam
                | VoltBoostPreset::BatterySaver
        );
        // The cave sky cull is on in every non-Off profile; the streaming
        // lookahead everywhere but Low-RAM Rescue, which prefers plain
        // rings while memory is rebuilt.
        self.cave_sky_cull = preset != VoltBoostPreset::Off;
        self.lookahead_stream = !matches!(
            preset,
            VoltBoostPreset::Off | VoltBoostPreset::LowRam
        );
        self.frustum_priority_mesh = preset != VoltBoostPreset::Off;
        // The ported Tier 1 and Tier 3 switches; the distant LOD boost stays
        // off in every profile, as the head's do.
        let plus = preset != VoltBoostPreset::Off;
        self.volt_boost_plus.greedy_quad_merge = plus;
        self.volt_boost_plus.cave_chunk_occlusion = plus;
        self.volt_boost_plus.zero_stutter_mesh_pool = plus;
        self.volt_boost_plus.offscreen_entity_skip = plus;
        self.volt_boost_plus.smart_frame_pacer = plus;
        self.cull_leaves = plus;
        self.distant_lod_boost = false;
        let (chunk_upload_cap, fast_shadows, glint_mode, prewarm_pipelines) = match preset {
            VoltBoostPreset::Off => (
                ChunkUploadCap::Auto,
                FastShadowMode::Vanilla,
                GlintTrimMode::Full,
                false,
            ),
            VoltBoostPreset::Balanced => (
                ChunkUploadCap::Balanced16,
                FastShadowMode::Nearby16,
                GlintTrimMode::Full,
                true,
            ),
            VoltBoostPreset::Competitive => (
                ChunkUploadCap::Balanced16,
                FastShadowMode::Nearby16,
                GlintTrimMode::NearbyOnly,
                true,
            ),
            VoltBoostPreset::UltraFps => (
                ChunkUploadCap::Smooth8,
                FastShadowMode::Off,
                GlintTrimMode::Off,
                true,
            ),
            VoltBoostPreset::BatterySaver => (
                ChunkUploadCap::Smooth8,
                FastShadowMode::Off,
                GlintTrimMode::NearbyOnly,
                true,
            ),
            VoltBoostPreset::LowRam => (
                ChunkUploadCap::Smooth4,
                FastShadowMode::Off,
                GlintTrimMode::Off,
                false,
            ),
        };
        self.chunk_upload_cap = chunk_upload_cap;
        self.fast_shadows = fast_shadows;
        self.glint_mode = glint_mode;
        self.prewarm_pipelines = prewarm_pipelines;
    }

    /// Binds the extra key `name` (`SORT_KEY`, `WAYPOINT_KEY`,
    /// `MINIMAP_ZOOM_KEY`); false for another name (a vanilla mapping).
    pub fn set_key(&mut self, name: &str, value: String) -> bool {
        match name {
            SORT_KEY => self.sort_key = value,
            WAYPOINT_KEY => self.waypoint_key = value,
            MINIMAP_ZOOM_KEY => self.minimap_zoom_key = value,
            _ => return false,
        }
        true
    }
    /// Written only once something differs from the defaults, so a vanilla
    /// setup gains no file.
    pub fn save(&self, dir: &Path) {
        let path = dir.join(FILE);
        if *self == Self::default() && !path.exists() {
            return;
        }
        let _ = fs::write(path, self.serialize());
    }
}

/// The field-of-view modifier target `Camera.tickFov` eases toward: vanilla's
/// (`getFieldOfViewModifier`) divided by a mod's zoom (`native_zoom`). A
/// zoom of 1 leaves it as is.
pub fn fov_target(vanilla: f32, zoom: f32) -> f32 {
    if zoom == 1.0 { vanilla } else { vanilla / zoom }
}
/// Frame times over the last few seconds: frames per second, the minimum
/// and the 1% low, recomputed twice a second.
pub struct FpsCounter {
    /// (end time in seconds, frame milliseconds), oldest first.
    frames: VecDeque<(f64, f32)>,
    shown: FpsReading,
    computed_at: f64,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FpsReading {
    pub fps: u32,
    pub min: u32,
    pub low: u32,
}
/// Seconds the minimum and 1% low look back.
const LOW_WINDOW: f64 = 5.0;
impl Default for FpsCounter {
    fn default() -> Self {
        Self {
            frames: VecDeque::new(),
            shown: FpsReading::default(),
            computed_at: f64::NEG_INFINITY,
        }
    }
}
impl FpsCounter {
    /// A frame that ended at `now` (seconds) and took `ms`.
    pub fn record(&mut self, now: f64, ms: f32) {
        self.frames.push_back((now, ms));
        while self
            .frames
            .front()
            .is_some_and(|&(t, _)| now - t > LOW_WINDOW)
        {
            self.frames.pop_front();
        }
        if now - self.computed_at >= 0.5 {
            self.computed_at = now;
            self.shown = self.compute(now);
        }
    }
    fn compute(&self, now: f64) -> FpsReading {
        let fps = self.frames.iter().filter(|&&(t, _)| now - t < 1.0).count() as u32;
        let mut times: Vec<f32> = self.frames.iter().map(|f| f.1).collect();
        if times.is_empty() {
            return FpsReading::default();
        }
        times.sort_by(|a, b| b.total_cmp(a));
        let to_fps = |ms: f32| (1000.0 / ms.max(0.001)).round() as u32;
        // The 1% low: the frame rate of the 99th percentile frame time.
        let low = times[(times.len() - 1) / 100];
        FpsReading {
            fps,
            min: to_fps(times[0]),
            low: to_fps(low),
        }
    }
    pub fn reading(&self) -> FpsReading {
        self.shown
    }
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

/// Clicks of one button in the last second.
#[derive(Default)]
pub struct Cps {
    clicks: VecDeque<f64>,
}
impl Cps {
    pub fn click(&mut self, now: f64) {
        self.clicks.push_back(now);
        self.forget(now);
    }
    fn forget(&mut self, now: f64) {
        while self.clicks.front().is_some_and(|&t| now - t >= 1.0) {
            self.clicks.pop_front();
        }
    }
    pub fn per_second(&mut self, now: f64) -> usize {
        self.forget(now);
        self.clicks.len()
    }
}
/// Two players sharing a window (`splitscreen_two`): halves this way.
pub const SPLIT_TWO: [&str; 3] = ["Auto", "Side by side", "Top and bottom"];
/// Three players sharing a window (`splitscreen_three`).
pub const SPLIT_THREE: [&str; 3] = ["One wide on top", "One tall on the left", "Side by side"];

/// The settings the Extras page shows, one per row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    Fullbright,
    DynamicLights,
    OwnNameTag,
    HideBlockParticles,
    AutoReconnect,
    UiSoundReserve,
    /// The Music... page on Music & Sounds (`Music page`).
    MusicPage,
    MouseTweaks,
    MouseTweaksWheel,
    Sort,
    SortKey,
    /// A new chat line slides in and fades in (`Chat::layout`).
    ChatAnimation,
    /// The coloured bar at the left of a chat line.
    ChatIndicator,
    /// Not on the Extras page: Horizon Settings shows No AI, its AI Agent page Agent Control.
    NoAi,
    AgentControl,
    /// On the Horizon Settings page.
    DiscordPresence,
    /// On the AI Agent page, next to Agent Control: the F3 recording shortcuts.
    ReproKeys,
    /// On the Splitscreen settings page.
    Splitscreen,
    WheelCyclesSettings,
    /// Shown only while cosmetics are enabled (`horizon_cosmetics::enabled`).
    CosmeticsMine,
    CosmeticsOthers,
    /// The ported Volt Minimap page.
    Minimap,
    MinimapCorner,
    MinimapSize,
    MinimapShape,
    MinimapZoom,
    MinimapRotate,
    MinimapFrameStyle,
    MinimapOpacity,
    MinimapZoomKey,
    MinimapChunkGrid,
    MinimapTerrain,
    MinimapEntityRadar,
    MinimapCoords,
    MinimapBiome,
    MinimapTime,
    /// The ported Volt Waypoints page.
    Waypoints,
    WaypointKey,
    WaypointAutoDeath,
    WaypointDimensionSync,
    WaypointHudButton,
    WaypointOnMinimap,
    WaypointBeams,
    WaypointLabels,
    WaypointDistance,
    WaypointCoords,
    WaypointEdgeArrows,
    WaypointScale,
    WaypointMaxDistance,
    /// The Volt Settings hub's Boost page: the profile and every ported switch.
    VoltPreset,
    VoltThreadBoost,
    VoltPerformanceHints,
    VoltDisplayHz,
    VoltDynamicFps,
    VoltAudioCap,
    VoltAutoDrs,
    VoltThermalGuard,
    VoltReflex,
    VoltParticleBudget,
    VoltChaosShield,
    VoltParticleFrustum,
    VoltEntityCap,
    VoltItemCap,
    VoltBlockEntityCap,
    VoltAnimThrottle,
    VoltActiveAnimCull,
    VoltFastTranslucency,
    VoltFastWeather,
    VoltCullSignText,
    VoltMobThrottle,
    VoltHubShield,
    VoltMenuBg,
    VoltLookahead,
    VoltCaveSkyCull,
    VoltFrustumMesh,
    VoltMemorySaver,
    /// The ported Volt Boost+ telemetry HUD.
    VoltPerfHud,
    VoltPerfHudCorner,
    /// The ported Volt Shaders page (`Options -> Volt Shaders`).
    VoltShaderEnable,
    VoltShaderProfile,
    VoltShaderReset,
    VoltShaderWavePlants,
    VoltShaderWaveLeaves,
    VoltShaderWaveWater,
    VoltShaderStormWind,
    VoltShaderSunShading,
    VoltShaderFoliageGlow,
    VoltShaderTorchStyle,
    VoltShaderRainWetness,
    VoltShaderWaterRipples,
    VoltShaderWaterGlint,
    VoltShaderClearWater,
    VoltShaderUnderwaterRefract,
    VoltShaderCustomSky,
    VoltShaderSunHalo,
    VoltShaderAurora,
    VoltShaderDepthHaze,
    VoltShaderTonemap,
    VoltShaderBloom,
    VoltShaderSunRays,
    VoltShaderVignette,
}
/// Every ported Volt Shaders toggle/choice item (search and tests walk this).
pub const VOLT_SHADER_ITEMS: &[Item] = &[
    Item::VoltShaderEnable,
    Item::VoltShaderProfile,
    Item::VoltShaderReset,
    Item::VoltShaderWavePlants,
    Item::VoltShaderWaveLeaves,
    Item::VoltShaderWaveWater,
    Item::VoltShaderStormWind,
    Item::VoltShaderSunShading,
    Item::VoltShaderFoliageGlow,
    Item::VoltShaderTorchStyle,
    Item::VoltShaderRainWetness,
    Item::VoltShaderWaterRipples,
    Item::VoltShaderWaterGlint,
    Item::VoltShaderClearWater,
    Item::VoltShaderUnderwaterRefract,
    Item::VoltShaderCustomSky,
    Item::VoltShaderSunHalo,
    Item::VoltShaderAurora,
    Item::VoltShaderDepthHaze,
    Item::VoltShaderTonemap,
    Item::VoltShaderBloom,
    Item::VoltShaderSunRays,
    Item::VoltShaderVignette,
];
/// The Extras page: headers (language keys) and items.
pub fn rows() -> Vec<Result<Item, &'static str>> {
    use Item::*;
    let mut rows = vec![
        Err("horizon.extras.header.lighting"),
        Ok(Fullbright),
        Ok(DynamicLights),
        Err("horizon.extras.header.third_person"),
        Ok(OwnNameTag),
        Err("horizon.extras.header.particles"),
        Ok(HideBlockParticles),
        Err("horizon.extras.header.inventory"),
        Ok(MouseTweaks),
        Ok(MouseTweaksWheel),
        Ok(Sort),
        Ok(SortKey),
        Err("horizon.extras.header.chat"),
        Ok(ChatAnimation),
        Ok(ChatIndicator),
        Err("horizon.extras.header.multiplayer"),
        Ok(AutoReconnect),
        Err("horizon.extras.header.sound"),
        Ok(UiSoundReserve),
        Ok(MusicPage),
        Err("horizon.extras.header.menus"),
        Ok(WheelCyclesSettings),
    ];
    if horizon_cosmetics::enabled() {
        rows.extend([
            Err("horizon.extras.header.cosmetics"),
            Ok(CosmeticsMine),
            Ok(CosmeticsOthers),
        ]);
    }
    rows
}
/// The Volt Settings hub's Boost page: the profile first, then every switch
/// the port carries, in its sections.
pub fn volt_boost_rows() -> Vec<Result<Item, &'static str>> {
    use Item::*;
    vec![
        Err("horizon.volt.header.profile"),
        Ok(VoltPreset),
        Ok(VoltPerfHud),
        Ok(VoltPerfHudCorner),
        Err("horizon.volt.header.threads"),
        Ok(VoltThreadBoost),
        Ok(VoltPerformanceHints),
        Ok(VoltDisplayHz),
        Err("horizon.volt.header.frames"),
        Ok(VoltDynamicFps),
        Ok(VoltAudioCap),
        Err("horizon.volt.header.render"),
        Ok(VoltAutoDrs),
        Ok(VoltThermalGuard),
        Ok(VoltReflex),
        Ok(VoltParticleBudget),
        Ok(VoltChaosShield),
        Ok(VoltParticleFrustum),
        Ok(VoltEntityCap),
        Ok(VoltItemCap),
        Ok(VoltBlockEntityCap),
        Ok(VoltAnimThrottle),
        Ok(VoltActiveAnimCull),
        Ok(VoltFastTranslucency),
        Ok(VoltFastWeather),
        Ok(VoltCullSignText),
        Ok(VoltMobThrottle),
        Ok(VoltHubShield),
        Ok(VoltMenuBg),
        Err("horizon.volt.header.streaming"),
        Ok(VoltLookahead),
        Ok(VoltCaveSkyCull),
        Ok(VoltFrustumMesh),
        Ok(VoltMemorySaver),
    ]
}
/// The ported Volt Waypoints settings rows (the Waypoint Manager follows them).
pub fn waypoint_rows() -> Vec<Result<Item, &'static str>> {
    use Item::*;
    vec![
        Err("Waypoints"),
        Ok(Waypoints),
        Ok(WaypointKey),
        Ok(WaypointAutoDeath),
        Ok(WaypointDimensionSync),
        Ok(WaypointHudButton),
        Ok(WaypointOnMinimap),
        Err("In-World Markers & Beams"),
        Ok(WaypointBeams),
        Ok(WaypointLabels),
        Ok(WaypointDistance),
        Ok(WaypointCoords),
        Ok(WaypointEdgeArrows),
        Ok(WaypointScale),
        Ok(WaypointMaxDistance),
    ]
}
/// The ported Volt Minimap ("Minimal with many features") settings rows.
pub fn minimap_rows() -> Vec<Result<Item, &'static str>> {
    use Item::*;
    vec![
        Err("Minimal Minimap"),
        Ok(Minimap),
        Ok(MinimapCorner),
        Ok(MinimapSize),
        Ok(MinimapShape),
        Ok(MinimapZoom),
        Ok(MinimapRotate),
        Ok(MinimapFrameStyle),
        Ok(MinimapOpacity),
        Ok(MinimapZoomKey),
        Ok(MinimapChunkGrid),
        Err("Terrain, Radar & Readouts"),
        Ok(MinimapTerrain),
        Ok(MinimapEntityRadar),
        Ok(WaypointOnMinimap),
        Ok(MinimapCoords),
        Ok(MinimapBiome),
        Ok(MinimapTime),
    ]
}
impl Item {
    /// The language key of the caption (`horizon.*`, assets/horizon/lang).
    pub fn caption_key(self) -> &'static str {
        match self {
            Item::Fullbright => "horizon.extras.fullbright",
            Item::DynamicLights => "horizon.extras.dynamic_lights",
            Item::OwnNameTag => "horizon.extras.own_name_tag",
            Item::HideBlockParticles => "horizon.extras.hide_block_particles",
            Item::AutoReconnect => "horizon.extras.auto_reconnect",
            Item::UiSoundReserve => "horizon.extras.ui_sound_reserve",
            Item::MusicPage => "horizon.extras.music_page",
            Item::MouseTweaks => "horizon.extras.mouse_tweaks",
            Item::MouseTweaksWheel => "horizon.extras.mouse_tweaks_wheel",
            Item::Sort => "horizon.extras.sort",
            Item::SortKey => "horizon.extras.sort_key",
            Item::NoAi => "horizon.extras.no_ai",
            Item::AgentControl => "horizon.extras.agent_control",
            Item::ReproKeys => "horizon.extras.repro_keys",
            Item::DiscordPresence => "horizon.extras.discord_presence",
            Item::Splitscreen => "horizon.extras.splitscreen",
            Item::ChatAnimation => "horizon.extras.chat_animation",
            Item::ChatIndicator => "horizon.extras.chat_indicator",
            Item::WheelCyclesSettings => "horizon.extras.wheel_cycles_settings",
            Item::CosmeticsMine => "horizon.extras.cosmetics_mine",
            Item::CosmeticsOthers => "horizon.extras.cosmetics_others",
            // The ported Volt minimap:
            Item::Minimap => "horizon.volt.minimap",
            Item::MinimapCorner => "horizon.volt.minimap.corner",
            Item::MinimapSize => "horizon.volt.minimap.size",
            Item::MinimapShape => "horizon.volt.minimap.shape",
            Item::MinimapZoom => "horizon.volt.minimap.zoom",
            Item::MinimapRotate => "horizon.volt.minimap.rotate",
            Item::MinimapFrameStyle => "horizon.volt.minimap.frame",
            Item::MinimapOpacity => "horizon.volt.minimap.opacity",
            Item::MinimapZoomKey => "horizon.volt.minimap.zoom_key",
            Item::MinimapChunkGrid => "horizon.volt.minimap.chunk_grid",
            Item::MinimapTerrain => "horizon.volt.minimap.terrain",
            Item::MinimapEntityRadar => "horizon.volt.minimap.radar",
            Item::MinimapCoords => "horizon.volt.minimap.coords",
            Item::MinimapBiome => "horizon.volt.minimap.biome",
            Item::MinimapTime => "horizon.volt.minimap.time",
            // The ported Volt waypoints:
            Item::Waypoints => "horizon.volt.waypoints",
            Item::WaypointKey => "horizon.volt.waypoints.key",
            Item::WaypointAutoDeath => "horizon.volt.waypoints.auto_death",
            Item::WaypointDimensionSync => "horizon.volt.waypoints.dimension_sync",
            Item::WaypointHudButton => "horizon.volt.waypoints.hud_button",
            Item::WaypointOnMinimap => "horizon.volt.waypoints.on_minimap",
            Item::WaypointBeams => "horizon.volt.waypoints.beams",
            Item::WaypointLabels => "horizon.volt.waypoints.labels",
            Item::WaypointDistance => "horizon.volt.waypoints.distance",
            Item::WaypointCoords => "horizon.volt.waypoints.coords",
            Item::WaypointEdgeArrows => "horizon.volt.waypoints.edge_arrows",
            Item::WaypointScale => "horizon.volt.waypoints.scale",
            Item::WaypointMaxDistance => "horizon.volt.waypoints.max_distance",
            Item::VoltPreset => "horizon.volt.preset",
            Item::VoltThreadBoost => "horizon.volt.thread_boost",
            Item::VoltPerformanceHints => "horizon.volt.hints",
            Item::VoltDisplayHz => "horizon.volt.display_hz",
            Item::VoltDynamicFps => "horizon.volt.dynamic_fps",
            Item::VoltAudioCap => "horizon.volt.audio",
            Item::VoltAutoDrs => "horizon.volt.auto_drs",
            Item::VoltThermalGuard => "horizon.volt.thermal",
            Item::VoltReflex => "horizon.volt.reflex",
            Item::VoltParticleBudget => "horizon.volt.particles",
            Item::VoltChaosShield => "horizon.volt.chaos",
            Item::VoltParticleFrustum => "horizon.volt.particle_frustum",
            Item::VoltEntityCap => "horizon.volt.entity_cap",
            Item::VoltItemCap => "horizon.volt.item_cap",
            Item::VoltBlockEntityCap => "horizon.volt.block_entity_cap",
            Item::VoltAnimThrottle => "horizon.volt.anim_throttle",
            Item::VoltActiveAnimCull => "horizon.volt.anim_cull",
            Item::VoltFastTranslucency => "horizon.volt.translucency",
            Item::VoltFastWeather => "horizon.volt.weather",
            Item::VoltCullSignText => "horizon.volt.sign_text",
            Item::VoltMobThrottle => "horizon.volt.mob_throttle",
            Item::VoltHubShield => "horizon.volt.hub_shield",
            Item::VoltMenuBg => "horizon.volt.menu_bg",
            Item::VoltLookahead => "horizon.volt.lookahead",
            Item::VoltCaveSkyCull => "horizon.volt.cave_sky",
            Item::VoltFrustumMesh => "horizon.volt.frustum_mesh",
            Item::VoltMemorySaver => "horizon.volt.memory",
            Item::VoltPerfHud => "horizon.volt.perf_hud",
            Item::VoltPerfHudCorner => "horizon.volt.perf_hud_corner",
            Item::VoltShaderEnable => "horizon.volt.shader_enable",
            Item::VoltShaderProfile => "horizon.volt.shader_profile",
            Item::VoltShaderReset => "horizon.volt.shader_reset",
            Item::VoltShaderWavePlants => "horizon.volt.shader_wave_plants",
            Item::VoltShaderWaveLeaves => "horizon.volt.shader_wave_leaves",
            Item::VoltShaderWaveWater => "horizon.volt.shader_wave_water",
            Item::VoltShaderStormWind => "horizon.volt.shader_storm_wind",
            Item::VoltShaderSunShading => "horizon.volt.shader_sun_shading",
            Item::VoltShaderFoliageGlow => "horizon.volt.shader_foliage_glow",
            Item::VoltShaderTorchStyle => "horizon.volt.shader_torch_style",
            Item::VoltShaderRainWetness => "horizon.volt.shader_rain_wetness",
            Item::VoltShaderWaterRipples => "horizon.volt.shader_water_ripples",
            Item::VoltShaderWaterGlint => "horizon.volt.shader_water_glint",
            Item::VoltShaderClearWater => "horizon.volt.shader_clear_water",
            Item::VoltShaderUnderwaterRefract => "horizon.volt.shader_underwater_refract",
            Item::VoltShaderCustomSky => "horizon.volt.shader_custom_sky",
            Item::VoltShaderSunHalo => "horizon.volt.shader_sun_halo",
            Item::VoltShaderAurora => "horizon.volt.shader_aurora",
            Item::VoltShaderDepthHaze => "horizon.volt.shader_depth_haze",
            Item::VoltShaderTonemap => "horizon.volt.shader_tonemap",
            Item::VoltShaderBloom => "horizon.volt.shader_bloom",
            Item::VoltShaderSunRays => "horizon.volt.shader_sun_rays",
            Item::VoltShaderVignette => "horizon.volt.shader_vignette",
        }
    }
    /// The language key of the description shown as the row's tooltip.
    pub fn description_key(self) -> &'static str {
        match self {
            Item::Fullbright => "horizon.extras.fullbright.tooltip",
            Item::DynamicLights => "horizon.extras.dynamic_lights.tooltip",
            Item::OwnNameTag => "horizon.extras.own_name_tag.tooltip",
            Item::HideBlockParticles => "horizon.extras.hide_block_particles.tooltip",
            Item::AutoReconnect => "horizon.extras.auto_reconnect.tooltip",
            Item::UiSoundReserve => "horizon.extras.ui_sound_reserve.tooltip",
            Item::MusicPage => "horizon.extras.music_page.tooltip",
            Item::MouseTweaks => "horizon.extras.mouse_tweaks.tooltip",
            Item::MouseTweaksWheel => "horizon.extras.mouse_tweaks_wheel.tooltip",
            Item::Sort => "horizon.extras.sort.tooltip",
            Item::SortKey => "horizon.extras.sort_key.tooltip",
            Item::NoAi => "horizon.extras.no_ai.tooltip",
            Item::AgentControl => "horizon.extras.agent_control.tooltip",
            Item::ReproKeys => "horizon.extras.repro_keys.tooltip",
            Item::DiscordPresence => "horizon.extras.discord_presence.tooltip",
            Item::Splitscreen => "horizon.extras.splitscreen.tooltip",
            Item::ChatAnimation => "horizon.extras.chat_animation.tooltip",
            Item::ChatIndicator => "horizon.extras.chat_indicator.tooltip",
            Item::WheelCyclesSettings => "horizon.extras.wheel_cycles_settings.tooltip",
            Item::CosmeticsMine => "horizon.extras.cosmetics_mine.tooltip",
            Item::CosmeticsOthers => "horizon.extras.cosmetics_others.tooltip",
            Item::Minimap => "horizon.volt.minimap.tooltip",
            Item::MinimapCorner => "horizon.volt.minimap.corner.tooltip",
            Item::MinimapSize => "horizon.volt.minimap.size.tooltip",
            Item::MinimapShape => "horizon.volt.minimap.shape.tooltip",
            Item::MinimapZoom => "horizon.volt.minimap.zoom.tooltip",
            Item::MinimapRotate => "horizon.volt.minimap.rotate.tooltip",
            Item::MinimapFrameStyle => "horizon.volt.minimap.frame.tooltip",
            Item::MinimapOpacity => "horizon.volt.minimap.opacity.tooltip",
            Item::MinimapZoomKey => "horizon.volt.minimap.zoom_key.tooltip",
            Item::MinimapChunkGrid => "horizon.volt.minimap.chunk_grid.tooltip",
            Item::MinimapTerrain => "horizon.volt.minimap.terrain.tooltip",
            Item::MinimapEntityRadar => "horizon.volt.minimap.radar.tooltip",
            Item::MinimapCoords => "horizon.volt.minimap.coords.tooltip",
            Item::MinimapBiome => "horizon.volt.minimap.biome.tooltip",
            Item::MinimapTime => "horizon.volt.minimap.time.tooltip",
            Item::Waypoints => "horizon.volt.waypoints.tooltip",
            Item::WaypointKey => "horizon.volt.waypoints.key.tooltip",
            Item::WaypointAutoDeath => "horizon.volt.waypoints.auto_death.tooltip",
            Item::WaypointDimensionSync => "horizon.volt.waypoints.dimension_sync.tooltip",
            Item::WaypointHudButton => "horizon.volt.waypoints.hud_button.tooltip",
            Item::WaypointOnMinimap => "horizon.volt.waypoints.on_minimap.tooltip",
            Item::WaypointBeams => "horizon.volt.waypoints.beams.tooltip",
            Item::WaypointLabels => "horizon.volt.waypoints.labels.tooltip",
            Item::WaypointDistance => "horizon.volt.waypoints.distance.tooltip",
            Item::WaypointCoords => "horizon.volt.waypoints.coords.tooltip",
            Item::WaypointEdgeArrows => "horizon.volt.waypoints.edge_arrows.tooltip",
            Item::WaypointScale => "horizon.volt.waypoints.scale.tooltip",
            Item::WaypointMaxDistance => "horizon.volt.waypoints.max_distance.tooltip",
            Item::VoltPreset => "horizon.volt.preset.tooltip",
            Item::VoltThreadBoost => "horizon.volt.thread_boost.tooltip",
            Item::VoltPerformanceHints => "horizon.volt.hints.tooltip",
            Item::VoltDisplayHz => "horizon.volt.display_hz.tooltip",
            Item::VoltDynamicFps => "horizon.volt.dynamic_fps.tooltip",
            Item::VoltAudioCap => "horizon.volt.audio.tooltip",
            Item::VoltAutoDrs => "horizon.volt.auto_drs.tooltip",
            Item::VoltThermalGuard => "horizon.volt.thermal.tooltip",
            Item::VoltReflex => "horizon.volt.reflex.tooltip",
            Item::VoltParticleBudget => "horizon.volt.particles.tooltip",
            Item::VoltChaosShield => "horizon.volt.chaos.tooltip",
            Item::VoltParticleFrustum => "horizon.volt.particle_frustum.tooltip",
            Item::VoltEntityCap => "horizon.volt.entity_cap.tooltip",
            Item::VoltItemCap => "horizon.volt.item_cap.tooltip",
            Item::VoltBlockEntityCap => "horizon.volt.block_entity_cap.tooltip",
            Item::VoltAnimThrottle => "horizon.volt.anim_throttle.tooltip",
            Item::VoltActiveAnimCull => "horizon.volt.anim_cull.tooltip",
            Item::VoltFastTranslucency => "horizon.volt.translucency.tooltip",
            Item::VoltFastWeather => "horizon.volt.weather.tooltip",
            Item::VoltCullSignText => "horizon.volt.sign_text.tooltip",
            Item::VoltMobThrottle => "horizon.volt.mob_throttle.tooltip",
            Item::VoltHubShield => "horizon.volt.hub_shield.tooltip",
            Item::VoltMenuBg => "horizon.volt.menu_bg.tooltip",
            Item::VoltLookahead => "horizon.volt.lookahead.tooltip",
            Item::VoltCaveSkyCull => "horizon.volt.cave_sky.tooltip",
            Item::VoltFrustumMesh => "horizon.volt.frustum_mesh.tooltip",
            Item::VoltMemorySaver => "horizon.volt.memory.tooltip",
            Item::VoltPerfHud => "horizon.volt.perf_hud.tooltip",
            Item::VoltPerfHudCorner => "horizon.volt.perf_hud_corner.tooltip",
            Item::VoltShaderEnable => "horizon.volt.shader_enable.tooltip",
            Item::VoltShaderProfile => "horizon.volt.shader_profile.tooltip",
            Item::VoltShaderReset => "horizon.volt.shader_reset.tooltip",
            Item::VoltShaderWavePlants => "horizon.volt.shader_wave_plants.tooltip",
            Item::VoltShaderWaveLeaves => "horizon.volt.shader_wave_leaves.tooltip",
            Item::VoltShaderWaveWater => "horizon.volt.shader_wave_water.tooltip",
            Item::VoltShaderStormWind => "horizon.volt.shader_storm_wind.tooltip",
            Item::VoltShaderSunShading => "horizon.volt.shader_sun_shading.tooltip",
            Item::VoltShaderFoliageGlow => "horizon.volt.shader_foliage_glow.tooltip",
            Item::VoltShaderTorchStyle => "horizon.volt.shader_torch_style.tooltip",
            Item::VoltShaderRainWetness => "horizon.volt.shader_rain_wetness.tooltip",
            Item::VoltShaderWaterRipples => "horizon.volt.shader_water_ripples.tooltip",
            Item::VoltShaderWaterGlint => "horizon.volt.shader_water_glint.tooltip",
            Item::VoltShaderClearWater => "horizon.volt.shader_clear_water.tooltip",
            Item::VoltShaderUnderwaterRefract => "horizon.volt.shader_underwater_refract.tooltip",
            Item::VoltShaderCustomSky => "horizon.volt.shader_custom_sky.tooltip",
            Item::VoltShaderSunHalo => "horizon.volt.shader_sun_halo.tooltip",
            Item::VoltShaderAurora => "horizon.volt.shader_aurora.tooltip",
            Item::VoltShaderDepthHaze => "horizon.volt.shader_depth_haze.tooltip",
            Item::VoltShaderTonemap => "horizon.volt.shader_tonemap.tooltip",
            Item::VoltShaderBloom => "horizon.volt.shader_bloom.tooltip",
            Item::VoltShaderSunRays => "horizon.volt.shader_sun_rays.tooltip",
            Item::VoltShaderVignette => "horizon.volt.shader_vignette.tooltip",
        }
    }
    /// The caption in the current language (English when the key is missing).
    pub fn caption_text(self) -> String {
        crate::lang::get_or(self.caption_key(), self.caption())
    }
    /// The description in the current language (English when the key is missing).
    pub fn description_text(self) -> String {
        crate::lang::get_or(self.description_key(), self.description())
    }
    /// The English caption: the fallback of [`Item::caption_text`], and the text of
    /// `horizon.extras.*` in en_us.json.
    pub fn caption(self) -> &'static str {
        match self {
            Item::Fullbright => "Fullbright",
            Item::DynamicLights => "Dynamic lights",
            Item::OwnNameTag => "Own name tag",
            Item::HideBlockParticles => "Hide block-break particles",
            Item::AutoReconnect => "Auto-reconnect",
            Item::UiSoundReserve => "Keep channels for UI sounds",
            Item::MusicPage => "Music page",
            Item::MouseTweaks => "Mouse Tweaks controls",
            Item::MouseTweaksWheel => "Scroll wheel moves items",
            Item::Sort => "Sort buttons",
            Item::SortKey => "Sort key",
            Item::ChatAnimation => "Smooth chat",
            Item::ChatIndicator => "Message indicator",
            Item::NoAi => "No AI",
            Item::AgentControl => "Agent Control",
            Item::ReproKeys => "Recording shortcuts",
            Item::DiscordPresence => "Discord Rich Presence",
            Item::Splitscreen => "Splitscreen",
            Item::WheelCyclesSettings => "Scroll wheel changes settings",
            Item::CosmeticsMine => "My cosmetics",
            Item::CosmeticsOthers => "Other players' cosmetics",
            // The ported Volt minimap:
            Item::Minimap => "Minimap",
            Item::MinimapCorner => "Corner",
            Item::MinimapSize => "Map Size",
            Item::MinimapShape => "Map Shape",
            Item::MinimapZoom => "Zoom Scale",
            Item::MinimapRotate => "Orientation",
            Item::MinimapFrameStyle => "Frame Style",
            Item::MinimapOpacity => "Map Opacity",
            Item::MinimapZoomKey => "Zoom Key",
            Item::MinimapChunkGrid => "Chunk Grid",
            Item::MinimapTerrain => "Terrain Layer",
            Item::MinimapEntityRadar => "Entity Radar",
            Item::MinimapCoords => "XYZ Readout",
            Item::MinimapBiome => "Heading & Biome",
            Item::MinimapTime => "World Clock",
            // The ported Volt waypoints:
            Item::Waypoints => "Waypoints",
            Item::WaypointKey => "Quick-Add Key",
            Item::WaypointAutoDeath => "Death Waypoints",
            Item::WaypointDimensionSync => "Nether 8x Sync",
            Item::WaypointHudButton => "HUD [+WP] Button",
            Item::WaypointOnMinimap => "Minimap Pins",
            Item::WaypointBeams => "Beacon Beams",
            Item::WaypointLabels => "Floating Labels",
            Item::WaypointDistance => "Show Distance",
            Item::WaypointCoords => "Show XYZ",
            Item::WaypointEdgeArrows => "Edge Arrows",
            Item::WaypointScale => "Marker Scale",
            Item::WaypointMaxDistance => "Max Render Dist",
            Item::VoltPreset => "Boost profile",
            Item::VoltThreadBoost => "big.LITTLE thread boost",
            Item::VoltPerformanceHints => "ADPF performance hints",
            Item::VoltDisplayHz => "Display refresh rate",
            Item::VoltDynamicFps => "Dynamic idle FPS",
            Item::VoltAudioCap => "Audio voice cap",
            Item::VoltAutoDrs => "Auto dynamic resolution",
            Item::VoltThermalGuard => "Thermal guard",
            Item::VoltReflex => "Volt Reflex low latency",
            Item::VoltParticleBudget => "Particle budget",
            Item::VoltChaosShield => "Explosion chaos shield",
            Item::VoltParticleFrustum => "Particle frustum cull",
            Item::VoltEntityCap => "Entity render cap",
            Item::VoltItemCap => "Dropped-item render cap",
            Item::VoltBlockEntityCap => "Block-entity render cap",
            Item::VoltAnimThrottle => "Animation throttle",
            Item::VoltActiveAnimCull => "Active-animation cull",
            Item::VoltFastTranslucency => "Fast translucency",
            Item::VoltFastWeather => "Fast weather",
            Item::VoltCullSignText => "Cull sign text",
            Item::VoltMobThrottle => "Distant mob throttle",
            Item::VoltHubShield => "Hub lag shield",
            Item::VoltMenuBg => "Menu backdrop",
            Item::VoltLookahead => "Streaming lookahead",
            Item::VoltCaveSkyCull => "Cave sky cull",
            Item::VoltFrustumMesh => "Frustum-priority meshing",
            Item::VoltMemorySaver => "Memory saver",
            Item::VoltPerfHud => "Live Perf HUD",
            Item::VoltPerfHudCorner => "Perf HUD Corner",
            Item::VoltShaderEnable => "Enable Shaders",
            Item::VoltShaderProfile => "Shader Profile",
            Item::VoltShaderReset => "Reset Profile",
            Item::VoltShaderWavePlants => "Waving Grass & Plants",
            Item::VoltShaderWaveLeaves => "Waving Leaves & Vines",
            Item::VoltShaderWaveWater => "Waving Water",
            Item::VoltShaderStormWind => "Storm Wind Boost",
            Item::VoltShaderSunShading => "Sun Angle Shading",
            Item::VoltShaderFoliageGlow => "Foliage Subsurface",
            Item::VoltShaderTorchStyle => "Torchlight Color",
            Item::VoltShaderRainWetness => "Rain Wet Sheen",
            Item::VoltShaderWaterRipples => "Water Ripples",
            Item::VoltShaderWaterGlint => "Water Sun Glint",
            Item::VoltShaderClearWater => "Clear Water",
            Item::VoltShaderUnderwaterRefract => "Underwater Wave",
            Item::VoltShaderCustomSky => "Atmospheric Sky",
            Item::VoltShaderSunHalo => "Sun & Moon Halo",
            Item::VoltShaderAurora => "Night Aurora",
            Item::VoltShaderDepthHaze => "Distance Haze",
            Item::VoltShaderTonemap => "Tonemapper",
            Item::VoltShaderBloom => "Bloom",
            Item::VoltShaderSunRays => "Sun Rays",
            Item::VoltShaderVignette => "Vignette",
        }
    }
    /// The switch this setting belongs to: it does nothing while that is off.
    pub fn parent(self) -> Option<Item> {
        use Item::*;
        Some(match self {
            UiSoundReserve => return None,
            MouseTweaksWheel => MouseTweaks,
            SortKey => Sort,
            ChatIndicator => ChatAnimation,
            WaypointKey
            | WaypointAutoDeath
            | WaypointDimensionSync
            | WaypointBeams
            | WaypointLabels
            | WaypointDistance
            | WaypointCoords
            | WaypointEdgeArrows
            | WaypointScale
            | WaypointMaxDistance
            | WaypointHudButton => Waypoints,
            MinimapCorner
            | MinimapSize
            | MinimapShape
            | MinimapZoom
            | MinimapRotate
            | MinimapFrameStyle
            | MinimapOpacity
            | MinimapTerrain
            | MinimapChunkGrid
            | MinimapEntityRadar
            | MinimapCoords
            | MinimapBiome
            | MinimapTime
            | MinimapZoomKey => Minimap,
            Fullbright | DynamicLights | OwnNameTag | HideBlockParticles | MouseTweaks | Sort
            | AutoReconnect | NoAi | AgentControl | ReproKeys | DiscordPresence | Splitscreen
            | ChatAnimation | WheelCyclesSettings | MusicPage | CosmeticsMine | CosmeticsOthers
            | Waypoints | WaypointOnMinimap | Minimap => {
                return None;
            }
            // Each Volt Boost switch stands on its own.
            VoltPreset | VoltThreadBoost | VoltPerformanceHints | VoltDisplayHz
            | VoltDynamicFps | VoltAudioCap | VoltAutoDrs | VoltThermalGuard | VoltReflex
            | VoltParticleBudget | VoltChaosShield | VoltParticleFrustum | VoltEntityCap
            | VoltItemCap | VoltBlockEntityCap | VoltAnimThrottle | VoltActiveAnimCull
            | VoltFastTranslucency | VoltFastWeather | VoltCullSignText | VoltMobThrottle
            | VoltHubShield | VoltMenuBg | VoltLookahead | VoltCaveSkyCull | VoltFrustumMesh
            | VoltMemorySaver | VoltPerfHud | VoltPerfHudCorner | VoltShaderEnable
            | VoltShaderProfile | VoltShaderReset | VoltShaderWavePlants | VoltShaderWaveLeaves
            | VoltShaderWaveWater | VoltShaderStormWind | VoltShaderSunShading
            | VoltShaderFoliageGlow | VoltShaderTorchStyle | VoltShaderRainWetness
            | VoltShaderWaterRipples | VoltShaderWaterGlint | VoltShaderClearWater
            | VoltShaderUnderwaterRefract | VoltShaderCustomSky | VoltShaderSunHalo
            | VoltShaderAurora | VoltShaderDepthHaze | VoltShaderTonemap | VoltShaderBloom
            | VoltShaderSunRays | VoltShaderVignette => {
                return None;
            }
        })
    }
    /// The English description: the fallback of [`Item::description_text`].
    pub fn description(self) -> &'static str {
        match self {
            Item::Fullbright => "Show blocks and entities at full light, even in dark areas.",
            Item::DynamicLights => {
                "Torches and other light sources held, dropped or on fire light the blocks around them."
            }
            Item::OwnNameTag => {
                "Shows your name above your head in third person, as other players see it. Hidden when the server hides your team's names or you're invisible."
            }
            Item::HideBlockParticles => {
                "Hides the debris particles that burst from a broken block and crumble off the one you mine. Other particles stay."
            }
            Item::AutoReconnect => {
                "Rejoins a server after a lost connection or a restart, never after a kick or ban."
            }
            Item::UiSoundReserve => {
                "Sounds in the world leave 16 of the 247 sound channels free, so a flood of them (a big explosion) can't silence menu clicks."
            }
            Item::MusicPage => {
                "Music & Sounds gets a Music... page: play any built-in track now, or turn one off so the game stops picking it. Nothing changes until you turn a track off."
            }
            Item::MouseTweaks => {
                "Right-drag places one per slot, left-drag gathers the held item, Shift + left-drag moves every slot."
            }
            Item::MouseTweaksWheel => {
                "Down sends one item of the slot to the other side, up brings one back."
            }
            Item::Sort => {
                "Sorts the inventory, chests, barrels, ender chests and shulker boxes in creative tab order, merging stacks."
            }
            Item::SortKey => {
                "Sorts the part under the cursor. Click, then press a key; Escape unbinds it."
            }
            Item::ChatAnimation => {
                "A new chat line slides in from the left while it fades in, instead of appearing whole."
            }
            Item::ChatIndicator => "The coloured bar at the left of a chat line.",
            Item::NoAi => {
                "Hides and turns off every AI feature: agent control, horizon-ctl and the MCP servers. The launcher's switch too."
            }
            Item::DiscordPresence => {
                "Shows Playing Horizon on your Discord profile while Discord runs: menus, single player or multiplayer, the dimension and the time played. Never a world's name or a server's address."
            }
            Item::Splitscreen => {
                "In a world, another controller can join as a new player: press A when it asks, then pick a profile and where to play."
            }
            Item::AgentControl => {
                "Your own AI agent can play this game through horizon-ctl or an MCP client, on this computer only."
            }
            Item::ReproKeys => {
                "F3+R starts or stops a repro recording and F3+E writes its bundle. Off by default: a recording copies the world."
            }
            Item::CosmeticsMine => {
                "Shows the cosmetics you wear on your own player, in third person and the inventory."
            }
            Item::CosmeticsOthers => {
                "Shows the cosmetics other Horizon players wear. Off, the game never asks what they wear."
            }
            Item::WheelCyclesSettings => {
                "The scroll wheel over a button in the settings changes its value, as in vanilla. Off, the wheel only scrolls the list."
            }
            // The ported Volt minimap:
            Item::Minimap => {
                "Master switch for Volt's minimal, feature-rich live world minimap (off by default)."
            }
            Item::MinimapCorner => {
                "Screen corner where the minimap and its info pills dock, automatically avoiding status effects."
            }
            Item::MinimapSize => {
                "Minimap side length in GUI pixels (Small 64, Medium 96, Large 128, XL 160)."
            }
            Item::MinimapShape => {
                "Switches between a crisp square minimap and a circular compass minimap."
            }
            Item::MinimapZoom => {
                "Map scale in blocks per pixel (0.5x zoomed-in detail up to 4x wide-area overview)."
            }
            Item::MinimapRotate => {
                "North Up keeps north at the top; Rotate turns the map and N/E/S/W compass ring with your look direction."
            }
            Item::MinimapFrameStyle => {
                "Chooses between Volt Crimson, Classic Dark and ultra-clean Minimal border bezels."
            }
            Item::MinimapOpacity => {
                "Controls minimap terrain translucency (50%, 75%, 90% or 100%)."
            }
            Item::MinimapZoomKey => {
                "Press in-game to cycle minimap zoom levels on the fly. Escape unbinds."
            }
            Item::MinimapChunkGrid => {
                "Overlays subtle 16x16 chunk boundary lines across the minimap."
            }
            Item::MinimapTerrain => {
                "Auto switches to a cave slice when underground; Surface always shows top sky-lit blocks; Cave always slices near head height."
            }
            Item::MinimapEntityRadar => {
                "Plots live dots for nearby Players (cyan), Hostile mobs (red), Passive mobs (green) and Dropped items (gold)."
            }
            Item::MinimapCoords => {
                "Shows a compact X, Y, Z coordinate readout pill beneath the minimap."
            }
            Item::MinimapBiome => {
                "Shows your cardinal heading and current biome beneath the minimap."
            }
            Item::MinimapTime => {
                "Shows the current in-game Day number and 24-hour clock beneath the minimap."
            }
            // The ported Volt waypoints:
            Item::Waypoints => {
                "Master switch for Volt Waypoints (off by default). Renders 3D world markers, beacon beams, death pins and per-world/server saved locations."
            }
            Item::WaypointKey => {
                "Press in-game to drop a new waypoint at your current coordinates immediately. Escape unbinds."
            }
            Item::WaypointAutoDeath => {
                "Automatically drops a Crimson 'Last Death' waypoint whenever you die in singleplayer or multiplayer (keeps up to 3 recent deaths)."
            }
            Item::WaypointDimensionSync => {
                "Projects Overworld waypoints into the Nether at 1/8 X/Z scale (and Nether waypoints into the Overworld at 8x scale)."
            }
            Item::WaypointHudButton => {
                "Adds a compact [+WP] quick-pin button on the touch/in-game HUD for one-tap waypoint creation on mobile."
            }
            Item::WaypointOnMinimap => {
                "Pins active waypoints onto the Volt Minimap and clamps distant markers onto the map border."
            }
            Item::WaypointBeams => {
                "Draws vertical translucent beacon light columns at nearby active waypoints."
            }
            Item::WaypointLabels => {
                "Shows floating diamond icons and nameplates in 3D space."
            }
            Item::WaypointDistance => {
                "Displays live distance in metres next to each waypoint label."
            }
            Item::WaypointCoords => {
                "Always shows [X, Y, Z] coordinates on waypoint badges (even when off, aiming directly at a marker reveals its coordinates)."
            }
            Item::WaypointEdgeArrows => {
                "Shows sleek coloured directional chevrons along the screen rim pointing toward off-screen waypoints."
            }
            Item::WaypointScale => {
                "Adjusts HUD marker and nameplate size (75%, 100% or 125%)."
            }
            Item::WaypointMaxDistance => {
                "Hides in-world waypoint markers beyond the chosen distance (Unlimited, 256m, 512m, 1024m or 2048m)."
            }
            Item::VoltPreset => "One tap sets every Volt Boost switch below to a profile; Off is vanilla.",
            Item::VoltThreadBoost => "Tightens the game threads’ timer slack on Android so frame pacing wakes crisp on 90 and 120 Hz displays.",
            Item::VoltPerformanceHints => "Lets Android raise the clock when a frame runs late (the ADPF hint session).",
            Item::VoltDisplayHz => "Asks the screen for this rate every frame; System default follows the frame cap.",
            Item::VoltDynamicFps => "A stronger cap while paused, in a menu or showing no world: 30 or 15 FPS.",
            Item::VoltAudioCap => "Caps a tick’s particle sounds (a farm or a TNT blast) at 8 or 4.",
            Item::VoltAutoDrs => "Eases the world’s resolution towards a frame target; the HUD stays native.",
            Item::VoltThermalGuard => "Under load, eases the resolution and the particle budget down until the phone cools.",
            Item::VoltReflex => "Works the GPU’s queue before the frame’s sleep and paces precisely while playing.",
            Item::VoltParticleBudget => "The active particle cap and the distance past which nothing renders.",
            Item::VoltChaosShield => "An explosion throws at most 24 debris particles, and a far one starts no firework show.",
            Item::VoltParticleFrustum => "Particles behind the camera skip their sprite and light lookups.",
            Item::VoltEntityCap => "Past the cap only the nearest mobs draw; the ones you fight never vanish first.",
            Item::VoltItemCap => "Past the cap only the nearest dropped items draw, with a distance cut.",
            Item::VoltBlockEntityCap => "Past the cap only the nearest chests, signs and banners draw.",
            Item::VoltAnimThrottle => "Animated textures upload every second tick (10 Hz instead of vanilla’s 20).",
            Item::VoltActiveAnimCull => "Skips animated textures no loaded chunk uses.",
            Item::VoltFastTranslucency => "Distant water and glass re-sort on a coarser grid with a smaller budget.",
            Item::VoltFastWeather => "Rain and snow draw within four blocks instead of fifteen.",
            Item::VoltCullSignText => "Signs’ standing text is skipped past sixteen blocks; the block stays.",
            Item::VoltMobThrottle => "Far mobs’ bodies and player layers draw without their animation alpha.",
            Item::VoltHubShield => "With more than 24 mobs, players beyond nametag range give up their render slot first.",
            Item::VoltMenuBg => "FastDark skips the multi-pass menu blur for a plain dark backdrop.",
            Item::VoltLookahead => "Chunks behind the movement and the look generate later than the ones ahead.",
            Item::VoltCaveSkyCull => "Sealed underground in the Overworld, the weather draw is skipped.",
            Item::VoltFrustumMesh => "Sections behind the view upload after every visible one.",
            Item::VoltMemorySaver => "On a world or dimension change, emptied queues and freed pages go back to the OS.",
            Item::VoltPerfHud => {
                "Shows a telemetry overlay with live FPS, 1% lows, frame pacing, active entities, chunks and resident RAM."
            }
            Item::VoltPerfHudCorner => "Screen corner where the live performance HUD docks.",
            Item::VoltShaderEnable => {
                "Master switch for the built-in native WGSL Volt Shaders (off by default). When OFF, no shader passes run and the customization controls stay hidden."
            }
            Item::VoltShaderProfile => {
                "Switches between the Potato, Low, Medium, High, Ultra and Custom native WGSL shader profiles tuned for mobile TBDR GPUs."
            }
            Item::VoltShaderReset => "Restores every Volt Shaders setting back to the active profile's defaults.",
            Item::VoltShaderWavePlants => {
                "Animates short grass, tall grass, ferns, flowers, crops and saplings with root-anchored vertex wind."
            }
            Item::VoltShaderWaveLeaves => {
                "Animates tree leaves, vines, hanging moss and kelp with seamless phase-coherent canopy sway."
            }
            Item::VoltShaderWaveWater => {
                "Adds gentle vertex surface motion to still water within 56 blocks at near-zero vertex cost."
            }
            Item::VoltShaderStormWind => {
                "Intensifies vegetation wind speed and sway amplitude dynamically during rain and thunderstorms."
            }
            Item::VoltShaderSunShading => {
                "Applies directional sunlight and sky bounce shading across terrain faces without shadow-map overhead."
            }
            Item::VoltShaderFoliageGlow => {
                "Adds warm sunlit subsurface translucency when looking through leaves, grass and crops toward the sun."
            }
            Item::VoltShaderTorchStyle => {
                "Selects block-light colour temperature for torches, lanterns and campfires (Vanilla, Warm Amber, Cozy Gold or Soul Cool)."
            }
            Item::VoltShaderRainWetness => {
                "Adds sky-exposed wet surface sheen and specular sky reflection on terrain blocks while raining."
            }
            Item::VoltShaderWaterRipples => {
                "Computes dual-wave procedural surface ripples on water with no extra texture fetches."
            }
            Item::VoltShaderWaterGlint => {
                "Renders crisp sun and moon specular highlights dancing across rippled water surfaces."
            }
            Item::VoltShaderClearWater => {
                "Clarifies shallow water alpha and deepens turquoise-to-abyss absorption."
            }
            Item::VoltShaderUnderwaterRefract => {
                "Applies subtle screen-space underwater wave refraction when the camera is submerged."
            }
            Item::VoltShaderCustomSky => {
                "Replaces the sky dome with a smooth zenith-to-horizon Rayleigh and Mie atmospheric gradient."
            }
            Item::VoltShaderSunHalo => {
                "Adds soft golden sun and cool silver moon atmospheric corona glow."
            }
            Item::VoltShaderAurora => {
                "Renders procedural northern lights ribbons across clear night skies (OFF, Subtle, Medium or Vibrant)."
            }
            Item::VoltShaderDepthHaze => {
                "Blends distant terrain into sun-tinted atmospheric perspective haze (OFF, Subtle or Lush)."
            }
            Item::VoltShaderTonemap => {
                "Selects the HDR-to-SDR tonemapping curve (OFF, Filmic ACES, Crisp Neutral or Lush Cinema)."
            }
            Item::VoltShaderBloom => {
                "Adds soft single-pass emissive and sky bloom with no intermediate render targets (OFF, Low 4-tap or High 8-tap)."
            }
            Item::VoltShaderSunRays => {
                "Casts lightweight depth-occluded crepuscular sun shafts when looking near the sun."
            }
            Item::VoltShaderVignette => {
                "Applies a gentle cinematic edge darkening around the frame (OFF, Subtle or Strong)."
            }
        }
    }
    /// The frame-time cost the main switch of an extra shows.
    pub fn cost(self) -> Option<crate::graphics_style::Cost> {
        use crate::graphics_style::Cost;
        match self {
            Item::Fullbright => Some(Cost::None),
            Item::UiSoundReserve => Some(Cost::None),
            Item::MusicPage => Some(Cost::None),
            Item::DynamicLights => Some(Cost::Low),
            Item::ChatAnimation => Some(Cost::None),
            Item::OwnNameTag
            | Item::HideBlockParticles
            | Item::AutoReconnect
            | Item::MouseTweaks
            | Item::Sort
            | Item::Splitscreen
            | Item::WheelCyclesSettings
            | Item::Waypoints => Some(Cost::None),
            Item::CosmeticsMine | Item::CosmeticsOthers => Some(Cost::Low),
            Item::Minimap => Some(Cost::Low),
            _ => None,
        }
    }
    /// On/off items and their value.
    pub fn switch(self, e: &Extras) -> Option<bool> {
        Some(match self {
            Item::Fullbright => e.fullbright,
            Item::DynamicLights => e.dynamic_lights,
            Item::OwnNameTag => e.own_name_tag,
            Item::HideBlockParticles => e.hide_block_particles,
            Item::AutoReconnect => e.auto_reconnect,
            Item::UiSoundReserve => e.ui_sound_reserve,
            Item::MusicPage => e.music_page,
            Item::MouseTweaks => e.mouse_tweaks,
            Item::MouseTweaksWheel => e.mouse_tweaks_wheel,
            Item::Sort => e.sort,
            Item::ChatAnimation => e.chat_animation,
            Item::ChatIndicator => e.chat_indicator,
            Item::NoAi => e.no_ai,
            Item::AgentControl => e.agent_control,
            Item::ReproKeys => e.repro_keys,
            Item::DiscordPresence => e.discord_presence,
            Item::Splitscreen => e.splitscreen,
            Item::WheelCyclesSettings => e.wheel_cycles_settings,
            Item::CosmeticsMine => e.cosmetics_mine,
            Item::CosmeticsOthers => e.cosmetics_others,
            // The ported Volt minimap:
            Item::Minimap => e.minimap,
            Item::MinimapChunkGrid => e.minimap_chunk_grid,
            Item::MinimapCoords => e.minimap_coords,
            Item::MinimapBiome => e.minimap_biome,
            Item::MinimapTime => e.minimap_time,
            // The ported Volt waypoints:
            Item::Waypoints => e.waypoints,
            Item::WaypointAutoDeath => e.waypoint_auto_death,
            Item::WaypointDimensionSync => e.waypoint_dimension_sync,
            Item::WaypointBeams => e.waypoint_beams,
            Item::WaypointLabels => e.waypoint_labels,
            Item::WaypointDistance => e.waypoint_distance,
            Item::WaypointCoords => e.waypoint_coords,
            Item::WaypointEdgeArrows => e.waypoint_edge_arrows,
            Item::WaypointOnMinimap => e.waypoint_on_minimap,
            Item::WaypointHudButton => e.waypoint_hud_button,
            Item::VoltThreadBoost => e.thread_boost,
            Item::VoltPerformanceHints => e.performance_hints,
            Item::VoltThermalGuard => e.thermal_guard,
            Item::VoltReflex => e.volt_reflex,
            Item::VoltChaosShield => e.chaos_particle_shield,
            Item::VoltParticleFrustum => e.particle_frustum_cull,
            Item::VoltAnimThrottle => e.anim_throttle,
            Item::VoltActiveAnimCull => e.active_anim_cull,
            Item::VoltFastTranslucency => e.fast_translucency,
            Item::VoltFastWeather => e.fast_weather,
            Item::VoltCullSignText => e.cull_sign_text,
            Item::VoltMobThrottle => e.distant_mob_throttle,
            Item::VoltHubShield => e.hub_lag_shield,
            Item::VoltLookahead => e.lookahead_stream,
            Item::VoltCaveSkyCull => e.cave_sky_cull,
            Item::VoltFrustumMesh => e.frustum_priority_mesh,
            Item::VoltMemorySaver => e.memory_saver,
            Item::VoltShaderEnable => e.volt_shaders.enabled,
            Item::VoltShaderWavePlants => e.volt_shaders.waving_plants,
            Item::VoltShaderWaveLeaves => e.volt_shaders.waving_leaves,
            Item::VoltShaderWaveWater => e.volt_shaders.waving_water,
            Item::VoltShaderStormWind => e.volt_shaders.storm_wind,
            Item::VoltShaderSunShading => e.volt_shaders.sun_shading,
            Item::VoltShaderFoliageGlow => e.volt_shaders.foliage_subsurface,
            Item::VoltShaderRainWetness => e.volt_shaders.rain_wetness,
            Item::VoltShaderWaterRipples => e.volt_shaders.water_ripples,
            Item::VoltShaderWaterGlint => e.volt_shaders.water_glint,
            Item::VoltShaderClearWater => e.volt_shaders.clear_water,
            Item::VoltShaderUnderwaterRefract => e.volt_shaders.underwater_refraction,
            Item::VoltShaderCustomSky => e.volt_shaders.custom_sky,
            Item::VoltShaderSunHalo => e.volt_shaders.sun_halo,
            Item::VoltShaderSunRays => e.volt_shaders.sun_rays,
            _ => return None,
        })
    }
    /// The value of the other items, as their button shows it.
    pub fn value(self, e: &Extras) -> String {
        match self {
            Item::SortKey => crate::keybinds::display_name(&e.sort_key).plain(),
            Item::WaypointKey => crate::keybinds::display_name(&e.waypoint_key).plain(),
            Item::MinimapZoomKey => crate::keybinds::display_name(&e.minimap_zoom_key).plain(),
            Item::WaypointScale => format!("{}%", e.waypoint_scale),
            Item::WaypointMaxDistance => {
                if e.waypoint_max_distance == 0 {
                    crate::lang::get_or("horizon.volt.waypoints.unlimited", "Unlimited")
                } else {
                    format!("{}m", e.waypoint_max_distance)
                }
            }
            Item::MinimapCorner => e.minimap_corner.label().into(),
            Item::MinimapSize => {
                let name = MINIMAP_SIZES
                    .iter()
                    .find(|s| s.0 == e.minimap_size)
                    .map_or("Medium", |s| s.1);
                crate::lang::get_or(
                    &format!("horizon.volt.minimap.size.{}", e.minimap_size),
                    name,
                )
            }
            Item::MinimapShape => if e.minimap_round {
                crate::lang::get_or("horizon.volt.minimap.round", "Round")
            } else {
                crate::lang::get_or("horizon.volt.minimap.square", "Square")
            }
            .into(),
            Item::MinimapZoom => format!("{}x", e.minimap_zoom),
            Item::MinimapRotate => if e.minimap_rotate {
                crate::lang::get_or("horizon.volt.minimap.rotated", "Rotate")
            } else {
                crate::lang::get_or("horizon.volt.minimap.north_up", "North Up")
            }
            .into(),
            Item::MinimapFrameStyle => e.minimap_frame.label().into(),
            Item::MinimapOpacity => format!("{}%", e.minimap_opacity),
            Item::MinimapTerrain => e.minimap_terrain_mode.label().into(),
            Item::MinimapEntityRadar => e.minimap_entities.label().into(),
            Item::VoltPreset => {
                use crate::volt_features::VoltBoostPreset;
                match e.volt_preset {
                    VoltBoostPreset::Off => crate::lang::get_or("options.off", "Off"),
                    p => crate::lang::get_or(p.label_key(), p.label()),
                }
            }
            Item::VoltDisplayHz => {
                if e.display_hz == 0 {
                    crate::lang::get_or("horizon.volt.hz.system", "System default")
                } else {
                    crate::lang::get_or("horizon.volt.hz", "%s Hz")
                        .replace("%s", &e.display_hz.to_string())
                }
            }
            Item::VoltDynamicFps => {
                use crate::volt_features::DynamicIdleFps;
                match e.dynamic_fps {
                    DynamicIdleFps::Off => crate::lang::get_or("options.off", "Off"),
                    DynamicIdleFps::Fps30 => crate::lang::get_or("horizon.volt.idle30", "30 FPS"),
                    DynamicIdleFps::Fps15 => crate::lang::get_or("horizon.volt.idle15", "15 FPS"),
                }
            }
            Item::VoltAudioCap => crate::lang::get_or(
                &format!("horizon.volt.audio.{}", e.audio_voice_cap.as_str()),
                e.audio_voice_cap.label(),
            ),
            Item::VoltAutoDrs => crate::lang::get_or(
                &format!("horizon.volt.drs.{}", e.auto_drs.as_str()),
                e.auto_drs.label(),
            ),
            Item::VoltParticleBudget => crate::lang::get_or(
                &format!("horizon.volt.particles.{}", e.particle_budget.as_str()),
                e.particle_budget.label(),
            ),
            Item::VoltEntityCap => crate::lang::get_or(
                &format!("horizon.volt.entity.{}", e.entity_cap.as_str()),
                e.entity_cap.label(),
            ),
            Item::VoltItemCap => crate::lang::get_or(
                &format!("horizon.volt.items.{}", e.item_render_cap.as_str()),
                e.item_render_cap.label(),
            ),
            Item::VoltBlockEntityCap => crate::lang::get_or(
                &format!("horizon.volt.block_entities.{}", e.block_entity_cap.as_str()),
                e.block_entity_cap.label(),
            ),
            Item::VoltMenuBg => crate::lang::get_or(e.menu_bg_mode.label_key(), e.menu_bg_mode.as_str()),
            Item::VoltPerfHud => match e.perf_hud {
                crate::volt_features::PerfHudMode::Off => crate::lang::get_or("options.off", "Off"),
                crate::volt_features::PerfHudMode::Compact => {
                    crate::lang::get_or("horizon.volt.perf_hud.compact", "Compact")
                }
                crate::volt_features::PerfHudMode::Full => {
                    crate::lang::get_or("horizon.volt.perf_hud.full", "Full Graph")
                }
            },
            Item::VoltPerfHudCorner => e.perf_hud_corner.label().into(),
            Item::VoltShaderEnable => {
                if e.volt_shaders.enabled {
                    crate::lang::get_or("options.on", "ON")
                } else {
                    crate::lang::get_or("options.off", "OFF")
                }
            }
            Item::VoltShaderProfile => e.volt_shaders.profile.label().into(),
            Item::VoltShaderReset => "Restore".into(),
            Item::VoltShaderTorchStyle => e.volt_shaders.torch_style.label().into(),
            Item::VoltShaderAurora => e.volt_shaders.aurora.label().into(),
            Item::VoltShaderDepthHaze => e.volt_shaders.depth_haze.label().into(),
            Item::VoltShaderTonemap => e.volt_shaders.tonemap.label().into(),
            Item::VoltShaderBloom => e.volt_shaders.bloom.label().into(),
            Item::VoltShaderVignette => e.volt_shaders.vignette.label().into(),
            other => match other.switch(e) {
                Some(true) => crate::lang::get_or("On", "On"),
                _ => crate::lang::get_or("Off", "Off"),
            },
        }
    }
    /// A click (`_back` for the right button: every item left is a switch,
    /// so both flip it). The sort key binds instead.
    pub fn cycle(self, e: &mut Extras, _back: bool) {
        // The ported Volt enums step through their own value lists.
        fn step<T: Copy + PartialEq>(slice: &[T], cur: T, back: bool) -> T {
            let idx = slice.iter().position(|&v| v == cur).unwrap_or(0);
            slice[if back {
                (idx + slice.len() - 1) % slice.len()
            } else {
                (idx + 1) % slice.len()
            }]
        }
        match self {
            Item::Fullbright => e.fullbright = !e.fullbright,
            Item::DynamicLights => e.dynamic_lights = !e.dynamic_lights,
            Item::OwnNameTag => e.own_name_tag = !e.own_name_tag,
            Item::HideBlockParticles => e.hide_block_particles = !e.hide_block_particles,
            Item::AutoReconnect => e.auto_reconnect = !e.auto_reconnect,
            Item::UiSoundReserve => e.ui_sound_reserve = !e.ui_sound_reserve,
            Item::MusicPage => e.music_page = !e.music_page,
            Item::MouseTweaks => e.mouse_tweaks = !e.mouse_tweaks,
            Item::MouseTweaksWheel => e.mouse_tweaks_wheel = !e.mouse_tweaks_wheel,
            Item::Sort => e.sort = !e.sort,
            Item::ChatAnimation => e.chat_animation = !e.chat_animation,
            Item::ChatIndicator => e.chat_indicator = !e.chat_indicator,
            Item::NoAi => e.no_ai = !e.no_ai,
            Item::AgentControl => e.agent_control = !e.agent_control,
            Item::ReproKeys => e.repro_keys = !e.repro_keys,
            Item::DiscordPresence => e.discord_presence = !e.discord_presence,
            Item::Splitscreen => e.splitscreen = !e.splitscreen,
            Item::WheelCyclesSettings => e.wheel_cycles_settings = !e.wheel_cycles_settings,
            Item::CosmeticsMine => e.cosmetics_mine = !e.cosmetics_mine,
            Item::CosmeticsOthers => e.cosmetics_others = !e.cosmetics_others,
            // The ported Volt minimap:
            Item::Minimap => e.minimap = !e.minimap,
            Item::MinimapCorner => e.minimap_corner = step(&Corner::ALL, e.minimap_corner, _back),
            Item::MinimapSize => {
                let sizes: [u8; 4] = MINIMAP_SIZES.map(|s| s.0);
                e.minimap_size = step(&sizes, e.minimap_size, _back);
            }
            Item::MinimapShape => e.minimap_round = !e.minimap_round,
            Item::MinimapZoom => e.minimap_zoom = step(&MINIMAP_ZOOMS, e.minimap_zoom, _back),
            Item::MinimapRotate => e.minimap_rotate = !e.minimap_rotate,
            Item::MinimapFrameStyle => e.minimap_frame = e.minimap_frame.cycle(_back),
            Item::MinimapOpacity => {
                e.minimap_opacity = step(&MINIMAP_OPACITIES, e.minimap_opacity, _back)
            }
            Item::MinimapTerrain => e.minimap_terrain_mode = e.minimap_terrain_mode.cycle(_back),
            Item::MinimapChunkGrid => e.minimap_chunk_grid = !e.minimap_chunk_grid,
            Item::MinimapEntityRadar => e.minimap_entities = e.minimap_entities.cycle(_back),
            Item::MinimapCoords => e.minimap_coords = !e.minimap_coords,
            Item::MinimapBiome => e.minimap_biome = !e.minimap_biome,
            Item::MinimapTime => e.minimap_time = !e.minimap_time,
            // The ported Volt waypoints:
            Item::Waypoints => e.waypoints = !e.waypoints,
            Item::WaypointAutoDeath => e.waypoint_auto_death = !e.waypoint_auto_death,
            Item::WaypointDimensionSync => e.waypoint_dimension_sync = !e.waypoint_dimension_sync,
            Item::WaypointHudButton => e.waypoint_hud_button = !e.waypoint_hud_button,
            Item::WaypointOnMinimap => e.waypoint_on_minimap = !e.waypoint_on_minimap,
            Item::WaypointBeams => e.waypoint_beams = !e.waypoint_beams,
            Item::WaypointLabels => e.waypoint_labels = !e.waypoint_labels,
            Item::WaypointDistance => e.waypoint_distance = !e.waypoint_distance,
            Item::WaypointCoords => e.waypoint_coords = !e.waypoint_coords,
            Item::WaypointEdgeArrows => e.waypoint_edge_arrows = !e.waypoint_edge_arrows,
            Item::WaypointScale => e.waypoint_scale = step(&WAYPOINT_SCALES, e.waypoint_scale, _back),
            Item::WaypointMaxDistance => {
                e.waypoint_max_distance = step(&WAYPOINT_DISTANCES, e.waypoint_max_distance, _back)
            }
            Item::VoltPreset => e.apply_volt_preset(e.volt_preset.next()),
            Item::VoltThreadBoost => e.thread_boost = !e.thread_boost,
            Item::VoltPerformanceHints => e.performance_hints = !e.performance_hints,
            Item::VoltDisplayHz => {
                let rates = DISPLAY_RATES;
                let i = rates.iter().position(|&hz| hz == e.display_hz).unwrap_or(0);
                e.display_hz = rates[(i + 1) % rates.len()];
            }
            Item::VoltDynamicFps => e.dynamic_fps = e.dynamic_fps.next(),
            Item::VoltAudioCap => e.audio_voice_cap = e.audio_voice_cap.next(),
            Item::VoltAutoDrs => e.auto_drs = e.auto_drs.next(),
            Item::VoltThermalGuard => e.thermal_guard = !e.thermal_guard,
            Item::VoltReflex => e.volt_reflex = !e.volt_reflex,
            Item::VoltParticleBudget => e.particle_budget = e.particle_budget.next(),
            Item::VoltChaosShield => e.chaos_particle_shield = !e.chaos_particle_shield,
            Item::VoltParticleFrustum => e.particle_frustum_cull = !e.particle_frustum_cull,
            Item::VoltEntityCap => e.entity_cap = e.entity_cap.next(),
            Item::VoltItemCap => e.item_render_cap = e.item_render_cap.next(),
            Item::VoltBlockEntityCap => e.block_entity_cap = e.block_entity_cap.next(),
            Item::VoltAnimThrottle => e.anim_throttle = !e.anim_throttle,
            Item::VoltActiveAnimCull => e.active_anim_cull = !e.active_anim_cull,
            Item::VoltFastTranslucency => e.fast_translucency = !e.fast_translucency,
            Item::VoltFastWeather => e.fast_weather = !e.fast_weather,
            Item::VoltCullSignText => e.cull_sign_text = !e.cull_sign_text,
            Item::VoltMobThrottle => e.distant_mob_throttle = !e.distant_mob_throttle,
            Item::VoltHubShield => e.hub_lag_shield = !e.hub_lag_shield,
            Item::VoltMenuBg => e.menu_bg_mode = e.menu_bg_mode.next(),
            Item::VoltLookahead => e.lookahead_stream = !e.lookahead_stream,
            Item::VoltCaveSkyCull => e.cave_sky_cull = !e.cave_sky_cull,
            Item::VoltFrustumMesh => e.frustum_priority_mesh = !e.frustum_priority_mesh,
            Item::VoltMemorySaver => e.memory_saver = !e.memory_saver,
            Item::VoltPerfHud => {
                e.perf_hud = step(&crate::volt_features::PerfHudMode::ALL, e.perf_hud, _back)
            }
            Item::VoltPerfHudCorner => {
                e.perf_hud_corner = step(&Corner::ALL, e.perf_hud_corner, _back)
            }
            Item::VoltShaderEnable => e.volt_shaders.enabled = !e.volt_shaders.enabled,
            Item::VoltShaderProfile => {
                use horizon_shader_api::volt_shaders::VoltShaderProfile;
                let presets = [
                    VoltShaderProfile::Potato,
                    VoltShaderProfile::Low,
                    VoltShaderProfile::Medium,
                    VoltShaderProfile::High,
                    VoltShaderProfile::Ultra,
                ];
                let chosen = match e.volt_shaders.profile {
                    VoltShaderProfile::Custom => {
                        if _back {
                            VoltShaderProfile::Ultra
                        } else {
                            VoltShaderProfile::Potato
                        }
                    }
                    cur => step(&presets, cur, _back),
                };
                e.volt_shaders.apply_profile(chosen);
            }
            Item::VoltShaderReset => {
                e.volt_shaders.reset_current_profile();
            }
            Item::VoltShaderWavePlants => {
                e.volt_shaders.waving_plants = !e.volt_shaders.waving_plants;
                e.volt_shaders.mark_custom();
            }
            Item::VoltShaderWaveLeaves => {
                e.volt_shaders.waving_leaves = !e.volt_shaders.waving_leaves;
                e.volt_shaders.mark_custom();
            }
            Item::VoltShaderWaveWater => {
                e.volt_shaders.waving_water = !e.volt_shaders.waving_water;
                e.volt_shaders.mark_custom();
            }
            Item::VoltShaderStormWind => {
                e.volt_shaders.storm_wind = !e.volt_shaders.storm_wind;
                e.volt_shaders.mark_custom();
            }
            Item::VoltShaderSunShading => {
                e.volt_shaders.sun_shading = !e.volt_shaders.sun_shading;
                e.volt_shaders.mark_custom();
            }
            Item::VoltShaderFoliageGlow => {
                e.volt_shaders.foliage_subsurface = !e.volt_shaders.foliage_subsurface;
                e.volt_shaders.mark_custom();
            }
            Item::VoltShaderTorchStyle => {
                e.volt_shaders.torch_style = step(
                    &horizon_shader_api::volt_shaders::TorchLightStyle::ALL,
                    e.volt_shaders.torch_style,
                    _back,
                );
                e.volt_shaders.mark_custom();
            }
            Item::VoltShaderRainWetness => {
                e.volt_shaders.rain_wetness = !e.volt_shaders.rain_wetness;
                e.volt_shaders.mark_custom();
            }
            Item::VoltShaderWaterRipples => {
                e.volt_shaders.water_ripples = !e.volt_shaders.water_ripples;
                e.volt_shaders.mark_custom();
            }
            Item::VoltShaderWaterGlint => {
                e.volt_shaders.water_glint = !e.volt_shaders.water_glint;
                e.volt_shaders.mark_custom();
            }
            Item::VoltShaderClearWater => {
                e.volt_shaders.clear_water = !e.volt_shaders.clear_water;
                e.volt_shaders.mark_custom();
            }
            Item::VoltShaderUnderwaterRefract => {
                e.volt_shaders.underwater_refraction = !e.volt_shaders.underwater_refraction;
                e.volt_shaders.mark_custom();
            }
            Item::VoltShaderCustomSky => {
                e.volt_shaders.custom_sky = !e.volt_shaders.custom_sky;
                e.volt_shaders.mark_custom();
            }
            Item::VoltShaderSunHalo => {
                e.volt_shaders.sun_halo = !e.volt_shaders.sun_halo;
                e.volt_shaders.mark_custom();
            }
            Item::VoltShaderAurora => {
                e.volt_shaders.aurora = step(
                    &horizon_shader_api::volt_shaders::AuroraMode::ALL,
                    e.volt_shaders.aurora,
                    _back,
                );
                e.volt_shaders.mark_custom();
            }
            Item::VoltShaderDepthHaze => {
                e.volt_shaders.depth_haze = step(
                    &horizon_shader_api::volt_shaders::DepthHazeMode::ALL,
                    e.volt_shaders.depth_haze,
                    _back,
                );
                e.volt_shaders.mark_custom();
            }
            Item::VoltShaderTonemap => {
                e.volt_shaders.tonemap = step(
                    &horizon_shader_api::volt_shaders::TonemapMode::ALL,
                    e.volt_shaders.tonemap,
                    _back,
                );
                e.volt_shaders.mark_custom();
            }
            Item::VoltShaderBloom => {
                e.volt_shaders.bloom = step(
                    &horizon_shader_api::volt_shaders::BloomMode::ALL,
                    e.volt_shaders.bloom,
                    _back,
                );
                e.volt_shaders.mark_custom();
            }
            Item::VoltShaderSunRays => {
                e.volt_shaders.sun_rays = !e.volt_shaders.sun_rays;
                e.volt_shaders.mark_custom();
            }
            Item::VoltShaderVignette => {
                e.volt_shaders.vignette = step(
                    &horizon_shader_api::volt_shaders::VignetteMode::ALL,
                    e.volt_shaders.vignette,
                    _back,
                );
                e.volt_shaders.mark_custom();
            }
            Item::SortKey | Item::WaypointKey | Item::MinimapZoomKey => {}
        }
    }
    /// The key name the options screen waits on for a key item.
    pub fn key_name(self) -> Option<&'static str> {
        match self {
            Item::SortKey => Some(SORT_KEY),
            Item::WaypointKey => Some(WAYPOINT_KEY),
            Item::MinimapZoomKey => Some(MINIMAP_ZOOM_KEY),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn everything_is_off_by_default_and_round_trips() {
        let d = Extras::default();
        assert!(!d.fullbright && !d.dynamic_lights);
        assert!(!d.zoom && !d.fps && !d.keystrokes && !d.minimap && !d.auto_reconnect);
        assert!(!d.mouse_tweaks && !d.sort && !d.own_name_tag && !d.ui_sound_reserve);
        assert!(!d.hide_block_particles);
        assert!(!d.music_page && d.music_off.is_empty());
        assert!(d.discord_presence && d.wheel_cycles_settings);
        assert_eq!(Extras::parse(""), d);
        assert_eq!(Extras::parse(&d.serialize()), d);
        let mut e = d.clone();
        Item::Fullbright.cycle(&mut e, false);
        assert_eq!(Item::Fullbright.switch(&e), Some(true));
        Item::DynamicLights.cycle(&mut e, false);
        assert_eq!(Item::DynamicLights.switch(&e), Some(true));
        assert_eq!(Extras::parse(&e.serialize()), e);
        e.zoom = true;
        e.zoom_key = "key.keyboard.c".into();
        e.zoom_level = 6;
        e.fps_corner = Corner::BottomRight;
        e.keystrokes = true;
        e.keystrokes_cps = false;
        e.moved = vec!["fps_overlay".into(), "keystrokes".into()];
        e.minimap = true;
        e.minimap_size = 128;
        e.minimap_round = true;
        e.minimap_zoom = 0.5;
        e.minimap_rotate = true;
        e.minimap_corner = Corner::BottomLeft;
        e.auto_reconnect = true;
        Item::OwnNameTag.cycle(&mut e, false);
        assert_eq!(Item::OwnNameTag.switch(&e), Some(true));
        assert_eq!(Item::HideBlockParticles.switch(&e), Some(false));
        Item::HideBlockParticles.cycle(&mut e, false);
        assert!(e.hide_block_particles);
        assert_eq!(Item::HideBlockParticles.switch(&e), Some(true));
        let only = Extras::parse("hideBlockParticles:true\n");
        assert!(only.hide_block_particles && !only.own_name_tag);
        e.mouse_tweaks = true;
        e.mouse_tweaks_wheel = false;
        e.sort = true;
        e.sort_key = "key.keyboard.r".into();
        e.chat_animation = true;
        e.chat_indicator = false;
        e.discord_presence = false;
        assert!(!e.splitscreen);
        Item::Splitscreen.cycle(&mut e, false);
        assert_eq!(Item::Splitscreen.switch(&e), Some(true));
        Item::WheelCyclesSettings.cycle(&mut e, false);
        assert_eq!(Item::WheelCyclesSettings.switch(&e), Some(false));
        e.music_off = vec![
            "minecraft:music/game/calm1".into(),
            "minecraft:music.menu".into(),
        ];
        assert!(!e.music_page);
        Item::MusicPage.cycle(&mut e, false);
        assert_eq!(Item::MusicPage.switch(&e), Some(true));
        Item::Waypoints.cycle(&mut e, false);
        Item::Minimap.cycle(&mut e, false);
        Item::MinimapSize.cycle(&mut e, false);
        Item::MinimapFrameStyle.cycle(&mut e, false);
        Item::MinimapTerrain.cycle(&mut e, false);
        Item::MinimapEntityRadar.cycle(&mut e, false);
        Item::WaypointScale.cycle(&mut e, false);
        Item::WaypointMaxDistance.cycle(&mut e, false);
        assert!(e.set_key(WAYPOINT_KEY, "key.keyboard.n".into()));
        assert!(e.set_key(MINIMAP_ZOOM_KEY, "key.keyboard.m".into()));
        assert_eq!(Extras::parse(&e.serialize()), e);
    }

    #[test]
    fn bad_lines_keep_defaults() {
        let e = Extras::parse("zoom:maybe\nzoomLevel:99\nfpsCorner:middle\nzoomKey:z\nnoise");
        assert!(!e.zoom);
        assert_eq!(e.zoom_level, 10);
        assert_eq!(e.fps_corner, Corner::TopLeft);
        assert_eq!(e.zoom_key, "key.keyboard.z");
    }

    #[test]
    fn a_mods_zoom_eases_like_the_spyglass() {
        // Camera.tickFov halves the distance to the target each tick.
        let mut camera = crate::camera::Camera::default();
        camera.tick(None, 1.0);
        for _ in 0..30 {
            camera.tick(None, 1.0);
        }
        let target = fov_target(1.0, 4.0);
        camera.tick(None, target);
        let first = camera.calculate_fov(1.0, 70, 1.0, None, Default::default(), false);
        assert!((first - 70.0 * 0.625).abs() < 1e-3, "{first}");
        for _ in 0..30 {
            camera.tick(None, target);
        }
        let settled = camera.calculate_fov(1.0, 70, 1.0, None, Default::default(), false);
        assert!((settled - 70.0 / 4.0).abs() < 1e-3, "{settled}");
        // Deeper than the 0.1 floor stops there, as vanilla clamps.
        for _ in 0..30 {
            camera.tick(None, fov_target(1.0, 20.0));
        }
        let floor = camera.calculate_fov(1.0, 70, 1.0, None, Default::default(), false);
        assert!((floor - 7.0).abs() < 1e-3, "{floor}");
    }

    #[test]
    fn fps_counter_reports_rate_minimum_and_low() {
        let mut c = FpsCounter::default();
        let mut t = 0.0;
        // Two seconds at 100 fps with one 50 ms hitch.
        for i in 0..200 {
            let ms = if i == 150 { 50.0 } else { 10.0 };
            t += f64::from(ms) / 1000.0;
            c.record(t, ms);
        }
        c.computed_at = f64::NEG_INFINITY;
        c.record(t + 0.01, 10.0);
        let r = c.reading();
        assert!((95..=101).contains(&r.fps), "{r:?}");
        assert_eq!(r.min, 20);
        assert_eq!(r.low, 100, "one frame in 200 is under the 1% mark");
        // Hitches beyond 1% of frames pull the 1% low down.
        let mut c = FpsCounter::default();
        let mut t = 0.0;
        for i in 0..100 {
            let ms = if i % 25 == 0 { 40.0 } else { 10.0 };
            t += f64::from(ms) / 1000.0;
            c.record(t, ms);
        }
        c.computed_at = f64::NEG_INFINITY;
        c.record(t, 10.0);
        assert_eq!(c.reading().low, 25);
    }

    #[test]
    fn old_frames_leave_the_window() {
        let mut c = FpsCounter::default();
        c.record(0.0, 100.0);
        for i in 1..=700 {
            c.record(f64::from(i) / 100.0, 10.0);
        }
        assert_eq!(c.reading().min, 100, "the 100 ms frame is 7 s old");
    }

    #[test]
    fn cps_counts_the_last_second() {
        let mut cps = Cps::default();
        for i in 0..10 {
            cps.click(f64::from(i) * 0.1);
        }
        assert_eq!(cps.per_second(0.95), 10);
        assert_eq!(cps.per_second(1.25), 7);
        assert_eq!(cps.per_second(3.0), 0);
    }

    #[test]
    fn items_cycle_through_their_values() {
        let mut e = Extras::default();
        Item::Sort.cycle(&mut e, true);
        assert_eq!(Item::Sort.value(&e), "On");
        // The minimap's old keys still parse, for the move to its mod.
        assert_eq!(
            Extras::parse("minimapSize:7\nminimapZoom:3").minimap_zoom,
            1.0
        );
        for rows_fn in [rows, waypoint_rows, minimap_rows, volt_boost_rows] {
            for item in rows_fn().into_iter().flatten() {
                assert!(!item.caption().is_empty() && !item.description().is_empty());
            }
        }
    }
}
