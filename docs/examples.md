# Examples

The workspace examples cover the current app-facing surface:

| Example | Focus |
| --- | --- |
| `hello` | Window, layout, titlebar, resizable pane, overlay badge, settings, actions, sidebar |
| `counter` | Explicit action handling, `actions!` descriptors, shortcuts, wrapping flex actions |
| `notes` | Sidebar app structure, tree navigation, table metadata, virtual list rows, styled surface, SVG media command, Staccato session metadata, tabs, badges, avatar, search field, text area, async resource and mutation views |
| `settings` | Declarative settings schema, navigation view, form layout, dropdown, grid layout, segmented control, slider, checkbox, progress bar, color well |
| `split-view` | Resizable split layout, tree sidebar, popover, spinner, scroll view |
| `shell-panel` | Staccato-style material preference, dialog, menu, toast, command palette, shell-facing actions |

Run any example with Cargo:

```sh
cargo run -p notes
```

Validate an example manifest:

```sh
cargo run -p stuk-cli -- validate examples/notes/Stuk.toml
```
