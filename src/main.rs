#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod app;
mod color;
mod cursor_color;
mod position;
mod tray;
mod utils;

use std::error::Error;

// use crate::app::App;

// fn main() -> Result<(), Box<dyn Error>> {
//     App::run()?;
//     Ok(())
// }

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let main_window = MainWindow::new()?;

    main_window.run()
}
