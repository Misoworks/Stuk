use stuk_actions::ActionDescriptor;
use stuk_platform::{
    AutostartEntry, BackendDescriptor, BackendKind, BackendStatus, ClipboardData,
    DeepLinkRegistration, FileDialogOptions, FileDialogResult, GenericPlatform,
    GlobalShortcutRegistration, MaterialEffect, MaterialResolution, MaterialResolver,
    NativeMessagingHost, Platform, PlatformCapabilities, PlatformError, PlatformOs, RuntimeTarget,
    SingleInstancePolicy, TrayIcon, WindowChrome, WindowHandle, WindowId, WindowOptions,
};
pub use stuk_platform::{SplitHint, StaccatoSession};
use stuk_style::{Material, Theme};

#[derive(Debug)]
pub struct StaccatoPlatform {
    inner: GenericPlatform,
}

impl StaccatoPlatform {
    pub fn new() -> Self {
        Self {
            inner: GenericPlatform::with_backend(staccato_backend()),
        }
    }

    pub fn inner(&self) -> &GenericPlatform {
        &self.inner
    }
}

impl Default for StaccatoPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl MaterialResolver for StaccatoPlatform {
    fn resolve_material(&self, material: &Material, theme: &Theme) -> MaterialResolution {
        match material {
            Material::Luca => MaterialResolution::with_effect(
                material,
                theme,
                MaterialEffect::CompositorBlur {
                    backend: "baton",
                    radius: 32.0,
                },
            ),
            Material::Maris => MaterialResolution::with_effect(
                material,
                theme,
                MaterialEffect::WallpaperMaterial { backend: "baton" },
            ),
            _ => MaterialResolution::fallback(material, theme),
        }
    }
}

impl Platform for StaccatoPlatform {
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

pub fn staccato_capabilities() -> PlatformCapabilities {
    PlatformCapabilities {
        native_windows: true,
        web_surface: false,
        mobile_shell: false,
        native_bridge: true,
        live_blur: true,
        transparent_windows: true,
        wallpaper_material: true,
        touch_input: false,
        pointer_input: true,
        keyboard_input: true,
        file_dialogs: true,
        shell_tabs: true,
        command_palette: true,
        workspace_sessions: true,
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

pub fn staccato_backend() -> BackendDescriptor {
    BackendDescriptor::new(
        "staccato",
        BackendKind::NativeDesktop,
        RuntimeTarget::desktop(PlatformOs::Linux),
        BackendStatus::Preview,
        staccato_capabilities(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staccato_resolves_luca_and_maris_to_baton_effects() {
        let platform = StaccatoPlatform::new();
        let theme = Theme::dark();

        assert_eq!(
            platform.resolve_material(&Material::Luca, &theme).effect,
            MaterialEffect::CompositorBlur {
                backend: "baton",
                radius: 32.0
            }
        );
        assert_eq!(
            platform.resolve_material(&Material::Maris, &theme).effect,
            MaterialEffect::WallpaperMaterial { backend: "baton" }
        );
        assert!(platform.platform_capabilities().command_palette);
    }

    #[test]
    fn session_metadata_tracks_shell_restore_hints() {
        let mut session = StaccatoSession::default();
        session.set_tab_title("Notes");
        session.set_document_id("note-1");
        session.set_restore_payload("{\"id\":\"note-1\"}");
        session.set_preferred_split(SplitHint::Right);

        assert_eq!(session.tab_title.as_deref(), Some("Notes"));
        assert_eq!(session.document_id.as_deref(), Some("note-1"));
        assert_eq!(session.preferred_split, Some(SplitHint::Right));
    }
}
