# Stuk Specification

## 0. Purpose

Stuk is a native UI framework and app-integration system for Staccato and Glacier.

Its purpose is to make native desktop development feel as fast and pleasant as modern web development, while keeping native rendering, low overhead, deep OS integration, accessibility, and first-class Staccato/Baton material support.

Stuk is not “GTK but prettier.” It is a Rust-first, GPU-rendered, declarative native framework designed around:

- web-dev velocity
- native runtime performance
- excellent defaults
- predictable layout
- polished widgets
- OS-level integration
- fast devtools
- agent-friendly project structure
- cross-platform fallback
- Staccato-first materials and shell APIs

The core promise:

```txt
Write obvious component trees.
Get polished native UI.
Iterate fast.
Integrate deeply with the OS.
Ship without dragging a browser engine behind every app.
```

Stuk should make native dev great again.

---

## 1. Naming

```txt
Stuk = STaccato UI Kit
Staccato = desktop environment
Baton = compositor
Luca = live glass material
Maris = wallpaper-derived mica-like material
Glacier = eventual OS
```

Package/crate names:

```txt
stuk
stuk-core
stuk-layout
stuk-render
stuk-style
stuk-text
stuk-widgets
stuk-platform
stuk-platform-staccato
stuk-platform-wayland
stuk-platform-windows
stuk-platform-macos
stuk-accessibility
stuk-actions
stuk-settings
stuk-manifest
stuk-devtools
stuk-cli
```

CLI:

```sh
stuk new notes
stuk dev
stuk build
stuk validate
stuk inspect
stuk preview
stuk doctor
stuk bundle
```

---

## 2. Product Thesis

Existing options are bad tradeoffs.

```txt
Web stack:
  fast development, good layout, good text, high overhead, weak native integration

GTK/Qt:
  native-ish, mature, inconsistent, visually tied to old ecosystems, slower iteration

Custom native:
  high performance and polish if done well, but usually huge implementation pain

Stuk:
  native GPU-rendered UI with web-like velocity and deep OS integration
```

Stuk should steal the good parts of web development:

- declarative UI
- component composition
- fast reload
- good layout primitives
- component previews
- inspector/devtools
- simple app structure
- predictable data flow
- strong tooling
- agent-friendly files

Stuk should avoid the bad parts:

- browser engine overhead
- CSS cascade hell
- DOM complexity
- app chrome hacks
- fake blur
- memory bloat
- unpredictable styling
- dependency soup

---

## 3. Goals

Stuk must:

1. Be Rust-first.
2. Use native GPU rendering, not webviews.
3. Own the widget/render/layout pipeline.
4. Support polished default widgets.
5. Support high-quality text rendering and input.
6. Support accessibility from the start.
7. Support Staccato/Baton materials.
8. Support command/action integration.
9. Support declarative settings schemas.
10. Support fast dev loops.
11. Support component previews.
12. Support a real inspector.
13. Be strongly structured for AI coding agents.
14. Be cross-platform where practical.
15. Degrade gracefully when platform features are missing.
16. Be measurable and aggressively performant.

Non-goals for the first versions:

- replacing every GTK/Qt app
- cloning CSS completely
- embedding Chromium/WebKit
- building a browser
- building a full app suite
- shipping Glacier OS
- supporting every widget imaginable immediately

---

## 4. Platform Strategy

Stuk has platform tiers.

### Tier 1: Staccato / Glacier

Best experience:

- Baton compositor material integration
- Luca live blur
- Maris wallpaper-derived material
- shell tabs
- Browser Mode integration
- command palette integration
- workspace session restore
- Staccato settings integration
- app permissions integration
- Staccato-native chrome

### Tier 2: Windows / macOS

Good experience:

- native windows
- wgpu rendering
- system clipboard
- drag and drop
- notifications
- IME
- accessibility
- material mapping where available
- native-ish window chrome options

### Tier 3: Generic Linux

Functional experience:

- Wayland/X11 windows
- wgpu rendering
- clipboard
- drag/drop where possible
- accessibility where possible
- solid/Maris fallback materials
- XDG integration

Apps request semantic features. Platforms resolve them.

Example:

```txt
Material::Luca
  Staccato -> Baton live blur
  Windows -> Acrylic-like fallback if available
  macOS -> vibrancy if available
  generic Linux -> fallback

Material::Maris
  Staccato -> wallpaper-derived material
  Windows -> Mica-like fallback if available
  macOS -> native material if available
  generic Linux -> tinted surface fallback
```

---

## 5. Technical Stack

Primary language:

```txt
Rust
```

Core dependencies / candidates:

```txt
wgpu        GPU renderer
taffy       layout engine
cosmic-text text shaping/editing
AccessKit   accessibility
winit       initial cross-platform window/input
resvg/usvg  SVG rendering
image       raster image loading
serde       config/manifest serialization
toml        manifest/config format
```

Stuk owns:

- view system
- component model
- widget library
- style/token system
- display list
- renderer orchestration
- platform abstraction
- devtools
- CLI
- manifest
- validation
- Staccato integration

Stuk should not reinvent:

- GPU APIs from scratch
- text shaping from scratch
- flex/grid layout from scratch
- accessibility protocols from scratch
- image decoding from scratch

---

## 6. Repository Structure

```txt
stuk/
├── Cargo.toml
├── README.md
├── stuk.md
├── AGENTS.md
├── crates/
│   ├── stuk/
│   ├── stuk-core/
│   ├── stuk-layout/
│   ├── stuk-render/
│   ├── stuk-style/
│   ├── stuk-text/
│   ├── stuk-widgets/
│   ├── stuk-platform/
│   ├── stuk-platform-staccato/
│   ├── stuk-platform-wayland/
│   ├── stuk-platform-windows/
│   ├── stuk-platform-macos/
│   ├── stuk-accessibility/
│   ├── stuk-actions/
│   ├── stuk-settings/
│   ├── stuk-manifest/
│   ├── stuk-devtools/
│   └── stuk-cli/
├── examples/
│   ├── hello/
│   ├── counter/
│   ├── notes/
│   ├── settings/
│   ├── split-view/
│   └── shell-panel/
├── templates/
│   ├── app-basic/
│   ├── app-sidebar/
│   ├── app-document/
│   └── component-library/
├── docs/
├── tests/
└── benches/
```

### Crate responsibilities

`stuk`:
- public facade
- `prelude`
- stable app-facing API

`stuk-core`:
- app runtime
- view tree
- component model
- lifecycle
- state/signals
- event dispatch

`stuk-layout`:
- Stuk layout primitives
- Taffy integration
- constraints
- measurement
- layout boxes

`stuk-render`:
- display list
- wgpu renderer
- text/image/SVG render integration
- clipping
- shadows
- materials
- damage tracking
- performance stats

`stuk-style`:
- tokens
- themes
- colors
- typography
- spacing
- radius
- animation curves
- component variants

`stuk-text`:
- text layout
- glyph cache
- text editing
- selection
- IME
- password fields
- text accessibility

`stuk-widgets`:
- built-in widgets

`stuk-platform`:
- platform abstraction traits

`stuk-platform-staccato`:
- Baton/Staccato integration

`stuk-actions`:
- actions
- shortcuts
- command registration

`stuk-settings`:
- settings schema
- settings storage
- generated settings UI support

`stuk-manifest`:
- `Stuk.toml` parser/validator

`stuk-devtools`:
- inspector
- performance overlay
- component preview

`stuk-cli`:
- command-line tooling

---

## 7. Public API Shape

Apps should usually import:

```rust
use stuk::prelude::*;
```

Minimal example:

```rust
use stuk::prelude::*;

fn main() -> stuk::Result {
    App::new()
        .id("com.example.hello")
        .name("Hello")
        .window(MainWindow)
        .run()
}

struct MainWindow;

impl View for MainWindow {
    fn view(&self, cx: &mut Cx) -> impl IntoView {
        Window::new()
            .title("Hello")
            .material(Material::Maris)
            .content(
                VStack::new()
                    .padding(24)
                    .spacing(12)
                    .child(Text::title("Hello from Stuk"))
                    .child(Button::primary("Click me").action("hello.click"))
            )
    }
}
```

Target macro syntax:

```rust
view! {
    Window(title: "Notes", material: Maris, chrome: Compact) {
        SplitView(direction: Horizontal) {
            Sidebar {
                SidebarItem(icon: "note", label: "All Notes")
                SidebarItem(icon: "tag", label: "Tags")
            }

            VStack(spacing: 0) {
                Toolbar(title: "Notes") {
                    Button.primary("New", action: "notes.new")
                    Button("Search", action: "notes.search")
                }

                TextEditor(bind: state.current_note.body)
            }
        }
    }
}
```

Macro syntax is optional. Builder syntax must always be available.

Public API rules:

- readable type errors
- low boilerplate
- no lifetime nightmares in normal UI code
- strong typing where useful
- stable patterns for agents
- excellent defaults
- advanced escape hatches

---

## 8. View and Component Model

The UI is a declarative view tree.

Pipeline:

```txt
state
→ view tree
→ reconciliation
→ style resolution
→ layout
→ display list
→ rendering
```

Conceptual trait:

```rust
pub trait View {
    fn view(&self, cx: &mut Cx) -> impl IntoView;
}
```

Stateful component shape:

```rust
pub trait Component {
    type State;
    type Action;

    fn init(cx: &mut Cx) -> Self::State;
    fn update(state: &mut Self::State, action: Self::Action, cx: &mut Cx);
    fn view(state: &Self::State, cx: &mut Cx) -> impl IntoView;
}
```

Stuk must support stable element identity for:

- focus preservation
- text input state
- scroll state
- animations
- list diffing
- hot reload

Example:

```rust
VirtualList::new(notes)
    .key(|note| note.id)
    .row(|note| NoteRow::new(note))
```

Conditional rendering must be straightforward:

```rust
if state.loading {
    Spinner::new("Loading")
} else {
    NotesList::new(state.notes.clone())
}
```

---

## 9. State Model

Use a hybrid model.

### Local signals

For local UI state:

```rust
let search = signal(String::new());
let popover_open = signal(false);
```

Use for:

- inputs
- hover/focus state
- temporary local UI
- popovers
- local component state

### App actions

For meaningful app behavior:

```rust
enum Action {
    NewNote,
    DeleteNote(NoteId),
    SearchChanged(String),
    Save,
}
```

Update:

```rust
impl AppState {
    fn update(&mut self, action: Action, cx: &mut Cx) {
        match action {
            Action::NewNote => self.create_note(),
            Action::DeleteNote(id) => self.delete_note(id),
            Action::SearchChanged(q) => self.search = q,
            Action::Save => self.save(cx),
        }
    }
}
```

This keeps code:

- explicit
- testable
- agent-readable
- compatible with command palette integration

Avoid pure magic reactivity where app behavior becomes invisible.

---

## 10. Async Model

Async must be normal, not bolted on.

### Resources

For async-loaded data:

```rust
let notes = resource("notes.load", || async {
    db.load_notes().await
});
```

View states:

```rust
ResourceView::new(notes)
    .loading(|| Spinner::new("Loading notes"))
    .empty(|| EmptyState::new("No notes yet"))
    .error(|err| ErrorView::new(err))
    .data(|notes| NoteList::new(notes))
```

### Mutations

For async user actions:

```rust
let save_note = mutation("notes.save", |note| async move {
    db.save_note(note).await
});
```

Mutation states:

```txt
idle
pending
success
error
```

### Tasks

```rust
cx.spawn(async move {
    do_work().await
});
```

Tasks should be cancellable when the owning window/component dies.

---

## 11. Layout System

Stuk layout should feel like the good parts of web layout without CSS selector/cascade nonsense.

Required primitives:

```txt
VStack
HStack
ZStack
Flex
Grid
SplitView
SidebarLayout
NavigationView
ScrollView
Spacer
Divider
Overlay
```

Common modifiers:

```rust
.padding(...)
.margin(...)
.width(...)
.height(...)
.min_width(...)
.max_width(...)
.min_height(...)
.max_height(...)
.align(...)
.gap(...)
.flex(...)
.opacity(...)
.clip(...)
.background(...)
.border(...)
.corner_radius(...)
.shadow(...)
```

Example:

```rust
HStack::new()
    .spacing(8)
    .padding(12)
    .child(Button::new("Cancel"))
    .child(Spacer::new())
    .child(Button::primary("Save"))
```

`SplitView` is essential:

```rust
SplitView::horizontal()
    .sidebar(Sidebar::new())
    .main(Editor::new())
    .resizable(true)
    .initial_ratio(0.28)
```

Layout inspector must show:

- boxes
- constraints
- computed sizes
- margins/padding
- overflow
- dirty regions

---

## 12. Styling System

Use tokens + variants + local overrides.

Do not clone CSS cascade.

Tokens example:

```toml
[color]
accent = "wallpaper"
text = "#f4f4f4"
text_muted = "#b8b8b8"
surface = "#171717"
surface_elevated = "#222222"
danger = "#ff4f64"
warning = "#ffcc66"
success = "#45d483"

[radius]
xs = 4
sm = 6
md = 10
lg = 16
xl = 22
pill = 999

[spacing]
xs = 4
sm = 8
md = 12
lg = 16
xl = 24
xxl = 32

[font]
family = "Inter"
mono_family = "JetBrains Mono"
size = 14
small = 12
title = 20
large_title = 28

[animation]
fast_ms = 90
normal_ms = 160
slow_ms = 240
curve = "emphasized-decelerate"
```

Component variants:

```rust
Button::primary("Save")
Button::secondary("Cancel")
Button::destructive("Delete")
Button::ghost("More")
```

Control sizes:

```txt
compact
small
regular
large
touch
```

Good apps should need almost no custom styling.

Avoid:

- global selector hell
- `nth-child`
- specificity wars
- unpredictable inheritance
- CSS-like accidental coupling

### 12.1 Styling Freedom

Stuk must not be visually restrictive.

The default Stuk design system should be polished and coherent, but apps must still have enough styling freedom to create beautiful, branded, expressive interfaces like they can on the web.

Stuk should support the good parts of web styling:

- flexible composition
- custom spacing
- custom colors
- gradients
- shadows
- borders
- rounded corners
- layered surfaces
- opacity
- transforms
- transitions
- responsive layout
- backdrop blur
- material effects
- custom component variants
- reusable style tokens
- scoped styling
- theme overrides

Stuk should avoid the bad parts:

- global cascade chaos
- selector specificity fights
- accidental inheritance
- layout hacks
- browser-only mental model
- fake blur effects when native compositor blur is available

The styling system should allow both approaches:

```rust
Button::primary("Save")
```

for fast, polished defaults, and:

```rust
Card::new()
    .background(LinearGradient::vertical([color("#1b1d2a"), color("#101116")]))
    .border(Border::new(color("#ffffff").opacity(0.12), 1.0))
    .corner_radius(22)
    .shadow(Shadow::soft())
    .backdrop_blur(BackdropBlur::Material(Material::Luca))
```

for custom expressive UI.

The goal is not to trap apps inside one rigid house style. The goal is to make the default beautiful while still giving developers enough power to make apps feel special.

### 12.2 Scoped Styles

Stuk may support scoped styles for reusable components.

Example concept:

```rust
style! {
    NoteCard {
        background: Material::Maris;
        radius: 18;
        padding: 14;
        border: Border::subtle();
        shadow: Shadow::soft();
    }

    NoteCard:hover {
        transform: translate_y(-2);
        shadow: Shadow::medium();
    }
}
```

Scoped styles must be local to a component/module unless explicitly exported. There should be no app-wide selector chaos by default.

### 12.3 Visual Effects as First-Class Styling

Visual effects must be native framework features, not hacks.

Stuk should support:

```txt
opacity
shadow
inner shadow later
blur
backdrop blur
compositor blur
saturation
brightness
contrast
noise
tint
gradient fills
rounded clipping
masks later
transforms
transitions
```

Effects should be represented semantically and resolved by the platform renderer.

Example:

```rust
Surface::new()
    .material(Material::Luca)
    .backdrop_blur(BackdropBlur::Compositor { radius: 32.0 })
    .saturation(1.25)
    .tint(color("#ffffff").opacity(0.18))
    .noise(0.035)
```

On Staccato/Baton, compositor-backed blur must be used when available.

On other platforms, Stuk should map to native platform blur/material APIs where possible or fall back gracefully.

### 12.4 Native Blur and Backdrop Blur

Blur is a first-class Stuk capability.

Stuk must distinguish:

```txt
Local blur:
  blur applied to content rendered inside the app surface

Backdrop blur:
  blur applied to what visually appears behind a surface

Compositor blur:
  blur requested from the OS/compositor because only the compositor can correctly sample behind the window
```

Local blur can be done by Stuk's renderer.

Backdrop/compositor blur must use platform integration where available.

On Staccato/Baton:

- `Material::Luca` should request Baton compositor blur.
- `BackdropBlur::Compositor` should use Baton or `ext-background-effect-v1` where appropriate.
- rounded blur regions must be supported.
- blur must be clipped to the app/window/surface region.
- blur must avoid leaking protected/secure content.
- blur must participate in damage tracking.

On generic Wayland:

- use `ext-background-effect-v1` if the compositor supports it.
- otherwise fall back to Maris/Solid/tinted surfaces.

On Windows/macOS:

- map to native background material APIs where available.
- otherwise fall back gracefully.

App authors should not need to manually screenshot the background, blur it, and fake a glass effect. That is cursed and must not be the normal path.

---

## 13. Material System

Materials are semantic surfaces.

```rust
pub enum Material {
    Solid(Color),
    Surface,
    SurfaceElevated,
    Window,
    Sidebar,
    Toolbar,
    Popover,
    Menu,
    Dialog,
    Maris,
    Luca,
}
```

### Luca

Live glass material.

On Staccato:

- Baton compositor live blur
- background effect
- tint
- saturation
- noise
- border highlight
- rounded clipping
- shadow

Use for:

- popovers
- command palette
- shell overlays
- translucent panels
- special surfaces

### Maris

Wallpaper-derived material.

On Staccato:

- wallpaper sampling
- tint extraction
- optional cached blur
- subtle noise
- lower cost than Luca
- not live blur of arbitrary windows

Use for:

- app windows
- sidebars
- panels
- settings pages
- battery-friendly surfaces

Apps ask for intent:

```rust
Window::new().material(Material::Maris)
Popover::new().material(Material::Luca)
```

Platform resolves it.

Stuk must never require app authors to implement fake blur manually.

---

## 14. Rendering Engine

Pipeline:

```txt
View tree
→ reconciliation
→ style resolution
→ layout
→ display list
→ damage calculation
→ GPU render
→ present
```

Display list commands:

```rust
pub enum DisplayCommand {
    Rect(RectCommand),
    RoundedRect(RoundedRectCommand),
    Border(BorderCommand),
    Shadow(ShadowCommand),
    Text(TextCommand),
    Image(ImageCommand),
    Svg(SvgCommand),
    Clip(ClipCommand),
    Transform(TransformCommand),
    Material(MaterialCommand),
}
```

Requirements:

- HiDPI support
- fractional scale support
- glyph cache
- image cache
- SVG cache
- shadow cache where useful
- clipping
- rounded corners
- opacity
- transforms
- smooth animation
- damage tracking
- performance stats

Stuk must avoid repainting everything when only a small area changed.

---

## 15. Text System

Text is one of the hardest and most important parts.

Required widgets:

```txt
Text
Label
SelectableText
TextField
TextArea
PasswordField
SearchField
TextEditor-lite
```

Text rendering requirements:

- crisp text
- font fallback
- shaping
- emoji
- ligatures where supported
- HiDPI
- fractional scale
- system font defaults
- configurable app fonts

Text editing requirements:

- caret
- selection
- mouse selection
- keyboard selection
- copy/paste
- cut
- undo/redo
- word movement
- line movement
- Home/End
- IME composition
- dead keys
- password masking
- placeholder text
- validation state
- accessibility labels

Use `cosmic-text` or equivalent serious shaping/editing support. Do not hand-roll shaping.

If text input feels bad, the framework feels fake.

---

## 16. Widgets

MVP widgets:

```txt
Window
Text
Button
IconButton
VStack
HStack
ZStack
ScrollView
TextField
Toggle
List
Sidebar
Toolbar
SplitView
Popover
Dialog
Spinner
EmptyState
ErrorView
```

Full core widget target:

```txt
Window
Titlebar
Toolbar
Button
IconButton
Toggle
Checkbox
Radio
Slider
TextField
TextArea
PasswordField
SearchField
Dropdown
Menu
ContextMenu
Popover
Dialog
Toast
Tabs
SegmentedControl
Sidebar
NavigationView
List
VirtualList
Tree
Table
Grid
Card
ScrollView
SplitView
ResizablePane
CommandPalette
SettingsPage
Form
ProgressBar
Spinner
EmptyState
ErrorView
Tooltip
Badge
Avatar
ColorWell
DatePicker later
TimePicker later
```

Every built-in widget must support:

- hover state
- focused state
- pressed state
- disabled state
- keyboard navigation
- accessibility role
- accessibility label/value
- theme tokens
- scaling
- reduced motion
- validation state where relevant

---

## 17. Accessibility

Use AccessKit.

Every widget must emit accessibility data:

- role
- label
- value
- state
- bounds
- actions
- focus state

Required features:

- keyboard navigation
- visible focus rings
- reduced motion
- high contrast
- text scaling
- accessible dialogs
- accessible menus
- accessible form fields
- screen-reader-compatible tree

`stuk validate` should warn about:

- icon buttons without labels
- form fields without labels
- inaccessible custom widgets
- low obvious contrast
- dialogs without titles
- unreachable keyboard controls

Accessibility is not a “later” feature. It must be in the widget model.

---

## 18. Actions and Command System

Actions are first-class.

```rust
pub struct ActionDescriptor {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub shortcut: Option<Shortcut>,
    pub category: Option<String>,
    pub enabled: bool,
    pub visible: bool,
}
```

Actions can be used by:

- buttons
- menus
- command palette
- keyboard shortcuts
- Staccato Shell
- settings/keybinding UI
- agents

Example:

```rust
actions! {
    new_note {
        id: "notes.new",
        label: "New Note",
        shortcut: "Ctrl+N",
        run: Action::NewNote,
    }

    search {
        id: "notes.search",
        label: "Search",
        shortcut: "Ctrl+F",
        run: Action::FocusSearch,
    }
}
```

On Staccato, app actions can appear in the global command palette.

---

## 19. Settings Schema

Apps should define settings declaratively.

Example:

```toml
[settings.editor.font_size]
type = "number"
label = "Editor font size"
default = 15
min = 10
max = 30

[settings.appearance.theme]
type = "enum"
label = "Theme"
values = ["system", "light", "dark"]
default = "system"

[settings.sync.enabled]
type = "boolean"
label = "Enable sync"
default = false
```

Runtime API:

```rust
let font_size = settings.get_number("editor.font_size");
settings.set("appearance.theme", "dark");
```

Stuk can render a settings page automatically:

```rust
SettingsPage::from_schema(app_settings_schema)
```

Staccato Settings can read the schema later.

---

## 20. Manifest: Stuk.toml

Every app should have `Stuk.toml`.

Example:

```toml
[app]
id = "com.lantharos.notes"
name = "Notes"
version = "0.1.0"
icon = "assets/icon.svg"

[window.main]
title = "Notes"
width = 1100
height = 760
min_width = 720
min_height = 480
material = "maris"
chrome = "compact"

[platform.staccato]
command_palette = true
workspace_sessions = true
shell_tabs = true
preferred_mode = "browser"
preferred_material = "maris"
preferred_chrome = "compact"

[permissions]
network = false
filesystem = "documents"
notifications = true
camera = false
microphone = false
background = false

[actions.notes.new]
label = "New Note"
shortcut = "Ctrl+N"

[actions.notes.search]
label = "Search"
shortcut = "Ctrl+F"

[settings.appearance.theme]
type = "enum"
label = "Theme"
values = ["system", "light", "dark"]
default = "system"
```

`stuk validate` checks:

- app ID format
- icon exists
- version valid
- window sizes valid
- material/chrome values valid
- action IDs valid
- shortcut conflicts
- settings schema valid
- permissions valid
- unsupported platform features

App IDs should use reverse DNS:

```txt
com.lantharos.notes
net.aveid.manager
dev.example.app
```

---

## 21. Platform Integration

Conceptual platform trait:

```rust
pub trait Platform {
    fn create_window(&mut self, options: WindowOptions) -> Result<WindowHandle>;
    fn destroy_window(&mut self, window: WindowId);
    fn request_redraw(&mut self, window: WindowId);
    fn set_title(&mut self, window: WindowId, title: &str);
    fn set_material(&mut self, window: WindowId, material: Material);
    fn set_chrome(&mut self, window: WindowId, chrome: WindowChrome);
    fn register_actions(&mut self, actions: &[ActionDescriptor]);
    fn read_clipboard(&self) -> Option<ClipboardData>;
    fn write_clipboard(&self, data: ClipboardData);
    fn open_file_dialog(&self, options: FileDialogOptions) -> FileDialogResult;
    fn platform_capabilities(&self) -> PlatformCapabilities;
}
```

Capabilities:

```rust
pub struct PlatformCapabilities {
    pub live_blur: bool,
    pub wallpaper_material: bool,
    pub shell_tabs: bool,
    pub command_palette: bool,
    pub workspace_sessions: bool,
    pub native_notifications: bool,
    pub system_dark_mode: bool,
    pub high_contrast: bool,
}
```

Window chrome:

```rust
pub enum WindowChrome {
    System,
    Stuk,
    Compact,
    Sidebar,
    None,
}
```

Chrome meanings:

- `System`: platform/system decorations
- `Stuk`: Stuk-rendered titlebar
- `Compact`: titlebar merged with toolbar
- `Sidebar`: sidebar-integrated app chrome
- `None`: fully custom, still accessible

---

## 22. Staccato Integration

Stuk apps should not merely run on Staccato. They should participate in it.

Capabilities:

- request Baton-backed Luca/Maris materials
- register actions with global command palette
- expose settings schema
- participate in workspace sessions
- participate in Browser Mode tabs
- provide document/session identity
- provide preferred split hints
- use Staccato file dialogs
- use Staccato notifications
- expose permissions
- use Staccato-native chrome

Example Staccato API:

```rust
cx.staccato().set_tab_title("Notes");
cx.staccato().set_document_id(note_id);
cx.staccato().set_preferred_split(SplitHint::Right);
cx.staccato().register_actions(actions);
```

Workspace session restore:

```rust
cx.session().set_document_id(note_id);
cx.session().set_restore_payload(payload);
```

Staccato manifest fields:

```toml
[platform.staccato]
command_palette = true
workspace_sessions = true
shell_tabs = true
preferred_mode = "browser"
preferred_material = "maris"
preferred_chrome = "compact"
```

---

## 23. Permissions and Security

Stuk apps declare permissions.

Initial categories:

```txt
network
filesystem
notifications
camera
microphone
location
clipboard
background
shell_integration
command_execution
screen_capture
input_capture
```

Example:

```toml
[permissions]
network = true
filesystem = "documents"
notifications = true
camera = false
microphone = false
background = false
```

Early versions may only declare permissions. Later Staccato/Glacier/Schelf can enforce them.

API design must not block future sandbox enforcement.

Dangerous APIs should require explicit permission declarations:

- command execution
- full filesystem access
- background processes
- screen capture
- input capture
- shell integration
- clipboard monitoring

---

## 24. Devtools

Stuk must have devtools because web-dev velocity requires tooling.

### `stuk dev`

```sh
stuk dev
```

Features:

- runs app in dev mode
- watches files
- reloads style/token changes
- rebuilds/relaunches when needed
- preserves window position where possible
- shows errors clearly
- enables inspector
- enables performance overlay

### Inspector

Open with:

```txt
Ctrl+Shift+I
```

or:

```sh
stuk inspect
```

Inspector shows:

- component tree
- layout boxes
- computed tokens/styles
- accessibility tree
- registered actions
- shortcuts
- platform capabilities
- render timing
- layout timing
- repaint/damage regions
- resources/mutations
- focused widget
- material resolution

### Performance overlay

Shows:

- FPS
- frame time
- CPU time
- GPU time where available
- layout time
- render time
- text shaping time
- dirty region
- draw calls
- glyph cache size
- image cache size
- memory estimate

### Component previews

```sh
stuk preview
```

App-defined previews:

```rust
preview! {
    NoteRowPreview => view! {
        NoteRow(note: fake_note())
    }

    SettingsPreview => view! {
        SettingsView(settings: fake_settings())
    }
}
```

Preview should support:

- themes
- density
- platform fallback simulation
- material simulation
- accessibility inspection
- layout inspection

---

## 25. CLI

Required commands:

```sh
stuk new <name>
stuk dev
stuk run
stuk build
stuk validate
stuk doctor
stuk inspect
stuk preview
stuk fmt
stuk check
stuk bundle
```

### `stuk new`

Creates a structured app.

```sh
stuk new notes
stuk new notes --template sidebar
stuk new settings --template settings
```

Output:

```txt
notes/
├── Stuk.toml
├── AGENTS.md
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── state.rs
│   ├── actions.rs
│   ├── views/
│   └── components/
└── assets/
```

### `stuk validate`

Checks:

- manifest
- action IDs
- shortcut conflicts
- settings schema
- accessibility warnings
- missing labels
- icon existence
- asset paths
- unsupported platform features
- bad permission values

Must support JSON:

```sh
stuk validate --json
```

### `stuk doctor`

Checks:

- Rust toolchain
- GPU/wgpu support
- platform backend support
- missing system deps
- Staccato/Baton integration if present
- renderer info

---

## 26. Agent-Friendly Design

Agent support is a first-class requirement.

Default app structure:

```txt
my-app/
├── Stuk.toml
├── AGENTS.md
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── state.rs
│   ├── actions.rs
│   ├── views/
│   │   ├── main_window.rs
│   │   ├── settings.rs
│   │   └── about.rs
│   └── components/
│       ├── sidebar.rs
│       └── toolbar.rs
├── assets/
└── tests/
```

Generated `AGENTS.md`:

```txt
# Agent Instructions

- Views live in `src/views/`.
- Reusable UI components live in `src/components/`.
- App state lives in `src/state.rs`.
- User actions live in `src/actions.rs`.
- App metadata, permissions, windows, actions, and settings schema live in `Stuk.toml`.
- Do not edit generated files in `.stuk/`.
- Prefer existing Stuk widgets before custom drawing.
- Use semantic materials (`Maris`, `Luca`, `Surface`) instead of hardcoded blur.
- Run `stuk validate` after changes.
- Run `cargo test` if logic changed.
```

`stuk validate --json` should output machine-readable diagnostics.

Example:

```json
{
  "ok": false,
  "diagnostics": [
    {
      "level": "error",
      "path": "Stuk.toml",
      "message": "Action notes.new has invalid shortcut Ctrl+",
      "fix_hint": "Use a valid shortcut such as Ctrl+N"
    }
  ]
}
```

Stable conventions matter because agents destroy messy projects.

---

## 27. Performance Requirements

Runtime targets:

```txt
60 FPS minimum
120/144Hz capable where hardware supports
low input latency
smooth scrolling
near-zero idle CPU
near-zero idle GPU
fast startup
memory far below webview equivalents for simple apps
```

Initial measurable goals for simple apps:

```txt
simple window visible quickly after process launch
idle CPU near 0%
frame budget at 60Hz: 16.6ms
frame budget at 120Hz: 8.3ms
no full relayout for local text changes where avoidable
no full repaint for tiny damage where avoidable
```

Performance features:

- damage tracking
- incremental layout invalidation
- glyph cache
- image cache
- SVG cache
- virtual lists
- batched draw calls
- render timing
- layout timing
- text timing
- tracing
- benchmarks

Benchmarks:

- view diffing
- layout
- text shaping
- virtualized list
- display list rendering
- startup
- animation overhead

Stuk should include a performance culture from day one. Do not “optimize later” after architecture makes it impossible.

---

## 28. Testing

Tests required:

- manifest parsing
- settings schema
- action registry
- shortcut parser
- layout primitives
- style token resolution
- widget state
- accessibility node generation
- platform capability resolution

Interaction tests:

- button click
- keyboard focus
- text input
- selection
- scrolling
- list selection
- dialog open/close
- menu navigation

Snapshot/golden tests:

- widgets
- themes
- layout regressions
- material fallbacks

Headless rendering should be supported eventually for CI screenshots.

---

## 29. Packaging

`stuk build`:

```sh
stuk build
stuk build --release
stuk build --target staccato
stuk build --target linux
stuk build --target windows
stuk build --target macos
```

`stuk bundle`:

```sh
stuk bundle --target staccato
stuk bundle --target flatpak
stuk bundle --target appimage
stuk bundle --target windows
stuk bundle --target macos
```

Staccato bundle should include:

- binary
- manifest
- icon
- resources
- permission metadata
- actions metadata
- settings schema
- desktop/app launcher metadata
- Staccato integration metadata

Flatpak support is important for generic Linux.

Schelf support can come later.

---

## 30. Documentation

Docs must include:

- getting started
- project structure
- view system
- state/actions
- layout
- styling/tokens
- widgets
- text input
- async/resources
- accessibility
- devtools
- Staccato integration
- packaging
- agent guide

Examples:

```txt
hello
counter
todo
notes
settings
file picker
split view
sidebar app
command palette
text editor lite
```

Docs should be code-heavy, not essay-heavy.

---

## 31. Implementation Milestones

### Milestone 1: Workspace Skeleton

Build:

- Rust workspace
- crates:
  - stuk
  - stuk-core
  - stuk-layout
  - stuk-render
  - stuk-style
  - stuk-widgets
  - stuk-platform
  - stuk-manifest
  - stuk-cli
- README
- `stuk.md`
- basic CLI

Done when:

```sh
cargo build
cargo test
stuk --help
```

works.

### Milestone 2: Native Window + GPU Renderer

Build:

- native window
- wgpu initialization
- resize handling
- scale factor handling
- render background
- render rounded rect
- render text
- display list abstraction

Done when example window shows background, text, and button-like rectangle.

### Milestone 3: Layout + View Tree

Build:

- basic view tree
- basic reconciliation
- VStack
- HStack
- ZStack
- padding
- spacing
- fixed/fill sizing
- display list generation

Done when a simple component tree renders correctly.

### Milestone 4: MVP Widgets

Build:

- Text
- Button
- IconButton
- Toggle
- TextField basic
- ScrollView basic
- Sidebar basic
- Toolbar basic
- SplitView basic

Done when a small settings/notes mockup can be rendered.

### Milestone 5: State + Actions

Build:

- signals
- app state
- action registry
- shortcuts
- event dispatch
- button action triggering

Done when counter/todo examples work.

### Milestone 6: Manifest + CLI

Build:

- `Stuk.toml` parser
- manifest validation
- `stuk new`
- `stuk dev`
- `stuk validate`
- templates
- generated `AGENTS.md`

Done when a new app can be created, run, and validated.

### Milestone 7: Text Input Serious Pass

Build:

- caret
- selection
- keyboard editing
- clipboard
- undo/redo
- IME start
- password field
- accessibility metadata

Done when text fields feel usable.

### Milestone 8: Styling + Themes

Build:

- tokens
- themes
- component variants
- light/dark mode
- density
- material enum
- Maris fallback

Done when app can switch theme and widgets update.

### Milestone 9: Devtools

Build:

- performance overlay
- layout inspector
- component tree inspector
- action inspector
- `stuk preview`

Done when native dev loop starts feeling meaningfully better.

### Milestone 10: Accessibility

Build:

- AccessKit integration
- roles for widgets
- labels
- focus traversal
- keyboard navigation
- validation warnings

Done when basic apps expose a useful accessibility tree.

### Milestone 11: Staccato Backend

Build:

- material requests to Baton/Staccato
- command registration with shell
- settings schema exposure
- session restore hooks
- Browser Mode tab metadata

Done when a Stuk app feels special on Staccato.

### Milestone 12: Performance Pass

Build:

- damage tracking
- glyph cache
- image cache
- virtual list
- benchmarks
- tracing
- startup optimization

Done when simple Stuk apps are clearly lighter than webview equivalents.

### Milestone 13: Cross-Platform Expansion

Build:

- generic Wayland polish
- Windows backend
- macOS backend
- packaging targets

Done when Stuk apps can run outside Staccato.

---

## 32. First Codex Goal

If starting from zero, give Codex this first narrow task:

```txt
Create the initial Stuk Rust workspace and implement a minimal native GPU-rendered window.

Requirements:
- Create a Cargo workspace with crates:
  - stuk
  - stuk-core
  - stuk-layout
  - stuk-render
  - stuk-style
  - stuk-widgets
  - stuk-platform
  - stuk-manifest
  - stuk-cli
- `stuk` should re-export a simple public prelude.
- `stuk-cli` should provide `stuk --help` and placeholder subcommands:
  - new
  - dev
  - build
  - validate
  - doctor
- Implement a minimal example app in `examples/hello`.
- The example app must open a native window.
- Initialize wgpu and render:
  - background
  - rounded rectangle
  - text
  - button-like rectangle with label
- Add a minimal display list abstraction.
- Add a minimal layout abstraction with VStack and HStack.
- Add placeholder public APIs for:
  - App
  - Window
  - Text
  - Button
  - VStack
  - HStack
- Add README instructions for running the hello example.
- Do not implement full widgets yet.
- Do not implement Staccato integration yet.
- Do not implement AI features.
- Keep architecture compatible with the full `stuk.md` spec.
```

---

## 33. Definition of “Does Not Suck”

Stuk does not suck when:

```txt
[ ] `stuk new app` works.
[ ] `stuk dev` gives a fast loop.
[ ] a basic app looks good without custom styling.
[ ] text rendering is crisp.
[ ] text input feels real.
[ ] scrolling is smooth.
[ ] animations are smooth.
[ ] layout is predictable.
[ ] components are easy to compose.
[ ] app structure is obvious.
[ ] `stuk validate` catches common mistakes.
[ ] accessibility exists.
[ ] performance is measurable.
[ ] Staccato integration is first-class.
[ ] apps do not drag a browser engine around.
```

---

## 34. Final Target

Target developer experience:

```sh
stuk new notes
cd notes
stuk dev
```

Developer writes:

```rust
view! {
    Window(title: "Notes", material: Maris, chrome: Compact) {
        SplitView {
            Sidebar {
                SidebarItem("All Notes")
                SidebarItem("Tags")
            }

            VStack {
                Toolbar(title: "Notes") {
                    Button.primary("New", action: "notes.new")
                }

                TextEditor(bind: state.current_note.body)
            }
        }
    }
}
```

Stuk provides:

- polished UI
- native rendering
- fast reload
- stable structure
- command palette integration
- settings schema
- materials
- accessibility
- packaging
- validation
- devtools

On Staccato:

- app actions appear globally
- Luca/Maris are powered by Baton
- windows can become Browser Mode tabs
- sessions restore into workspaces
- settings integrate with the OS
- permissions are visible

The final feeling:

```txt
The speed of web dev.
The performance of native.
The polish of a coherent OS.
The structure agents need.
```
