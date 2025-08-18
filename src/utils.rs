use std::error::Error;

use windows::Win32::UI::HiDpi::{PROCESS_PER_MONITOR_DPI_AWARE, SetProcessDpiAwareness};

pub fn set_dpi_awareness() -> Result<(), Box<dyn Error>> {
    unsafe {
        SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE)?;
    }
    Ok(())
}
