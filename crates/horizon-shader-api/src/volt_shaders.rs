//! Built-in mobile-first native WGSL "Volt Shaders" configuration (BSL +
//! Complementary class): the performance/quality profile presets, the
//! aesthetic style and every per-option value the settings UI edits and
//! `Extras` saves.
//!
//! This is the configuration half of Volt's shader module; the WGSL pack,
//! sprite classification and pipeline specialization constants are runtime
//! pieces built outside this settings path.

/// Built-in Volt Shaders performance/quality profiles, plus `Custom`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum VoltShaderProfile {
    Potato,
    Low,
    #[default]
    Medium,
    High,
    Ultra,
    Custom,
}

impl VoltShaderProfile {
    pub const ALL: [Self; 6] = [
        Self::Potato,
        Self::Low,
        Self::Medium,
        Self::High,
        Self::Ultra,
        Self::Custom,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Potato => "potato",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Ultra => "ultra",
            Self::Custom => "custom",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Potato => "Potato",
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Ultra => "Ultra",
            Self::Custom => "Custom",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.as_str() == s)
    }

    pub fn cycle(self, back: bool) -> Self {
        let idx = Self::ALL.iter().position(|&p| p == self).unwrap_or(2);
        let next = if back {
            (idx + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (idx + 1) % Self::ALL.len()
        };
        Self::ALL[next]
    }
}

/// Master aesthetic look inspired by BSL Shaders and Complementary Reimagined.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ShaderVisualStyle {
    #[default]
    BslSoftCinema,
    ComplementaryCrisp,
    VoltBalanced,
    PastelDream,
}

impl ShaderVisualStyle {
    pub const ALL: [Self; 4] = [
        Self::BslSoftCinema,
        Self::ComplementaryCrisp,
        Self::VoltBalanced,
        Self::PastelDream,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::BslSoftCinema => "bsl_soft",
            Self::ComplementaryCrisp => "complementary_crisp",
            Self::VoltBalanced => "volt_balanced",
            Self::PastelDream => "pastel_dream",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::BslSoftCinema => "BSL Soft",
            Self::ComplementaryCrisp => "Complementary",
            Self::VoltBalanced => "Volt Balanced",
            Self::PastelDream => "Pastel Dream",
        }
    }

    pub fn index(self) -> u32 {
        match self {
            Self::BslSoftCinema => 0,
            Self::ComplementaryCrisp => 1,
            Self::VoltBalanced => 2,
            Self::PastelDream => 3,
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

/// Procedural sky cloud rendering mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CloudMode {
    Off,
    #[default]
    BslVolumetric,
    ComplementaryVoxel,
    CirrusWisps,
}

impl CloudMode {
    pub const ALL: [Self; 4] = [
        Self::Off,
        Self::BslVolumetric,
        Self::ComplementaryVoxel,
        Self::CirrusWisps,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::BslVolumetric => "bsl_volumetric",
            Self::ComplementaryVoxel => "complementary_voxel",
            Self::CirrusWisps => "cirrus_wisps",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::BslVolumetric => "BSL Puffy 3D",
            Self::ComplementaryVoxel => "Voxel Reimagined",
            Self::CirrusWisps => "Soft Cirrus",
        }
    }

    pub fn index(self) -> u32 {
        match self {
            Self::Off => 0,
            Self::BslVolumetric => 1,
            Self::ComplementaryVoxel => 2,
            Self::CirrusWisps => 3,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.as_str() == s)
    }

    pub fn cycle(self, back: bool) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(1);
        Self::ALL[if back {
            (idx + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (idx + 1) % Self::ALL.len()
        }]
    }
}

/// Block-light color temperature style for torches, lanterns, lava & glowstone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TorchLightStyle {
    Vanilla,
    #[default]
    WarmAmber,
    CozyGold,
    SoulCool,
}

impl TorchLightStyle {
    pub const ALL: [Self; 4] = [
        Self::Vanilla,
        Self::WarmAmber,
        Self::CozyGold,
        Self::SoulCool,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vanilla => "vanilla",
            Self::WarmAmber => "warm_amber",
            Self::CozyGold => "cozy_gold",
            Self::SoulCool => "soul_cool",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Vanilla => "Vanilla",
            Self::WarmAmber => "Warm Amber",
            Self::CozyGold => "Cozy Gold",
            Self::SoulCool => "Soul Cool",
        }
    }

    pub fn index(self) -> u32 {
        match self {
            Self::Vanilla => 0,
            Self::WarmAmber => 1,
            Self::CozyGold => 2,
            Self::SoulCool => 3,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.as_str() == s)
    }

    pub fn cycle(self, back: bool) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(1);
        Self::ALL[if back {
            (idx + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (idx + 1) % Self::ALL.len()
        }]
    }
}

/// Procedural night-sky Aurora Borealis intensity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AuroraMode {
    Off,
    #[default]
    Subtle,
    Medium,
    Vibrant,
}

impl AuroraMode {
    pub const ALL: [Self; 4] = [Self::Off, Self::Subtle, Self::Medium, Self::Vibrant];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Subtle => "subtle",
            Self::Medium => "medium",
            Self::Vibrant => "vibrant",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::Subtle => "Subtle",
            Self::Medium => "Medium",
            Self::Vibrant => "Vibrant",
        }
    }

    pub fn index(self) -> u32 {
        match self {
            Self::Off => 0,
            Self::Subtle => 1,
            Self::Medium => 2,
            Self::Vibrant => 3,
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

/// Distance depth haze mode (uses `scene_depth` in the post stage).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DepthHazeMode {
    #[default]
    Off,
    Subtle,
    Lush,
}

impl DepthHazeMode {
    pub const ALL: [Self; 3] = [Self::Off, Self::Subtle, Self::Lush];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Subtle => "subtle",
            Self::Lush => "lush",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::Subtle => "Subtle",
            Self::Lush => "Lush",
        }
    }

    pub fn index(self) -> u32 {
        match self {
            Self::Off => 0,
            Self::Subtle => 1,
            Self::Lush => 2,
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

/// Filmic tonemapping curve.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TonemapMode {
    Off,
    #[default]
    FilmicAces,
    CrispNeutral,
    LushCinema,
}

impl TonemapMode {
    pub const ALL: [Self; 4] = [
        Self::Off,
        Self::FilmicAces,
        Self::CrispNeutral,
        Self::LushCinema,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::FilmicAces => "filmic_aces",
            Self::CrispNeutral => "crisp_neutral",
            Self::LushCinema => "lush_cinema",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::FilmicAces => "Filmic ACES",
            Self::CrispNeutral => "Crisp Neutral",
            Self::LushCinema => "Lush Cinema",
        }
    }

    pub fn index(self) -> u32 {
        match self {
            Self::Off => 0,
            Self::FilmicAces => 1,
            Self::CrispNeutral => 2,
            Self::LushCinema => 3,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.as_str() == s)
    }

    pub fn cycle(self, back: bool) -> Self {
        let idx = Self::ALL.iter().position(|&v| v == self).unwrap_or(1);
        Self::ALL[if back {
            (idx + Self::ALL.len() - 1) % Self::ALL.len()
        } else {
            (idx + 1) % Self::ALL.len()
        }]
    }
}

/// Single-pass hardware-bilinear highlight bloom quality.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BloomMode {
    #[default]
    Off,
    Low,
    High,
}

impl BloomMode {
    pub const ALL: [Self; 3] = [Self::Off, Self::Low, Self::High];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Low => "low",
            Self::High => "high",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::Low => "Low (Fast)",
            Self::High => "High (Smooth)",
        }
    }

    pub fn index(self) -> u32 {
        match self {
            Self::Off => 0,
            Self::Low => 1,
            Self::High => 2,
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

/// Screen-edge cinematic vignette intensity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum VignetteMode {
    #[default]
    Off,
    Subtle,
    Strong,
}

impl VignetteMode {
    pub const ALL: [Self; 3] = [Self::Off, Self::Subtle, Self::Strong];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Subtle => "subtle",
            Self::Strong => "strong",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::Subtle => "Subtle",
            Self::Strong => "Strong",
        }
    }

    pub fn index(self) -> u32 {
        match self {
            Self::Off => 0,
            Self::Subtle => 1,
            Self::Strong => 2,
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

/// Full configuration for Built-In Volt Shaders (`enabled` is `false` by default).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VoltShadersConfig {
    /// Master toggle (OFF by default). When false, all customization options are hidden.
    pub enabled: bool,
    pub profile: VoltShaderProfile,
    /// Overall BSL / Complementary aesthetic palette & shading curve.
    pub style: ShaderVisualStyle,
    // --- Wind & Moving Vegetation ---
    pub waving_plants: bool,
    pub waving_leaves: bool,
    pub waving_water: bool,
    /// Percentage `25..=200` (100 = 1.0x).
    pub wind_speed: u16,
    /// Percentage `25..=200` (100 = 1.0x).
    pub wind_strength: u16,
    pub storm_wind: bool,
    // --- Lighting, Integrated Auto-PBR & Shadows ---
    pub sun_shading: bool,
    /// Percentage `0..=100`.
    pub shadow_richness: u16,
    pub cloud_shadows: bool,
    pub foliage_subsurface: bool,
    pub glowing_ores: bool,
    /// Percentage `25..=200` (100 = 1.0x).
    pub ore_glow: u16,
    pub auto_pbr: bool,
    pub torch_style: TorchLightStyle,
    /// Percentage `50..=200` (100 = 1.0x).
    pub torch_intensity: u16,
    /// Percentage `0..=200` (100 = 1.0x).
    pub emissive_glow: u16,
    pub rain_wetness: bool,
    // --- Water ---
    pub water_ripples: bool,
    pub water_foam: bool,
    /// Percentage `0..=150` (100 = 1.0x).
    pub water_reflection: u16,
    pub water_glint: bool,
    pub clear_water: bool,
    pub underwater_refraction: bool,
    // --- Sky, Volumetric/Voxel Clouds & Atmosphere ---
    pub custom_sky: bool,
    pub clouds_mode: CloudMode,
    /// Percentage `30..=160` (100 = balanced cloud density).
    pub cloud_coverage: u16,
    /// Percentage `0..=200` (100 = 1.0x).
    pub sunset_glow: u16,
    pub sun_halo: bool,
    pub aurora: AuroraMode,
    pub night_sky_plus: bool,
    pub post_rain_rainbow: bool,
    pub depth_haze: DepthHazeMode,
    // --- Eye-Comfort, Post-Processing & Color Grading ---
    pub eye_comfort: bool,
    pub tonemap: TonemapMode,
    pub bloom: BloomMode,
    pub sun_rays: bool,
    /// Percentage `50..=180` (100 = neutral).
    pub vibrance: u16,
    /// Percentage `50..=180` (100 = neutral).
    pub saturation: u16,
    /// Percentage `70..=150` (100 = neutral).
    pub contrast: u16,
    /// Percentage `70..=150` (100 = neutral).
    pub exposure: u16,
    /// Signed temperature `-50..=50` (0 = neutral, positive = warm, negative = cool).
    pub warmth: i16,
    pub vignette: VignetteMode,
}

impl Default for VoltShadersConfig {
    fn default() -> Self {
        let mut cfg = Self::preset(VoltShaderProfile::Medium);
        cfg.enabled = false;
        cfg
    }
}

impl VoltShadersConfig {
    /// Returns a configuration populated with `profile`'s performance-tuned values.
    pub fn preset(profile: VoltShaderProfile) -> Self {
        match profile {
            VoltShaderProfile::Potato => Self {
                enabled: true,
                profile: VoltShaderProfile::Potato,
                style: ShaderVisualStyle::BslSoftCinema,
                waving_plants: true,
                waving_leaves: false,
                waving_water: false,
                wind_speed: 100,
                wind_strength: 85,
                storm_wind: false,
                sun_shading: true,
                shadow_richness: 30,
                cloud_shadows: false,
                foliage_subsurface: false,
                glowing_ores: true,
                ore_glow: 85,
                auto_pbr: false,
                torch_style: TorchLightStyle::WarmAmber,
                torch_intensity: 108,
                emissive_glow: 40,
                rain_wetness: false,
                water_ripples: false,
                water_foam: false,
                water_reflection: 30,
                water_glint: false,
                clear_water: true,
                underwater_refraction: false,
                custom_sky: true,
                clouds_mode: CloudMode::CirrusWisps,
                cloud_coverage: 85,
                sunset_glow: 75,
                sun_halo: false,
                aurora: AuroraMode::Off,
                night_sky_plus: false,
                post_rain_rainbow: false,
                depth_haze: DepthHazeMode::Off,
                eye_comfort: true,
                tonemap: TonemapMode::CrispNeutral,
                bloom: BloomMode::Off,
                sun_rays: false,
                vibrance: 112,
                saturation: 101,
                contrast: 102,
                exposure: 101,
                warmth: 2,
                vignette: VignetteMode::Off,
            },
            VoltShaderProfile::Low => Self {
                enabled: true,
                profile: VoltShaderProfile::Low,
                style: ShaderVisualStyle::BslSoftCinema,
                waving_plants: true,
                waving_leaves: true,
                waving_water: false,
                wind_speed: 100,
                wind_strength: 95,
                storm_wind: true,
                sun_shading: true,
                shadow_richness: 45,
                cloud_shadows: false,
                foliage_subsurface: true,
                glowing_ores: true,
                ore_glow: 95,
                auto_pbr: false,
                torch_style: TorchLightStyle::WarmAmber,
                torch_intensity: 118,
                emissive_glow: 65,
                rain_wetness: false,
                water_ripples: true,
                water_foam: true,
                water_reflection: 60,
                water_glint: true,
                clear_water: true,
                underwater_refraction: false,
                custom_sky: true,
                clouds_mode: CloudMode::BslVolumetric,
                cloud_coverage: 95,
                sunset_glow: 95,
                sun_halo: true,
                aurora: AuroraMode::Off,
                night_sky_plus: true,
                post_rain_rainbow: false,
                depth_haze: DepthHazeMode::Off,
                eye_comfort: true,
                tonemap: TonemapMode::FilmicAces,
                bloom: BloomMode::Off,
                sun_rays: false,
                vibrance: 116,
                saturation: 103,
                contrast: 103,
                exposure: 101,
                warmth: 4,
                vignette: VignetteMode::Off,
            },
            VoltShaderProfile::Medium | VoltShaderProfile::Custom => Self {
                enabled: true,
                profile,
                style: ShaderVisualStyle::BslSoftCinema,
                waving_plants: true,
                waving_leaves: true,
                waving_water: true,
                wind_speed: 100,
                wind_strength: 100,
                storm_wind: true,
                sun_shading: true,
                shadow_richness: 58,
                cloud_shadows: true,
                foliage_subsurface: true,
                glowing_ores: true,
                ore_glow: 110,
                auto_pbr: true,
                torch_style: TorchLightStyle::WarmAmber,
                torch_intensity: 126,
                emissive_glow: 90,
                rain_wetness: true,
                water_ripples: true,
                water_foam: true,
                water_reflection: 85,
                water_glint: true,
                clear_water: true,
                underwater_refraction: false,
                custom_sky: true,
                clouds_mode: CloudMode::BslVolumetric,
                cloud_coverage: 100,
                sunset_glow: 115,
                sun_halo: true,
                aurora: AuroraMode::Subtle,
                night_sky_plus: true,
                post_rain_rainbow: true,
                depth_haze: DepthHazeMode::Off,
                eye_comfort: true,
                tonemap: TonemapMode::FilmicAces,
                bloom: BloomMode::Off,
                sun_rays: false,
                vibrance: 120,
                saturation: 104,
                contrast: 104,
                exposure: 102,
                warmth: 5,
                vignette: VignetteMode::Off,
            },
            VoltShaderProfile::High => Self {
                enabled: true,
                profile: VoltShaderProfile::High,
                style: ShaderVisualStyle::ComplementaryCrisp,
                waving_plants: true,
                waving_leaves: true,
                waving_water: true,
                wind_speed: 100,
                wind_strength: 105,
                storm_wind: true,
                sun_shading: true,
                shadow_richness: 68,
                cloud_shadows: true,
                foliage_subsurface: true,
                glowing_ores: true,
                ore_glow: 125,
                auto_pbr: true,
                torch_style: TorchLightStyle::WarmAmber,
                torch_intensity: 132,
                emissive_glow: 105,
                rain_wetness: true,
                water_ripples: true,
                water_foam: true,
                water_reflection: 100,
                water_glint: true,
                clear_water: true,
                underwater_refraction: true,
                custom_sky: true,
                clouds_mode: CloudMode::ComplementaryVoxel,
                cloud_coverage: 105,
                sunset_glow: 130,
                sun_halo: true,
                aurora: AuroraMode::Medium,
                night_sky_plus: true,
                post_rain_rainbow: true,
                depth_haze: DepthHazeMode::Subtle,
                eye_comfort: true,
                tonemap: TonemapMode::FilmicAces,
                bloom: BloomMode::Low,
                sun_rays: false,
                vibrance: 124,
                saturation: 105,
                contrast: 105,
                exposure: 103,
                warmth: 6,
                vignette: VignetteMode::Off,
            },
            VoltShaderProfile::Ultra => Self {
                enabled: true,
                profile: VoltShaderProfile::Ultra,
                style: ShaderVisualStyle::BslSoftCinema,
                waving_plants: true,
                waving_leaves: true,
                waving_water: true,
                wind_speed: 100,
                wind_strength: 110,
                storm_wind: true,
                sun_shading: true,
                shadow_richness: 76,
                cloud_shadows: true,
                foliage_subsurface: true,
                glowing_ores: true,
                ore_glow: 135,
                auto_pbr: true,
                torch_style: TorchLightStyle::CozyGold,
                torch_intensity: 140,
                emissive_glow: 120,
                rain_wetness: true,
                water_ripples: true,
                water_foam: true,
                water_reflection: 118,
                water_glint: true,
                clear_water: true,
                underwater_refraction: true,
                custom_sky: true,
                clouds_mode: CloudMode::BslVolumetric,
                cloud_coverage: 110,
                sunset_glow: 145,
                sun_halo: true,
                aurora: AuroraMode::Vibrant,
                night_sky_plus: true,
                post_rain_rainbow: true,
                depth_haze: DepthHazeMode::Lush,
                eye_comfort: true,
                tonemap: TonemapMode::LushCinema,
                bloom: BloomMode::High,
                sun_rays: true,
                vibrance: 126,
                saturation: 106,
                contrast: 106,
                exposure: 104,
                warmth: 7,
                vignette: VignetteMode::Subtle,
            },
        }
    }

    /// Applies `profile` while preserving `self.enabled`. Selecting `Custom`
    /// keeps the current option values untouched so they can be tweaked freely.
    pub fn apply_profile(&mut self, profile: VoltShaderProfile) {
        if profile == VoltShaderProfile::Custom {
            self.profile = VoltShaderProfile::Custom;
            return;
        }
        let enabled = self.enabled;
        *self = Self::preset(profile);
        self.enabled = enabled;
    }

    /// Resets options to the active preset (or `Medium` when on `Custom`).
    pub fn reset_current_profile(&mut self) {
        let target = if self.profile == VoltShaderProfile::Custom {
            VoltShaderProfile::Medium
        } else {
            self.profile
        };
        let enabled = self.enabled;
        *self = Self::preset(target);
        self.enabled = enabled;
    }

    /// Marks the active profile as `Custom` whenever an individual option is edited.
    pub fn mark_custom(&mut self) {
        self.profile = VoltShaderProfile::Custom;
    }
}
