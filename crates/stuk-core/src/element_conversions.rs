use crate::control_elements::{
    AvatarElement, BadgeElement, CardElement, CheckboxElement, ProgressBarElement, RadioElement,
    SegmentedControlElement, SliderElement, TabsElement, TooltipElement,
};
use crate::element::{
    ButtonElement, DividerElement, Element, FrameElement, IconButtonElement, ScrollViewElement,
    SidebarElement, SpacerElement, SplitViewElement, StackElement, TextElement, TextFieldElement,
    ToggleElement, ToolbarElement, WindowElement,
};
use crate::layout_elements::{FlexElement, GridElement, OverlayElement};
use crate::list_elements::VirtualListElement;
use crate::media_elements::MediaElement;
use crate::surface_elements::SurfaceElement;

impl From<WindowElement> for Element {
    fn from(value: WindowElement) -> Self {
        Self::Window(value)
    }
}

impl From<TextElement> for Element {
    fn from(value: TextElement) -> Self {
        Self::Text(value)
    }
}

impl From<ButtonElement> for Element {
    fn from(value: ButtonElement) -> Self {
        Self::Button(value)
    }
}

impl From<IconButtonElement> for Element {
    fn from(value: IconButtonElement) -> Self {
        Self::IconButton(value)
    }
}

impl From<ToggleElement> for Element {
    fn from(value: ToggleElement) -> Self {
        Self::Toggle(value)
    }
}

impl From<CheckboxElement> for Element {
    fn from(value: CheckboxElement) -> Self {
        Self::Checkbox(value)
    }
}

impl From<RadioElement> for Element {
    fn from(value: RadioElement) -> Self {
        Self::Radio(value)
    }
}

impl From<SliderElement> for Element {
    fn from(value: SliderElement) -> Self {
        Self::Slider(value)
    }
}

impl From<ProgressBarElement> for Element {
    fn from(value: ProgressBarElement) -> Self {
        Self::ProgressBar(value)
    }
}

impl From<TabsElement> for Element {
    fn from(value: TabsElement) -> Self {
        Self::Tabs(value)
    }
}

impl From<SegmentedControlElement> for Element {
    fn from(value: SegmentedControlElement) -> Self {
        Self::SegmentedControl(value)
    }
}

impl From<BadgeElement> for Element {
    fn from(value: BadgeElement) -> Self {
        Self::Badge(value)
    }
}

impl From<AvatarElement> for Element {
    fn from(value: AvatarElement) -> Self {
        Self::Avatar(value)
    }
}

impl From<CardElement> for Element {
    fn from(value: CardElement) -> Self {
        Self::Card(value)
    }
}

impl From<TooltipElement> for Element {
    fn from(value: TooltipElement) -> Self {
        Self::Tooltip(value)
    }
}

impl From<TextFieldElement> for Element {
    fn from(value: TextFieldElement) -> Self {
        Self::TextField(value)
    }
}

impl From<StackElement> for Element {
    fn from(value: StackElement) -> Self {
        Self::Stack(value)
    }
}

impl From<FlexElement> for Element {
    fn from(value: FlexElement) -> Self {
        Self::Flex(value)
    }
}

impl From<GridElement> for Element {
    fn from(value: GridElement) -> Self {
        Self::Grid(value)
    }
}

impl From<OverlayElement> for Element {
    fn from(value: OverlayElement) -> Self {
        Self::Overlay(value)
    }
}

impl From<SurfaceElement> for Element {
    fn from(value: SurfaceElement) -> Self {
        Self::Surface(value)
    }
}

impl From<MediaElement> for Element {
    fn from(value: MediaElement) -> Self {
        Self::Media(value)
    }
}

impl From<FrameElement> for Element {
    fn from(value: FrameElement) -> Self {
        Self::Frame(value)
    }
}

impl From<SpacerElement> for Element {
    fn from(value: SpacerElement) -> Self {
        Self::Spacer(value)
    }
}

impl From<DividerElement> for Element {
    fn from(value: DividerElement) -> Self {
        Self::Divider(value)
    }
}

impl From<ScrollViewElement> for Element {
    fn from(value: ScrollViewElement) -> Self {
        Self::ScrollView(value)
    }
}

impl From<VirtualListElement> for Element {
    fn from(value: VirtualListElement) -> Self {
        Self::VirtualList(value)
    }
}

impl From<SidebarElement> for Element {
    fn from(value: SidebarElement) -> Self {
        Self::Sidebar(value)
    }
}

impl From<ToolbarElement> for Element {
    fn from(value: ToolbarElement) -> Self {
        Self::Toolbar(value)
    }
}

impl From<SplitViewElement> for Element {
    fn from(value: SplitViewElement) -> Self {
        Self::SplitView(value)
    }
}
