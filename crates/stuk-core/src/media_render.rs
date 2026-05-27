use stuk_layout::Rect;
use stuk_render::{DisplayList, ImageCommand, SvgCommand};
use stuk_style::Theme;

use crate::media_elements::{MediaElement, MediaSource};

pub(crate) fn render_media(
    media: &MediaElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
) {
    match media.source {
        MediaSource::Image => list.push(ImageCommand {
            id: media.id.clone(),
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: bounds.height,
            opacity: media.opacity,
        }),
        MediaSource::Svg => list.push(SvgCommand {
            id: media.id.clone(),
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: bounds.height,
            tint: media.tint.map(|color| theme.resolve_color(color)),
            opacity: media.opacity,
        }),
    }
}
