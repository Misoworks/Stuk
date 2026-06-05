use std::{
    collections::BTreeMap,
    path::Path,
    process::{Command, ExitCode},
};

use crate::project::validate_local_manifest_for_cargo;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildOptions {
    pub release: bool,
    pub target: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildTarget {
    Staccato,
    Linux,
    Windows,
    Macos,
    Android,
    Ios,
    Web,
}

impl BuildTarget {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "staccato" => Some(Self::Staccato),
            "linux" => Some(Self::Linux),
            "windows" => Some(Self::Windows),
            "macos" => Some(Self::Macos),
            "android" => Some(Self::Android),
            "ios" => Some(Self::Ios),
            "web" | "wasm" => Some(Self::Web),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Staccato => "staccato",
            Self::Linux => "linux",
            Self::Windows => "windows",
            Self::Macos => "macos",
            Self::Android => "android",
            Self::Ios => "ios",
            Self::Web => "web",
        }
    }

    pub(crate) fn rust_target(self) -> Option<&'static str> {
        match self {
            Self::Staccato | Self::Linux if cfg!(target_os = "linux") => None,
            Self::Staccato | Self::Linux if cfg!(target_arch = "aarch64") => {
                Some("aarch64-unknown-linux-gnu")
            }
            Self::Staccato | Self::Linux => Some("x86_64-unknown-linux-gnu"),
            Self::Windows => Some("x86_64-pc-windows-msvc"),
            Self::Macos if cfg!(target_arch = "aarch64") => Some("aarch64-apple-darwin"),
            Self::Macos => Some("x86_64-apple-darwin"),
            Self::Android => Some("aarch64-linux-android"),
            Self::Ios => Some("aarch64-apple-ios"),
            Self::Web => Some("wasm32-unknown-unknown"),
        }
    }

    fn manifest_targets(self) -> &'static [&'static str] {
        match self {
            Self::Staccato | Self::Linux => &["desktop", "linux"],
            Self::Windows => &["desktop", "windows"],
            Self::Macos => &["desktop", "macos"],
            Self::Android => &["android"],
            Self::Ios => &["ios"],
            Self::Web => &["web"],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildCommandPlan {
    pub cargo_args: Vec<String>,
    pub stuk_target: Option<BuildTarget>,
}

impl BuildOptions {
    pub fn command_plan(&self) -> Result<BuildCommandPlan, String> {
        let stuk_target = match self.target.as_deref() {
            Some(target) => Some(BuildTarget::parse(target).ok_or_else(|| {
                "unknown build target; use staccato, linux, windows, macos, android, ios, or web"
                    .to_string()
            })?),
            None => None,
        };

        let mut cargo_args = vec!["build".to_string()];
        if self.release {
            cargo_args.push("--release".to_string());
        }
        if let Some(rust_target) = stuk_target.and_then(BuildTarget::rust_target) {
            cargo_args.push("--target".to_string());
            cargo_args.push(rust_target.to_string());
        }

        Ok(BuildCommandPlan {
            cargo_args,
            stuk_target,
        })
    }
}

pub fn run_build(options: BuildOptions) -> Result<ExitCode, String> {
    validate_local_manifest_for_cargo("build")?;
    let plan = options.command_plan()?;
    if let Some(target) = plan.stuk_target {
        validate_manifest_supports_target(target)?;
    }

    if let Some(target) = plan.stuk_target {
        println!("Building for {}", target.as_str());
    }

    let status = Command::new("cargo")
        .args(&plan.cargo_args)
        .env(
            "STUK_BUILD_TARGET",
            plan.stuk_target
                .map(BuildTarget::as_str)
                .unwrap_or("native"),
        )
        .status()
        .map_err(|error| format!("failed to run cargo build: {error}"))?;

    Ok(exit_code_from_status(status.code()))
}

fn validate_manifest_supports_target(target: BuildTarget) -> Result<(), String> {
    if !Path::new("Stuk.toml").is_file() {
        return Ok(());
    }
    let manifest = stuk_manifest::parse_file("Stuk.toml").map_err(|error| error.to_string())?;
    if manifest_targets_allow(&manifest.targets, target) {
        return Ok(());
    }
    Err(format!(
        "Stuk.toml does not enable target `{}`; update [targets] before building it",
        target.as_str()
    ))
}

fn manifest_targets_allow(targets: &BTreeMap<String, bool>, target: BuildTarget) -> bool {
    if targets.is_empty() {
        return matches!(
            target,
            BuildTarget::Staccato | BuildTarget::Linux | BuildTarget::Windows | BuildTarget::Macos
        );
    }

    let requires = target.manifest_targets();
    requires.iter().all(|name| {
        if *name == "desktop" {
            return targets.get(*name).copied().unwrap_or(false);
        }
        let platform_specific_desktop = matches!(*name, "linux" | "windows" | "macos");
        if platform_specific_desktop && !targets.contains_key(*name) {
            return true;
        }
        targets.get(*name).copied().unwrap_or(false)
    })
}

fn exit_code_from_status(code: Option<i32>) -> ExitCode {
    match code.and_then(|code| u8::try_from(code).ok()) {
        Some(code) => ExitCode::from(code),
        None => ExitCode::from(1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_build_targets() {
        assert_eq!(BuildTarget::parse("staccato"), Some(BuildTarget::Staccato));
        assert_eq!(BuildTarget::parse("linux"), Some(BuildTarget::Linux));
        assert_eq!(BuildTarget::parse("windows"), Some(BuildTarget::Windows));
        assert_eq!(BuildTarget::parse("macos"), Some(BuildTarget::Macos));
        assert_eq!(BuildTarget::parse("android"), Some(BuildTarget::Android));
        assert_eq!(BuildTarget::parse("ios"), Some(BuildTarget::Ios));
        assert_eq!(BuildTarget::parse("web"), Some(BuildTarget::Web));
        assert_eq!(BuildTarget::parse("wasm"), Some(BuildTarget::Web));
        assert_eq!(BuildTarget::parse("freebsd"), None);
    }

    #[test]
    fn plans_release_build_args() {
        let plan = BuildOptions {
            release: true,
            target: None,
        }
        .command_plan()
        .unwrap();

        assert_eq!(plan.cargo_args, vec!["build", "--release"]);
        assert_eq!(plan.stuk_target, None);
    }

    #[test]
    fn plans_cross_target_build_args() {
        let plan = BuildOptions {
            release: false,
            target: Some("windows".to_string()),
        }
        .command_plan()
        .unwrap();

        assert_eq!(
            plan.cargo_args,
            vec!["build", "--target", "x86_64-pc-windows-msvc"]
        );
        assert_eq!(plan.stuk_target, Some(BuildTarget::Windows));
    }

    #[test]
    fn build_targets_must_be_enabled_by_manifest_targets() {
        let targets = BTreeMap::from([
            ("desktop".to_string(), true),
            ("linux".to_string(), true),
            ("windows".to_string(), false),
            ("android".to_string(), false),
            ("web".to_string(), true),
        ]);

        assert!(manifest_targets_allow(&targets, BuildTarget::Linux));
        assert!(manifest_targets_allow(&targets, BuildTarget::Web));
        assert!(!manifest_targets_allow(&targets, BuildTarget::Android));
        assert!(!manifest_targets_allow(&targets, BuildTarget::Windows));
    }
}
