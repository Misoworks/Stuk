use stuk_actions::ActionDescriptor;
use stuk_platform::{
    ClipboardData, FileDialogOptions, FileDialogResult, GenericPlatform, MaterialEffect,
    MaterialResolution, MaterialResolver, Platform, PlatformCapabilities, PlatformError,
    WindowChrome, WindowHandle, WindowId, WindowOptions,
};
use stuk_style::{Material, Theme};

#[derive(Debug)]
pub struct WindowsPlatform {
    inner: GenericPlatform,
}

impl WindowsPlatform {
    pub fn new() -> Self {
        Self {
            inner: GenericPlatform::with_capabilities(windows_capabilities()),
        }
    }
}

impl Default for WindowsPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl MaterialResolver for WindowsPlatform {
    fn resolve_material(&self, material: &Material, theme: &Theme) -> MaterialResolution {
        match material {
            Material::Luca => MaterialResolution::with_effect(
                material,
                theme,
                MaterialEffect::NativeMaterial { name: "acrylic" },
            ),
            Material::Maris | Material::Window => MaterialResolution::with_effect(
                material,
                theme,
                MaterialEffect::NativeMaterial { name: "mica" },
            ),
            _ => MaterialResolution::fallback(material, theme),
        }
    }
}

impl Platform for WindowsPlatform {
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
}

pub fn windows_capabilities() -> PlatformCapabilities {
    PlatformCapabilities {
        live_blur: true,
        wallpaper_material: false,
        shell_tabs: false,
        command_palette: false,
        workspace_sessions: false,
        native_notifications: true,
        system_dark_mode: true,
        high_contrast: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_maps_semantic_materials_to_native_effects() {
        let platform = WindowsPlatform::new();
        let theme = Theme::dark();

        assert_eq!(
            platform.resolve_material(&Material::Luca, &theme).effect,
            MaterialEffect::NativeMaterial { name: "acrylic" }
        );
        assert_eq!(
            platform.resolve_material(&Material::Maris, &theme).effect,
            MaterialEffect::NativeMaterial { name: "mica" }
        );
        assert!(platform.platform_capabilities().native_notifications);
    }
}
