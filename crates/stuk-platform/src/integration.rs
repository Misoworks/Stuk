use std::{cell::RefCell, collections::BTreeMap, path::PathBuf};

use stuk_actions::ActionDescriptor;
use stuk_style::Material;

use crate::{
    BackendDescriptor, ClipboardData, PlatformCapabilities, PlatformError, WindowChrome,
    WindowOptions,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId(u64);

impl WindowId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowHandle {
    pub id: WindowId,
    pub options: WindowOptions,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileDialogMode {
    OpenFile,
    OpenDirectory,
    SaveFile,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileDialogFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

impl FileDialogFilter {
    pub fn new(
        name: impl Into<String>,
        extensions: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            name: name.into(),
            extensions: extensions.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileDialogOptions {
    pub title: Option<String>,
    pub mode: FileDialogMode,
    pub multiple: bool,
    pub filters: Vec<FileDialogFilter>,
}

impl FileDialogOptions {
    pub fn open_file() -> Self {
        Self::default()
    }

    pub fn open_directory() -> Self {
        Self {
            mode: FileDialogMode::OpenDirectory,
            ..Self::default()
        }
    }

    pub fn save_file() -> Self {
        Self {
            mode: FileDialogMode::SaveFile,
            ..Self::default()
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    pub fn filter(mut self, filter: FileDialogFilter) -> Self {
        self.filters.push(filter);
        self
    }
}

impl Default for FileDialogOptions {
    fn default() -> Self {
        Self {
            title: None,
            mode: FileDialogMode::OpenFile,
            multiple: false,
            filters: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileDialogResult {
    Cancelled,
    Selected(Vec<PathBuf>),
}

impl FileDialogResult {
    pub fn selected(path: impl Into<PathBuf>) -> Self {
        Self::Selected(vec![path.into()])
    }

    pub fn selected_paths(paths: impl IntoIterator<Item = impl Into<PathBuf>>) -> Self {
        Self::Selected(paths.into_iter().map(Into::into).collect())
    }

    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }

    pub fn paths(&self) -> &[PathBuf] {
        match self {
            Self::Cancelled => &[],
            Self::Selected(paths) => paths,
        }
    }

    pub fn first_path(&self) -> Option<&PathBuf> {
        self.paths().first()
    }
}

pub trait Platform {
    fn create_window(&mut self, options: WindowOptions) -> Result<WindowHandle, PlatformError>;
    fn destroy_window(&mut self, window: WindowId);
    fn request_redraw(&mut self, window: WindowId);
    fn set_title(&mut self, window: WindowId, title: &str);
    fn set_material(&mut self, window: WindowId, material: Material);
    fn set_chrome(&mut self, window: WindowId, chrome: WindowChrome);
    fn register_actions(&mut self, actions: &[ActionDescriptor]);
    fn read_clipboard(&self) -> Option<ClipboardData>;
    fn write_clipboard(&self, data: ClipboardData);
    fn open_file_dialog(&self, options: FileDialogOptions) -> FileDialogResult;
    fn platform_capabilities(&self) -> PlatformCapabilities;

    fn backend(&self) -> BackendDescriptor {
        BackendDescriptor::generic()
    }
}

#[derive(Debug)]
pub struct GenericPlatform {
    backend: BackendDescriptor,
    capabilities: PlatformCapabilities,
    next_window_id: u64,
    windows: BTreeMap<WindowId, GenericWindow>,
    actions: Vec<ActionDescriptor>,
    clipboard: RefCell<Option<ClipboardData>>,
    next_file_dialog_result: RefCell<Option<FileDialogResult>>,
}

impl GenericPlatform {
    pub fn new() -> Self {
        Self::with_backend(BackendDescriptor::generic())
    }

    pub fn with_capabilities(capabilities: PlatformCapabilities) -> Self {
        Self::with_backend(BackendDescriptor {
            capabilities,
            ..BackendDescriptor::generic()
        })
    }

    pub fn with_backend(backend: BackendDescriptor) -> Self {
        Self {
            capabilities: backend.capabilities,
            backend,
            next_window_id: 1,
            windows: BTreeMap::new(),
            actions: Vec::new(),
            clipboard: RefCell::new(None),
            next_file_dialog_result: RefCell::new(None),
        }
    }

    pub fn windows(&self) -> impl Iterator<Item = (&WindowId, &WindowOptions)> {
        self.windows
            .iter()
            .map(|(id, window)| (id, &window.options))
    }

    pub fn actions(&self) -> &[ActionDescriptor] {
        &self.actions
    }

    pub fn material(&self, window: WindowId) -> Option<Material> {
        self.windows
            .get(&window)
            .and_then(|window| window.material.clone())
    }

    pub fn redraw_requested(&self, window: WindowId) -> bool {
        self.windows
            .get(&window)
            .map(|window| window.redraw_requested)
            .unwrap_or(false)
    }

    pub fn set_next_file_dialog_result(&self, result: FileDialogResult) {
        *self.next_file_dialog_result.borrow_mut() = Some(result);
    }
}

impl Default for GenericPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl Platform for GenericPlatform {
    fn create_window(&mut self, options: WindowOptions) -> Result<WindowHandle, PlatformError> {
        let options = options.resolved_for_capabilities(self.capabilities);
        let id = WindowId(self.next_window_id);
        self.next_window_id += 1;
        self.windows.insert(
            id,
            GenericWindow {
                options: options.clone(),
                material: None,
                redraw_requested: false,
            },
        );
        Ok(WindowHandle { id, options })
    }

    fn destroy_window(&mut self, window: WindowId) {
        self.windows.remove(&window);
    }

    fn request_redraw(&mut self, window: WindowId) {
        if let Some(window) = self.windows.get_mut(&window) {
            window.redraw_requested = true;
        }
    }

    fn set_title(&mut self, window: WindowId, title: &str) {
        if let Some(window) = self.windows.get_mut(&window) {
            window.options.title = title.to_string();
        }
    }

    fn set_material(&mut self, window: WindowId, material: Material) {
        if let Some(window) = self.windows.get_mut(&window) {
            window.material = Some(material);
        }
    }

    fn set_chrome(&mut self, window: WindowId, chrome: WindowChrome) {
        if let Some(window) = self.windows.get_mut(&window) {
            window.options.chrome = chrome;
        }
    }

    fn register_actions(&mut self, actions: &[ActionDescriptor]) {
        self.actions = actions.to_vec();
    }

    fn read_clipboard(&self) -> Option<ClipboardData> {
        self.clipboard.borrow().clone()
    }

    fn write_clipboard(&self, data: ClipboardData) {
        *self.clipboard.borrow_mut() = Some(data);
    }

    fn open_file_dialog(&self, _options: FileDialogOptions) -> FileDialogResult {
        self.next_file_dialog_result
            .borrow_mut()
            .take()
            .unwrap_or(FileDialogResult::Cancelled)
    }

    fn platform_capabilities(&self) -> PlatformCapabilities {
        self.capabilities
    }

    fn backend(&self) -> BackendDescriptor {
        self.backend.clone()
    }
}

#[derive(Clone, Debug)]
struct GenericWindow {
    options: WindowOptions,
    material: Option<Material>,
    redraw_requested: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use stuk_actions::ActionDescriptor;

    #[test]
    fn generic_platform_tracks_window_updates() {
        let mut platform = GenericPlatform::new();
        let handle = platform
            .create_window(WindowOptions {
                title: "Notes".to_string(),
                ..WindowOptions::default()
            })
            .unwrap();

        platform.set_title(handle.id, "Drafts");
        platform.set_material(handle.id, Material::Maris);
        platform.set_chrome(handle.id, WindowChrome::Compact);
        platform.request_redraw(handle.id);

        let window = platform.windows().next().unwrap().1;
        assert_eq!(window.title, "Drafts");
        assert_eq!(window.chrome, WindowChrome::Compact);
        assert_eq!(platform.material(handle.id), Some(Material::Maris));
        assert!(platform.redraw_requested(handle.id));

        platform.destroy_window(handle.id);
        assert_eq!(platform.windows().count(), 0);
    }

    #[test]
    fn generic_platform_stores_actions_clipboard_and_dialog_result() {
        let mut platform = GenericPlatform::new();
        let actions = vec![ActionDescriptor::new("notes.new", "New")];

        platform.register_actions(&actions);
        platform.write_clipboard(ClipboardData::text("copied"));
        platform.set_next_file_dialog_result(FileDialogResult::selected("notes.txt"));

        assert_eq!(platform.actions(), actions.as_slice());
        assert_eq!(
            platform.read_clipboard().map(ClipboardData::into_text),
            Some("copied".to_string())
        );
        assert_eq!(
            platform
                .open_file_dialog(FileDialogOptions::open_file())
                .first_path()
                .and_then(|path| path.to_str()),
            Some("notes.txt")
        );
        assert!(
            platform
                .open_file_dialog(FileDialogOptions::open_file())
                .is_cancelled()
        );
    }
}
