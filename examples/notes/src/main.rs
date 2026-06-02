use stuk::prelude::*;

fn main() -> stuk::Result {
    App::new()
        .id("dev.stuk.notes")
        .name("Notes")
        .window(NotesWindow::default())
        .run()
}

#[derive(Clone, Debug)]
struct Note {
    title: TextInputState,
    body: TextInputState,
    saved: bool,
}

struct NotesWindow {
    notes: Vec<Note>,
    selected: usize,
    input: TextInputManager,
}

impl Default for NotesWindow {
    fn default() -> Self {
        Self {
            notes: vec![
                Note {
                    title: TextInputState::new("Product notes"),
                    body: TextInputState::new(
                        "Keep the app monochrome, calm, and functional.\nMake editing fast enough to feel native.",
                    ),
                    saved: true,
                },
                Note {
                    title: TextInputState::new("Runtime checklist"),
                    body: TextInputState::new(
                        "Install the embedded CEF runtime locally.\nShow a native installing window while it downloads.",
                    ),
                    saved: true,
                },
                Note {
                    title: TextInputState::new("Design pass"),
                    body: TextInputState::new(
                        "Remove decorative cards.\nUse square sidebars and restrained controls.",
                    ),
                    saved: false,
                },
            ],
            selected: 0,
            input: TextInputManager::default(),
        }
    }
}

impl View for NotesWindow {
    fn view(&self, _cx: &mut Cx) -> Element {
        let note = &self.notes[self.selected];
        let title_selection = note.title.selection();
        let body_selection = note.body.selection();

        Window::new()
            .title("Notes")
            .size(920, 600)
            .glass()
            .titlebar_sidebar_blur_region(260, 38, 14)
            .content_opaque_region(260, 38)
            .rounded_window_region(14)
            .content(
                SplitView::new(
                    Sidebar::new()
                        .width(260.0)
                        .opacity(0.42)
                        .child(Text::new("Notes").size(22.0))
                        .child(
                            Frame::new(Button::primary("New").align_start().action("notes.new"))
                                .fill_width(),
                        )
                        .child(Divider::horizontal())
                        .child(self.note_list())
                        .child(Spacer::new())
                        .child(Text::new(format!("{} notes", self.notes.len())).muted()),
                    Surface::new(
                        Flex::column()
                            .padding(28.0)
                            .gap(12.0)
                            .align(FlexAlign::Stretch)
                            .fill_width()
                            .fill_height()
                            .child(
                                Grid::new(
                                    vec![
                                        GridTrack::fraction(1.0),
                                        GridTrack::fixed(78.0),
                                        GridTrack::fixed(82.0),
                                    ],
                                    vec![GridTrack::fixed(40.0)],
                                )
                                .column_gap(10.0)
                                .fill_width()
                                .cell(
                                    0,
                                    0,
                                    TextField::new(note.title.text())
                                        .placeholder("Title")
                                        .focused(self.input.is_focused("Title"))
                                        .selection(title_selection.anchor, title_selection.focus),
                                )
                                .cell(1, 0, Button::secondary("Save").action("notes.save"))
                                .cell(
                                    2,
                                    0,
                                    Button::ghost("Delete").action("notes.delete"),
                                ),
                            )
                            .flex_child(
                                FlexChildElement::new(
                                    Frame::new(
                                        TextArea::new(note.body.text())
                                            .placeholder("Body")
                                            .focused(self.input.is_focused("Body"))
                                            .selection(body_selection.anchor, body_selection.focus),
                                    )
                                    .fill_width()
                                    .fill_height(),
                                )
                                .grow(1.0),
                            ),
                    )
                    .material(Material::Solid(Color::rgb(0.09, 0.09, 0.09)))
                    .corner_radius(0.0)
                    .fill_width()
                    .fill_height(),
                )
                .initial_ratio(0.28)
                .resizable(true),
            )
            .into()
    }

    fn actions(&self, _cx: &mut Cx) -> Vec<ActionDescriptor> {
        let mut actions = vec![
            ActionDescriptor::new("notes.new", "New Note")
                .shortcut_str("Ctrl+N")
                .unwrap(),
            ActionDescriptor::new("notes.save", "Save Note")
                .shortcut_str("Ctrl+S")
                .unwrap(),
            ActionDescriptor::new("notes.delete", "Delete Note"),
        ];

        for index in 0..self.notes.len() {
            actions.push(ActionDescriptor::new(
                format!("notes.select.{index}"),
                format!("Select note {}", index + 1),
            ));
        }

        actions
    }

    fn handle_action(&mut self, action_id: &str, _cx: &mut Cx) {
        if let Some(index) = action_id
            .strip_prefix("notes.select.")
            .and_then(|value| value.parse::<usize>().ok())
        {
            if index < self.notes.len() {
                self.selected = index;
                self.input.clear_focus();
            }
            return;
        }

        match action_id {
            "notes.new" => self.new_note(),
            "notes.save" => self.current_note_mut().saved = true,
            "notes.delete" => self.delete_note(),
            _ => self.handle_input_action(action_id),
        }
    }
}

impl NotesWindow {
    fn note_list(&self) -> Element {
        let mut stack = VStack::new().spacing(8.0);

        for (index, note) in self.notes.iter().enumerate() {
            let button = if index == self.selected {
                Button::primary(note.title.text())
            } else {
                Button::ghost(note.title.text())
            }
            .align_start()
            .action(format!("notes.select.{index}"));
            stack = stack.child(Frame::new(button).fill_width());
        }

        stack.into()
    }

    fn current_note_mut(&mut self) -> &mut Note {
        &mut self.notes[self.selected]
    }

    fn new_note(&mut self) {
        self.notes.push(Note {
            title: TextInputState::new(format!("Untitled {}", self.notes.len() + 1)),
            body: TextInputState::new(""),
            saved: false,
        });
        self.selected = self.notes.len() - 1;
        let mut fields = NoteInputs {
            note: &mut self.notes[self.selected],
        };
        self.input.focus_input("Title", &mut fields);
    }

    fn delete_note(&mut self) {
        if self.notes.len() == 1 {
            self.notes[0].title.set_text("Untitled");
            self.notes[0].body.set_text("");
            self.notes[0].saved = false;
            self.selected = 0;
            return;
        }

        self.notes.remove(self.selected);
        self.selected = self.selected.min(self.notes.len() - 1);
        self.input.clear_focus();
    }

    fn handle_input_action(&mut self, action_id: &str) {
        let input = &mut self.input;
        let note = &mut self.notes[self.selected];
        let mut fields = NoteInputs { note };
        let result = input.handle_action(action_id, &mut fields);
        if result.changed {
            self.notes[self.selected].saved = false;
        }
    }
}

struct NoteInputs<'a> {
    note: &'a mut Note,
}

impl TextInputResolver for NoteInputs<'_> {
    fn input_mut(&mut self, id: &str) -> Option<&mut TextInputState> {
        match id {
            "Title" => Some(&mut self.note.title),
            "Body" => Some(&mut self.note.body),
            _ => None,
        }
    }
}
