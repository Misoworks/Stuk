use std::{env, io, path::PathBuf};

use stuk_platform::{AutostartEntry, DeepLinkRegistration, NativeMessagingHost};
use windows_registry::CURRENT_USER;

pub(super) fn write_autostart_entry(entry: &AutostartEntry) -> io::Result<()> {
    let key = CURRENT_USER.create("Software\\Microsoft\\Windows\\CurrentVersion\\Run")?;
    if entry.enabled {
        key.set_string(&entry.name, &entry.command)
            .map_err(io::Error::other)
    } else {
        match key.remove_value(&entry.name) {
            Ok(()) => Ok(()),
            Err(_) => Ok(()),
        }
    }
}

pub(super) fn register_deep_links(registration: &DeepLinkRegistration) -> io::Result<()> {
    let command = env::current_exe()
        .map(|path| format!("\"{}\" \"%1\"", path.display()))
        .unwrap_or_else(|_| "\"%1\"".to_string());
    for scheme in &registration.schemes {
        let scheme = sanitize_scheme(scheme);
        if scheme.is_empty() {
            continue;
        }
        let key = CURRENT_USER.create(format!("Software\\Classes\\{scheme}"))?;
        key.set_string("", format!("URL:{scheme} Protocol"))
            .map_err(io::Error::other)?;
        key.set_string("URL Protocol", "")
            .map_err(io::Error::other)?;
        CURRENT_USER
            .create(format!("Software\\Classes\\{scheme}\\shell\\open\\command"))?
            .set_string("", &command)
            .map_err(io::Error::other)?;
    }
    Ok(())
}

pub(super) fn register_native_messaging_host(host: &NativeMessagingHost) -> io::Result<()> {
    let name = sanitize_native_host_name(&host.id);
    let manifest = native_messaging_manifest(host, &name);
    let base = env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir);
    let path = base
        .join("Stuk")
        .join("NativeMessagingHosts")
        .join(format!("{name}.json"));
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, manifest)?;

    for browser in [
        "Google\\Chrome",
        "Chromium",
        "BraveSoftware\\Brave-Browser",
        "Microsoft\\Edge",
    ] {
        CURRENT_USER
            .create(format!("Software\\{browser}\\NativeMessagingHosts\\{name}"))?
            .set_string("", path.display().to_string())
            .map_err(io::Error::other)?;
    }
    Ok(())
}

fn native_messaging_manifest(host: &NativeMessagingHost, name: &str) -> String {
    let allowed = host
        .allowed_origins
        .iter()
        .map(|origin| format!("\"{}\"", json_value(origin)))
        .collect::<Vec<_>>();
    format!(
        "{{\n  \"name\": \"{}\",\n  \"description\": \"{}\",\n  \"path\": \"{}\",\n  \"type\": \"stdio\",\n  \"allowed_origins\": [{}]\n}}\n",
        json_value(name),
        json_value(&host.name),
        json_value(&host.executable.display().to_string()),
        allowed.join(", ")
    )
}

fn sanitize_scheme(value: &str) -> String {
    sanitize_with(&value.to_ascii_lowercase(), |ch| {
        ch.is_ascii_alphanumeric() || matches!(ch, '+' | '.' | '-')
    })
}

fn sanitize_native_host_name(value: &str) -> String {
    sanitize_with(&value.to_ascii_lowercase(), |ch| {
        ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.')
    })
}

fn sanitize_with(value: &str, valid: impl Fn(char) -> bool) -> String {
    let sanitized = value
        .chars()
        .map(|ch| if valid(ch) { ch } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    if sanitized.is_empty() {
        "app".to_string()
    } else {
        sanitized
    }
}

fn json_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}
