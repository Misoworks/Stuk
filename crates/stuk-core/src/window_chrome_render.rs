use stuk_actions::ActionHitRegion;
use stuk_layout::Rect;
use stuk_platform::WindowChrome;
use stuk_render::{DisplayList, RectCommand, RoundedRectCommand, TextCommand};
use stuk_style::{NumberSpacing, TextAlign, TextWrap, Theme};

const CHROME_HEIGHT: f32 = 38.0;
const CONTROL_SIZE: f32 = 24.0;
const CONTROL_GAP: f32 = 8.0;

pub(crate) const ACTION_WINDOW_CLOSE: &str = "window.close";
pub(crate) const ACTION_WINDOW_MINIMIZE: &str = "window.minimize";
pub(crate) const ACTION_WINDOW_TOGGLE_MAXIMIZE: &str = "window.toggle-maximize";

#[derive(Clone, Copy, Debug)]
enum ChromeControlKind {
    Close,
    Maximize,
    Minimize,
}

pub(crate) fn content_bounds(chrome: WindowChrome, bounds: Rect) -> Rect {
    if uses_stuk_chrome(chrome) {
        Rect::new(
            bounds.x,
            bounds.y + CHROME_HEIGHT,
            bounds.width,
            (bounds.height - CHROME_HEIGHT).max(1.0),
        )
    } else {
        bounds
    }
}

pub(crate) fn render_window_chrome(
    chrome: WindowChrome,
    title: &str,
    bounds: Rect,
    radius: f32,
    theme: &Theme,
    list: &mut DisplayList,
    hit_regions: &mut Vec<ActionHitRegion>,
) {
    if !uses_stuk_chrome(chrome) {
        return;
    }

    let titlebar_color = theme.colors.toolbar.opacity(0.58);
    list.push(RoundedRectCommand {
        x: bounds.x,
        y: bounds.y,
        width: bounds.width,
        height: CHROME_HEIGHT,
        radius,
        color: titlebar_color,
    });
    if radius > 0.0 {
        list.push(RectCommand {
            x: bounds.x,
            y: bounds.y + (CHROME_HEIGHT - radius).max(0.0),
            width: bounds.width,
            height: radius.min(CHROME_HEIGHT),
            color: titlebar_color,
        });
    }
    list.push(RectCommand {
        x: bounds.x,
        y: bounds.y + CHROME_HEIGHT - 1.0,
        width: bounds.width,
        height: 1.0,
        color: theme.colors.outline.opacity(0.55),
    });

    list.push(TextCommand {
        text: title.to_string(),
        x: bounds.x,
        y: bounds.y + 8.0,
        width: bounds.width.max(1.0),
        height: 22.0,
        size: 14.0,
        line_height: 20.0,
        color: theme.colors.text,
        wrap: TextWrap::Pretty,
        align: TextAlign::Center,
        number_spacing: NumberSpacing::Proportional,
    });

    let right = bounds.x + bounds.width - 12.0;
    let y = bounds.y + (CHROME_HEIGHT - CONTROL_SIZE) * 0.5;
    chrome_control(
        Rect::new(right - CONTROL_SIZE, y, CONTROL_SIZE, CONTROL_SIZE),
        ChromeControlKind::Close,
        ACTION_WINDOW_CLOSE,
        theme,
        list,
        hit_regions,
    );
    chrome_control(
        Rect::new(
            right - CONTROL_SIZE * 2.0 - CONTROL_GAP,
            y,
            CONTROL_SIZE,
            CONTROL_SIZE,
        ),
        ChromeControlKind::Maximize,
        ACTION_WINDOW_TOGGLE_MAXIMIZE,
        theme,
        list,
        hit_regions,
    );
    chrome_control(
        Rect::new(
            right - CONTROL_SIZE * 3.0 - CONTROL_GAP * 2.0,
            y,
            CONTROL_SIZE,
            CONTROL_SIZE,
        ),
        ChromeControlKind::Minimize,
        ACTION_WINDOW_MINIMIZE,
        theme,
        list,
        hit_regions,
    );
}

fn uses_stuk_chrome(chrome: WindowChrome) -> bool {
    matches!(
        chrome,
        WindowChrome::Stuk | WindowChrome::Compact | WindowChrome::Sidebar
    )
}

fn chrome_control(
    rect: Rect,
    kind: ChromeControlKind,
    action: &str,
    theme: &Theme,
    list: &mut DisplayList,
    hit_regions: &mut Vec<ActionHitRegion>,
) {
    let hovered = list
        .hovered_region
        .as_deref()
        .is_some_and(|id| id == ActionHitRegion::region_id_for(rect, action));
    let pressed = list
        .pressed_region
        .as_deref()
        .is_some_and(|id| id == ActionHitRegion::region_id_for(rect, action));
    let fill_alpha = if pressed {
        0.28
    } else if hovered {
        0.18
    } else {
        0.08
    };

    list.push(RoundedRectCommand {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
        radius: 999.0,
        color: theme.colors.text.opacity(fill_alpha),
    });
    render_control_icon(
        kind,
        rect,
        theme
            .colors
            .text
            .opacity(if hovered || pressed { 0.95 } else { 0.68 }),
        list,
    );

    hit_regions.push(ActionHitRegion::new(rect, action));
}

fn render_control_icon(
    kind: ChromeControlKind,
    rect: Rect,
    color: stuk_style::Color,
    list: &mut DisplayList,
) {
    match kind {
        ChromeControlKind::Close => {
            let center_x = rect.x + rect.width * 0.5;
            let center_y = rect.y + rect.height * 0.5;
            for (dx, dy) in [
                (-4.0, -4.0),
                (-2.0, -2.0),
                (0.0, 0.0),
                (2.0, 2.0),
                (4.0, 4.0),
                (-4.0, 4.0),
                (-2.0, 2.0),
                (2.0, -2.0),
                (4.0, -4.0),
            ] {
                list.push(RectCommand {
                    x: center_x + dx - 0.9,
                    y: center_y + dy - 0.9,
                    width: 1.8,
                    height: 1.8,
                    color,
                });
            }
        }
        ChromeControlKind::Maximize => {
            let x = rect.x + (rect.width - 9.0) * 0.5;
            let y = rect.y + (rect.height - 9.0) * 0.5;
            list.push(RectCommand {
                x,
                y,
                width: 9.0,
                height: 1.5,
                color,
            });
            list.push(RectCommand {
                x,
                y: y + 7.5,
                width: 9.0,
                height: 1.5,
                color,
            });
            list.push(RectCommand {
                x,
                y,
                width: 1.5,
                height: 9.0,
                color,
            });
            list.push(RectCommand {
                x: x + 7.5,
                y,
                width: 1.5,
                height: 9.0,
                color,
            });
        }
        ChromeControlKind::Minimize => {
            list.push(RectCommand {
                x: rect.x + (rect.width - 9.0) * 0.5,
                y: rect.y + rect.height * 0.5 - 0.75,
                width: 9.0,
                height: 1.5,
                color,
            });
        }
    }
}
