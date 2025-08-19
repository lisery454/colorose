use std::sync::{Arc, Mutex};

use crate::model::{color::Color, position::Position, wheel_mode::WheelMode};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct AppState {
    pub position: Position,
    pub color: Color,
    pub screen_colors: Vec<Color>,
    
    pub screen_tex_size: usize,
    pub screen_sample_size: usize,
    pub wheel_mode: WheelMode,
}

impl AppState {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(AppState {
            screen_tex_size: 21,
            screen_sample_size: 1,
            wheel_mode: WheelMode::HSV,
            ..Default::default()
        }))
    }
}
