use std::{cell::RefCell, collections::BTreeMap, path::PathBuf};

use stuk_actions::{ActionDescriptor, Shortcut};
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrayIcon {
    pub id: String,
    pub title: String,
    pub icon_path: Option<PathBuf>,
    pub tooltip: Option<String>,
    pub menu: Vec<TrayMenuItem>,
}

impl TrayIcon {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            icon_path: None,
            tooltip: None,
            menu: Vec::new(),
        }
    }

    pub fn icon_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.icon_path = Some(path.into());
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn menu_item(mut self, item: TrayMenuItem) -> Self {
        self.menu.push(item);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrayMenuItem {
    pub id: String,
    pub label: String,
    pub action: Option<String>,
    pub enabled: bool,
    pub separator: bool,
}

impl TrayMenuItem {
    pub fn action(
        id: impl Into<String>,
        label: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            action: Some(action.into()),
            enabled: true,
            separator: false,
        }
    }

    pub fn separator(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            action: None,
            enabled: false,
            separator: true,
        }
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutostartEntry {
    pub id: String,
    pub name: String,
    pub command: String,
    pub enabled: bool,
}

impl AutostartEntry {
    pub fn new(id: impl Into<String>, name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            command: command.into(),
            enabled: true,
        }
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlobalShortcutRegistration {
    pub id: String,
    pub shortcut: Shortcut,
    pub action: String,
    pub app_id: Option<String>,
    pub app_name: Option<String>,
    pub description: Option<String>,
    pub desktop_command: Option<String>,
}

impl GlobalShortcutRegistration {
    pub fn new(id: impl Into<String>, shortcut: Shortcut, action: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            shortcut,
            action: action.into(),
            app_id: None,
            app_name: None,
            description: None,
            desktop_command: None,
        }
    }

    pub fn app_id(mut self, app_id: impl Into<String>) -> Self {
        self.app_id = Some(app_id.into());
        self
    }

    pub fn app_name(mut self, app_name: impl Into<String>) -> Self {
        self.app_name = Some(app_name.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn desktop_command(mut self, command: impl Into<String>) -> Self {
        self.desktop_command = Some(command.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeepLinkRegistration {
    pub id: String,
    pub schemes: Vec<String>,
}

impl DeepLinkRegistration {
    pub fn new(
        id: impl Into<String>,
        schemes: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            id: id.into(),
            schemes: schemes.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeMessagingHost {
    pub id: String,
    pub name: String,
    pub executable: PathBuf,
    pub allowed_origins: Vec<String>,
}

impl NativeMessagingHost {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        executable: impl Into<PathBuf>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            executable: executable.into(),
            allowed_origins: Vec::new(),
        }
    }

    pub fn allow_origin(mut self, origin: impl Into<String>) -> Self {
        self.allowed_origins.push(origin.into());
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SingleInstancePolicy {
    #[default]
    AllowMultiple,
    ReuseExisting,
    FocusExisting,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrayActivation {
    pub tray_id: String,
    pub item_id: Option<String>,
    pub action: Option<String>,
}

impl TrayActivation {
    pub fn new(tray_id: impl Into<String>) -> Self {
        Self {
            tray_id: tray_id.into(),
            item_id: None,
            action: None,
        }
    }

    pub fn item(
        tray_id: impl Into<String>,
        item_id: impl Into<String>,
        action: Option<String>,
    ) -> Self {
        Self {
            tray_id: tray_id.into(),
            item_id: Some(item_id.into()),
            action,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlobalShortcutActivation {
    pub id: String,
    pub action: String,
    pub activation_token: Option<String>,
}

impl GlobalShortcutActivation {
    pub fn new(id: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            action: action.into(),
            activation_token: None,
        }
    }

    pub fn activation_token(mut self, token: impl Into<String>) -> Self {
        self.activation_token = Some(token.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SingleInstanceActivation {
    pub policy: SingleInstancePolicy,
    pub arguments: Vec<String>,
    pub working_directory: Option<PathBuf>,
    pub activation_token: Option<String>,
}

impl SingleInstanceActivation {
    pub fn new(policy: SingleInstancePolicy, arguments: Vec<String>) -> Self {
        Self {
            policy,
            arguments,
            working_directory: None,
            activation_token: None,
        }
    }

    pub fn working_directory(mut self, directory: impl Into<PathBuf>) -> Self {
        self.working_directory = Some(directory.into());
        self
    }

    pub fn activation_token(mut self, token: impl Into<String>) -> Self {
        self.activation_token = Some(token.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlatformEvent {
    Tray(TrayActivation),
    GlobalShortcut(GlobalShortcutActivation),
    SingleInstance(SingleInstanceActivation),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CredentialKey {
    pub service: String,
    pub account: String,
}

impl CredentialKey {
    pub fn new(service: impl Into<String>, account: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            account: account.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CredentialSecret {
    Text(String),
    Bytes(Vec<u8>),
}

impl CredentialSecret {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }

    pub fn bytes(bytes: impl Into<Vec<u8>>) -> Self {
        Self::Bytes(bytes.into())
    }

    pub fn into_bytes(self) -> Vec<u8> {
        match self {
            Self::Text(text) => text.into_bytes(),
            Self::Bytes(bytes) => bytes,
        }
    }
}

pub trait Platform {
    fn create_window(&mut self, options: WindowOptions) -> Result<WindowHandle, PlatformError>;
    fn destroy_window(&mut self, window: WindowId);
    fn request_redraw(&mut self, window: WindowId);
    fn set_title(&mut self, window: WindowId, title: &str);
    fn set_material(&mut self, window: WindowId, material: Material);
    fn set_chrome(&mut self, window: WindowId, chrome: WindowChrome);
    fn set_window_visible(&mut self, window: WindowId, visible: bool) -> bool {
        let _ = (window, visible);
        false
    }
    fn present_window(&mut self, window: WindowId) -> bool {
        let _ = window;
        false
    }
    fn set_window_always_on_top(&mut self, window: WindowId, always_on_top: bool) -> bool {
        let _ = (window, always_on_top);
        false
    }
    fn register_actions(&mut self, actions: &[ActionDescriptor]);
    fn read_clipboard(&self) -> Option<ClipboardData>;
    fn write_clipboard(&self, data: ClipboardData);
    fn open_file_dialog(&self, options: FileDialogOptions) -> FileDialogResult;
    fn platform_capabilities(&self) -> PlatformCapabilities;

    fn backend(&self) -> BackendDescriptor {
        BackendDescriptor::generic()
    }

    fn set_tray_icon(&mut self, icon: TrayIcon) -> bool {
        let _ = icon;
        false
    }

    fn remove_tray_icon(&mut self, id: &str) -> bool {
        let _ = id;
        false
    }

    fn set_autostart(&mut self, entry: AutostartEntry) -> bool {
        let _ = entry;
        false
    }

    fn register_global_shortcut(&mut self, registration: GlobalShortcutRegistration) -> bool {
        let _ = registration;
        false
    }

    fn unregister_global_shortcut(&mut self, id: &str) -> bool {
        let _ = id;
        false
    }

    fn register_deep_links(&mut self, registration: DeepLinkRegistration) -> bool {
        let _ = registration;
        false
    }

    fn register_native_messaging_host(&mut self, host: NativeMessagingHost) -> bool {
        let _ = host;
        false
    }

    fn set_single_instance_policy(&mut self, policy: SingleInstancePolicy) -> bool {
        let _ = policy;
        false
    }

    fn take_platform_events(&mut self) -> Vec<PlatformEvent> {
        Vec::new()
    }

    fn write_credential(&self, key: &CredentialKey, secret: CredentialSecret) -> bool {
        let _ = (key, secret);
        false
    }

    fn read_credential(&self, key: &CredentialKey) -> Option<CredentialSecret> {
        let _ = key;
        None
    }

    fn delete_credential(&self, key: &CredentialKey) -> bool {
        let _ = key;
        false
    }
}

#[derive(Debug)]
pub struct GenericPlatform {
    backend: BackendDescriptor,
    capabilities: PlatformCapabilities,
    next_window_id: u64,
    windows: BTreeMap<WindowId, GenericWindow>,
    actions: Vec<ActionDescriptor>,
    tray_icons: BTreeMap<String, TrayIcon>,
    autostart_entries: BTreeMap<String, AutostartEntry>,
    global_shortcuts: BTreeMap<String, GlobalShortcutRegistration>,
    deep_links: BTreeMap<String, DeepLinkRegistration>,
    native_messaging_hosts: BTreeMap<String, NativeMessagingHost>,
    single_instance_policy: SingleInstancePolicy,
    platform_events: RefCell<Vec<PlatformEvent>>,
    credentials: RefCell<BTreeMap<CredentialKey, CredentialSecret>>,
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
            tray_icons: BTreeMap::new(),
            autostart_entries: BTreeMap::new(),
            global_shortcuts: BTreeMap::new(),
            deep_links: BTreeMap::new(),
            native_messaging_hosts: BTreeMap::new(),
            single_instance_policy: SingleInstancePolicy::AllowMultiple,
            platform_events: RefCell::new(Vec::new()),
            credentials: RefCell::new(BTreeMap::new()),
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

    pub fn tray_icons(&self) -> impl Iterator<Item = (&str, &TrayIcon)> {
        self.tray_icons.iter().map(|(id, icon)| (id.as_str(), icon))
    }

    pub fn autostart_entries(&self) -> impl Iterator<Item = (&str, &AutostartEntry)> {
        self.autostart_entries
            .iter()
            .map(|(id, entry)| (id.as_str(), entry))
    }

    pub fn global_shortcuts(&self) -> impl Iterator<Item = (&str, &GlobalShortcutRegistration)> {
        self.global_shortcuts
            .iter()
            .map(|(id, shortcut)| (id.as_str(), shortcut))
    }

    pub fn deep_links(&self) -> impl Iterator<Item = (&str, &DeepLinkRegistration)> {
        self.deep_links
            .iter()
            .map(|(id, links)| (id.as_str(), links))
    }

    pub fn native_messaging_hosts(&self) -> impl Iterator<Item = (&str, &NativeMessagingHost)> {
        self.native_messaging_hosts
            .iter()
            .map(|(id, host)| (id.as_str(), host))
    }

    pub fn single_instance_policy(&self) -> SingleInstancePolicy {
        self.single_instance_policy
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

    pub fn push_platform_event(&self, event: PlatformEvent) {
        self.platform_events.borrow_mut().push(event);
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

    fn set_window_visible(&mut self, window: WindowId, visible: bool) -> bool {
        let Some(window) = self.windows.get_mut(&window) else {
            return false;
        };
        window.options.visible = visible;
        true
    }

    fn present_window(&mut self, window: WindowId) -> bool {
        let Some(window) = self.windows.get_mut(&window) else {
            return false;
        };
        window.options.visible = true;
        window.options.active = true;
        true
    }

    fn set_window_always_on_top(&mut self, window: WindowId, always_on_top: bool) -> bool {
        let Some(window) = self.windows.get_mut(&window) else {
            return false;
        };
        window.options.always_on_top = always_on_top;
        true
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

    fn set_tray_icon(&mut self, icon: TrayIcon) -> bool {
        if !self.capabilities.tray_icons {
            return false;
        }
        self.tray_icons.insert(icon.id.clone(), icon);
        true
    }

    fn remove_tray_icon(&mut self, id: &str) -> bool {
        if !self.capabilities.tray_icons {
            return false;
        }
        self.tray_icons.remove(id).is_some()
    }

    fn set_autostart(&mut self, entry: AutostartEntry) -> bool {
        if !self.capabilities.autostart {
            return false;
        }
        self.autostart_entries.insert(entry.id.clone(), entry);
        true
    }

    fn register_global_shortcut(&mut self, registration: GlobalShortcutRegistration) -> bool {
        if !self.capabilities.global_shortcuts {
            return false;
        }
        self.global_shortcuts
            .insert(registration.id.clone(), registration);
        true
    }

    fn unregister_global_shortcut(&mut self, id: &str) -> bool {
        if !self.capabilities.global_shortcuts {
            return false;
        }
        self.global_shortcuts.remove(id).is_some()
    }

    fn register_deep_links(&mut self, registration: DeepLinkRegistration) -> bool {
        if !self.capabilities.deep_links {
            return false;
        }
        self.deep_links
            .insert(registration.id.clone(), registration);
        true
    }

    fn register_native_messaging_host(&mut self, host: NativeMessagingHost) -> bool {
        if !self.capabilities.native_messaging {
            return false;
        }
        self.native_messaging_hosts.insert(host.id.clone(), host);
        true
    }

    fn set_single_instance_policy(&mut self, policy: SingleInstancePolicy) -> bool {
        if !self.capabilities.single_instance {
            return false;
        }
        self.single_instance_policy = policy;
        true
    }

    fn take_platform_events(&mut self) -> Vec<PlatformEvent> {
        self.platform_events.borrow_mut().drain(..).collect()
    }

    fn write_credential(&self, key: &CredentialKey, secret: CredentialSecret) -> bool {
        if !self.capabilities.credential_storage && !self.capabilities.secure_storage {
            return false;
        }
        self.credentials.borrow_mut().insert(key.clone(), secret);
        true
    }

    fn read_credential(&self, key: &CredentialKey) -> Option<CredentialSecret> {
        if !self.capabilities.credential_storage && !self.capabilities.secure_storage {
            return None;
        }
        self.credentials.borrow().get(key).cloned()
    }

    fn delete_credential(&self, key: &CredentialKey) -> bool {
        if !self.capabilities.credential_storage && !self.capabilities.secure_storage {
            return false;
        }
        self.credentials.borrow_mut().remove(key).is_some()
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
        platform.set_window_visible(handle.id, false);
        platform.set_window_always_on_top(handle.id, true);
        platform.present_window(handle.id);
        platform.request_redraw(handle.id);

        let window = platform.windows().next().unwrap().1;
        assert_eq!(window.title, "Drafts");
        assert_eq!(window.chrome, WindowChrome::Compact);
        assert!(window.visible);
        assert!(window.active);
        assert!(window.always_on_top);
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

    #[test]
    fn generic_platform_tracks_desktop_services_when_supported() {
        let mut platform = GenericPlatform::with_capabilities(PlatformCapabilities {
            tray_icons: true,
            autostart: true,
            global_shortcuts: true,
            deep_links: true,
            native_messaging: true,
            single_instance: true,
            ..PlatformCapabilities::generic()
        });

        assert!(platform.set_tray_icon(
            TrayIcon::new("main", "Klarkey").menu_item(TrayMenuItem::action(
                "show",
                "Show",
                "palette.show"
            ))
        ));
        assert!(platform.set_autostart(AutostartEntry::new(
            "klarkey",
            "Klarkey",
            "klarkey --background"
        )));
        assert!(
            platform.register_global_shortcut(GlobalShortcutRegistration::new(
                "palette",
                Shortcut::parse("Ctrl+Space").unwrap(),
                "palette.toggle"
            ))
        );
        assert!(platform.register_deep_links(DeepLinkRegistration::new("main", ["klarkey"])));
        assert!(
            platform.register_native_messaging_host(
                NativeMessagingHost::new("extension", "Klarkey Extension", "/usr/bin/klarkey")
                    .allow_origin("chrome-extension://example/")
            )
        );
        assert!(platform.set_single_instance_policy(SingleInstancePolicy::FocusExisting));

        assert_eq!(platform.tray_icons().count(), 1);
        assert_eq!(platform.autostart_entries().count(), 1);
        assert_eq!(platform.global_shortcuts().count(), 1);
        assert_eq!(platform.deep_links().count(), 1);
        assert_eq!(platform.native_messaging_hosts().count(), 1);
        assert_eq!(
            platform.single_instance_policy(),
            SingleInstancePolicy::FocusExisting
        );
    }
}
