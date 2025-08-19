#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod app;
mod model;
mod service;
mod ui;

use crate::app::App;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    App::run()?;
    Ok(())
}
