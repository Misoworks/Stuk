use stuk::prelude::*;

fn main() -> stuk::Result {
    App::new()
        .id("dev.stuk.hello")
        .name("Hello Stuk")
        .on_action(|action| println!("action: {action}"))
        .window(MainWindow::default())
        .run()
}

#[derive(Default)]
struct MainWindow {
    clicks: u32,
}

impl View for MainWindow {
    fn view(&self, cx: &mut Cx) -> stuk::Element {
        Window::new()
            .title("Hello Stuk")
            .material(Material::Maris)
            .chrome(WindowChrome::System)
            .content(
                ResizablePane::new(
                    Sidebar::new()
                        .child(Text::new("All Notes"))
                        .child(Text::new("Pinned").muted())
                        .child(
                            Toggle::new(
                                "Sync",
                                cx.setting_bool("sync.enabled").unwrap_or_default(),
                            )
                            .action("settings.sync.enabled"),
                        )
                        .child(Spacer::new())
                        .child(IconButton::new("?", "Help").action("app.help")),
                    VStack::new()
                        .padding(24.0)
                        .spacing(14.0)
                        .child(
                            Titlebar::new("Notes")
                                .subtitle("Native view tree")
                                .action(Button::primary("New").action("hello.click"))
                                .action(IconButton::new("I", "Inspect").action("stuk.inspect")),
                        )
                        .child(SearchField::new("").placeholder("Find notes"))
                        .child(Divider::horizontal())
                        .child(
                            ScrollView::new(
                                VStack::new()
                                    .spacing(10.0)
                                    .child(Text::title("Hello from Stuk"))
                                    .child(
                                        Text::new(format!(
                                            "Theme: {}",
                                            cx.setting_text("appearance.theme")
                                                .unwrap_or_else(|| "system".to_string())
                                        ))
                                        .muted(),
                                    )
                                    .child(
                                        Text::new(format!(
                                            "Button actions triggered {} times",
                                            self.clicks
                                        ))
                                        .muted(),
                                    )
                                    .child(
                                        Text::new(
                                            "MVP widgets now render through the native display list.",
                                        )
                                        .muted(),
                                    )
                                    .child(
                                        HStack::new()
                                            .spacing(10.0)
                                            .child(
                                                Overlay::new(
                                                    Button::primary("Click me")
                                                        .action("hello.click"),
                                                    Badge::new(self.clicks.to_string()),
                                                )
                                                .alignment(OverlayAlignment::TopEnd)
                                                .offset(8.0, -8.0),
                                            )
                                            .child(Button::new("Inspect").action("stuk.inspect")),
                                    ),
                            )
                            .fill_width()
                            .height(160.0),
                        ),
                )
                .initial_ratio(0.28)
                .resizable(true),
            )
            .into()
    }

    fn actions(&self, _cx: &mut Cx) -> Vec<ActionDescriptor> {
        vec![
            ActionDescriptor::new("hello.click", "Click me").shortcut(Shortcut::new(
                Modifiers {
                    ctrl: true,
                    ..Modifiers::default()
                },
                "Return",
            )),
            ActionDescriptor::new("stuk.inspect", "Inspect").shortcut(Shortcut::new(
                Modifiers {
                    ctrl: true,
                    shift: true,
                    ..Modifiers::default()
                },
                "I",
            )),
            ActionDescriptor::new("settings.sync.enabled", "Toggle sync"),
            ActionDescriptor::new("app.help", "Help")
                .shortcut(Shortcut::new(Modifiers::default(), "F1")),
        ]
    }

    fn settings(&self, _cx: &mut Cx) -> SettingsSchema {
        let mut schema = SettingsSchema::new();
        schema
            .insert(SettingDefinition::number(
                "editor.font_size",
                "Editor font size",
                15.0,
                Some(10.0),
                Some(30.0),
            ))
            .expect("settings schema should be valid");
        schema
            .insert(SettingDefinition::enumeration(
                "appearance.theme",
                "Theme",
                vec![
                    "system".to_string(),
                    "light".to_string(),
                    "dark".to_string(),
                ],
                "system",
            ))
            .expect("settings schema should be valid");
        schema
            .insert(SettingDefinition::enumeration(
                "appearance.density",
                "Density",
                vec![
                    "compact".to_string(),
                    "regular".to_string(),
                    "touch".to_string(),
                ],
                "regular",
            ))
            .expect("settings schema should be valid");
        schema
            .insert(SettingDefinition::boolean(
                "sync.enabled",
                "Enable sync",
                true,
            ))
            .expect("settings schema should be valid");
        schema
    }

    fn handle_action(&mut self, action_id: &str, _cx: &mut Cx) {
        match action_id {
            "hello.click" => self.clicks += 1,
            _ => {}
        }
    }
}
