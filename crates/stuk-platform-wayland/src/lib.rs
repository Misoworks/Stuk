#[allow(dead_code)]
mod blur;
mod desktop_services;

use stuk_actions::ActionDescriptor;
use stuk_platform::{
    AutostartEntry, BackendDescriptor, ClipboardData, DeepLinkRegistration, FileDialogOptions,
    FileDialogResult, GenericPlatform, GlobalShortcutRegistration, MaterialEffect,
    MaterialResolution, MaterialResolver, NativeMessagingHost, Platform, PlatformCapabilities,
    PlatformError, SingleInstancePolicy, TrayIcon, WindowChrome, WindowHandle, WindowId,
    WindowOptions,
};
use stuk_style::{Material, Theme};

use crate::desktop_services::LinuxDesktopServices;

#[derive(Debug)]
pub struct WaylandPlatform {
    inner: GenericPlatform,
    desktop_services: LinuxDesktopServices,
    background_effects: bool,
}

impl WaylandPlatform {
    pub fn new() -> Self {
        Self::from_background_effect_support(blur::has_background_effect_protocol())
    }

    pub fn from_background_effect_support(background_effects: bool) -> Self {
        Self {
            inner: GenericPlatform::with_backend(BackendDescriptor::linux_wayland(
                background_effects,
            )),
            desktop_services: LinuxDesktopServices,
            background_effects,
        }
    }

    pub fn with_background_effects() -> Self {
        Self::from_background_effect_support(true)
    }

    pub fn without_background_effects() -> Self {
        Self::from_background_effect_support(false)
    }

    pub fn background_effects(&self) -> bool {
        self.background_effects
    }
}

impl Default for WaylandPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl MaterialResolver for WaylandPlatform {
    fn resolve_material(&self, material: &Material, theme: &Theme) -> MaterialResolution {
        match material {
            Material::Luca if self.background_effects => MaterialResolution::with_effect(
                material,
                theme,
                MaterialEffect::CompositorBlur {
                    backend: "ext-background-effect-v1",
                    radius: 28.0,
                },
            ),
            _ => MaterialResolution::fallback(material, theme),
        }
    }
}

impl Platform for WaylandPlatform {
    fn create_window(&mut self, options: WindowOptions) -> Result<WindowHandle, PlatformError> {
        self.inner.create_window(options)
    }

    fn destroy_window(&mut self, window: WindowId) {
        self.inner.destroy_window(window);
    }

    fn request_redraw(&mut self, window: WindowId) {
        self.inner.request_redraw(window);
    }

    fn set_title(&mut self, window: WindowId, title: &str) {
        self.inner.set_title(window, title);
    }

    fn set_material(&mut self, window: WindowId, material: Material) {
        self.inner.set_material(window, material);
    }

    fn set_chrome(&mut self, window: WindowId, chrome: WindowChrome) {
        self.inner.set_chrome(window, chrome);
    }

    fn set_window_visible(&mut self, window: WindowId, visible: bool) -> bool {
        self.inner.set_window_visible(window, visible)
    }

    fn present_window(&mut self, window: WindowId) -> bool {
        self.inner.present_window(window)
    }

    fn set_window_always_on_top(&mut self, window: WindowId, always_on_top: bool) -> bool {
        self.inner.set_window_always_on_top(window, always_on_top)
    }

    fn register_actions(&mut self, actions: &[ActionDescriptor]) {
        self.inner.register_actions(actions);
    }

    fn read_clipboard(&self) -> Option<ClipboardData> {
        self.inner.read_clipboard()
    }

    fn write_clipboard(&self, data: ClipboardData) {
        self.inner.write_clipboard(data);
    }

    fn open_file_dialog(&self, options: FileDialogOptions) -> FileDialogResult {
        self.inner.open_file_dialog(options)
    }

    fn platform_capabilities(&self) -> PlatformCapabilities {
        self.inner.platform_capabilities()
    }

    fn backend(&self) -> BackendDescriptor {
        self.inner.backend()
    }

    fn set_tray_icon(&mut self, _icon: TrayIcon) -> bool {
        false
    }

    fn remove_tray_icon(&mut self, _id: &str) -> bool {
        false
    }

    fn set_autostart(&mut self, entry: AutostartEntry) -> bool {
        if !self.desktop_services.set_autostart(&entry) {
            return false;
        }
        self.inner.set_autostart(entry)
    }

    fn register_global_shortcut(&mut self, _registration: GlobalShortcutRegistration) -> bool {
        false
    }

    fn unregister_global_shortcut(&mut self, _id: &str) -> bool {
        false
    }

    fn register_deep_links(&mut self, registration: DeepLinkRegistration) -> bool {
        if !self.desktop_services.register_deep_links(&registration) {
            return false;
        }
        self.inner.register_deep_links(registration)
    }

    fn register_native_messaging_host(&mut self, host: NativeMessagingHost) -> bool {
        if !self.desktop_services.register_native_messaging_host(&host) {
            return false;
        }
        self.inner.register_native_messaging_host(host)
    }

    fn set_single_instance_policy(&mut self, _policy: SingleInstancePolicy) -> bool {
        false
    }
}

pub fn wayland_capabilities(background_effects: bool) -> PlatformCapabilities {
    PlatformCapabilities::desktop_linux(background_effects, background_effects)
}

pub fn wayland_backend(background_effects: bool) -> BackendDescriptor {
    BackendDescriptor::linux_wayland(background_effects)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wayland_uses_background_effect_protocol_when_available() {
        let platform = WaylandPlatform::with_background_effects();
        let effect = platform
            .resolve_material(&Material::Luca, &Theme::dark())
            .effect;

        assert_eq!(
            effect,
            MaterialEffect::CompositorBlur {
                backend: "ext-background-effect-v1",
                radius: 28.0
            }
        );
        assert!(platform.platform_capabilities().live_blur);
    }

    #[test]
    fn wayland_falls_back_without_background_effect_protocol() {
        let platform = WaylandPlatform::without_background_effects();
        let effect = platform
            .resolve_material(&Material::Luca, &Theme::dark())
            .effect;

        assert_eq!(effect, MaterialEffect::TintedFallback);
        assert!(!platform.platform_capabilities().live_blur);
    }
}
