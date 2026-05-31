use stuk_core::Element;
use stuk_style::Color;

use crate::{Badge, Button, Divider, EmptyState, Frame, HStack, Text, VStack};

#[derive(Clone, Debug)]
pub struct ColorWell {
    label: String,
    color: Color,
    action: Option<String>,
    disabled: bool,
}

impl ColorWell {
    pub fn new(label: impl Into<String>, color: Color) -> Self {
        Self {
            label: label.into(),
            color,
            action: None,
            disabled: false,
        }
    }

    pub fn action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl From<ColorWell> for Element {
    fn from(well: ColorWell) -> Self {
        let swatch = Badge::new(well.label.clone()).color(well.color);
        match well.action {
            Some(action) => HStack::new()
                .spacing(8.0)
                .child(swatch)
                .child(
                    Button::secondary(well.label)
                        .action(action)
                        .disabled(well.disabled),
                )
                .into(),
            None => swatch.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Table {
    columns: Vec<TableColumn>,
    rows: Vec<TableRow>,
    empty_title: String,
    spacing: f32,
}

impl Table {
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            empty_title: "No rows".to_string(),
            spacing: 8.0,
        }
    }

    pub fn column(mut self, label: impl Into<String>) -> Self {
        self.columns.push(TableColumn::new(label));
        self
    }

    pub fn table_column(mut self, column: TableColumn) -> Self {
        self.columns.push(column);
        self
    }

    pub fn row(mut self, row: TableRow) -> Self {
        self.rows.push(row);
        self
    }

    pub fn empty_title(mut self, title: impl Into<String>) -> Self {
        self.empty_title = title.into();
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Table> for Element {
    fn from(table: Table) -> Self {
        if table.rows.is_empty() {
            return EmptyState::new(table.empty_title).into();
        }

        let mut stack = VStack::new().spacing(table.spacing);
        if !table.columns.is_empty() {
            stack = stack.child(table_header(&table.columns));
        }
        for row in table.rows {
            stack = stack.child(table_row(row, &table.columns));
        }
        stack.into()
    }
}

#[derive(Clone, Debug)]
pub struct TableColumn {
    label: String,
    width: Option<f32>,
}

impl TableColumn {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            width: None,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width.max(1.0));
        self
    }
}

#[derive(Clone, Debug)]
pub struct TableRow {
    cells: Vec<Element>,
}

impl TableRow {
    pub fn new() -> Self {
        Self { cells: Vec::new() }
    }

    pub fn cell(mut self, cell: impl Into<Element>) -> Self {
        self.cells.push(cell.into());
        self
    }

    pub fn text_cell(self, text: impl Into<String>) -> Self {
        self.cell(Text::new(text))
    }

    pub fn numeric_cell(self, text: impl Into<String>) -> Self {
        self.cell(Text::new(text).tabular_nums())
    }
}

impl Default for TableRow {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct Tree {
    items: Vec<TreeNode>,
    empty_title: String,
    spacing: f32,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            empty_title: "No items".to_string(),
            spacing: 6.0,
        }
    }

    pub fn item(mut self, item: TreeNode) -> Self {
        self.items.push(item);
        self
    }

    pub fn node(self, node: TreeNode) -> Self {
        self.item(node)
    }

    pub fn empty_title(mut self, title: impl Into<String>) -> Self {
        self.empty_title = title.into();
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Tree> for Element {
    fn from(tree: Tree) -> Self {
        if tree.items.is_empty() {
            return EmptyState::new(tree.empty_title).into();
        }

        let mut stack = VStack::new().spacing(tree.spacing);
        for item in tree.items {
            stack = stack.child(tree_item(item, 0, tree.spacing));
        }
        stack.into()
    }
}

#[derive(Clone, Debug)]
pub struct TreeNode {
    label: String,
    action: Option<String>,
    expanded: bool,
    disabled: bool,
    children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            action: None,
            expanded: false,
            disabled: false,
            children: Vec::new(),
        }
    }

    pub fn action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn child(mut self, child: TreeNode) -> Self {
        self.children.push(child);
        self
    }
}

fn table_header(columns: &[TableColumn]) -> Element {
    let mut row = HStack::new().spacing(12.0);
    for column in columns {
        row = row.child(table_cell(
            Text::new(column.label.clone()).muted().into(),
            column,
        ));
    }
    VStack::new()
        .spacing(6.0)
        .child(row)
        .child(Divider::horizontal())
        .into()
}

fn table_row(row: TableRow, columns: &[TableColumn]) -> Element {
    let mut cells = HStack::new().spacing(12.0);
    for (index, cell) in row.cells.into_iter().enumerate() {
        match columns.get(index) {
            Some(column) => cells = cells.child(table_cell(cell, column)),
            None => cells = cells.child(cell),
        }
    }
    cells.into()
}

fn table_cell(cell: Element, column: &TableColumn) -> Element {
    match column.width {
        Some(width) => Frame::new(cell).width(width).into(),
        None => cell,
    }
}

fn tree_item(item: TreeNode, depth: usize, spacing: f32) -> Element {
    let row = tree_row(&item, depth);
    if item.children.is_empty() || !item.expanded {
        return row;
    }

    let mut stack = VStack::new().spacing(spacing).child(row);
    for child in item.children {
        stack = stack.child(tree_item(child, depth + 1, spacing));
    }
    stack.into()
}

fn tree_row(item: &TreeNode, depth: usize) -> Element {
    let marker = if item.children.is_empty() {
        ""
    } else if item.expanded {
        "v "
    } else {
        "> "
    };
    let label = format!("{}{}{}", "  ".repeat(depth), marker, item.label);
    match &item.action {
        Some(action) => Button::ghost(label)
            .action(action.clone())
            .disabled(item.disabled)
            .into(),
        None => Text::new(label).muted().into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stuk_style::NumberSpacing;

    #[test]
    fn numeric_cell_uses_tabular_numbers() {
        let row = TableRow::new().numeric_cell("1,240");
        let Element::Text(text) = row.cells.into_iter().next().unwrap() else {
            panic!("numeric cells should render as text");
        };

        assert_eq!(text.number_spacing, NumberSpacing::Tabular);
    }
}
