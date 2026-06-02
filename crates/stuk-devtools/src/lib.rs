mod accessibility;
mod bundle;
mod element;
mod layout_metrics;
mod manifest;
#[cfg(test)]
mod manifest_tests;
mod performance;
mod platform;
mod preview;

pub use accessibility::{
    AccessibilityDiagnosticInspection, AccessibilityInspection, inspect_accessibility,
};
pub use bundle::{BundlePlan, BundleTarget, StaccatoBundleMetadata};
pub use element::{
    ElementSnapshot, LayoutSnapshot, inspect_element, inspect_layout, inspect_layout_for_window,
};
pub use layout_metrics::LayoutMetrics;
pub use manifest::{
    ActionInspection, AppInspection, DiagnosticInspection, ManifestInspection,
    PermissionInspection, TargetInspection, WindowInspection, inspect_manifest,
    inspect_manifest_with_base_dir,
};
pub use performance::{FrameHealth, PerformanceOverlay, PerformanceSample};
pub use platform::{
    BackendInspection, CapabilityInspection, MaterialInspection, PlatformInspection,
    inspect_platform,
};
pub use preview::{PreviewDescriptor, PreviewElement, PreviewRegistry};
