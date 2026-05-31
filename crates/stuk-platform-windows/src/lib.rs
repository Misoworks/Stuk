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
    backdrop: WindowsBackdrop,
}

impl WindowsPlatform {
    pub fn new() -> Self {
        Self::with_backdrop(WindowsBackdrop::Acrylic)
    }

    pub fn with_backdrop(backdrop: WindowsBackdrop) -> Self {
        Self {
            inner: GenericPlatform::with_capabilities(windows_capabilities(backdrop)),
            backdrop,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowsBackdrop {
    Acrylic,
    Mica,
    Disabled,
}

impl Default for WindowsPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl MaterialResolver for WindowsPlatform {
    fn resolve_material(&self, material: &Material, theme: &Theme) -> MaterialResolution {
        match material {
            Material::Luca if self.backdrop != WindowsBackdrop::Disabled => {
                MaterialResolution::with_effect(
                    material,
                    theme,
                    MaterialEffect::NativeMaterial {
                        name: self.backdrop.native_name(),
                    },
                )
            }
            Material::Maris | Material::Window if self.backdrop != WindowsBackdrop::Disabled => {
                MaterialResolution::with_effect(
                    material,
                    theme,
                    MaterialEffect::NativeMaterial { name: "mica" },
                )
            }
            _ => MaterialResolution::fallback(material, theme),
        }
    }
}

impl WindowsBackdrop {
    fn native_name(self) -> &'static str {
        match self {
            Self::Acrylic => "acrylic",
            Self::Mica => "mica",
            Self::Disabled => "solid",
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

pub fn windows_capabilities(backdrop: WindowsBackdrop) -> PlatformCapabilities {
    PlatformCapabilities {
        live_blur: backdrop != WindowsBackdrop::Disabled,
        transparent_windows: backdrop != WindowsBackdrop::Disabled,
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

    #[test]
    fn windows_allows_mica_override_for_live_backdrop() {
        let platform = WindowsPlatform::with_backdrop(WindowsBackdrop::Mica);

        assert_eq!(
            platform
                .resolve_material(&Material::Luca, &Theme::dark())
                .effect,
            MaterialEffect::NativeMaterial { name: "mica" }
        );
    }
}
