use stuk_actions::ActionHitRegion;
use stuk_layout::Rect;
use stuk_render::{DisplayList, RoundedRectCommand, TextCommand};
use stuk_style::{Color, Theme};

use crate::element::{ButtonElement, IconButtonElement, TextFieldElement, ToggleElement};
use crate::{
    AvatarElement, BadgeElement, CheckboxElement, ProgressBarElement, RadioElement, SliderElement,
    TooltipElement,
};

pub(crate) fn render_button(
    button: &ButtonElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
    hit_regions: &mut Vec<ActionHitRegion>,
) {
    let button_bounds = Rect::new(bounds.x, bounds.y, bounds.width.min(220.0), bounds.height);
    let fill = if button.disabled {
        button.variant.fill_for(theme).opacity(0.4)
    } else {
        button.variant.fill_for(theme)
    };
    rounded(list, button_bounds, theme.radius.md, fill);
    label(
        list,
        &button.label,
        button_bounds.x + 16.0,
        button_bounds.y + 9.0,
        (button_bounds.width - 32.0).max(1.0),
        button.variant.text_for(theme),
    );
    push_action_region(
        hit_regions,
        button_bounds,
        button.action.as_deref(),
        !button.disabled,
    );
}

pub(crate) fn render_icon_button(
    button: &IconButtonElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
    hit_regions: &mut Vec<ActionHitRegion>,
) {
    let rect = Rect::new(
        bounds.x,
        bounds.y,
        bounds.width.min(38.0),
        bounds.height.min(38.0),
    );
    rounded(
        list,
        rect,
        theme.radius.md,
        theme
            .colors
            .control
            .opacity(if button.disabled { 0.45 } else { 1.0 }),
    );
    label(
        list,
        &button.icon,
        rect.x + 11.0,
        rect.y + 8.0,
        (rect.width - 22.0).max(1.0),
        theme
            .colors
            .text
            .opacity(if button.disabled { 0.45 } else { 1.0 }),
    );
    push_action_region(
        hit_regions,
        rect,
        button.action.as_deref(),
        !button.disabled,
    );
}

pub(crate) fn render_toggle(
    toggle: &ToggleElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
    hit_regions: &mut Vec<ActionHitRegion>,
) {
    let track = Rect::new(bounds.x, bounds.y + 3.0, 42.0, 24.0);
    let knob_x = if toggle.checked {
        track.x + 20.0
    } else {
        track.x + 4.0
    };
    rounded(
        list,
        track,
        theme.radius.pill,
        if toggle.checked {
            theme.colors.accent
        } else {
            theme.colors.control
        },
    );
    rounded(
        list,
        Rect::new(knob_x, track.y + 4.0, 16.0, 16.0),
        theme.radius.pill,
        theme.colors.on_accent,
    );
    label(
        list,
        &toggle.label,
        bounds.x + 54.0,
        bounds.y + 1.0,
        (bounds.width - 54.0).max(1.0),
        theme
            .colors
            .text
            .opacity(if toggle.disabled { 0.45 } else { 1.0 }),
    );
    push_action_region(
        hit_regions,
        bounds,
        toggle.action.as_deref(),
        !toggle.disabled,
    );
}

pub(crate) fn render_checkbox(
    checkbox: &CheckboxElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
    hit_regions: &mut Vec<ActionHitRegion>,
) {
    let box_rect = Rect::new(bounds.x, bounds.y + 3.0, 20.0, 20.0);
    rounded(
        list,
        box_rect,
        theme.radius.sm,
        if checkbox.checked {
            theme.colors.accent
        } else {
            theme.colors.control
        },
    );
    if checkbox.checked {
        rounded(
            list,
            Rect::new(box_rect.x + 5.0, box_rect.y + 5.0, 10.0, 10.0),
            theme.radius.xs,
            theme.colors.on_accent,
        );
    }
    label(
        list,
        &checkbox.label,
        bounds.x + 30.0,
        bounds.y + 2.0,
        (bounds.width - 30.0).max(1.0),
        theme
            .colors
            .text
            .opacity(if checkbox.disabled { 0.45 } else { 1.0 }),
    );
    push_action_region(
        hit_regions,
        bounds,
        checkbox.action.as_deref(),
        !checkbox.disabled,
    );
}

pub(crate) fn render_radio(
    radio: &RadioElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
    hit_regions: &mut Vec<ActionHitRegion>,
) {
    let outer = Rect::new(bounds.x, bounds.y + 3.0, 20.0, 20.0);
    rounded(list, outer, theme.radius.pill, theme.colors.control);
    if radio.selected {
        rounded(
            list,
            Rect::new(outer.x + 5.0, outer.y + 5.0, 10.0, 10.0),
            theme.radius.pill,
            theme.colors.accent,
        );
    }
    label(
        list,
        &radio.label,
        bounds.x + 30.0,
        bounds.y + 2.0,
        (bounds.width - 30.0).max(1.0),
        theme
            .colors
            .text
            .opacity(if radio.disabled { 0.45 } else { 1.0 }),
    );
    push_action_region(
        hit_regions,
        bounds,
        radio.action.as_deref(),
        !radio.disabled,
    );
}

pub(crate) fn render_slider(
    slider: &SliderElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
    hit_regions: &mut Vec<ActionHitRegion>,
) {
    let label_height = if let Some(text) = &slider.label {
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
    let track = Rect::new(
        bounds.x,
        bounds.y + label_height + 12.0,
        bounds.width.min(240.0),
        6.0,
    );
    let ratio = slider_ratio(slider.value, slider.min, slider.max);
    rounded(list, track, theme.radius.pill, theme.colors.control);
    rounded(
        list,
        Rect::new(
            track.x,
            track.y,
            (track.width * ratio).max(1.0),
            track.height,
        ),
        theme.radius.pill,
        theme
            .colors
            .accent
            .opacity(if slider.disabled { 0.45 } else { 1.0 }),
    );
    rounded(
        list,
        Rect::new(
            track.x + track.width * ratio - 6.0,
            track.y - 5.0,
            16.0,
            16.0,
        ),
        theme.radius.pill,
        theme.colors.text,
    );
    push_action_region(
        hit_regions,
        bounds,
        slider.action.as_deref(),
        !slider.disabled,
    );
}

pub(crate) fn render_progress_bar(
    progress: &ProgressBarElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
) {
    let label_height = if let Some(text) = &progress.label {
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
    let track = Rect::new(
        bounds.x,
        bounds.y + label_height + 6.0,
        bounds.width.min(220.0),
        8.0,
    );
    rounded(list, track, theme.radius.pill, theme.colors.control);
    rounded(
        list,
        Rect::new(
            track.x,
            track.y,
            track.width * progress_ratio(progress),
            track.height,
        ),
        theme.radius.pill,
        theme.colors.accent,
    );
}

pub(crate) fn render_badge(
    badge: &BadgeElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
) {
    let rect = Rect::new(
        bounds.x,
        bounds.y,
        bounds.width.min(180.0),
        bounds.height.min(24.0),
    );
    rounded(
        list,
        rect,
        theme.radius.pill,
        theme.resolve_color(badge.color).opacity(0.22),
    );
    label(
        list,
        &badge.label,
        rect.x + 10.0,
        rect.y + 3.0,
        (rect.width - 20.0).max(1.0),
        theme.colors.text,
    );
}

pub(crate) fn render_avatar(
    avatar: &AvatarElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
) {
    let size = bounds.width.min(bounds.height).clamp(32.0, 46.0);
    let rect = Rect::new(bounds.x, bounds.y, size, size);
    rounded(list, rect, theme.radius.pill, theme.colors.control);
    label(
        list,
        &avatar.initials,
        rect.x + 10.0,
        rect.y + 9.0,
        (rect.width - 20.0).max(1.0),
        theme.colors.text,
    );
}

pub(crate) fn render_text_field(
    field: &TextFieldElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
) {
    let label_height = if let Some(label_text) = &field.label {
        label(
            list,
            label_text,
            bounds.x,
            bounds.y,
            bounds.width,
            theme.colors.text_muted,
        );
        22.0
    } else {
        0.0
    };
    let field_rect = Rect::new(
        bounds.x,
        bounds.y + label_height,
        bounds.width.min(280.0),
        38.0,
    );
    rounded(
        list,
        field_rect,
        theme.radius.md,
        theme
            .colors
            .control
            .opacity(if field.disabled { 0.45 } else { 1.0 }),
    );
    let text = if field.text.is_empty() {
        &field.placeholder
    } else {
        &field.text
    };
    label(
        list,
        text,
        field_rect.x + 12.0,
        field_rect.y + 9.0,
        (field_rect.width - 24.0).max(1.0),
        if field.text.is_empty() {
            theme.colors.text_muted
        } else {
            theme.colors.text
        },
    );
}

pub(crate) fn render_tooltip_label(
    tooltip: &TooltipElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
) {
    if tooltip.label.trim().is_empty() {
        return;
    }
    let width = (tooltip.label.chars().count() as f32 * 7.0 + 18.0).clamp(64.0, 240.0);
    let rect = Rect::new(bounds.x + bounds.width + 8.0, bounds.y, width, 26.0);
    rounded(list, rect, theme.radius.sm, theme.colors.surface_elevated);
    label(
        list,
        &tooltip.label,
        rect.x + 9.0,
        rect.y + 4.0,
        (rect.width - 18.0).max(1.0),
        theme.colors.text,
    );
}

fn slider_ratio(value: f32, min: f32, max: f32) -> f32 {
    if max <= min {
        return 0.0;
    }
    ((value - min) / (max - min)).clamp(0.0, 1.0)
}

fn progress_ratio(progress: &ProgressBarElement) -> f32 {
    if progress.max <= 0.0 {
        return 0.0;
    }
    (progress.value / progress.max).clamp(0.0, 1.0)
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
