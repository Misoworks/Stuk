use std::{env, path::Path, process::Command};

use stuk_manifest::{DiagnosticLevel, parse_file, validate_with_base_dir};

pub struct DoctorReport {
    checks: Vec<DoctorCheck>,
}

impl DoctorReport {
    pub fn collect() -> Self {
        let mut report = Self { checks: Vec::new() };
        report.check_tool("Rust toolchain", "cargo", "cargo", &["--version"]);
        report.check_tool("Rust toolchain", "rustc", "rustc", &["--version"]);
        report.check_workspace();
        report.check_platform();
        report.check_renderer();
        report.check_system_dependencies();
        report.check_staccato();
        report
    }

    pub fn has_errors(&self) -> bool {
        self.checks
            .iter()
            .any(|check| check.status == DoctorStatus::Error)
    }

    pub fn to_text(&self) -> String {
        let mut output = String::from("Doctor\n");
        let mut current_category = "";
        for check in &self.checks {
            if check.category != current_category {
                current_category = &check.category;
                output.push_str(current_category);
                output.push('\n');
            }
            output.push_str(&format!(
                "  {} {}: {}\n",
                check.status.as_str(),
                check.name,
                check.message
            ));
            if let Some(detail) = &check.detail {
                output.push_str(&format!("    {detail}\n"));
            }
            if let Some(fix_hint) = &check.fix_hint {
                output.push_str(&format!("    fix: {fix_hint}\n"));
            }
        }
        output
    }

    pub fn to_json(&self) -> String {
        let checks = self
            .checks
            .iter()
            .map(DoctorCheck::to_json)
            .collect::<Vec<_>>()
            .join(",");
        format!("{{\"ok\":{},\"checks\":[{}]}}", !self.has_errors(), checks)
    }

    fn check_tool(&mut self, category: &str, name: &str, command: &str, args: &[&str]) {
        match command_stdout(command, args) {
            Ok(output) => self.ok(category, name, output, None),
            Err(error) => self.error(
                category,
                name,
                format!("{command} is not available"),
                Some(error),
                Some(format!("Install {command} and ensure it is on PATH.")),
            ),
        }
    }

    fn check_workspace(&mut self) {
        if Path::new("Cargo.toml").is_file() {
            self.ok("Workspace", "Cargo.toml", "found", None);
        } else {
            self.warn(
                "Workspace",
                "Cargo.toml",
                "not found",
                None,
                Some("Run this command from a Stuk app or workspace root.".to_string()),
            );
        }

        if Path::new("Stuk.toml").is_file() {
            self.check_manifest();
        } else {
            self.warn(
                "Workspace",
                "Stuk.toml",
                "not found",
                None,
                Some("Run from a Stuk app root to validate app metadata.".to_string()),
            );
        }
    }

    fn check_manifest(&mut self) {
        match parse_file("Stuk.toml") {
            Ok(manifest) => {
                let diagnostics = validate_with_base_dir(&manifest, ".");
                let errors = diagnostics
                    .iter()
                    .filter(|diagnostic| diagnostic.level == DiagnosticLevel::Error)
                    .count();
                let warnings = diagnostics.len().saturating_sub(errors);
                if errors > 0 {
                    self.error(
                        "Workspace",
                        "Stuk.toml",
                        format!("{errors} validation error(s)"),
                        None,
                        Some("Run `stuk validate` for detailed diagnostics.".to_string()),
                    );
                } else if warnings > 0 {
                    self.warn(
                        "Workspace",
                        "Stuk.toml",
                        format!("{warnings} warning(s)"),
                        None,
                        Some("Run `stuk validate` for detailed diagnostics.".to_string()),
                    );
                } else {
                    self.ok("Workspace", "Stuk.toml", "valid", None);
                }
            }
            Err(error) => self.error(
                "Workspace",
                "Stuk.toml",
                "failed to parse",
                Some(error.to_string()),
                Some("Fix the manifest syntax before running the app.".to_string()),
            ),
        }
    }

    fn check_platform(&mut self) {
        self.ok(
            "Platform",
            "os",
            env::consts::OS,
            Some(env::consts::ARCH.to_string()),
        );
        if cfg!(target_os = "linux") {
            match display_backend() {
                Some(display) => self.ok("Platform", "display", display, None),
                None => self.warn(
                    "Platform",
                    "display",
                    "no Wayland or X11 display detected",
                    None,
                    Some(
                        "Set WAYLAND_DISPLAY or DISPLAY before running native windows.".to_string(),
                    ),
                ),
            }
        } else {
            self.ok("Platform", "backend", "generic native backend", None);
        }
    }

    fn check_renderer(&mut self) {
        self.ok("Renderer", "gpu", "wgpu backend linked", None);
        self.ok("Renderer", "text", "glyphon text renderer linked", None);
        self.ok(
            "Renderer",
            "accessibility",
            "AccessKit tree support linked",
            None,
        );
    }

    fn check_system_dependencies(&mut self) {
        if !cfg!(target_os = "linux") {
            self.ok(
                "System dependencies",
                "native deps",
                "using platform framework defaults",
                None,
            );
            return;
        }

        if command_stdout("pkg-config", &["--version"]).is_err() {
            self.warn(
                "System dependencies",
                "pkg-config",
                "not found",
                None,
                Some(
                    "Install pkg-config to diagnose Wayland/X11 development libraries.".to_string(),
                ),
            );
            return;
        }

        self.ok("System dependencies", "pkg-config", "found", None);
        self.check_pkg_config("wayland-client");
        self.check_pkg_config("xkbcommon");
        self.check_pkg_config("x11");
    }

    fn check_pkg_config(&mut self, package: &str) {
        if command_success("pkg-config", &["--exists", package]) {
            self.ok("System dependencies", package, "found", None);
        } else {
            self.warn(
                "System dependencies",
                package,
                "not found by pkg-config",
                None,
                Some(format!(
                    "Install the {package} development package if native builds fail."
                )),
            );
        }
    }

    fn check_staccato(&mut self) {
        if env_present("BATON_SOCKET") {
            self.ok("Staccato", "Baton", "BATON_SOCKET detected", None);
        } else {
            self.warn(
                "Staccato",
                "Baton",
                "not detected",
                None,
                Some("Generic material fallbacks will be used outside Baton.".to_string()),
            );
        }

        if env_present("STACCATO_SESSION") || desktop_name_contains("staccato") {
            self.ok("Staccato", "session", "detected", None);
        } else {
            self.warn(
                "Staccato",
                "session",
                "not detected",
                None,
                Some(
                    "Shell tabs, command palette, and workspace sessions will use fallbacks."
                        .to_string(),
                ),
            );
        }
    }

    fn ok(
        &mut self,
        category: impl Into<String>,
        name: impl Into<String>,
        message: impl Into<String>,
        detail: Option<String>,
    ) {
        self.push(
            DoctorStatus::Ok,
            category,
            name,
            message,
            detail,
            None::<String>,
        );
    }

    fn warn(
        &mut self,
        category: impl Into<String>,
        name: impl Into<String>,
        message: impl Into<String>,
        detail: Option<String>,
        fix_hint: Option<String>,
    ) {
        self.push(
            DoctorStatus::Warning,
            category,
            name,
            message,
            detail,
            fix_hint,
        );
    }

    fn error(
        &mut self,
        category: impl Into<String>,
        name: impl Into<String>,
        message: impl Into<String>,
        detail: Option<String>,
        fix_hint: Option<String>,
    ) {
        self.push(
            DoctorStatus::Error,
            category,
            name,
            message,
            detail,
            fix_hint,
        );
    }

    fn push(
        &mut self,
        status: DoctorStatus,
        category: impl Into<String>,
        name: impl Into<String>,
        message: impl Into<String>,
        detail: Option<String>,
        fix_hint: Option<String>,
    ) {
        self.checks.push(DoctorCheck {
            category: category.into(),
            name: name.into(),
            status,
            message: message.into(),
            detail,
            fix_hint,
        });
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DoctorCheck {
    category: String,
    name: String,
    status: DoctorStatus,
    message: String,
    detail: Option<String>,
    fix_hint: Option<String>,
}

impl DoctorCheck {
    fn to_json(&self) -> String {
        format!(
            "{{\"category\":\"{}\",\"name\":\"{}\",\"status\":\"{}\",\"message\":\"{}\",\"detail\":{},\"fix_hint\":{}}}",
            escape_json(&self.category),
            escape_json(&self.name),
            self.status.as_str(),
            escape_json(&self.message),
            optional_json_string(self.detail.as_deref()),
            optional_json_string(self.fix_hint.as_deref())
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DoctorStatus {
    Ok,
    Warning,
    Error,
}

impl DoctorStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

pub fn run_doctor(json: bool) -> std::process::ExitCode {
    let report = DoctorReport::collect();
    if json {
        println!("{}", report.to_json());
    } else {
        print!("{}", report.to_text());
    }
    if report.has_errors() {
        std::process::ExitCode::from(1)
    } else {
        std::process::ExitCode::SUCCESS
    }
}

fn command_stdout(command: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(command)
        .args(args)
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(output.status.to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn command_success(command: &str, args: &[&str]) -> bool {
    Command::new(command)
        .args(args)
        .status()
        .is_ok_and(|status| status.success())
}

fn display_backend() -> Option<&'static str> {
    if env_present("WAYLAND_DISPLAY") {
        Some("Wayland")
    } else if env_present("DISPLAY") {
        Some("X11")
    } else {
        None
    }
}

fn env_present(name: &str) -> bool {
    env::var_os(name).is_some_and(|value| !value.is_empty())
}

fn desktop_name_contains(value: &str) -> bool {
    env::var("XDG_CURRENT_DESKTOP")
        .map(|desktop| desktop.to_ascii_lowercase().contains(value))
        .unwrap_or(false)
}

fn optional_json_string(value: Option<&str>) -> String {
    value
        .map(|value| format!("\"{}\"", escape_json(value)))
        .unwrap_or_else(|| "null".to_string())
}

fn escape_json(value: &str) -> String {
    let mut output = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            ch if ch.is_control() => output.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => output.push(ch),
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_machine_readable_report() {
        let report = DoctorReport {
            checks: vec![DoctorCheck {
                category: "Renderer".to_string(),
                name: "gpu".to_string(),
                status: DoctorStatus::Ok,
                message: "wgpu".to_string(),
                detail: None,
                fix_hint: None,
            }],
        };

        assert_eq!(
            report.to_json(),
            "{\"ok\":true,\"checks\":[{\"category\":\"Renderer\",\"name\":\"gpu\",\"status\":\"ok\",\"message\":\"wgpu\",\"detail\":null,\"fix_hint\":null}]}"
        );
    }

    #[test]
    fn reports_errors_as_not_ok() {
        let report = DoctorReport {
            checks: vec![DoctorCheck {
                category: "Workspace".to_string(),
                name: "Stuk.toml".to_string(),
                status: DoctorStatus::Error,
                message: "invalid".to_string(),
                detail: None,
                fix_hint: None,
            }],
        };

        assert!(report.has_errors());
        assert!(report.to_json().starts_with("{\"ok\":false"));
    }
}
