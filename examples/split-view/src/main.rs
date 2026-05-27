use stuk::prelude::*;

fn main() -> stuk::Result {
    App::new()
        .id("dev.stuk.split-view")
        .name("Split View")
        .window(SplitViewWindow::default())
        .run()
}

#[derive(Default)]
struct SplitViewWindow {
    refreshes: u32,
}

impl View for SplitViewWindow {
    fn view(&self, _cx: &mut Cx) -> Element {
        Window::new()
            .title("Split View")
            .material(Material::Maris)
            .chrome(WindowChrome::Compact)
            .size(980, 640)
            .content(
                SplitView::new(
                    Sidebar::new()
                        .child(Text::title("Workspace"))
                        .child(Button::primary("Open").action("workspace.open"))
                        .child(Divider::horizontal())
                        .child(
                            Tree::new().node(
                                TreeNode::new("Inbox")
                                    .action("workspace.open")
                                    .expanded(true)
                                    .child(
                                        TreeNode::new("Planning").action("workspace.open"),
                                    )
                                    .child(
                                        TreeNode::new("Archive")
                                            .action("workspace.open")
                                            .disabled(true),
                                    ),
                            ),
                        ),
                    VStack::new()
                        .padding(24.0)
                        .spacing(14.0)
                        .child(
                            Toolbar::new("Planning")
                                .child(Button::secondary("Refresh").action("workspace.refresh"))
                                .child(IconButton::new("?", "Help").action("workspace.open")),
                        )
                        .child(
                            Popover::new(
                                Text::new(format!("Refresh count: {}", self.refreshes)).muted(),
                            )
                            .title("Pane status"),
                        )
                        .child(Divider::horizontal())
                        .child(
                            ScrollView::new(
                                List::new()
                                    .spacing(10.0)
                                    .child(Text::title("Predictable layout"))
                                    .child(Text::new("The sidebar and main pane are regular composable Stuk views."))
                                    .child(Text::new("The split ratio is clamped and can be marked resizable for platform integrations.").muted())
                                    .child(Spinner::new("Waiting for file watcher events")),
                            )
                            .fill_width()
                            .height(300.0),
                        ),
                )
                .initial_ratio(0.32)
                .resizable(true),
            )
            .into()
    }

    fn actions(&self, _cx: &mut Cx) -> Vec<ActionDescriptor> {
        vec![
            ActionDescriptor::new("workspace.refresh", "Refresh").shortcut(Shortcut::new(
                Modifiers {
                    ctrl: true,
                    ..Modifiers::default()
                },
                "R",
            )),
            ActionDescriptor::new("workspace.open", "Open").shortcut(Shortcut::new(
                Modifiers {
                    ctrl: true,
                    ..Modifiers::default()
                },
                "O",
            )),
        ]
    }

    fn handle_action(&mut self, action_id: &str, _cx: &mut Cx) {
        if action_id == "workspace.refresh" {
            self.refreshes += 1;
        }
    }
}
