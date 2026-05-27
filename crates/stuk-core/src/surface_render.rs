use stuk_layout::Rect;
use stuk_render::{BorderCommand, DisplayList, MaterialCommand, ShadowCommand};
use stuk_style::Theme;

use crate::surface_elements::SurfaceElement;

pub(crate) fn render_surface_commands(
    surface: &SurfaceElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
) -> Rect {
    if let Some(shadow) = &surface.shadow {
        list.push(ShadowCommand {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: bounds.height,
            radius: surface.radius,
            offset_x: shadow.offset_x,
            offset_y: shadow.offset_y,
            blur: shadow.blur,
            spread: shadow.spread,
            color: theme
                .resolve_color(shadow.color)
                .opacity(surface.opacity.clamp(0.0, 1.0)),
        });
    }

    list.push(MaterialCommand {
        material: surface.material.clone(),
        x: bounds.x,
        y: bounds.y,
        width: bounds.width,
        height: bounds.height,
        radius: surface.radius,
        fallback: surface
            .material
            .fallback_color_for(theme)
            .opacity(surface.opacity.clamp(0.0, 1.0)),
    });

    if let Some(border) = &surface.border {
        list.push(BorderCommand {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: bounds.height,
            radius: surface.radius,
            thickness: border.thickness,
            color: theme
                .resolve_color(border.color)
                .opacity(surface.opacity.clamp(0.0, 1.0)),
        });
    }

    bounds.inset(surface.padding)
}
