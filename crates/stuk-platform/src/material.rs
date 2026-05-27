use stuk_style::{Color, Material, Theme};

use crate::GenericPlatform;

#[derive(Clone, Debug, PartialEq)]
pub struct MaterialResolution {
    pub material: Material,
    pub effect: MaterialEffect,
    pub fallback: Color,
}

impl MaterialResolution {
    pub fn fallback(material: &Material, theme: &Theme) -> Self {
        Self {
            material: material.clone(),
            effect: if matches!(material, Material::Solid(_)) {
                MaterialEffect::Solid
            } else {
                MaterialEffect::TintedFallback
            },
            fallback: material.fallback_color_for(theme),
        }
    }

    pub fn with_effect(material: &Material, theme: &Theme, effect: MaterialEffect) -> Self {
        Self {
            material: material.clone(),
            effect,
            fallback: material.fallback_color_for(theme),
        }
    }

    pub fn requires_compositor(&self) -> bool {
        matches!(self.effect, MaterialEffect::CompositorBlur { .. })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MaterialEffect {
    Solid,
    TintedFallback,
    WallpaperMaterial { backend: &'static str },
    CompositorBlur { backend: &'static str, radius: f32 },
    NativeMaterial { name: &'static str },
}

pub trait MaterialResolver {
    fn resolve_material(&self, material: &Material, theme: &Theme) -> MaterialResolution;
}

impl MaterialResolver for GenericPlatform {
    fn resolve_material(&self, material: &Material, theme: &Theme) -> MaterialResolution {
        MaterialResolution::fallback(material, theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_resolution_uses_tinted_fallbacks_for_semantic_materials() {
        let theme = Theme::dark();
        let resolution = MaterialResolution::fallback(&Material::Maris, &theme);

        assert_eq!(resolution.effect, MaterialEffect::TintedFallback);
        assert_eq!(resolution.fallback, theme.colors.window);
        assert!(!resolution.requires_compositor());
    }
}
