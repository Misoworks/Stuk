use stuk_actions::ActionDescriptor;
use stuk_core::Element;
use stuk_style::{ButtonVariant, Color};

use crate::{Badge, Button, EmptyState, Frame, HStack, List, Overlay, Popover, Text, VStack};

#[derive(Clone, Debug)]
pub struct Form {
    rows: Vec<FormRow>,
    spacing: f32,
}

impl Form {
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            spacing: 14.0,
        }
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn row(mut self, label: impl Into<String>, field: impl Into<Element>) -> Self {
        self.rows.push(FormRow::new(label, field));
        self
    }

    pub fn form_row(mut self, row: FormRow) -> Self {
        self.rows.push(row);
        self
    }
}

impl Default for Form {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Form> for Element {
    fn from(form: Form) -> Self {
        let mut stack = VStack::new().spacing(form.spacing);
        for row in form.rows {
            stack = stack.child(row);
        }
        stack.into()
    }
}

#[derive(Clone, Debug)]
pub struct FormRow {
    label: String,
    field: Element,
    helper: Option<String>,
    error: Option<String>,
}

impl FormRow {
    pub fn new(label: impl Into<String>, field: impl Into<Element>) -> Self {
        Self {
            label: label.into(),
            field: field.into(),
            helper: None,
            error: None,
        }
    }

    pub fn helper(mut self, helper: impl Into<String>) -> Self {
        self.helper = Some(helper.into());
        self
    }

    pub fn error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }
}

impl From<FormRow> for Element {
    fn from(row: FormRow) -> Self {
        let mut stack = VStack::new()
            .spacing(6.0)
            .child(Text::new(row.label).muted())
            .child(row.field);
        if let Some(helper) = row.helper {
            stack = stack.child(Text::new(helper).muted());
        }
        if let Some(error) = row.error {
            stack = stack.child(Text::new(error).color(Color::DANGER));
        }
        stack.into()
    }
}

#[derive(Clone, Debug)]
pub struct Dropdown {
    label: String,
    selected: Option<String>,
    options: Vec<DropdownOption>,
    action: Option<String>,
    action_prefix: Option<String>,
    open: bool,
}

impl Dropdown {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            selected: None,
            options: Vec::new(),
            action: None,
            action_prefix: None,
            open: false,
        }
    }

    pub fn selected(mut self, id: impl Into<String>) -> Self {
        self.selected = Some(id.into());
        self
    }

    pub fn option(mut self, id: impl Into<String>, label: impl Into<String>) -> Self {
        self.options.push(DropdownOption::new(id, label));
        self
    }

    pub fn disabled_option(mut self, id: impl Into<String>, label: impl Into<String>) -> Self {
        self.options
            .push(DropdownOption::new(id, label).disabled(true));
        self
    }

    pub fn action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    pub fn action_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.action_prefix = Some(prefix.into());
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
}

impl From<Dropdown> for Element {
    fn from(dropdown: Dropdown) -> Self {
        let button = dropdown_button(&dropdown);
        if !dropdown.open {
            return button.into();
        }

        let menu = dropdown_menu(&dropdown);
        Overlay::new(button, Popover::new(menu).title(dropdown.label)).into()
    }
}

#[derive(Clone, Debug)]
pub struct DropdownOption {
    id: String,
    label: String,
    disabled: bool,
}

impl DropdownOption {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Clone, Debug)]
pub struct Menu {
    title: Option<String>,
    items: Vec<MenuItem>,
    empty_title: String,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            title: None,
            items: Vec::new(),
            empty_title: "No items".to_string(),
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn empty_title(mut self, title: impl Into<String>) -> Self {
        self.empty_title = title.into();
        self
    }

    pub fn item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn action(mut self, action: ActionDescriptor) -> Self {
        self.items.push(MenuItem::from(action));
        self
    }

    pub fn actions(mut self, actions: impl IntoIterator<Item = ActionDescriptor>) -> Self {
        self.items.extend(actions.into_iter().map(MenuItem::from));
        self
    }
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Menu> for Element {
    fn from(menu: Menu) -> Self {
        let mut content = VStack::new().spacing(8.0);
        if let Some(title) = menu.title {
            content = content.child(Text::new(title).muted());
        }

        if menu.items.is_empty() {
            return content.child(EmptyState::new(menu.empty_title)).into();
        }

        let mut list = List::new().spacing(6.0);
        for item in menu.items {
            list = list.child(item_button(item));
        }
        content.child(list).into()
    }
}

#[derive(Clone, Debug)]
pub struct MenuItem {
    label: String,
    action: Option<String>,
    shortcut: Option<String>,
    disabled: bool,
    variant: ButtonVariant,
}

impl MenuItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            action: None,
            shortcut: None,
            disabled: false,
            variant: ButtonVariant::Ghost,
        }
    }

    pub fn action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    pub fn shortcut(mut self, shortcut: impl ToString) -> Self {
        self.shortcut = Some(shortcut.to_string());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn destructive(mut self) -> Self {
        self.variant = ButtonVariant::Destructive;
        self
    }
}

impl From<ActionDescriptor> for MenuItem {
    fn from(action: ActionDescriptor) -> Self {
        let mut item = Self::new(action.label)
            .action(action.id)
            .disabled(!action.enabled);
        if let Some(shortcut) = action.shortcut {
            item = item.shortcut(shortcut);
        }
        item
    }
}

#[derive(Clone, Debug)]
pub struct ContextMenu {
    child: Element,
    menu: Menu,
    open: bool,
}

impl ContextMenu {
    pub fn new(child: impl Into<Element>) -> Self {
        Self {
            child: child.into(),
            menu: Menu::new(),
            open: false,
        }
    }

    pub fn menu(mut self, menu: Menu) -> Self {
        self.menu = menu;
        self
    }

    pub fn action(mut self, action: ActionDescriptor) -> Self {
        self.menu = self.menu.action(action);
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
}

impl From<ContextMenu> for Element {
    fn from(context_menu: ContextMenu) -> Self {
        if context_menu.open {
            Overlay::new(
                context_menu.child,
                Popover::new(context_menu.menu).title("Context"),
            )
            .into()
        } else {
            context_menu.child
        }
    }
}

#[derive(Clone, Debug)]
pub struct Toast {
    title: String,
    message: Option<String>,
    kind: ToastKind,
    action: Option<MenuItem>,
}

impl Toast {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: None,
            kind: ToastKind::Info,
            action: None,
        }
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn kind(mut self, kind: ToastKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn action(mut self, action: MenuItem) -> Self {
        self.action = Some(action);
        self
    }
}

impl From<Toast> for Element {
    fn from(toast: Toast) -> Self {
        let mut text = VStack::new().spacing(4.0).child(Text::new(toast.title));
        if let Some(message) = toast.message {
            text = text.child(Text::new(message).muted());
        }

        let mut row = HStack::new()
            .spacing(10.0)
            .child(Badge::new(toast.kind.label()).color(toast.kind.color()))
            .child(text);
        if let Some(action) = toast.action {
            row = row.child(item_button(action));
        }

        Frame::new(row).fill_width().into()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

impl ToastKind {
    fn label(self) -> &'static str {
        match self {
            Self::Info => "Info",
            Self::Success => "Saved",
            Self::Warning => "Notice",
            Self::Error => "Error",
        }
    }

    fn color(self) -> Color {
        match self {
            Self::Info => Color::ACCENT,
            Self::Success => Color::rgb(0.32, 0.7, 0.46),
            Self::Warning => Color::rgb(0.84, 0.64, 0.3),
            Self::Error => Color::DANGER,
        }
    }
}

fn dropdown_button(dropdown: &Dropdown) -> Button {
    let selected = dropdown
        .selected
        .as_deref()
        .and_then(|id| dropdown.options.iter().find(|option| option.id == id))
        .map(|option| option.label.as_str())
        .unwrap_or("Choose");
    let mut button = Button::secondary(format!("{}: {}", dropdown.label, selected));
    match &dropdown.action {
        Some(action) => button = button.action(action.clone()),
        None => button = button.disabled(true),
    }
    button
}

fn dropdown_menu(dropdown: &Dropdown) -> Menu {
    let mut menu = Menu::new();
    for option in &dropdown.options {
        let action = dropdown
            .action_prefix
            .as_ref()
            .map(|prefix| format!("{prefix}.{}", option.id))
            .unwrap_or_else(|| option.id.clone());
        menu = menu.item(
            MenuItem::new(option.label.clone())
                .action(action)
                .disabled(option.disabled),
        );
    }
    menu
}

fn item_button(item: MenuItem) -> Button {
    let label = match item.shortcut {
        Some(shortcut) => format!("{}  {}", item.label, shortcut),
        None => item.label,
    };
    let mut button = Button::new(label).variant(item.variant);
    match item.action {
        Some(action) if !item.disabled => button = button.action(action),
        _ => button = button.disabled(true),
    }
    button
}
