use std::collections::BTreeSet;

use crate::PlatformCapabilities;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PlatformFamily {
    Desktop,
    Mobile,
    Web,
}

impl PlatformFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::Mobile => "mobile",
            Self::Web => "web",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PlatformOs {
    Linux,
    Windows,
    Macos,
    Android,
    Ios,
    Web,
    Unknown,
}

impl PlatformOs {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Linux => "linux",
            Self::Windows => "windows",
            Self::Macos => "macos",
            Self::Android => "android",
            Self::Ios => "ios",
            Self::Web => "web",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AppTarget {
    Desktop,
    Linux,
    Windows,
    Macos,
    Android,
    Ios,
    Web,
}

impl AppTarget {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::Linux => "linux",
            Self::Windows => "windows",
            Self::Macos => "macos",
            Self::Android => "android",
            Self::Ios => "ios",
            Self::Web => "web",
        }
    }

    pub fn matches(self, target: RuntimeTarget) -> bool {
        match self {
            Self::Desktop => target.is_desktop(),
            Self::Linux => target == RuntimeTarget::desktop(PlatformOs::Linux),
            Self::Windows => target == RuntimeTarget::desktop(PlatformOs::Windows),
            Self::Macos => target == RuntimeTarget::desktop(PlatformOs::Macos),
            Self::Android => target == RuntimeTarget::mobile(PlatformOs::Android),
            Self::Ios => target == RuntimeTarget::mobile(PlatformOs::Ios),
            Self::Web => target.is_web(),
        }
    }

    pub fn current() -> Self {
        let target = BackendDescriptor::current_native().target;
        Self::from_runtime_target(target).unwrap_or(Self::Desktop)
    }

    pub fn from_runtime_target(target: RuntimeTarget) -> Option<Self> {
        match target {
            RuntimeTarget {
                family: PlatformFamily::Desktop,
                os: PlatformOs::Linux,
            } => Some(Self::Linux),
            RuntimeTarget {
                family: PlatformFamily::Desktop,
                os: PlatformOs::Windows,
            } => Some(Self::Windows),
            RuntimeTarget {
                family: PlatformFamily::Desktop,
                os: PlatformOs::Macos,
            } => Some(Self::Macos),
            RuntimeTarget {
                family: PlatformFamily::Mobile,
                os: PlatformOs::Android,
            } => Some(Self::Android),
            RuntimeTarget {
                family: PlatformFamily::Mobile,
                os: PlatformOs::Ios,
            } => Some(Self::Ios),
            RuntimeTarget {
                family: PlatformFamily::Web,
                os: PlatformOs::Web,
            } => Some(Self::Web),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BackendKind {
    NativeDesktop,
    NativeMobile,
    BrowserWeb,
    CefWebView,
    Headless,
}

impl BackendKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NativeDesktop => "native-desktop",
            Self::NativeMobile => "native-mobile",
            Self::BrowserWeb => "browser-web",
            Self::CefWebView => "cef-webview",
            Self::Headless => "headless",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PlatformOverrideKind {
    AppShell,
    Page,
    Component,
    Command,
    Service,
}

impl PlatformOverrideKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AppShell => "app-shell",
            Self::Page => "page",
            Self::Component => "component",
            Self::Command => "command",
            Self::Service => "service",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlatformOverride {
    pub target: AppTarget,
    pub kind: PlatformOverrideKind,
    pub id: String,
}

impl PlatformOverride {
    pub fn new(target: AppTarget, kind: PlatformOverrideKind, id: impl Into<String>) -> Self {
        Self {
            target,
            kind,
            id: id.into(),
        }
    }

    pub fn matches(
        &self,
        runtime_target: RuntimeTarget,
        kind: PlatformOverrideKind,
        id: &str,
    ) -> bool {
        self.target.matches(runtime_target) && self.kind == kind && self.id == id
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlatformOverrideRegistry {
    overrides: Vec<PlatformOverride>,
}

impl PlatformOverrideRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, override_entry: PlatformOverride) {
        if !self.overrides.contains(&override_entry) {
            self.overrides.push(override_entry);
        }
    }

    pub fn has(&self, runtime_target: RuntimeTarget, kind: PlatformOverrideKind, id: &str) -> bool {
        self.overrides
            .iter()
            .any(|entry| entry.matches(runtime_target, kind, id))
    }

    pub fn entries(&self) -> &[PlatformOverride] {
        &self.overrides
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendStatus {
    Available,
    Preview,
    Unsupported,
}

impl BackendStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Preview => "preview",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeTarget {
    pub family: PlatformFamily,
    pub os: PlatformOs,
}

impl RuntimeTarget {
    pub const fn desktop(os: PlatformOs) -> Self {
        Self {
            family: PlatformFamily::Desktop,
            os,
        }
    }

    pub const fn mobile(os: PlatformOs) -> Self {
        Self {
            family: PlatformFamily::Mobile,
            os,
        }
    }

    pub const fn web() -> Self {
        Self {
            family: PlatformFamily::Web,
            os: PlatformOs::Web,
        }
    }

    pub fn target_names(self) -> Vec<&'static str> {
        match self.family {
            PlatformFamily::Desktop => vec!["desktop", self.os.as_str()],
            PlatformFamily::Mobile => vec![self.os.as_str()],
            PlatformFamily::Web => vec!["web"],
        }
    }

    pub fn is_desktop(self) -> bool {
        self.family == PlatformFamily::Desktop
    }

    pub fn is_mobile(self) -> bool {
        self.family == PlatformFamily::Mobile
    }

    pub fn is_web(self) -> bool {
        self.family == PlatformFamily::Web
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackendDescriptor {
    pub name: String,
    pub kind: BackendKind,
    pub target: RuntimeTarget,
    pub status: BackendStatus,
    pub capabilities: PlatformCapabilities,
    pub limitations: Vec<String>,
}

impl BackendDescriptor {
    pub fn new(
        name: impl Into<String>,
        kind: BackendKind,
        target: RuntimeTarget,
        status: BackendStatus,
        capabilities: PlatformCapabilities,
    ) -> Self {
        Self {
            name: name.into(),
            kind,
            target,
            status,
            capabilities,
            limitations: Vec::new(),
        }
    }

    pub fn limitation(mut self, limitation: impl Into<String>) -> Self {
        self.limitations.push(limitation.into());
        self
    }

    pub fn generic() -> Self {
        Self::new(
            "generic",
            BackendKind::Headless,
            RuntimeTarget::desktop(PlatformOs::Unknown),
            BackendStatus::Preview,
            PlatformCapabilities::generic(),
        )
        .limitation("no OS windowing backend is attached")
    }

    pub fn linux_wayland(background_effects: bool) -> Self {
        Self::new(
            "linux-wayland",
            BackendKind::NativeDesktop,
            RuntimeTarget::desktop(PlatformOs::Linux),
            BackendStatus::Available,
            PlatformCapabilities::desktop_linux(background_effects, true),
        )
    }

    pub fn linux_x11() -> Self {
        Self::new(
            "linux-x11",
            BackendKind::NativeDesktop,
            RuntimeTarget::desktop(PlatformOs::Linux),
            BackendStatus::Available,
            PlatformCapabilities::desktop_linux(false, false),
        )
        .limitation("X11 compatibility backend has no native compositor blur")
    }

    pub fn windows() -> Self {
        Self::new(
            "windows",
            BackendKind::NativeDesktop,
            RuntimeTarget::desktop(PlatformOs::Windows),
            BackendStatus::Available,
            PlatformCapabilities::desktop_windows(true),
        )
    }

    pub fn macos() -> Self {
        Self::new(
            "macos",
            BackendKind::NativeDesktop,
            RuntimeTarget::desktop(PlatformOs::Macos),
            BackendStatus::Available,
            PlatformCapabilities::desktop_macos(true),
        )
    }

    pub fn android() -> Self {
        Self::new(
            "android",
            BackendKind::NativeMobile,
            RuntimeTarget::mobile(PlatformOs::Android),
            BackendStatus::Preview,
            PlatformCapabilities::mobile_android(),
        )
        .limitation("mobile shell, packaging, and platform accessibility are not complete yet")
    }

    pub fn ios() -> Self {
        Self::new(
            "ios",
            BackendKind::NativeMobile,
            RuntimeTarget::mobile(PlatformOs::Ios),
            BackendStatus::Preview,
            PlatformCapabilities::mobile_ios(),
        )
        .limitation("mobile shell, packaging, and platform accessibility are not complete yet")
    }

    pub fn browser_web() -> Self {
        Self::new(
            "web",
            BackendKind::BrowserWeb,
            RuntimeTarget::web(),
            BackendStatus::Preview,
            PlatformCapabilities::browser_web(),
        )
        .limitation("wasm packaging and browser service adapters are still preview")
    }

    pub fn cef_webview() -> Self {
        Self::new(
            "cef-webview",
            BackendKind::CefWebView,
            RuntimeTarget::desktop(current_desktop_os()),
            BackendStatus::Available,
            PlatformCapabilities::cef_webview(),
        )
    }

    pub fn current_native() -> Self {
        current_native_backend()
    }

    pub fn supports_targets(&self, targets: &TargetSet) -> bool {
        targets.supports(self.target)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TargetSet {
    enabled: BTreeSet<String>,
}

impl TargetSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn desktop() -> Self {
        Self::from_enabled(["desktop", current_desktop_os().as_str()])
    }

    pub fn from_enabled(values: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        Self {
            enabled: values
                .into_iter()
                .map(|value| value.as_ref().to_string())
                .collect(),
        }
    }

    pub fn insert(&mut self, target: impl Into<String>) {
        self.enabled.insert(target.into());
    }

    pub fn contains(&self, target: &str) -> bool {
        self.enabled.contains(target)
    }

    pub fn is_empty(&self) -> bool {
        self.enabled.is_empty()
    }

    pub fn enabled(&self) -> impl Iterator<Item = &str> {
        self.enabled.iter().map(String::as_str)
    }

    pub fn supports(&self, target: RuntimeTarget) -> bool {
        if self.enabled.is_empty() {
            return target.is_desktop();
        }

        match target.family {
            PlatformFamily::Desktop => {
                self.enabled.contains("desktop")
                    && (!self.has_platform_specific_desktop_targets()
                        || self.enabled.contains(target.os.as_str()))
            }
            PlatformFamily::Mobile | PlatformFamily::Web => {
                self.enabled.contains(target.os.as_str())
            }
        }
    }

    fn has_platform_specific_desktop_targets(&self) -> bool {
        ["linux", "windows", "macos"]
            .iter()
            .any(|target| self.enabled.contains(*target))
    }
}

pub fn current_native_backend() -> BackendDescriptor {
    #[cfg(target_arch = "wasm32")]
    {
        return BackendDescriptor::browser_web();
    }

    #[cfg(target_os = "android")]
    {
        return BackendDescriptor::android();
    }

    #[cfg(target_os = "ios")]
    {
        return BackendDescriptor::ios();
    }

    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            return BackendDescriptor::linux_wayland(true);
        }
        return BackendDescriptor::linux_x11();
    }

    #[cfg(target_os = "windows")]
    {
        return BackendDescriptor::windows();
    }

    #[cfg(target_os = "macos")]
    {
        return BackendDescriptor::macos();
    }

    #[allow(unreachable_code)]
    BackendDescriptor::generic()
}

pub fn current_desktop_os() -> PlatformOs {
    #[cfg(target_os = "linux")]
    {
        return PlatformOs::Linux;
    }

    #[cfg(target_os = "windows")]
    {
        return PlatformOs::Windows;
    }

    #[cfg(target_os = "macos")]
    {
        return PlatformOs::Macos;
    }

    #[allow(unreachable_code)]
    PlatformOs::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_target_set_can_be_generic_or_platform_specific() {
        let linux = RuntimeTarget::desktop(PlatformOs::Linux);
        let windows = RuntimeTarget::desktop(PlatformOs::Windows);

        assert!(TargetSet::from_enabled(["desktop"]).supports(linux));
        assert!(TargetSet::from_enabled(["desktop"]).supports(windows));
        assert!(TargetSet::from_enabled(["desktop", "linux"]).supports(linux));
        assert!(!TargetSet::from_enabled(["desktop", "linux"]).supports(windows));
    }

    #[test]
    fn mobile_and_web_targets_are_explicit() {
        assert!(
            TargetSet::from_enabled(["android"])
                .supports(RuntimeTarget::mobile(PlatformOs::Android))
        );
        assert!(TargetSet::from_enabled(["web"]).supports(RuntimeTarget::web()));
        assert!(!TargetSet::from_enabled(["desktop"]).supports(RuntimeTarget::web()));
    }
}
