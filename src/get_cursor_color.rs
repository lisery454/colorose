use std::{error::Error, fmt::Display};

use screenshots::Screen;
use windows::Win32::{Foundation::POINT, UI::WindowsAndMessaging::GetCursorPos};

use crate::utils::clamp;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x:{:} y:{:}", self.x, self.y)
    }
}

impl Into<String> for Position {
    fn into(self) -> String {
        format!("x:{:} y:{:}", self.x, self.y)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct HSL {
    pub h: f32, // 0..=360
    pub s: f32, // 0..=1
    pub l: f32, // 0..=1
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "r:{:} g:{:} b:{:}", self.r, self.g, self.b)
    }
}

impl Into<String> for Color {
    fn into(self) -> String {
        format!("r:{:} g:{:} b:{:}", self.r, self.g, self.b)
    }
}

impl Color {
    pub fn revert(&self) -> Self {
        return Color {
            r: 255 - self.r,
            g: 255 - self.g,
            b: 255 - self.b,
        };
    }

    pub fn to_hsl(&self) -> HSL {
        // 转成 0..=1 浮点
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        // 计算亮度
        let l = (max + min) / 2.0;

        // 计算饱和度
        let s = if delta == 0.0 {
            0.0
        } else {
            delta / (1.0 - (2.0 * l - 1.0).abs())
        };

        // 计算色相
        let mut h = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };

        if h < 0.0 {
            h += 360.0;
        }

        HSL { h, s, l }
    }
}

impl Into<String> for HSL {
    fn into(self) -> String {
        format!(
            "h:{:.1} s:{:.1} l:{:.1}",
            self.h,
            self.s * 100.0,
            self.l * 100.0
        )
    }
}
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

pub fn get_pixel_color_and_tip_position(
    position: Position,
    old_tip_position: Position,
) -> Result<(Color, Position), GetCursorColorError> {
    let distance_x = (60, -250); // right, left
    let distance_y = (30, -140); // bottom, up
    let limit = 250;

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
            let image = screen
                .capture_area(
                    physical_x - physical_screen_x,
                    physical_y - physical_screen_y,
                    1,
                    1,
                )
                .or(Err(GetCursorColorError::UnableGetBuffer))?;

            let pixel = image.get_pixel(0, 0);

            let mut target_tip_position: Position;
            if physical_x > physical_screen_x + physical_screen_width - limit {
                if physical_y > physical_screen_y + physical_screen_height - limit {
                    target_tip_position = Position {
                        x: distance_x.1,
                        y: distance_y.1,
                    };
                } else {
                    target_tip_position = Position {
                        x: distance_x.1,
                        y: distance_y.0,
                    };
                }
            } else {
                if physical_y > physical_screen_y + physical_screen_height - limit {
                    target_tip_position = Position {
                        x: distance_x.0,
                        y: distance_y.1,
                    };
                } else {
                    target_tip_position = Position {
                        x: distance_x.0,
                        y: distance_y.0,
                    };
                }
            }

            target_tip_position.x += physical_x;
            target_tip_position.y += physical_y;

            let mut current_tip_position = Position {
                x: old_tip_position.x,
                y: old_tip_position.y,
            };

            current_tip_position.x = clamp(old_tip_position.x, target_tip_position.x, 0.7);
            current_tip_position.y = clamp(old_tip_position.y, target_tip_position.y, 0.7);

            return Ok((
                Color {
                    r: pixel.0[0],
                    g: pixel.0[1],
                    b: pixel.0[2],
                },
                current_tip_position,
            ));
        }
    }

    Err(GetCursorColorError::ScreenCountIsZero)
}
