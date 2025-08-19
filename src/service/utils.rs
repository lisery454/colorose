use std::{error::Error, path::Path, sync::Arc};

use egui::IconData;
use image::ImageReader;
use windows::Win32::{
    Foundation::HWND,
    Graphics::Dwm::{DWMWINDOWATTRIBUTE, DwmSetWindowAttribute},
    UI::HiDpi::{PROCESS_PER_MONITOR_DPI_AWARE, SetProcessDpiAwareness},
};

pub fn set_dpi_awareness() -> Result<(), Box<dyn Error>> {
    unsafe {
        SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE)?;
    }
    Ok(())
}

pub fn enable_acrylic_effect(hwnd: HWND) -> windows::core::Result<()> {
    unsafe {
        const DWMWA_SYSTEMBACKDROP_TYPE: DWMWINDOWATTRIBUTE = DWMWINDOWATTRIBUTE(38);
        const DWMSBT_ACRYLIC: u32 = 2; // 亚克力效果

        DwmSetWindowAttribute(
            hwnd,
            DWMWA_SYSTEMBACKDROP_TYPE,
            &DWMSBT_ACRYLIC as *const _ as *const _,
            std::mem::size_of_val(&DWMSBT_ACRYLIC) as u32,
        )?;
    }

    Ok(())
}

pub fn load_icon_data(path: impl AsRef<Path>) -> Option<Arc<IconData>> {
    let img = ImageReader::open(path).ok()?.decode().ok()?;
    let rgba = img.to_rgba8();

    let width = rgba.width();
    let height = rgba.height();
    Some(Arc::new(IconData {
        rgba: rgba.into_raw(),
        width: width as _,
        height: height as _,
    }))
}
