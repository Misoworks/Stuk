# Stuk Specification

## 0. Purpose

Stuk is a native UI framework and app-integration system for desktop, mobile, and web apps.

Its purpose is to make native app development feel as fast and pleasant as modern web development, while keeping native rendering, low overhead, deep platform integration, accessibility, and platform-native background effects where the target supports them.

Stuk is not “GTK but prettier.” It is a Rust-first, GPU-rendered, declarative native framework designed around:

- web-dev velocity
- native runtime performance
- excellent defaults
- predictable layout
- polished widgets
- OS-level integration
- fast devtools
- agent-friendly project structure
- cross-platform desktop, mobile, and web fallback
- platform-native window chrome, blur, vibrancy, and fallback surfaces

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
7. Support platform background effects with honest capability fallback.
8. Support command/action integration.
9. Support declarative settings schemas.
10. Support fast dev loops.
11. Support component previews.
12. Support a real inspector.
13. Be strongly structured for AI coding agents.
14. Target Linux, Windows, macOS, Android, iOS, and WebAssembly/web where practical.
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

Stuk has platform tiers. The app-facing component, state, style, layout, text, action, and rendering model should be shared across all targets. Each platform backend owns window/surface creation, input, IME, accessibility, clipboard, permissions, packaging, and native capability resolution.

### Tier 1: Desktop

Primary production target:

- Linux Wayland first, X11 compatibility
- Windows
- macOS
- native windows
- wgpu rendering
- system clipboard
- drag and drop
- notifications where available
- IME
- accessibility
- native background effects where available
- Stuk chrome or system chrome

### Tier 2: Mobile

First-class target once the desktop core is stable:

- Android
- iOS
- wgpu surface rendering
- touch/pointer input
- virtual keyboard and IME
- safe-area insets
- app lifecycle suspend/resume
- mobile accessibility trees
- platform permissions
- app store packaging metadata

### Tier 3: Web

Supported through a WASM + canvas/WebGPU shell:

- shared Stuk view/layout/state code
- Stuk-created browser canvas appended and focused by the framework by default
- WebGPU preferred, WebGL/canvas fallback only if viable
- browser clipboard and file APIs through permission-gated adapters
- DOM-free widget rendering by default
- web accessibility mapping where practical
- no native blur promises beyond what browser APIs can honestly provide

### Future Tier: Staccato / Glacier

Best experience once those platform APIs exist:

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

Apps request platform features. Platforms resolve them and report whether the effect actually worked.

Example:

```txt
Window backdrop effect
  Linux/Wayland -> ext-background-effect-v1 blur if the compositor supports it
  Windows -> Acrylic by default, Mica when requested
  macOS -> NSVisualEffectView vibrancy when available
  unsupported -> opaque/tinted fallback
```

Materials such as Maris and Luca are future semantic names. Stuk v1 should expose plain platform background-effect requests and platform overrides first; semantic materials should be added when there is a concrete app use case.

### Target and UI Reuse Models

Stuk apps should be able to choose how much UI they share:

```txt
Shared UI:
  one Stuk component tree adapts across desktop, mobile, and web through responsive layout,
  capabilities, and target-aware services.

Shared logic, platform UI:
  domain/state/services are shared, while desktop/mobile/web each provide their own shell,
  navigation, or pages.

Base UI with overrides:
  most views are shared, but a target can replace a page, component, route, or command behavior.
```

Recommended structure for apps that need platform variation:

```txt
src/
├── domain/              # pure app logic, no platform imports
├── state.rs             # shared app state
├── services/            # capability-facing traits and shared service orchestration
├── views/               # shared Stuk UI
├── components/          # shared reusable UI
└── platforms/
    ├── mod.rs           # target selection boundary
    ├── desktop.rs       # desktop shell/overrides
    ├── mobile.rs        # mobile shell/overrides
    ├── web.rs           # web shell/overrides
    ├── linux.rs
    ├── windows.rs
    ├── macos.rs
    ├── android.rs
    └── ios.rs
```

Rules:

- Shared domain logic must not import platform/window/webview crates directly.
- Shared views/components should prefer `Cx` capabilities, responsive layout, and service traits over
  `cfg` branches.
- Target-specific imports belong in `src/platforms/`, platform crates, or generated boundary files.
- Platform overrides should replace clear units: app shell, route/page, component, command, or
  service implementation.
- Unsupported features must be represented as capabilities or typed errors, not hidden panics.
- `stuk validate` should flag unsupported target/runtime combinations and broad bridge permissions.
- `Cx` exposes the resolved backend descriptor, runtime target, and capabilities so shared UI can
  branch on `cx.is_desktop()`, `cx.is_mobile()`, `cx.is_web()`, or `cx.capabilities()` without
  scattering platform imports through views.
- `App` may be given an explicit `BackendDescriptor` for platform override testing or alternate
  runners. In normal apps it defaults to the current native/backend target.
- `TargetSet` and backend descriptors are the framework-level boundary between manifests, build
  tooling, and app code. Desktop can be generic (`desktop = true`) or narrowed by OS; mobile and web
  must be explicit.
- `PlatformOverrideRegistry` records target-specific replacements for app shells, pages,
  components, commands, and services. Generated projects should start with `src/platforms/desktop.rs`,
  `src/platforms/mobile.rs`, and `src/platforms/web.rs` so target logic has a stable home from day one.

WebView target rules:

- CEF-backed webview apps are desktop-only native apps.
- WebView apps must not target native Android/iOS runtimes.
- The same web UI can target the browser as a web build by bypassing the CEF runtime and using web
  service adapters instead of native bridge commands.
- Desktop-only webview logic must be behind target/capability checks and must have a web fallback,
  disabled state, or explicit unsupported error.

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
│   ├── stuk-platform-android/
│   ├── stuk-platform-ios/
│   ├── stuk-platform-web/
│   ├── stuk-accessibility/
│   ├── stuk-actions/
│   ├── stuk-settings/
│   ├── stuk-manifest/
│   ├── stuk-devtools/
│   └── stuk-cli/
├── examples/
│   └── web-runtime/
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

Responsive layout is first-class. View code can read the current viewport from `Cx` and resolve
typed `Responsive<T>` values against standard breakpoints:

```rust
let columns = cx.responsive(&Responsive::new(1).medium(2).expanded(3));
let show_sidebar = cx.is_at_least(Breakpoint::Medium);
```

Breakpoints are semantic, not platform names:

- `Compact`: phone/narrow windows
- `Medium`: large phone, small tablet, narrow desktop
- `Expanded`: tablet/normal desktop
- `Wide`: large desktop or external displays

The same responsive API must work on desktop, mobile, and web targets.

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

On Linux/Wayland:

- bind `ext_background_effect_manager_v1` from the active `wl_display` registry and request `ext_background_effect_surface_v1` on the window `wl_surface`.
- keep the effect object alive for the lifetime of the native window.
- listen for the manager `capabilities` event and only enable blur when the compositor advertises the `blur` capability.
- create an `ext_background_effect_surface_v1`, set a `wl_region` through `set_blur_region`,
  and apply it with `wl_surface.commit`.
- support adaptive `wl_region` declarations for full-window blur, rounded window input regions,
  rounded sidebar blur regions, opaque regions, and explicit fixed rect lists. Adaptive regions
  must be recalculated after window resize so compositor blur/input masks keep matching the UI.
- use `wl_surface.set_input_region` for shaped click regions and `wl_surface.set_opaque_region`
  when a surface area is known to be opaque.
- otherwise fall back to opaque/tinted surfaces.
- expose `PlatformCapabilities::live_blur` and `PlatformCapabilities::transparent_windows` so apps can gate transparent surfaces behind the effect actually working.

On Windows:

- use `DwmSetWindowAttribute` for acrylic/mica effects where available.
- allow explicit material overrides (e.g. `acrylic`, `mica`, `mica-alt`) in window constructors.
- otherwise fall back gracefully.

On macOS:

- use `NSVisualEffectView` with vibrancy where available.
- otherwise fall back gracefully.

Apps should check `PlatformCapabilities::live_blur` before relying on transparency:

```rust
if cx.capabilities().live_blur && cx.capabilities().transparent_windows {
    Surface::new().backdrop_blur(BackdropBlur::Compositor { radius: 32.0 })
} else {
    Surface::new().background(Color::SURFACE)
}
```

App authors should not need to manually screenshot the background, blur it, and fake a glass effect. That is cursed and must not be the normal path.

Named app materials are future work. Until there is a concrete product use case, the runtime should prioritize explicit platform background effects and reliable fallbacks over a large semantic material system.

---

## 13. Surfaces and Background Effects

Early Stuk should keep surfaces simple. Opaque and tinted surfaces are the baseline; platform background effects are opt-in and must degrade cleanly.

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
}
```

### Current behaviour

All materials must resolve to a platform-appropriate effect or a surface colour. Apps relying on transparency must check `PlatformCapabilities::live_blur` and `PlatformCapabilities::transparent_windows` and provide an opaque fallback.

### Current Platform Effects

```rust
WaylandPlatform::with_background_effects(); // ext_background_effect_v1 blur
WindowsPlatform::with_backdrop(WindowsBackdrop::Acrylic);
WindowsPlatform::with_backdrop(WindowsBackdrop::Mica);
MacosPlatform::with_vibrancy(MacosVibrancy::HudWindow);
```

Platform overrides:

App authors may request specific native effects by name when the platform supports them:

```rust
Window::new()
    .transparent(true)
    .background_effect(WindowBackgroundEffect::Mica) // Windows: Mica
    .material(Material::Surface)
```

`WindowBackgroundEffect` names the requested per-window effect. Platforms resolve it against `PlatformCapabilities` before creating the window. If the requested effect or transparency is not available, the platform clears the effect request and creates an opaque window so apps can use the same constructor while still gating glass-dependent layouts behind capability checks.

For common app windows, Stuk should expose a cohesive shortcut rather than requiring authors to remember every low-level option:

```rust
Window::new()
    .title("Notes")
    .size(760, 520)
    .glass()
```

`Window::glass()` means transparent window, compositor blur where supported, `Material::Window`, and native system chrome by default. Stuk-rendered chrome remains available as an explicit `chrome(WindowChrome::Stuk)` override once the app wants custom integrated controls. It must not be a visual-only preset; it is a functional window primitive.

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
- per-text wrapping intent: normal, balanced titles, and pretty paragraph wrapping
- proportional numbers by default with opt-in tabular numbers for counters and tables
- table rows provide numeric cells that opt into tabular numbers without requiring custom text styling

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
Image
Svg
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

Media widgets should render inspection-quality assets by default:

- raster images get a subtle theme-aware one-pixel outline so light or dark content does not disappear into the window
- SVG icons do not outline by default, but can opt in when used as framed artwork rather than iconography
- apps can disable the image outline when the asset already includes an intentional edge

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
transparent = false
background_effect = "none"

[platform.staccato]
command_palette = true
workspace_sessions = true
shell_tabs = true
preferred_mode = "browser"
preferred_material = "maris"
preferred_chrome = "compact"

[targets]
desktop = true
linux = true
windows = true
macos = true
android = false
ios = false
web = false

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
- unsupported target/runtime combinations

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
    fn backend(&self) -> BackendDescriptor;
}
```

Backend descriptors:

```rust
pub struct BackendDescriptor {
    pub name: String,
    pub kind: BackendKind,
    pub target: RuntimeTarget,
    pub status: BackendStatus,
    pub capabilities: PlatformCapabilities,
    pub limitations: Vec<String>,
}
```

Capabilities:

```rust
pub struct PlatformCapabilities {
    pub native_windows: bool,
    pub web_surface: bool,
    pub mobile_shell: bool,
    pub native_bridge: bool,
    pub live_blur: bool,
    pub transparent_windows: bool,
    pub wallpaper_material: bool,
    pub touch_input: bool,
    pub pointer_input: bool,
    pub keyboard_input: bool,
    pub file_dialogs: bool,
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

Target declarations:

```toml
[targets]
desktop = true
linux = true
windows = true
macos = true
android = false
ios = false
web = true
```

`desktop = true` means the app is intended to run on the generic desktop app shell. Platform-specific
flags narrow or expand generated bundles. Webview apps may set `web = true` for a browser build, but
must keep `android` and `ios` false unless they also provide native Stuk mobile UI entries.

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
stuk build --target android
stuk build --target ios
stuk build --target web
```

`stuk bundle`:

```sh
stuk bundle --target staccato
stuk bundle --target flatpak
stuk bundle --target appimage
stuk bundle --target windows
stuk bundle --target macos
stuk bundle --target android
stuk bundle --target ios
stuk bundle --target web
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


---

## Addendum: Stuk WebView, CEF Runtime, and Hybrid UI

### Purpose

Stuk should not be limited to only native widgets.

Stuk is an app runtime first. It should support both:

```txt
Stuk Native
  GPU-rendered native widgets for fully native apps and shell-grade UI.

Stuk WebView
  CEF-backed embedded web UI for high-velocity apps that still want a Rust backend,
  permissions, actions, settings schema, and shared runtime management.
```

This is not a contradiction. It is a practical split.

Some apps should be fully native. Some apps benefit from web UI velocity. Stuk should allow both without forcing developers into Tauri/WebKitGTK, Electron sidecars, or each app bundling its own browser runtime.

The real enemy is not web UI. The real enemy is:

```txt
random platform webviews
bad Linux WebKitGTK behavior
per-app browser runtime bloat
sidecar/backend duplication
weak OS integration
fake blur/material hacks
inconsistent app structure
```

Stuk WebView should provide the practical speed of web UI while keeping Stuk's Rust runtime and OS integration model.

### Architecture

Stuk should be structured as:

```txt
Stuk Runtime
├── app lifecycle
├── windows
├── platform backend
├── permissions
├── actions
├── settings schema
├── sessions
├── resources/assets
├── packaging metadata
├── platform integration
├── Stuk Native renderer
└── Stuk WebView renderer
    ├── CEF host
    ├── JavaScript bridge
    ├── Rust command bridge
    ├── asset loader
    ├── dev server bridge
    ├── security policy
    └── runtime resolver
```

Required crates to add to the workspace:

```txt
stuk-webview
stuk-web-runtime
```

`stuk-webview` owns:

- CEF-backed embedded webview windows.
- JavaScript bridge support.
- Rust command bridge.
- asset loading.
- dev server integration.
- webview security policy.
- hybrid native/web surfaces.
- Stuk-owned webview host windows.
- transparent webview composition when the selected backend supports it.
- embedded CEF child-window and off-screen rendering backends.

`stuk-web-runtime` owns:

- shared CEF runtime detection.
- system runtime discovery.
- user-local runtime installation.
- app-bundled runtime fallback.
- runtime version validation.
- checksum/signature verification.
- runtime garbage collection.
- runtime diagnostics.

### UI Modes

A Stuk app may use:

```txt
Native:
  Stuk renders the full interface using its native renderer and widgets.

WebView:
  Stuk hosts a CEF webview and exposes typed Rust commands to the frontend.

Hybrid:
  Stuk combines native and web UI in one window.
```

### WebView Host Model

Stuk must own the application window for webview apps.

The CEF sample applications (`cefsimple`, `cefclient`) are acceptable only as a temporary bootstrap
fallback while the Stuk CEF host is being built. They must not be considered production chrome
because they create their own top-level windows, title bars, theme handling, cache settings, and
window behavior.

The production model is:

```txt
Stuk Window
  owns platform window, transparency, blur/material, chrome, controls, resize, drag regions

CEF Browser
  embedded as window content or rendered off-screen into Stuk's renderer
```

Stuk should support two CEF host strategies:

```txt
Embedded child backend:
  CEF browser is created with CefWindowInfo::SetAsChild(parent, rect).
  Stuk keeps the outer native window and places the browser in the content rect.
  This backend must use a Stuk-owned native window, reserve Stuk chrome/control regions before
  embedding the browser, and must not expose CEF sample chrome.
  On Linux this backend is X11 compatibility only. It must never be the default on Wayland.

Off-screen backend:
  CEF browser is created with CefWindowInfo::SetAsWindowless(kNullWindowHandle).
  CEF paints into buffers/textures that Stuk composites with native widgets and materials.
```

Off-screen rendering is the default Linux/Wayland backend. Stuk creates the only visible top-level
window, enables CEF windowless rendering in a helper process, receives BGRA paint buffers from
`CefRenderHandler::OnPaint`, uploads them as dynamic renderer textures, and composites them behind
Stuk-owned chrome, controls, overlays, and material effects. The helper CEF process must never show
its own browser chrome, titlebar, or default page.

Embedded child windows are compatibility-only. On Linux they are X11 compatibility backends and must
never be the default on Wayland. A Wayland CEF toplevel fallback may exist behind an explicit
developer override for debugging, but production Wayland webviews should use Stuk-owned off-screen
composition.

For Linux/Wayland, transparent webview composition means a single Stuk-owned top-level Wayland
surface with transparent regions that the compositor can blur behind. The browser body may be
transparent, but Stuk still has to composite the browser output into that same surface. If the
browser is a separate native child/top-level surface, compositor blur and Stuk chrome cannot be
made reliable across compositors. Therefore Wayland transparency requires the off-screen CEF
backend or a custom Stuk CEF runtime that exposes an equivalent buffer/texture handoff.

CEF windows used by Stuk must:

- set a Stuk-specific root cache path, never the CEF sample default.
- keep browser state in a user cache profile, not inside the shared runtime install.
- default to an isolated CEF root/cache per launched webview window so one app/window crash cannot
  take down other Stuk webview windows.
- handle CEF same-profile relaunches by creating or activating a Stuk window in the existing host
  process, never by showing default Chromium chrome.
- load the app URL explicitly, never fall back to Chrome/Google defaults.
- support transparent painting only on backends that can actually composite it correctly.
- expose resize and focus through the Stuk window lifecycle.
- expose Stuk-owned drag regions and window-control actions when Stuk chrome is requested.
- expose bridge command declarations to the host and cancel bridge navigation before the page leaves
  the app surface.
- keep browser chrome hidden.
- never create their own visible top-level app titlebar.

WebView transparency maps to the same material pipeline as native Stuk windows:

```rust
WebViewWindow::new()
    .entry("ui/dist/index.html")
    .glass()
    .blur_region(WindowRegion::adaptive_rounded_left(248, 16))
    .chrome(WindowChrome::Stuk)
```

When transparency is requested:

- Linux/Wayland should use the same background effect capability path as native Stuk windows when the backend supports transparent composition.
- CEF hosts must set transparent browser backgrounds and should inject a transparent document root
  default so `html`/`body` do not accidentally force an opaque page.
- Windows should use Acrylic by default, with Mica/Mica Alt override support.
- macOS should use vibrancy through the native window material layer.
- the web content background must be allowed to be transparent.
- apps must be able to gate transparent web UI on `transparent_windows` and `live_blur`.

If transparent web composition is unavailable, Stuk should fall back to a tinted opaque surface and
report the fallback through platform capabilities/diagnostics.

Native window example:

```rust
App::new()
    .id("com.example.notes")
    .window(
        NativeWindow::new()
            .title("Notes")
            .material(Material::Maris)
            .content(NotesView)
    )
    .run()
```

WebView window example:

```rust
App::new()
    .id("net.aveid.klarkey")
    .window(
        WebViewWindow::new()
            .title("Klarkey")
            .entry("ui/dist/index.html")
            .dev_server("http://localhost:5173")
            .material(Material::Maris)
            .chrome(WindowChrome::Compact)
    )
    .run()
```

Hybrid window example:

```rust
Window::new()
    .material(Material::Maris)
    .chrome(WindowChrome::Sidebar)
    .content(
        SplitView::horizontal()
            .sidebar(NativeSidebar::new())
            .main(WebView::new("ui/dist/index.html"))
    )
```

Hybrid support is important because some UI should be native and compositor-integrated, while complex content can remain web-based.

Examples:

- native sidebar + webview main content
- native titlebar/chrome + web app body
- native command palette + web app body
- native settings pane + web dashboard
- Staccato material window + CEF web content

### WebView App Structure

A webview Stuk app should use a predictable structure:

```txt
klarkey/
├── Stuk.toml
├── AGENTS.md
├── src/
│   ├── main.rs
│   ├── commands.rs
│   ├── state.rs
│   ├── services/
│   └── platforms/
│       ├── desktop.rs
│       └── web.rs
├── ui/
│   ├── package.json
│   ├── src/
│   ├── vite.config.ts
│   └── dist/
├── assets/
└── tests/
```

Rust backend logic lives in `src/`.

Web UI lives in `ui/`.

Native commands live in `src/commands.rs`.

Privileged Rust services live behind command handlers or service traits in `src/services/`.

Target-specific desktop/web behavior lives in `src/platforms/`.

The webview must never get broad native access by default.

Generated `AGENTS.md` for webview apps should include:

```txt
# Agent Instructions

- Rust backend logic lives in `src/`.
- Web UI lives in `ui/src/`.
- Native commands live in `src/commands.rs`.
- Privileged services live in `src/services/`.
- Platform-specific behavior lives in `src/platforms/`.
- Do not expose broad filesystem, shell, network, or credential APIs to the webview unless explicitly declared in `Stuk.toml`.
- Prefer typed Stuk commands over ad-hoc IPC.
- Keep privileged native APIs allowlisted.
- Desktop-only commands need a browser/web fallback or an explicit unsupported state.
- Run `stuk validate` after changes.
```

### Rust Command Bridge

Rust side:

```rust
#[stuk::command]
async fn unlock_vault(id: String, state: State<AppState>) -> Result<()> {
    state.vault.unlock(id).await
}
```

App registration:

```rust
App::new()
    .id("net.aveid.klarkey")
    .command(unlock_vault)
    .window(
        WebViewWindow::new()
            .entry("ui/dist/index.html")
            .dev_server("http://localhost:5173")
            .material(Material::Maris)
    )
    .run()
```

JavaScript side:

```ts
import { invoke, actions, settings, window } from "@stuk/web";

await invoke("unlock_vault", { id: "main" });

actions.register({
  id: "klarkey.lock",
  label: "Lock Vault",
  shortcut: "Ctrl+L",
});
```

The bridge must be:

- typed where practical
- allowlisted
- permission-aware
- origin-aware
- serializable
- debuggable
- safe by default
- target-aware
- generated into TypeScript bindings where possible
- available to desktop webviews through Stuk IPC
- replaceable by web adapters when the same UI is compiled for the browser

Current implementation checkpoint:

- `BridgeRegistry` declares allowlisted commands and can generate a minimal JavaScript facade.
- The Linux CEF host receives declared command names, installs `window.stuk.bridge`, cancels
  `stuk://bridge/...` navigation, and forwards bridge calls to Rust over a per-host IPC channel.
- `WebViewWindow::bridge_handler`, `bridge_handler_async`, and descriptor-aware variants let apps
  register Rust command handlers without writing runtime install, process, or CEF IPC code.
- Bridge descriptors support permissions, allowed origins, target metadata, params schema, and
  generated capability JSON.
- Runtime bridge dispatch enforces descriptor permissions, target availability, and origin policy
  before calling Rust handlers.
- Linux/Wayland webviews use a Stuk-owned off-screen CEF backend by default. Stuk owns the native
  window, titlebar, resize/drag behavior, background effect regions, transparent composition, and CEF
  frame presentation.
- Remaining production hardening: typed TypeScript binding generation, manifest-to-runtime command
  permission wiring, devtools tracing, event streaming, IME hardening, GPU shared-texture handoff,
  and broader platform OSR backends.

Bridge commands are the only default way for web UI to touch native Rust capabilities. Web code must
not get raw filesystem, process, credential, shell, or arbitrary OS access. A command may declare
target availability:

```rust
#[stuk::command(targets = ["desktop"], permission = "filesystem")]
async fn reveal_file(path: DocumentPath, services: Services) -> Result<()> {
    services.files.reveal(path).await
}
```

Generated web bindings should make unavailable commands explicit:

```ts
if (capabilities.commands.includes("reveal_file")) {
  await commands.revealFile({ path });
}
```

For browser builds of a webview app, `@stuk/web` resolves to a browser adapter. It can call HTTP,
WASM, local browser storage, or no-op/unsupported implementations, but it must not pretend native
desktop capabilities exist.

### WebView Security

Default webview policy must be strict.

Default:

```toml
[webview.security]
remote_content = false
devtools = "dev-only"
allow_eval = false
allow_node = false
csp = "default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'"
```

Remote content requires explicit opt-in:

```toml
[webview.security]
remote_content = true
allowed_origins = ["https://example.com"]
```

Privileged Stuk bridge APIs must not be exposed to arbitrary remote origins.

Rules:

- no Node.js-style full system access
- no arbitrary command execution
- no broad filesystem access by default
- no remote privileged UI by default
- explicit command allowlist
- explicit permissions
- strict CSP
- devtools disabled outside dev unless enabled
- bridge calls logged in dev mode
- dangerous APIs require manifest permissions

### Shared CEF Runtime

Stuk must support a shared CEF runtime so every app does not bundle its own browser engine.

The shared runtime is the browser engine installation only. Browser profiles, cache, localStorage,
cookies, and process-singleton roots are app/window state and must live in user cache directories
such as `~/.cache/stuk/webviews/<app-key>/instances/<instance-key>/`. Multiple apps and windows must
be able to run against the same installed runtime at the same time without sharing a CEF
`root_cache_path` by default. Persistent/shared profiles should be explicit opt-in app policy.

Runtime search order:

```txt
1. compatible user-local runtime for shared/user-preferred modes
2. compatible system runtime when present
3. compatible app-bundled runtime when the app opts into bundling
4. install a verified user-local runtime if policy allows
5. fail with a clear error
```

Runtime locations:

```txt
System:
  /usr/lib/stuk/cef/<version>/
  /opt/stuk/cef/<version>/ if distro chooses

User-local:
  ~/.local/share/stuk/runtimes/cef/<version>/

Bundled:
  <app>/runtimes/cef/<version>/
```

CEF package policy:

```txt
standard:
  default for Stuk's embedded host because it includes headers, CMake files, libcef_dll, resources,
  and libcef needed to build `stuk-cef-host`.

client:
  acceptable only for temporary external-process fallbacks or diagnostics.

minimal:
  not acceptable for the embedded host unless Stuk already has a prebuilt compatible host binary.
```

On Glacier/Staccato, the OS should provide:

```txt
stuk-web-runtime-cef
```

or equivalent, installed and updated through the OS update mechanism.

On normal Linux distributions, Stuk should support:

- distro package if available
- user-local runtime install without root
- app-bundled fallback for portable bundles

### Runtime Manager

`stuk-web-runtime` must provide:

```txt
runtime detection
version validation
checksum/signature verification
runtime installation
runtime removal
garbage collection
runtime diagnostics
```

CLI commands:

```sh
stuk runtime list
stuk runtime install cef
stuk runtime remove cef <version>
stuk runtime doctor
```

Example output:

```txt
CEF runtimes:
  126.0.6478.114 system /usr/lib/stuk/cef/126
  125.0.6422.60 user ~/.local/share/stuk/runtimes/cef/125
```

Runtime config:

```toml
[webview]
engine = "cef"
runtime = "shared-preferred"
min_version = "126"
runtime_index_url = "https://cef-builds.spotifycdn.com/index.json"
allow_user_runtime_install = true
allow_bundled_runtime = true
```

Runtime modes:

```txt
system-required
system-preferred
user-preferred
shared-preferred
bundled
disabled
```

Stuk must not silently install systemwide CEF. System installs must go through the OS/package manager or an explicit privileged installer flow.

User-local runtime installs are allowed with explicit user consent and verified downloads. The default install target is `~/.local/share/stuk/runtimes/cef/<version>-<package>/`; first launch may show an "Installing required dependencies" step for webview apps that allow user-local runtime installation. Apps that need zero first-launch downloads can opt into bundling the renderer, accepting the larger bundle.

Runtime indexes must be configurable. The default index may point at the upstream CEF build index,
but Stuk can point at a project-owned index, for example an R2 bucket containing Stuk-patched CEF
standard archives. Index entries may use relative archive names resolved beside the index URL or
absolute archive URLs. This lets Stuk version-track custom CEF builds without requiring app code to
know whether the runtime came from upstream CEF, an OS package, a bundled runtime, or a Stuk-hosted
runtime.

### Manifest Fields

`Stuk.toml` should support webview configuration:

```toml
[webview]
engine = "cef"
runtime = "shared-preferred"
entry = "ui/dist/index.html"
allow_user_runtime_install = true
allow_bundled_runtime = true

[webview.dev]
command = "npm run dev"
url = "http://localhost:5173"

[webview.security]
remote_content = false
devtools = "dev-only"
allow_eval = false
allow_node = false
csp = "default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'"
```

### Development Mode

`stuk dev` for webview apps should:

```txt
start Rust runtime
start configured frontend dev server
open CEF webview
hot reload frontend
recompile/relaunch backend when Rust changes
preserve window state where possible
show bridge errors clearly
show security/CSP errors clearly
show CEF runtime status clearly
```

Example:

```toml
[webview.dev]
command = "npm run dev"
url = "http://localhost:5173"
```

Production:

```toml
[webview]
entry = "ui/dist/index.html"
```

### WebView Packaging

A Stuk webview bundle includes:

- Rust app binary
- web assets
- `Stuk.toml`
- permissions metadata
- actions metadata
- settings schema
- optional bundled CEF runtime if target requires it

Preferred on Glacier:

```txt
app bundle depends on shared system CEF runtime
```

Preferred on generic Linux:

```txt
user-local shared CEF runtime or distro package
```

Portable fallback:

```txt
bundled CEF runtime
```

### CLI Additions

Add:

```sh
stuk runtime list
stuk runtime install cef
stuk runtime remove cef <version>
stuk runtime doctor
```

`stuk doctor` should include CEF runtime status when the app uses webview mode.

`stuk validate` should validate:

- CEF runtime config
- dev server config
- production entry path
- security policy
- bridge exposure
- remote origin rules
- runtime fallback policy

### Implementation Milestone: Stuk WebView + Shared CEF Runtime

Add a milestone after the native/runtime MVP:

```txt
Milestone: Stuk WebView + Shared CEF Runtime

Build:
- `stuk-webview`
- `stuk-web-runtime`
- CEF runtime detection
- user-local runtime support
- bundled runtime fallback
- webview window
- Rust command bridge
- JavaScript bridge package
- strict webview security defaults
- Vite/dev-server bridge
- `stuk runtime list`
- `stuk runtime doctor`

Done when:
A Klarkey-style app can use a CEF webview UI with a Rust backend, shared runtime detection, hot frontend reload, typed command calls, strict security defaults, and Staccato-compatible manifest metadata.

Klarkey-class native requirements:

- fixed-size palette windows with Stuk chrome.
- always-on-top and initially hidden/visible window policies.
- global shortcuts and explicit show/hide/present actions.
- strict native command registration for privileged operations.
- focus requests from the native runtime into the UI.
- transparent/material windows gated behind real platform capability checks.
- tray, deep-link, browser-extension/native-messaging, credential, and secure-storage services as
  declared app permissions rather than ad hoc side channels.
```

### Why This Exists

Stuk WebView exists because:

- WebKitGTK is not acceptable as the only Linux webview backend.
- Tauri becomes painful for apps needing reliable motion, blur, consistent runtime behavior, or non-awful Linux webview behavior.
- Electron duplicates huge runtimes and encourages sidecar/backend weirdness.
- Many real apps still benefit from web UI velocity.
- Rust backend + CEF frontend + Stuk integration is a practical bridge.
- A shared runtime gives smaller apps and centralized security updates.

### Schelf-Class Migration Target

A Tauri app like Schelf should be able to move to Stuk without rewriting its web UI first:

- keep the existing Vite/web UI as the CEF webview frontend.
- replace `@tauri-apps/api/core.invoke` with `window.stuk.bridge.invoke` or generated
  `@stuk/web` bindings.
- declare Linux package-management commands as bridge descriptors with explicit permissions such
  as `filesystem`, `package-manager`, `process`, `network`, and `clipboard`.
- keep desktop-only package operations unavailable in browser/mobile targets unless a web adapter
  provides a real fallback.
- use `WebViewWindow::glass()` plus adaptive Wayland regions for rounded window input masks and
  sidebar-only blur.
- let Stuk own runtime install, window chrome, drag regions, window controls, transparent CEF
  backgrounds, and bridge security policy.

The migration is complete when Schelf can run on Linux Wayland through Stuk WebView with the same
native package-management backend semantics, no Tauri/WebKitGTK dependency, reusable shared CEF
runtime resolution, and no app-local Wayland blur/input-region code.

Stuk Native remains the long-term fully native path.

Stuk WebView is the high-velocity practical path.

Both must coexist under one coherent runtime.
