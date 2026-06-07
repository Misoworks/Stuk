# Stuk Implementation Guide

Stuk is the native Rust app framework/runtime. Use it for GPU-rendered native UI, native widgets,
actions, settings, platform services, accessibility, and packaging metadata. Webviews are not owned
by Stuk; use Fenestra through `stuk-fenestra` when an app needs an HTML/CSS/JS surface.

## Crate Map

| Crate | Owns |
| --- | --- |
| `stuk` | Public app-facing facade and prelude |
| `stuk-core` | App lifecycle, view tree, component model, reconciliation, focus, async state |
| `stuk-widgets` | Built-in controls, navigation, feedback, text widgets, app shells |
| `stuk-layout` | Flex, grid, responsive layout primitives |
| `stuk-render` | Display list, GPU renderer, damage tracking, shape/text/media commands |
| `stuk-style` | Tokens, colors, typography, spacing, radius, materials, theme data |
| `stuk-text` | Text editing, caret, selection, undo/redo, clipboard, IME primitives |
| `stuk-actions` | Actions, shortcuts, command registration |
| `stuk-settings` | Settings schema and storage |
| `stuk-manifest` | App metadata, windows, permissions, actions, settings validation |
| `stuk-platform-*` | Platform backends and native OS integrations |
| `stuk-devtools` | Inspector snapshots, manifest diagnostics, previews, performance samples |
| `stuk-cli` | Project creation, dev/build/check, validation, inspect, source install, bundle |

## App Shape

Generated apps should keep UI, state, actions, and settings separate:

```txt
src/
  main.rs
  app.rs
  state.rs
  actions.rs
  settings.rs
  views/
  components/
Stuk.toml
Cargo.toml
```

Use the public facade:

```rust
use stuk::prelude::*;
```

Minimal app entry:

```rust
fn main() -> stuk::Result {
    App::new()
        .id("com.example.notes")
        .name("Notes")
        .window(MainWindow::default())
        .run()
}

#[derive(Default)]
struct MainWindow;

impl View for MainWindow {
    fn view(&self, _cx: &mut Cx) -> Element {
        Window::new()
            .title("Notes")
            .size(980, 680)
            .content(Text::new("Notes"))
    }
}
```

Keep persistent app work in Rust state/services. Views should describe UI and dispatch actions; they
should not own long-running jobs directly.

## Manifest

`Stuk.toml` is the source of truth for app metadata, windows, permissions, actions, shortcuts,
settings, icons, and packaging metadata. Validate it during development:

```sh
stuk validate
stuk inspect --json Stuk.toml
```

Typical manifest data:

```toml
[app]
id = "com.example.notes"
name = "Notes"
version = "0.1.0"

[[windows]]
id = "main"
title = "Notes"
width = 980
height = 680
material = "surface"
chrome = "integrated"
```

Use manifest permissions for native capabilities. Do not hide privileged behavior in arbitrary UI
code.

## Native UI

Prefer built-in primitives before custom drawing. They carry layout, focus, accessibility, input,
styling, and display-list behavior.

Common app structure:

```rust
let sidebar = Sidebar::new()
    .item(NavigationItem::new("Inbox", "nav.inbox").selected(true))
    .item(NavigationItem::new("Archive", "nav.archive"));

let editor = TextEditorLite::new(state.current_note_body())
    .action("notes.body.changed")
    .fill_width()
    .fill_height();

SidebarLayout::new(sidebar, editor)
    .initial_ratio(0.28)
    .resizable(true)
```

Controls should dispatch actions:

```rust
let toolbar = HStack::new()
    .spacing(8.0)
    .child(Button::primary("New").action("notes.new"))
    .child(Button::secondary("Save").action("notes.save"))
    .child(IconButton::new("trash").action("notes.delete"));
```

Use text widgets instead of custom input logic:

```rust
let title = TextField::new(state.title())
    .label("Title")
    .action("notes.title.changed")
    .fill_width();

let body = TextArea::new(state.body())
    .label("Body")
    .action("notes.body.changed")
    .fill_width()
    .fill_height();
```

## Layout

Use stack layout for ordinary rows/columns, `Flex` for wrapping or grow behavior, `Grid` for stable
form/table-like structure, and `Frame` for constraints:

```rust
let actions = Flex::row()
    .gap(8.0)
    .wrap(FlexWrap::Wrap)
    .child(Button::primary("Run").action("task.run"))
    .child(Button::secondary("Stop").action("task.stop"));

let settings = Grid::new(
    vec![GridTrack::fixed(180.0), GridTrack::fraction(1.0)],
    vec![GridTrack::fit(), GridTrack::fit()],
)
.gap(12.0)
.cell(0, 0, Label::new("Density"))
.cell(1, 0, SegmentedControl::new("density").option("compact").option("regular"));
```

Responsive behavior belongs in layout primitives and view composition, not in per-control hacks.
Prefer `fill_width`, `fill_height`, min/max constraints, stable row heights, virtual lists, and
split panes before custom measurement.

## Actions And Shortcuts

Actions are data. Declare them once and reuse across buttons, menus, command palettes, and shortcuts:

```rust
fn actions(&self, _cx: &mut Cx) -> Vec<ActionDescriptor> {
    actions! {
        new_note {
            id: "notes.new",
            label: "New Note",
            shortcut: "Ctrl+N",
            category: "Notes",
        }
        save_note {
            id: "notes.save",
            label: "Save",
            shortcut: "Ctrl+S",
            category: "Notes",
        }
    }
}
```

Route action handling to app state/services. Keep UI widgets declarative.

## Settings

Use `stuk-settings` for typed settings schemas and stores. Generated settings pages can render from
the schema:

```rust
NavigationView::new(
    "Settings",
    ScrollView::new(SettingsPage::from_schema(cx.settings_schema().clone())),
)
```

Settings should be validated by the manifest and exposed through a single app state boundary.

## Async Work

Use `ResourceView` and `MutationView` for UI states around async work. Durable or long-running work
should live in Rust services, not in a view function:

```rust
ResourceView::new(state.notes_resource())
    .loading(Spinner::new())
    .empty(EmptyState::new("No notes"))
    .error(|error| ErrorView::new(error.to_string()))
    .ready(|notes| VirtualList::from_items(notes))
```

For webview apps, long-running Rust work should be exposed through Fenestra bridge commands and
activity leases rather than relying on web page timers.

## Materials And Windows

Stuk owns native window/material policy. Use semantic materials and let platform backends resolve
them:

```rust
Window::new()
    .title("Notes")
    .glass()
    .rounded_window_region(14)
    .content_opaque_region(260, 38)
    .content(MainContent::new())
```

Linux uses Wayland background-effect support where available. Windows and macOS backends should map
the same semantic API to Acrylic/Mica or Vibrancy where supported. Apps should gate transparency or
fall back to opaque content when materials are unavailable.

## Fenestra Integration

Use `stuk-fenestra` when a Stuk app needs a webview window or hybrid surface:

```rust
use stuk::prelude::*;
use stuk_fenestra::WebViewWindow;

fn main() -> stuk::Result {
    App::new()
        .id("com.example.hybrid")
        .window(
            WebViewWindow::new()
                .entry("ui/dist/index.html")
                .vite_dev_server(5173)
                .fenestra_chrome()
                .glass(),
        )
        .run()
}
```

Fenestra owns CEF runtime resolution, webview hosting, JS bridge transport, activity leases, and web
runtime packaging. Stuk owns app lifecycle, native windows, actions, settings, permissions,
materials, and packaging metadata.

Hosted web apps use the same adapter. `url(...)` is the production entry, `dev_url(...)` overrides
it during development, and `allowed_origin(...)` declares extra document origins that may use the
bridge:

```rust
WebViewWindow::new()
    .url("https://raday.lantharos.com")
    .dev_url("http://localhost:5173")
    .allowed_origin("https://preview.raday.lantharos.com")
    .fenestra_chrome()
```

## CLI

Create, validate, run, inspect, and bundle:

```sh
stuk new notes --template sidebar
stuk validate
stuk dev
stuk build --release
stuk inspect --json Stuk.toml
stuk bundle --target linux Stuk.toml
stuk bundle --target macos --release --out dist Stuk.toml
stuk install .
stuk update --all
```

`stuk bundle` stages app metadata, binaries, manifests, icons, and platform launcher files. Use
Fenestra bundle commands for standalone webview-only apps; use Stuk bundle commands when Stuk owns
the app manifest and native lifecycle.

## Implementation Rules

- Keep public app API in `crates/stuk`.
- Keep native widget behavior in `crates/stuk-widgets`.
- Keep text editing behavior in `crates/stuk-text`.
- Keep layout primitives in `crates/stuk-layout`.
- Keep renderer/display-list work in `crates/stuk-render`.
- Keep platform-specific services in `crates/stuk-platform-*`.
- Keep Fenestra webview/runtime code out of Stuk core crates.
- Prefer built-in widgets and semantic materials over app-local custom controls.
- Add new abstractions only when they remove real app-level boilerplate or enforce consistency.
