use stuk_actions::ActionDescriptor;
use stuk_platform::{
    BackendDescriptor, ClipboardData, FileDialogOptions, FileDialogResult, GenericPlatform,
    MaterialResolution, MaterialResolver, Platform, PlatformCapabilities, PlatformError,
    WindowChrome, WindowHandle, WindowId, WindowOptions,
};
use stuk_style::{Material, Theme};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebCanvasOptions {
    pub append_to_document: bool,
    pub prevent_default: bool,
    pub focusable: bool,
    pub container_id: Option<String>,
}

impl Default for WebCanvasOptions {
    fn default() -> Self {
        Self {
            append_to_document: true,
            prevent_default: true,
            focusable: true,
            container_id: None,
        }
    }
}

impl WebCanvasOptions {
    pub fn embedded(container_id: impl Into<String>) -> Self {
        Self {
            container_id: Some(container_id.into()),
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebRunOptions {
    pub canvas: WebCanvasOptions,
    pub hydrate_existing_dom: bool,
    pub service_worker: bool,
}

impl Default for WebRunOptions {
    fn default() -> Self {
        Self {
            canvas: WebCanvasOptions::default(),
            hydrate_existing_dom: false,
            service_worker: false,
        }
    }
}

#[derive(Debug)]
pub struct WebPlatform {
    inner: GenericPlatform,
    run_options: WebRunOptions,
}

impl WebPlatform {
    pub fn new() -> Self {
        Self {
            inner: GenericPlatform::with_backend(BackendDescriptor::browser_web()),
            run_options: WebRunOptions::default(),
        }
    }

    pub fn with_run_options(run_options: WebRunOptions) -> Self {
        Self {
            inner: GenericPlatform::with_backend(BackendDescriptor::browser_web()),
            run_options,
        }
    }

    pub fn inner(&self) -> &GenericPlatform {
        &self.inner
    }

    pub fn run_options(&self) -> &WebRunOptions {
        &self.run_options
    }
}

impl Default for WebPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl MaterialResolver for WebPlatform {
    fn resolve_material(&self, material: &Material, theme: &Theme) -> MaterialResolution {
        MaterialResolution::fallback(material, theme)
    }
}

impl Platform for WebPlatform {
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

pub fn web_capabilities() -> PlatformCapabilities {
    PlatformCapabilities::browser_web()
}

pub fn web_backend() -> BackendDescriptor {
    BackendDescriptor::browser_web()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_backend_is_browser_target_without_native_bridge() {
        let platform = WebPlatform::new();
        let backend = platform.backend();

        assert_eq!(backend.name, "web");
        assert!(backend.target.is_web());
        assert!(platform.platform_capabilities().web_surface);
        assert!(!platform.platform_capabilities().native_bridge);
        assert!(platform.run_options().canvas.append_to_document);
    }
}
