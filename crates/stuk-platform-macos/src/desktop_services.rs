#[cfg(not(target_os = "macos"))]
#[derive(Debug, Default)]
pub(crate) struct MacosDesktopServices;

#[cfg(not(target_os = "macos"))]
impl MacosDesktopServices {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn take_events(&self) -> Vec<stuk_platform::PlatformEvent> {
        Vec::new()
    }

    pub(crate) fn set_tray_icon(&self, _icon: &stuk_platform::TrayIcon) -> bool {
        false
    }

    pub(crate) fn remove_tray_icon(&self, _id: &str) -> bool {
        false
    }

    pub(crate) fn set_autostart(&self, _entry: &stuk_platform::AutostartEntry) -> bool {
        false
    }

    pub(crate) fn register_global_shortcut(
        &self,
        _registration: &stuk_platform::GlobalShortcutRegistration,
    ) -> bool {
        false
    }

    pub(crate) fn unregister_global_shortcut(&self, _id: &str) -> bool {
        false
    }

    pub(crate) fn register_deep_links(
        &self,
        _registration: &stuk_platform::DeepLinkRegistration,
    ) -> bool {
        false
    }

    pub(crate) fn register_native_messaging_host(
        &self,
        _host: &stuk_platform::NativeMessagingHost,
    ) -> bool {
        false
    }

    pub(crate) fn set_single_instance_policy(
        &self,
        _policy: stuk_platform::SingleInstancePolicy,
    ) -> bool {
        false
    }

    pub(crate) fn write_credential(
        &self,
        _key: &stuk_platform::CredentialKey,
        _secret: stuk_platform::CredentialSecret,
    ) -> bool {
        false
    }

    pub(crate) fn read_credential(
        &self,
        _key: &stuk_platform::CredentialKey,
    ) -> Option<stuk_platform::CredentialSecret> {
        None
    }

    pub(crate) fn delete_credential(&self, _key: &stuk_platform::CredentialKey) -> bool {
        false
    }
}

#[cfg(target_os = "macos")]
mod native {
    use std::{
        cell::RefCell,
        collections::BTreeMap,
        sync::{Arc, Mutex, OnceLock},
    };

    use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState, hotkey::HotKey};
    use stuk_platform::{
        AutostartEntry, CredentialKey, CredentialSecret, DeepLinkRegistration,
        GlobalShortcutActivation, GlobalShortcutRegistration, NativeMessagingHost, PlatformEvent,
        SingleInstancePolicy, TrayActivation, TrayIcon,
    };
    use tray_icon::{
        MouseButtonState, TrayIconBuilder, TrayIconEvent,
        menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    };

    use crate::{
        desktop_files::{register_deep_links, register_native_messaging_host, write_launch_agent},
        single_instance::{SingleInstanceGuard, SingleInstanceSetup},
    };

    type EventQueue = Arc<Mutex<Vec<PlatformEvent>>>;

    pub(crate) struct MacosDesktopServices {
        events: EventQueue,
        tray: RefCell<Option<TrayRuntime>>,
        menu_actions: RefCell<BTreeMap<String, TrayMenuAction>>,
        shortcut_manager: RefCell<Option<GlobalHotKeyManager>>,
        shortcuts: RefCell<BTreeMap<String, ShortcutRuntime>>,
        single_instance: RefCell<Option<SingleInstanceGuard>>,
    }

    impl std::fmt::Debug for MacosDesktopServices {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter
                .debug_struct("MacosDesktopServices")
                .field(
                    "events",
                    &self.events.lock().map(|events| events.len()).ok(),
                )
                .field("shortcuts", &self.shortcuts.borrow().len())
                .finish_non_exhaustive()
        }
    }

    impl Default for MacosDesktopServices {
        fn default() -> Self {
            Self {
                events: Arc::new(Mutex::new(Vec::new())),
                tray: RefCell::new(None),
                menu_actions: RefCell::new(BTreeMap::new()),
                shortcut_manager: RefCell::new(None),
                shortcuts: RefCell::new(BTreeMap::new()),
                single_instance: RefCell::new(None),
            }
        }
    }

    impl MacosDesktopServices {
        pub(crate) fn new() -> Self {
            Self::default()
        }

        pub(crate) fn take_events(&self) -> Vec<PlatformEvent> {
            self.drain_tray_events();
            self.drain_menu_events();
            self.drain_shortcut_events();
            self.events
                .lock()
                .map(|mut events| events.drain(..).collect())
                .unwrap_or_default()
        }

        pub(crate) fn set_tray_icon(&self, icon: &TrayIcon) -> bool {
            let menu = match tray_menu(icon, &mut self.menu_actions.borrow_mut()) {
                Some(menu) => menu,
                None => return false,
            };
            let mut builder = TrayIconBuilder::new()
                .with_id(icon.id.clone())
                .with_title(&icon.title)
                .with_menu(Box::new(menu))
                .with_menu_on_left_click(false)
                .with_icon_as_template(true);
            if let Some(tooltip) = icon.tooltip.as_deref() {
                builder = builder.with_tooltip(tooltip);
            }
            if let Some(image) = tray_image(icon) {
                builder = builder.with_icon(image);
            }
            let Ok(native) = builder.build() else {
                return false;
            };
            *self.tray.borrow_mut() = Some(TrayRuntime {
                id: icon.id.clone(),
                _native: native,
            });
            true
        }

        pub(crate) fn remove_tray_icon(&self, id: &str) -> bool {
            let mut tray = self.tray.borrow_mut();
            if tray.as_ref().is_some_and(|tray| tray.id == id) {
                *tray = None;
                self.menu_actions
                    .borrow_mut()
                    .retain(|_, action| action.tray_id != id);
                true
            } else {
                false
            }
        }

        pub(crate) fn set_autostart(&self, entry: &AutostartEntry) -> bool {
            write_launch_agent(entry).is_ok()
        }

        pub(crate) fn register_global_shortcut(
            &self,
            registration: &GlobalShortcutRegistration,
        ) -> bool {
            let Some(hotkey) = shortcut_to_hotkey(&registration.shortcut) else {
                return false;
            };
            if self.shortcut_manager.borrow().is_none() {
                let Ok(manager) = GlobalHotKeyManager::new() else {
                    return false;
                };
                *self.shortcut_manager.borrow_mut() = Some(manager);
            }
            let registered = self
                .shortcut_manager
                .borrow()
                .as_ref()
                .is_some_and(|manager| manager.register(hotkey).is_ok());
            if !registered {
                return false;
            }
            self.shortcuts.borrow_mut().insert(
                registration.id.clone(),
                ShortcutRuntime {
                    hotkey,
                    action: registration.action.clone(),
                },
            );
            true
        }

        pub(crate) fn unregister_global_shortcut(&self, id: &str) -> bool {
            let Some(runtime) = self.shortcuts.borrow_mut().remove(id) else {
                return false;
            };
            self.shortcut_manager
                .borrow()
                .as_ref()
                .is_some_and(|manager| manager.unregister(runtime.hotkey).is_ok())
        }

        pub(crate) fn register_deep_links(&self, _registration: &DeepLinkRegistration) -> bool {
            register_deep_links(_registration).is_ok()
        }

        pub(crate) fn register_native_messaging_host(&self, host: &NativeMessagingHost) -> bool {
            register_native_messaging_host(host).is_ok()
        }

        pub(crate) fn set_single_instance_policy(&self, policy: SingleInstancePolicy) -> bool {
            *self.single_instance.borrow_mut() = None;
            if policy == SingleInstancePolicy::AllowMultiple {
                return true;
            }
            match SingleInstanceGuard::acquire(policy, Arc::clone(&self.events)) {
                SingleInstanceSetup::Primary(guard) => {
                    *self.single_instance.borrow_mut() = Some(guard);
                    true
                }
                SingleInstanceSetup::AlreadyRunning | SingleInstanceSetup::Failed => false,
            }
        }

        pub(crate) fn write_credential(
            &self,
            key: &CredentialKey,
            secret: CredentialSecret,
        ) -> bool {
            let Some(entry) = credential_entry(key) else {
                return false;
            };
            match secret {
                CredentialSecret::Text(text) => entry.set_password(&text).is_ok(),
                CredentialSecret::Bytes(bytes) => entry.set_secret(&bytes).is_ok(),
            }
        }

        pub(crate) fn read_credential(&self, key: &CredentialKey) -> Option<CredentialSecret> {
            credential_entry(key)?
                .get_secret()
                .ok()
                .map(secret_from_bytes)
        }

        pub(crate) fn delete_credential(&self, key: &CredentialKey) -> bool {
            credential_entry(key).is_some_and(|entry| entry.delete_credential().is_ok())
        }

        fn push(&self, event: PlatformEvent) {
            if let Ok(mut events) = self.events.lock() {
                events.push(event);
            }
        }

        fn drain_tray_events(&self) {
            while let Ok(event) = TrayIconEvent::receiver().try_recv() {
                match event {
                    TrayIconEvent::Click {
                        id, button_state, ..
                    } if button_state == MouseButtonState::Up => {
                        self.push(PlatformEvent::Tray(TrayActivation::new(id.0)));
                    }
                    TrayIconEvent::DoubleClick { id, .. } => {
                        self.push(PlatformEvent::Tray(TrayActivation::new(id.0)));
                    }
                    _ => {}
                }
            }
        }

        fn drain_menu_events(&self) {
            while let Ok(event) = MenuEvent::receiver().try_recv() {
                let key = event.id.0;
                if let Some(action) = self.menu_actions.borrow().get(&key).cloned() {
                    self.push(PlatformEvent::Tray(TrayActivation::item(
                        action.tray_id,
                        action.item_id,
                        action.action,
                    )));
                }
            }
        }

        fn drain_shortcut_events(&self) {
            while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                if event.state != HotKeyState::Pressed {
                    continue;
                }
                if let Some((id, runtime)) = self
                    .shortcuts
                    .borrow()
                    .iter()
                    .find(|(_, runtime)| runtime.hotkey.id() == event.id)
                {
                    self.push(PlatformEvent::GlobalShortcut(
                        GlobalShortcutActivation::new(id.clone(), runtime.action.clone()),
                    ));
                }
            }
        }
    }

    struct TrayRuntime {
        id: String,
        _native: tray_icon::TrayIcon,
    }

    #[derive(Clone)]
    struct TrayMenuAction {
        tray_id: String,
        item_id: String,
        action: Option<String>,
    }

    struct ShortcutRuntime {
        hotkey: HotKey,
        action: String,
    }

    fn tray_menu(icon: &TrayIcon, actions: &mut BTreeMap<String, TrayMenuAction>) -> Option<Menu> {
        actions.retain(|_, action| action.tray_id != icon.id);
        let menu = Menu::new();
        for item in &icon.menu {
            if item.separator {
                if menu.append(&PredefinedMenuItem::separator()).is_err() {
                    return None;
                }
                continue;
            }
            let event_id = format!("{}:{}", icon.id, item.id);
            let native = MenuItem::with_id(&event_id, &item.label, item.enabled, None);
            if menu.append(&native).is_err() {
                return None;
            }
            actions.insert(
                event_id,
                TrayMenuAction {
                    tray_id: icon.id.clone(),
                    item_id: item.id.clone(),
                    action: item.action.clone(),
                },
            );
        }
        Some(menu)
    }

    fn tray_image(icon: &TrayIcon) -> Option<tray_icon::Icon> {
        if let Some(path) = icon.icon_path.as_deref() {
            if let Ok(image) = image::open(path) {
                let rgba = image.into_rgba8();
                let (width, height) = rgba.dimensions();
                return tray_icon::Icon::from_rgba(rgba.into_raw(), width, height).ok();
            }
        }
        let mut rgba = vec![0; 16 * 16 * 4];
        for pixel in rgba.chunks_mut(4) {
            pixel.copy_from_slice(&[220, 220, 220, 255]);
        }
        tray_icon::Icon::from_rgba(rgba, 16, 16).ok()
    }

    fn shortcut_to_hotkey(shortcut: &stuk_actions::Shortcut) -> Option<HotKey> {
        let mut parts = Vec::new();
        if shortcut.modifiers.ctrl {
            parts.push("Ctrl".to_string());
        }
        if shortcut.modifiers.alt {
            parts.push("Alt".to_string());
        }
        if shortcut.modifiers.shift {
            parts.push("Shift".to_string());
        }
        if shortcut.modifiers.meta {
            parts.push("Super".to_string());
        }
        parts.push(shortcut.key.clone());
        parts.join("+").parse().ok()
    }

    fn credential_entry(key: &CredentialKey) -> Option<keyring_core::Entry> {
        if !ensure_keychain_store() {
            return None;
        }
        keyring_core::Entry::new(&key.service, &key.account).ok()
    }

    fn ensure_keychain_store() -> bool {
        static STORE_READY: OnceLock<bool> = OnceLock::new();
        *STORE_READY.get_or_init(
            || match apple_native_keyring_store::keychain::Store::new() {
                Ok(store) => {
                    keyring_core::set_default_store(store);
                    true
                }
                Err(_) => false,
            },
        )
    }

    fn secret_from_bytes(bytes: Vec<u8>) -> CredentialSecret {
        match String::from_utf8(bytes) {
            Ok(text) => CredentialSecret::Text(text),
            Err(error) => CredentialSecret::Bytes(error.into_bytes()),
        }
    }
}

#[cfg(target_os = "macos")]
pub(crate) use native::MacosDesktopServices;
