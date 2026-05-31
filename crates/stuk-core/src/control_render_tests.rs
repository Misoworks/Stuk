use stuk_actions::ActionHitRegion;
use stuk_layout::Rect;
use stuk_render::{DisplayCommand, DisplayList};
use stuk_style::{ButtonVariant, Color, Theme};

use crate::control_render::{render_button, render_icon_button};
use crate::element::{ButtonElement, IconButtonElement};

#[test]
fn icon_button_hit_region_is_at_least_forty_points() {
    let button = IconButtonElement {
        icon: "S".to_string(),
        label: "Search".to_string(),
        action: Some("notes.search".to_string()),
        disabled: false,
    };
    let mut list = DisplayList::new(Color::WINDOW);
    let mut hit_regions = Vec::new();

    render_icon_button(
        &button,
        Rect::new(0.0, 0.0, 38.0, 38.0),
        &Theme::dark(),
        &mut list,
        &mut hit_regions,
    );

    assert_eq!(hit_regions.len(), 1);
    assert!(hit_regions[0].rect.width >= 40.0);
    assert!(hit_regions[0].rect.height >= 40.0);
}

#[test]
fn pressed_button_stays_optically_aligned() {
    let button = ButtonElement {
        label: "Save".to_string(),
        variant: ButtonVariant::Primary,
        action: Some("document.save".to_string()),
        disabled: false,
        text_align: stuk_style::ControlTextAlign::Center,
    };
    let mut list = DisplayList::new(Color::WINDOW);
    list.pressed_region = Some(ActionHitRegion::region_id_for(
        Rect::new(24.0, 0.0, 72.0, 40.0),
        "document.save",
    ));
    let mut hit_regions = Vec::new();

    render_button(
        &button,
        Rect::new(0.0, 0.0, 120.0, 40.0),
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
        .expect("button should draw rounded rect");

    assert!((rounded.height - 36.0).abs() < 0.01);
}

#[test]
fn hovered_ghost_button_draws_visible_fill() {
    let button = ButtonElement {
        label: "Inspect".to_string(),
        variant: ButtonVariant::Ghost,
        action: Some("stuk.inspect".to_string()),
        disabled: false,
        text_align: stuk_style::ControlTextAlign::Center,
    };
    let mut list = DisplayList::new(Color::WINDOW);
    list.hovered_region = Some(ActionHitRegion::region_id_for(
        Rect::new(11.62, 0.0, 96.76, 40.0),
        "stuk.inspect",
    ));
    let mut hit_regions = Vec::new();

    render_button(
        &button,
        Rect::new(0.0, 0.0, 120.0, 40.0),
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
        .expect("hovered ghost button should draw a fill");

    assert!(rounded.color.a > 0.4);
}
