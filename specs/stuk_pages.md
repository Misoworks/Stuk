# Stuk Pages, Components, Resources, and Pagination Addendum

## 0. Purpose

This document extends the main `stuk.md` specification with app architecture rules for:

- pages/screens
- routing/navigation
- reusable components
- component props
- slots/composition
- state/actions
- resources
- pagination
- virtual lists
- tables/data grids
- generated code structure
- agent-friendly organization

The goal is to make Stuk apps stay readable as they grow.

Stuk must not allow apps to devolve into one giant `main.rs` or a single 4,000-line view file.

Stuk should make clean architecture the default path.

---

## 1. Core Architecture Model

A Stuk app should be organized around these concepts:

```txt
Page / Screen
  A top-level UI state or route inside a window.

Component
  A reusable UI building block.

Action
  A typed description of user intent or app command.

State
  The app's source of truth.

Resource
  Async-loaded data and its loading/error/refresh state.

Service
  External logic such as database, API, sync, crypto, storage.

Route
  The current app location or navigation state.

PaginatedResource
  A resource that loads data page-by-page or cursor-by-cursor.

VirtualList / DataTable
  Efficient presentation for large collections.
```

The clean mental model:

```txt
Views describe UI.
Components compose UI.
Actions describe user intent.
State owns app truth.
Resources own async loading.
Services talk to external systems.
Routes describe where the user is.
```

Example flow:

```txt
User clicks button
→ component emits Action::DeleteNote(id)
→ AppState::update handles action
→ service/resource mutates data
→ state/resource changes
→ view updates
```

No random global mutable state. No giant view files. No action logic hidden in deeply nested widgets.

---

## 2. Recommended App Structure

Default Stuk native app structure:

```txt
my-app/
├── Stuk.toml
├── AGENTS.md
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── state.rs
│   ├── actions.rs
│   ├── routes.rs
│   ├── models/
│   │   ├── note.rs
│   │   └── user.rs
│   ├── services/
│   │   ├── db.rs
│   │   ├── api.rs
│   │   └── sync.rs
│   ├── resources/
│   │   ├── notes.rs
│   │   └── account.rs
│   ├── views/
│   │   ├── main_window.rs
│   │   ├── notes_page.rs
│   │   ├── settings_page.rs
│   │   └── onboarding_page.rs
│   ├── components/
│   │   ├── app_sidebar.rs
│   │   ├── note_list.rs
│   │   ├── note_row.rs
│   │   ├── toolbar.rs
│   │   └── empty_state.rs
│   └── ui/
│       ├── theme.rs
│       ├── icons.rs
│       └── layout.rs
├── assets/
└── tests/
```

For small apps, this can be simpler:

```txt
src/
├── main.rs
├── app.rs
├── state.rs
├── actions.rs
├── views/
└── components/
```

For larger components, folder-based organization is allowed:

```txt
components/
└── note_list/
    ├── mod.rs
    ├── note_list.rs
    ├── note_row.rs
    ├── empty.rs
    └── loading.rs
```

Stuk templates should start simple, but support growing into this structure.

---

## 3. WebView App Structure

For Stuk WebView apps:

```txt
notes-app/
├── Stuk.toml
├── AGENTS.md
├── src/
│   ├── main.rs
│   ├── commands.rs
│   ├── state.rs
│   ├── services/
│   │   ├── storage.rs
│   │   └── sync.rs
│   └── permissions.rs
├── ui/
│   ├── package.json
│   ├── vite.config.ts
│   └── src/
│       ├── main.ts
│       ├── app.ts
│       ├── routes.ts
│       ├── actions.ts
│       ├── state/
│       ├── resources/
│       ├── views/
│       ├── components/
│       └── lib/
│           └── stuk.ts
├── assets/
└── tests/
```

Rules:

- Rust backend logic lives in `src/`.
- Native commands live in `src/commands.rs`.
- Privileged services live in `src/services/`.
- Web UI lives in `ui/src/`.
- Web components live in `ui/src/components/`.
- Web pages/screens live in `ui/src/views/`.
- Shared action IDs should be declared in `Stuk.toml`.
- Runtime install, native window chrome, drag regions, window controls, and webview hosting belong
  to Stuk, not the app.
- Shared CEF runtime files are reusable across apps; browser profile/cache paths are isolated and
  managed by Stuk so concurrent windows do not fight over CEF process-singleton state.
- WebView apps using Stuk chrome should only provide the visible content structure for titlebars and
  controls; drag-region forwarding and native window commands are framework behavior.
- The webview must not get broad native access by default.

---

## 4. Pages / Screens

Stuk should have a first-class page/screen concept.

A page is a top-level view inside a window.

Pages are used for:

- settings pages
- onboarding steps
- document views
- vault views
- account screens
- admin screens
- detail screens
- app sections

Conceptual trait:

```rust
pub trait Page {
    fn id(&self) -> PageId;
    fn title(&self) -> String;
    fn view(&self, cx: &mut Cx) -> impl IntoView;
}
```

Example:

```rust
pub struct NotesPage;

impl View for NotesPage {
    fn view(&self, cx: &mut Cx) -> impl IntoView {
        view! {
            VStack {
                Toolbar(title: "Notes") {
                    Button.primary("New", action: Action::NewNote)
                }

                NotesList()
            }
        }
    }
}
```

A window should compose pages instead of containing all UI inline.

Bad:

```rust
impl View for MainWindow {
    fn view(&self, cx: &mut Cx) -> impl IntoView {
        view! {
            // 900 lines of nested UI
        }
    }
}
```

Good:

```rust
impl View for MainWindow {
    fn view(&self, cx: &mut Cx) -> impl IntoView {
        view! {
            AppShell {
                slot sidebar { AppSidebar() }
                slot toolbar { MainToolbar() }
                slot content { RouteView(route: state.route) }
            }
        }
    }
}
```

---

## 5. Routing and Navigation

Stuk should support typed app routing.

Desktop apps do not need browser URL routing by default, but they do need structured navigation.

Example route enum:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    Home,
    Notes,
    Note(NoteId),
    Settings,
    SettingsAppearance,
    Account,
    Onboarding,
}
```

State:

```rust
pub struct AppState {
    pub route: Route,
    pub selected_note: Option<NoteId>,
}
```

Action:

```rust
pub enum Action {
    Navigate(Route),
    SelectNote(NoteId),
}
```

Update:

```rust
impl AppState {
    pub fn update(&mut self, action: Action, cx: &mut Cx) {
        match action {
            Action::Navigate(route) => self.route = route,
            Action::SelectNote(id) => {
                self.selected_note = Some(id);
                self.route = Route::Note(id);
            }
        }
    }
}
```

Route view:

```rust
pub struct RouteView {
    pub route: Route,
}

impl View for RouteView {
    fn view(&self, cx: &mut Cx) -> impl IntoView {
        match self.route {
            Route::Home => view! { HomePage() },
            Route::Notes => view! { NotesPage() },
            Route::Note(id) => view! { NotePage(id: id) },
            Route::Settings => view! { SettingsPage() },
            Route::SettingsAppearance => view! { AppearanceSettingsPage() },
            Route::Account => view! { AccountPage() },
            Route::Onboarding => view! { OnboardingPage() },
        }
    }
}
```

Stuk should provide optional navigation helpers:

```rust
NavigationStack::new()
NavigationSplitView::new()
NavigationSidebar::new()
RouteView::new(route)
```

But apps should be able to use plain Rust enums and pattern matching.

---

## 6. Components

Components are reusable UI pieces.

They should:

- receive explicit props
- render UI
- emit actions/events
- avoid hidden global mutations
- avoid owning async data unless they are explicitly container components
- expose accessibility labels/roles where needed
- use semantic Stuk widgets/materials

Example component:

```rust
pub struct NoteRow {
    pub note: NoteSummary,
    pub selected: bool,
}

impl View for NoteRow {
    fn view(&self, cx: &mut Cx) -> impl IntoView {
        view! {
            HStack(spacing: 10, padding: 10) {
                Icon("note")
                VStack {
                    Text(&self.note.title).style(TextStyle::BodyStrong)
                    Text(&self.note.preview).style(TextStyle::Muted)
                }
            }
            .selected(self.selected)
            .on_click(Action::SelectNote(self.note.id))
        }
    }
}
```

Good:

```rust
.on_click(Action::SelectNote(note.id))
```

Bad:

```rust
.on_click(|| GLOBAL_STATE.lock().unwrap().selected_note = Some(note.id))
```

Components should keep behavior discoverable.

---

## 7. Props

Components should have explicit props.

Conceptual prop struct:

```rust
#[derive(Props)]
pub struct NoteRowProps {
    pub note: NoteSummary,
    pub selected: bool,
    pub on_select: Action,
}
```

Usage:

```rust
NoteRow {
    note,
    selected: note.id == state.selected_note,
    on_select: Action::SelectNote(note.id),
}
```

Even if exact syntax changes, the design rule is:

```txt
component inputs must be explicit
component outputs must be explicit
```

This helps humans, agents, tests, previews, and refactors.

---

## 8. Slots and Composition

Stuk should support slot-based composition.

Slot composition avoids inheritance/widget-subclass soup.

Builder example:

```rust
AppShell::new()
    .sidebar(AppSidebar::new())
    .toolbar(MainToolbar::new())
    .content(RouteView::new(state.route))
```

Macro example:

```rust
view! {
    AppShell {
        slot sidebar { AppSidebar() }
        slot toolbar { MainToolbar() }
        slot content { RouteView(route: state.route) }
    }
}
```

Useful slot-based components:

```txt
AppShell
NavigationSplitView
SidebarLayout
ToolbarLayout
SettingsPage
Card
Dialog
Popover
Table
DataTable
```

Slots should be named where structure matters.

---

## 9. Smart vs Presentational Components

Stuk docs and templates should teach this distinction.

### Presentational component

Receives data and emits actions.

Example:

```rust
NoteRow {
    note,
    selected,
    on_select: Action::SelectNote(note.id),
}
```

Presentational components should not fetch data.

### Smart/container component

Connects to resources/state and passes data down.

Example:

```rust
pub struct NotesPage;

impl View for NotesPage {
    fn view(&self, cx: &mut Cx) -> impl IntoView {
        let notes = cx.resource::<NotesResource>();

        view! {
            ResourceView(notes) {
                loading { Spinner("Loading notes") }
                empty { EmptyState("No notes yet") }
                error(err) { ErrorView(err) }
                data(items) { NoteList(notes: items) }
            }
        }
    }
}
```

Recommended split:

```txt
Page owns resources.
List renders collections.
Row renders one item.
```

This keeps components reusable.

---

## 10. Actions

Components should emit actions for meaningful behavior.

Examples:

```rust
Button::primary("Save").action(Action::Save)
```

```rust
TextField::new()
    .value(state.search)
    .on_change(Action::SearchChanged)
```

Actions should describe user intent:

```rust
pub enum Action {
    NewNote,
    DeleteNote(NoteId),
    SelectNote(NoteId),
    SearchChanged(String),
    Save,
    Navigate(Route),
    LoadMoreNotes,
}
```

Avoid embedding important app logic deep inside inline closures.

Small local closures are acceptable for local UI state, but app-level mutations should go through actions.

---

## 11. Resources

Resources represent async data.

A resource owns:

- loading state
- loaded data
- error state
- refresh state
- stale state
- retry behavior
- cancellation behavior

Basic resource example:

```rust
let account = resource("account.current", || async {
    api.current_account().await
});
```

Resource view:

```rust
ResourceView::new(account)
    .loading(|| Spinner::new("Loading account"))
    .empty(|| EmptyState::new("No account"))
    .error(|err| ErrorView::new(err))
    .data(|account| AccountView::new(account))
```

Macro target:

```rust
view! {
    Resource(account) {
        loading { Spinner("Loading account") }
        empty { EmptyState("No account") }
        error(err) { ErrorView(err) }
        data(account) { AccountView(account) }
    }
}
```

Resources should be separate from presentational components.

---

## 12. Pagination

Pagination should be a first-class data/resource pattern, not just a widget.

Stuk should support:

```txt
cursor pagination
offset pagination
page-number pagination
infinite scrolling
manual "Load more"
refresh
virtualized rendering
```

### 12.1 Page Type

Conceptual type:

```rust
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<PageCursor>,
    pub total: Option<u64>,
}
```

Pagination modes:

```rust
pub enum PaginationMode {
    Cursor,
    Offset,
    PageNumber,
}
```

### 12.2 PaginatedResource

Conceptual API:

```rust
let notes = PaginatedResource::cursor("notes.list", |cursor| async move {
    notes_api.list(ListNotes {
        after: cursor,
        limit: 50,
    }).await
});
```

It should expose:

```rust
notes.items()
notes.is_loading()
notes.is_loading_next_page()
notes.is_refreshing()
notes.error()
notes.has_next_page()
notes.load_next()
notes.refresh()
notes.reset()
```

### 12.3 Resource States

Paginated resources must model:

```txt
initial_loading
loaded
empty
loading_more
refreshing
error_initial
error_next_page
end_reached
stale
```

This prevents every app from reinventing broken loading states.

### 12.4 PaginatedList

UI helper:

```rust
PaginatedList::new(notes)
    .row(|note| NoteRow::new(note))
    .empty(|| EmptyState::new("No notes"))
    .error(|err| ErrorView::new(err))
    .loading_more(|| Spinner::small())
```

Macro target:

```rust
view! {
    PaginatedList(resource: notes) {
        loading { Spinner("Loading notes") }
        empty { EmptyState("No notes yet") }
        error(err) { ErrorView(err) }
        row(note) { NoteRow(note) }
        load_more { Button("Load more", action: Action::LoadMoreNotes) }
    }
}
```

### 12.5 Infinite Scroll

For load-as-user-scrolls:

```rust
VirtualList::new(notes.items())
    .row(|note| NoteRow::new(note))
    .on_near_end(Action::LoadMoreNotes)
```

Update:

```rust
Action::LoadMoreNotes => {
    self.notes.load_next(cx);
}
```

### 12.6 Classic Page Controls

For admin/table UIs:

```rust
PaginationControls::new()
    .current_page(state.users.page)
    .page_size(state.users.page_size)
    .total(state.users.total)
    .on_page_change(Action::UsersPageChanged)
```

Classic pagination should support:

- first page
- previous page
- next page
- last page
- page size
- total count
- loading state
- disabled states

---

## 13. Virtual Lists

Large lists must not render all rows.

Stuk should provide:

```txt
VirtualList
VirtualGrid
VirtualTable
```

VirtualList requirements:

- render only visible rows
- support variable row height eventually
- support keyboard navigation
- support selection
- support accessibility
- support smooth scrolling
- support scroll-to-index
- support sticky headers eventually
- support pagination/infinite loading

Example:

```rust
VirtualList::new(notes.items())
    .key(|note| note.id)
    .row(|note| NoteRow::new(note))
    .estimated_row_height(48)
    .on_select(Action::SelectNote)
    .on_near_end(Action::LoadMoreNotes)
```

---

## 14. Tables and Data Grids

Serious apps need good tables.

Stuk should eventually include:

```txt
Table
VirtualTable
DataGrid
PaginatedTable
```

Features:

- columns
- sorting
- filtering
- pagination
- selection
- row actions
- keyboard navigation
- sticky header
- virtual rows
- resizable columns later
- column visibility later
- export/copy later

Example:

```rust
DataTable::new(users)
    .column("Name", |u| Text(&u.name))
    .column("Email", |u| Text(&u.email))
    .column("Status", |u| Badge::new(&u.status))
    .sort_by(state.sort)
    .pagination(state.pagination)
    .on_sort(Action::UsersSortChanged)
    .on_page(Action::UsersPageChanged)
```

`PaginatedTable` should integrate with `PaginatedResource`.

Example:

```rust
PaginatedTable::new(users_resource)
    .columns(user_columns())
    .empty(|| EmptyState::new("No users"))
    .error(|err| ErrorView::new(err))
    .on_sort(Action::UsersSortChanged)
```

Tables are important for:

- settings
- admin apps
- audit logs
- vault items
- device lists
- account sessions
- developer tools
- package/app management

Native toolkits often make tables feel old. Stuk should make them feel modern.

---

## 15. CLI Generators

Stuk should include generators to preserve clean structure.

Commands:

```sh
stuk generate page Settings
stuk generate component NoteRow
stuk generate resource Notes
stuk generate action notes.new
```

Aliases:

```sh
stuk g page Settings
stuk g component NoteRow
stuk g resource Notes
stuk g action notes.new
```

Generated page example:

```rust
use stuk::prelude::*;

pub struct SettingsPage;

impl View for SettingsPage {
    fn view(&self, cx: &mut Cx) -> impl IntoView {
        view! {
            SettingsPage {
                Section("General") {
                    Text("Settings")
                }
            }
        }
    }
}
```

Generated component example:

```rust
use stuk::prelude::*;

pub struct NoteRow;

impl View for NoteRow {
    fn view(&self, cx: &mut Cx) -> impl IntoView {
        view! {
            HStack {
                Text("Note row")
            }
        }
    }
}
```

Generators should update module exports where appropriate.

Generated files should follow project conventions.

---

## 16. Templates

Stuk should provide templates.

Required templates:

```txt
app-basic
app-sidebar
app-settings
app-document
app-webview
app-hybrid
component-library
```

Example:

```sh
stuk new notes --template app-sidebar
```

Generated structure:

```txt
src/
├── main.rs
├── app.rs
├── state.rs
├── actions.rs
├── routes.rs
├── resources/
│   └── notes.rs
├── views/
│   ├── main_window.rs
│   ├── notes_page.rs
│   └── settings_page.rs
└── components/
    ├── app_sidebar.rs
    ├── note_list.rs
    └── note_row.rs
```

Templates should include:

- `AGENTS.md`
- basic routes
- basic actions
- example components
- example settings schema where relevant
- example resource where relevant
- `stuk validate` compatibility

---

## 17. File Naming Conventions

Recommended conventions:

```txt
views/settings_page.rs
views/main_window.rs
views/onboarding_page.rs

components/note_row.rs
components/note_list.rs
components/app_sidebar.rs

resources/notes.rs
resources/account.rs

services/db.rs
services/api.rs

models/note.rs
models/user.rs
```

Avoid enterprise-style nonsense:

```txt
SettingsPageViewControllerFactoryImplFinal.rs
```

File names should be boring and predictable.

Agents perform better with boring names.

Humans do too.

---

## 18. Validation Rules

`stuk validate` should detect codebase structure problems where possible.

Warnings:

- giant view file above configured line threshold
- component file with too many unrelated components
- action declared in code but missing manifest metadata where required
- manifest action unused
- route declared but unreachable
- component preview missing for exported component, optional warning
- unlabeled icon button
- resource without error state handling
- paginated list without loading-more handling
- virtual list missing stable key
- table rows missing stable key
- webview command exposed without permission metadata

Example diagnostic:

```json
{
  "level": "warning",
  "path": "src/views/main_window.rs",
  "message": "MainWindow view is 742 lines. Consider extracting pages/components.",
  "fix_hint": "Move route content into `src/views/*_page.rs` and reusable UI into `src/components/`."
}
```

---

## 19. Component Previews

Every reusable component should be previewable.

Example:

```rust
preview! {
    NoteRowPreview => view! {
        NoteRow {
            note: fake_note(),
            selected: false,
            on_select: Action::Noop,
        }
    }

    SelectedNoteRowPreview => view! {
        NoteRow {
            note: fake_note(),
            selected: true,
            on_select: Action::Noop,
        }
    }
}
```

`stuk preview` should show:

- app components
- built-in Stuk components
- theme variants
- density variants
- light/dark mode
- accessibility tree
- layout boxes

This gives native development the component isolation that web dev gets from Storybook-like workflows.

---

## 20. Testing Components

Stuk should support component tests.

Example:

```rust
#[test]
fn note_row_emits_select_action() {
    let note = fake_note();

    let mut test = ComponentTest::new(NoteRow {
        note: note.clone(),
        selected: false,
        on_select: Action::SelectNote(note.id),
    });

    test.click();
    test.expect_action(Action::SelectNote(note.id));
}
```

Testing should support:

- render component
- query by role/label
- click
- type text
- press keys
- inspect emitted actions
- inspect accessibility nodes

This keeps components reliable and agent edits safer.

---

## 21. Summary

Stuk should make clean app architecture the default.

Core rules:

```txt
Page = screen
Component = reusable UI
Action = user intent
State = app truth
Resource = async data
Service = external logic
Route = current location
PaginatedResource = page/cursor loading
VirtualList/DataTable = efficient collection display
```

The framework should provide:

- file conventions
- templates
- generators
- resource primitives
- pagination primitives
- virtual lists
- data tables
- component previews
- validation
- agent-friendly project structure

The goal is simple:

```txt
A Stuk app should stay readable after it becomes real.
```
