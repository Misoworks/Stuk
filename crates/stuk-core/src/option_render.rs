use stuk_actions::ActionHitRegion;
use stuk_layout::Rect;
use stuk_render::{DisplayList, RectCommand, RoundedRectCommand, TextCommand};
use stuk_style::{Color, NumberSpacing, TextAlign, TextWrap, Theme};

use crate::{ControlOptionElement, SegmentedControlElement, TabsElement};

const MIN_HIT_SIZE: f32 = 40.0;
const PRESS_SCALE: f32 = 0.96;

pub(crate) fn render_tabs(
    tabs: &TabsElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
    hit_regions: &mut Vec<ActionHitRegion>,
) {
    render_option_row(
        &tabs.options,
        tabs.selected,
        bounds,
        theme,
        list,
        hit_regions,
        true,
    );
}

pub(crate) fn render_segmented_control(
    control: &SegmentedControlElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
    hit_regions: &mut Vec<ActionHitRegion>,
) {
    let label_height = if let Some(text) = &control.label {
        label(
            list,
            text,
            bounds.x,
            bounds.y,
            bounds.width,
            theme.colors.text_muted,
        );
        20.0
    } else {
        0.0
    };
    let row = Rect::new(bounds.x, bounds.y + label_height, bounds.width, 34.0);
    render_option_row(
        &control.options,
        control.selected,
        row,
        theme,
        list,
        hit_regions,
        false,
    );
}

fn render_option_row(
    options: &[ControlOptionElement],
    selected: usize,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
    hit_regions: &mut Vec<ActionHitRegion>,
    underline: bool,
) {
    let mut x = bounds.x;
    for (index, option) in options.iter().enumerate() {
        let width = (option.label.chars().count() as f32 * 7.5 + 28.0).clamp(56.0, 180.0);
        let rect = Rect::new(x, bounds.y, width, bounds.height.min(36.0));
        let hit_rect = min_hit_rect(rect);
        let hovered = is_hovered(list, option.action.as_deref(), hit_rect) && !option.disabled;
        let pressed = is_pressed(list, option.action.as_deref(), hit_rect) && !option.disabled;
        let visual = pressed_rect(rect, pressed);
        if index == selected {
            rounded(list, visual, theme.radius.md, theme.colors.control);
            if underline {
                list.push(RectCommand {
                    x: visual.x + 12.0,
                    y: visual.y + visual.height - 2.0,
                    width: (visual.width - 24.0).max(1.0),
                    height: 2.0,
                    color: theme.colors.accent,
                });
            }
        } else if hovered {
            rounded(
                list,
                visual,
                theme.radius.md,
                theme.colors.control.opacity(0.42),
            );
        }
        label(
            list,
            &option.label,
            visual.x + 14.0,
            visual.y + 8.0,
            (visual.width - 28.0).max(1.0),
            theme
                .colors
                .text
                .opacity(if option.disabled { 0.45 } else { 1.0 }),
        );
        push_action_region(
            hit_regions,
            hit_rect,
            option.action.as_deref(),
            !option.disabled,
        );
        x += width;
    }
}

fn rounded(list: &mut DisplayList, rect: Rect, radius: f32, color: Color) {
    list.push(RoundedRectCommand {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
        radius,
        color,
    });
}

fn label(list: &mut DisplayList, text: &str, x: f32, y: f32, width: f32, color: Color) {
    list.push(TextCommand {
        text: text.to_string(),
        x,
        y,
        width,
        height: 20.0,
        size: 14.0,
        line_height: 20.0,
        color,
        wrap: TextWrap::Pretty,
        align: TextAlign::Start,
        number_spacing: NumberSpacing::Proportional,
    });
}

fn push_action_region(
    hit_regions: &mut Vec<ActionHitRegion>,
    rect: Rect,
    action: Option<&str>,
    enabled: bool,
) {
    if let Some(action) = action {
        let mut region = ActionHitRegion::new(rect, action);
        region.enabled = enabled;
        hit_regions.push(region);
    }
}

fn is_hovered(list: &DisplayList, action: Option<&str>, rect: Rect) -> bool {
    action.is_some_and(|action| {
        list.hovered_region.as_deref()
            == Some(ActionHitRegion::region_id_for(rect, action).as_str())
    })
}

fn is_pressed(list: &DisplayList, action: Option<&str>, rect: Rect) -> bool {
    action.is_some_and(|action| {
        list.pressed_region.as_deref()
            == Some(ActionHitRegion::region_id_for(rect, action).as_str())
    })
}

fn pressed_rect(rect: Rect, pressed: bool) -> Rect {
    if !pressed {
        return rect;
    }
    let width = rect.width * PRESS_SCALE;
    let height = rect.height * PRESS_SCALE;
    Rect::new(
        rect.x + (rect.width - width) * 0.5,
        rect.y + (rect.height - height) * 0.5,
        width,
        height,
    )
}

fn min_hit_rect(rect: Rect) -> Rect {
    let width = rect.width.max(MIN_HIT_SIZE);
    let height = rect.height.max(MIN_HIT_SIZE);
    Rect::new(
        rect.x + (rect.width - width) * 0.5,
        rect.y + (rect.height - height) * 0.5,
        width,
        height,
    )
}

#[cfg(test)]
mod tests {
    use stuk_render::DisplayCommand;

    use super::*;

    #[test]
    fn option_hit_regions_are_at_least_forty_points_high() {
        let tabs = TabsElement {
            options: vec![ControlOptionElement {
                id: "one".to_string(),
                label: "One".to_string(),
                action: Some("tabs.one".to_string()),
                disabled: false,
            }],
            selected: 0,
        };
        let mut list = DisplayList::new(Color::WINDOW);
        let mut hit_regions = Vec::new();

        render_tabs(
            &tabs,
            Rect::new(0.0, 0.0, 90.0, 36.0),
            &Theme::dark(),
            &mut list,
            &mut hit_regions,
        );

        assert_eq!(hit_regions.len(), 1);
        assert!(hit_regions[0].rect.height >= 40.0);
    }

    #[test]
    fn pressed_option_uses_subtle_scale() {
        let tabs = TabsElement {
            options: vec![ControlOptionElement {
                id: "one".to_string(),
                label: "One".to_string(),
                action: Some("tabs.one".to_string()),
                disabled: false,
            }],
            selected: 0,
        };
        let mut list = DisplayList::new(Color::WINDOW);
        list.pressed_region = Some(ActionHitRegion::region_id_for(
            Rect::new(0.0, -2.0, 56.0, 40.0),
            "tabs.one",
        ));
        let mut hit_regions = Vec::new();

        render_tabs(
            &tabs,
            Rect::new(0.0, 0.0, 90.0, 36.0),
            &Theme::dark(),
            &mut list,
            &mut hit_regions,
        );

        let rounded = list
            .commands
            .iter()
            .find_map(|command| match command {
                DisplayCommand::RoundedRect(rect) => Some(rect),
                _ => None,
            })
            .expect("selected tab should draw rounded rect");

        assert!((rounded.height - 34.56).abs() < 0.01);
    }
}
