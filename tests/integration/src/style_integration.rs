#[allow(unused_imports)]
use stuk_style::{Color, Density, Material, Theme, ThemeMode};

#[test]
fn dark_theme_has_expected_tokens() {
    let theme = Theme::dark();
    assert_eq!(theme.mode, ThemeMode::Dark);
    assert_eq!(theme.colors.text, Color::rgb(0.96, 0.96, 0.96));
}

#[test]
fn light_theme_has_expected_tokens() {
    let theme = Theme::light();
    assert_eq!(theme.mode, ThemeMode::Light);
}

#[test]
fn theme_from_settings() {
    let dark = Theme::from_settings(Some("dark"), None);
    assert_eq!(dark.mode, ThemeMode::Dark);
    let light = Theme::from_settings(Some("light"), None);
    assert_eq!(light.mode, ThemeMode::Light);
    let system = Theme::from_settings(None, None);
    assert_eq!(system.mode, ThemeMode::System);
}

#[test]
fn material_enum_covers_all_spec_variants() {
    let variants = [
        Material::Surface,
        Material::SurfaceElevated,
        Material::Window,
        Material::Sidebar,
        Material::Toolbar,
        Material::Popover,
        Material::Menu,
        Material::Dialog,
        Material::Maris,
        Material::Luca,
    ];
    assert_eq!(variants.len(), 10);
}

#[test]
fn color_opacity() {
    let color = Color::TEXT.opacity(0.5);
    assert!((color.a - 0.5).abs() < 0.01);
}

#[test]
fn theme_resolves_material_colors() {
    let theme = Theme::dark();
    let maris = theme.material_color(&Material::Maris);
    assert_ne!(maris.r, 0.0);
}

#[test]
fn density_parsing() {
    assert_eq!(Density::parse("compact"), Some(Density::Compact));
    assert_eq!(Density::parse("regular"), Some(Density::Regular));
    assert_eq!(Density::parse("touch"), Some(Density::Touch));
    assert_eq!(Density::parse("invalid"), None);
}

#[test]
fn color_rgb_u8() {
    let color = Color::rgb_u8(255, 0, 0);
    assert!((color.r - 1.0).abs() < 0.01);
    assert!(color.g.abs() < 0.01);
}
