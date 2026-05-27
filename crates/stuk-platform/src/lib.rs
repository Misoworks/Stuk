mod integration;
mod material;
mod session;

use std::sync::Arc;

use stuk_accessibility::AccessibilityTree;
use stuk_actions::{ActionHitRegion, Modifiers as StukModifiers, Shortcut};
use stuk_layout::Size;
use stuk_render::{DisplayList, GpuRenderer, RendererError};
use thiserror::Error;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, ModifiersState},
    window::{Window, WindowAttributes, WindowId as WinitWindowId},
};

pub use integration::{
    FileDialogFilter, FileDialogMode, FileDialogOptions, FileDialogResult, GenericPlatform,
    Platform, WindowHandle, WindowId,
};
pub use material::{MaterialEffect, MaterialResolution, MaterialResolver};
pub use session::{SplitHint, StaccatoSession};

pub type NativeActionHandler = Arc<dyn Fn(&str)>;

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
        !matches!(self, Self::None)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlatformCapabilities {
    pub live_blur: bool,
    pub wallpaper_material: bool,
    pub shell_tabs: bool,
    pub command_palette: bool,
    pub workspace_sessions: bool,
    pub native_notifications: bool,
    pub system_dark_mode: bool,
    pub high_contrast: bool,
}

impl PlatformCapabilities {
    pub fn generic() -> Self {
        Self {
            live_blur: false,
            wallpaper_material: false,
            shell_tabs: false,
            command_palette: false,
            workspace_sessions: false,
            native_notifications: false,
            system_dark_mode: true,
            high_contrast: false,
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
}

impl NativeFrame {
    pub fn new(display_list: DisplayList) -> Self {
        Self {
            display_list,
            hit_regions: Vec::new(),
            accessibility_tree: AccessibilityTree::empty(),
        }
    }
}

impl From<DisplayList> for NativeFrame {
    fn from(display_list: DisplayList) -> Self {
        Self::new(display_list)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowOptions {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub min_width: u32,
    pub min_height: u32,
    pub chrome: WindowChrome,
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            title: "Stuk".to_string(),
            width: 980,
            height: 680,
            min_width: 420,
            min_height: 280,
            chrome: WindowChrome::System,
        }
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
    F: Fn(Size) -> NativeFrame + 'static,
{
    options: WindowOptions,
    render: F,
    action_handler: Option<NativeActionHandler>,
    shortcuts: Vec<(Shortcut, String)>,
    state: Option<NativeState>,
}

impl<F> NativeApp<F>
where
    F: Fn(Size) -> NativeFrame + 'static,
{
    pub fn new(options: WindowOptions, render: F) -> Self {
        Self {
            options,
            render,
            action_handler: None,
            shortcuts: Vec::new(),
            state: None,
        }
    }

    pub fn shortcuts(mut self, shortcuts: Vec<(Shortcut, String)>) -> Self {
        self.shortcuts = shortcuts;
        self
    }

    pub fn on_action(mut self, handler: NativeActionHandler) -> Self {
        self.action_handler = Some(handler);
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
    F: Fn(Size) -> NativeFrame + 'static,
{
    app: NativeApp<F>,
}

impl<F> ApplicationHandler for NativeHandler<F>
where
    F: Fn(Size) -> NativeFrame + 'static,
{
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.app.state.is_some() {
            return;
        }

        let options = &self.app.options;
        let attributes = WindowAttributes::default()
            .with_title(options.title.clone())
            .with_surface_size(LogicalSize::new(
                f64::from(options.width),
                f64::from(options.height),
            ))
            .with_min_surface_size(LogicalSize::new(
                f64::from(options.min_width),
                f64::from(options.min_height),
            ))
            .with_decorations(options.chrome.uses_native_decorations());

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

        self.app.state = Some(NativeState {
            window,
            renderer,
            last_frame: None,
            modifiers: ModifiersState::default(),
        });
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
                state.window.request_redraw();
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                let size = state.window.surface_size();
                state
                    .renderer
                    .resize(size.width, size.height, scale_factor as f32);
                state.window.request_redraw();
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                state.modifiers = modifiers.state();
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic: false,
                ..
            } if event.state == ElementState::Pressed && !event.repeat => {
                if let Some(shortcut) = shortcut_from_key_event(&event, state.modifiers)
                    && let Some(action_id) = action_for_shortcut(&self.app.shortcuts, &shortcut)
                    && let Some(handler) = &self.app.action_handler
                {
                    handler(action_id);
                    state.window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                state.window.pre_present_notify();
                let frame = (self.app.render)(state.renderer.logical_size());
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
                if let Some(action_id) = hit_action(state.last_frame.as_ref(), x, y)
                    && let Some(handler) = &self.app.action_handler
                {
                    handler(action_id);
                    state.window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

struct NativeState {
    window: Arc<dyn Window>,
    renderer: GpuRenderer,
    last_frame: Option<NativeFrame>,
    modifiers: ModifiersState,
}

fn hit_action(frame: Option<&NativeFrame>, x: f32, y: f32) -> Option<&str> {
    frame?
        .hit_regions
        .iter()
        .rev()
        .find(|region| region.enabled && region.contains(x, y))
        .map(|region| region.action_id.as_str())
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
    fn only_none_disables_native_decorations_for_now() {
        assert!(WindowChrome::System.uses_native_decorations());
        assert!(WindowChrome::Stuk.uses_native_decorations());
        assert!(!WindowChrome::None.uses_native_decorations());
    }

    #[test]
    fn generic_capabilities_are_conservative() {
        let capabilities = PlatformCapabilities::generic();

        assert!(!capabilities.live_blur);
        assert!(!capabilities.command_palette);
        assert!(capabilities.system_dark_mode);
    }

    #[test]
    fn clipboard_data_carries_text_payloads() {
        let data = ClipboardData::text("notes");

        assert_eq!(data.as_text(), "notes");
        assert!(!data.is_empty());
        assert_eq!(data.into_text(), "notes");
    }
}
