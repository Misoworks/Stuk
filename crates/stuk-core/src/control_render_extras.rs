use std::cell::RefCell;

use glyphon::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Wrap};
use stuk_actions::ActionHitRegion;
use stuk_layout::Rect;
use stuk_render::{DisplayList, RectCommand, RoundedRectCommand, TextCommand};
use stuk_style::{ButtonVariant, Color, TextAlign, TextWrap, Theme};

use crate::control_render::{centered_label_y, label, label_centered, rounded};
use crate::{AvatarElement, BadgeElement, ProgressBarElement, TextFieldElement, TooltipElement};

pub(crate) fn render_progress_bar(
    progress: &ProgressBarElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
) {
    let track_width = bounds.width.min(220.0);
    let track_x = bounds.x + (bounds.width - track_width).max(0.0) * 0.5;
    let label_height = if let Some(text) = &progress.label {
        label_centered(
            list,
            text,
            Rect::new(track_x, bounds.y, track_width, 20.0),
            theme.colors.text_muted,
        );
        20.0
    } else {
        0.0
    };
    let track = Rect::new(track_x, bounds.y + label_height + 6.0, track_width, 8.0);
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
        theme.resolve_color(progress.color),
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
    hit_regions: &mut Vec<ActionHitRegion>,
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
        20.0
    } else {
        0.0
    };
    let field_height = if field.multiline {
        (bounds.height.max(120.0) - label_height - 4.0).max(96.0)
    } else {
        40.0
    };
    let label_gap = if field.label.is_some() { 4.0 } else { 0.0 };
    let field_rect = Rect::new(
        bounds.x,
        bounds.y + label_height + label_gap,
        bounds.width.max(1.0),
        field_height,
    );
    let focus_action = text_field_focus_action(field);
    let mut focus_region = ActionHitRegion::new(field_rect, &focus_action);
    focus_region.enabled = !field.disabled;
    let focused =
        field.focused || list.focused_region.as_deref() == Some(focus_region.region_id.as_str());
    let hovered =
        field.background && list.hovered_region.as_deref() == Some(focus_region.region_id.as_str());
    if field.background {
        list.push(RoundedRectCommand {
            x: field_rect.x,
            y: field_rect.y,
            width: field_rect.width,
            height: field_rect.height,
            radius: theme.radius.md,
            color: if field.disabled {
                theme.colors.control.opacity(0.35)
            } else if focused {
                theme.colors.control.opacity(0.78)
            } else if hovered {
                theme.colors.control.opacity(0.64)
            } else {
                theme.colors.control.opacity(0.50)
            },
        });
        list.push(RoundedRectCommand {
            x: field_rect.x,
            y: field_rect.y,
            width: field_rect.width,
            height: field_rect.height,
            radius: theme.radius.md,
            color: theme
                .colors
                .accent
                .opacity(if focused { 0.12 } else { 0.0 }),
        });
    }
    let text = if field.text.is_empty() {
        &field.placeholder
    } else {
        &field.text
    };
    let text_color = if field.text.is_empty() {
        theme.colors.text_muted
    } else {
        theme.colors.text
    };
    let inset_x = field.padding_x;
    let inset_y = field.padding_y;
    let text_x = field_rect.x + inset_x;
    let text_y = if field.multiline {
        field_rect.y + inset_y
    } else {
        centered_label_y(field_rect, 20.0)
    };
    let text_layout = input_text_layout(
        &field.text,
        field_rect,
        field.multiline,
        field.padding_x,
        field.padding_y,
    );
    if focused
        && !field.disabled
        && let Some((anchor, focus)) = field.selection
    {
        for rect in text_layout.selection_rects(anchor.min(focus), anchor.max(focus)) {
            list.push(RoundedRectCommand {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
                radius: 3.0,
                color: Color::rgb(0.0, 0.0, 0.0).opacity(0.24),
            });
        }
    }
    if field.multiline {
        list.push(TextCommand {
            text: text.to_string(),
            x: text_x,
            y: text_y,
            width: (field_rect.width - inset_x * 2.0).max(1.0),
            height: (field_rect.height - inset_y * 2.0).max(TEXT_FIELD_LINE_HEIGHT),
            size: 14.0,
            line_height: TEXT_FIELD_LINE_HEIGHT,
            color: text_color,
            wrap: TextWrap::Pretty,
            align: TextAlign::Start,
            number_spacing: stuk_style::NumberSpacing::Proportional,
        });
    } else {
        label(
            list,
            text,
            text_x,
            text_y,
            (field_rect.width - inset_x * 2.0).max(1.0),
            text_color,
        );
    }
    if focused && !field.disabled {
        let caret = field.caret.unwrap_or_else(|| field.text.chars().count());
        let (caret_x, caret_y) = text_layout.caret(caret);
        list.push(RectCommand {
            x: caret_x,
            y: caret_y,
            width: 1.25,
            height: 20.0,
            color: theme.colors.accent,
        });
    }
    hit_regions.push(focus_region);
    push_text_caret_regions(
        hit_regions,
        &focus_action,
        &field.text,
        field_rect,
        field.padding_x,
        &text_layout,
    );
}

const TEXT_FIELD_LINE_HEIGHT: f32 = 20.0;

thread_local! {
    static TEXT_LAYOUT_FONT_SYSTEM: RefCell<FontSystem> = RefCell::new(FontSystem::new());
}

fn push_text_caret_regions(
    hit_regions: &mut Vec<ActionHitRegion>,
    focus_action: &str,
    text: &str,
    rect: Rect,
    inset_x: f32,
    layout: &InputTextLayout,
) {
    let Some(field_id) = focus_action.strip_prefix("__stuk.input.focus.") else {
        return;
    };
    let count = text.chars().count();
    for index in 0..=count {
        let (caret_x, caret_y) = layout.caret(index);
        let previous = index
            .checked_sub(1)
            .map(|previous_index| layout.caret(previous_index));
        let next = (index < count).then(|| layout.caret(index + 1));
        let line_left = rect.x + inset_x;
        let line_right = rect.x + rect.width - inset_x;
        let left = previous
            .filter(|(_, y)| (*y - caret_y).abs() < 0.5)
            .map(|(x, _)| (x + caret_x) * 0.5)
            .unwrap_or(line_left);
        let right = next
            .filter(|(_, y)| (*y - caret_y).abs() < 0.5)
            .map(|(x, _)| (x + caret_x) * 0.5)
            .unwrap_or(line_right);
        let x = left.min(caret_x - 5.0).max(line_left);
        let width = (right.max(caret_x + 5.0).min(line_right) - x).max(10.0);
        let action = format!("__stuk.input.caret.{field_id}.{index}");
        hit_regions.push(ActionHitRegion::new(
            Rect::new(x, caret_y, width, TEXT_FIELD_LINE_HEIGHT),
            action,
        ));
    }
}

#[derive(Clone, Debug)]
struct InputTextLayout {
    rect: Rect,
    inset_x: f32,
    inset_y: f32,
    char_positions: Vec<(f32, f32)>,
    lines: Vec<InputTextLine>,
}

#[derive(Clone, Debug)]
struct InputTextLine {
    start: usize,
    end: usize,
    y: f32,
}

impl InputTextLayout {
    fn caret(&self, caret: usize) -> (f32, f32) {
        let index = caret.min(self.char_positions.len().saturating_sub(1));
        self.char_positions
            .get(index)
            .copied()
            .unwrap_or((self.rect.x + self.inset_x, self.rect.y + self.inset_y))
    }

    fn selection_rects(&self, start: usize, end: usize) -> Vec<Rect> {
        let mut rects = Vec::new();
        for line in &self.lines {
            let selection_start = start.max(line.start).min(line.end);
            let selection_end = end.max(line.start).min(line.end);
            if selection_start < selection_end {
                let (start_x, _) = self.caret(selection_start);
                let (end_x, _) = self.caret(selection_end);
                rects.push(Rect::new(
                    start_x.min(end_x),
                    line.y,
                    (end_x - start_x).abs().max(1.0),
                    TEXT_FIELD_LINE_HEIGHT,
                ));
            }
        }
        rects
    }
}

fn input_text_layout(
    text: &str,
    rect: Rect,
    multiline: bool,
    inset_x: f32,
    inset_y: f32,
) -> InputTextLayout {
    let count = text.chars().count();
    let fallback_y = if multiline {
        rect.y + inset_y
    } else {
        centered_label_y(rect, TEXT_FIELD_LINE_HEIGHT)
    };
    let mut layout = InputTextLayout {
        rect,
        inset_x,
        inset_y,
        char_positions: vec![(rect.x + inset_x, fallback_y); count + 1],
        lines: Vec::new(),
    };
    if text.is_empty() {
        layout.lines.push(InputTextLine {
            start: 0,
            end: 0,
            y: fallback_y,
        });
        return layout;
    }

    TEXT_LAYOUT_FONT_SYSTEM.with_borrow_mut(|font_system| {
        let mut buffer = Buffer::new(font_system, Metrics::new(14.0, TEXT_FIELD_LINE_HEIGHT));
        let width = (rect.width - inset_x * 2.0).max(1.0);
        let height = (rect.height - inset_y * 2.0).max(TEXT_FIELD_LINE_HEIGHT);
        buffer.set_size(
            font_system,
            Some(width),
            Some(if multiline {
                height
            } else {
                TEXT_FIELD_LINE_HEIGHT
            }),
        );
        buffer.set_wrap(
            font_system,
            if multiline {
                Wrap::WordOrGlyph
            } else {
                Wrap::None
            },
        );
        buffer.set_text(
            font_system,
            text,
            &Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(font_system, false);

        let line_offsets = line_char_offsets(text);
        let mut source_line_y = vec![None; line_offsets.len()];
        for run in buffer.layout_runs() {
            let line_base = line_offsets.get(run.line_i).copied().unwrap_or(0);
            let y = if multiline {
                rect.y + inset_y + run.line_top
            } else {
                fallback_y
            };
            source_line_y[run.line_i] =
                Some(source_line_y[run.line_i].map_or(y, |known_y: f32| known_y.max(y)));
            let mut line_start = None;
            let mut line_end = line_base;

            for glyph in run.glyphs {
                let char_start = line_base + byte_to_char_index(run.text, glyph.start);
                let char_end = line_base + byte_to_char_index(run.text, glyph.end);
                line_start =
                    Some(line_start.map_or(char_start, |start: usize| start.min(char_start)));
                line_end = line_end.max(char_end);
                let span = char_end.saturating_sub(char_start).max(1);
                for step in 0..=span {
                    let char_index = char_start + step;
                    if char_index >= layout.char_positions.len() {
                        continue;
                    }
                    let t = step as f32 / span as f32;
                    layout.char_positions[char_index] =
                        (rect.x + inset_x + glyph.x + glyph.w * t, y);
                }
            }

            if let Some(last) = run.glyphs.last() {
                line_end = line_end.max(line_base + byte_to_char_index(run.text, last.end));
            }
            layout.lines.push(InputTextLine {
                start: line_start.unwrap_or(line_base),
                end: line_end,
                y,
            });
        }
        repair_explicit_line_carets(
            &mut layout,
            &line_offsets,
            &source_line_y,
            rect,
            inset_x,
            fallback_y,
        );
    });

    if layout.lines.is_empty() {
        layout.lines.push(InputTextLine {
            start: 0,
            end: count,
            y: fallback_y,
        });
    }

    layout
}

fn repair_explicit_line_carets(
    layout: &mut InputTextLayout,
    line_offsets: &[usize],
    source_line_y: &[Option<f32>],
    rect: Rect,
    inset_x: f32,
    fallback_y: f32,
) {
    let left = rect.x + inset_x;
    let count = layout.char_positions.len().saturating_sub(1);
    let mut previous_y = fallback_y - TEXT_FIELD_LINE_HEIGHT;
    for (line_index, offset) in line_offsets.iter().copied().enumerate() {
        if offset > count {
            continue;
        }
        let y = source_line_y
            .get(line_index)
            .and_then(|value| *value)
            .unwrap_or(previous_y + TEXT_FIELD_LINE_HEIGHT);
        previous_y = y;
        layout.char_positions[offset] = (left, y);

        let next_offset = line_offsets
            .get(line_index + 1)
            .copied()
            .unwrap_or(count + 1);
        if next_offset == offset + 1 || offset == count {
            layout.lines.push(InputTextLine {
                start: offset,
                end: offset,
                y,
            });
        }
    }
}

fn line_char_offsets(text: &str) -> Vec<usize> {
    let mut offsets = Vec::new();
    let mut offset = 0;
    for line in text.split('\n') {
        offsets.push(offset);
        offset += line.chars().count() + 1;
    }
    offsets
}

fn byte_to_char_index(text: &str, byte_index: usize) -> usize {
    text.get(..byte_index.min(text.len()))
        .unwrap_or(text)
        .chars()
        .count()
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

fn progress_ratio(progress: &ProgressBarElement) -> f32 {
    if progress.max <= 0.0 {
        return 0.0;
    }
    (progress.value / progress.max).clamp(0.0, 1.0)
}

fn text_field_focus_action(field: &TextFieldElement) -> String {
    let identity = field
        .label
        .as_deref()
        .filter(|value| !value.is_empty())
        .or_else(|| {
            if field.placeholder.is_empty() {
                None
            } else {
                Some(field.placeholder.as_str())
            }
        })
        .unwrap_or("field");
    format!("__stuk.input.focus.{identity}")
}

pub(crate) fn is_hovered(list: &DisplayList, action: Option<&str>, rect: Rect) -> bool {
    action.is_some_and(|action| {
        list.hovered_region.as_deref()
            == Some(ActionHitRegion::region_id_for(rect, action).as_str())
    })
}

pub(crate) fn is_pressed(list: &DisplayList, action: Option<&str>, rect: Rect) -> bool {
    action.is_some_and(|action| {
        list.pressed_region.as_deref()
            == Some(ActionHitRegion::region_id_for(rect, action).as_str())
    })
}

pub(crate) fn button_fill(
    theme: &Theme,
    variant: ButtonVariant,
    hovered: bool,
    pressed: bool,
    enabled: bool,
) -> Color {
    let base = variant.fill_for(theme);
    if !enabled {
        return base.opacity(0.30);
    }
    match variant {
        ButtonVariant::Ghost if pressed => theme.colors.control.opacity(0.58),
        ButtonVariant::Ghost if hovered => theme.colors.control.opacity(0.46),
        ButtonVariant::Ghost => Color::rgba(1.0, 1.0, 1.0, 0.0),
        _ if pressed => darker(base, 0.04),
        _ if hovered => lighter(base, 0.04),
        _ => base,
    }
}

pub(crate) fn lighter(color: Color, amount: f32) -> Color {
    Color::rgba(
        color.r + (1.0 - color.r) * amount,
        color.g + (1.0 - color.g) * amount,
        color.b + (1.0 - color.b) * amount,
        color.a,
    )
}

pub(crate) fn darker(color: Color, amount: f32) -> Color {
    Color::rgba(
        color.r * (1.0 - amount),
        color.g * (1.0 - amount),
        color.b * (1.0 - amount),
        color.a,
    )
}
