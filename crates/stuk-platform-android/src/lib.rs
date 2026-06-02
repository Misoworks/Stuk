use stuk_actions::ActionDescriptor;
use stuk_platform::{
    BackendDescriptor, ClipboardData, FileDialogOptions, FileDialogResult, GenericPlatform,
    MaterialResolution, MaterialResolver, Platform, PlatformCapabilities, PlatformError,
    WindowChrome, WindowHandle, WindowId, WindowOptions,
};
use stuk_style::{Material, Theme};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AndroidLifecyclePhase {
    Created,
    Started,
    Resumed,
    Paused,
    Stopped,
    Destroyed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AndroidNavigationMode {
    Gesture,
    ThreeButton,
    Unknown,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AndroidShellOptions {
    pub lifecycle: AndroidLifecyclePhase,
    pub navigation_mode: AndroidNavigationMode,
    pub edge_to_edge: bool,
    pub ime_aware: bool,
    pub safe_area_top: f32,
    pub safe_area_right: f32,
    pub safe_area_bottom: f32,
    pub safe_area_left: f32,
}

impl Default for AndroidShellOptions {
    fn default() -> Self {
        Self {
            lifecycle: AndroidLifecyclePhase::Created,
            navigation_mode: AndroidNavigationMode::Unknown,
            edge_to_edge: true,
            ime_aware: true,
            safe_area_top: 0.0,
            safe_area_right: 0.0,
            safe_area_bottom: 0.0,
            safe_area_left: 0.0,
        }
    }
}

#[derive(Debug)]
pub struct AndroidPlatform {
    inner: GenericPlatform,
    shell: AndroidShellOptions,
}

impl AndroidPlatform {
    pub fn new() -> Self {
        Self {
            inner: GenericPlatform::with_backend(BackendDescriptor::android()),
            shell: AndroidShellOptions::default(),
        }
    }

    pub fn with_shell(shell: AndroidShellOptions) -> Self {
        Self {
            inner: GenericPlatform::with_backend(BackendDescriptor::android()),
            shell,
        }
    }

    pub fn inner(&self) -> &GenericPlatform {
        &self.inner
    }

    pub fn shell(&self) -> &AndroidShellOptions {
        &self.shell
    }
}

impl Default for AndroidPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl MaterialResolver for AndroidPlatform {
    fn resolve_material(&self, material: &Material, theme: &Theme) -> MaterialResolution {
        MaterialResolution::fallback(material, theme)
    }
}

impl Platform for AndroidPlatform {
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

pub fn android_capabilities() -> PlatformCapabilities {
    PlatformCapabilities::mobile_android()
}

pub fn android_backend() -> BackendDescriptor {
    BackendDescriptor::android()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn android_backend_is_mobile_touch_target() {
        let platform = AndroidPlatform::new();

        assert!(platform.backend().target.is_mobile());
        assert!(platform.platform_capabilities().mobile_shell);
        assert!(platform.platform_capabilities().touch_input);
        assert!(platform.platform_capabilities().native_bridge);
        assert!(platform.shell().edge_to_edge);
    }
}
