use stuk_devtools::BundlePlan;
use stuk_manifest::Manifest;

pub(super) fn bundle_metadata(plan: &BundlePlan) -> String {
    format!(
        "target = \"{}\"\napp_id = \"{}\"\napp_name = \"{}\"\nversion = \"{}\"\nbinary = \"{}\"\n",
        plan.target.as_str(),
        quote_toml(&plan.app.id),
        quote_toml(&plan.app.name),
        quote_toml(&plan.app.version),
        quote_toml(&plan.binary_name)
    )
}

pub(super) fn webview_metadata(manifest: &Manifest) -> String {
    format!(
        "engine = \"{}\"\nruntime = \"{}\"\nentry = \"{}\"\nallow_user_install = {}\nallow_bundled = {}\n",
        quote_toml(manifest.webview.engine.as_deref().unwrap_or("cef")),
        quote_toml(
            manifest
                .webview
                .runtime
                .as_deref()
                .unwrap_or("shared-preferred")
        ),
        quote_toml(manifest.webview.entry.as_deref().unwrap_or_default()),
        manifest.webview.allow_user_install.unwrap_or(true),
        manifest.webview.allow_bundled.unwrap_or(false)
    )
}

pub(super) fn mobile_metadata(manifest: &Manifest, plan: &BundlePlan) -> String {
    format!(
        "target = \"{}\"\napp_id = \"{}\"\nname = \"{}\"\nversion = \"{}\"\n",
        plan.target.as_str(),
        quote_toml(&manifest.app.id),
        quote_toml(&manifest.app.name),
        quote_toml(&manifest.app.version)
    )
}

pub(super) fn info_plist(manifest: &Manifest, executable_name: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>{}</string>
<key>CFBundleName</key><string>{}</string>
<key>CFBundleDisplayName</key><string>{}</string>
<key>CFBundleExecutable</key><string>{}</string>
<key>CFBundleVersion</key><string>{}</string>
<key>CFBundleShortVersionString</key><string>{}</string>
<key>LSMinimumSystemVersion</key><string>12.0</string>
</dict></plist>
"#,
        escape_xml(&manifest.app.id),
        escape_xml(&manifest.app.name),
        escape_xml(&manifest.app.name),
        escape_xml(executable_name),
        escape_xml(&manifest.app.version),
        escape_xml(&manifest.app.version)
    )
}

pub(super) fn windows_manifest(manifest: &Manifest) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity version="{}.0" processorArchitecture="*" name="{}" type="win32"/>
  <description>{}</description>
  <dependency><dependentAssembly><assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0" processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*"/></dependentAssembly></dependency>
</assembly>
"#,
        escape_xml(&manifest.app.version),
        escape_xml(&manifest.app.id),
        escape_xml(&manifest.app.name)
    )
}

pub(super) fn desktop_entry(manifest: &Manifest, executable_name: &str) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName={}\nExec={}\nTerminal=false\nCategories=Utility;\n",
        manifest.app.name, executable_name
    )
}

pub(super) fn flatpak_manifest(manifest: &Manifest, executable_name: &str) -> String {
    format!(
        "{{\"app-id\":\"{}\",\"runtime\":\"org.freedesktop.Platform\",\"runtime-version\":\"24.08\",\"sdk\":\"org.freedesktop.Sdk\",\"command\":\"{}\",\"modules\":[]}}\n",
        escape_json(&manifest.app.id),
        escape_json(executable_name)
    )
}

pub(super) fn app_run(executable_name: &str) -> String {
    format!(
        "#!/bin/sh\nHERE=\"$(dirname \"$(readlink -f \"$0\")\")\"\nexec \"$HERE/usr/bin/{executable_name}\" \"$@\"\n"
    )
}

pub(super) fn sanitize_path_name(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

fn quote_toml(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

pub(super) fn escape_json(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
