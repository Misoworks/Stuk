mod backend;
mod integration;
mod material;
mod session;
mod wayland_background_effect;

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use stuk_accessibility::AccessibilityTree;
use stuk_actions::{ActionHitRegion, Modifiers as StukModifiers, Shortcut};
use stuk_layout::Size;
use stuk_render::{
    BorderCommand, DisplayCommand, DisplayList, GpuRenderer, RectCommand, RendererError,
    RoundedRectCommand, ShadowCommand, TextCommand,
};
use thiserror::Error;
use winit::{
    application::ApplicationHandler,
    cursor::{Cursor, CursorIcon},
    dpi::LogicalSize,
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, ModifiersState},
    window::{Window, WindowAttributes, WindowId as WinitWindowId, WindowLevel},
};

pub use backend::{
    AppTarget, BackendDescriptor, BackendKind, BackendStatus, PlatformFamily, PlatformOs,
    PlatformOverride, PlatformOverrideKind, PlatformOverrideRegistry, RuntimeTarget, TargetSet,
    current_desktop_os, current_native_backend,
};
pub use integration::{
    AutostartEntry, CredentialKey, CredentialSecret, DeepLinkRegistration, FileDialogFilter,
    FileDialogMode, FileDialogOptions, FileDialogResult, GenericPlatform, GlobalShortcutActivation,
    GlobalShortcutRegistration, NativeMessagingHost, Platform, PlatformEvent,
    SingleInstanceActivation, SingleInstancePolicy, TrayActivation, TrayIcon, TrayMenuItem,
    WindowHandle, WindowId,
};
pub use material::{MaterialEffect, MaterialResolution, MaterialResolver};
pub use session::{SplitHint, StaccatoSession};
pub use wayland_background_effect::WaylandEffect as WindowEffect;

pub type NativeActionHandler = Arc<dyn Fn(&str)>;
pub type NativeScrollHandler = Arc<dyn Fn(f32, f32, f32, f32)>; // x, y, delta_x, delta_y

pub fn request_window_effect(
    window: &Arc<dyn Window>,
    options: &WindowOptions,
) -> Option<WindowEffect> {
    wayland_background_effect::request(window, options)
}

const ACTION_WINDOW_CLOSE: &str = "window.close";
const ACTION_WINDOW_MINIMIZE: &str = "window.minimize";
const ACTION_WINDOW_TOGGLE_MAXIMIZE: &str = "window.toggle-maximize";
const ACTION_INPUT_FOCUS_PREFIX: &str = "__stuk.input.focus";
const ACTION_INPUT_CARET_PREFIX: &str = "__stuk.input.caret.";
const ACTION_INPUT_CARET_DOWN_PREFIX: &str = "__stuk.input.caret_down.";
const ACTION_INPUT_CARET_DRAG_PREFIX: &str = "__stuk.input.caret_drag.";
const ACTION_INPUT_CARET_UP_PREFIX: &str = "__stuk.input.caret_up.";
const ACTION_INPUT_WORD_PREFIX: &str = "__stuk.input.word.";
const ANIMATION_MS: f32 = 140.0;
const DOUBLE_CLICK_MS: u128 = 420;
const CARET_BLINK_MS: u64 = 500;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClipboardData {
    Text(String),
}

impl ClipboardData {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }

    pub fn as_text(&self) -> &str {
        match self {
            Self::Text(text) => text,
        }
    }

    pub fn into_text(self) -> String {
        match self {
            Self::Text(text) => text,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.as_text().is_empty()
    }
}

pub fn read_clipboard_text() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        None
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        arboard::Clipboard::new()
            .and_then(|mut clipboard| clipboard.get_text())
            .ok()
    }
}

pub fn write_clipboard_text(text: &str) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = text;
        false
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        arboard::Clipboard::new()
            .and_then(|mut clipboard| clipboard.set_text(text.to_string()))
            .is_ok()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WindowChrome {
    #[default]
    System,
    Stuk,
    Compact,
    Sidebar,
    None,
}

impl WindowChrome {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "system" => Some(Self::System),
            "stuk" => Some(Self::Stuk),
            "compact" => Some(Self::Compact),
            "sidebar" => Some(Self::Sidebar),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Stuk => "stuk",
            Self::Compact => "compact",
            Self::Sidebar => "sidebar",
            Self::None => "none",
        }
    }

    pub fn uses_native_decorations(self) -> bool {
        matches!(self, Self::System)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WindowBackgroundEffect {
    #[default]
    None,
    Blur,
    Luca,
    Niko,
    Maris,
    Acrylic,
    Mica,
    MicaAlt,
    Vibrancy,
    HudWindow,
    Sidebar,
    UnderWindowBackground,
}

impl WindowBackgroundEffect {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "none" => Some(Self::None),
            "blur" => Some(Self::Blur),
            "luca" => Some(Self::Luca),
            "niko" => Some(Self::Niko),
            "maris" => Some(Self::Maris),
            "acrylic" => Some(Self::Acrylic),
            "mica" => Some(Self::Mica),
            "mica-alt" => Some(Self::MicaAlt),
            "vibrancy" => Some(Self::Vibrancy),
            "hud-window" => Some(Self::HudWindow),
            "sidebar" => Some(Self::Sidebar),
            "under-window-background" => Some(Self::UnderWindowBackground),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Blur => "blur",
            Self::Luca => "luca",
            Self::Niko => "niko",
            Self::Maris => "maris",
            Self::Acrylic => "acrylic",
            Self::Mica => "mica",
            Self::MicaAlt => "mica-alt",
            Self::Vibrancy => "vibrancy",
            Self::HudWindow => "hud-window",
            Self::Sidebar => "sidebar",
            Self::UnderWindowBackground => "under-window-background",
        }
    }

    pub fn requires_transparency(self) -> bool {
        !matches!(self, Self::None)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlatformCapabilities {
    pub native_windows: bool,
    pub web_surface: bool,
    pub mobile_shell: bool,
    pub native_bridge: bool,
    pub live_blur: bool,
    pub transparent_windows: bool,
    pub wallpaper_material: bool,
    pub touch_input: bool,
    pub pointer_input: bool,
    pub keyboard_input: bool,
    pub file_dialogs: bool,
    pub shell_tabs: bool,
    pub command_palette: bool,
    pub workspace_sessions: bool,
    pub native_notifications: bool,
    pub tray_icons: bool,
    pub autostart: bool,
    pub global_shortcuts: bool,
    pub deep_links: bool,
    pub single_instance: bool,
    pub native_messaging: bool,
    pub secure_storage: bool,
    pub credential_storage: bool,
    pub system_dark_mode: bool,
    pub high_contrast: bool,
}

impl PlatformCapabilities {
    pub fn generic() -> Self {
        Self {
            native_windows: false,
            web_surface: false,
            mobile_shell: false,
            native_bridge: false,
            live_blur: false,
            transparent_windows: false,
            wallpaper_material: false,
            touch_input: false,
            pointer_input: true,
            keyboard_input: true,
            file_dialogs: false,
            shell_tabs: false,
            command_palette: false,
            workspace_sessions: false,
            native_notifications: false,
            tray_icons: false,
            autostart: false,
            global_shortcuts: false,
            deep_links: false,
            single_instance: false,
            native_messaging: false,
            secure_storage: false,
            credential_storage: false,
            system_dark_mode: true,
            high_contrast: false,
        }
    }

    pub fn desktop_linux(live_blur: bool, transparent_windows: bool) -> Self {
        Self {
            native_windows: true,
            web_surface: false,
            mobile_shell: false,
            native_bridge: true,
            live_blur,
            transparent_windows,
            wallpaper_material: false,
            touch_input: false,
            pointer_input: true,
            keyboard_input: true,
            file_dialogs: true,
            shell_tabs: false,
            command_palette: false,
            workspace_sessions: false,
            native_notifications: true,
            tray_icons: true,
            autostart: true,
            global_shortcuts: true,
            deep_links: true,
            single_instance: true,
            native_messaging: true,
            secure_storage: true,
            credential_storage: true,
            system_dark_mode: true,
            high_contrast: true,
        }
    }

    pub fn desktop_windows(backdrop: bool) -> Self {
        Self {
            native_windows: true,
            web_surface: false,
            mobile_shell: false,
            native_bridge: true,
            live_blur: backdrop,
            transparent_windows: backdrop,
            wallpaper_material: false,
            touch_input: false,
            pointer_input: true,
            keyboard_input: true,
            file_dialogs: true,
            shell_tabs: false,
            command_palette: false,
            workspace_sessions: false,
            native_notifications: true,
            tray_icons: true,
            autostart: true,
            global_shortcuts: true,
            deep_links: true,
            single_instance: true,
            native_messaging: true,
            secure_storage: true,
            credential_storage: true,
            system_dark_mode: true,
            high_contrast: true,
        }
    }

    pub fn desktop_macos(vibrancy: bool) -> Self {
        Self {
            native_windows: true,
            web_surface: false,
            mobile_shell: false,
            native_bridge: true,
            live_blur: vibrancy,
            transparent_windows: vibrancy,
            wallpaper_material: vibrancy,
            touch_input: false,
            pointer_input: true,
            keyboard_input: true,
            file_dialogs: true,
            shell_tabs: false,
            command_palette: false,
            workspace_sessions: false,
            native_notifications: true,
            tray_icons: true,
            autostart: true,
            global_shortcuts: true,
            deep_links: true,
            single_instance: true,
            native_messaging: true,
            secure_storage: true,
            credential_storage: true,
            system_dark_mode: true,
            high_contrast: true,
        }
    }

    pub fn mobile_android() -> Self {
        Self {
            native_windows: false,
            web_surface: false,
            mobile_shell: true,
            native_bridge: true,
            live_blur: false,
            transparent_windows: false,
            wallpaper_material: false,
            touch_input: true,
            pointer_input: true,
            keyboard_input: true,
            file_dialogs: false,
            shell_tabs: false,
            command_palette: false,
            workspace_sessions: false,
            native_notifications: true,
            tray_icons: false,
            autostart: false,
            global_shortcuts: false,
            deep_links: true,
            single_instance: false,
            native_messaging: false,
            secure_storage: true,
            credential_storage: true,
            system_dark_mode: true,
            high_contrast: true,
        }
    }

    pub fn mobile_ios() -> Self {
        Self {
            native_windows: false,
            web_surface: false,
            mobile_shell: true,
            native_bridge: true,
            live_blur: true,
            transparent_windows: false,
            wallpaper_material: true,
            touch_input: true,
            pointer_input: true,
            keyboard_input: true,
            file_dialogs: false,
            shell_tabs: false,
            command_palette: false,
            workspace_sessions: false,
            native_notifications: true,
            tray_icons: false,
            autostart: false,
            global_shortcuts: false,
            deep_links: true,
            single_instance: false,
            native_messaging: false,
            secure_storage: true,
            credential_storage: true,
            system_dark_mode: true,
            high_contrast: true,
        }
    }

    pub fn browser_web() -> Self {
        Self {
            native_windows: false,
            web_surface: true,
            mobile_shell: false,
            native_bridge: false,
            live_blur: false,
            transparent_windows: false,
            wallpaper_material: false,
            touch_input: true,
            pointer_input: true,
            keyboard_input: true,
            file_dialogs: true,
            shell_tabs: false,
            command_palette: false,
            workspace_sessions: false,
            native_notifications: false,
            tray_icons: false,
            autostart: false,
            global_shortcuts: false,
            deep_links: true,
            single_instance: false,
            native_messaging: false,
            secure_storage: false,
            credential_storage: false,
            system_dark_mode: true,
            high_contrast: true,
        }
    }

    pub fn cef_webview() -> Self {
        Self {
            native_windows: true,
            web_surface: true,
            mobile_shell: false,
            native_bridge: true,
            live_blur: true,
            transparent_windows: true,
            wallpaper_material: false,
            touch_input: false,
            pointer_input: true,
            keyboard_input: true,
            file_dialogs: true,
            shell_tabs: false,
            command_palette: false,
            workspace_sessions: false,
            native_notifications: true,
            tray_icons: true,
            autostart: true,
            global_shortcuts: true,
            deep_links: true,
            single_instance: true,
            native_messaging: true,
            secure_storage: true,
            credential_storage: true,
            system_dark_mode: true,
            high_contrast: true,
        }
    }

    pub fn supports_background_effect(self, effect: WindowBackgroundEffect) -> bool {
        match effect {
            WindowBackgroundEffect::None => true,
            WindowBackgroundEffect::Blur
            | WindowBackgroundEffect::Luca
            | WindowBackgroundEffect::Niko => self.live_blur && self.transparent_windows,
            WindowBackgroundEffect::Maris => self.wallpaper_material && self.transparent_windows,
            WindowBackgroundEffect::Acrylic
            | WindowBackgroundEffect::Mica
            | WindowBackgroundEffect::MicaAlt
            | WindowBackgroundEffect::Vibrancy
            | WindowBackgroundEffect::HudWindow
            | WindowBackgroundEffect::Sidebar
            | WindowBackgroundEffect::UnderWindowBackground => {
                self.live_blur && self.transparent_windows
            }
        }
    }
}

impl Default for PlatformCapabilities {
    fn default() -> Self {
        Self::generic()
    }
}

#[derive(Clone, Debug)]
pub struct NativeFrame {
    pub display_list: DisplayList,
    pub hit_regions: Vec<ActionHitRegion>,
    pub accessibility_tree: AccessibilityTree,
    pub hovered_id: Option<String>,
    pub pressed_id: Option<String>,
    pub continuous_redraw: bool,
}

impl NativeFrame {
    pub fn new(display_list: DisplayList) -> Self {
        Self {
            display_list,
            hit_regions: Vec::new(),
            accessibility_tree: AccessibilityTree::empty(),
            hovered_id: None,
            pressed_id: None,
            continuous_redraw: false,
        }
    }
}

impl From<DisplayList> for NativeFrame {
    fn from(display_list: DisplayList) -> Self {
        Self::new(display_list)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowRegionRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl WindowRegionRect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width: width.max(0),
            height: height.max(0),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.width <= 0 || self.height <= 0
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WindowRegion {
    pub rects: Vec<WindowRegionRect>,
    pub adaptive: Option<WindowRegionAdaptive>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WindowRegionAdaptive {
    Full,
    RoundedRect {
        radius: i32,
    },
    RoundedLeft {
        width: i32,
        radius: i32,
    },
    TitlebarAndSidebar {
        sidebar_width: i32,
        titlebar_height: i32,
        radius: i32,
    },
    ContentAfterSidebar {
        sidebar_width: i32,
        titlebar_height: i32,
    },
}

impl WindowRegion {
    pub fn empty() -> Self {
        Self {
            rects: Vec::new(),
            adaptive: None,
        }
    }

    pub fn rect(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self::empty().add_rect(x, y, width, height)
    }

    pub fn full(width: i32, height: i32) -> Self {
        Self::rect(0, 0, width, height)
    }

    pub fn adaptive_full() -> Self {
        Self {
            rects: Vec::new(),
            adaptive: Some(WindowRegionAdaptive::Full),
        }
    }

    pub fn rounded_rect(width: i32, height: i32, radius: i32) -> Self {
        let mut region = Self::empty();
        let radius = radius.min(width / 2).min(height / 2).max(0);
        if radius == 0 {
            return Self::full(width, height);
        }

        for y in 0..height.max(0) {
            let inset = rounded_region_row_inset(y, height, radius);
            region = region.add_rect(inset, y, width - inset * 2, 1);
        }
        region
    }

    pub fn adaptive_rounded_rect(radius: i32) -> Self {
        Self {
            rects: Vec::new(),
            adaptive: Some(WindowRegionAdaptive::RoundedRect { radius }),
        }
    }

    pub fn rounded_left(width: i32, height: i32, radius: i32) -> Self {
        let mut region = Self::empty();
        let radius = radius.min(width).min(height / 2).max(0);
        if radius == 0 {
            return Self::full(width, height);
        }

        for y in 0..height.max(0) {
            let inset = rounded_region_row_inset(y, height, radius);
            region = region.add_rect(inset, y, width - inset, 1);
        }
        region
    }

    pub fn adaptive_rounded_left(width: i32, radius: i32) -> Self {
        Self {
            rects: Vec::new(),
            adaptive: Some(WindowRegionAdaptive::RoundedLeft { width, radius }),
        }
    }

    pub fn adaptive_titlebar_sidebar(
        sidebar_width: i32,
        titlebar_height: i32,
        radius: i32,
    ) -> Self {
        Self {
            rects: Vec::new(),
            adaptive: Some(WindowRegionAdaptive::TitlebarAndSidebar {
                sidebar_width,
                titlebar_height,
                radius,
            }),
        }
    }

    pub fn adaptive_content_after_sidebar(sidebar_width: i32, titlebar_height: i32) -> Self {
        Self {
            rects: Vec::new(),
            adaptive: Some(WindowRegionAdaptive::ContentAfterSidebar {
                sidebar_width,
                titlebar_height,
            }),
        }
    }

    pub fn add_rect(mut self, x: i32, y: i32, width: i32, height: i32) -> Self {
        let rect = WindowRegionRect::new(x, y, width, height);
        if !rect.is_empty() {
            self.rects.push(rect);
        }
        self
    }

    pub fn is_empty(&self) -> bool {
        self.rects.is_empty() && self.adaptive.is_none()
    }

    pub fn resolved_rects(&self, width: i32, height: i32) -> Vec<WindowRegionRect> {
        match self.adaptive {
            Some(WindowRegionAdaptive::Full) => WindowRegion::full(width, height).rects,
            Some(WindowRegionAdaptive::RoundedRect { radius }) => {
                WindowRegion::rounded_rect(width, height, radius).rects
            }
            Some(WindowRegionAdaptive::RoundedLeft {
                width: sidebar_width,
                radius,
            }) => WindowRegion::rounded_left(sidebar_width, height, radius).rects,
            Some(WindowRegionAdaptive::TitlebarAndSidebar {
                sidebar_width,
                titlebar_height,
                radius,
            }) => {
                WindowRegion::titlebar_sidebar(
                    width,
                    height,
                    sidebar_width,
                    titlebar_height,
                    radius,
                )
                .rects
            }
            Some(WindowRegionAdaptive::ContentAfterSidebar {
                sidebar_width,
                titlebar_height,
            }) => {
                WindowRegion::content_after_sidebar(width, height, sidebar_width, titlebar_height)
                    .rects
            }
            None => self.rects.clone(),
        }
    }

    fn titlebar_sidebar(
        width: i32,
        height: i32,
        sidebar_width: i32,
        titlebar_height: i32,
        radius: i32,
    ) -> Self {
        let mut region = Self::empty();
        let width = width.max(0);
        let height = height.max(0);
        let sidebar_width = sidebar_width.clamp(0, width);
        let titlebar_height = titlebar_height.clamp(0, height);
        let radius = radius.min(width / 2).min(height / 2).max(0);

        for y in 0..titlebar_height {
            let inset = if radius > 0 && y < radius {
                rounded_region_row_inset(y, radius * 2, radius)
            } else {
                0
            };
            region = region.add_rect(inset, y, width - inset * 2, 1);
        }

        for y in titlebar_height..height {
            let inset = if radius > 0 && y >= height - radius {
                rounded_region_row_inset(y, height, radius)
            } else {
                0
            };
            region = region.add_rect(inset, y, sidebar_width - inset, 1);
        }

        region
    }

    fn content_after_sidebar(
        width: i32,
        height: i32,
        sidebar_width: i32,
        titlebar_height: i32,
    ) -> Self {
        let width = width.max(0);
        let height = height.max(0);
        let sidebar_width = sidebar_width.clamp(0, width);
        let titlebar_height = titlebar_height.clamp(0, height);
        Self::rect(
            sidebar_width,
            titlebar_height,
            width - sidebar_width,
            height - titlebar_height,
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WindowRegions {
    pub blur: Option<WindowRegion>,
    pub opaque: Option<WindowRegion>,
    pub input: Option<WindowRegion>,
}

impl WindowRegions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn blur(mut self, region: WindowRegion) -> Self {
        self.blur = Some(region);
        self
    }

    pub fn opaque(mut self, region: WindowRegion) -> Self {
        self.opaque = Some(region);
        self
    }

    pub fn input(mut self, region: WindowRegion) -> Self {
        self.input = Some(region);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.blur.as_ref().is_none_or(WindowRegion::is_empty)
            && self.opaque.as_ref().is_none_or(WindowRegion::is_empty)
            && self.input.as_ref().is_none_or(WindowRegion::is_empty)
    }

    pub fn rounded_window(width: i32, height: i32, radius: i32) -> Self {
        let region = WindowRegion::rounded_rect(width, height, radius);
        Self::new().input(region)
    }

    pub fn adaptive_rounded_window(radius: i32) -> Self {
        Self::new().input(WindowRegion::adaptive_rounded_rect(radius))
    }

    pub fn rounded_sidebar(sidebar_width: i32, height: i32, radius: i32) -> Self {
        Self::new().blur(WindowRegion::rounded_left(sidebar_width, height, radius))
    }

    pub fn adaptive_rounded_sidebar(sidebar_width: i32, radius: i32) -> Self {
        Self::new().blur(WindowRegion::adaptive_rounded_left(sidebar_width, radius))
    }
}

fn rounded_region_row_inset(y: i32, height: i32, radius: i32) -> i32 {
    let top = y < radius;
    let bottom = y >= height - radius;
    if !top && !bottom {
        return 0;
    }

    let center_y = if top { radius } else { height - radius - 1 };
    let dy = (y - center_y).abs() as f64;
    let radius = radius as f64;
    (radius - (radius * radius - dy * dy).max(0.0).sqrt()).ceil() as i32
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowOptions {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub min_width: u32,
    pub min_height: u32,
    pub chrome: WindowChrome,
    pub resizable: bool,
    pub visible: bool,
    pub active: bool,
    pub always_on_top: bool,
    pub transparent: bool,
    pub background_effect: WindowBackgroundEffect,
    pub regions: WindowRegions,
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            title: "Stuk".to_string(),
            width: 760,
            height: 520,
            min_width: 420,
            min_height: 280,
            chrome: WindowChrome::System,
            resizable: true,
            visible: true,
            active: true,
            always_on_top: false,
            transparent: false,
            background_effect: WindowBackgroundEffect::None,
            regions: WindowRegions::default(),
        }
    }
}

impl WindowOptions {
    pub fn resolved_for_capabilities(mut self, capabilities: PlatformCapabilities) -> Self {
        if !capabilities.supports_background_effect(self.background_effect) {
            self.background_effect = WindowBackgroundEffect::None;
            self.regions.blur = None;
        }
        if self.transparent && !capabilities.transparent_windows {
            self.transparent = false;
        }
        if self.background_effect.requires_transparency() {
            self.transparent = self.transparent && capabilities.transparent_windows;
        }
        if !self.transparent {
            self.regions.blur = None;
        }
        self
    }
}

#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("failed to create event loop: {0}")]
    EventLoop(String),
    #[error("failed to create window: {0}")]
    Window(String),
    #[error("renderer failed: {0}")]
    Renderer(#[from] RendererError),
}

pub struct NativeApp<F>
where
    F: Fn(Size, Option<&str>, Option<&str>, Option<&str>) -> NativeFrame + 'static,
{
    backend: BackendDescriptor,
    options: WindowOptions,
    render: F,
    action_handler: Option<NativeActionHandler>,
    scroll_handler: Option<NativeScrollHandler>,
    shortcuts: Vec<(Shortcut, String)>,
    state: Option<NativeState>,
}

impl<F> NativeApp<F>
where
    F: Fn(Size, Option<&str>, Option<&str>, Option<&str>) -> NativeFrame + 'static,
{
    pub fn new(options: WindowOptions, render: F) -> Self {
        Self {
            backend: BackendDescriptor::current_native(),
            options: options.clone(),
            render,
            action_handler: None,
            scroll_handler: None,
            shortcuts: Vec::new(),
            state: None,
        }
    }

    pub fn backend(mut self, backend: BackendDescriptor) -> Self {
        self.backend = backend;
        self
    }

    pub fn shortcuts(mut self, shortcuts: Vec<(Shortcut, String)>) -> Self {
        self.shortcuts = shortcuts;
        self
    }

    pub fn on_action(mut self, handler: NativeActionHandler) -> Self {
        self.action_handler = Some(handler);
        self
    }

    pub fn on_scroll(mut self, handler: NativeScrollHandler) -> Self {
        self.scroll_handler = Some(handler);
        self
    }

    pub fn run(self) -> Result<(), PlatformError> {
        let event_loop =
            EventLoop::new().map_err(|error| PlatformError::EventLoop(error.to_string()))?;
        event_loop
            .run_app(NativeHandler { app: self })
            .map_err(|error| PlatformError::EventLoop(error.to_string()))
    }
}

struct NativeHandler<F>
where
    F: Fn(Size, Option<&str>, Option<&str>, Option<&str>) -> NativeFrame + 'static,
{
    app: NativeApp<F>,
}

impl<F> ApplicationHandler for NativeHandler<F>
where
    F: Fn(Size, Option<&str>, Option<&str>, Option<&str>) -> NativeFrame + 'static,
{
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.app.state.is_some() {
            return;
        }

        let options = self
            .app
            .options
            .clone()
            .resolved_for_capabilities(self.app.backend.capabilities);
        let attributes = platform_window_attributes(
            WindowAttributes::default()
                .with_title(options.title.clone())
                .with_surface_size(LogicalSize::new(
                    f64::from(options.width),
                    f64::from(options.height),
                ))
                .with_min_surface_size(LogicalSize::new(
                    f64::from(options.min_width),
                    f64::from(options.min_height),
                ))
                .with_resizable(options.resizable)
                .with_decorations(options.chrome.uses_native_decorations())
                .with_visible(options.visible)
                .with_active(options.active)
                .with_window_level(if options.always_on_top {
                    WindowLevel::AlwaysOnTop
                } else {
                    WindowLevel::Normal
                })
                .with_transparent(options.transparent)
                .with_blur(options.background_effect.requires_transparency()),
        );

        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::<dyn Window>::from(window),
            Err(error) => {
                eprintln!("failed to create window: {error}");
                event_loop.exit();
                return;
            }
        };
        let renderer = match pollster::block_on(GpuRenderer::new(window.clone())) {
            Ok(renderer) => renderer,
            Err(error) => {
                eprintln!("failed to initialize renderer: {error}");
                event_loop.exit();
                return;
            }
        };
        let _background_effect = wayland_background_effect::request(&window, &options);

        self.app.state = Some(NativeState {
            window,
            renderer,
            _background_effect,
            options: options.clone(),
            chrome: options.chrome,
            last_frame: None,
            modifiers: ModifiersState::default(),
            cursor_x: 0.0,
            cursor_y: 0.0,
            hovered: None,
            pressed: None,
            focused: None,
            animation_from: None,
            animation_target: None,
            animation_started: Instant::now(),
            caret_started: Instant::now(),
            caret_next_redraw: Instant::now() + Duration::from_millis(CARET_BLINK_MS),
            cursor_icon: CursorIcon::Default,
            text_drag: None,
            last_text_click: None,
        });
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        let Some(state) = &mut self.app.state else {
            return;
        };
        if state.focused.is_none() {
            event_loop.set_control_flow(ControlFlow::Wait);
            return;
        }

        let now = Instant::now();
        if now >= state.caret_next_redraw {
            state.window.request_redraw();
            state.caret_next_redraw = now + Duration::from_millis(CARET_BLINK_MS);
        }
        set_control_flow_until(event_loop, state.caret_next_redraw);
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        id: WinitWindowId,
        event: WindowEvent,
    ) {
        let Some(state) = &mut self.app.state else {
            return;
        };

        if id != state.window.id() {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::SurfaceResized(size) => {
                state
                    .renderer
                    .resize(size.width, size.height, state.window.scale_factor() as f32);
                state.update_window_regions(size.width, size.height);
                state.window.request_redraw();
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                let size = state.window.surface_size();
                state
                    .renderer
                    .resize(size.width, size.height, scale_factor as f32);
                state.update_window_regions(size.width, size.height);
                state.window.request_redraw();
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                state.modifiers = modifiers.state();
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic: false,
                ..
            } if event.state == ElementState::Pressed => {
                if state.focused.is_some()
                    && let Some(command) = input_command_from_key_event(&event, state.modifiers)
                    && let Some(handler) = &self.app.action_handler
                {
                    handler(&command);
                    state.caret_started = Instant::now();
                    state.window.request_redraw();
                } else if !event.repeat
                    && let Some(shortcut) = shortcut_from_key_event(&event, state.modifiers)
                    && let Some(action_id) = action_for_shortcut(&self.app.shortcuts, &shortcut)
                    && let Some(handler) = &self.app.action_handler
                {
                    handler(action_id);
                    state.window.request_redraw();
                } else if state.focused.is_some()
                    && let Some(key_name) = key_name_for_input(&event, state.modifiers)
                    && let Some(handler) = &self.app.action_handler
                {
                    handler(&format!("input.key.{key_name}"));
                    state.caret_started = Instant::now();
                    state.window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                state.window.pre_present_notify();
                let mut frame = (self.app.render)(
                    state.renderer.logical_size(),
                    state.hovered.as_deref(),
                    state.pressed.as_deref(),
                    state.focused.as_deref(),
                );
                let interactive =
                    state.focused.is_some() || state.hovered.is_some() || state.pressed.is_some();
                if interactive {
                    state.animation_from = None;
                    state.animation_target = Some(frame.display_list.clone());
                } else if state.animation_target.as_ref() != Some(&frame.display_list) {
                    state.animation_from = state
                        .last_frame
                        .as_ref()
                        .map(|previous| previous.display_list.clone());
                    state.animation_target = Some(frame.display_list.clone());
                    state.animation_started = Instant::now();
                }
                if !interactive && let Some(target) = &state.animation_target {
                    let elapsed = state.animation_started.elapsed().as_secs_f32() * 1000.0;
                    let progress = (elapsed / ANIMATION_MS).clamp(0.0, 1.0);
                    if let Some(from) = &state.animation_from {
                        frame.display_list = interpolate_display_list(from, target, progress);
                    }
                    if progress < 1.0 {
                        state.window.request_redraw();
                    } else {
                        state.animation_from = None;
                    }
                }
                if state.focused.is_some() {
                    apply_caret_blink(&mut frame.display_list, state.caret_started);
                    state.caret_next_redraw =
                        next_caret_redraw(state.caret_started, Instant::now());
                    set_control_flow_until(event_loop, state.caret_next_redraw);
                }
                if frame.continuous_redraw {
                    state.window.request_redraw();
                }
                if let Err(error) = state.renderer.render(&frame.display_list) {
                    eprintln!("render failed: {error}");
                    event_loop.exit();
                    return;
                }
                state.last_frame = Some(frame);
            }
            WindowEvent::PointerButton {
                state: ElementState::Released,
                position,
                primary,
                button,
                ..
            } if primary && button.clone().mouse_button() == Some(MouseButton::Left) => {
                let scale = state.window.scale_factor() as f32;
                let x = position.x as f32 / scale;
                let y = position.y as f32 / scale;
                if let Some(field_id) = &state.text_drag
                    && let Some(hit) = nearest_caret_hit(state.last_frame.as_ref(), field_id, x, y)
                {
                    state.focused = Some(hit.region_id);
                    state.caret_started = Instant::now();
                    if let Some(handler) = &self.app.action_handler {
                        handler(
                            &hit.action_id
                                .replace(ACTION_INPUT_CARET_PREFIX, ACTION_INPUT_CARET_UP_PREFIX),
                        );
                    }
                    state.window.request_redraw();
                } else if let Some(hit) = hit_test(state.last_frame.as_ref(), x, y) {
                    if hit.action_id.starts_with(ACTION_INPUT_FOCUS_PREFIX) {
                        state.focused = Some(hit.region_id);
                        state.caret_started = Instant::now();
                        if let Some(handler) = &self.app.action_handler {
                            handler(&hit.action_id);
                        }
                        state.window.request_redraw();
                    } else if hit.action_id.starts_with(ACTION_INPUT_CARET_PREFIX) {
                        state.focused = Some(hit.region_id);
                        state.caret_started = Instant::now();
                        if let Some(handler) = &self.app.action_handler {
                            handler(
                                &hit.action_id.replace(
                                    ACTION_INPUT_CARET_PREFIX,
                                    ACTION_INPUT_CARET_UP_PREFIX,
                                ),
                            );
                        }
                        state.window.request_redraw();
                    } else if handle_builtin_action(&state.window, event_loop, &hit.action_id) {
                        state.window.request_redraw();
                    } else if let Some(handler) = &self.app.action_handler {
                        handler(&hit.action_id);
                        state.window.request_redraw();
                    }
                } else if state.focused.take().is_some() {
                    state.window.request_redraw();
                }
                state.pressed = None;
                state.text_drag = None;
            }
            WindowEvent::PointerMoved { position, .. } => {
                let scale = state.window.scale_factor() as f32;
                state.cursor_x = position.x as f32 / scale;
                state.cursor_y = position.y as f32 / scale;
                let new_hovered =
                    hit_test(state.last_frame.as_ref(), state.cursor_x, state.cursor_y)
                        .map(|hit| hit.region_id);
                let next_cursor =
                    hit_test(state.last_frame.as_ref(), state.cursor_x, state.cursor_y)
                        .map(|hit| {
                            if hit.action_id.starts_with(ACTION_INPUT_FOCUS_PREFIX)
                                || hit.action_id.starts_with(ACTION_INPUT_CARET_PREFIX)
                            {
                                CursorIcon::Text
                            } else {
                                CursorIcon::Pointer
                            }
                        })
                        .unwrap_or(CursorIcon::Default);
                if state.cursor_icon != next_cursor {
                    state.cursor_icon = next_cursor;
                    state.window.set_cursor(Cursor::Icon(next_cursor));
                }
                if state.hovered != new_hovered {
                    state.hovered = new_hovered;
                    state.window.request_redraw();
                }
                if let Some(field_id) = &state.text_drag
                    && let Some(hit) = nearest_caret_hit(
                        state.last_frame.as_ref(),
                        field_id,
                        state.cursor_x,
                        state.cursor_y,
                    )
                    && let Some(handler) = &self.app.action_handler
                {
                    handler(
                        &hit.action_id
                            .replace(ACTION_INPUT_CARET_PREFIX, ACTION_INPUT_CARET_DRAG_PREFIX),
                    );
                    state.caret_started = Instant::now();
                    state.window.request_redraw();
                }
            }
            WindowEvent::PointerButton {
                state: ElementState::Pressed,
                primary: true,
                position,
                button,
                ..
            } if button.clone().mouse_button() == Some(MouseButton::Left) => {
                let scale = state.window.scale_factor() as f32;
                let x = position.x as f32 / scale;
                let y = position.y as f32 / scale;
                if state.chrome.uses_stuk_drag_region() && y <= 38.0 {
                    if hit_test(state.last_frame.as_ref(), x, y).is_none() {
                        let _ = state.window.drag_window();
                    }
                }
                if let Some(hit) = hit_test(state.last_frame.as_ref(), x, y) {
                    if hit.action_id.starts_with(ACTION_INPUT_CARET_PREFIX) {
                        state.focused = Some(hit.region_id.clone());
                        state.text_drag = caret_field_id(&hit.action_id);
                        if let Some(handler) = &self.app.action_handler {
                            let now = Instant::now();
                            let double_click =
                                state.last_text_click.as_ref().is_some_and(|click| {
                                    click.action_id == hit.action_id
                                        && now.duration_since(click.at).as_millis()
                                            <= DOUBLE_CLICK_MS
                                });
                            if double_click {
                                handler(
                                    &hit.action_id.replace(
                                        ACTION_INPUT_CARET_PREFIX,
                                        ACTION_INPUT_WORD_PREFIX,
                                    ),
                                );
                                state.text_drag = None;
                            } else {
                                handler(&hit.action_id.replace(
                                    ACTION_INPUT_CARET_PREFIX,
                                    ACTION_INPUT_CARET_DOWN_PREFIX,
                                ));
                            }
                            state.last_text_click = Some(TextClick {
                                action_id: hit.action_id.clone(),
                                at: now,
                            });
                        }
                        state.caret_started = Instant::now();
                    } else if hit.action_id.starts_with(ACTION_INPUT_FOCUS_PREFIX) {
                        state.focused = Some(hit.region_id.clone());
                        state.text_drag = focus_field_id(&hit.action_id);
                        if let Some(handler) = &self.app.action_handler {
                            handler(&hit.action_id);
                            if let Some(field_id) = &state.text_drag
                                && let Some(caret_hit) =
                                    nearest_caret_hit(state.last_frame.as_ref(), field_id, x, y)
                            {
                                handler(&caret_hit.action_id.replace(
                                    ACTION_INPUT_CARET_PREFIX,
                                    ACTION_INPUT_CARET_DOWN_PREFIX,
                                ));
                            }
                        }
                        state.caret_started = Instant::now();
                    } else {
                        state.last_text_click = None;
                    }
                    state.pressed = Some(hit.region_id);
                } else {
                    state.pressed = None;
                    state.last_text_click = None;
                }
                state.window.request_redraw();
            }
            WindowEvent::MouseWheel { delta, phase, .. } => {
                if phase != winit::event::TouchPhase::Moved {
                    let (dx, dy) = match delta {
                        MouseScrollDelta::LineDelta(x, y) => (x * 20.0, y * 20.0),
                        MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                    };
                    if let Some(handler) = &self.app.scroll_handler {
                        handler(0.0, 0.0, dx, dy);
                        state.window.request_redraw();
                    }
                }
            }
            _ => {}
        }
    }
}

fn set_control_flow_until(event_loop: &dyn ActiveEventLoop, deadline: Instant) {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = deadline;
        event_loop.set_control_flow(ControlFlow::Wait);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        event_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
    }
}

fn platform_window_attributes(attributes: WindowAttributes) -> WindowAttributes {
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowAttributesWeb;

        attributes.with_platform_attributes(Box::new(
            WindowAttributesWeb::default()
                .with_append(true)
                .with_prevent_default(true)
                .with_focusable(true),
        ))
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        attributes
    }
}

struct NativeState {
    window: Arc<dyn Window>,
    renderer: GpuRenderer,
    _background_effect: Option<wayland_background_effect::WaylandEffect>,
    options: WindowOptions,
    chrome: WindowChrome,
    last_frame: Option<NativeFrame>,
    modifiers: ModifiersState,
    cursor_x: f32,
    cursor_y: f32,
    hovered: Option<String>,
    pressed: Option<String>,
    focused: Option<String>,
    animation_from: Option<DisplayList>,
    animation_target: Option<DisplayList>,
    animation_started: Instant,
    caret_started: Instant,
    caret_next_redraw: Instant,
    cursor_icon: CursorIcon,
    text_drag: Option<String>,
    last_text_click: Option<TextClick>,
}

impl NativeState {
    fn update_window_regions(&self, width: u32, height: u32) {
        if let Some(effect) = &self._background_effect {
            let logical_width = (width as f64 / self.window.scale_factor())
                .round()
                .clamp(1.0, f64::from(i32::MAX)) as i32;
            let logical_height = (height as f64 / self.window.scale_factor())
                .round()
                .clamp(1.0, f64::from(i32::MAX)) as i32;
            let _ = effect.update(&self.options, logical_width, logical_height);
        }
    }
}

#[derive(Clone, Debug)]
struct TextClick {
    action_id: String,
    at: Instant,
}

impl WindowChrome {
    fn uses_stuk_drag_region(self) -> bool {
        matches!(self, Self::Stuk | Self::Compact | Self::Sidebar)
    }
}

#[derive(Clone, Debug)]
struct HitTarget {
    region_id: String,
    action_id: String,
}

fn hit_test(frame: Option<&NativeFrame>, x: f32, y: f32) -> Option<HitTarget> {
    frame?
        .hit_regions
        .iter()
        .rev()
        .find(|region| region.enabled && region.contains(x, y))
        .map(|region| HitTarget {
            region_id: region.region_id.clone(),
            action_id: region.action_id.clone(),
        })
}

fn nearest_caret_hit(
    frame: Option<&NativeFrame>,
    field_id: &str,
    x: f32,
    y: f32,
) -> Option<HitTarget> {
    if let Some(hit) = hit_test(frame, x, y)
        && caret_field_id(&hit.action_id).as_deref() == Some(field_id)
    {
        return Some(hit);
    }

    frame?
        .hit_regions
        .iter()
        .filter(|region| {
            region.enabled && caret_field_id(&region.action_id).as_deref() == Some(field_id)
        })
        .min_by(|a, b| {
            let a_distance = rect_distance_squared(&a.rect, x, y);
            let b_distance = rect_distance_squared(&b.rect, x, y);
            a_distance.total_cmp(&b_distance)
        })
        .map(|region| HitTarget {
            region_id: region.region_id.clone(),
            action_id: region.action_id.clone(),
        })
}

fn rect_distance_squared(rect: &stuk_layout::Rect, x: f32, y: f32) -> f32 {
    let closest_x = x.clamp(rect.x, rect.x + rect.width);
    let closest_y = y.clamp(rect.y, rect.y + rect.height);
    let dx = x - closest_x;
    let dy = y - closest_y;
    dx * dx + dy * dy
}

fn caret_field_id(action_id: &str) -> Option<String> {
    let value = action_id
        .strip_prefix(ACTION_INPUT_CARET_PREFIX)
        .or_else(|| action_id.strip_prefix(ACTION_INPUT_CARET_DOWN_PREFIX))
        .or_else(|| action_id.strip_prefix(ACTION_INPUT_CARET_DRAG_PREFIX))
        .or_else(|| action_id.strip_prefix(ACTION_INPUT_CARET_UP_PREFIX))
        .or_else(|| action_id.strip_prefix(ACTION_INPUT_WORD_PREFIX))?;
    let (field_id, _) = value.rsplit_once('.')?;
    Some(field_id.to_string())
}

fn focus_field_id(action_id: &str) -> Option<String> {
    action_id
        .strip_prefix(ACTION_INPUT_FOCUS_PREFIX)
        .and_then(|value| value.strip_prefix('.'))
        .map(ToString::to_string)
}

fn interpolate_display_list(
    from: &DisplayList,
    target: &DisplayList,
    progress: f32,
) -> DisplayList {
    if from.commands.len() != target.commands.len() {
        return target.clone();
    }

    let eased = 1.0 - (1.0 - progress).powi(3);
    let mut list = target.clone();
    list.commands = from
        .commands
        .iter()
        .zip(target.commands.iter())
        .map(|(from, target)| interpolate_command(from, target, eased))
        .collect();
    list
}

fn interpolate_command(from: &DisplayCommand, target: &DisplayCommand, t: f32) -> DisplayCommand {
    match (from, target) {
        (DisplayCommand::Rect(from), DisplayCommand::Rect(target)) => {
            DisplayCommand::Rect(RectCommand {
                x: target.x,
                y: target.y,
                width: target.width,
                height: target.height,
                color: lerp_color(from.color, target.color, t),
            })
        }
        (DisplayCommand::RoundedRect(from), DisplayCommand::RoundedRect(target)) => {
            DisplayCommand::RoundedRect(RoundedRectCommand {
                x: target.x,
                y: target.y,
                width: target.width,
                height: target.height,
                radius: target.radius,
                color: lerp_color(from.color, target.color, t),
            })
        }
        (DisplayCommand::Border(from), DisplayCommand::Border(target)) => {
            DisplayCommand::Border(BorderCommand {
                x: target.x,
                y: target.y,
                width: target.width,
                height: target.height,
                radius: target.radius,
                thickness: target.thickness,
                color: lerp_color(from.color, target.color, t),
            })
        }
        (DisplayCommand::Shadow(from), DisplayCommand::Shadow(target)) => {
            DisplayCommand::Shadow(ShadowCommand {
                x: target.x,
                y: target.y,
                width: target.width,
                height: target.height,
                radius: target.radius,
                offset_x: target.offset_x,
                offset_y: target.offset_y,
                blur: target.blur,
                spread: target.spread,
                color: lerp_color(from.color, target.color, t),
            })
        }
        (DisplayCommand::Text(from), DisplayCommand::Text(target)) if from.text == target.text => {
            DisplayCommand::Text(TextCommand {
                text: target.text.clone(),
                x: target.x,
                y: target.y,
                width: target.width,
                height: target.height,
                size: target.size,
                line_height: target.line_height,
                color: lerp_color(from.color, target.color, t),
                wrap: target.wrap,
                align: target.align,
                number_spacing: target.number_spacing,
            })
        }
        _ => target.clone(),
    }
}

fn apply_caret_blink(list: &mut DisplayList, started: Instant) {
    let elapsed_ms = started.elapsed().as_millis() % 1000;
    let alpha = if elapsed_ms < 520 { 1.0 } else { 0.0 };
    for command in &mut list.commands {
        if let DisplayCommand::Rect(rect) = command
            && rect.width <= 2.0
            && rect.height >= 16.0
            && rect.height <= 24.0
        {
            rect.color.a *= alpha;
        }
    }
}

fn next_caret_redraw(started: Instant, now: Instant) -> Instant {
    let elapsed = now.duration_since(started);
    let blink = Duration::from_millis(CARET_BLINK_MS);
    let elapsed_ms = elapsed.as_millis() as u64;
    let next_tick = (elapsed_ms / CARET_BLINK_MS + 1) * CARET_BLINK_MS;
    started + Duration::from_millis(next_tick).max(blink)
}

fn lerp(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t
}

fn lerp_color(from: stuk_style::Color, to: stuk_style::Color, t: f32) -> stuk_style::Color {
    stuk_style::Color::rgba(
        lerp(from.r, to.r, t),
        lerp(from.g, to.g, t),
        lerp(from.b, to.b, t),
        lerp(from.a, to.a, t),
    )
}

fn action_for_shortcut<'a>(
    shortcuts: &'a [(Shortcut, String)],
    shortcut: &Shortcut,
) -> Option<&'a str> {
    shortcuts
        .iter()
        .find(|(candidate, _)| candidate == shortcut)
        .map(|(_, action_id)| action_id.as_str())
}

fn handle_builtin_action(
    window: &Arc<dyn Window>,
    event_loop: &dyn ActiveEventLoop,
    action_id: &str,
) -> bool {
    match action_id {
        ACTION_WINDOW_CLOSE => {
            event_loop.exit();
            true
        }
        ACTION_WINDOW_MINIMIZE => {
            window.set_minimized(true);
            true
        }
        ACTION_WINDOW_TOGGLE_MAXIMIZE => {
            window.set_maximized(!window.is_maximized());
            true
        }
        _ => false,
    }
}

fn input_command_from_key_event(event: &KeyEvent, modifiers: ModifiersState) -> Option<String> {
    let key = key_name(&event.key_without_modifiers)?;
    if modifiers.control_key() || modifiers.meta_key() {
        return match key.as_str() {
            "A" | "a" => Some("input.edit.select_all".to_string()),
            "C" | "c" => Some("input.edit.copy".to_string()),
            "X" | "x" => Some("input.edit.cut".to_string()),
            "V" | "v" => Some(input_paste_action()),
            "ArrowLeft" => Some(selection_command("input.move.word_left", modifiers)),
            "ArrowRight" => Some(selection_command("input.move.word_right", modifiers)),
            "Home" => Some(selection_command("input.move.start", modifiers)),
            "End" => Some(selection_command("input.move.end", modifiers)),
            _ => None,
        };
    }
    if modifiers.shift_key() {
        return match key.as_str() {
            "ArrowLeft" => Some("input.move.left.select".to_string()),
            "ArrowRight" => Some("input.move.right.select".to_string()),
            "Home" => Some("input.move.line_start.select".to_string()),
            "End" => Some("input.move.line_end.select".to_string()),
            _ => None,
        };
    }
    None
}

fn selection_command(base: &str, modifiers: ModifiersState) -> String {
    if modifiers.shift_key() {
        format!("{base}.select")
    } else {
        base.to_string()
    }
}

fn input_paste_action() -> String {
    let text = read_clipboard_text().unwrap_or_default();
    format!("input.edit.paste.{}", encode_action_text(&text))
}

fn encode_action_text(text: &str) -> String {
    text.bytes()
        .flat_map(|byte| {
            if byte.is_ascii_alphanumeric() || matches!(byte, b' ' | b'.' | b',' | b'-' | b'_') {
                vec![byte as char]
            } else {
                format!("%{byte:02X}").chars().collect::<Vec<_>>()
            }
        })
        .collect()
}

fn shortcut_from_key_event(event: &KeyEvent, modifiers: ModifiersState) -> Option<Shortcut> {
    key_name(&event.key_without_modifiers)
        .map(|key| Shortcut::new(shortcut_modifiers(modifiers), key))
}

fn shortcut_modifiers(modifiers: ModifiersState) -> StukModifiers {
    StukModifiers {
        ctrl: modifiers.control_key(),
        alt: modifiers.alt_key(),
        shift: modifiers.shift_key(),
        meta: modifiers.meta_key(),
    }
}

fn key_name(key: &Key) -> Option<String> {
    match key.as_ref() {
        Key::Character(" ") => Some("Space".to_string()),
        Key::Character(value) if !value.is_empty() => Some(value.to_string()),
        Key::Named(named) => Some(named.to_string()),
        _ => None,
    }
}

fn key_name_for_input(event: &KeyEvent, modifiers: ModifiersState) -> Option<String> {
    let has_mod = modifiers.control_key() || modifiers.alt_key() || modifiers.meta_key();
    if has_mod {
        return None;
    }
    match event.logical_key.as_ref() {
        Key::Character(c) if !c.is_empty() => Some(c.to_string()),
        Key::Named(named) => match named.to_string().as_str() {
            "Backspace" => Some("Backspace".to_string()),
            "Enter" => Some("Enter".to_string()),
            "Tab" => Some("Tab".to_string()),
            "ArrowLeft" => Some("ArrowLeft".to_string()),
            "ArrowRight" => Some("ArrowRight".to_string()),
            "ArrowUp" => Some("ArrowUp".to_string()),
            "ArrowDown" => Some("ArrowDown".to_string()),
            "Home" => Some("Home".to_string()),
            "End" => Some("End".to_string()),
            "Delete" => Some("Delete".to_string()),
            "Space" => Some(" ".to_string()),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_window_chrome_values() {
        assert_eq!(WindowChrome::parse("system"), Some(WindowChrome::System));
        assert_eq!(WindowChrome::parse("compact"), Some(WindowChrome::Compact));
        assert_eq!(WindowChrome::parse("none"), Some(WindowChrome::None));
        assert_eq!(WindowChrome::parse("floating"), None);
    }

    #[test]
    fn system_chrome_uses_native_decorations() {
        assert!(WindowChrome::System.uses_native_decorations());
        assert!(!WindowChrome::Stuk.uses_native_decorations());
        assert!(!WindowChrome::Compact.uses_native_decorations());
        assert!(!WindowChrome::Sidebar.uses_native_decorations());
        assert!(!WindowChrome::None.uses_native_decorations());
    }

    #[test]
    fn parses_background_effect_values() {
        assert_eq!(
            WindowBackgroundEffect::parse("luca"),
            Some(WindowBackgroundEffect::Luca)
        );
        assert_eq!(
            WindowBackgroundEffect::parse("niko"),
            Some(WindowBackgroundEffect::Niko)
        );
        assert_eq!(
            WindowBackgroundEffect::parse("maris"),
            Some(WindowBackgroundEffect::Maris)
        );
        assert_eq!(
            WindowBackgroundEffect::parse("mica"),
            Some(WindowBackgroundEffect::Mica)
        );
        assert_eq!(
            WindowBackgroundEffect::parse("under-window-background"),
            Some(WindowBackgroundEffect::UnderWindowBackground)
        );
        assert_eq!(WindowBackgroundEffect::Luca.as_str(), "luca");
        assert_eq!(WindowBackgroundEffect::MicaAlt.as_str(), "mica-alt");
        assert!(WindowBackgroundEffect::Acrylic.requires_transparency());
        assert!(!WindowBackgroundEffect::None.requires_transparency());
        assert_eq!(WindowBackgroundEffect::parse("sparkles"), None);
    }

    #[test]
    fn window_options_drop_effects_without_capability_support() {
        let options = WindowOptions {
            transparent: true,
            background_effect: WindowBackgroundEffect::Mica,
            ..WindowOptions::default()
        }
        .resolved_for_capabilities(PlatformCapabilities::generic());

        assert!(!options.transparent);
        assert_eq!(options.background_effect, WindowBackgroundEffect::None);
    }

    #[test]
    fn generic_capabilities_are_conservative() {
        let capabilities = PlatformCapabilities::generic();

        assert!(!capabilities.live_blur);
        assert!(!capabilities.transparent_windows);
        assert!(!capabilities.command_palette);
        assert!(!capabilities.secure_storage);
        assert!(!capabilities.credential_storage);
        assert!(capabilities.system_dark_mode);
    }

    #[test]
    fn native_desktop_capabilities_include_secure_storage() {
        for capabilities in [
            PlatformCapabilities::desktop_linux(true, true),
            PlatformCapabilities::desktop_windows(true),
            PlatformCapabilities::desktop_macos(true),
        ] {
            assert!(capabilities.secure_storage);
            assert!(capabilities.credential_storage);
            assert!(capabilities.tray_icons);
            assert!(capabilities.global_shortcuts);
            assert!(capabilities.single_instance);
        }
    }

    #[test]
    fn clipboard_data_carries_text_payloads() {
        let data = ClipboardData::text("notes");

        assert_eq!(data.as_text(), "notes");
        assert!(!data.is_empty());
        assert_eq!(data.into_text(), "notes");
    }
}

pub fn read_os_clipboard() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        for (cmd, extra_args) in &[
            ("wl-paste", vec!["--no-newline"]),
            ("xclip", vec!["-selection", "clipboard", "-o"]),
            ("xsel", vec!["--clipboard"]),
        ] {
            if let Ok(output) = std::process::Command::new(cmd).args(extra_args).output() {
                if output.status.success() {
                    return Some(String::from_utf8_lossy(&output.stdout).to_string());
                }
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("pbpaste").output() {
            if output.status.success() {
                return Some(String::from_utf8_lossy(&output.stdout).to_string());
            }
        }
    }
    None
}

pub fn write_os_clipboard(text: &str) {
    #[cfg(target_os = "linux")]
    {
        let _ = try_write_clipboard("wl-copy", &["--trim-newline"], text);
        let _ = try_write_clipboard("xclip", &["-selection", "clipboard"], text);
    }
    #[cfg(target_os = "macos")]
    {
        let _ = try_write_clipboard("pbcopy", &[], text);
    }
    #[cfg(target_os = "windows")]
    {
        let _ = text;
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = text;
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn try_write_clipboard(cmd: &str, args: &[&str], text: &str) -> std::io::Result<()> {
    use std::io::Write;
    let mut child = std::process::Command::new(cmd)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(text.as_bytes());
    }
    child.wait()?;
    Ok(())
}
