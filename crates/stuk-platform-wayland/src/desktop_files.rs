use std::{collections::BTreeSet, env, fs, io, path::PathBuf};

use stuk_platform::{AutostartEntry, DeepLinkRegistration, NativeMessagingHost};

pub(super) fn write_autostart_entry(entry: &AutostartEntry) -> io::Result<()> {
    let path = config_home()?
        .join("autostart")
        .join(format!("{}.desktop", sanitize_desktop_id(&entry.id)));
    if !entry.enabled {
        match fs::remove_file(path) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error),
        }
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        desktop_entry(&entry.id, &entry.name, &entry.command, &[]),
    )
}

pub(super) fn register_deep_links(registration: &DeepLinkRegistration) -> io::Result<()> {
    let desktop_id = format!("{}.desktop", sanitize_desktop_id(&registration.id));
    let path = config_home()?.join("mimeapps.list");
    let mut content = fs::read_to_string(&path).unwrap_or_default();
    for scheme in &registration.schemes {
        let scheme = sanitize_scheme(scheme);
        if !scheme.is_empty() {
            content = set_mime_default(&content, &scheme, &desktop_id);
        }
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)
}

pub(super) fn register_native_messaging_host(host: &NativeMessagingHost) -> io::Result<()> {
    let name = sanitize_native_host_name(&host.id);
    let chrome_manifest = native_messaging_manifest(host, &name, "allowed_origins");
    for browser in ["google-chrome", "chromium", "BraveSoftware/Brave-Browser"] {
        let path = config_home()?
            .join(browser)
            .join("NativeMessagingHosts")
            .join(format!("{name}.json"));
        write_file(path, &chrome_manifest)?;
    }

    let firefox_manifest = native_messaging_manifest(host, &name, "allowed_extensions");
    write_file(
        home_dir()?
            .join(".mozilla/native-messaging-hosts")
            .join(format!("{name}.json")),
        &firefox_manifest,
    )
}

pub(super) fn write_file(path: PathBuf, contents: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)
}

pub(super) fn desktop_entry(id: &str, name: &str, command: &str, schemes: &[String]) -> String {
    let mime_types = schemes
        .iter()
        .map(|scheme| format!("x-scheme-handler/{}", sanitize_scheme(scheme)))
        .collect::<Vec<_>>();
    let mime_line = if mime_types.is_empty() {
        String::new()
    } else {
        format!("MimeType={};\n", mime_types.join(";"))
    };
    format!(
        "[Desktop Entry]\nType=Application\nName={}\nExec={}\nIcon={}\nTerminal=false\nCategories=Utility;\n{}",
        desktop_value(name),
        desktop_value(command),
        desktop_value(id),
        mime_line
    )
}

pub(super) fn sanitize_desktop_id(value: &str) -> String {
    sanitize_with(value, |ch| {
        ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_')
    })
}

pub(super) fn data_home() -> io::Result<PathBuf> {
    if let Some(path) = env::var_os("XDG_DATA_HOME") {
        return Ok(PathBuf::from(path));
    }
    Ok(home_dir()?.join(".local/share"))
}

fn set_mime_default(content: &str, scheme: &str, desktop_id: &str) -> String {
    let key = format!("x-scheme-handler/{scheme}");
    let value = format!("{key}={desktop_id}");
    let mut lines = content.lines().map(ToOwned::to_owned).collect::<Vec<_>>();
    let Some(section_start) = lines
        .iter()
        .position(|line| line.trim() == "[Default Applications]")
    else {
        if !lines.is_empty() && lines.last().is_some_and(|line| !line.is_empty()) {
            lines.push(String::new());
        }
        lines.push("[Default Applications]".to_string());
        lines.push(value);
        return finish_lines(lines);
    };

    let section_end = lines
        .iter()
        .enumerate()
        .skip(section_start + 1)
        .find_map(|(index, line)| line.trim().starts_with('[').then_some(index))
        .unwrap_or(lines.len());

    if let Some(index) = lines[section_start + 1..section_end]
        .iter()
        .position(|line| {
            line.split_once('=')
                .is_some_and(|(line_key, _)| line_key == key)
        })
    {
        lines[section_start + 1 + index] = value;
    } else {
        lines.insert(section_end, value);
    }
    finish_lines(lines)
}

fn native_messaging_manifest(host: &NativeMessagingHost, name: &str, allowed_key: &str) -> String {
    let allowed = host
        .allowed_origins
        .iter()
        .map(|origin| format!("\"{}\"", json_value(origin)))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    format!(
        "{{\n  \"name\": \"{}\",\n  \"description\": \"{}\",\n  \"path\": \"{}\",\n  \"type\": \"stdio\",\n  \"{}\": [{}]\n}}\n",
        json_value(name),
        json_value(&host.name),
        json_value(&host.executable.display().to_string()),
        allowed_key,
        allowed.join(", ")
    )
}

fn finish_lines(lines: Vec<String>) -> String {
    let mut output = lines.join("\n");
    output.push('\n');
    output
}

fn desktop_value(value: &str) -> String {
    value.replace(['\n', '\r'], " ")
}

fn json_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
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

fn config_home() -> io::Result<PathBuf> {
    if let Some(path) = env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(path));
    }
    Ok(home_dir()?.join(".config"))
}

fn home_dir() -> io::Result<PathBuf> {
    env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "HOME is required for Linux desktop integration",
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mime_defaults_are_inserted_and_replaced() {
        let content = "[Added Associations]\ntext/plain=editor.desktop;\n";
        let content = set_mime_default(content, "klarkey", "klarkey.desktop");
        assert!(content.contains("[Default Applications]"));
        assert!(content.contains("x-scheme-handler/klarkey=klarkey.desktop"));

        let content = set_mime_default(&content, "klarkey", "other.desktop");
        assert!(content.contains("x-scheme-handler/klarkey=other.desktop"));
        assert_eq!(content.matches("x-scheme-handler/klarkey=").count(), 1);
    }

    #[test]
    fn native_messaging_manifest_escapes_values() {
        let host = NativeMessagingHost::new("com.example.host", "Example Host", "/bin/echo")
            .allow_origin("chrome-extension://abc/");
        let manifest = native_messaging_manifest(&host, "com.example.host", "allowed_origins");
        assert!(manifest.contains("\"name\": \"com.example.host\""));
        assert!(manifest.contains("\"allowed_origins\": [\"chrome-extension://abc/\"]"));
    }
}
