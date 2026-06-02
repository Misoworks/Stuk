use std::{cell::RefCell, future::Future, rc::Rc, sync::Arc};

use stuk_actions::{ActionDescriptor, ActionRegistry, ActionRegistryError};
use stuk_layout::{Breakpoint, Responsive, Size};
use stuk_platform::{
    AppTarget, BackendDescriptor, NativeApp, PlatformCapabilities, PlatformError, RuntimeTarget,
    StaccatoSession, WindowOptions,
};
use stuk_settings::{SettingKind, SettingValue, SettingsSchema, SettingsStore, SettingsStoreError};
use stuk_style::Theme;
use thiserror::Error;

use crate::element::Element;
use crate::lower::{build_window, render_window};
use crate::window_chrome_render::{
    ACTION_WINDOW_CLOSE, ACTION_WINDOW_MINIMIZE, ACTION_WINDOW_TOGGLE_MAXIMIZE,
};
use crate::{
    async_state::{Mutation, Resource, mutation, resource, resource_value},
    session::{SessionCx, StaccatoCx},
    task::{TaskHandle, spawn_task},
};

pub type Result<T = ()> = std::result::Result<T, StukError>;

#[derive(Debug, Error)]
pub enum StukError {
    #[error("{0}")]
    Platform(#[from] PlatformError),
    #[error("{0}")]
    ActionRegistry(#[from] ActionRegistryError),
    #[error("app has no window")]
    MissingWindow,
}

#[derive(Clone, Debug)]
pub struct Cx {
    app_id: String,
    app_name: String,
    backend: BackendDescriptor,
    viewport_size: Size,
    settings_schema: Rc<SettingsSchema>,
    settings_store: Rc<RefCell<SettingsStore>>,
    staccato_session: Rc<RefCell<StaccatoSession>>,
}

impl Cx {
    pub(crate) fn new(app_id: &str, app_name: &str) -> Self {
        let settings_schema = Rc::new(SettingsSchema::new());
        let settings_store = Rc::new(RefCell::new(SettingsStore::new()));
        Self::with_settings(app_id, app_name, settings_schema, settings_store)
    }

    pub(crate) fn with_settings(
        app_id: &str,
        app_name: &str,
        settings_schema: Rc<SettingsSchema>,
        settings_store: Rc<RefCell<SettingsStore>>,
    ) -> Self {
        Self::with_settings_and_session(
            app_id,
            app_name,
            settings_schema,
            settings_store,
            Rc::new(RefCell::new(StaccatoSession::default())),
        )
    }

    pub(crate) fn with_settings_and_session(
        app_id: &str,
        app_name: &str,
        settings_schema: Rc<SettingsSchema>,
        settings_store: Rc<RefCell<SettingsStore>>,
        staccato_session: Rc<RefCell<StaccatoSession>>,
    ) -> Self {
        Self::with_settings_and_session_and_backend(
            app_id,
            app_name,
            settings_schema,
            settings_store,
            staccato_session,
            BackendDescriptor::current_native(),
        )
    }

    pub(crate) fn with_settings_and_session_and_backend(
        app_id: &str,
        app_name: &str,
        settings_schema: Rc<SettingsSchema>,
        settings_store: Rc<RefCell<SettingsStore>>,
        staccato_session: Rc<RefCell<StaccatoSession>>,
        backend: BackendDescriptor,
    ) -> Self {
        Self {
            app_id: app_id.to_string(),
            app_name: app_name.to_string(),
            backend,
            viewport_size: Size::default(),
            settings_schema,
            settings_store,
            staccato_session,
        }
    }

    pub(crate) fn set_viewport_size(&mut self, size: Size) {
        self.viewport_size = size;
    }

    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    pub fn app_name(&self) -> &str {
        &self.app_name
    }

    pub fn viewport_size(&self) -> Size {
        self.viewport_size
    }

    pub fn platform_backend(&self) -> &BackendDescriptor {
        &self.backend
    }

    pub fn platform_target(&self) -> RuntimeTarget {
        self.backend.target
    }

    pub fn capabilities(&self) -> PlatformCapabilities {
        self.backend.capabilities
    }

    pub fn is_desktop(&self) -> bool {
        self.backend.target.is_desktop()
    }

    pub fn is_mobile(&self) -> bool {
        self.backend.target.is_mobile()
    }

    pub fn is_web(&self) -> bool {
        self.backend.target.is_web()
    }

    pub fn app_target(&self) -> Option<AppTarget> {
        AppTarget::from_runtime_target(self.backend.target)
    }

    pub fn matches_target(&self, target: AppTarget) -> bool {
        target.matches(self.backend.target)
    }

    pub fn viewport_width(&self) -> f32 {
        self.viewport_size.width
    }

    pub fn breakpoint(&self) -> Breakpoint {
        Breakpoint::from_size(self.viewport_size)
    }

    pub fn is_at_least(&self, breakpoint: Breakpoint) -> bool {
        self.breakpoint().is_at_least(breakpoint)
    }

    pub fn responsive<T: Clone>(&self, value: &Responsive<T>) -> T {
        value.resolve(self.breakpoint())
    }

    pub fn settings_schema(&self) -> &SettingsSchema {
        self.settings_schema.as_ref()
    }

    pub fn settings_store(&self) -> SettingsStore {
        self.settings_store.borrow().clone()
    }

    pub fn setting(&self, id: &str) -> Option<SettingValue> {
        self.settings_store.borrow().get(id).cloned()
    }

    pub fn setting_bool(&self, id: &str) -> Option<bool> {
        self.settings_store.borrow().get_bool(id)
    }

    pub fn setting_number(&self, id: &str) -> Option<f64> {
        self.settings_store.borrow().get_number(id)
    }

    pub fn setting_text(&self, id: &str) -> Option<String> {
        self.settings_store
            .borrow()
            .get_text(id)
            .map(str::to_string)
    }

    pub fn set_setting(
        &mut self,
        id: &str,
        value: impl Into<SettingValue>,
    ) -> std::result::Result<(), SettingsStoreError> {
        self.settings_store
            .borrow_mut()
            .set(self.settings_schema.as_ref(), id, value)
    }

    pub fn apply_setting_action(&mut self, action_id: &str, prefix: &str) -> bool {
        let Some(setting_action) = action_id.strip_prefix(&format!("{prefix}.")) else {
            return false;
        };

        let target = self.settings_schema.definitions().find_map(|definition| {
            if setting_action == definition.id && definition.kind == SettingKind::Boolean {
                return Some((definition.id.clone(), None));
            }

            let value_prefix = format!("{}.", definition.id);
            setting_action
                .strip_prefix(&value_prefix)
                .map(|value| (definition.id.clone(), Some(value.to_string())))
        });

        match target {
            Some((id, None)) => {
                let next = !self.setting_bool(&id).unwrap_or_default();
                self.set_setting(&id, next).is_ok()
            }
            Some((id, Some(value))) => self.set_setting(&id, value).is_ok(),
            None => false,
        }
    }

    pub fn theme(&self) -> Theme {
        Theme::from_settings(
            self.setting_text("appearance.theme").as_deref(),
            self.setting_text("appearance.density").as_deref(),
        )
    }

    pub fn staccato(&self) -> StaccatoCx {
        StaccatoCx::new(Rc::clone(&self.staccato_session))
    }

    pub fn session(&self) -> SessionCx {
        SessionCx::new(Rc::clone(&self.staccato_session))
    }

    pub fn staccato_session(&self) -> StaccatoSession {
        self.staccato_session.borrow().clone()
    }

    pub fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) -> TaskHandle {
        spawn_task(future)
    }

    pub fn resource<T, E, F, Fut>(&self, id: impl Into<String>, load: F) -> Resource<T, E>
    where
        T: Send + 'static,
        E: Send + 'static,
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = std::result::Result<T, E>> + Send + 'static,
    {
        resource(id, load)
    }

    pub fn resource_value<T, F, Fut>(
        &self,
        id: impl Into<String>,
        load: F,
    ) -> Resource<T, std::convert::Infallible>
    where
        T: Send + 'static,
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = T> + Send + 'static,
    {
        resource_value(id, load)
    }

    pub fn mutation<I, T, E, F, Fut>(&self, id: impl Into<String>, run: F) -> Mutation<I, T, E>
    where
        I: Send + 'static,
        T: Send + 'static,
        E: Send + 'static,
        F: Fn(I) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = std::result::Result<T, E>> + Send + 'static,
    {
        mutation(id, run)
    }
}

pub trait View {
    fn view(&self, cx: &mut Cx) -> Element;

    fn actions(&self, _cx: &mut Cx) -> Vec<ActionDescriptor> {
        Vec::new()
    }

    fn settings(&self, _cx: &mut Cx) -> SettingsSchema {
        SettingsSchema::new()
    }

    fn handle_action(&mut self, _action_id: &str, _cx: &mut Cx) {}
}

pub trait IntoView {
    fn into_view(self) -> Element;
}

impl<T> IntoView for T
where
    T: Into<Element>,
{
    fn into_view(self) -> Element {
        self.into()
    }
}

pub struct App<V> {
    id: Option<String>,
    name: Option<String>,
    window: Option<V>,
    backend: Option<BackendDescriptor>,
    action_handler: Option<Arc<dyn Fn(&str)>>,
    settings_schema: Option<SettingsSchema>,
}

impl App<()> {
    pub fn new() -> Self {
        Self {
            id: None,
            name: None,
            window: None,
            backend: None,
            action_handler: None,
            settings_schema: None,
        }
    }
}

impl Default for App<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> App<V> {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn window<N>(self, window: N) -> App<N> {
        App {
            id: self.id,
            name: self.name,
            window: Some(window),
            backend: self.backend,
            action_handler: self.action_handler,
            settings_schema: self.settings_schema,
        }
    }

    pub fn backend(mut self, backend: BackendDescriptor) -> Self {
        self.backend = Some(backend);
        self
    }

    pub fn settings(mut self, schema: SettingsSchema) -> Self {
        self.settings_schema = Some(schema);
        self
    }

    pub fn on_action<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str) + 'static,
    {
        self.action_handler = Some(Arc::new(handler));
        self
    }
}

impl<V> App<V>
where
    V: View + 'static,
{
    pub fn run(self) -> Result {
        let App {
            id,
            name,
            window,
            backend,
            action_handler,
            settings_schema,
        } = self;
        let root = Rc::new(RefCell::new(window.ok_or(StukError::MissingWindow)?));
        let app_id = id.unwrap_or_else(|| "dev.stuk.app".to_string());
        let app_name = name.unwrap_or_else(|| "Stuk".to_string());
        let settings_schema = {
            let mut cx = Cx::new(&app_id, &app_name);
            let root = root.borrow();
            settings_schema.unwrap_or_else(|| root.settings(&mut cx))
        };
        let settings_schema = Rc::new(settings_schema);
        let settings_store = Rc::new(RefCell::new(SettingsStore::from_schema(
            settings_schema.as_ref(),
        )));
        let staccato_session = Rc::new(RefCell::new(StaccatoSession::default()));
        let backend = backend.unwrap_or_else(BackendDescriptor::current_native);
        let initial = {
            let root = root.borrow();
            let mut cx = Cx::with_settings_and_session_and_backend(
                &app_id,
                &app_name,
                Rc::clone(&settings_schema),
                Rc::clone(&settings_store),
                Rc::clone(&staccato_session),
                backend.clone(),
            );
            build_window(&*root, &mut cx)
        };
        let action_registry = {
            let mut cx = Cx::with_settings_and_session_and_backend(
                &app_id,
                &app_name,
                Rc::clone(&settings_schema),
                Rc::clone(&settings_store),
                Rc::clone(&staccato_session),
                backend.clone(),
            );
            let root = root.borrow();
            ActionRegistry::from_actions(root.actions(&mut cx))?
        };
        let shortcuts = action_registry
            .iter()
            .filter_map(|action| {
                action
                    .shortcut
                    .clone()
                    .map(|shortcut| (shortcut, action.id.clone()))
            })
            .collect::<Vec<_>>();
        let root_for_render = Rc::clone(&root);
        let root_for_actions = Rc::clone(&root);
        let app_id_for_render = app_id.clone();
        let app_name_for_render = app_name.clone();
        let app_id_for_actions = app_id.clone();
        let app_name_for_actions = app_name.clone();
        let schema_for_render = Rc::clone(&settings_schema);
        let store_for_render = Rc::clone(&settings_store);
        let schema_for_actions = Rc::clone(&settings_schema);
        let store_for_actions = Rc::clone(&settings_store);
        let session_for_render = Rc::clone(&staccato_session);
        let session_for_actions = Rc::clone(&staccato_session);
        let action_registry = Rc::new(action_registry);
        let backend_for_render = backend.clone();
        let backend_for_actions = backend.clone();

        NativeApp::new(
            WindowOptions {
                title: initial.title,
                width: initial.width,
                height: initial.height,
                chrome: initial.chrome,
                resizable: initial.resizable,
                visible: initial.visible,
                active: initial.active,
                always_on_top: initial.always_on_top,
                transparent: initial.transparent,
                background_effect: initial.background_effect,
                regions: initial.regions,
                ..WindowOptions::default()
            },
            move |size, hovered, pressed, focused| {
                let root = root_for_render.borrow();
                let mut cx = Cx::with_settings_and_session_and_backend(
                    &app_id_for_render,
                    &app_name_for_render,
                    Rc::clone(&schema_for_render),
                    Rc::clone(&store_for_render),
                    Rc::clone(&session_for_render),
                    backend_for_render.clone(),
                );
                cx.set_viewport_size(size);
                let window = build_window(&*root, &mut cx);
                render_window(&window, size, hovered, pressed, focused)
            },
        )
        .backend(backend)
        .shortcuts(shortcuts)
        .on_action(Arc::new(move |action_id| {
            if matches!(
                action_id,
                ACTION_WINDOW_CLOSE | ACTION_WINDOW_MINIMIZE | ACTION_WINDOW_TOGGLE_MAXIMIZE
            ) {
                return;
            }
            let enabled = action_registry
                .get(action_id)
                .map(|action| action.enabled)
                .unwrap_or(true);
            if !enabled {
                return;
            }

            if let Some(handler) = &action_handler {
                handler(action_id);
            }

            let mut cx = Cx::with_settings_and_session_and_backend(
                &app_id_for_actions,
                &app_name_for_actions,
                Rc::clone(&schema_for_actions),
                Rc::clone(&store_for_actions),
                Rc::clone(&session_for_actions),
                backend_for_actions.clone(),
            );
            cx.apply_setting_action(action_id, "settings");
            root_for_actions
                .borrow_mut()
                .handle_action(action_id, &mut cx);
        }))
        .run()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stuk_settings::SettingDefinition;

    #[test]
    fn applies_settings_page_actions() {
        let mut schema = SettingsSchema::new();
        schema
            .insert(SettingDefinition::boolean("sync.enabled", "Sync", false))
            .unwrap();
        schema
            .insert(SettingDefinition::enumeration(
                "appearance.theme",
                "Theme",
                vec!["system".to_string(), "dark".to_string()],
                "system",
            ))
            .unwrap();

        let schema = Rc::new(schema);
        let store = Rc::new(RefCell::new(SettingsStore::from_schema(schema.as_ref())));
        let mut cx = Cx::with_settings("dev.stuk.test", "Test", Rc::clone(&schema), store);

        assert!(cx.apply_setting_action("settings.sync.enabled", "settings"));
        assert_eq!(cx.setting_bool("sync.enabled"), Some(true));

        assert!(cx.apply_setting_action("settings.appearance.theme.dark", "settings"));
        assert_eq!(cx.setting_text("appearance.theme").as_deref(), Some("dark"));
        assert_eq!(cx.theme().mode, stuk_style::ThemeMode::Dark);
    }

    #[test]
    fn exposes_shared_staccato_session_handles() {
        let schema = Rc::new(SettingsSchema::new());
        let store = Rc::new(RefCell::new(SettingsStore::new()));
        let cx = Cx::with_settings("dev.stuk.test", "Test", schema, store);

        cx.staccato().set_tab_title("Notes");
        cx.staccato()
            .set_preferred_split(stuk_platform::SplitHint::Right);
        cx.session().set_document_id("note-1");
        cx.session().set_restore_payload("{\"id\":\"note-1\"}");

        let session = cx.staccato_session();
        assert_eq!(session.tab_title.as_deref(), Some("Notes"));
        assert_eq!(session.document_id.as_deref(), Some("note-1"));
        assert_eq!(
            session.preferred_split,
            Some(stuk_platform::SplitHint::Right)
        );
    }

    #[test]
    fn exposes_platform_backend_and_capabilities_to_views() {
        let schema = Rc::new(SettingsSchema::new());
        let store = Rc::new(RefCell::new(SettingsStore::new()));
        let backend = BackendDescriptor::browser_web();
        let cx = Cx::with_settings_and_session_and_backend(
            "dev.stuk.test",
            "Test",
            schema,
            store,
            Rc::new(RefCell::new(StaccatoSession::default())),
            backend,
        );

        assert!(cx.is_web());
        assert!(!cx.is_desktop());
        assert!(cx.capabilities().web_surface);
        assert_eq!(cx.platform_backend().name, "web");
    }
}
