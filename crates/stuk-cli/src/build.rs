use std::process::{Command, ExitCode};

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
}

impl BuildTarget {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "staccato" => Some(Self::Staccato),
            "linux" => Some(Self::Linux),
            "windows" => Some(Self::Windows),
            "macos" => Some(Self::Macos),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Staccato => "staccato",
            Self::Linux => "linux",
            Self::Windows => "windows",
            Self::Macos => "macos",
        }
    }

    fn rust_target(self) -> Option<&'static str> {
        match self {
            Self::Staccato | Self::Linux if cfg!(target_os = "linux") => None,
            Self::Staccato | Self::Linux if cfg!(target_arch = "aarch64") => {
                Some("aarch64-unknown-linux-gnu")
            }
            Self::Staccato | Self::Linux => Some("x86_64-unknown-linux-gnu"),
            Self::Windows => Some("x86_64-pc-windows-msvc"),
            Self::Macos if cfg!(target_arch = "aarch64") => Some("aarch64-apple-darwin"),
            Self::Macos => Some("x86_64-apple-darwin"),
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
                "unknown build target; use staccato, linux, windows, or macos".to_string()
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
}
