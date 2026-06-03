use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, OnceLock},
    thread::{self, JoinHandle},
};

use futures_util::StreamExt;
use ksni::blocking::TrayMethods;
use stuk_actions::Shortcut;
use stuk_platform::{
    AutostartEntry, CredentialKey, CredentialSecret, DeepLinkRegistration,
    GlobalShortcutActivation, GlobalShortcutRegistration, NativeMessagingHost, PlatformEvent,
    SingleInstancePolicy, TrayActivation, TrayIcon, TrayMenuItem,
};

use crate::{
    desktop_files::{
        data_home, desktop_entry, register_deep_links, register_native_messaging_host,
        sanitize_desktop_id, write_autostart_entry, write_file,
    },
    single_instance::{SingleInstanceGuard, SingleInstanceSetup},
};

pub(super) type EventQueue = Arc<Mutex<Vec<PlatformEvent>>>;

pub(crate) struct LinuxDesktopServices {
    events: EventQueue,
    tray: Mutex<Option<TrayRuntime>>,
    shortcuts: Mutex<BTreeMap<String, ShortcutRuntime>>,
    single_instance: Mutex<Option<SingleInstanceGuard>>,
}

impl std::fmt::Debug for LinuxDesktopServices {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LinuxDesktopServices")
            .field(
                "events",
                &self.events.lock().map(|events| events.len()).ok(),
            )
            .finish_non_exhaustive()
    }
}

impl Default for LinuxDesktopServices {
    fn default() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            tray: Mutex::new(None),
            shortcuts: Mutex::new(BTreeMap::new()),
            single_instance: Mutex::new(None),
        }
    }
}

impl LinuxDesktopServices {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn take_events(&self) -> Vec<PlatformEvent> {
        self.events
            .lock()
            .map(|mut events| events.drain(..).collect())
            .unwrap_or_default()
    }

    pub(crate) fn set_tray_icon(&self, icon: &TrayIcon) -> bool {
        let tray = LinuxTray {
            icon: icon.clone(),
            events: Arc::clone(&self.events),
        };
        let handle = match tray.assume_sni_available(true).spawn() {
            Ok(handle) => handle,
            Err(_) => return false,
        };
        let Ok(mut active) = self.tray.lock() else {
            return false;
        };
        if let Some(existing) = active.take() {
            existing.shutdown();
        }
        *active = Some(TrayRuntime { handle });
        true
    }

    pub(crate) fn remove_tray_icon(&self, id: &str) -> bool {
        let Ok(mut active) = self.tray.lock() else {
            return false;
        };
        let Some(existing) = active.take() else {
            return false;
        };
        if existing.id() != id {
            *active = Some(existing);
            return false;
        }
        existing.shutdown();
        true
    }

    pub(crate) fn set_autostart(&self, entry: &AutostartEntry) -> bool {
        write_autostart_entry(entry).is_ok()
    }

    pub(crate) fn register_global_shortcut(
        &self,
        registration: &GlobalShortcutRegistration,
    ) -> bool {
        let registration = registration.clone();
        let events = Arc::clone(&self.events);
        let shortcut_id = registration.id.clone();
        let thread = thread::spawn(move || {
            let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };
            let _ = runtime.block_on(run_portal_shortcut(registration, events));
        });
        let Ok(mut shortcuts) = self.shortcuts.lock() else {
            return false;
        };
        shortcuts.insert(shortcut_id, ShortcutRuntime { thread });
        true
    }

    pub(crate) fn unregister_global_shortcut(&self, id: &str) -> bool {
        self.shortcuts
            .lock()
            .map(|mut shortcuts| shortcuts.remove(id).is_some())
            .unwrap_or(false)
    }

    pub(crate) fn register_deep_links(&self, registration: &DeepLinkRegistration) -> bool {
        register_deep_links(registration).is_ok()
    }

    pub(crate) fn register_native_messaging_host(&self, host: &NativeMessagingHost) -> bool {
        register_native_messaging_host(host).is_ok()
    }

    pub(crate) fn set_single_instance_policy(&self, policy: SingleInstancePolicy) -> bool {
        let Ok(mut active) = self.single_instance.lock() else {
            return false;
        };
        *active = None;
        if policy == SingleInstancePolicy::AllowMultiple {
            return true;
        }
        match SingleInstanceGuard::acquire(policy, Arc::clone(&self.events)) {
            SingleInstanceSetup::Primary(guard) => {
                *active = Some(guard);
                true
            }
            SingleInstanceSetup::AlreadyRunning => false,
            SingleInstanceSetup::Failed => false,
        }
    }

    pub(crate) fn write_credential(&self, key: &CredentialKey, secret: CredentialSecret) -> bool {
        let Some(entry) = credential_entry(key) else {
            return false;
        };
        match secret {
            CredentialSecret::Text(text) => entry.set_password(&text).is_ok(),
            CredentialSecret::Bytes(bytes) => entry.set_secret(&bytes).is_ok(),
        }
    }

    pub(crate) fn read_credential(&self, key: &CredentialKey) -> Option<CredentialSecret> {
        let entry = credential_entry(key)?;
        entry.get_secret().ok().map(secret_from_bytes)
    }

    pub(crate) fn delete_credential(&self, key: &CredentialKey) -> bool {
        credential_entry(key).is_some_and(|entry| entry.delete_credential().is_ok())
    }
}

struct TrayRuntime {
    handle: ksni::blocking::Handle<LinuxTray>,
}

impl TrayRuntime {
    fn id(&self) -> String {
        self.handle
            .update(|tray| tray.icon.id.clone())
            .unwrap_or_default()
    }

    fn shutdown(self) {
        self.handle.shutdown().wait();
    }
}

struct ShortcutRuntime {
    thread: JoinHandle<()>,
}

impl Drop for ShortcutRuntime {
    fn drop(&mut self) {
        let _ = self.thread.thread().id();
    }
}

#[derive(Clone)]
struct LinuxTray {
    icon: TrayIcon,
    events: EventQueue,
}

impl LinuxTray {
    fn push(&self, event: PlatformEvent) {
        if let Ok(mut events) = self.events.lock() {
            events.push(event);
        }
    }
}

impl ksni::Tray for LinuxTray {
    fn id(&self) -> String {
        sanitize_desktop_id(&self.icon.id)
    }

    fn title(&self) -> String {
        self.icon.title.clone()
    }

    fn icon_name(&self) -> String {
        self.icon
            .icon_path
            .as_ref()
            .and_then(|path| path.file_stem())
            .and_then(|name| name.to_str())
            .map(str::to_string)
            .unwrap_or_else(|| "application-x-executable".to_string())
    }

    fn icon_theme_path(&self) -> String {
        self.icon
            .icon_path
            .as_ref()
            .and_then(|path| path.parent())
            .map(|path| path.display().to_string())
            .unwrap_or_default()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            title: self.icon.title.clone(),
            description: self.icon.tooltip.clone().unwrap_or_default(),
            ..ksni::ToolTip::default()
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        self.push(PlatformEvent::Tray(TrayActivation::new(
            self.icon.id.clone(),
        )));
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        self.icon.menu.iter().map(tray_menu_item).collect()
    }
}

fn tray_menu_item(item: &TrayMenuItem) -> ksni::MenuItem<LinuxTray> {
    if item.separator {
        return ksni::MenuItem::Separator;
    }
    let item_id = item.id.clone();
    let action = item.action.clone();
    let label = item.label.clone();
    let enabled = item.enabled;
    ksni::menu::StandardItem {
        label,
        enabled,
        activate: Box::new(move |tray: &mut LinuxTray| {
            tray.push(PlatformEvent::Tray(TrayActivation::item(
                tray.icon.id.clone(),
                item_id.clone(),
                action.clone(),
            )));
        }),
        ..ksni::menu::StandardItem::default()
    }
    .into()
}

async fn run_portal_shortcut(
    registration: GlobalShortcutRegistration,
    events: EventQueue,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use ashpd::desktop::{
        CreateSessionOptions,
        global_shortcuts::{BindShortcutsOptions, GlobalShortcuts, NewShortcut},
    };

    ensure_portal_app_registration(&registration).await?;

    let Some(trigger) = portal_trigger_for_shortcut(&registration.shortcut) else {
        return Err("global shortcut is not supported by the Linux portal backend".into());
    };
    let portal = GlobalShortcuts::new().await?;
    let session = portal
        .create_session(CreateSessionOptions::default())
        .await?;
    let mut activations = portal.receive_activated().await?;
    let description = registration
        .description
        .as_deref()
        .unwrap_or(&registration.action);
    let shortcut = NewShortcut::new(registration.id.as_str(), description)
        .preferred_trigger(Some(trigger.as_str()));
    let request = portal
        .bind_shortcuts(&session, &[shortcut], None, BindShortcutsOptions::default())
        .await?;
    let response = request.response()?;
    if !response
        .shortcuts()
        .iter()
        .any(|shortcut| shortcut.id() == registration.id)
    {
        return Err("the portal did not bind the requested shortcut".into());
    }

    while let Some(event) = activations.next().await {
        if event.shortcut_id() != registration.id {
            continue;
        }
        let mut activation =
            GlobalShortcutActivation::new(registration.id.clone(), registration.action.clone());
        if let Some(token) = activation_token_from_options(event.options()) {
            activation = activation.activation_token(token);
        }
        if let Ok(mut events) = events.lock() {
            events.push(PlatformEvent::GlobalShortcut(activation));
        }
    }
    Ok(())
}

fn portal_trigger_for_shortcut(shortcut: &Shortcut) -> Option<String> {
    let mut parts = Vec::new();
    if shortcut.modifiers.ctrl {
        parts.push("CTRL".to_string());
    }
    if shortcut.modifiers.alt {
        parts.push("ALT".to_string());
    }
    if shortcut.modifiers.shift {
        parts.push("SHIFT".to_string());
    }
    if shortcut.modifiers.meta {
        parts.push("LOGO".to_string());
    }
    let mut key = shortcut.key.trim().to_string();
    if key.is_empty() {
        return None;
    }
    if key.len() == 1 && key.is_ascii() {
        key.make_ascii_lowercase();
    }
    parts.push(key);
    Some(parts.join("+"))
}

fn activation_token_from_options(
    options: &std::collections::HashMap<String, ashpd::zvariant::OwnedValue>,
) -> Option<String> {
    let value = options.get("activation_token")?.try_clone().ok()?;
    String::try_from(value)
        .ok()
        .filter(|token| !token.trim().is_empty())
}

async fn ensure_portal_app_registration(
    registration: &GlobalShortcutRegistration,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let Some(app_id) = registration.app_id.as_deref() else {
        return Ok(());
    };
    if let Some(command) = registration.desktop_command.as_deref() {
        write_file(
            data_home()?
                .join("applications")
                .join(format!("{}.desktop", sanitize_desktop_id(app_id))),
            &desktop_entry(
                app_id,
                registration.description.as_deref().unwrap_or(app_id),
                command,
                &[],
            ),
        )?;
    }
    let app_id = ashpd::AppID::try_from(app_id)?;
    match ashpd::register_host_app(app_id).await {
        Ok(()) => Ok(()),
        Err(error) if portal_app_already_registered(&error) => Ok(()),
        Err(error) => Err(Box::new(error)),
    }
}

fn portal_app_already_registered(error: &ashpd::Error) -> bool {
    let message = error.to_string();
    message.contains("already associated") || message.contains("already registered")
}

fn credential_entry(key: &CredentialKey) -> Option<keyring_core::Entry> {
    if !ensure_keyring_store() {
        return None;
    }
    keyring_core::Entry::new(&key.service, &key.account).ok()
}

fn ensure_keyring_store() -> bool {
    static STORE_READY: OnceLock<bool> = OnceLock::new();
    *STORE_READY.get_or_init(|| match dbus_secret_service_keyring_store::Store::new() {
        Ok(store) => {
            keyring_core::set_default_store(store);
            true
        }
        Err(_) => false,
    })
}

fn secret_from_bytes(bytes: Vec<u8>) -> CredentialSecret {
    match String::from_utf8(bytes) {
        Ok(text) => CredentialSecret::Text(text),
        Err(error) => CredentialSecret::Bytes(error.into_bytes()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn secure_storage_round_trips_through_secret_service() {
        let services = LinuxDesktopServices::new();
        let key = CredentialKey::new(
            "dev.stuk.secure-storage-smoke",
            format!(
                "{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ),
        );

        assert!(services.write_credential(&key, CredentialSecret::text("secret")));
        assert_eq!(
            services.read_credential(&key),
            Some(CredentialSecret::Text("secret".to_string()))
        );
        assert!(services.delete_credential(&key));
    }
}
