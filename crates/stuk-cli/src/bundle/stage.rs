use std::{
    fs, io,
    path::{Path, PathBuf},
};

use stuk_devtools::{BundlePlan, BundleTarget};
use stuk_manifest::Manifest;

use crate::build::BuildTarget;

use super::{
    build_target_for_bundle,
    metadata::{
        app_run, bundle_metadata, desktop_entry, flatpak_manifest, info_plist, mobile_metadata,
        sanitize_path_name, webview_metadata, windows_manifest,
    },
};

pub(super) fn stage_bundle(
    target: BundleTarget,
    manifest: &Manifest,
    plan: &BundlePlan,
    manifest_path: &Path,
    source_dir: &Path,
    binary: &Path,
    executable_name: &str,
    bundle_dir: &Path,
) -> Result<(), String> {
    match target {
        BundleTarget::Macos => stage_macos(
            manifest,
            plan,
            manifest_path,
            source_dir,
            binary,
            executable_name,
            bundle_dir,
        ),
        BundleTarget::Windows => stage_windows(
            manifest,
            plan,
            manifest_path,
            source_dir,
            binary,
            executable_name,
            bundle_dir,
        ),
        BundleTarget::Flatpak => stage_flatpak(
            manifest,
            plan,
            manifest_path,
            source_dir,
            binary,
            executable_name,
            bundle_dir,
        ),
        BundleTarget::AppImage => stage_appimage(
            manifest,
            plan,
            manifest_path,
            source_dir,
            binary,
            executable_name,
            bundle_dir,
        ),
        BundleTarget::Staccato => stage_simple_desktop(
            manifest,
            plan,
            manifest_path,
            source_dir,
            binary,
            executable_name,
            bundle_dir,
            "staccato",
        ),
        BundleTarget::Android | BundleTarget::Ios => stage_mobile(
            manifest,
            plan,
            manifest_path,
            source_dir,
            binary,
            executable_name,
            bundle_dir,
        ),
        BundleTarget::Web => stage_web(manifest, plan, manifest_path, source_dir, bundle_dir),
    }
}

fn stage_macos(
    manifest: &Manifest,
    plan: &BundlePlan,
    manifest_path: &Path,
    source_dir: &Path,
    binary: &Path,
    executable_name: &str,
    bundle_dir: &Path,
) -> Result<(), String> {
    let app_dir = bundle_dir.join(format!("{}.app", sanitize_path_name(&manifest.app.name)));
    let contents = app_dir.join("Contents");
    let macos = contents.join("MacOS");
    let resources = contents.join("Resources");
    fs::create_dir_all(&macos).map_err(|error| error.to_string())?;
    fs::create_dir_all(&resources).map_err(|error| error.to_string())?;
    copy_binary(binary, &macos.join(executable_name))?;
    fs::write(
        contents.join("Info.plist"),
        info_plist(manifest, executable_name),
    )
    .map_err(|error| error.to_string())?;
    stage_common_metadata(manifest, plan, manifest_path, source_dir, &resources)?;
    Ok(())
}

fn stage_windows(
    manifest: &Manifest,
    plan: &BundlePlan,
    manifest_path: &Path,
    source_dir: &Path,
    binary: &Path,
    executable_name: &str,
    bundle_dir: &Path,
) -> Result<(), String> {
    let app_dir = bundle_dir.join(sanitize_path_name(&manifest.app.name));
    let resources = app_dir.join("resources");
    fs::create_dir_all(&resources).map_err(|error| error.to_string())?;
    copy_binary(binary, &app_dir.join(format!("{executable_name}.exe")))?;
    fs::write(
        resources.join("windows-app-manifest.xml"),
        windows_manifest(manifest),
    )
    .map_err(|error| error.to_string())?;
    stage_common_metadata(manifest, plan, manifest_path, source_dir, &resources)?;
    Ok(())
}

fn stage_flatpak(
    manifest: &Manifest,
    plan: &BundlePlan,
    manifest_path: &Path,
    source_dir: &Path,
    binary: &Path,
    executable_name: &str,
    bundle_dir: &Path,
) -> Result<(), String> {
    let files = bundle_dir.join("files");
    let bin_dir = files.join("bin");
    let applications = files.join("share/applications");
    fs::create_dir_all(&bin_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(&applications).map_err(|error| error.to_string())?;
    copy_binary(binary, &bin_dir.join(executable_name))?;
    fs::write(
        applications.join(format!("{}.desktop", manifest.app.id)),
        desktop_entry(manifest, executable_name),
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        bundle_dir.join(format!("{}.flatpak.json", manifest.app.id)),
        flatpak_manifest(manifest, executable_name),
    )
    .map_err(|error| error.to_string())?;
    stage_common_metadata(
        manifest,
        plan,
        manifest_path,
        source_dir,
        &files.join("share/stuk"),
    )?;
    Ok(())
}

fn stage_appimage(
    manifest: &Manifest,
    plan: &BundlePlan,
    manifest_path: &Path,
    source_dir: &Path,
    binary: &Path,
    executable_name: &str,
    bundle_dir: &Path,
) -> Result<(), String> {
    let app_dir = bundle_dir.join("AppDir");
    let bin_dir = app_dir.join("usr/bin");
    fs::create_dir_all(&bin_dir).map_err(|error| error.to_string())?;
    copy_binary(binary, &bin_dir.join(executable_name))?;
    fs::write(app_dir.join("AppRun"), app_run(executable_name))
        .map_err(|error| error.to_string())?;
    make_executable(&app_dir.join("AppRun")).map_err(|error| error.to_string())?;
    fs::write(
        app_dir.join(format!("{}.desktop", manifest.app.id)),
        desktop_entry(manifest, executable_name),
    )
    .map_err(|error| error.to_string())?;
    stage_common_metadata(
        manifest,
        plan,
        manifest_path,
        source_dir,
        &app_dir.join("usr/share/stuk"),
    )?;
    Ok(())
}

fn stage_simple_desktop(
    manifest: &Manifest,
    plan: &BundlePlan,
    manifest_path: &Path,
    source_dir: &Path,
    binary: &Path,
    executable_name: &str,
    bundle_dir: &Path,
    kind: &str,
) -> Result<(), String> {
    let bin_dir = bundle_dir.join("bin");
    fs::create_dir_all(&bin_dir).map_err(|error| error.to_string())?;
    copy_binary(binary, &bin_dir.join(executable_name))?;
    stage_common_metadata(
        manifest,
        plan,
        manifest_path,
        source_dir,
        &bundle_dir.join("resources"),
    )?;
    fs::write(
        bundle_dir.join(format!("{kind}.toml")),
        bundle_metadata(plan),
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn stage_mobile(
    manifest: &Manifest,
    plan: &BundlePlan,
    manifest_path: &Path,
    source_dir: &Path,
    binary: &Path,
    executable_name: &str,
    bundle_dir: &Path,
) -> Result<(), String> {
    stage_simple_desktop(
        manifest,
        plan,
        manifest_path,
        source_dir,
        binary,
        executable_name,
        bundle_dir,
        plan.target.as_str(),
    )?;
    fs::write(
        bundle_dir.join("mobile.toml"),
        mobile_metadata(manifest, plan),
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn stage_web(
    manifest: &Manifest,
    plan: &BundlePlan,
    manifest_path: &Path,
    source_dir: &Path,
    bundle_dir: &Path,
) -> Result<(), String> {
    stage_common_metadata(
        manifest,
        plan,
        manifest_path,
        source_dir,
        &bundle_dir.join("resources"),
    )?;
    if let Some(entry) = manifest.webview.entry.as_deref() {
        copy_web_entry(source_dir, entry, &bundle_dir.join("web"))?;
    }
    Ok(())
}

fn stage_common_metadata(
    manifest: &Manifest,
    plan: &BundlePlan,
    manifest_path: &Path,
    source_dir: &Path,
    resources: &Path,
) -> Result<(), String> {
    fs::create_dir_all(resources).map_err(|error| error.to_string())?;
    fs::copy(manifest_path, resources.join("Stuk.toml")).map_err(|error| error.to_string())?;
    fs::write(resources.join("bundle.toml"), bundle_metadata(plan))
        .map_err(|error| error.to_string())?;
    if let Some(icon) = manifest.app.icon.as_deref() {
        let icon_path = source_dir.join(icon);
        if icon_path.is_file() {
            let icon_name = icon_path.file_name().unwrap_or_default().to_os_string();
            fs::copy(&icon_path, resources.join(icon_name)).map_err(|error| error.to_string())?;
        }
    }
    if manifest.webview.entry.is_some() {
        fs::write(resources.join("webview.toml"), webview_metadata(manifest))
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn copy_web_entry(source_dir: &Path, entry: &str, destination: &Path) -> Result<(), String> {
    let entry_path = source_dir.join(entry);
    if entry_path.is_dir() {
        copy_dir_recursive(&entry_path, destination).map_err(|error| error.to_string())
    } else if entry_path.is_file() {
        let parent = entry_path.parent().unwrap_or(source_dir);
        copy_dir_recursive(parent, destination).map_err(|error| error.to_string())
    } else {
        Err(format!(
            "webview entry does not exist: {}",
            entry_path.display()
        ))
    }
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &destination_path)?;
        } else {
            fs::copy(&source_path, &destination_path)?;
        }
    }
    Ok(())
}

fn copy_binary(source: &Path, destination: &Path) -> Result<(), String> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::copy(source, destination).map_err(|error| error.to_string())?;
    make_executable(destination).map_err(|error| error.to_string())
}

pub(super) fn binary_path(
    source_dir: &Path,
    target: BundleTarget,
    release: bool,
    executable_name: &str,
) -> PathBuf {
    let profile = if release { "release" } else { "debug" };
    let file_name = if target == BundleTarget::Windows {
        format!("{executable_name}.exe")
    } else {
        executable_name.to_string()
    };
    let rust_target = build_target_for_bundle(target).and_then(BuildTarget::rust_target);
    let candidates = source_dir
        .ancestors()
        .map(|ancestor| {
            let mut path = ancestor.join("target");
            if let Some(rust_target) = rust_target {
                path = path.join(rust_target);
            }
            path.join(profile).join(&file_name)
        })
        .collect::<Vec<_>>();
    candidates
        .iter()
        .find(|candidate| candidate.is_file())
        .cloned()
        .unwrap_or_else(|| {
            candidates
                .into_iter()
                .next()
                .unwrap_or_else(|| source_dir.join("target").join(profile).join(file_name))
        })
}

fn make_executable(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(())
    }
}
