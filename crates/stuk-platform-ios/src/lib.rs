use stuk_actions::ActionDescriptor;
use stuk_platform::{
    BackendDescriptor, ClipboardData, FileDialogOptions, FileDialogResult, GenericPlatform,
    MaterialEffect, MaterialResolution, MaterialResolver, Platform, PlatformCapabilities,
    PlatformError, WindowChrome, WindowHandle, WindowId, WindowOptions,
};
use stuk_style::{Material, Theme};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IosScenePhase {
    Inactive,
    Active,
    Background,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IosStatusBarStyle {
    Automatic,
    Light,
    Dark,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IosShellOptions {
    pub scene_phase: IosScenePhase,
    pub status_bar_style: IosStatusBarStyle,
    pub prefers_home_indicator_hidden: bool,
    pub edge_to_edge: bool,
    pub keyboard_aware: bool,
    pub safe_area_top: f32,
    pub safe_area_right: f32,
    pub safe_area_bottom: f32,
    pub safe_area_left: f32,
}

impl Default for IosShellOptions {
    fn default() -> Self {
        Self {
            scene_phase: IosScenePhase::Inactive,
            status_bar_style: IosStatusBarStyle::Automatic,
            prefers_home_indicator_hidden: false,
            edge_to_edge: true,
            keyboard_aware: true,
            safe_area_top: 0.0,
            safe_area_right: 0.0,
            safe_area_bottom: 0.0,
            safe_area_left: 0.0,
        }
    }
}

#[derive(Debug)]
pub struct IosPlatform {
    inner: GenericPlatform,
    shell: IosShellOptions,
}

impl IosPlatform {
    pub fn new() -> Self {
        Self {
            inner: GenericPlatform::with_backend(BackendDescriptor::ios()),
            shell: IosShellOptions::default(),
        }
    }

    pub fn with_shell(shell: IosShellOptions) -> Self {
        Self {
            inner: GenericPlatform::with_backend(BackendDescriptor::ios()),
            shell,
        }
    }

    pub fn inner(&self) -> &GenericPlatform {
        &self.inner
    }

    pub fn shell(&self) -> &IosShellOptions {
        &self.shell
    }
}

impl Default for IosPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl MaterialResolver for IosPlatform {
    fn resolve_material(&self, material: &Material, theme: &Theme) -> MaterialResolution {
        match material {
            Material::Window | Material::Sidebar | Material::Toolbar => {
                MaterialResolution::with_effect(
                    material,
                    theme,
                    MaterialEffect::NativeMaterial {
                        name: "system-material",
                    },
                )
            }
            _ => MaterialResolution::fallback(material, theme),
        }
    }
}

impl Platform for IosPlatform {
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

    fn backend(&self) -> BackendDescriptor {
        self.inner.backend()
    }
}

pub fn ios_capabilities() -> PlatformCapabilities {
    PlatformCapabilities::mobile_ios()
}

pub fn ios_backend() -> BackendDescriptor {
    BackendDescriptor::ios()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ios_backend_is_mobile_touch_target_with_material_fallbacks() {
        let platform = IosPlatform::new();

        assert!(platform.backend().target.is_mobile());
        assert!(platform.platform_capabilities().mobile_shell);
        assert!(platform.platform_capabilities().touch_input);
        assert!(platform.shell().edge_to_edge);
        assert_eq!(
            platform
                .resolve_material(&Material::Window, &Theme::dark())
                .effect,
            MaterialEffect::NativeMaterial {
                name: "system-material"
            }
        );
    }
}
