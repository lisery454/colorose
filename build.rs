use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir_str = env::var("OUT_DIR").unwrap();
    let target_dir = Path::new(&out_dir_str).parent().unwrap().parent().unwrap().parent().unwrap();
    // println!("cargo:warning={:?}", target_dir);
    fs::create_dir_all(target_dir.join("resources")).unwrap();
    copy_dir_all("resources", target_dir.join("resources")).unwrap();

    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set("SubSystem", "Windows");
        res.set_icon("resources/app-icon.ico")
            .set_resource_file("resource.rc");

        res.compile().unwrap();
    }
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
