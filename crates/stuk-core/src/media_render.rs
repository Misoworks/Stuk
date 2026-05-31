use stuk_layout::Rect;
use stuk_render::{BorderCommand, DisplayList, ImageCommand, SvgCommand};
use stuk_style::{Color, Theme, ThemeMode};

use crate::media_elements::{MediaElement, MediaSource};

pub(crate) fn render_media(
    media: &MediaElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
) {
    match media.source {
        MediaSource::Image => {
            list.push(ImageCommand {
                id: media.id.clone(),
                x: bounds.x,
                y: bounds.y,
                width: bounds.width,
                height: bounds.height,
                opacity: media.opacity,
            });
            render_outline(media, bounds, theme, list);
        }
        MediaSource::Svg => {
            list.push(SvgCommand {
                id: media.id.clone(),
                x: bounds.x,
                y: bounds.y,
                width: bounds.width,
                height: bounds.height,
                tint: media.tint.map(|color| theme.resolve_color(color)),
                opacity: media.opacity,
            });
            render_outline(media, bounds, theme, list);
        }
    }
}

fn render_outline(media: &MediaElement, bounds: Rect, theme: &Theme, list: &mut DisplayList) {
    if !media.outline {
        return;
    }

    let edge = match theme.mode {
        ThemeMode::Light => Color::rgb(0.0, 0.0, 0.0),
        ThemeMode::System | ThemeMode::Dark => Color::WHITE,
    };

    list.push(BorderCommand {
        x: bounds.x,
        y: bounds.y,
        width: bounds.width,
        height: bounds.height,
        radius: 0.0,
        thickness: 1.0,
        color: edge.opacity(0.10),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use stuk_render::DisplayCommand;

    #[test]
    fn image_draws_subtle_outline() {
        let media = MediaElement::new(MediaSource::Image, "hero");
        let mut list = DisplayList::new(Color::WINDOW);

        render_media(
            &media,
            Rect::new(0.0, 0.0, 100.0, 80.0),
            &Theme::dark(),
            &mut list,
        );

        assert!(matches!(list.commands[0], DisplayCommand::Image(_)));
        let DisplayCommand::Border(border) = &list.commands[1] else {
            panic!("image should draw an outline");
        };
        assert_eq!(border.color, Color::WHITE.opacity(0.10));
    }

    #[test]
    fn svg_skips_outline_by_default() {
        let media = MediaElement::new(MediaSource::Svg, "icon");
        let mut list = DisplayList::new(Color::WINDOW);

        render_media(
            &media,
            Rect::new(0.0, 0.0, 24.0, 24.0),
            &Theme::dark(),
            &mut list,
        );

        assert_eq!(list.commands.len(), 1);
        assert!(matches!(list.commands[0], DisplayCommand::Svg(_)));
    }

    #[test]
    fn svg_can_opt_into_outline() {
        let mut media = MediaElement::new(MediaSource::Svg, "icon");
        media.outline = true;
        let mut list = DisplayList::new(Color::WINDOW);

        render_media(
            &media,
            Rect::new(0.0, 0.0, 24.0, 24.0),
            &Theme::light(),
            &mut list,
        );

        assert!(matches!(list.commands[0], DisplayCommand::Svg(_)));
        let DisplayCommand::Border(border) = &list.commands[1] else {
            panic!("outlined svg should draw a border");
        };
        assert_eq!(border.color, Color::rgb(0.0, 0.0, 0.0).opacity(0.10));
    }
}
