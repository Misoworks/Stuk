use std::{
    env,
    ffi::{CString, c_char, c_void},
    fs, io,
    path::PathBuf,
};

use stuk_platform::{AutostartEntry, DeepLinkRegistration, NativeMessagingHost};

pub(super) fn write_launch_agent(entry: &AutostartEntry) -> io::Result<()> {
    let path = home_dir()?
        .join("Library/LaunchAgents")
        .join(format!("{}.plist", sanitize_id(&entry.id)));
    if !entry.enabled {
        match fs::remove_file(path) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error),
        }
    }
    write_file(path, &launch_agent_plist(entry))
}

pub(super) fn register_native_messaging_host(host: &NativeMessagingHost) -> io::Result<()> {
    let name = sanitize_native_host_name(&host.id);
    let chrome_manifest = native_messaging_manifest(host, &name, "allowed_origins");
    let app_support = home_dir()?.join("Library/Application Support");
    for browser in [
        "Google/Chrome",
        "Chromium",
        "BraveSoftware/Brave-Browser",
        "Microsoft Edge",
    ] {
        write_file(
            app_support
                .join(browser)
                .join("NativeMessagingHosts")
                .join(format!("{name}.json")),
            &chrome_manifest,
        )?;
    }
    let firefox_manifest = native_messaging_manifest(host, &name, "allowed_extensions");
    write_file(
        app_support
            .join("Mozilla/NativeMessagingHosts")
            .join(format!("{name}.json")),
        &firefox_manifest,
    )
}

pub(super) fn register_deep_links(registration: &DeepLinkRegistration) -> io::Result<()> {
    let handler = cf_string(&registration.id)?;
    let mut result = Ok(());
    for scheme in &registration.schemes {
        let scheme = sanitize_scheme(scheme);
        if scheme.is_empty() {
            continue;
        }
        let scheme = cf_string(&scheme)?;
        let status = unsafe { LSSetDefaultHandlerForURLScheme(scheme.as_ptr(), handler.as_ptr()) };
        if status != 0 {
            result = Err(io::Error::other(format!(
                "LaunchServices rejected URL scheme registration with status {status}"
            )));
        }
    }
    result
}

fn launch_agent_plist(entry: &AutostartEntry) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\">\n<dict>\n  <key>Label</key>\n  <string>{}</string>\n  <key>ProgramArguments</key>\n  <array>\n    <string>/bin/sh</string>\n    <string>-lc</string>\n    <string>{}</string>\n  </array>\n  <key>RunAtLoad</key>\n  <true/>\n</dict>\n</plist>\n",
        xml_value(&entry.id),
        xml_value(&entry.command),
    )
}

fn native_messaging_manifest(host: &NativeMessagingHost, name: &str, allowed_key: &str) -> String {
    let allowed = host
        .allowed_origins
        .iter()
        .map(|origin| format!("\"{}\"", json_value(origin)))
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

fn write_file(path: PathBuf, contents: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)
}

fn home_dir() -> io::Result<PathBuf> {
    env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "HOME is required for macOS desktop integration",
        )
    })
}

fn sanitize_id(value: &str) -> String {
    sanitize_with(value, |ch| {
        ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_')
    })
}

fn sanitize_native_host_name(value: &str) -> String {
    sanitize_with(&value.to_ascii_lowercase(), |ch| {
        ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.')
    })
}

fn sanitize_scheme(value: &str) -> String {
    sanitize_with(&value.to_ascii_lowercase(), |ch| {
        ch.is_ascii_alphanumeric() || matches!(ch, '+' | '.' | '-')
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

fn xml_value(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

struct CfString(*const c_void);

impl CfString {
    fn as_ptr(&self) -> *const c_void {
        self.0
    }
}

impl Drop for CfString {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CFRelease(self.0) };
        }
    }
}

fn cf_string(value: &str) -> io::Result<CfString> {
    let value = CString::new(value)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "string contains NUL"))?;
    let string = unsafe {
        CFStringCreateWithCString(std::ptr::null(), value.as_ptr(), K_CF_STRING_ENCODING_UTF8)
    };
    if string.is_null() {
        Err(io::Error::other("failed to create CoreFoundation string"))
    } else {
        Ok(CfString(string))
    }
}

const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFStringCreateWithCString(
        alloc: *const c_void,
        c_str: *const c_char,
        encoding: u32,
    ) -> *const c_void;
    fn CFRelease(cf: *const c_void);
}

#[link(name = "CoreServices", kind = "framework")]
unsafe extern "C" {
    fn LSSetDefaultHandlerForURLScheme(scheme: *const c_void, handler: *const c_void) -> i32;
}
