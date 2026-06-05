# Stuk

Stuk is the initial Rust workspace for the native UI framework described in `stuk.md`.

This repository currently implements the early native toolkit path:

- Cargo workspace with the initial Stuk crates.
- `stuk` public facade and prelude.
- Basic `App`, `Window`, `Titlebar`, `Text`, `Label`, `SelectableText`, `TextField`, `TextArea`, `PasswordField`, `SearchField`, `TextEditorLite`, `Image`, `Svg`, `Button`, `IconButton`, `Toggle`, `Checkbox`, `Radio`, `Slider`, `ProgressBar`, `Tabs`, `SegmentedControl`, `Dropdown`, `Menu`, `ContextMenu`, `CommandPalette`, `Toast`, `Badge`, `Avatar`, `Card`, `Surface`, `Tooltip`, `ColorWell`, `ScrollView`, `Sidebar`, `SidebarLayout`, `NavigationView`, `NavigationItem`, `Toolbar`, `SplitView`, `ResizablePane`, `Form`, `FormRow`, `Table`, `Tree`, `List`, `VirtualList`, `Popover`, `Dialog`, `Spinner`, `EmptyState`, `ErrorView`, `ResourceView`, `MutationView`, `VStack`, `HStack`, `ZStack`, `Flex`, `Grid`, `Overlay`, `Frame`, `Spacer`, and `Divider` APIs.
- Minimal stack, flex, grid, overlay, styled surface, margin, fill/fixed/min/max sizing, keyed virtual-list rows, reconciliation, broad display-list commands, and damage tracking abstractions.
- Local signals, component state wrappers, cancellable task handles, async resources, and mutations for app-side state.
- Action descriptors, `actions!` declarations, shortcut parsing, manifest shortcut validation, keyboard shortcut dispatch, and button click dispatch.
- Declarative settings schemas, runtime settings stores, manifest validation, and generated settings page support.
- Theme tokens for colors, spacing, radius, typography, animation, light/dark mode, and density.
- Platform integration traits, generic in-memory platform backend, Staccato/Wayland/Windows/macOS capability crates, material resolution, file dialog types, clipboard payloads, window chrome metadata, Staccato session metadata, and generic capability defaults.
- Text input state primitives for caret movement, word and line navigation, selection, replacement, deletion, undo/redo, clipboard operations, IME composition, and secure display text.
- AccessKit-backed accessibility tree generation, focus traversal, and view-tree accessibility diagnostics for built-in widgets.
- Devtools snapshots for component trees, layout boxes and metrics, accessibility diagnostics, manifests, permission details, platform capabilities, material resolution, performance samples, and previews.
- Native `winit` window backed by `wgpu`.
- Text rendering through `glyphon`.
- CLI commands for project creation, validation, manifest inspection, structured doctor reports, watched dev loops, and Cargo-backed run/build/check flows.
- Manifest validation covers app IDs, semantic versions, icon paths, window sizing, materials, chrome modes, actions, settings, and permissions.

## Run Examples

```sh
cargo run -p hello
cargo run -p counter
cargo run -p notes
cargo run -p settings
cargo run -p split-view
cargo run -p shell-panel
```

Each example opens a native window and renders a Stuk view tree into a GPU display list.

## CLI

```sh
cargo run -p stuk-cli -- --help
cargo run -p stuk-cli -- new notes --template sidebar
cargo run -p stuk-cli -- new prefs --template settings
cargo run -p stuk-cli -- dev --once
cargo run -p stuk-cli -- build --release
cargo run -p stuk-cli -- build --target staccato
cargo run -p stuk-cli -- doctor --json
cargo run -p stuk-cli -- validate examples/notes/Stuk.toml
cargo run -p stuk-cli -- inspect examples/notes/Stuk.toml
cargo run -p stuk-cli -- inspect --json examples/notes/Stuk.toml
cargo run -p stuk-cli -- preview --theme dark examples/notes/Stuk.toml
cargo run -p stuk-cli -- preview --json examples/notes/Stuk.toml
cargo run -p stuk-cli -- bundle --target staccato examples/notes/Stuk.toml
cargo run -p stuk-cli -- bundle --target flatpak --json examples/notes/Stuk.toml
cargo run -p stuk-cli -- bundle --target macos --release --out dist examples/notes/Stuk.toml
cargo run -p stuk-cli -- bundle --target windows --no-build --out dist examples/notes/Stuk.toml
```

`stuk bundle` builds by default, then stages an Electron-builder-style distributable directory under
`dist/<target>/<app-id>/`. The staged output includes the app binary, `Stuk.toml`, bundle metadata,
icons when declared, webview metadata/assets when `[webview].entry` is present, and target-specific
launcher files such as `.app/Contents/Info.plist`, `.desktop`, AppImage `AppRun`, Flatpak JSON, or
Windows app manifest files. Use `--no-build` when CI has already produced the target binary.

The same inspection APIs are available from the `stuk` facade for app-side tooling and previews:

```rust
use stuk::prelude::*;

let previews = preview! {
    DraftPreview => Text::new("Draft")
};

let tree = previews.inspect("DraftPreview");
let accessibility = previews.inspect_accessibility("DraftPreview");
```

Local UI state can use signals while app behavior stays explicit through actions:

```rust
use stuk::prelude::*;

let search = signal(String::new());
search.set("notes".to_string());
```

Async app state can render loading, empty, error, and ready states with `ResourceView` and `MutationView`. See `docs/async.md` for the basic pattern.

After installing the CLI binary, the executable name is `stuk`.

Generated apps include a structured `src/` layout, `Stuk.toml`, and a Cargo project that can be run with:

```sh
stuk dev
```

## Check

```sh
cargo build --workspace
cargo test --workspace
```

## License

Stuk is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
