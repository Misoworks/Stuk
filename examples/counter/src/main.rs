use stuk::prelude::*;

fn main() -> stuk::Result {
    App::new()
        .id("dev.stuk.counter")
        .name("Counter")
        .window(CounterWindow::default())
        .run()
}

#[derive(Default)]
struct CounterWindow {
    count: i32,
}

impl View for CounterWindow {
    fn view(&self, _cx: &mut Cx) -> Element {
        Window::new()
            .title("Counter")
            .material(Material::Maris)
            .chrome(WindowChrome::System)
            .size(760, 480)
            .content(
                VStack::new()
                    .padding(28.0)
                    .spacing(16.0)
                    .child(Text::title("Counter"))
                    .child(Text::new(format!("Current value: {}", self.count)).size(20.0))
                    .child(
                        Flex::row()
                            .gap(10.0)
                            .wrap(FlexWrap::Wrap)
                            .child(Button::primary("Increment").action("counter.increment"))
                            .child(Button::secondary("Decrement").action("counter.decrement"))
                            .child(Button::ghost("Reset").action("counter.reset")),
                    )
                    .child(Divider::horizontal())
                    .child(EmptyState::new("Actions keep behavior explicit").message(
                        "Buttons, shortcuts, and command surfaces all target the same action IDs.",
                    )),
            )
            .into()
    }

    fn actions(&self, _cx: &mut Cx) -> Vec<ActionDescriptor> {
        actions! {
            increment {
                id: "counter.increment",
                label: "Increment",
                shortcut: "Ctrl+ArrowUp",
                category: "Counter",
            }
            decrement {
                id: "counter.decrement",
                label: "Decrement",
                shortcut: "Ctrl+ArrowDown",
                category: "Counter",
            }
            reset {
                id: "counter.reset",
                label: "Reset",
                shortcut: "Ctrl+Backspace",
                category: "Counter",
            }
        }
    }

    fn handle_action(&mut self, action_id: &str, _cx: &mut Cx) {
        match action_id {
            "counter.increment" => self.count += 1,
            "counter.decrement" => self.count -= 1,
            "counter.reset" => self.count = 0,
            _ => {}
        }
    }
}
