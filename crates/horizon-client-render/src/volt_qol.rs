//! Volt QoL, Volt Visuals, Volt Boost+, Volt Chat and Volt Mods configuration
//! data structures (the ported config half of Volt Launcher's `volt_qol`
//! module): `#2` Zoom & Freelook, `#3` World Inspector & Spawn-Light Overlay,
//! `#8` Volt Blueprint, `#9` Volt Cam & Replay Ring, `#15` Sound Physics Lite
//! & Muffler, `#17` Interactive World Atlas, `#18` Volt JEI Recipe Browser,
//! `#19` Seamless Connected Glass/Textures, Better Grass Sides, 3D Dropped
//! Items, `#21` 3D Positional Proximity Voice Chat, `#25` First-Person Hand &
//! Viewmodel Customizer and the 24 built-in Volt Mods.
//!
//! Only the configuration types live here; the HUD/screen engines are ported
//! separately.

// ============================================================================
// 1. VOLT QOL CONFIG (`Page::VoltQol`)
// ============================================================================

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BlueprintPreset {
    #[default]
    StarterCabin,
    Watchtower,
    CropFarm9x9,
    PortalShrine,
    MobFarmDeck,
    WorldClipboard,
}

impl BlueprintPreset {
    pub const ALL: [Self; 6] = [
        Self::StarterCabin,
        Self::Watchtower,
        Self::CropFarm9x9,
        Self::PortalShrine,
        Self::MobFarmDeck,
        Self::WorldClipboard,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::StarterCabin => "starter_cabin",
            Self::Watchtower => "watchtower",
            Self::CropFarm9x9 => "crop_farm",
            Self::PortalShrine => "portal_shrine",
            Self::MobFarmDeck => "mob_farm",
            Self::WorldClipboard => "clipboard",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::StarterCabin => "Cozy Oak Cabin (7x5x7)",
            Self::Watchtower => "Stone Watchtower (5x9x5)",
            Self::CropFarm9x9 => "Irrigated 9x9 Farm",
            Self::PortalShrine => "Obsidian Portal Frame",
            Self::MobFarmDeck => "Spawning Pad (9x4x9)",
            Self::WorldClipboard => "Copied World Region",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.as_str() == s)
    }

    pub fn cycle(self, back: bool) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0);
        Self::ALL[if back {
            (idx + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (idx + 1) % Self::ALL.len()
        }]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AtlasViewMode {
    #[default]
    Surface,
    HeightRelief,
    BiomeTint,
    SlimeChunks,
}

impl AtlasViewMode {
    pub const ALL: [Self; 4] = [
        Self::Surface,
        Self::HeightRelief,
        Self::BiomeTint,
        Self::SlimeChunks,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Surface => "surface",
            Self::HeightRelief => "relief",
            Self::BiomeTint => "biome",
            Self::SlimeChunks => "slime",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Surface => "Surface + Relief",
            Self::HeightRelief => "Elevation Contour",
            Self::BiomeTint => "Lush Biome Map",
            Self::SlimeChunks => "Slime Chunk Radar",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.as_str() == s)
    }

    pub fn cycle(self, back: bool) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0);
        Self::ALL[if back {
            (idx + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (idx + 1) % Self::ALL.len()
        }]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VoltQolConfig {
    // #2 Zoom & 360 Freelook
    pub zoom_enabled: bool,
    pub zoom_fov_pct: u16,
    pub zoom_smooth: bool,
    pub freelook_enabled: bool,
    // #3 World Inspector & Mob-Spawn Light-0 Overlay
    pub world_inspector: bool,
    pub spawn_light_overlay: bool,
    // #8 Volt Blueprint
    pub blueprint_enabled: bool,
    pub blueprint_preset: BlueprintPreset,
    pub blueprint_layer: i16,
    pub blueprint_checklist: bool,
    // #9 Volt Cam & Replay Ring
    pub photo_mode_overlays: bool,
    pub cam_roll_deg: i16,
    pub cam_letterbox: bool,
    pub replay_buffer: bool,
    /// The draggable on-screen replay button (Start / Pause / End).
    pub replay_button: bool,
    /// The button's centre, in percent of the screen.
    pub replay_btn_x_pct: u16,
    pub replay_btn_y_pct: u16,
    /// The button's edge in HUD units (hold the button and drag up/down).
    pub replay_btn_size: u16,
    /// The button's opacity in percent (hold and drag left/right).
    pub replay_btn_opacity_pct: u16,
    // #15 Sound Physics Lite & Muffler
    pub sound_cave_reverb: bool,
    pub sound_underwater_lpf: bool,
    pub sound_wall_occlusion: bool,
    pub muffle_portal_pct: u16,
    pub muffle_piston_pct: u16,
    pub muffle_minecart_pct: u16,
    pub muffle_explosion_pct: u16,
    pub muffle_rain_pct: u16,
    // #17 World Atlas
    pub atlas_enabled: bool,
    pub atlas_mode: AtlasViewMode,
    pub atlas_grid: bool,
    // #25 First-Person Hand & Viewmodel Customizer
    pub viewmodel_custom: bool,
    pub vm_offset_x: i16,
    pub vm_offset_y: i16,
    pub vm_offset_z: i16,
    pub vm_scale_pct: u16,
    pub vm_roll_deg: i16,
    pub vm_swing_speed_pct: u16,
    pub vm_classic_animations: bool,
    pub vm_totem_scale_pct: u16,
}

impl Default for VoltQolConfig {
    fn default() -> Self {
        Self {
            zoom_enabled: false,
            zoom_fov_pct: 28,
            zoom_smooth: true,
            freelook_enabled: false,
            world_inspector: false,
            spawn_light_overlay: false,
            blueprint_enabled: false,
            blueprint_preset: BlueprintPreset::StarterCabin,
            blueprint_layer: -1,
            blueprint_checklist: true,
            photo_mode_overlays: false,
            cam_roll_deg: 0,
            cam_letterbox: false,
            replay_buffer: false,
            replay_button: false,
            replay_btn_x_pct: 7,
            replay_btn_y_pct: 38,
            replay_btn_size: 34,
            replay_btn_opacity_pct: 85,
            sound_cave_reverb: false,
            sound_underwater_lpf: true,
            sound_wall_occlusion: false,
            muffle_portal_pct: 100,
            muffle_piston_pct: 100,
            muffle_minecart_pct: 100,
            muffle_explosion_pct: 100,
            muffle_rain_pct: 100,
            atlas_enabled: true,
            atlas_mode: AtlasViewMode::Surface,
            atlas_grid: true,
            viewmodel_custom: false,
            vm_offset_x: 0,
            vm_offset_y: 0,
            vm_offset_z: 0,
            vm_scale_pct: 100,
            vm_roll_deg: 0,
            vm_swing_speed_pct: 100,
            vm_classic_animations: false,
            vm_totem_scale_pct: 100,
        }
    }
}

// ============================================================================
// 2. VOLT VISUALS & VOLT BOOST+ CONFIGS (`#19` & `#6`)
// ============================================================================

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct VoltVisualsConfig {
    /// `#19` Seamless Connected Glass, Stained Glass & Panes (`mesher.rs` UV border trim).
    pub connected_glass: bool,
    /// `#19` Lush full-green Grass Block sides when adjacent to grass/foliage.
    pub better_grass_sides: bool,
    /// `#19` 3D Physics-resting / tumbling dropped item presentation.
    pub dropped_items_3d: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct VoltBoostPlusConfig {
    /// `#6` Coalesces adjacent identical coplanar opaque block faces in `mesher.rs`.
    pub greedy_quad_merge: bool,
    /// `#6` Culls sealed underground/indoor chunk sections occluded from camera.
    pub cave_chunk_occlusion: bool,
    /// `#6` Recycles chunk mesh allocations to eliminate GC/allocator frame spikes.
    pub zero_stutter_mesh_pool: bool,
    /// `#6` Skips distant block-entity animations & off-frustum entity pose updates.
    pub offscreen_entity_skip: bool,
    /// `#6` Smooths mobile frame pacing to reduce thermal spikes on Mali/Adreno GPUs.
    pub smart_frame_pacer: bool,
}

// ============================================================================
// 3. VOLT CHAT & 3D PROXIMITY VOICE CONFIG + UDP ENGINE (`#21`)
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VoltChatConfig {
    pub voice_enabled: bool,
    pub voice_ptt_mode: bool,
    pub voice_vad_threshold: u16,
    pub voice_mic_gain_pct: u16,
    pub voice_output_gain_pct: u16,
    pub voice_distance_blocks: u16,
    pub voice_cave_muffle: bool,
    pub voice_hud_button: bool,
    pub voice_loopback_test: bool,
    pub voice_port: u16,
    // Chat enhancements
    pub chat_timestamps: bool,
    pub chat_compact_spam: bool,
    pub chat_mention_ping: bool,
    pub chat_quick_bar: bool,
}

impl Default for VoltChatConfig {
    fn default() -> Self {
        Self {
            voice_enabled: false,
            voice_ptt_mode: true,
            voice_vad_threshold: 22,
            voice_mic_gain_pct: 100,
            voice_output_gain_pct: 100,
            voice_distance_blocks: 48,
            voice_cave_muffle: true,
            voice_hud_button: true,
            voice_loopback_test: false,
            voice_port: 24454,
            chat_timestamps: false,
            chat_compact_spam: false,
            chat_mention_ping: true,
            chat_quick_bar: false,
        }
    }
}

// ============================================================================
// 4. VOLT MODS CONFIG (24 Built-In Pure-Rust Native Mods, all OFF by default)
// ============================================================================

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CrosshairStyle {
    #[default]
    Vanilla,
    PrecisionDot,
    CrispPlus,
    CircleRing,
    PvpChevron,
}

impl CrosshairStyle {
    pub const ALL: [Self; 5] = [
        Self::Vanilla,
        Self::PrecisionDot,
        Self::CrispPlus,
        Self::CircleRing,
        Self::PvpChevron,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vanilla => "vanilla",
            Self::PrecisionDot => "dot",
            Self::CrispPlus => "crisp_plus",
            Self::CircleRing => "circle",
            Self::PvpChevron => "chevron",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Vanilla => "Vanilla",
            Self::PrecisionDot => "Precision Dot",
            Self::CrispPlus => "Crisp Plus",
            Self::CircleRing => "Tactical Ring",
            Self::PvpChevron => "PvP Chevron",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.as_str() == s)
    }

    pub fn cycle(self, back: bool) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0);
        Self::ALL[if back {
            (idx + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (idx + 1) % Self::ALL.len()
        }]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ClientTimeWeather {
    #[default]
    ServerLive,
    SunnyNoon,
    GoldenSunset,
    StarryMidnight,
    CozyRain,
}

impl ClientTimeWeather {
    pub const ALL: [Self; 5] = [
        Self::ServerLive,
        Self::SunnyNoon,
        Self::GoldenSunset,
        Self::StarryMidnight,
        Self::CozyRain,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ServerLive => "live",
            Self::SunnyNoon => "noon",
            Self::GoldenSunset => "sunset",
            Self::StarryMidnight => "midnight",
            Self::CozyRain => "rain",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::ServerLive => "Server Live",
            Self::SunnyNoon => "Sunny Noon",
            Self::GoldenSunset => "Golden Sunset",
            Self::StarryMidnight => "Starry Night",
            Self::CozyRain => "Cozy Rain",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.as_str() == s)
    }

    pub fn cycle(self, back: bool) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(0);
        Self::ALL[if back {
            (idx + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (idx + 1) % Self::ALL.len()
        }]
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct VoltModsConfig {
    // Combat & PvP HUDs
    pub armor_hud: bool,
    pub potion_hud: bool,
    pub cps_hud: bool,
    pub reach_hud: bool,
    pub fps_ping_pill: bool,
    pub keystrokes_hud: bool,
    pub custom_crosshair: CrosshairStyle,
    pub low_shield_fire: bool,
    pub no_hurtcam: bool,
    pub toggle_sprint: bool,
    // Survival & Inventory
    pub appleskin_hud: bool,
    pub shulker_preview: bool,
    pub tool_break_shield: bool,
    pub auto_tool_hint: bool,
    // World & Combat Visuals
    pub trajectory_arc: bool,
    pub damage_popups: bool,
    pub hitbox_plus: bool,
    pub damage_direction_hud: bool,
    // Utility, Technical & Atmosphere
    pub chat_calculator: bool,
    pub enchant_guide: bool,
    pub slime_tech_hud: bool,
    pub death_tracker: bool,
    pub time_weather: ClientTimeWeather,
    pub oled_afk_saver: bool,
}

impl VoltModsConfig {
    pub fn active_count(&self) -> usize {
        let bools = [
            self.armor_hud,
            self.potion_hud,
            self.cps_hud,
            self.reach_hud,
            self.fps_ping_pill,
            self.keystrokes_hud,
            self.custom_crosshair != CrosshairStyle::Vanilla,
            self.low_shield_fire,
            self.no_hurtcam,
            self.toggle_sprint,
            self.appleskin_hud,
            self.shulker_preview,
            self.tool_break_shield,
            self.auto_tool_hint,
            self.trajectory_arc,
            self.damage_popups,
            self.hitbox_plus,
            self.damage_direction_hud,
            self.chat_calculator,
            self.enchant_guide,
            self.slime_tech_hud,
            self.death_tracker,
            self.time_weather != ClientTimeWeather::ServerLive,
            self.oled_afk_saver,
        ];
        bools.into_iter().filter(|&v| v).count()
    }
}
