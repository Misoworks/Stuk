use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

use stuk_devtools::{BundlePlan, BundleTarget};
use stuk_manifest::{parse_file, validate_with_base_dir};

use crate::build::BuildTarget;

mod metadata;
mod stage;

use metadata::{escape_json, sanitize_path_name};
use stage::{binary_path, stage_bundle};

#[derive(Debug)]
pub struct BundleOptions {
    pub target: String,
    pub json: bool,
    pub manifest: PathBuf,
    pub out: PathBuf,
    pub release: bool,
    pub no_build: bool,
}

pub fn run_bundle(options: BundleOptions) -> Result<ExitCode, String> {
    let Some(target) = BundleTarget::parse(&options.target) else {
        return Err("unknown bundle target; use staccato, flatpak, appimage, windows, macos, android, ios, or web".to_string());
    };
    let manifest_path = absolute_path(&options.manifest)?;
    let manifest = parse_file(&manifest_path).map_err(|error| error.to_string())?;
    let source_dir = manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let diagnostics = validate_with_base_dir(&manifest, &source_dir);
    let plan = BundlePlan::from_manifest(&manifest, &diagnostics, target, &manifest_path);

    if !plan.ok {
        if options.json {
            println!("{}", plan.to_json());
        } else {
            print!("{}", plan.to_text());
        }
        return Ok(ExitCode::from(1));
    }

    let executable_name = cargo_package_name(&source_dir.join("Cargo.toml"))
        .unwrap_or_else(|| plan.binary_name.clone());
    if !options.no_build {
        build_bundle_target(&source_dir, target, options.release)?;
    }
    let binary = binary_path(&source_dir, target, options.release, &executable_name);
    if target != BundleTarget::Web && !binary.is_file() {
        return Err(format!(
            "built binary was not found at {}; pass --no-build only when the binary already exists",
            binary.display()
        ));
    }

    let bundle_dir = options
        .out
        .join(target.as_str())
        .join(sanitize_path_name(&manifest.app.id));
    if bundle_dir.exists() {
        fs::remove_dir_all(&bundle_dir).map_err(|error| error.to_string())?;
    }
    fs::create_dir_all(&bundle_dir).map_err(|error| error.to_string())?;
    stage_bundle(
        target,
        &manifest,
        &plan,
        &manifest_path,
        &source_dir,
        &binary,
        &executable_name,
        &bundle_dir,
    )?;

    if options.json {
        println!(
            "{{\"ok\":true,\"target\":\"{}\",\"path\":\"{}\",\"plan\":{}}}",
            target.as_str(),
            escape_json(&bundle_dir.display().to_string()),
            plan.to_json()
        );
    } else {
        println!("Bundled {} to {}", manifest.app.name, bundle_dir.display());
    }
    Ok(ExitCode::SUCCESS)
}

fn build_bundle_target(
    source_dir: &Path,
    target: BundleTarget,
    release: bool,
) -> Result<(), String> {
    let Some(build_target) = build_target_for_bundle(target) else {
        return Ok(());
    };
    let cargo_manifest = source_dir.join("Cargo.toml");
    if !cargo_manifest.is_file() {
        return Err(format!(
            "missing Cargo.toml at {}",
            cargo_manifest.display()
        ));
    }

    let mut command = Command::new("cargo");
    command
        .arg("build")
        .arg("--manifest-path")
        .arg(&cargo_manifest);
    if release {
        command.arg("--release");
    }
    if let Some(rust_target) = build_target.rust_target() {
        command.arg("--target").arg(rust_target);
    }
    command.env("STUK_BUILD_TARGET", build_target.as_str());
    let status = command
        .status()
        .map_err(|error| format!("failed to run cargo build: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("cargo build failed for {}", build_target.as_str()))
    }
}

fn build_target_for_bundle(target: BundleTarget) -> Option<BuildTarget> {
    match target {
        BundleTarget::Staccato | BundleTarget::Flatpak | BundleTarget::AppImage => {
            BuildTarget::parse("linux")
        }
        BundleTarget::Windows => BuildTarget::parse("windows"),
        BundleTarget::Macos => BuildTarget::parse("macos"),
        BundleTarget::Android => BuildTarget::parse("android"),
        BundleTarget::Ios => BuildTarget::parse("ios"),
        BundleTarget::Web => BuildTarget::parse("web"),
    }
}

fn absolute_path(path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    Ok(std::env::current_dir()
        .map_err(|error| error.to_string())?
        .join(path))
}

fn cargo_package_name(path: &Path) -> Option<String> {
    let source = fs::read_to_string(path).ok()?;
    let mut in_package = false;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
        }
        if in_package && trimmed.starts_with("name") {
            return trimmed.split_once('=').map(|(_, value)| {
                value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .replace('-', "_")
            });
        }
    }
    None
}
