use std::{
    collections::BTreeMap,
    env, fs, io,
    path::{Path, PathBuf},
    process::{Child, Command, ExitCode},
    thread,
    time::{Duration, SystemTime},
};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use crate::templates::{
    ProjectContext, Template, actions_rs, agents_md, app_rs, cargo_toml, main_rs, main_window_rs,
    platforms_desktop_rs, platforms_mobile_rs, platforms_mod_rs, platforms_web_rs, settings_rs,
    state_rs, stuk_toml,
};

pub struct CreateProjectOptions {
    pub name: String,
    pub template: String,
}

pub struct DevOptions {
    pub once: bool,
    pub poll_ms: u64,
}

pub fn create_project(options: CreateProjectOptions) -> Result<(), String> {
    let template = Template::parse(&options.template)?;
    let project_dir = PathBuf::from(&options.name);
    let app_name = app_name_from_path(&project_dir)?;
    let package_name = package_name(&app_name)?;
    let app_id = format!("dev.local.{}", package_name.replace('_', "-"));

    if project_dir.exists()
        && project_dir
            .read_dir()
            .map_err(read_dir_error(&project_dir))?
            .next()
            .is_some()
    {
        return Err(format!(
            "{} already exists and is not empty",
            project_dir.display()
        ));
    }

    fs::create_dir_all(project_dir.join("src/views")).map_err(write_error(&project_dir))?;
    fs::create_dir_all(project_dir.join("src/components")).map_err(write_error(&project_dir))?;
    fs::create_dir_all(project_dir.join("src/domain")).map_err(write_error(&project_dir))?;
    fs::create_dir_all(project_dir.join("src/platforms")).map_err(write_error(&project_dir))?;
    fs::create_dir_all(project_dir.join("src/services")).map_err(write_error(&project_dir))?;
    fs::create_dir_all(project_dir.join("assets")).map_err(write_error(&project_dir))?;

    let context = ProjectContext {
        app_name,
        package_name,
        app_id,
        stuk_dependency: stuk_dependency(&project_dir),
    };

    write_file(&project_dir.join("Cargo.toml"), &cargo_toml(&context))?;
    write_file(&project_dir.join("Stuk.toml"), &stuk_toml(&context))?;
    write_file(&project_dir.join("AGENTS.md"), &agents_md())?;
    write_file(&project_dir.join("src/main.rs"), &main_rs(&context))?;
    write_file(&project_dir.join("src/app.rs"), &app_rs())?;
    write_file(&project_dir.join("src/settings.rs"), &settings_rs())?;
    write_file(&project_dir.join("src/state.rs"), &state_rs())?;
    write_file(&project_dir.join("src/actions.rs"), &actions_rs())?;
    write_file(&project_dir.join("src/domain/mod.rs"), "")?;
    write_file(
        &project_dir.join("src/platforms/mod.rs"),
        &platforms_mod_rs(),
    )?;
    write_file(
        &project_dir.join("src/platforms/desktop.rs"),
        &platforms_desktop_rs(),
    )?;
    write_file(
        &project_dir.join("src/platforms/mobile.rs"),
        &platforms_mobile_rs(),
    )?;
    write_file(
        &project_dir.join("src/platforms/web.rs"),
        &platforms_web_rs(),
    )?;
    write_file(&project_dir.join("src/services/mod.rs"), "")?;
    write_file(
        &project_dir.join("src/views/mod.rs"),
        "pub mod main_window;\n",
    )?;
    write_file(
        &project_dir.join("src/views/main_window.rs"),
        &main_window_rs(template),
    )?;
    write_file(&project_dir.join("src/components/mod.rs"), "")?;

    println!(
        "Created {} with the {} template",
        project_dir.display(),
        template.name()
    );
    println!("Run it with:");
    println!("  cd {}", project_dir.display());
    println!("  stuk dev");
    Ok(())
}

pub fn run_dev(options: DevOptions) -> Result<ExitCode, String> {
    if options.once {
        return run_cargo_command("dev", &["run"]);
    }

    validate_local_manifest_for_cargo("dev")?;

    let poll_interval = Duration::from_millis(options.poll_ms.max(100));
    let mut snapshot = watch_snapshot(Path::new("."))?;
    let mut child = Some(spawn_cargo_run()?);

    println!(
        "Stuk dev is watching app files every {} ms. Press Ctrl+C to stop.",
        poll_interval.as_millis()
    );

    loop {
        thread::sleep(poll_interval);

        let next_snapshot = watch_snapshot(Path::new("."))?;
        if next_snapshot != snapshot {
            snapshot = next_snapshot;
            println!("Change detected; restarting app.");
            if let Some(child) = &mut child {
                stop_child(child)?;
            }
            child = Some(spawn_cargo_run()?);
            continue;
        }

        if let Some(running) = &mut child {
            if let Some(status) = running
                .try_wait()
                .map_err(|error| format!("failed to inspect dev app process: {error}"))?
            {
                println!("App exited with {status}; waiting for changes.");
                child = None;
            }
        }
    }
}

pub fn run_cargo_command(label: &str, args: &[&str]) -> Result<ExitCode, String> {
    validate_local_manifest_for_cargo(label)?;

    let status = Command::new("cargo")
        .args(args)
        .status()
        .map_err(|error| format!("failed to run cargo {label}: {error}"))?;
    Ok(exit_code_from_status(status.code()))
}

pub(crate) fn validate_local_manifest_for_cargo(label: &str) -> Result<(), String> {
    if !matches!(label, "dev" | "run" | "check" | "build") || !Path::new("Stuk.toml").is_file() {
        return Ok(());
    }

    let manifest = stuk_manifest::parse_file("Stuk.toml").map_err(|error| error.to_string())?;
    let diagnostics = stuk_manifest::validate_with_base_dir(&manifest, ".");
    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.level == stuk_manifest::DiagnosticLevel::Error)
    {
        return Err("Stuk.toml has validation errors; run `stuk validate` for details".to_string());
    }
    Ok(())
}

fn write_file(path: &Path, contents: &str) -> Result<(), String> {
    fs::write(path, contents)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn spawn_cargo_run() -> Result<Child, String> {
    let mut command = Command::new("cargo");
    command.arg("run");

    #[cfg(unix)]
    command.process_group(0);

    command
        .spawn()
        .map_err(|error| format!("failed to start cargo run: {error}"))
}

fn stop_child(child: &mut Child) -> Result<(), String> {
    match child.try_wait() {
        Ok(Some(_)) => return Ok(()),
        Ok(None) => {}
        Err(error) => return Err(format!("failed to inspect dev app process: {error}")),
    }

    #[cfg(unix)]
    if stop_process_group(child.id()).is_ok() {
        return child
            .wait()
            .map(|_| ())
            .map_err(|error| format!("failed to wait for dev app process: {error}"));
    }

    match child.kill() {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::InvalidInput => {}
        Err(error) => return Err(format!("failed to stop dev app process: {error}")),
    }
    child
        .wait()
        .map(|_| ())
        .map_err(|error| format!("failed to wait for dev app process: {error}"))
}

#[cfg(unix)]
fn stop_process_group(pid: u32) -> Result<(), String> {
    let status = Command::new("kill")
        .args(["-TERM", "--", &format!("-{pid}")])
        .status()
        .map_err(|error| format!("failed to signal dev app process group: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("kill exited with {status}"))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WatchedFile {
    modified: Option<SystemTime>,
    len: u64,
}

fn watch_snapshot(root: &Path) -> Result<BTreeMap<PathBuf, WatchedFile>, String> {
    let mut files = BTreeMap::new();
    collect_watched_files(root, root, &mut files)?;
    Ok(files)
}

fn collect_watched_files(
    root: &Path,
    dir: &Path,
    files: &mut BTreeMap<PathBuf, WatchedFile>,
) -> Result<(), String> {
    let entries =
        fs::read_dir(dir).map_err(|error| format!("failed to watch {}: {error}", dir.display()))?;

    for entry in entries {
        let entry = entry.map_err(|error| format!("failed to watch {}: {error}", dir.display()))?;
        let path = entry.path();
        let metadata = entry
            .metadata()
            .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;

        if metadata.is_dir() {
            if !should_skip_watch_dir(&path) {
                collect_watched_files(root, &path, files)?;
            }
            continue;
        }

        if metadata.is_file() && is_watched_file(&path) {
            let relative = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
            files.insert(
                relative,
                WatchedFile {
                    modified: metadata.modified().ok(),
                    len: metadata.len(),
                },
            );
        }
    }
    Ok(())
}

fn should_skip_watch_dir(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("target" | ".git" | ".stuk")
    )
}

fn is_watched_file(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("Cargo.toml" | "Stuk.toml" | "AGENTS.md")
    ) || matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("rs" | "toml" | "wgsl" | "svg" | "png" | "jpg" | "jpeg" | "webp")
    )
}

fn app_name_from_path(path: &Path) -> Result<String, String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(title_case)
        .filter(|name| !name.is_empty())
        .ok_or_else(|| "project name must not be empty".to_string())
}

fn package_name(app_name: &str) -> Result<String, String> {
    let mut package = String::new();
    for ch in app_name.chars() {
        if ch.is_ascii_alphanumeric() {
            package.push(ch.to_ascii_lowercase());
        } else if !package.ends_with('_') {
            package.push('_');
        }
    }
    let package = package.trim_matches('_').to_string();
    if package.is_empty() {
        Err("project name must contain at least one ASCII letter or number".to_string())
    } else {
        Ok(package)
    }
}

fn title_case(value: &str) -> String {
    value
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn stuk_dependency(project_dir: &Path) -> String {
    let Ok(cwd) = env::current_dir() else {
        return "stuk = \"0.1.0\"".to_string();
    };
    let local_stuk = cwd.join("crates/stuk");
    if local_stuk.join("Cargo.toml").is_file() {
        let path = relative_path(project_dir, &local_stuk);
        format!("stuk = {{ path = \"{}\" }}", path.display())
    } else {
        "stuk = \"0.1.0\"".to_string()
    }
}

fn relative_path(from: &Path, to: &Path) -> PathBuf {
    let from = absolute_path(from);
    let to = absolute_path(to);
    let from_components = from.components().collect::<Vec<_>>();
    let to_components = to.components().collect::<Vec<_>>();
    let common = from_components
        .iter()
        .zip(&to_components)
        .take_while(|(left, right)| left == right)
        .count();
    let mut relative = PathBuf::new();
    for _ in common..from_components.len() {
        relative.push("..");
    }
    for component in &to_components[common..] {
        relative.push(component.as_os_str());
    }
    if relative.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        relative
    }
}

fn absolute_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

fn read_dir_error(path: &Path) -> impl FnOnce(std::io::Error) -> String + '_ {
    move |error| format!("failed to inspect {}: {error}", path.display())
}

fn write_error(path: &Path) -> impl FnOnce(std::io::Error) -> String + '_ {
    move |error| format!("failed to create {}: {error}", path.display())
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
    fn watch_snapshot_tracks_source_manifest_and_assets() {
        let dir = env::temp_dir().join(format!("stuk-dev-watch-{}", std::process::id()));
        let src = dir.join("src");
        let target = dir.join("target");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&target).unwrap();
        fs::write(dir.join("Stuk.toml"), "[app]\n").unwrap();
        fs::write(src.join("main.rs"), "fn main() {}\n").unwrap();
        fs::write(target.join("ignored.rs"), "ignored\n").unwrap();

        let snapshot = watch_snapshot(&dir).unwrap();
        let _ = fs::remove_dir_all(&dir);

        assert!(snapshot.contains_key(Path::new("Stuk.toml")));
        assert!(snapshot.contains_key(Path::new("src/main.rs")));
        assert!(!snapshot.contains_key(Path::new("target/ignored.rs")));
    }
}
