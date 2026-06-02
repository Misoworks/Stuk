use stuk_actions::ActionDescriptor;
use stuk_platform::{
    AutostartEntry, BackendDescriptor, BackendKind, BackendStatus, ClipboardData,
    DeepLinkRegistration, FileDialogOptions, FileDialogResult, GenericPlatform,
    GlobalShortcutRegistration, MaterialEffect, MaterialResolution, MaterialResolver,
    NativeMessagingHost, Platform, PlatformCapabilities, PlatformError, PlatformOs, RuntimeTarget,
    SingleInstancePolicy, TrayIcon, WindowChrome, WindowHandle, WindowId, WindowOptions,
};
use stuk_style::{Material, Theme};

#[derive(Debug)]
pub struct MacosPlatform {
    inner: GenericPlatform,
    vibrancy: MacosVibrancy,
}

impl MacosPlatform {
    pub fn new() -> Self {
        Self::with_vibrancy(MacosVibrancy::HudWindow)
    }

    pub fn with_vibrancy(vibrancy: MacosVibrancy) -> Self {
        Self {
            inner: GenericPlatform::with_backend(macos_backend(vibrancy)),
            vibrancy,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MacosVibrancy {
    HudWindow,
    Sidebar,
    UnderWindowBackground,
    Disabled,
}

impl Default for MacosPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl MaterialResolver for MacosPlatform {
    fn resolve_material(&self, material: &Material, theme: &Theme) -> MaterialResolution {
        match material {
            Material::Luca | Material::Popover | Material::Dialog
                if self.vibrancy != MacosVibrancy::Disabled =>
            {
                MaterialResolution::with_effect(
                    material,
                    theme,
                    MaterialEffect::NativeMaterial {
                        name: self.vibrancy.native_name(),
                    },
                )
            }
            Material::Maris | Material::Window | Material::Sidebar
                if self.vibrancy != MacosVibrancy::Disabled =>
            {
                MaterialResolution::with_effect(
                    material,
                    theme,
                    MaterialEffect::NativeMaterial {
                        name: "under-window-background",
                    },
                )
            }
            _ => MaterialResolution::fallback(material, theme),
        }
    }
}

impl MacosVibrancy {
    fn native_name(self) -> &'static str {
        match self {
            Self::HudWindow => "hud-window",
            Self::Sidebar => "sidebar",
            Self::UnderWindowBackground => "under-window-background",
            Self::Disabled => "solid",
        }
    }
}

impl Platform for MacosPlatform {
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

    fn set_tray_icon(&mut self, icon: TrayIcon) -> bool {
        self.inner.set_tray_icon(icon)
    }

    fn remove_tray_icon(&mut self, id: &str) -> bool {
        self.inner.remove_tray_icon(id)
    }

    fn set_autostart(&mut self, entry: AutostartEntry) -> bool {
        self.inner.set_autostart(entry)
    }

    fn register_global_shortcut(&mut self, registration: GlobalShortcutRegistration) -> bool {
        self.inner.register_global_shortcut(registration)
    }

    fn unregister_global_shortcut(&mut self, id: &str) -> bool {
        self.inner.unregister_global_shortcut(id)
    }

    fn register_deep_links(&mut self, registration: DeepLinkRegistration) -> bool {
        self.inner.register_deep_links(registration)
    }

    fn register_native_messaging_host(&mut self, host: NativeMessagingHost) -> bool {
        self.inner.register_native_messaging_host(host)
    }

    fn set_single_instance_policy(&mut self, policy: SingleInstancePolicy) -> bool {
        self.inner.set_single_instance_policy(policy)
    }
}

pub fn macos_capabilities(vibrancy: MacosVibrancy) -> PlatformCapabilities {
    PlatformCapabilities::desktop_macos(vibrancy != MacosVibrancy::Disabled)
}

pub fn macos_backend(vibrancy: MacosVibrancy) -> BackendDescriptor {
    BackendDescriptor::new(
        "macos",
        BackendKind::NativeDesktop,
        RuntimeTarget::desktop(PlatformOs::Macos),
        BackendStatus::Available,
        macos_capabilities(vibrancy),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macos_maps_semantic_materials_to_native_effects() {
        let platform = MacosPlatform::new();
        let theme = Theme::dark();

        assert_eq!(
            platform.resolve_material(&Material::Luca, &theme).effect,
            MaterialEffect::NativeMaterial { name: "hud-window" }
        );
        assert_eq!(
            platform.resolve_material(&Material::Maris, &theme).effect,
            MaterialEffect::NativeMaterial {
                name: "under-window-background"
            }
        );
        assert!(platform.platform_capabilities().native_notifications);
    }

    #[test]
    fn macos_can_disable_vibrancy() {
        let platform = MacosPlatform::with_vibrancy(MacosVibrancy::Disabled);

        assert_eq!(
            platform
                .resolve_material(&Material::Luca, &Theme::dark())
                .effect,
            MaterialEffect::TintedFallback
        );
        assert!(!platform.platform_capabilities().transparent_windows);
    }
}
