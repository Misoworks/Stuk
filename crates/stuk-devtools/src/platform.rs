use stuk_platform::{
    BackendDescriptor, MaterialEffect, MaterialResolver, Platform, PlatformCapabilities,
};
use stuk_style::{Color, Material, Theme};

#[derive(Clone, Debug, PartialEq)]
pub struct PlatformInspection {
    pub backend: BackendInspection,
    pub capabilities: Vec<CapabilityInspection>,
    pub materials: Vec<MaterialInspection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackendInspection {
    pub name: String,
    pub kind: String,
    pub family: String,
    pub os: String,
    pub status: String,
    pub limitations: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityInspection {
    pub name: String,
    pub supported: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MaterialInspection {
    pub material: String,
    pub effect: String,
    pub detail: Option<String>,
    pub requires_compositor: bool,
    pub fallback: Color,
}

pub fn inspect_platform<P>(
    platform: &P,
    theme: &Theme,
    materials: impl IntoIterator<Item = Material>,
) -> PlatformInspection
where
    P: Platform + MaterialResolver,
{
    PlatformInspection {
        backend: inspect_backend(platform.backend()),
        capabilities: inspect_capabilities(platform.platform_capabilities()),
        materials: materials
            .into_iter()
            .map(|material| inspect_material(platform, theme, material))
            .collect(),
    }
}

fn inspect_backend(backend: BackendDescriptor) -> BackendInspection {
    BackendInspection {
        name: backend.name,
        kind: backend.kind.as_str().to_string(),
        family: backend.target.family.as_str().to_string(),
        os: backend.target.os.as_str().to_string(),
        status: backend.status.as_str().to_string(),
        limitations: backend.limitations,
    }
}

fn inspect_capabilities(capabilities: PlatformCapabilities) -> Vec<CapabilityInspection> {
    vec![
        capability("native_windows", capabilities.native_windows),
        capability("web_surface", capabilities.web_surface),
        capability("mobile_shell", capabilities.mobile_shell),
        capability("native_bridge", capabilities.native_bridge),
        capability("live_blur", capabilities.live_blur),
        capability("transparent_windows", capabilities.transparent_windows),
        capability("wallpaper_material", capabilities.wallpaper_material),
        capability("touch_input", capabilities.touch_input),
        capability("pointer_input", capabilities.pointer_input),
        capability("keyboard_input", capabilities.keyboard_input),
        capability("file_dialogs", capabilities.file_dialogs),
        capability("shell_tabs", capabilities.shell_tabs),
        capability("command_palette", capabilities.command_palette),
        capability("workspace_sessions", capabilities.workspace_sessions),
        capability("native_notifications", capabilities.native_notifications),
        capability("system_dark_mode", capabilities.system_dark_mode),
        capability("high_contrast", capabilities.high_contrast),
    ]
}

fn capability(name: &str, supported: bool) -> CapabilityInspection {
    CapabilityInspection {
        name: name.to_string(),
        supported,
    }
}

fn inspect_material<P>(platform: &P, theme: &Theme, material: Material) -> MaterialInspection
where
    P: MaterialResolver,
{
    let resolution = platform.resolve_material(&material, theme);
    let (effect, detail) = material_effect(&resolution.effect);
    MaterialInspection {
        material: material_name(&material).to_string(),
        effect: effect.to_string(),
        detail,
        requires_compositor: resolution.requires_compositor(),
        fallback: resolution.fallback,
    }
}

fn material_effect(effect: &MaterialEffect) -> (&'static str, Option<String>) {
    match effect {
        MaterialEffect::Solid => ("solid", None),
        MaterialEffect::TintedFallback => ("tinted_fallback", None),
        MaterialEffect::WallpaperMaterial { backend } => {
            ("wallpaper_material", Some((*backend).to_string()))
        }
        MaterialEffect::CompositorBlur { backend, radius } => (
            "compositor_blur",
            Some(format!("{backend} radius {radius:.1}")),
        ),
        MaterialEffect::NativeMaterial { name } => ("native_material", Some((*name).to_string())),
    }
}

fn material_name(material: &Material) -> &'static str {
    match material {
        Material::Solid(_) => "solid",
        Material::Surface => "surface",
        Material::SurfaceElevated => "surface_elevated",
        Material::Window => "window",
        Material::Sidebar => "sidebar",
        Material::Toolbar => "toolbar",
        Material::Popover => "popover",
        Material::Menu => "menu",
        Material::Dialog => "dialog",
        Material::Maris => "maris",
        Material::Luca => "luca",
    }
}
