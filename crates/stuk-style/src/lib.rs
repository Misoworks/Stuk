#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);
    pub const TEXT: Self = Self::rgb(0.92, 0.92, 0.88);
    pub const TEXT_MUTED: Self = Self::rgb(0.62, 0.64, 0.61);
    pub const ACCENT: Self = Self::rgb(0.42, 0.68, 0.55);
    pub const SURFACE: Self = Self::rgb(0.086, 0.09, 0.086);
    pub const SURFACE_ELEVATED: Self = Self::rgb(0.13, 0.135, 0.128);
    pub const WINDOW: Self = Self::rgb(0.055, 0.06, 0.056);
    pub const DANGER: Self = Self::rgb(0.9, 0.25, 0.3);

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb_u8(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(
            f32::from(r) / 255.0,
            f32::from(g) / 255.0,
            f32::from(b) / 255.0,
            1.0,
        )
    }

    pub fn opacity(self, alpha: f32) -> Self {
        Self {
            a: self.a * alpha.clamp(0.0, 1.0),
            ..self
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

impl ThemeMode {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "system" => Some(Self::System),
            "light" => Some(Self::Light),
            "dark" => Some(Self::Dark),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Density {
    Compact,
    Regular,
    Touch,
}

impl Density {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "compact" => Some(Self::Compact),
            "regular" => Some(Self::Regular),
            "touch" => Some(Self::Touch),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub mode: ThemeMode,
    pub density: Density,
    pub colors: ColorTokens,
    pub radius: RadiusTokens,
    pub spacing: SpacingTokens,
    pub font: FontTokens,
    pub animation: AnimationTokens,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            mode: ThemeMode::Dark,
            density: Density::Regular,
            colors: ColorTokens::dark(),
            radius: RadiusTokens::default(),
            spacing: SpacingTokens::default(),
            font: FontTokens::default(),
            animation: AnimationTokens::default(),
        }
    }

    pub fn light() -> Self {
        Self {
            mode: ThemeMode::Light,
            density: Density::Regular,
            colors: ColorTokens::light(),
            radius: RadiusTokens::default(),
            spacing: SpacingTokens::default(),
            font: FontTokens::default(),
            animation: AnimationTokens::default(),
        }
    }

    pub fn from_settings(theme: Option<&str>, density: Option<&str>) -> Self {
        let mode = theme
            .and_then(ThemeMode::parse)
            .unwrap_or(ThemeMode::System);
        let mut theme = match mode {
            ThemeMode::System | ThemeMode::Dark => Self::dark(),
            ThemeMode::Light => Self::light(),
        };
        theme.mode = mode;
        theme.density = density.and_then(Density::parse).unwrap_or(Density::Regular);
        theme
    }

    pub fn material_color(&self, material: &Material) -> Color {
        match material {
            Material::Solid(color) => *color,
            Material::Surface => self.colors.surface,
            Material::SurfaceElevated => self.colors.surface_elevated,
            Material::Window | Material::Maris => self.colors.window,
            Material::Sidebar => self.colors.sidebar,
            Material::Toolbar => self.colors.toolbar,
            Material::Popover | Material::Menu | Material::Dialog | Material::Luca => {
                self.colors.surface_elevated
            }
        }
    }

    pub fn resolve_color(&self, color: Color) -> Color {
        if color == Color::TEXT {
            self.colors.text
        } else if color == Color::TEXT_MUTED {
            self.colors.text_muted
        } else if color == Color::ACCENT {
            self.colors.accent
        } else if color == Color::SURFACE {
            self.colors.surface
        } else if color == Color::SURFACE_ELEVATED {
            self.colors.surface_elevated
        } else if color == Color::WINDOW {
            self.colors.window
        } else if color == Color::DANGER {
            self.colors.danger
        } else if color == Color::WHITE.opacity(0.08) {
            self.colors.outline
        } else {
            color
        }
    }

    pub fn button_fill(&self, variant: ButtonVariant) -> Color {
        match variant {
            ButtonVariant::Primary => self.colors.accent,
            ButtonVariant::Secondary => self.colors.control,
            ButtonVariant::Destructive => self.colors.danger,
            ButtonVariant::Ghost => Color::rgba(1.0, 1.0, 1.0, 0.0),
        }
    }

    pub fn button_text(&self, variant: ButtonVariant) -> Color {
        match variant {
            ButtonVariant::Primary | ButtonVariant::Destructive => self.colors.on_accent,
            ButtonVariant::Secondary | ButtonVariant::Ghost => self.colors.text,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorTokens {
    pub text: Color,
    pub text_muted: Color,
    pub accent: Color,
    pub on_accent: Color,
    pub surface: Color,
    pub surface_elevated: Color,
    pub window: Color,
    pub sidebar: Color,
    pub toolbar: Color,
    pub control: Color,
    pub outline: Color,
    pub danger: Color,
    pub warning: Color,
    pub success: Color,
}

impl ColorTokens {
    pub fn dark() -> Self {
        Self {
            text: Color::TEXT,
            text_muted: Color::TEXT_MUTED,
            accent: Color::ACCENT,
            on_accent: Color::rgb(0.035, 0.04, 0.035),
            surface: Color::SURFACE,
            surface_elevated: Color::SURFACE_ELEVATED,
            window: Color::WINDOW,
            sidebar: Color::rgb(0.075, 0.08, 0.076),
            toolbar: Color::rgb(0.1, 0.105, 0.098),
            control: Color::rgb(0.2, 0.21, 0.2),
            outline: Color::WHITE.opacity(0.08),
            danger: Color::DANGER,
            warning: Color::rgb(1.0, 0.8, 0.4),
            success: Color::rgb(0.27, 0.83, 0.51),
        }
    }

    pub fn light() -> Self {
        Self {
            text: Color::rgb(0.1, 0.105, 0.098),
            text_muted: Color::rgb(0.42, 0.43, 0.4),
            accent: Color::rgb(0.28, 0.53, 0.42),
            on_accent: Color::rgb(0.98, 0.98, 0.94),
            surface: Color::rgb(0.93, 0.925, 0.9),
            surface_elevated: Color::rgb(0.985, 0.98, 0.95),
            window: Color::rgb(0.96, 0.955, 0.93),
            sidebar: Color::rgb(0.89, 0.885, 0.86),
            toolbar: Color::rgb(0.92, 0.915, 0.89),
            control: Color::rgb(0.84, 0.835, 0.8),
            outline: Color::rgb(0.1, 0.105, 0.098).opacity(0.12),
            danger: Color::rgb(0.72, 0.16, 0.2),
            warning: Color::rgb(0.72, 0.48, 0.12),
            success: Color::rgb(0.16, 0.56, 0.32),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RadiusTokens {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub pill: f32,
}

impl Default for RadiusTokens {
    fn default() -> Self {
        Self {
            xs: 4.0,
            sm: 6.0,
            md: 10.0,
            lg: 16.0,
            xl: 22.0,
            pill: 999.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpacingTokens {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
}

impl Default for SpacingTokens {
    fn default() -> Self {
        Self {
            xs: 4.0,
            sm: 8.0,
            md: 12.0,
            lg: 16.0,
            xl: 24.0,
            xxl: 32.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FontTokens {
    pub family: String,
    pub mono_family: String,
    pub size: f32,
    pub small: f32,
    pub title: f32,
    pub large_title: f32,
}

impl Default for FontTokens {
    fn default() -> Self {
        Self {
            family: "System".to_string(),
            mono_family: "System Mono".to_string(),
            size: 14.0,
            small: 12.0,
            title: 20.0,
            large_title: 28.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationCurve {
    Linear,
    EaseOut,
    EmphasizedDecelerate,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimationTokens {
    pub fast_ms: u32,
    pub normal_ms: u32,
    pub slow_ms: u32,
    pub curve: AnimationCurve,
}

impl Default for AnimationTokens {
    fn default() -> Self {
        Self {
            fast_ms: 90,
            normal_ms: 160,
            slow_ms: 240,
            curve: AnimationCurve::EmphasizedDecelerate,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Material {
    Solid(Color),
    Surface,
    SurfaceElevated,
    Window,
    Sidebar,
    Toolbar,
    Popover,
    Menu,
    Dialog,
    Maris,
    Luca,
}

impl Material {
    pub fn fallback_color(&self) -> Color {
        Theme::dark().material_color(self)
    }

    pub fn fallback_color_for(&self, theme: &Theme) -> Color {
        theme.material_color(self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Destructive,
    Ghost,
}

impl ButtonVariant {
    pub fn fill(self) -> Color {
        Theme::dark().button_fill(self)
    }

    pub fn text(self) -> Color {
        Theme::dark().button_text(self)
    }

    pub fn fill_for(self, theme: &Theme) -> Color {
        theme.button_fill(self)
    }

    pub fn text_for(self, theme: &Theme) -> Color {
        theme.button_text(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_theme_from_settings() {
        let light = Theme::from_settings(Some("light"), Some("compact"));
        assert_eq!(light.mode, ThemeMode::Light);
        assert_eq!(light.density, Density::Compact);
        assert_eq!(light.resolve_color(Color::TEXT), light.colors.text);

        let fallback = Theme::from_settings(Some("missing"), Some("unknown"));
        assert_eq!(fallback.mode, ThemeMode::System);
        assert_eq!(fallback.density, Density::Regular);
    }

    #[test]
    fn resolves_materials_and_variants_from_tokens() {
        let theme = Theme::light();
        assert_eq!(
            Material::SurfaceElevated.fallback_color_for(&theme),
            theme.colors.surface_elevated
        );
        assert_eq!(ButtonVariant::Primary.fill_for(&theme), theme.colors.accent);
        assert_eq!(ButtonVariant::Secondary.text_for(&theme), theme.colors.text);
    }
}
