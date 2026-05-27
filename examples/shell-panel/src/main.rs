use stuk::prelude::*;

fn main() -> stuk::Result {
    App::new()
        .id("dev.stuk.shell-panel")
        .name("Shell Panel")
        .window(ShellPanelWindow::default())
        .run()
}

#[derive(Default)]
struct ShellPanelWindow {
    pinned: bool,
    commands: u32,
}

impl View for ShellPanelWindow {
    fn view(&self, cx: &mut Cx) -> Element {
        Window::new()
            .title("Shell Panel")
            .material(Material::Luca)
            .chrome(WindowChrome::Compact)
            .size(860, 520)
            .content(
                VStack::new()
                    .padding(22.0)
                    .spacing(14.0)
                    .child(
                        Toolbar::new("Shell Panel")
                            .child(Button::primary("Command").action("panel.command"))
                            .child(Badge::new(if self.pinned { "Pinned" } else { "Floating" }))
                            .child(Toggle::new("Pinned", self.pinned).action("panel.pin")),
                    )
                    .child(
                        HStack::new()
                            .spacing(12.0)
                            .child(
                                Popover::new(
                                    VStack::new()
                                        .spacing(8.0)
                                        .child(Text::new("Luca material request"))
                                        .child(
                                            Text::new("Falls back through platform capabilities.")
                                                .muted(),
                                        ),
                                )
                                .title("Material"),
                            )
                            .child(
                                Popover::new(Menu::new().title("Panel").actions(self.actions(cx)))
                                    .title("Actions"),
                            )
                            .child(
                                Dialog::new(
                                    "Session",
                                    VStack::new()
                                        .spacing(10.0)
                                        .child(
                                            HStack::new()
                                                .spacing(8.0)
                                                .child(Avatar::new("Panel Session", "PS"))
                                                .child(Text::new(format!(
                                                    "Command palette opened {} times.",
                                                    self.commands
                                                ))),
                                        )
                                        .child(
                                            ProgressBar::new(self.commands as f32, 8.0)
                                                .label("Command warmup"),
                                        ),
                                )
                                .action(Button::secondary("Pin").action("panel.pin"))
                                .action(Button::primary("Command").action("panel.command")),
                            ),
                    )
                    .child(
                        Toast::new(if self.commands == 0 {
                            "Panel ready"
                        } else {
                            "Command handled"
                        })
                        .message(format!(
                            "{} command requests in this session",
                            self.commands
                        ))
                        .kind(if self.commands == 0 {
                            ToastKind::Info
                        } else {
                            ToastKind::Success
                        })
                        .action(MenuItem::new("Command").action("panel.command")),
                    )
                    .child(
                        ContextMenu::new(Button::secondary("Panel Menu").action("panel.command"))
                            .menu(Menu::new().actions(self.actions(cx)))
                            .open(self.commands > 0),
                    )
                    .child(
                        CommandPalette::new()
                            .title("Commands")
                            .actions(self.actions(cx)),
                    )
                    .child(Divider::horizontal())
                    .child(
                        List::new()
                            .child(Text::new("Registers shell-facing action descriptors."))
                            .child(
                                Text::new("Declares Staccato platform preferences in Stuk.toml.")
                                    .muted(),
                            )
                            .child(
                                Text::new(
                                    "Uses compositor material semantics instead of custom blur.",
                                )
                                .muted(),
                            ),
                    ),
            )
            .into()
    }

    fn actions(&self, _cx: &mut Cx) -> Vec<ActionDescriptor> {
        vec![
            ActionDescriptor::new("panel.pin", "Pin Panel").shortcut(Shortcut::new(
                Modifiers {
                    ctrl: true,
                    ..Modifiers::default()
                },
                "P",
            )),
            ActionDescriptor::new("panel.command", "Open Command Palette").shortcut(Shortcut::new(
                Modifiers {
                    ctrl: true,
                    ..Modifiers::default()
                },
                "K",
            )),
        ]
    }

    fn handle_action(&mut self, action_id: &str, _cx: &mut Cx) {
        match action_id {
            "panel.pin" => self.pinned = !self.pinned,
            "panel.command" => self.commands += 1,
            _ => {}
        }
    }
}
