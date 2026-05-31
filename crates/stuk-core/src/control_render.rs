use stuk_actions::ActionHitRegion;
use stuk_layout::Rect;
use stuk_render::{DisplayList, RoundedRectCommand, ShadowCommand, TextCommand};
use stuk_style::{ButtonVariant, Color, NumberSpacing, TextAlign, TextWrap, Theme};

pub(crate) use crate::control_render_extras::{
    button_fill, darker, is_hovered, is_pressed, lighter, render_avatar, render_badge,
    render_progress_bar, render_text_field, render_tooltip_label,
};
use crate::element::{ButtonElement, IconButtonElement, ToggleElement};
use crate::{CheckboxElement, RadioElement, SliderElement};

const MIN_HIT_SIZE: f32 = 40.0;
const PRESS_SCALE: f32 = 1.0;
const BUTTON_X_PADDING: f32 = 18.0;

pub(crate) fn render_button(
    button: &ButtonElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
    hit_regions: &mut Vec<ActionHitRegion>,
) {
    let height = 36.0f32.min(bounds.height.max(32.0));
    let label_width = text_width(&button.label, 14.0);
    let width = match button.text_align {
        stuk_style::ControlTextAlign::Start => bounds.width.min(280.0).max(72.0),
        stuk_style::ControlTextAlign::Center => (label_width + BUTTON_X_PADDING * 2.0)
            .max(72.0)
            .min(bounds.width.min(280.0)),
    };
    let x = match button.text_align {
        stuk_style::ControlTextAlign::Start => bounds.x,
        stuk_style::ControlTextAlign::Center => bounds.x + (bounds.width - width) * 0.5,
    };
    let btn = Rect::new(x, bounds.y + (bounds.height - height) * 0.5, width, height);
    let hit_rect = min_hit_rect(btn);
    let is_hovered = is_hovered(list, button.action.as_deref(), hit_rect);
    let is_pressed = is_pressed(list, button.action.as_deref(), hit_rect);
    let opaque = !button.disabled;
    let visual = pressed_rect(btn, is_pressed && opaque);
    let fill = button_fill(theme, button.variant, is_hovered, is_pressed, opaque);
    let text_color = if opaque {
        button.variant.text_for(theme)
    } else {
        button.variant.text_for(theme).opacity(0.4)
    };
    let is_solid = matches!(
        button.variant,
        ButtonVariant::Primary | ButtonVariant::Destructive
    );

    if is_solid && opaque {
        let shadow_color = if matches!(button.variant, ButtonVariant::Primary) {
            theme.colors.accent.opacity(0.35)
        } else {
            theme.colors.danger.opacity(0.3)
        };
        list.push(ShadowCommand {
            x: visual.x,
            y: visual.y,
            width: visual.width,
            height: visual.height,
            radius: theme.radius.md,
            offset_x: 0.0,
            offset_y: if is_pressed { 1.0 } else { 2.0 },
            blur: if is_pressed { 5.0 } else { 8.0 },
            spread: 0.0,
            color: shadow_color,
        });
    }

    rounded(list, visual, theme.radius.md, fill);
    match button.text_align {
        stuk_style::ControlTextAlign::Center => {
            label_centered(list, &button.label, visual, text_color)
        }
        stuk_style::ControlTextAlign::Start => label(
            list,
            &button.label,
            visual.x + BUTTON_X_PADDING,
            centered_label_y(visual, 20.0),
            (visual.width - BUTTON_X_PADDING * 2.0).max(1.0),
            text_color,
        ),
    }
    push_action_region(hit_regions, hit_rect, button.action.as_deref(), opaque);
}

pub(crate) fn render_icon_button(
    button: &IconButtonElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
    hit_regions: &mut Vec<ActionHitRegion>,
) {
    let size = 34.0f32.min(bounds.width.min(bounds.height));
    let rect = Rect::new(
        bounds.x + (bounds.width - size) * 0.5,
        bounds.y + (bounds.height - size) * 0.5,
        size,
        size,
    );
    let hit_rect = min_hit_rect(rect);
    let is_hovered = is_hovered(list, button.action.as_deref(), hit_rect);
    let is_pressed = is_pressed(list, button.action.as_deref(), hit_rect);
    let visual = pressed_rect(rect, is_pressed && !button.disabled);
    let opacity = if button.disabled {
        0.3
    } else if is_pressed {
        0.7
    } else if is_hovered {
        0.9
    } else {
        0.6
    };

    let fill_opacity = if is_pressed && !button.disabled {
        0.28
    } else if is_hovered && !button.disabled {
        0.20
    } else {
        0.0
    };
    rounded(
        list,
        visual,
        theme.radius.md,
        theme.colors.text.opacity(fill_opacity),
    );
    label_centered(
        list,
        &button.icon,
        visual,
        theme.colors.text.opacity(opacity),
    );
    push_action_region(
        hit_regions,
        hit_rect,
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
    let hit_rect = min_hit_rect(bounds);
    let is_hovered = is_hovered(list, toggle.action.as_deref(), hit_rect);
    let is_pressed = is_pressed(list, toggle.action.as_deref(), hit_rect);
    let track = Rect::new(bounds.x, bounds.y + 1.0, 44.0, 26.0);
    let visual_track = pressed_rect(track, is_pressed && !toggle.disabled);
    let knob_x = if toggle.checked {
        visual_track.x + 20.0
    } else {
        visual_track.x + 4.0
    };
    let track_color = if toggle.checked {
        if is_pressed && !toggle.disabled {
            darker(theme.colors.accent, 0.10)
        } else if is_hovered && !toggle.disabled {
            lighter(theme.colors.accent, 0.10)
        } else {
            theme.colors.accent
        }
    } else if is_pressed && !toggle.disabled {
        darker(theme.colors.control, 0.08)
    } else if is_hovered && !toggle.disabled {
        lighter(theme.colors.control, 0.10)
    } else {
        theme.colors.control
    };
    rounded(
        list,
        visual_track,
        theme.radius.pill,
        track_color.opacity(if toggle.disabled { 0.45 } else { 1.0 }),
    );
    rounded(
        list,
        Rect::new(knob_x, visual_track.y + 4.0, 18.0, 18.0),
        theme.radius.pill,
        theme.colors.on_accent,
    );
    label(
        list,
        &toggle.label,
        bounds.x + 54.0,
        centered_label_y(bounds, 20.0),
        (bounds.width - 54.0).max(1.0),
        theme
            .colors
            .text
            .opacity(if toggle.disabled { 0.45 } else { 1.0 }),
    );
    push_action_region(
        hit_regions,
        hit_rect,
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
    let hit_rect = min_hit_rect(bounds);
    let is_hovered = is_hovered(list, checkbox.action.as_deref(), hit_rect);
    let is_pressed = is_pressed(list, checkbox.action.as_deref(), hit_rect);
    let box_rect = Rect::new(bounds.x, bounds.y + 3.0, 20.0, 20.0);
    let visual_box = pressed_rect(box_rect, is_pressed && !checkbox.disabled);
    rounded(
        list,
        visual_box,
        theme.radius.sm,
        if checkbox.checked {
            theme.colors.accent
        } else if is_hovered && !checkbox.disabled {
            lighter(theme.colors.control, 0.10)
        } else if is_pressed && !checkbox.disabled {
            darker(theme.colors.control, 0.08)
        } else {
            theme.colors.control
        },
    );
    if checkbox.checked {
        rounded(
            list,
            Rect::new(visual_box.x + 5.0, visual_box.y + 5.0, 10.0, 10.0),
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
        hit_rect,
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
    let hit_rect = min_hit_rect(bounds);
    let is_hovered = is_hovered(list, radio.action.as_deref(), hit_rect);
    let is_pressed = is_pressed(list, radio.action.as_deref(), hit_rect);
    let outer = Rect::new(bounds.x, bounds.y + 3.0, 20.0, 20.0);
    let visual_outer = pressed_rect(outer, is_pressed && !radio.disabled);
    rounded(
        list,
        visual_outer,
        theme.radius.pill,
        if is_hovered && !radio.disabled {
            lighter(theme.colors.control, 0.10)
        } else if is_pressed && !radio.disabled {
            darker(theme.colors.control, 0.08)
        } else {
            theme.colors.control
        },
    );
    if radio.selected {
        rounded(
            list,
            Rect::new(visual_outer.x + 5.0, visual_outer.y + 5.0, 10.0, 10.0),
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
        hit_rect,
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
        min_hit_rect(bounds),
        slider.action.as_deref(),
        !slider.disabled,
    );
}

fn slider_ratio(value: f32, min: f32, max: f32) -> f32 {
    if max <= min {
        return 0.0;
    }
    ((value - min) / (max - min)).clamp(0.0, 1.0)
}

pub(crate) fn rounded(list: &mut DisplayList, rect: Rect, radius: f32, color: Color) {
    list.push(RoundedRectCommand {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
        radius,
        color,
    });
}

pub(crate) fn label(list: &mut DisplayList, text: &str, x: f32, y: f32, width: f32, color: Color) {
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

pub(crate) fn label_centered(list: &mut DisplayList, text: &str, rect: Rect, color: Color) {
    let line_height = 20.0;
    list.push(TextCommand {
        text: text.to_string(),
        x: rect.x,
        y: centered_label_y(rect, line_height),
        width: rect.width.max(1.0),
        height: line_height,
        size: 14.0,
        line_height,
        color,
        wrap: TextWrap::Pretty,
        align: TextAlign::Center,
        number_spacing: NumberSpacing::Proportional,
    });
}

pub(crate) fn centered_label_y(rect: Rect, line_height: f32) -> f32 {
    rect.y + (rect.height - line_height) * 0.5
}

pub(crate) fn text_width(text: &str, size: f32) -> f32 {
    text.chars().count() as f32 * size * 0.62
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
