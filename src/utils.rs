use std::error::Error;

use windows::Win32::UI::HiDpi::{PROCESS_PER_MONITOR_DPI_AWARE, SetProcessDpiAwareness};

pub fn set_dpi_awareness() -> Result<(), Box<dyn Error>> {
    unsafe {
        SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE)?;
    }
    Ok(())
}

pub fn clamp(a: i32, b: i32, f: f32) -> i32 {
    return (a as f32 * f + b as f32 * (1.0 - f)).round() as i32;
}
