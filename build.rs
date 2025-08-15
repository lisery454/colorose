fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set("SubSystem", "Windows");
        res.set_icon("resources/app-icon.ico")
            .set_resource_file("resource.rc");

        res.compile().unwrap();
    }

    slint_build::compile("ui/app-window.slint").expect("Slint build failed");
}
