use std::{error::Error, fmt::Display};

use screenshots::Screen;
use windows::Win32::{Foundation::POINT, UI::WindowsAndMessaging::GetCursorPos};

use crate::model::{color::Color, position::Position};

#[derive(Debug)]
pub enum GetCursorColorError {
    UnableGetMousePosition,
    UnableGetScreens,
    UnableGetBuffer,
    ScreenCountIsZero,
}

impl Display for GetCursorColorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetCursorColorError::UnableGetMousePosition => {
                write!(f, "unable to get mouse position")
            }
            GetCursorColorError::UnableGetScreens => write!(f, "unable to get screens"),
            GetCursorColorError::UnableGetBuffer => write!(f, "unable to get screen buffer"),
            GetCursorColorError::ScreenCountIsZero => write!(f, "screen count is zero"),
        }
    }
}

impl Error for GetCursorColorError {}

pub fn get_mouse_position() -> Result<Position, GetCursorColorError> {
    unsafe {
        let mut point = POINT::default();
        match GetCursorPos(&mut point) {
            Ok(_) => {
                // println!("Mouse position: ({}, {})", point.x, point.y);
                Ok(Position {
                    x: point.x,
                    y: point.y,
                })
            }
            Err(_) => Err(GetCursorColorError::UnableGetMousePosition),
        }
    }
}

pub struct ScreenData {
    pub cursor_pixel_color: Color,
    pub screen_pixel_colors: Vec<Color>,
}
pub fn get_screen_data(
    position: Position,
    screen_tex_size: usize,
    screen_sample_size: usize,
) -> Result<ScreenData, GetCursorColorError> {
    let screens = Screen::all().or(Err(GetCursorColorError::UnableGetScreens))?;
    for screen in screens {
        let scale = screen.display_info.scale_factor;
        let physical_x = position.x;
        let physical_y = position.y;

        let physical_screen_x = (screen.display_info.x as f32 * scale).round() as i32;
        let physical_screen_y = (screen.display_info.y as f32 * scale).round() as i32;
        let physical_screen_width = (screen.display_info.width as f32 * scale).round() as i32;
        let physical_screen_height = (screen.display_info.height as f32 * scale).round() as i32;

        if physical_screen_x <= physical_x
            && physical_x < physical_screen_x + physical_screen_width
            && physical_screen_y <= physical_y
            && physical_y < physical_screen_y + physical_screen_height
        {
            let half_size = (screen_tex_size as f32 / 2.0).floor() as u32;
            let half_sample_size = (screen_sample_size as f32 / 2.0).floor() as i32;
            let image = screen
                .capture_area(
                    physical_x - physical_screen_x - half_size as i32,
                    physical_y - physical_screen_y - half_size as i32,
                    half_size * 2 + 1,
                    half_size * 2 + 1,
                )
                .or(Err(GetCursorColorError::UnableGetBuffer))?;

            let mut sample_colors = vec![];
            for dx in -half_sample_size..=half_sample_size {
                for dy in -half_sample_size..=half_sample_size {
                    let x = (half_size as i32 + dx) as u32;
                    let y = (half_size as i32 + dy) as u32;

                    let pixel = image.get_pixel(x, y);
                    sample_colors.push(Color {
                        r: pixel.0[0],
                        g: pixel.0[1],
                        b: pixel.0[2],
                    });
                }
            }
            let len = sample_colors.len() as f32;
            let color = Color {
                r: (sample_colors.iter().map(|c| c.r as u32).sum::<u32>() as f32 / len) as u8,
                g: (sample_colors.iter().map(|c| c.g as u32).sum::<u32>() as f32 / len) as u8,
                b: (sample_colors.iter().map(|c| c.b as u32).sum::<u32>() as f32 / len) as u8,
            };

            let (width, height) = image.dimensions();
            let mut colors = vec![];
            for y in 0..height {
                for x in 0..width {
                    let pixel = image.get_pixel(x, y);
                    let color = Color::new(pixel[0], pixel[1], pixel[2]);
                    colors.push(color);
                }
            }

            return Ok(ScreenData {
                cursor_pixel_color: color,
                screen_pixel_colors: colors,
            });
        }
    }

    Err(GetCursorColorError::ScreenCountIsZero)
}
