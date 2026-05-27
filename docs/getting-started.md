# Getting Started

Run an example app from the workspace:

```sh
cargo run -p hello
```

Create and validate a new app:

```sh
cargo run -p stuk-cli -- new notes --template sidebar
cd notes
stuk validate
stuk dev
```

The generated project uses a `Stuk.toml` manifest for app metadata, windows, permissions, actions, and settings. App code imports the public API through:

```rust
use stuk::prelude::*;
```

Actions can be declared as data and reused by buttons, shortcuts, menus, and command surfaces:

```rust
fn actions(&self, _cx: &mut Cx) -> Vec<ActionDescriptor> {
    actions! {
        new_note {
            id: "notes.new",
            label: "New Note",
            shortcut: "Ctrl+N",
            category: "Notes",
        }
    }
}
```

## Layout Primitives

Stack widgets cover common app structure:

```rust
let actions = HStack::new()
    .spacing(8.0)
    .child(Button::secondary("Cancel"))
    .child(Button::primary("Save").action("notes.save"));
```

Use `Titlebar` when a pane needs a title, optional subtitle, and actions:

```rust
let titlebar = Titlebar::new("Notes")
    .subtitle("Project outline")
    .action(Button::primary("New").action("notes.new"));
```

Use `Flex` for wrapping or growing rows:

```rust
let toolbar = Flex::row()
    .gap(10.0)
    .wrap(FlexWrap::Wrap)
    .child(Button::primary("New").action("notes.new"))
    .child(Button::secondary("Search").action("notes.search"));
```

Use `Grid` when controls need stable rows and columns:

```rust
let settings = Grid::new(
    vec![GridTrack::fraction(1.0), GridTrack::fraction(1.0)],
    vec![GridTrack::fit(), GridTrack::fit()],
)
.gap(12.0)
.cell(0, 0, TextField::new("").label("Name"))
.cell(1, 0, Checkbox::new("Enable sync", true));
```

Use `Overlay` when one element needs to sit on top of another without changing the base layout:

```rust
let inbox = Overlay::new(
    Button::secondary("Inbox").action("mail.inbox"),
    Badge::new("3"),
)
.alignment(OverlayAlignment::TopEnd)
.offset(8.0, -8.0);
```

Use `Surface` for semantic material, rounded corners, borders, and shadows:

```rust
let note = Surface::new(Text::new("Project outline"))
    .material(Material::SurfaceElevated)
    .margin(8.0)
    .padding(16.0)
    .corner_radius(18.0)
    .border(SurfaceBorder::subtle())
    .shadow(SurfaceShadow::soft())
    .min_width(280.0)
    .max_width(520.0);
```

Use `Image` and `Svg` for media that lowers into display-list asset commands:

```rust
let icon = Svg::new("icons/note")
    .label("Note")
    .size(22.0, 22.0)
    .tint(Color::ACCENT);

let preview = Image::new("previews/project-outline")
    .label("Project outline preview")
    .size(320.0, 180.0);
```

Use `NavigationView` for common sidebar apps:

```rust
let settings = NavigationView::new(
    "Settings",
    ScrollView::new(SettingsPage::from_schema(cx.settings_schema().clone())),
)
.item(NavigationItem::new("Appearance", "settings.nav.appearance").selected(true))
.item(NavigationItem::new("Editor", "settings.nav.editor"))
.item(NavigationItem::new("Sync", "settings.nav.sync"))
.resizable(true);
```

Use `SidebarLayout` when the sidebar and content panes are already assembled:

```rust
let workspace = SidebarLayout::new(Sidebar::new(), TextEditorLite::new(""))
    .initial_ratio(0.3)
    .resizable(true);
```

Use `ResizablePane` for a generic two-pane view:

```rust
let panes = ResizablePane::new(List::new(), TextEditorLite::new(""))
    .initial_ratio(0.34)
    .resizable(true);
```

Use `Frame` when a child needs explicit or constrained sizing:

```rust
let editor = Frame::new(TextEditorLite::new(""))
    .margin(8.0)
    .min_width(360.0)
    .max_height(420.0)
    .fill_width();
```

Forms and option menus are ordinary view-tree widgets:

```rust
let preferences = Form::new()
    .row(
        "Density",
        Dropdown::new("Density")
            .selected("regular")
            .option("compact", "Compact")
            .option("regular", "Regular")
            .option("touch", "Touch")
            .action("settings.density.open")
            .action_prefix("settings.density"),
    )
    .form_row(
        FormRow::new(
            "Shortcuts",
            Menu::new().action(ActionDescriptor::new("notes.new", "New Note")),
        )
        .helper("Visible actions can also feed CommandPalette."),
    )
    .row("Accent", ColorWell::new("Accent", Color::ACCENT).action("settings.accent"));
```

Tables and trees cover dense data without custom drawing:

```rust
let outline = Tree::new().item(
    TreeNode::new("Planning")
        .action("notes.section.planning")
        .expanded(true)
        .child(TreeNode::new("Release").action("notes.section.release")),
);

let shortcuts = Table::new()
    .table_column(TableColumn::new("Action").width(180.0))
    .column("Shortcut")
    .row(TableRow::new().text_cell("New Note").text_cell("Ctrl+N"));
```

Lower-level flex and grid layout helpers are available for custom widgets:

```rust
let boxes = flex_layout(
    FlexLayout::row().gap(12.0).wrap(FlexWrap::Wrap),
    Rect::new(0.0, 0.0, 320.0, 180.0),
    &[
        FlexItem::new(Size::new(120.0, 36.0)),
        FlexItem::new(Size::new(120.0, 36.0)).grow(1.0),
    ],
);

let grid = GridLayout::new(
    vec![GridTrack::fixed(220.0), GridTrack::fraction(1.0)],
    vec![GridTrack::fit(), GridTrack::fraction(1.0)],
).gap(12.0);
```

Large repeated views can use keyed virtual rows so rendering, focus, and inspection only walk the visible slice:

```rust
let notes = VirtualList::new()
    .row_height(42.0)
    .viewport_height(240.0)
    .row("project-outline", Button::primary("Project outline").action("notes.select"))
    .row("meeting-notes", Button::ghost("Meeting notes").action("notes.select"));
```

Platform inspection can surface capabilities and material fallbacks for devtools:

```rust
let platform = GenericPlatform::new();
let inspection = inspect_platform(&platform, &Theme::dark(), [Material::Maris, Material::Luca]);
```

Manifest inspection exposes declared permissions for tooling and packaging:

```rust
let permissions = inspect_manifest(&manifest).permissions;
```

Layout inspection exposes computed boxes plus padding, margin, gaps, sizing constraints, and overflow:

```rust
let layout = previews.inspect_layout("ProjectPreview", Rect::new(0.0, 0.0, 980.0, 680.0));
```

Staccato session metadata can be updated from a view context:

```rust
cx.staccato().set_tab_title("Project outline");
cx.staccato().set_preferred_split(SplitHint::Right);
cx.session().set_document_id("note.project-outline");
cx.session().set_restore_payload("{\"note\":\"project-outline\"}");
```

## Useful Commands

```sh
stuk dev
stuk run
stuk build --release
stuk validate --json
stuk inspect
stuk preview
stuk doctor --json
stuk bundle --target staccato
```

## Workspace Examples

```sh
cargo run -p counter
cargo run -p notes
cargo run -p settings
cargo run -p split-view
cargo run -p shell-panel
```

See `docs/async.md` for rendering async resources and mutations in app views.
