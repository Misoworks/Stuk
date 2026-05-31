pub mod input;

pub use input::{TextInputAction, TextInputManager, TextInputResolver};
pub use stuk_accessibility::{
    AccessibilityNode, AccessibilityTree, Node as AccessKitNode, Role as AccessibilityRole,
    Toggled as AccessibilityToggled,
};
pub use stuk_actions::{ActionDescriptor, ActionHitRegion, ActionRegistry, Modifiers, Shortcut};
pub use stuk_core::{
    AccessibilityDiagnostic, AccessibilityDiagnosticKind, AccessibilityDiagnosticLevel, App,
    CancellationToken, Component, ComponentState, Cx, Element, FlexChildElement, FlexElement,
    FocusDirection, FocusTarget, FocusTraversal, GridChildElement, GridElement, IntoView,
    MediaElement, MediaSource, Mutation, MutationState, NavigationSplitState, NavigationStack,
    OverlayAlignment, OverlayElement, Page, PageCursor, PageId, PaginatedResource,
    PaginatedResourcePhase, PaginatedResourceSnapshot, PaginationCxExt, PaginationMode, Resource,
    ResourceState, Result, RouteState, Screen, SessionCx, Signal, StaccatoCx, StukError,
    SurfaceBorder, SurfaceElement, SurfaceShadow, TaskHandle, View, cursor_resource, focus_targets,
    mutation, paginated_resource, resource, resource_value, signal, spawn_cancellable_task,
    spawn_task, validate_accessibility,
};
pub use stuk_devtools::{
    AccessibilityDiagnosticInspection, AccessibilityInspection, ActionInspection, AppInspection,
    BundlePlan, BundleTarget, CapabilityInspection, DiagnosticInspection, ElementSnapshot,
    FrameHealth, LayoutSnapshot, ManifestInspection, MaterialInspection, PerformanceOverlay,
    PerformanceSample, PermissionInspection, PlatformInspection, PreviewDescriptor, PreviewElement,
    PreviewRegistry, StaccatoBundleMetadata, WindowInspection, inspect_accessibility,
    inspect_element, inspect_layout, inspect_layout_for_window, inspect_manifest,
    inspect_manifest_with_base_dir, inspect_platform, preview,
};
pub use stuk_layout::{
    Axis, EdgeInsets, FlexAlign, FlexItem, FlexJustify, FlexLayout, FlexWrap, GridItem, GridLayout,
    GridTrack, Length, Point, Rect, Size, flex_layout, grid_layout,
};
pub use stuk_manifest as manifest;
pub use stuk_platform::{
    ClipboardData, FileDialogFilter, FileDialogMode, FileDialogOptions, FileDialogResult,
    GenericPlatform, MaterialEffect, MaterialResolution, MaterialResolver, Platform,
    PlatformCapabilities, WindowBackgroundEffect, WindowChrome, WindowHandle, WindowId,
    read_clipboard_text, write_clipboard_text,
};
pub use stuk_platform_macos::{MacosPlatform, MacosVibrancy, macos_capabilities};
pub use stuk_platform_staccato::{
    SplitHint, StaccatoPlatform, StaccatoSession, staccato_capabilities,
};
pub use stuk_platform_wayland::{WaylandPlatform, wayland_capabilities};
pub use stuk_platform_windows::{WindowsBackdrop, WindowsPlatform, windows_capabilities};
pub use stuk_render::{
    BorderCommand, ClipCommand, DisplayCommand, DisplayDamage, DisplayList, ImageCommand,
    MaterialCommand, RectCommand, RoundedRectCommand, ShadowCommand, SvgCommand, TextCommand,
    TransformCommand,
};
pub use stuk_settings::{
    SettingDefinition, SettingKind, SettingValue, SettingsSchema, SettingsStore,
};
pub use stuk_style::{
    AnimationTokens, ButtonVariant, Color, ColorTokens, Density, FontTokens, Material,
    NumberSpacing, RadiusTokens, SpacingTokens, TextAlign, TextWrap, Theme, ThemeMode,
};
pub use stuk_text::{TextComposition, TextInputState, TextRange, TextSelection};
pub use stuk_widgets::{
    Avatar, Badge, Button, Card, Center, Checkbox, ColorWell, CommandPalette, ContextMenu, Dialog,
    Divider, Dropdown, DropdownOption, EmptyState, ErrorView, Flex, Form, FormRow, Frame, Grid,
    HStack, IconButton, Image, Label, List, Menu, MenuItem, MutationView, NavigationItem,
    NavigationView, Overlay, PasswordField, Popover, ProgressBar, Radio, ResizablePane,
    ResourceView, ScrollView, SearchField, SegmentedControl, SelectableText, SettingsPage, Sidebar,
    SidebarLayout, Slider, Spacer, Spinner, SplitView, Surface, Svg, Table, TableColumn, TableRow,
    Tabs, Text, TextArea, TextEditorLite, TextField, Titlebar, Toast, ToastKind, Toggle, Toolbar,
    Tooltip, Tree, TreeNode, VStack, VirtualList, Window, ZStack,
};

#[macro_export]
macro_rules! actions {
    (
        $(
            $name:ident {
                id: $id:expr,
                label: $label:expr
                $(, $field:ident : $value:expr)*
                $(,)?
            }
        )*
    ) => {{
        vec![
            $(
                $crate::__stuk_action_descriptor!(
                    $crate::ActionDescriptor::new($id, $label)
                    $(, $field: $value)*
                )
            ),*
        ]
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __stuk_action_descriptor {
    ($action:expr) => {
        $action
    };
    ($action:expr, description: $value:expr $(, $field:ident : $rest:expr)*) => {
        $crate::__stuk_action_descriptor!($action.description($value) $(, $field: $rest)*)
    };
    ($action:expr, category: $value:expr $(, $field:ident : $rest:expr)*) => {
        $crate::__stuk_action_descriptor!($action.category($value) $(, $field: $rest)*)
    };
    ($action:expr, enabled: $value:expr $(, $field:ident : $rest:expr)*) => {
        $crate::__stuk_action_descriptor!($action.enabled($value) $(, $field: $rest)*)
    };
    ($action:expr, visible: $value:expr $(, $field:ident : $rest:expr)*) => {
        $crate::__stuk_action_descriptor!($action.visible($value) $(, $field: $rest)*)
    };
    ($action:expr, shortcut: $value:expr $(, $field:ident : $rest:expr)*) => {
        $crate::__stuk_action_descriptor!(
            $action.shortcut(
                $crate::Shortcut::parse($value).expect("actions! shortcut should parse")
            )
            $(, $field: $rest)*
        )
    };
}

pub mod prelude {
    pub use crate::{
        AccessKitNode, AccessibilityDiagnostic, AccessibilityDiagnosticInspection,
        AccessibilityDiagnosticKind, AccessibilityDiagnosticLevel, AccessibilityInspection,
        AccessibilityNode, AccessibilityRole, AccessibilityToggled, AccessibilityTree,
        ActionDescriptor, ActionInspection, ActionRegistry, AnimationTokens, App, AppInspection,
        Avatar, Badge, BorderCommand, BundlePlan, BundleTarget, Button, ButtonVariant,
        CancellationToken, CapabilityInspection, Card, Center, Checkbox, ClipCommand,
        ClipboardData, Color, ColorTokens, ColorWell, CommandPalette, Component, ComponentState,
        ContextMenu, Cx, Density, DiagnosticInspection, Dialog, DisplayCommand, DisplayDamage,
        DisplayList, Divider, Dropdown, DropdownOption, Element, ElementSnapshot, EmptyState,
        ErrorView, FileDialogFilter, FileDialogMode, FileDialogOptions, FileDialogResult, Flex,
        FlexAlign, FlexChildElement, FlexElement, FlexItem, FlexJustify, FlexLayout, FlexWrap,
        FocusDirection, FocusTarget, FocusTraversal, FontTokens, Form, FormRow, Frame, FrameHealth,
        GenericPlatform, Grid, GridChildElement, GridElement, GridItem, GridLayout, GridTrack,
        HStack, IconButton, Image, ImageCommand, IntoView, Label, LayoutSnapshot, Length, List,
        MacosPlatform, MacosVibrancy, ManifestInspection, Material, MaterialCommand,
        MaterialEffect, MaterialInspection, MaterialResolution, MaterialResolver, MediaElement,
        MediaSource, Menu, MenuItem, Modifiers, Mutation, MutationState, MutationView,
        NavigationItem, NavigationSplitState, NavigationStack, NavigationView, NumberSpacing,
        Overlay, OverlayAlignment, OverlayElement, Page, PageCursor, PageId, PaginatedResource,
        PaginatedResourcePhase, PaginatedResourceSnapshot, PaginationCxExt, PaginationMode,
        PasswordField, PerformanceOverlay, PerformanceSample, PermissionInspection, Platform,
        PlatformCapabilities, PlatformInspection, Popover, PreviewDescriptor, PreviewElement,
        PreviewRegistry, ProgressBar, Radio, RadiusTokens, RectCommand, ResizablePane, Resource,
        ResourceState, ResourceView, Result, RoundedRectCommand, RouteState, Screen, ScrollView,
        SearchField, SegmentedControl, SelectableText, SessionCx, SettingDefinition, SettingKind,
        SettingValue, SettingsPage, SettingsSchema, SettingsStore, ShadowCommand, Shortcut,
        Sidebar, SidebarLayout, Signal, Slider, Spacer, SpacingTokens, Spinner, SplitHint,
        SplitView, StaccatoBundleMetadata, StaccatoCx, StaccatoPlatform, StaccatoSession, Surface,
        SurfaceBorder, SurfaceElement, SurfaceShadow, Svg, SvgCommand, Table, TableColumn,
        TableRow, Tabs, TaskHandle, Text, TextAlign, TextArea, TextCommand, TextComposition,
        TextEditorLite, TextField, TextInputAction, TextInputManager, TextInputResolver,
        TextInputState, TextRange, TextSelection, TextWrap, Theme, ThemeMode, Titlebar, Toast,
        ToastKind, Toggle, Toolbar, Tooltip, TransformCommand, Tree, TreeNode, VStack, View,
        VirtualList, WaylandPlatform, Window, WindowBackgroundEffect, WindowChrome, WindowHandle,
        WindowId, WindowInspection, WindowsBackdrop, WindowsPlatform, ZStack, actions,
        cursor_resource, flex_layout, focus_targets, grid_layout, inspect_accessibility,
        inspect_element, inspect_layout, inspect_layout_for_window, inspect_manifest,
        inspect_manifest_with_base_dir, inspect_platform, macos_capabilities, mutation,
        paginated_resource, preview, read_clipboard_text, resource, resource_value, signal,
        spawn_cancellable_task, spawn_task, staccato_capabilities, validate_accessibility,
        wayland_capabilities, windows_capabilities, write_clipboard_text,
    };
}
