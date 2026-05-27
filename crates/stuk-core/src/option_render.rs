use stuk_actions::ActionHitRegion;
use stuk_layout::Rect;
use stuk_render::{DisplayList, RectCommand, RoundedRectCommand, TextCommand};
use stuk_style::{Color, Theme};

use crate::{ControlOptionElement, SegmentedControlElement, TabsElement};

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
        if index == selected {
            rounded(list, rect, theme.radius.md, theme.colors.control);
            if underline {
                list.push(RectCommand {
                    x: rect.x + 12.0,
                    y: rect.y + rect.height - 2.0,
                    width: (rect.width - 24.0).max(1.0),
                    height: 2.0,
                    color: theme.colors.accent,
                });
            }
        }
        label(
            list,
            &option.label,
            rect.x + 14.0,
            rect.y + 8.0,
            (rect.width - 28.0).max(1.0),
            theme
                .colors
                .text
                .opacity(if option.disabled { 0.45 } else { 1.0 }),
        );
        push_action_region(
            hit_regions,
            rect,
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
