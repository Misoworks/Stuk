use stuk_core::Element;
use stuk_layout::{Axis, EdgeInsets, Length, Rect, Size};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutMetrics {
    pub padding: EdgeInsets,
    pub margin: EdgeInsets,
    pub horizontal_gap: Option<f32>,
    pub vertical_gap: Option<f32>,
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
    pub overflow: Option<Size>,
}

pub(crate) fn layout_metrics(element: &Element, rect: Rect, intrinsic_size: Size) -> LayoutMetrics {
    let mut metrics = LayoutMetrics {
        overflow: overflow_size(rect, intrinsic_size),
        ..LayoutMetrics::default()
    };

    match element {
        Element::Window(window) => {
            metrics.width = Some(Length::Fixed(window.width as f32));
            metrics.height = Some(Length::Fixed(window.height as f32));
        }
        Element::Stack(stack) => {
            metrics.padding = stack.padding;
            set_axis_gap(&mut metrics, stack.axis, stack.spacing, 0.0);
        }
        Element::Flex(flex) => {
            metrics.padding = flex.layout.padding;
            metrics.width = Some(flex.width);
            metrics.height = Some(flex.height);
            set_axis_gap(
                &mut metrics,
                flex.layout.axis,
                flex.layout.gap,
                flex.layout.line_gap,
            );
        }
        Element::Grid(grid) => {
            metrics.padding = grid.layout.padding;
            metrics.horizontal_gap = Some(grid.layout.column_gap);
            metrics.vertical_gap = Some(grid.layout.row_gap);
            metrics.width = Some(grid.width);
            metrics.height = Some(grid.height);
        }
        Element::Surface(surface) => {
            metrics.padding = surface.padding;
            metrics.margin = surface.margin;
            metrics.width = Some(surface.width);
            metrics.height = Some(surface.height);
            metrics.min_width = surface.min_width;
            metrics.max_width = surface.max_width;
            metrics.min_height = surface.min_height;
            metrics.max_height = surface.max_height;
        }
        Element::Frame(frame) => {
            metrics.margin = frame.margin;
            metrics.width = Some(frame.width);
            metrics.height = Some(frame.height);
            metrics.min_width = frame.min_width;
            metrics.max_width = frame.max_width;
            metrics.min_height = frame.min_height;
            metrics.max_height = frame.max_height;
        }
        Element::Spacer(spacer) => {
            metrics.width = Some(spacer.width);
            metrics.height = Some(spacer.height);
        }
        Element::Media(media) => {
            metrics.width = Some(media.width);
            metrics.height = Some(media.height);
        }
        Element::ScrollView(scroll_view) => {
            metrics.width = Some(scroll_view.width);
            metrics.height = Some(scroll_view.height);
        }
        Element::VirtualList(list) => {
            metrics.width = Some(list.width);
            metrics.height = Some(Length::Fixed(list.viewport_height.max(1.0)));
        }
        Element::Sidebar(sidebar) => {
            metrics.padding = EdgeInsets::all(14.0);
            metrics.vertical_gap = Some(8.0);
            metrics.width = Some(Length::Fixed(sidebar.width));
        }
        _ => {}
    }

    metrics
}

fn set_axis_gap(metrics: &mut LayoutMetrics, axis: Axis, primary: f32, cross: f32) {
    match axis {
        Axis::Horizontal => {
            metrics.horizontal_gap = Some(primary);
            metrics.vertical_gap = Some(cross);
        }
        Axis::Vertical => {
            metrics.horizontal_gap = Some(cross);
            metrics.vertical_gap = Some(primary);
        }
        Axis::Depth => {}
    }
}

fn overflow_size(rect: Rect, intrinsic_size: Size) -> Option<Size> {
    let overflow = Size::new(
        (intrinsic_size.width - rect.width).max(0.0),
        (intrinsic_size.height - rect.height).max(0.0),
    );
    (overflow.width > 0.0 || overflow.height > 0.0).then_some(overflow)
}
