use stuk::prelude::*;

fn main() -> stuk::Result {
    App::new()
        .id("dev.stuk.settings")
        .name("Settings")
        .window(SettingsWindow)
        .run()
}

struct SettingsWindow;

impl View for SettingsWindow {
    fn view(&self, cx: &mut Cx) -> Element {
        let density = cx
            .setting_text("appearance.density")
            .unwrap_or_else(|| "regular".to_string());
        let controls = Grid::new(
            vec![GridTrack::fraction(1.0), GridTrack::fraction(1.0)],
            vec![GridTrack::fit(), GridTrack::fit(), GridTrack::fit()],
        )
        .gap(14.0)
        .fill_width()
        .cell(
            0,
            0,
            SegmentedControl::new("Theme")
                .option("system", "System")
                .option("light", "Light")
                .option("dark", "Dark")
                .selected(match cx.setting_text("appearance.theme").as_deref() {
                    Some("light") => 1,
                    Some("dark") => 2,
                    _ => 0,
                })
                .action_prefix("settings.appearance.theme"),
        )
        .cell(
            1,
            0,
            Slider::new(
                cx.setting_number("editor.font_size").unwrap_or(15.0) as f32,
                10.0,
                30.0,
            )
            .label("Editor font size")
            .action("settings.editor.font_size"),
        )
        .cell(
            0,
            1,
            Checkbox::new(
                "Enable sync",
                cx.setting_bool("sync.enabled").unwrap_or(false),
            )
            .action("settings.sync.enabled"),
        )
        .cell(1, 1, ProgressBar::new(3.0, 4.0).label("Settings coverage"))
        .cell(
            0,
            2,
            Dropdown::new("Density")
                .selected(density)
                .option("compact", "Compact")
                .option("regular", "Regular")
                .option("touch", "Touch")
                .action("settings.appearance.density.open")
                .action_prefix("settings.appearance.density"),
        )
        .cell(
            1,
            2,
            ColorWell::new("Accent", Color::ACCENT).action("settings.appearance.accent"),
        );

        Window::new()
            .title("Settings")
            .material(Material::Maris)
            .chrome(WindowChrome::Sidebar)
            .size(900, 620)
            .content(
                NavigationView::new(
                    "Settings",
                    ScrollView::new(
                        Form::new().row("Controls", controls).row(
                            "All settings",
                            SettingsPage::from_schema(cx.settings_schema().clone())
                                .values(cx.settings_store())
                                .action_prefix("settings")
                                .title("App Settings"),
                        ),
                    )
                    .fill_width()
                    .height(500.0),
                )
                .item(NavigationItem::new("Appearance", "settings.nav.appearance").selected(true))
                .item(NavigationItem::new("Editor", "settings.nav.editor"))
                .item(NavigationItem::new("Sync", "settings.nav.sync"))
                .footer(
                    Toggle::new(
                        "Enable sync",
                        cx.setting_bool("sync.enabled").unwrap_or(false),
                    )
                    .action("settings.sync.enabled"),
                )
                .initial_ratio(0.26)
                .resizable(true),
            )
            .into()
    }

    fn settings(&self, _cx: &mut Cx) -> SettingsSchema {
        let mut schema = SettingsSchema::new();
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
            .insert(SettingDefinition::number(
                "editor.font_size",
                "Editor font size",
                15.0,
                Some(10.0),
                Some(30.0),
            ))
            .expect("settings schema should be valid");
        schema
            .insert(SettingDefinition::boolean(
                "sync.enabled",
                "Enable sync",
                false,
            ))
            .expect("settings schema should be valid");
        schema
    }

    fn actions(&self, _cx: &mut Cx) -> Vec<ActionDescriptor> {
        vec![
            ActionDescriptor::new("settings.appearance.theme.system", "Use System Theme"),
            ActionDescriptor::new("settings.appearance.theme.light", "Use Light Theme"),
            ActionDescriptor::new("settings.appearance.theme.dark", "Use Dark Theme"),
            ActionDescriptor::new("settings.appearance.density.open", "Open Density Menu"),
            ActionDescriptor::new("settings.appearance.density.compact", "Use Compact Density"),
            ActionDescriptor::new("settings.appearance.density.regular", "Use Regular Density"),
            ActionDescriptor::new("settings.appearance.density.touch", "Use Touch Density"),
            ActionDescriptor::new("settings.appearance.accent", "Choose Accent Color"),
            ActionDescriptor::new("settings.editor.font_size", "Adjust Editor Font Size"),
            ActionDescriptor::new("settings.sync.enabled", "Toggle Sync"),
            ActionDescriptor::new("settings.nav.appearance", "Show Appearance Settings"),
            ActionDescriptor::new("settings.nav.editor", "Show Editor Settings"),
            ActionDescriptor::new("settings.nav.sync", "Show Sync Settings"),
        ]
    }
}
