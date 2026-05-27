use stuk::prelude::*;

fn main() -> stuk::Result {
    App::new()
        .id("dev.stuk.notes")
        .name("Notes")
        .window(NotesWindow::default())
        .run()
}

struct NotesWindow {
    drafts: u32,
    saves: u32,
    recent_notes: Resource<Vec<String>, String>,
    save_note: Mutation<String, String, String>,
}

impl Default for NotesWindow {
    fn default() -> Self {
        Self {
            drafts: 0,
            saves: 0,
            recent_notes: resource("notes.recent", || async {
                Ok::<_, String>(vec![
                    "Project outline".to_string(),
                    "Meeting notes".to_string(),
                    "Release checklist".to_string(),
                ])
            }),
            save_note: mutation("notes.save", |body: String| async move {
                Ok::<_, String>(format!("Saved {body}"))
            }),
        }
    }
}

impl View for NotesWindow {
    fn view(&self, cx: &mut Cx) -> Element {
        cx.staccato().set_tab_title("Project outline");
        cx.staccato().set_preferred_split(SplitHint::Right);
        cx.session().set_document_id("note.project-outline");
        cx.session()
            .set_restore_payload("{\"note\":\"project-outline\"}");

        let mut note_list = VirtualList::new()
            .row_height(42.0)
            .viewport_height(220.0)
            .overscan(2)
            .row(
                "project-outline",
                Button::primary("Project outline").action("notes.select"),
            )
            .row(
                "meeting-notes",
                Button::ghost("Meeting notes").action("notes.select"),
            )
            .row(
                "release-checklist",
                Button::ghost("Release checklist").action("notes.select"),
            );

        for index in 0..self.drafts {
            note_list = note_list.row(
                format!("draft-{}", index + 1),
                Button::secondary(format!("Untitled draft {}", index + 1)).action("notes.select"),
            );
        }

        Window::new()
            .title("Notes")
            .material(Material::Maris)
            .chrome(WindowChrome::Compact)
            .size(1040, 680)
            .content(
                SplitView::new(
                    Sidebar::new()
                        .child(Text::title("Notes"))
                        .child(Button::primary("New Note").action("notes.new"))
                        .child(Divider::horizontal())
                        .child(
                            Tree::new().item(
                                TreeNode::new("Workspace")
                                    .action("notes.select")
                                    .expanded(true)
                                    .child(TreeNode::new("Project").action("notes.select"))
                                    .child(TreeNode::new("Archive").action("notes.select")),
                            ),
                        )
                        .child(Divider::horizontal())
                        .child(note_list)
                        .child(Spacer::new())
                        .child(Text::new(format!("Saved {} times", self.saves)).muted()),
                    VStack::new()
                        .padding(24.0)
                        .spacing(14.0)
                        .child(
                            Toolbar::new("Project outline")
                                .child(Button::primary("Save").action("notes.save"))
                                .child(Badge::new(format!("{} saves", self.saves)))
                                .child(IconButton::new("I", "Inspect").action("stuk.inspect")),
                        )
                        .child(
                            Tabs::new()
                                .tab("overview", "Overview")
                                .tab("editor", "Editor")
                                .tab("history", "History")
                                .selected(1)
                                .action_prefix("notes.view"),
                        )
                        .child(SearchField::new("").placeholder("Find notes"))
                        .child(Divider::horizontal())
                        .child(
                            ScrollView::new(
                                VStack::new()
                                    .spacing(12.0)
                                    .child(
                                        Surface::new(
                                            HStack::new()
                                                .spacing(10.0)
                                                .child(
                                                    Svg::new("icons/note")
                                                        .decorative()
                                                        .size(22.0, 22.0)
                                                        .tint(Color::ACCENT),
                                                )
                                                .child(Avatar::new("Mira Sol", "MS"))
                                                .child(Text::title("Project outline")),
                                        )
                                        .material(Material::SurfaceElevated)
                                        .margin(2.0)
                                        .padding(16.0)
                                        .corner_radius(16.0)
                                        .border(SurfaceBorder::subtle())
                                        .shadow(SurfaceShadow::soft())
                                        .min_width(280.0)
                                        .max_width(520.0),
                                    )
                                    .child(Text::new("Write the first native Stuk notes workflow."))
                                    .child(
                                        TextArea::new("Use SplitView, Toolbar, Sidebar, and List.")
                                            .label("Body"),
                                    )
                                    .child(
                                        Table::new()
                                            .table_column(TableColumn::new("Field").width(120.0))
                                            .column("Value")
                                            .row(
                                                TableRow::new()
                                                    .text_cell("Status")
                                                    .cell(Badge::new("Draft")),
                                            )
                                            .row(
                                                TableRow::new()
                                                    .text_cell("Owner")
                                                    .text_cell("Mira Sol"),
                                            ),
                                    )
                                    .child(
                                        MutationView::new(self.save_note.clone())
                                            .idle(|| Text::new("No save yet").muted())
                                            .pending(|| Spinner::new("Saving note"))
                                            .success(|message| Badge::new(message.clone()))
                                            .error(|error| ErrorView::new(error.clone())),
                                    )
                                    .child(
                                        ResourceView::new(self.recent_notes.clone())
                                            .loading(|| Spinner::new("Loading recent notes"))
                                            .empty_when(|notes| notes.is_empty())
                                            .empty(|| EmptyState::new("No recent notes"))
                                            .error(|error| ErrorView::new(error.clone()))
                                            .data(recent_notes_list),
                                    )
                                    .child(
                                        EmptyState::new("No attachments")
                                            .message("Drop support can connect here once platform drag and drop lands."),
                                    ),
                            )
                            .fill_width()
                            .height(380.0),
                        ),
                )
                .initial_ratio(0.3)
                .resizable(true),
            )
            .into()
    }

    fn actions(&self, _cx: &mut Cx) -> Vec<ActionDescriptor> {
        vec![
            ActionDescriptor::new("notes.new", "New Note").shortcut(Shortcut::new(
                Modifiers {
                    ctrl: true,
                    ..Modifiers::default()
                },
                "N",
            )),
            ActionDescriptor::new("notes.save", "Save Note").shortcut(Shortcut::new(
                Modifiers {
                    ctrl: true,
                    ..Modifiers::default()
                },
                "S",
            )),
            ActionDescriptor::new("notes.view.overview", "Show Overview"),
            ActionDescriptor::new("notes.view.editor", "Show Editor"),
            ActionDescriptor::new("notes.view.history", "Show History"),
            ActionDescriptor::new("notes.select", "Select Note"),
            ActionDescriptor::new("stuk.inspect", "Inspect"),
        ]
    }

    fn handle_action(&mut self, action_id: &str, _cx: &mut Cx) {
        match action_id {
            "notes.new" => self.drafts += 1,
            "notes.save" => {
                self.saves += 1;
                self.save_note.submit(format!("draft {}", self.saves));
            }
            _ => {}
        }
    }
}

fn recent_notes_list(notes: &Vec<String>) -> Element {
    let mut list = List::new().spacing(6.0);

    for note in notes {
        list = list.child(Text::new(note.clone()).muted());
    }

    VStack::new()
        .spacing(8.0)
        .child(Text::new("Recent notes").muted())
        .child(list)
        .into()
}
