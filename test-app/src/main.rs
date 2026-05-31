mod actions;
mod app;
mod components;
mod settings;
mod state;
mod views;

use app::MainWindow;
use stuk::prelude::*;

fn main() -> stuk::Result {
    App::new()
        .id("dev.local.test-app")
        .name("Test App")
        .window(MainWindow::default())
        .run()
}
