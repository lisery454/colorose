use tray_item::{IconSource, TrayItem};

pub fn init_tray() -> TrayItem {
    let mut tray = TrayItem::new("Colorose", IconSource::Resource("app-icon")).unwrap();
    tray.add_menu_item("Exit", || {
        std::process::exit(0);
    })
    .unwrap();
    tray
}
