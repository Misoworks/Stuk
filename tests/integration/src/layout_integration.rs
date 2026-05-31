#[allow(unused_imports)]
#[allow(unused_imports)]
use stuk_layout::{Axis, EdgeInsets, Length, Rect, Size, stack_size};

#[test]
fn size_creation_and_clamping() {
    let size = Size {
        width: 800.0,
        height: 600.0,
    };
    assert_eq!(size.width, 800.0);
    assert_eq!(size.height, 600.0);
    let clamped = size.clamp_non_negative();
    assert_eq!(clamped.width, 800.0);
    let negative = Size {
        width: -10.0,
        height: 0.0,
    };
    assert_eq!(negative.clamp_non_negative().width, 0.0);
}

#[test]
fn edge_insets_layout() {
    let insets = EdgeInsets {
        top: 12.0,
        left: 16.0,
        bottom: 12.0,
        right: 16.0,
    };
    assert_eq!(insets.horizontal(), 32.0);
    assert_eq!(insets.vertical(), 24.0);
}

#[test]
fn axis_variants() {
    assert!(matches!(Axis::Horizontal, Axis::Horizontal));
    assert!(matches!(Axis::Vertical, Axis::Vertical));
}

#[test]
fn length_resolves() {
    assert!(Length::Fill.is_fill());
    assert_eq!(Length::Fixed(42.0).resolve(0.0), 42.0);
    assert_eq!(Length::Fit.resolve(100.0), 100.0);
}

#[test]
fn rect_inset() {
    let rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
    };
    let insets = EdgeInsets {
        top: 10.0,
        left: 20.0,
        bottom: 10.0,
        right: 20.0,
    };
    let inset = rect.inset(insets);
    assert_eq!(inset.x, 20.0);
    assert_eq!(inset.y, 10.0);
    assert_eq!(inset.width, 760.0);
    assert_eq!(inset.height, 580.0);
}

#[test]
fn stack_size_vertical() {
    let children = vec![
        Size {
            width: 100.0,
            height: 50.0,
        },
        Size {
            width: 80.0,
            height: 40.0,
        },
    ];
    let padding = EdgeInsets {
        top: 10.0,
        left: 10.0,
        bottom: 10.0,
        right: 10.0,
    };
    let size = stack_size(Axis::Vertical, padding, 8.0, &children);
    assert_eq!(size.width, 120.0);
    assert_eq!(size.height, 118.0);
}
