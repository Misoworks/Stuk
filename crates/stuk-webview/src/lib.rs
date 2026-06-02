use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::{
    collections::{BTreeMap, BTreeSet},
    future::Future,
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use stuk::prelude::*;
use thiserror::Error;
use winit::{
    application::ApplicationHandler,
    cursor::{Cursor, CursorIcon},
    dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window as WinitWindow, WindowAttributes, WindowId},
};
use x11rb::{
    connection::Connection,
    protocol::xproto::{
        Arc as X11Arc, ConfigureWindowAux, ConnectionExt, CoordMode, CreateGCAux, Gcontext, Point,
        Rectangle, Window as X11Window,
    },
    rust_connection::RustConnection,
};

mod osr;
mod osr_host;
mod osr_protocol;

pub use stuk_platform::{WindowBackgroundEffect, WindowChrome, WindowRegion, WindowRegions};
pub use stuk_style::Material;
pub use stuk_web_runtime::{
    RuntimeConfig, RuntimeEngine, RuntimeError, RuntimeInfo, RuntimeInstallProgress,
    RuntimeInstallStep, RuntimeMode, install_user_runtime_with_progress,
    launchable_cef_host_candidates, remove_user_minimal_runtime_if_client_requested,
    resolve_runtime,
};

pub const INSTALLING_WINDOW_ARG: &str = "--stuk-webview-installing-runtime";
pub const NATIVE_HOST_ARG: &str = "--stuk-webview-native-host";

const HOST_CMAKE: &str = include_str!("../host/linux/CMakeLists.txt");
const HOST_MAIN: &str = include_str!("../host/linux/main.cc");
const HOST_APP_H: &str = include_str!("../host/linux/app.h");
const HOST_APP_CC: &str = include_str!("../host/linux/app.cc");
const HOST_HANDLER_H: &str = include_str!("../host/linux/handler.h");
const HOST_HANDLER_CC: &str = include_str!("../host/linux/handler.cc");
const HOST_OSR_HANDLER_H: &str = include_str!("../host/linux/osr_handler.h");
const HOST_OSR_HANDLER_CC: &str = include_str!("../host/linux/osr_handler.cc");
const WEBVIEW_TITLEBAR_HEIGHT: u32 = 48;
static WEBVIEW_INSTANCE_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Error)]
pub enum WebViewError {
    #[error("CEF runtime not found; run `stuk runtime install cef` or configure a runtime path")]
    RuntimeNotFound,
    #[error("{0}")]
    Runtime(#[from] RuntimeError),
    #[error("CEF runtime version {found} is below minimum {required}")]
    RuntimeVersionTooLow { found: String, required: String },
    #[error("CEF runtime at {path} failed integrity check")]
    RuntimeIntegrityFailed { path: PathBuf },
    #[error("webview creation failed: {message}")]
    CreationFailed { message: String },
    #[error("bridge error: {message}")]
    BridgeError { message: String },
    #[error("security policy violation: {message}")]
    SecurityViolation { message: String },
}

type WebViewResult<T> = std::result::Result<T, WebViewError>;

#[derive(Clone, Debug)]
pub struct WebViewConfig {
    pub entry: Option<String>,
    pub dev_url: Option<String>,
    pub dev_command: Option<String>,
    pub title: String,
    pub material: Material,
    pub chrome: WindowChrome,
    pub transparent: bool,
    pub background_effect: WindowBackgroundEffect,
    pub regions: WindowRegions,
    pub security: WebViewSecurity,
    pub runtime: RuntimeConfig,
    pub bridge: BridgeRegistry,
}

impl Default for WebViewConfig {
    fn default() -> Self {
        Self {
            entry: None,
            dev_url: None,
            dev_command: None,
            title: "Stuk".to_string(),
            material: Material::Maris,
            chrome: WindowChrome::System,
            transparent: true,
            background_effect: WindowBackgroundEffect::None,
            regions: WindowRegions::default(),
            security: WebViewSecurity::default(),
            runtime: RuntimeConfig::default(),
            bridge: BridgeRegistry::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct WebViewSecurity {
    pub remote_content: bool,
    pub allowed_origins: Vec<String>,
    pub allowed_bridge_permissions: Vec<String>,
    pub devtools: WebViewDevtools,
    pub allow_eval: bool,
    pub allow_node: bool,
    pub csp: String,
}

impl Default for WebViewSecurity {
    fn default() -> Self {
        Self {
            remote_content: false,
            allowed_origins: Vec::new(),
            allowed_bridge_permissions: Vec::new(),
            devtools: WebViewDevtools::DevOnly,
            allow_eval: false,
            allow_node: false,
            csp: "default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'"
                .to_string(),
        }
    }
}

impl WebViewSecurity {
    pub fn allow_origin(mut self, origin: impl Into<String>) -> Self {
        self.allowed_origins.push(origin.into());
        self
    }

    pub fn allow_bridge_permission(mut self, permission: impl Into<String>) -> Self {
        self.allowed_bridge_permissions.push(permission.into());
        self
    }

    pub fn remote_content(mut self, enabled: bool) -> Self {
        self.remote_content = enabled;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebViewDevtools {
    Disabled,
    DevOnly,
    Enabled,
}

#[derive(Clone, Debug)]
pub struct WebViewWindow {
    pub config: WebViewConfig,
    bridge_handlers: BridgeHandlers,
}

pub struct WebViewProcess {
    child: Child,
    bridge_thread: Option<JoinHandle<()>>,
}

impl WebViewProcess {
    pub fn id(&self) -> u32 {
        self.child.id()
    }

    pub fn wait(mut self) -> io::Result<ExitStatus> {
        let status = self.child.wait();
        if let Some(thread) = self.bridge_thread.take() {
            let _ = thread.join();
        }
        status
    }
}

impl WebViewWindow {
    pub fn new() -> Self {
        Self {
            config: WebViewConfig::default(),
            bridge_handlers: BridgeHandlers::default(),
        }
    }

    pub fn entry(mut self, path: impl Into<String>) -> Self {
        self.config.entry = Some(path.into());
        self
    }

    pub fn dev_url(mut self, url: impl Into<String>) -> Self {
        self.config.dev_url = Some(url.into());
        self
    }

    pub fn dev_command(mut self, command: impl Into<String>) -> Self {
        self.config.dev_command = Some(command.into());
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.config.title = title.into();
        self
    }

    pub fn material(mut self, material: Material) -> Self {
        self.config.material = material;
        self
    }

    pub fn transparent(mut self, transparent: bool) -> Self {
        self.config.transparent = transparent;
        if !transparent {
            self.config.background_effect = WindowBackgroundEffect::None;
            self.config.regions.blur = None;
        }
        self
    }

    pub fn opaque(mut self) -> Self {
        self.config.transparent = false;
        self.config.background_effect = WindowBackgroundEffect::None;
        self.config.regions.blur = None;
        self
    }

    pub fn glass(mut self) -> Self {
        self.config.material = Material::Window;
        self.config.transparent = true;
        self.config.background_effect = WindowBackgroundEffect::Blur;
        self
    }

    pub fn background_effect(mut self, effect: WindowBackgroundEffect) -> Self {
        self.config.background_effect = effect;
        if effect.requires_transparency() {
            self.config.transparent = true;
        }
        self
    }

    pub fn regions(mut self, regions: WindowRegions) -> Self {
        self.config.regions = regions;
        self
    }

    pub fn blur_region(mut self, region: WindowRegion) -> Self {
        self.config.regions.blur = Some(region);
        self
    }

    pub fn input_region(mut self, region: WindowRegion) -> Self {
        self.config.regions.input = Some(region);
        self
    }

    pub fn chrome(mut self, chrome: WindowChrome) -> Self {
        self.config.chrome = chrome;
        self
    }

    pub fn security(mut self, security: WebViewSecurity) -> Self {
        self.config.security = security;
        self
    }

    pub fn runtime(mut self, runtime: RuntimeConfig) -> Self {
        self.config.runtime = runtime;
        self
    }

    pub fn bridge(mut self, bridge: BridgeRegistry) -> Self {
        self.config.bridge = bridge;
        self
    }

    pub fn bridge_command(mut self, command_name: impl Into<String>) -> Self {
        self.config.bridge.register(command_name);
        self
    }

    pub fn bridge_handler<F>(mut self, command_name: impl Into<String>, handler: F) -> Self
    where
        F: Fn(BridgeCommand) -> BridgeResult + Send + Sync + 'static,
    {
        let name = command_name.into();
        self.config.bridge.register(name.clone());
        self.bridge_handlers.register(name, handler);
        self
    }

    pub fn bridge_descriptor_handler<F>(
        mut self,
        descriptor: BridgeCommandDescriptor,
        handler: F,
    ) -> Self
    where
        F: Fn(BridgeCommand) -> BridgeResult + Send + Sync + 'static,
    {
        let name = descriptor.name.clone();
        self.config.bridge.register_descriptor(descriptor);
        self.bridge_handlers.register(name, handler);
        self
    }

    pub fn bridge_handler_async<F, Fut>(self, command_name: impl Into<String>, handler: F) -> Self
    where
        F: Fn(BridgeCommand) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = BridgeResult> + Send + 'static,
    {
        self.bridge_descriptor_handler_async(BridgeCommandDescriptor::new(command_name), handler)
    }

    pub fn bridge_descriptor_handler_async<F, Fut>(
        mut self,
        descriptor: BridgeCommandDescriptor,
        handler: F,
    ) -> Self
    where
        F: Fn(BridgeCommand) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = BridgeResult> + Send + 'static,
    {
        let name = descriptor.name.clone();
        self.config.bridge.register_descriptor(descriptor);
        self.bridge_handlers
            .register(name, move |command| pollster::block_on(handler(command)));
        self
    }

    pub fn launch(self) -> WebViewResult<WebViewProcess> {
        let runtime = resolve_runtime(&self.config.runtime)?;
        self.launch_with_runtime(runtime)
    }

    pub fn launch_or_install(self) -> WebViewResult<WebViewProcess> {
        let runtime = resolve_or_install_runtime(&self.config.runtime)?;
        self.launch_with_runtime(runtime)
    }

    pub fn launch_with_runtime(mut self, runtime: RuntimeInfo) -> WebViewResult<WebViewProcess> {
        self.ensure_default_bridge_handlers();
        let url = self.entry_url()?;
        let runtime_dir = runtime.location.path();
        if let Some(process) =
            launch_native_host_process(runtime_dir, &self.config, &self.bridge_handlers, &url)?
        {
            return Ok(process);
        }

        let Some(mut command) = runtime_command(runtime_dir, &self.config, &url, false) else {
            return Err(WebViewError::CreationFailed {
                message: format!(
                    "no launchable CEF executable found in {}",
                    runtime_dir.display()
                ),
            });
        };
        let previous_windows = if self.config.chrome != WindowChrome::System {
            x11_client_windows()
        } else {
            BTreeSet::new()
        };
        let bridge_runtime = BridgeRuntime::new(
            self.bridge_handlers.clone(),
            self.config.bridge.clone(),
            self.config.security.clone(),
        );
        prepare_bridge_command(&mut command, &self.bridge_handlers);
        let mut child = command
            .spawn()
            .map_err(|error| WebViewError::CreationFailed {
                message: error.to_string(),
            })?;
        if self.config.chrome != WindowChrome::System {
            remove_system_decorations(child.id(), previous_windows);
        }
        let bridge_thread = spawn_bridge_dispatch(&mut child, bridge_runtime);
        Ok(WebViewProcess {
            child,
            bridge_thread,
        })
    }

    fn entry_url(&self) -> WebViewResult<String> {
        if let Some(url) = &self.config.dev_url {
            return Ok(url.clone());
        }

        let Some(entry) = &self.config.entry else {
            return Err(WebViewError::CreationFailed {
                message: "webview has no entry or dev url".to_string(),
            });
        };
        let entry_path = PathBuf::from(entry);
        let path = if entry_path.is_absolute() {
            entry_path
        } else {
            std::env::current_dir()
                .map_err(|error| WebViewError::CreationFailed {
                    message: error.to_string(),
                })?
                .join(entry_path)
        };
        let path = path
            .canonicalize()
            .map_err(|error| WebViewError::CreationFailed {
                message: format!("failed to resolve webview entry: {error}"),
            })?;
        Ok(format!("file://{}", path.display()))
    }

    fn ensure_default_bridge_handlers(&mut self) {
        for command in self.config.bridge.commands() {
            if self.bridge_handlers.contains(&command) {
                continue;
            }
            let command_name = command.clone();
            self.bridge_handlers.register(command, move |_| {
                Err(BridgeError::new(format!(
                    "Bridge command `{command_name}` has no Rust handler"
                )))
            });
        }
    }
}

pub fn run_installing_window_from_args(args: &[String]) -> bool {
    let Some(index) = args.iter().position(|arg| arg == INSTALLING_WINDOW_ARG) else {
        return false;
    };
    let status_path = args.get(index + 1).map(PathBuf::from);
    run_installing_window(status_path);
    true
}

pub fn run_native_host_from_args(args: &[String]) -> bool {
    if osr::run_from_args(args) {
        return true;
    }
    let Some(index) = args.iter().position(|arg| arg == NATIVE_HOST_ARG) else {
        return false;
    };
    let Some(config_path) = args.get(index + 1).map(PathBuf::from) else {
        eprintln!("missing webview native host config path");
        std::process::exit(1);
    };
    if let Err(error) = run_native_host(config_path) {
        eprintln!("webview native host failed: {error}");
        std::process::exit(1);
    }
    true
}

fn prepare_bridge_command(command: &mut Command, bridge_handlers: &BridgeHandlers) {
    if bridge_handlers.is_empty() {
        command.stdin(Stdio::null()).stdout(Stdio::null());
        return;
    }
    command.stdin(Stdio::piped()).stdout(Stdio::piped());
}

fn spawn_bridge_dispatch(
    child: &mut Child,
    bridge_runtime: BridgeRuntime,
) -> Option<JoinHandle<()>> {
    if bridge_runtime.is_empty() {
        return None;
    }
    let stdout = child.stdout.take()?;
    let mut stdin = child.stdin.take()?;
    Some(thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(std::result::Result::ok) {
            let Some(request) = BridgeIpcRequest::parse(&line) else {
                continue;
            };
            let response = bridge_runtime.dispatch(request.command);
            let line = BridgeIpcResponse::from_result(request.browser_id, request.id, response)
                .serialize();
            if writeln!(stdin, "{line}").is_err() {
                break;
            }
            let _ = stdin.flush();
        }
    }))
}

#[derive(Debug)]
struct BridgeIpcRequest {
    browser_id: String,
    id: String,
    command: BridgeCommand,
}

impl BridgeIpcRequest {
    fn parse(line: &str) -> Option<Self> {
        let parts = line.splitn(6, '\t').collect::<Vec<_>>();
        if parts.first().copied()? != "STUK_BRIDGE_REQUEST" {
            return None;
        }
        if parts.len() == 5 {
            let params = serde_json::from_str(parts[4]).ok()?;
            return Some(Self {
                browser_id: parts[1].to_string(),
                id: parts[2].to_string(),
                command: BridgeCommand {
                    name: parts[3].to_string(),
                    params,
                    origin: None,
                },
            });
        }
        if parts.len() != 6 {
            return None;
        }
        let params = serde_json::from_str(parts[5]).ok()?;
        let origin = if parts[3].is_empty() {
            None
        } else {
            Some(parts[3].to_string())
        };
        Some(Self {
            browser_id: parts[1].to_string(),
            id: parts[2].to_string(),
            command: BridgeCommand {
                name: parts[4].to_string(),
                params,
                origin,
            },
        })
    }
}

#[derive(Debug)]
struct BridgeIpcResponse {
    browser_id: String,
    id: String,
    ok: bool,
    payload: serde_json::Value,
}

impl BridgeIpcResponse {
    fn from_result(browser_id: String, id: String, result: BridgeResult) -> Self {
        match result {
            Ok(response) => Self {
                browser_id,
                id,
                ok: true,
                payload: response.result,
            },
            Err(error) => Self {
                browser_id,
                id,
                ok: false,
                payload: serde_json::json!({ "message": error.message }),
            },
        }
    }

    fn serialize(&self) -> String {
        let status = if self.ok { "ok" } else { "error" };
        let payload = serde_json::to_string(&self.payload).unwrap_or_else(|_| "null".to_string());
        format!(
            "STUK_BRIDGE_RESPONSE\t{}\t{}\t{status}\t{payload}",
            self.browser_id, self.id
        )
    }
}

pub fn resolve_or_install_runtime(
    config: &RuntimeConfig,
) -> std::result::Result<RuntimeInfo, RuntimeError> {
    remove_user_minimal_runtime_if_client_requested(config)?;
    match resolve_runtime(config) {
        Ok(runtime) => Ok(runtime),
        Err(_) if config.allow_user_install => install_runtime_with_window(config),
        Err(error) => Err(error),
    }
}

fn install_runtime_with_window(
    config: &RuntimeConfig,
) -> std::result::Result<RuntimeInfo, RuntimeError> {
    let status_path = std::env::temp_dir().join(format!(
        "stuk-runtime-install-{}.status",
        std::process::id()
    ));
    write_install_status(
        &status_path,
        &RuntimeInstallProgress::new(
            RuntimeInstallStep::Preparing,
            None,
            "Preparing shared web runtime",
        ),
    );
    let mut status_window = std::env::current_exe().ok().and_then(|exe| {
        Command::new(exe)
            .arg(INSTALLING_WINDOW_ARG)
            .arg(&status_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .ok()
    });

    let result = install_user_runtime_with_progress(config, |progress| {
        write_install_status(&status_path, &progress);
    });

    if let Some(child) = status_window.as_mut() {
        let _ = child.kill();
        let _ = child.wait();
    }
    let _ = std::fs::remove_file(status_path);

    result
}

fn run_installing_window(status_path: Option<PathBuf>) {
    if let Err(error) = App::new()
        .id("dev.stuk.webview.installing")
        .name("Installing dependencies")
        .window(InstallingWindow { status_path })
        .run()
    {
        eprintln!("failed to show installing window: {error}");
        std::process::exit(1);
    }
}

struct InstallingWindow {
    status_path: Option<PathBuf>,
}

impl View for InstallingWindow {
    fn view(&self, _cx: &mut Cx) -> stuk::Element {
        let status = self.status();
        Window::new()
            .title("Stuk")
            .size(460, 210)
            .glass()
            .continuous_redraw(true)
            .content(
                Center::new(
                    Flex::column()
                        .gap(8.0)
                        .align(FlexAlign::Center)
                        .child(
                            Frame::new(
                                Text::new("Installing required dependencies")
                                    .size(22.0)
                                    .balance()
                                    .centered(),
                            )
                            .width(390.0),
                        )
                        .child(
                            Frame::new(
                                Text::new("Preparing the shared web runtime for this app.")
                                    .muted()
                                    .pretty()
                                    .centered(),
                            )
                            .width(360.0),
                        )
                        .child(
                            Frame::new(
                                ProgressBar::new(status.progress * 100.0, 100.0)
                                    .label(status.message.clone())
                                    .color(Color::rgb(0.78, 0.78, 0.78)),
                            )
                            .width(260.0),
                        )
                        .width(390.0),
                )
                .padding(28.0),
            )
            .into()
    }
}

impl InstallingWindow {
    fn status(&self) -> InstallStatus {
        self.status_path
            .as_deref()
            .and_then(read_install_status)
            .unwrap_or_default()
    }
}

#[derive(Clone, Debug)]
struct InstallStatus {
    progress: f32,
    message: String,
}

impl Default for InstallStatus {
    fn default() -> Self {
        Self {
            progress: 0.0,
            message: "Preparing shared web runtime".to_string(),
        }
    }
}

fn write_install_status(path: &Path, progress: &RuntimeInstallProgress) {
    let fraction = progress.fraction.unwrap_or(match progress.step {
        RuntimeInstallStep::Preparing => 0.05,
        RuntimeInstallStep::RemovingOldRuntime => 0.10,
        RuntimeInstallStep::Downloading => 0.20,
        RuntimeInstallStep::Verifying => 0.72,
        RuntimeInstallStep::Extracting => 0.84,
        RuntimeInstallStep::Installing => 0.94,
        RuntimeInstallStep::Complete => 1.0,
    });
    let body = format!(
        "{:?}\n{}\n{}\n",
        progress.step,
        fraction,
        progress.message.replace('\n', " ")
    );
    let _ = std::fs::write(path, body);
}

fn read_install_status(path: &Path) -> Option<InstallStatus> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut lines = text.lines();
    let _step = match lines.next()? {
        "Preparing" => RuntimeInstallStep::Preparing,
        "RemovingOldRuntime" => RuntimeInstallStep::RemovingOldRuntime,
        "Downloading" => RuntimeInstallStep::Downloading,
        "Verifying" => RuntimeInstallStep::Verifying,
        "Extracting" => RuntimeInstallStep::Extracting,
        "Installing" => RuntimeInstallStep::Installing,
        "Complete" => RuntimeInstallStep::Complete,
        _ => RuntimeInstallStep::Preparing,
    };
    let progress = lines.next()?.parse::<f32>().ok()?.clamp(0.0, 1.0);
    let message = lines
        .next()
        .unwrap_or("Preparing shared web runtime")
        .to_string();
    Some(InstallStatus { progress, message })
}

fn launch_native_host_process(
    runtime_dir: &Path,
    config: &WebViewConfig,
    bridge_handlers: &BridgeHandlers,
    url: &str,
) -> WebViewResult<Option<WebViewProcess>> {
    #[cfg(target_os = "linux")]
    {
        let host_binary = ensure_stuk_cef_host(runtime_dir)
            .map_err(|message| WebViewError::CreationFailed { message })?;
        if !use_x11_embedded_compat() {
            if !use_wayland_windowed_compat() {
                return osr::launch_process(runtime_dir, config, bridge_handlers, url).map(Some);
            }
            return launch_wayland_cef_host_process(
                runtime_dir,
                &host_binary,
                config,
                bridge_handlers,
                url,
            )
            .map(Some);
        }

        let host_config_path =
            std::env::temp_dir().join(format!("stuk-webview-host-{}.json", webview_instance_key()));
        let body = serde_json::json!({
            "runtime_dir": runtime_dir,
            "host_binary": host_binary,
            "url": url,
            "title": config.title,
            "width": 800,
            "height": 600,
            "transparent": config.transparent,
            "background_effect": config.background_effect.as_str(),
            "chrome": config.chrome.as_str(),
            "bridge_commands": config.bridge.commands(),
        });
        std::fs::write(&host_config_path, body.to_string()).map_err(|error| {
            WebViewError::CreationFailed {
                message: format!("failed to write webview host config: {error}"),
            }
        })?;
        let exe = std::env::current_exe().map_err(|error| WebViewError::CreationFailed {
            message: error.to_string(),
        })?;
        let mut command = Command::new(exe);
        command
            .arg(NATIVE_HOST_ARG)
            .arg(&host_config_path)
            .env("WINIT_UNIX_BACKEND", "x11")
            .env_remove("WAYLAND_DISPLAY")
            .stderr(Stdio::inherit());
        prepare_bridge_command(&mut command, bridge_handlers);
        let mut child = command
            .spawn()
            .map_err(|error| WebViewError::CreationFailed {
                message: format!("failed to launch webview native host: {error}"),
            })?;
        let bridge_thread = spawn_bridge_dispatch(
            &mut child,
            BridgeRuntime::new(
                bridge_handlers.clone(),
                config.bridge.clone(),
                config.security.clone(),
            ),
        );
        return Ok(Some(WebViewProcess {
            child,
            bridge_thread,
        }));
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = runtime_dir;
        let _ = config;
        let _ = url;
        Ok(None)
    }
}

#[cfg(target_os = "linux")]
fn use_x11_embedded_compat() -> bool {
    match std::env::var("STUK_WEBVIEW_BACKEND") {
        Ok(value) => matches!(value.as_str(), "x11" | "x11-embedded" | "compat"),
        Err(_) => {
            std::env::var_os("WAYLAND_DISPLAY").is_none() && std::env::var_os("DISPLAY").is_some()
        }
    }
}

#[cfg(target_os = "linux")]
fn use_wayland_windowed_compat() -> bool {
    std::env::var("STUK_WEBVIEW_BACKEND").is_ok_and(|value| {
        matches!(
            value.as_str(),
            "windowed" | "cef-windowed" | "wayland-windowed"
        )
    })
}

#[cfg(target_os = "linux")]
fn launch_wayland_cef_host_process(
    runtime_dir: &Path,
    host_binary: &Path,
    config: &WebViewConfig,
    bridge_handlers: &BridgeHandlers,
    url: &str,
) -> WebViewResult<WebViewProcess> {
    let release_dir = runtime_dir.join("Release");
    let cache_dir = webview_cache_dir(runtime_dir, &config.title, url);
    std::fs::create_dir_all(&cache_dir).map_err(|error| WebViewError::CreationFailed {
        message: format!("failed to create CEF cache dir: {error}"),
    })?;
    let mut command = Command::new(host_binary);
    command
        .arg(format!("--url={url}"))
        .arg(format!("--stuk-title={}", config.title))
        .arg("--stuk-window-mode=toplevel")
        .arg("--stuk-ozone-platform=wayland")
        .arg(format!("--stuk-width={}", 800))
        .arg(format!("--stuk-height={}", 600))
        .arg(format!(
            "--stuk-background-effect={}",
            config.background_effect.as_str()
        ))
        .arg(format!(
            "--stuk-bridge-commands={}",
            config.bridge.commands().join(",")
        ))
        .arg(format!("--root-cache-path={}", cache_dir.display()))
        .arg(format!(
            "--cache-path={}",
            cache_dir.join("browser").display()
        ))
        .arg("--ozone-platform=wayland")
        .arg("--enable-features=UseOzonePlatform")
        .arg("--disable-features=Vulkan,DefaultANGLEVulkan,VulkanFromANGLE")
        .arg("--disable-vulkan")
        .arg("--disable-gpu")
        .current_dir(&release_dir)
        .env("GDK_BACKEND", "wayland")
        .env("XDG_SESSION_TYPE", "wayland")
        .env("LD_LIBRARY_PATH", ld_library_path(&release_dir))
        .stdin(Stdio::null());
    if config.transparent {
        command
            .arg("--stuk-transparent")
            .arg("--enable-transparent-visuals")
            .arg("--transparent-painting-enabled")
            .arg("--default-background-color=0x00000000");
    }
    if config.chrome != WindowChrome::System {
        command.arg("--stuk-frameless");
    }
    prepare_bridge_command(&mut command, bridge_handlers);
    let mut child = command
        .spawn()
        .map_err(|error| WebViewError::CreationFailed {
            message: format!("failed to launch Wayland CEF host: {error}"),
        })?;
    let bridge_thread = spawn_bridge_dispatch(
        &mut child,
        BridgeRuntime::new(
            bridge_handlers.clone(),
            config.bridge.clone(),
            config.security.clone(),
        ),
    );
    Ok(WebViewProcess {
        child,
        bridge_thread,
    })
}

#[cfg(target_os = "linux")]
fn ensure_stuk_cef_host(runtime_dir: &Path) -> std::result::Result<PathBuf, String> {
    let binary = runtime_dir.join("Release").join("stuk-cef-host");
    let source_dir = runtime_dir.join(".stuk-host-src");
    let build_dir = runtime_dir.join(".stuk-host-build");
    let source_stamp = build_dir.join("stuk-host-source.fnv");
    let expected_stamp = host_source_fingerprint();
    if binary.is_file()
        && std::fs::read_to_string(&source_stamp).is_ok_and(|stamp| stamp.trim() == expected_stamp)
    {
        return Ok(binary);
    }
    if !runtime_dir.join("include").is_dir()
        || !runtime_dir.join("libcef_dll").is_dir()
        || !runtime_dir.join("cmake").is_dir()
    {
        return Err(format!(
            "CEF runtime at {} is not a standard CEF distribution; reinstall the standard runtime",
            runtime_dir.display()
        ));
    }

    std::fs::create_dir_all(&source_dir).map_err(|error| error.to_string())?;
    std::fs::create_dir_all(&build_dir).map_err(|error| error.to_string())?;
    write_host_source(&source_dir)?;

    let generator = if command_available("ninja") {
        "Ninja"
    } else {
        "Unix Makefiles"
    };
    run_checked(
        Command::new("cmake")
            .arg("-S")
            .arg(&source_dir)
            .arg("-B")
            .arg(&build_dir)
            .arg("-G")
            .arg(generator)
            .arg("-DCMAKE_BUILD_TYPE=Release")
            .arg(format!("-DCEF_ROOT={}", runtime_dir.display())),
    )?;
    run_checked(
        Command::new("cmake")
            .arg("--build")
            .arg(&build_dir)
            .arg("--target")
            .arg("stuk-cef-host")
            .arg("--parallel"),
    )?;

    if binary.is_file() {
        std::fs::write(source_stamp, expected_stamp).map_err(|error| error.to_string())?;
        Ok(binary)
    } else {
        Err(format!(
            "CEF host build did not create {}",
            binary.display()
        ))
    }
}

#[cfg(target_os = "linux")]
fn write_host_source(source_dir: &Path) -> std::result::Result<(), String> {
    for (name, body) in [
        ("CMakeLists.txt", HOST_CMAKE),
        ("main.cc", HOST_MAIN),
        ("app.h", HOST_APP_H),
        ("app.cc", HOST_APP_CC),
        ("handler.h", HOST_HANDLER_H),
        ("handler.cc", HOST_HANDLER_CC),
        ("osr_handler.h", HOST_OSR_HANDLER_H),
        ("osr_handler.cc", HOST_OSR_HANDLER_CC),
    ] {
        std::fs::write(source_dir.join(name), body).map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn host_source_fingerprint() -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for body in [
        HOST_CMAKE,
        HOST_MAIN,
        HOST_APP_H,
        HOST_APP_CC,
        HOST_HANDLER_H,
        HOST_HANDLER_CC,
        HOST_OSR_HANDLER_H,
        HOST_OSR_HANDLER_CC,
    ] {
        for byte in body.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    format!("{hash:016x}")
}

#[cfg(target_os = "linux")]
fn command_available(name: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {name} >/dev/null 2>&1"))
        .status()
        .is_ok_and(|status| status.success())
}

#[cfg(target_os = "linux")]
fn run_checked(command: &mut Command) -> std::result::Result<(), String> {
    let output = command.output().map_err(|error| error.to_string())?;
    if output.status.success() {
        return Ok(());
    }
    Err(format!(
        "command failed: {}\n{}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}

fn run_native_host(config_path: PathBuf) -> std::result::Result<(), String> {
    let text = std::fs::read_to_string(&config_path).map_err(|error| error.to_string())?;
    let config: serde_json::Value =
        serde_json::from_str(&text).map_err(|error| error.to_string())?;
    let runtime_dir = config
        .get("runtime_dir")
        .and_then(serde_json::Value::as_str)
        .map(PathBuf::from)
        .ok_or_else(|| "native host config missing runtime_dir".to_string())?;
    let host_binary = config
        .get("host_binary")
        .and_then(serde_json::Value::as_str)
        .map(PathBuf::from)
        .ok_or_else(|| "native host config missing host_binary".to_string())?;
    let url = config
        .get("url")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "native host config missing url".to_string())?
        .to_string();
    let title = config
        .get("title")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("Stuk")
        .to_string();
    let width = config
        .get("width")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(800) as u32;
    let height = config
        .get("height")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(600) as u32;
    let transparent = config
        .get("transparent")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let background_effect = config
        .get("background_effect")
        .and_then(serde_json::Value::as_str)
        .and_then(WindowBackgroundEffect::parse)
        .unwrap_or(WindowBackgroundEffect::None);
    let chrome = config
        .get("chrome")
        .and_then(serde_json::Value::as_str)
        .and_then(WindowChrome::parse)
        .unwrap_or(WindowChrome::System);
    let bridge_commands = config
        .get("bridge_commands")
        .and_then(serde_json::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let _ = std::fs::remove_file(config_path);

    NativeWebViewHost {
        runtime_dir,
        host_binary,
        url,
        title,
        width,
        height,
        transparent,
        background_effect,
        chrome,
        bridge_commands,
        window: None,
        child: None,
        child_window: None,
        surface_size: PhysicalSize::new(width, height),
        titlebar: WebViewTitlebarState::default(),
        launch_attempted: false,
        started: Instant::now(),
    }
    .run()
}

struct NativeWebViewHost {
    runtime_dir: PathBuf,
    host_binary: PathBuf,
    url: String,
    title: String,
    width: u32,
    height: u32,
    transparent: bool,
    background_effect: WindowBackgroundEffect,
    chrome: WindowChrome,
    bridge_commands: Vec<String>,
    window: Option<Arc<dyn WinitWindow>>,
    child: Option<Child>,
    child_window: Option<X11Window>,
    surface_size: PhysicalSize<u32>,
    titlebar: WebViewTitlebarState,
    launch_attempted: bool,
    started: Instant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WebViewTitlebarControl {
    Minimize,
    Maximize,
    Close,
}

#[derive(Debug)]
struct WebViewTitlebarState {
    hovered: Option<WebViewTitlebarControl>,
    pressed: Option<WebViewTitlebarControl>,
    cursor: CursorIcon,
}

impl Default for WebViewTitlebarState {
    fn default() -> Self {
        Self {
            hovered: None,
            pressed: None,
            cursor: CursorIcon::Default,
        }
    }
}

impl NativeWebViewHost {
    fn run(self) -> std::result::Result<(), String> {
        let event_loop = EventLoop::new().map_err(|error| error.to_string())?;
        event_loop.run_app(self).map_err(|error| error.to_string())
    }

    fn parent_xid(&self) -> Option<X11Window> {
        let window = self.window.as_ref()?;
        match window.window_handle().ok()?.as_raw() {
            RawWindowHandle::Xlib(handle) => Some(handle.window as X11Window),
            RawWindowHandle::Xcb(handle) => Some(handle.window.get()),
            _ => None,
        }
    }

    fn titlebar_height(&self, window: &Arc<dyn WinitWindow>) -> u32 {
        webview_titlebar_height(self.chrome, window.scale_factor())
    }

    fn content_bounds(&self, window: &Arc<dyn WinitWindow>) -> (i32, i32, u32, u32) {
        let titlebar_height = self.titlebar_height(window);
        let height = self
            .surface_size
            .height
            .saturating_sub(titlebar_height)
            .max(1);
        (
            0,
            titlebar_height as i32,
            self.surface_size.width.max(1),
            height,
        )
    }

    fn resize_child(&self) {
        let (Some(window), Some(child_window)) = (&self.window, self.child_window) else {
            return;
        };
        let (x, y, width, height) = self.content_bounds(window);
        let _ = resize_x11_window(child_window, x, y, width, height);
    }

    fn redraw_chrome(&mut self) {
        if self.chrome.uses_native_decorations() {
            return;
        }
        let Some(parent) = self.parent_xid() else {
            return;
        };
        let Some(window) = &self.window else {
            return;
        };
        let titlebar_height = self.titlebar_height(window);
        let _ = draw_x11_webview_chrome(
            parent,
            self.surface_size.width,
            self.surface_size.height,
            titlebar_height,
            &self.title,
            self.titlebar.hovered,
            self.titlebar.pressed,
        );
    }

    fn update_hover(&mut self, window: &Arc<dyn WinitWindow>, x: f64, y: f64) {
        let titlebar_height = self.titlebar_height(window);
        let hovered = titlebar_control_at(self.surface_size.width, titlebar_height, x, y);
        let cursor = if hovered.is_some() {
            CursorIcon::Pointer
        } else {
            CursorIcon::Default
        };
        let changed = self.titlebar.hovered != hovered || self.titlebar.cursor != cursor;
        self.titlebar.hovered = hovered;
        if self.titlebar.cursor != cursor {
            self.titlebar.cursor = cursor;
            window.set_cursor(Cursor::Icon(cursor));
        }
        if changed {
            window.request_redraw();
        }
    }

    fn press_titlebar(&mut self, window: &Arc<dyn WinitWindow>, x: f64, y: f64) -> bool {
        let titlebar_height = self.titlebar_height(window);
        if titlebar_height == 0 || y > f64::from(titlebar_height) {
            return false;
        }
        if let Some(control) = titlebar_control_at(self.surface_size.width, titlebar_height, x, y) {
            self.titlebar.pressed = Some(control);
            window.request_redraw();
        } else {
            let _ = window.drag_window();
        }
        true
    }

    fn release_titlebar(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window: &Arc<dyn WinitWindow>,
        x: f64,
        y: f64,
    ) -> bool {
        let titlebar_height = self.titlebar_height(window);
        let control = titlebar_control_at(self.surface_size.width, titlebar_height, x, y);
        let handled = if let Some(pressed) = self.titlebar.pressed.take() {
            if control == Some(pressed) {
                self.activate_titlebar_control(event_loop, window, pressed);
            }
            true
        } else {
            titlebar_height > 0 && y <= f64::from(titlebar_height)
        };
        if handled {
            window.request_redraw();
        }
        handled
    }

    fn activate_titlebar_control(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window: &Arc<dyn WinitWindow>,
        control: WebViewTitlebarControl,
    ) {
        match control {
            WebViewTitlebarControl::Minimize => window.set_minimized(true),
            WebViewTitlebarControl::Maximize => window.set_maximized(!window.is_maximized()),
            WebViewTitlebarControl::Close => {
                if let Some(child) = self.child.as_mut() {
                    let _ = child.kill();
                    let _ = child.wait();
                }
                event_loop.exit();
            }
        }
    }

    fn launch_child(&mut self, event_loop: &dyn ActiveEventLoop) {
        let Some(parent) = self.parent_xid() else {
            eprintln!("webview native host requires an X11 parent window");
            event_loop.exit();
            return;
        };
        let Some(window) = &self.window else {
            event_loop.exit();
            return;
        };
        let (x, y, width, height) = self.content_bounds(window);
        let release_dir = self.runtime_dir.join("Release");
        let cache_dir = webview_cache_dir(&self.runtime_dir, &self.title, &self.url);
        let _ = std::fs::create_dir_all(&cache_dir);
        let mut command = Command::new(&self.host_binary);
        command
            .arg(format!("--url={}", self.url))
            .arg(format!("--stuk-parent-window=0x{parent:x}"))
            .arg(format!("--stuk-x={x}"))
            .arg(format!("--stuk-y={y}"))
            .arg(format!("--stuk-width={}", width.max(1)))
            .arg(format!("--stuk-height={}", height.max(1)))
            .arg(format!(
                "--stuk-background-effect={}",
                self.background_effect.as_str()
            ))
            .arg(format!(
                "--stuk-bridge-commands={}",
                self.bridge_commands.join(",")
            ))
            .arg(format!("--root-cache-path={}", cache_dir.display()))
            .arg(format!(
                "--cache-path={}",
                cache_dir.join("browser").display()
            ))
            .arg("--ozone-platform=x11")
            .current_dir(&release_dir)
            .env_remove("WAYLAND_DISPLAY")
            .env("GDK_BACKEND", "x11")
            .env("XDG_SESSION_TYPE", "x11")
            .env("LD_LIBRARY_PATH", ld_library_path(&release_dir));
        if self.transparent {
            command
                .arg("--stuk-transparent")
                .arg("--enable-transparent-visuals")
                .arg("--transparent-painting-enabled")
                .arg("--default-background-color=0x00000000");
        }
        if !self.bridge_commands.is_empty() {
            command.stdin(Stdio::piped()).stdout(Stdio::piped());
        } else {
            command.stdin(Stdio::null()).stdout(Stdio::null());
        }
        let child = match command.spawn() {
            Ok(child) => child,
            Err(error) => {
                eprintln!("failed to launch CEF child: {error}");
                event_loop.exit();
                return;
            }
        };
        self.child = Some(child);
        if !self.bridge_commands.is_empty()
            && let Some(child) = self.child.as_mut()
        {
            spawn_native_host_bridge_proxy(child);
        }
        for _ in 0..100 {
            if let Some(window_id) = find_x11_child(parent) {
                self.child_window = Some(window_id);
                self.resize_child();
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        eprintln!("CEF host started but child browser window was not visible yet");
    }
}

impl ApplicationHandler for NativeWebViewHost {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let mut attributes = WindowAttributes::default()
            .with_title(self.title.clone())
            .with_surface_size(LogicalSize::new(
                f64::from(self.width),
                f64::from(self.height),
            ))
            .with_decorations(self.chrome.uses_native_decorations())
            .with_transparent(self.transparent);
        if let Some(position) = centered_window_position(event_loop, self.width, self.height) {
            attributes = attributes.with_position(position);
        }
        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::<dyn WinitWindow>::from(window),
            Err(error) => {
                eprintln!("failed to create webview native host window: {error}");
                event_loop.exit();
                return;
            }
        };
        self.surface_size = window.surface_size();
        self.window = Some(window);
        if let Some(window) = &self.window {
            window.request_redraw();
        }
        self.launch_attempted = true;
        self.launch_child(event_loop);
    }

    fn window_event(&mut self, event_loop: &dyn ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(window) = self.window.clone() else {
            return;
        };
        if id != window.id() {
            return;
        }
        match event {
            WindowEvent::CloseRequested => {
                if let Some(child) = self.child.as_mut() {
                    let _ = child.kill();
                    let _ = child.wait();
                }
                event_loop.exit();
            }
            WindowEvent::SurfaceResized(size) => {
                self.surface_size = size;
                self.resize_child();
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                self.redraw_chrome();
            }
            WindowEvent::PointerMoved {
                position, primary, ..
            } if primary => {
                self.update_hover(&window, position.x, position.y);
            }
            WindowEvent::PointerLeft { primary, .. } if primary => {
                self.titlebar.hovered = None;
                if self.titlebar.cursor != CursorIcon::Default {
                    self.titlebar.cursor = CursorIcon::Default;
                    window.set_cursor(Cursor::Icon(CursorIcon::Default));
                }
                window.request_redraw();
            }
            WindowEvent::PointerButton {
                state: ElementState::Pressed,
                primary: true,
                position,
                button,
                ..
            } if button.clone().mouse_button() == Some(MouseButton::Left) => {
                let _ = self.press_titlebar(&window, position.x, position.y);
            }
            WindowEvent::PointerButton {
                state: ElementState::Released,
                primary: true,
                position,
                button,
                ..
            } if button.clone().mouse_button() == Some(MouseButton::Left) => {
                let _ = self.release_titlebar(event_loop, &window, position.x, position.y);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.started.elapsed() > Duration::from_millis(500)
            && let Some(child) = self.child.as_mut()
            && matches!(child.try_wait(), Ok(Some(_)))
        {
            event_loop.exit();
        }
    }
}

fn centered_window_position(
    event_loop: &dyn ActiveEventLoop,
    width: u32,
    height: u32,
) -> Option<PhysicalPosition<i32>> {
    let monitor = event_loop
        .primary_monitor()
        .or_else(|| event_loop.available_monitors().next())?;
    let mode = monitor.current_video_mode()?;
    let monitor_size = mode.size();
    let monitor_position = monitor.position()?;
    let scale = monitor.scale_factor().max(1.0);
    let physical_width = (f64::from(width) * scale).round() as i32;
    let physical_height = (f64::from(height) * scale).round() as i32;
    let x = monitor_position.x + (monitor_size.width as i32 - physical_width).max(0) / 2;
    let y = monitor_position.y + (monitor_size.height as i32 - physical_height).max(0) / 2;
    Some(PhysicalPosition::new(x, y))
}

fn spawn_native_host_bridge_proxy(child: &mut Child) {
    if let Some(stdout) = child.stdout.take() {
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            let mut output = io::stdout();
            for line in reader.lines().map_while(std::result::Result::ok) {
                if writeln!(output, "{line}").is_err() {
                    break;
                }
                let _ = output.flush();
            }
        });
    }

    if let Some(mut stdin) = child.stdin.take() {
        thread::spawn(move || {
            let input = io::stdin();
            for line in input.lock().lines().map_while(std::result::Result::ok) {
                if writeln!(stdin, "{line}").is_err() {
                    break;
                }
                let _ = stdin.flush();
            }
        });
    }
}

fn webview_titlebar_height(chrome: WindowChrome, scale_factor: f64) -> u32 {
    if matches!(
        chrome,
        WindowChrome::Stuk | WindowChrome::Compact | WindowChrome::Sidebar
    ) {
        (f64::from(WEBVIEW_TITLEBAR_HEIGHT) * scale_factor.max(1.0)).round() as u32
    } else {
        0
    }
}

fn titlebar_control_at(
    surface_width: u32,
    titlebar_height: u32,
    x: f64,
    y: f64,
) -> Option<WebViewTitlebarControl> {
    if titlebar_height == 0 || y < 0.0 || y > f64::from(titlebar_height) {
        return None;
    }
    let size = (f64::from(titlebar_height) * 0.62).clamp(22.0, 28.0);
    let gap = 8.0;
    let right = 10.0;
    let y0 = (f64::from(titlebar_height) - size) * 0.5;
    let close_x = f64::from(surface_width) - right - size;
    let maximize_x = close_x - gap - size;
    let minimize_x = maximize_x - gap - size;
    [
        (WebViewTitlebarControl::Minimize, minimize_x),
        (WebViewTitlebarControl::Maximize, maximize_x),
        (WebViewTitlebarControl::Close, close_x),
    ]
    .into_iter()
    .find_map(|(control, x0)| {
        (x >= x0 && x <= x0 + size && y >= y0 && y <= y0 + size).then_some(control)
    })
}

fn draw_x11_webview_chrome(
    window: X11Window,
    surface_width: u32,
    surface_height: u32,
    titlebar_height: u32,
    title: &str,
    hovered: Option<WebViewTitlebarControl>,
    pressed: Option<WebViewTitlebarControl>,
) -> std::result::Result<(), String> {
    let (connection, _) = RustConnection::connect(None).map_err(|error| error.to_string())?;
    let background = create_gc(&connection, window, 0x181818, 1)?;
    let titlebar = create_gc(&connection, window, 0x2c2c30, 1)?;
    let separator = create_gc(&connection, window, 0x3a3a3d, 1)?;
    let text = create_gc(&connection, window, 0xf3f3f1, 1)?;
    let icon = create_gc(&connection, window, 0xf3f3f1, 2)?;
    let icon_muted = create_gc(&connection, window, 0xd7d7d4, 2)?;
    connection
        .poly_fill_rectangle(
            window,
            background,
            &[Rectangle {
                x: 0,
                y: 0,
                width: u16_saturating(surface_width),
                height: u16_saturating(surface_height),
            }],
        )
        .map_err(|error| error.to_string())?;
    if titlebar_height > 0 {
        connection
            .poly_fill_rectangle(
                window,
                titlebar,
                &[Rectangle {
                    x: 0,
                    y: 0,
                    width: u16_saturating(surface_width),
                    height: u16_saturating(titlebar_height),
                }],
            )
            .map_err(|error| error.to_string())?;
        connection
            .poly_line(
                CoordMode::ORIGIN,
                window,
                separator,
                &[
                    Point {
                        x: 0,
                        y: i16_saturating(titlebar_height.saturating_sub(1)),
                    },
                    Point {
                        x: i16_saturating(surface_width),
                        y: i16_saturating(titlebar_height.saturating_sub(1)),
                    },
                ],
            )
            .map_err(|error| error.to_string())?;
        draw_x11_title_text(
            &connection,
            window,
            titlebar,
            text,
            surface_width,
            titlebar_height,
            title,
        )?;
        draw_x11_titlebar_controls(
            &connection,
            window,
            surface_width,
            titlebar_height,
            hovered,
            pressed,
            icon,
            icon_muted,
        )?;
    }
    for gc in [background, titlebar, separator, text, icon, icon_muted] {
        let _ = connection.free_gc(gc);
    }
    connection.flush().map_err(|error| error.to_string())
}

fn create_gc(
    connection: &RustConnection,
    window: X11Window,
    color: u32,
    line_width: u32,
) -> std::result::Result<Gcontext, String> {
    let gc = connection
        .generate_id()
        .map_err(|error| error.to_string())?;
    connection
        .create_gc(
            gc,
            window,
            &CreateGCAux::new()
                .foreground(color)
                .background(0x2c2c30)
                .line_width(line_width)
                .graphics_exposures(0),
        )
        .map_err(|error| error.to_string())?;
    Ok(gc)
}

fn draw_x11_title_text(
    connection: &RustConnection,
    window: X11Window,
    background_gc: Gcontext,
    text_gc: Gcontext,
    surface_width: u32,
    titlebar_height: u32,
    title: &str,
) -> std::result::Result<(), String> {
    let title = title.as_bytes();
    let approx_width = title.len() as i32 * 7;
    let x = ((surface_width as i32 - approx_width) / 2).max(16);
    let y = ((titlebar_height as i32 + 9) / 2).max(16);
    connection
        .poly_fill_rectangle(
            window,
            background_gc,
            &[Rectangle {
                x: i16_saturating_i32(x - 4),
                y: i16_saturating_i32(y - 13),
                width: u16_saturating((approx_width + 8).max(1) as u32),
                height: 18,
            }],
        )
        .map_err(|error| error.to_string())?;
    connection
        .image_text8(
            window,
            text_gc,
            i16_saturating_i32(x),
            i16_saturating_i32(y),
            title,
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn draw_x11_titlebar_controls(
    connection: &RustConnection,
    window: X11Window,
    surface_width: u32,
    titlebar_height: u32,
    hovered: Option<WebViewTitlebarControl>,
    pressed: Option<WebViewTitlebarControl>,
    icon_gc: Gcontext,
    icon_muted_gc: Gcontext,
) -> std::result::Result<(), String> {
    let size = (f64::from(titlebar_height) * 0.62)
        .clamp(22.0, 28.0)
        .round() as i16;
    let gap = 8;
    let right = 10;
    let y = ((titlebar_height as i16 - size) / 2).max(0);
    let close_x = i16_saturating(surface_width) - right - size;
    let maximize_x = close_x - gap - size;
    let minimize_x = maximize_x - gap - size;
    for (control, x) in [
        (WebViewTitlebarControl::Minimize, minimize_x),
        (WebViewTitlebarControl::Maximize, maximize_x),
        (WebViewTitlebarControl::Close, close_x),
    ] {
        let fill = if pressed == Some(control) {
            0x555557
        } else if hovered == Some(control) {
            0x47474a
        } else {
            0x3b3b3e
        };
        let fill_gc = create_gc(connection, window, fill, 1)?;
        connection
            .poly_fill_arc(
                window,
                fill_gc,
                &[X11Arc {
                    x,
                    y,
                    width: size as u16,
                    height: size as u16,
                    angle1: 0,
                    angle2: 360 * 64,
                }],
            )
            .map_err(|error| error.to_string())?;
        let _ = connection.free_gc(fill_gc);
        draw_x11_control_icon(
            connection,
            window,
            if hovered == Some(control) || pressed == Some(control) {
                icon_gc
            } else {
                icon_muted_gc
            },
            control,
            x,
            y,
            size,
        )?;
    }
    Ok(())
}

fn draw_x11_control_icon(
    connection: &RustConnection,
    window: X11Window,
    gc: Gcontext,
    control: WebViewTitlebarControl,
    x: i16,
    y: i16,
    size: i16,
) -> std::result::Result<(), String> {
    let c = size / 2;
    let left = x + c - 5;
    let right = x + c + 5;
    let top = y + c - 5;
    let bottom = y + c + 5;
    let middle = y + c + 4;
    let points = match control {
        WebViewTitlebarControl::Minimize => vec![
            Point { x: left, y: middle },
            Point {
                x: right,
                y: middle,
            },
        ],
        WebViewTitlebarControl::Maximize => vec![
            Point { x: left, y: top },
            Point { x: right, y: top },
            Point {
                x: right,
                y: bottom,
            },
            Point { x: left, y: bottom },
            Point { x: left, y: top },
        ],
        WebViewTitlebarControl::Close => {
            connection
                .poly_line(
                    CoordMode::ORIGIN,
                    window,
                    gc,
                    &[
                        Point { x: left, y: top },
                        Point {
                            x: right,
                            y: bottom,
                        },
                    ],
                )
                .map_err(|error| error.to_string())?;
            vec![Point { x: right, y: top }, Point { x: left, y: bottom }]
        }
    };
    connection
        .poly_line(CoordMode::ORIGIN, window, gc, &points)
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn u16_saturating(value: u32) -> u16 {
    value.min(u32::from(u16::MAX)) as u16
}

fn i16_saturating(value: u32) -> i16 {
    value.min(i16::MAX as u32) as i16
}

fn i16_saturating_i32(value: i32) -> i16 {
    value.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16
}

impl Default for WebViewWindow {
    fn default() -> Self {
        Self::new()
    }
}

fn runtime_command(
    runtime_dir: &Path,
    config: &WebViewConfig,
    url: &str,
    force_x11: bool,
) -> Option<Command> {
    for candidate in cef_executable_candidates(runtime_dir) {
        if !candidate.is_file() {
            continue;
        }
        let cache_dir = webview_cache_dir(runtime_dir, &config.title, url);
        let _ = std::fs::create_dir_all(&cache_dir);
        let mut command = Command::new(&candidate);
        command
            .arg(format!("--url={url}"))
            .arg("--enable-chrome-runtime")
            .arg("--use-alloy-style")
            .arg("--use-views")
            .arg("--hide-frame")
            .arg("--disable-vulkan")
            .arg("--disable-gpu")
            .arg("--hide-controls")
            .arg("--hide-overlays")
            .arg(format!(
                "--stuk-bridge-commands={}",
                config.bridge.commands().join(",")
            ))
            .arg(format!("--root-cache-path={}", cache_dir.display()))
            .arg(format!(
                "--cache-path={}",
                cache_dir.join("browser").display()
            ))
            .current_dir(candidate.parent().unwrap_or(runtime_dir));
        if config.transparent {
            command
                .arg("--enable-transparent-visuals")
                .arg("--transparent-painting-enabled")
                .arg("--default-background-color=0x00000000");
        }
        if force_x11 {
            command.arg("--ozone-platform=x11");
        } else if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            command
                .arg("--ozone-platform=wayland")
                .arg("--enable-features=UseOzonePlatform");
        }
        return Some(command);
    }
    let _ = url;
    None
}

fn cef_executable_candidates(runtime_dir: &Path) -> Vec<PathBuf> {
    launchable_cef_host_candidates(runtime_dir)
}

fn ld_library_path(release_dir: &Path) -> String {
    let existing = std::env::var("LD_LIBRARY_PATH").unwrap_or_default();
    if existing.is_empty() {
        release_dir.display().to_string()
    } else {
        format!("{}:{existing}", release_dir.display())
    }
}

fn webview_cache_dir(_runtime_dir: &Path, title: &str, url: &str) -> PathBuf {
    user_cache_home()
        .join("stuk")
        .join("webviews")
        .join(webview_cache_key(title, url))
        .join("instances")
        .join(webview_instance_key())
}

fn webview_cache_key(title: &str, url: &str) -> String {
    let executable = std::env::current_exe()
        .ok()
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    format!("{:016x}", stable_hash(&[&executable, title, url]))
}

fn webview_instance_key() -> String {
    let counter = WEBVIEW_INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{}-{counter}-{timestamp}", std::process::id())
}

fn user_cache_home() -> PathBuf {
    std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
        .unwrap_or_else(std::env::temp_dir)
}

fn stable_hash(values: &[&str]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for value in values {
        for byte in value.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash ^= 0xff;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn remove_system_decorations(pid: u32, previous_windows: BTreeSet<String>) {
    #[cfg(target_os = "linux")]
    {
        for _ in 0..40 {
            let window_id =
                find_x11_window_for_pid(pid).or_else(|| find_new_x11_window(&previous_windows));
            let Some(window_id) = window_id else {
                std::thread::sleep(Duration::from_millis(75));
                continue;
            };
            let _ = Command::new("xprop")
                .args([
                    "-id",
                    &window_id,
                    "-f",
                    "_MOTIF_WM_HINTS",
                    "32c",
                    "-set",
                    "_MOTIF_WM_HINTS",
                    "0x2, 0x0, 0x0, 0x0, 0x0",
                ])
                .output();
            return;
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        let _ = previous_windows;
    }
}

fn find_x11_child(parent: X11Window) -> Option<X11Window> {
    let (connection, _) = RustConnection::connect(None).ok()?;
    connection
        .query_tree(parent)
        .ok()?
        .reply()
        .ok()?
        .children
        .into_iter()
        .next()
}

fn resize_x11_window(
    child: X11Window,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> std::result::Result<(), String> {
    let (connection, _) = RustConnection::connect(None).map_err(|error| error.to_string())?;
    connection
        .configure_window(
            child,
            &ConfigureWindowAux::new()
                .x(x)
                .y(y)
                .width(width.max(1))
                .height(height.max(1)),
        )
        .map_err(|error| error.to_string())?;
    connection.flush().map_err(|error| error.to_string())
}

#[cfg(target_os = "linux")]
fn find_x11_window_for_pid(pid: u32) -> Option<String> {
    for window_id in x11_client_windows() {
        let output = Command::new("xprop")
            .args(["-id", &window_id, "_NET_WM_PID"])
            .output()
            .ok()?;
        if !output.status.success() {
            continue;
        }
        let props = String::from_utf8_lossy(&output.stdout);
        if props
            .split(|ch: char| !ch.is_ascii_digit())
            .any(|part| part.parse::<u32>().ok() == Some(pid))
        {
            return Some(window_id);
        }
    }

    None
}

fn x11_client_windows() -> BTreeSet<String> {
    let root = Command::new("xprop")
        .args(["-root", "_NET_CLIENT_LIST"])
        .output();
    let Ok(root) = root else {
        return BTreeSet::new();
    };
    if !root.status.success() {
        return BTreeSet::new();
    }

    let text = String::from_utf8_lossy(&root.stdout);
    text.split(|ch: char| ch.is_whitespace() || ch == ',')
        .filter(|part| part.starts_with("0x"))
        .map(ToString::to_string)
        .collect()
}

#[cfg(not(target_os = "linux"))]
fn x11_client_windows() -> BTreeSet<String> {
    BTreeSet::new()
}

#[cfg(target_os = "linux")]
fn find_new_x11_window(previous_windows: &BTreeSet<String>) -> Option<String> {
    x11_client_windows()
        .into_iter()
        .find(|window_id| !previous_windows.contains(window_id))
}

#[derive(Clone, Debug)]
pub struct BridgeCommand {
    pub name: String,
    pub params: serde_json::Value,
    pub origin: Option<String>,
}

#[derive(Clone, Debug)]
pub struct BridgeResponse {
    pub result: serde_json::Value,
}

impl BridgeResponse {
    pub fn json(result: serde_json::Value) -> Self {
        Self { result }
    }
}

#[derive(Clone, Debug)]
pub struct BridgeError {
    pub message: String,
}

impl BridgeError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

pub type BridgeResult = std::result::Result<BridgeResponse, BridgeError>;
type BridgeHandler = Arc<dyn Fn(BridgeCommand) -> BridgeResult + Send + Sync>;

#[derive(Clone, Default)]
pub struct BridgeHandlers {
    handlers: BTreeMap<String, BridgeHandler>,
}

impl std::fmt::Debug for BridgeHandlers {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BridgeHandlers")
            .field("commands", &self.commands())
            .finish()
    }
}

impl BridgeHandlers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<F>(&mut self, command_name: impl Into<String>, handler: F)
    where
        F: Fn(BridgeCommand) -> BridgeResult + Send + Sync + 'static,
    {
        self.handlers.insert(command_name.into(), Arc::new(handler));
    }

    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }

    pub fn contains(&self, command_name: &str) -> bool {
        self.handlers.contains_key(command_name)
    }

    pub fn commands(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }

    fn dispatch(&self, command: BridgeCommand) -> BridgeResult {
        let Some(handler) = self.handlers.get(&command.name) else {
            return Err(BridgeError::new(format!(
                "Bridge command `{}` is not registered",
                command.name
            )));
        };
        handler(command)
    }
}

#[derive(Clone, Debug)]
struct BridgeRuntime {
    handlers: BridgeHandlers,
    registry: BridgeRegistry,
    security: WebViewSecurity,
}

impl BridgeRuntime {
    fn new(handlers: BridgeHandlers, registry: BridgeRegistry, security: WebViewSecurity) -> Self {
        Self {
            handlers,
            registry,
            security,
        }
    }

    fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }

    fn dispatch(&self, command: BridgeCommand) -> BridgeResult {
        let descriptor = self.registry.descriptor(&command.name);
        self.validate_permissions(&command, descriptor)?;
        self.validate_targets(&command, descriptor)?;
        self.validate_origin(&command, descriptor)?;
        self.handlers.dispatch(command)
    }

    fn validate_targets(
        &self,
        command: &BridgeCommand,
        descriptor: Option<&BridgeCommandDescriptor>,
    ) -> std::result::Result<(), BridgeError> {
        let Some(descriptor) = descriptor else {
            return Ok(());
        };
        if descriptor.targets.is_empty() {
            return Ok(());
        }
        let active = current_bridge_targets();
        if descriptor
            .targets
            .iter()
            .any(|target| active.iter().any(|active| active == target))
        {
            return Ok(());
        }
        Err(BridgeError::new(format!(
            "Bridge command `{}` is unavailable on this target",
            command.name
        )))
    }

    fn validate_permissions(
        &self,
        command: &BridgeCommand,
        descriptor: Option<&BridgeCommandDescriptor>,
    ) -> std::result::Result<(), BridgeError> {
        let Some(descriptor) = descriptor else {
            return Ok(());
        };
        for permission in &descriptor.permissions {
            if !self
                .security
                .allowed_bridge_permissions
                .iter()
                .any(|allowed| allowed == permission || allowed == "*")
            {
                return Err(BridgeError::new(format!(
                    "Bridge command `{}` requires permission `{permission}`",
                    command.name
                )));
            }
        }
        Ok(())
    }

    fn validate_origin(
        &self,
        command: &BridgeCommand,
        descriptor: Option<&BridgeCommandDescriptor>,
    ) -> std::result::Result<(), BridgeError> {
        let Some(origin) = command.origin.as_deref() else {
            return Ok(());
        };
        if is_local_bridge_origin(origin) {
            return Ok(());
        }

        let command_origins = descriptor
            .map(|descriptor| descriptor.allowed_origins.as_slice())
            .unwrap_or(&[]);
        if origin_matches_any(origin, command_origins) {
            return Ok(());
        }

        if self.security.remote_content
            && origin_matches_any(origin, self.security.allowed_origins.as_slice())
        {
            return Ok(());
        }

        Err(BridgeError::new(format!(
            "Bridge command `{}` is not allowed from origin `{origin}`",
            command.name
        )))
    }
}

fn is_local_bridge_origin(origin: &str) -> bool {
    origin == "null"
        || origin == "about:blank"
        || origin.starts_with("file://")
        || origin.starts_with("devtools://")
}

fn origin_matches_any(origin: &str, allowed: &[String]) -> bool {
    allowed.iter().any(|candidate| {
        candidate == origin
            || candidate == "*"
            || (candidate.ends_with("/*") && origin.starts_with(candidate.trim_end_matches('*')))
    })
}

fn current_bridge_targets() -> &'static [&'static str] {
    #[cfg(target_os = "linux")]
    {
        &["desktop", "linux"]
    }
    #[cfg(target_os = "windows")]
    {
        &["desktop", "windows"]
    }
    #[cfg(target_os = "macos")]
    {
        &["desktop", "macos"]
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        &["desktop"]
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BridgeCommandDescriptor {
    pub name: String,
    pub description: Option<String>,
    pub params_schema: Option<serde_json::Value>,
    pub permissions: Vec<String>,
    pub allowed_origins: Vec<String>,
    pub targets: Vec<String>,
}

impl BridgeCommandDescriptor {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            params_schema: None,
            permissions: Vec::new(),
            allowed_origins: Vec::new(),
            targets: Vec::new(),
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn params_schema(mut self, schema: serde_json::Value) -> Self {
        self.params_schema = Some(schema);
        self
    }

    pub fn permission(mut self, permission: impl Into<String>) -> Self {
        self.permissions.push(permission.into());
        self
    }

    pub fn allowed_origin(mut self, origin: impl Into<String>) -> Self {
        self.allowed_origins.push(origin.into());
        self
    }

    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.targets.push(target.into());
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct BridgeRegistry {
    commands: Vec<BridgeCommandDescriptor>,
}

impl BridgeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, command_name: impl Into<String>) {
        self.register_descriptor(BridgeCommandDescriptor::new(command_name));
    }

    pub fn register_descriptor(&mut self, command: BridgeCommandDescriptor) {
        if !self.is_registered(&command.name) {
            self.commands.push(command);
        }
    }

    pub fn is_registered(&self, command_name: &str) -> bool {
        self.commands.iter().any(|c| c.name == command_name)
    }

    pub fn descriptors(&self) -> &[BridgeCommandDescriptor] {
        &self.commands
    }

    pub fn descriptor(&self, command_name: &str) -> Option<&BridgeCommandDescriptor> {
        self.commands
            .iter()
            .find(|command| command.name == command_name)
    }

    pub fn commands(&self) -> Vec<String> {
        self.commands
            .iter()
            .map(|command| command.name.clone())
            .collect()
    }

    pub fn capabilities_json(&self) -> serde_json::Value {
        serde_json::json!({
            "commands": self.commands.iter().map(|command| {
                serde_json::json!({
                    "name": &command.name,
                    "description": &command.description,
                    "paramsSchema": &command.params_schema,
                    "permissions": &command.permissions,
                    "allowedOrigins": &command.allowed_origins,
                    "targets": &command.targets,
                })
            }).collect::<Vec<_>>()
        })
    }

    pub fn js_api(&self) -> String {
        let commands = serde_json::to_string(&self.commands()).unwrap_or_else(|_| "[]".to_string());
        let capabilities =
            serde_json::to_string(&self.capabilities_json()).unwrap_or_else(|_| "{}".to_string());
        format!(
            r#"(function(){{
  const commands = new Set({commands});
  const capabilities = {capabilities};
  const pending = new Map();
  let nextId = 1;
  window.__stukBridgeResolve = function(id, ok, payload) {{
    const key = String(id);
    const entry = pending.get(key);
    if (!entry) return;
    pending.delete(key);
    if (ok) {{
      entry.resolve(payload);
    }} else {{
      entry.reject(new Error((payload && payload.message) || "Stuk bridge command failed"));
    }}
  }};
  window.stuk = window.stuk || {{}};
  window.stuk.bridge = {{
    __native: true,
    commands: Array.from(commands),
    capabilities,
    invoke(name, params = {{}}) {{
      if (!commands.has(name)) {{
        return Promise.reject(new Error(`Stuk bridge command not registered: ${{name}}`));
      }}
      const id = String(nextId++);
      const payload = encodeURIComponent(JSON.stringify(params));
      const url = `stuk://bridge/${{encodeURIComponent(id)}}?name=${{encodeURIComponent(name)}}&payload=${{payload}}`;
      return new Promise((resolve, reject) => {{
        pending.set(id, {{ resolve, reject }});
        setTimeout(() => {{
          if (pending.has(id)) {{
            pending.delete(id);
            reject(new Error(`Stuk bridge command timed out: ${{name}}`));
          }}
        }}, 60000);
        window.location.href = url;
      }});
    }}
  }};
}})();"#
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn webview_window_has_secure_defaults() {
        let window = WebViewWindow::new();
        let security = &window.config.security;
        assert!(!security.remote_content);
        assert!(!security.allow_eval);
        assert!(!security.allow_node);
        assert_eq!(security.devtools, WebViewDevtools::DevOnly);
        assert!(security.csp.contains("default-src 'self'"));
    }

    #[test]
    fn bridge_registry_tracks_commands() {
        let mut registry = BridgeRegistry::new();
        registry.register("unlock_vault");
        registry.register("save_note");
        registry.register("unlock_vault");
        assert!(registry.is_registered("unlock_vault"));
        assert!(registry.is_registered("save_note"));
        assert!(!registry.is_registered("delete_all"));
        assert_eq!(registry.commands().len(), 2);
    }

    #[test]
    fn webview_config_builder() {
        let window = WebViewWindow::new()
            .entry("ui/dist/index.html")
            .dev_url("http://localhost:5173")
            .material(Material::Maris)
            .chrome(WindowChrome::Compact)
            .transparent(true);
        assert_eq!(window.config.entry.as_deref(), Some("ui/dist/index.html"));
        assert_eq!(
            window.config.dev_url.as_deref(),
            Some("http://localhost:5173")
        );
        assert!(window.config.transparent);
        assert_eq!(window.config.runtime.engine, RuntimeEngine::Cef);
    }
}
