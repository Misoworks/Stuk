use std::path::PathBuf;

use stuk::prelude::*;
use stuk_web_runtime::{RuntimeConfig, RuntimeMode, detect_runtime, user_runtime_path};
use stuk_webview::{
    BridgeRegistry, WebViewSecurity, WebViewWindow, WindowChrome, run_installing_window_from_args,
    run_native_host_from_args,
};

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if run_installing_window_from_args(&args) {
        return;
    }
    if run_native_host_from_args(&args) {
        return;
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let runtime = RuntimeConfig {
        mode: RuntimeMode::SharedPreferred,
        allow_user_install: true,
        bundled_dir: Some(manifest_dir.clone()),
        ..RuntimeConfig::default()
    };
    let window = WebViewWindow::new()
        .title("Notes")
        .entry(manifest_dir.join("ui/index.html").display().to_string())
        .chrome(WindowChrome::Stuk)
        .material(Material::Surface)
        .transparent(true)
        .security(WebViewSecurity::default())
        .runtime(runtime.clone());
    let mut bridge = BridgeRegistry::new();
    bridge.register("notes.list");
    bridge.register("notes.create");

    println!("Stuk WebView runtime example");
    println!("entry: {}", window.config.entry.as_deref().unwrap_or(""));
    println!(
        "user runtime dir: {}",
        user_runtime_path(runtime.engine).display()
    );
    println!("detected runtimes: {}", detect_runtime(&runtime).len());
    println!("bridge commands: {}", bridge.commands().join(", "));
    println!("resolving web runtime");

    match window.launch_or_install() {
        Ok(process) => {
            println!("launched webview process {}", process.id());
            let _ = process.wait();
        }
        Err(error) => {
            eprintln!("failed to launch webview: {error}");
            std::process::exit(1);
        }
    }
}
